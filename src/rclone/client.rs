//! Rclone JSON-RPC client implementation.

use crate::error::{LazyFileError, Result};
use crate::rclone::commands;
use crate::rclone::types::{
    ConfigCreateRequest, ConfigDeleteRequest, ConfigUpdateRequest, CopyFileRequest,
    DeleteFileRequest, FileItem, ListFilesResponse, ListRemotesResponse, MkdirRequest,
    MoveFileRequest, PurgeRequest, SyncCopyRequest,
};
use reqwest::Client;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, error, info, trace, warn};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// HTTP client for communicating with rclone rc daemon.
#[derive(Debug)]
pub struct RcloneClient {
    base_url: String,
    client: Client,
}

impl RcloneClient {
    /// Creates a new RcloneClient.
    ///
    /// # Arguments
    /// * `host` - Host address of rclone daemon (e.g., "localhost")
    /// * `port` - Port number of rclone daemon (e.g., 5572)
    ///
    /// # Errors
    /// Returns error if the HTTP client cannot be constructed, e.g.
    /// when the system TLS or DNS resolver configuration fails to
    /// load.
    pub fn new(host: &str, port: u16) -> Result<Self> {
        let base_url = format!("http://{}:{}", host, port);
        trace!(base_url = %base_url, "creating RcloneClient");
        let client = Client::builder().timeout(REQUEST_TIMEOUT).build()?;
        Ok(Self { base_url, client })
    }

    /// Sends a POST request with a JSON body, returning the
    /// response text on success.
    async fn post_json<B: Serialize>(&self, endpoint: &'static str, body: &B) -> Result<String> {
        let url = format!("{}/{}", self.base_url, endpoint);
        trace!(endpoint, "POST request");

        let response = self
            .client
            .post(&url)
            .json(body)
            .send()
            .await
            .map_err(|e| LazyFileError::RcloneApi {
                endpoint,
                message: e.to_string(),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!(endpoint, %status, "request failed");
            return Err(LazyFileError::RcloneApi {
                endpoint,
                message: format!("{}: {}", status, body),
            });
        }

        response.text().await.map_err(|e| LazyFileError::RcloneApi {
            endpoint,
            message: e.to_string(),
        })
    }

    /// Sends a POST command and discards the response body.
    async fn post_command<B: Serialize>(&self, endpoint: &'static str, body: &B) -> Result<()> {
        self.post_json(endpoint, body).await?;
        Ok(())
    }

    /// Lists all configured remotes.
    ///
    /// # Errors
    /// Returns error if rclone daemon is unreachable or responds
    /// with an error.
    pub async fn list_remotes(&self) -> Result<Vec<String>> {
        debug!("listing remotes");
        let body = self
            .post_json(commands::LIST_REMOTES, &serde_json::json!({}))
            .await?;
        trace!(body = %body, "list_remotes response");

        let remotes = parse_list_remotes(&body)
            .inspect_err(|e| warn!(error = %e, "malformed list_remotes response"))?;
        info!(count = remotes.len(), "loaded remotes");
        Ok(remotes)
    }

    /// Lists files in a remote path.
    ///
    /// # Errors
    /// Returns error if rclone daemon is unreachable or responds
    /// with an error.
    pub async fn list_files(&self, remote: &str, path: &str) -> Result<Vec<FileItem>> {
        let fs = format!("{}:", remote);
        let remote_path = path.trim_start_matches('/');
        debug!(remote, path = remote_path, "listing files");

        let body = self
            .post_json(
                commands::LIST_FILES,
                &serde_json::json!({ "fs": fs, "remote": remote_path }),
            )
            .await?;
        trace!(body = %body, "list_files response");

        let items = parse_list_files(&body)
            .inspect_err(|e| warn!(error = %e, "malformed list_files response"))?;
        info!(count = items.len(), "loaded files");
        Ok(items)
    }

    /// Creates a new remote configuration.
    ///
    /// # Errors
    /// Returns error if rclone daemon is unreachable or responds
    /// with an error.
    pub async fn create_remote(
        &self,
        name: &str,
        remote_type: &str,
        parameters: HashMap<String, String>,
    ) -> Result<()> {
        debug!(remote = name, r#type = remote_type, "creating remote");
        let request = ConfigCreateRequest {
            name: name.to_string(),
            remote_type: remote_type.to_string(),
            parameters,
        };
        self.post_command(commands::CONFIG_CREATE, &request).await?;
        info!(remote = name, "remote created");
        Ok(())
    }

    /// Updates an existing remote configuration.
    ///
    /// # Errors
    /// Returns error if rclone daemon is unreachable or responds
    /// with an error.
    pub async fn update_remote(
        &self,
        name: &str,
        parameters: HashMap<String, String>,
    ) -> Result<()> {
        debug!(remote = name, "updating remote");
        let request = ConfigUpdateRequest {
            name: name.to_string(),
            parameters,
        };
        self.post_command(commands::CONFIG_UPDATE, &request).await?;
        info!(remote = name, "remote updated");
        Ok(())
    }

    /// Deletes a remote configuration.
    ///
    /// # Errors
    /// Returns error if rclone daemon is unreachable or responds
    /// with an error.
    pub async fn delete_remote(&self, name: &str) -> Result<()> {
        debug!(remote = name, "deleting remote");
        let request = ConfigDeleteRequest {
            name: name.to_string(),
        };
        self.post_command(commands::CONFIG_DELETE, &request).await?;
        info!(remote = name, "remote deleted");
        Ok(())
    }

    /// Creates a new directory in a remote.
    ///
    /// # Errors
    /// Returns error if rclone daemon is unreachable or responds
    /// with an error.
    pub async fn mkdir(&self, remote: &str, path: &str) -> Result<()> {
        let fs = format!("{}:", remote);
        let remote_path = path.trim_start_matches('/');
        debug!(remote, path = remote_path, "creating directory");
        let request = MkdirRequest {
            fs,
            remote: remote_path.to_string(),
        };
        self.post_command(commands::MKDIR, &request).await?;
        info!(remote, path = remote_path, "directory created");
        Ok(())
    }

    /// Deletes a file from a remote.
    ///
    /// # Errors
    /// Returns error if rclone daemon is unreachable or responds
    /// with an error.
    pub async fn delete_file(&self, remote: &str, path: &str) -> Result<()> {
        let fs = format!("{}:", remote);
        let remote_path = path.trim_start_matches('/');
        debug!(remote, path = remote_path, "deleting file");
        let request = DeleteFileRequest {
            fs,
            remote: remote_path.to_string(),
        };
        self.post_command(commands::DELETE_FILE, &request).await?;
        info!(remote, path = remote_path, "file deleted");
        Ok(())
    }

    /// Deletes a directory and its contents from a remote.
    ///
    /// # Errors
    /// Returns error if rclone daemon is unreachable or responds
    /// with an error.
    pub async fn purge(&self, remote: &str, path: &str) -> Result<()> {
        let fs = format!("{}:", remote);
        let remote_path = path.trim_start_matches('/');
        debug!(remote, path = remote_path, "purging directory");
        let request = PurgeRequest {
            fs,
            remote: remote_path.to_string(),
        };
        self.post_command(commands::PURGE, &request).await?;
        info!(remote, path = remote_path, "directory purged");
        Ok(())
    }

    /// Copies a file between remotes or within the same remote.
    ///
    /// # Errors
    /// Returns error if rclone daemon is unreachable or responds
    /// with an error.
    pub async fn copy_file(
        &self,
        src_remote: &str,
        src_path: &str,
        dst_remote: &str,
        dst_path: &str,
    ) -> Result<()> {
        let src = src_path.trim_start_matches('/');
        let dst = dst_path.trim_start_matches('/');
        debug!(
            src_remote,
            src_path = src,
            dst_remote,
            dst_path = dst,
            "copying file"
        );
        let request = CopyFileRequest {
            src_fs: format!("{}:", src_remote),
            src_remote: src.to_string(),
            dst_fs: format!("{}:", dst_remote),
            dst_remote: dst.to_string(),
        };
        self.post_command(commands::COPY_FILE, &request).await?;
        info!("file copied");
        Ok(())
    }

    /// Moves a file between remotes or within the same remote.
    ///
    /// # Errors
    /// Returns error if rclone daemon is unreachable or responds
    /// with an error.
    pub async fn move_file(
        &self,
        src_remote: &str,
        src_path: &str,
        dst_remote: &str,
        dst_path: &str,
    ) -> Result<()> {
        let src = src_path.trim_start_matches('/');
        let dst = dst_path.trim_start_matches('/');
        debug!(
            src_remote,
            src_path = src,
            dst_remote,
            dst_path = dst,
            "moving file"
        );
        let request = MoveFileRequest {
            src_fs: format!("{}:", src_remote),
            src_remote: src.to_string(),
            dst_fs: format!("{}:", dst_remote),
            dst_remote: dst.to_string(),
        };
        self.post_command(commands::MOVE_FILE, &request).await?;
        info!("file moved");
        Ok(())
    }

    /// Syncs/copies a directory between remotes.
    ///
    /// # Errors
    /// Returns error if rclone daemon is unreachable or responds
    /// with an error.
    pub async fn sync_copy(&self, src_remote: &str, dst_remote: &str) -> Result<()> {
        debug!(src_remote, dst_remote, "syncing/copying");
        let request = SyncCopyRequest {
            src_fs: format!("{}:", src_remote),
            dst_fs: format!("{}:", dst_remote),
        };
        self.post_command(commands::SYNC_COPY, &request).await?;
        info!("sync copy completed");
        Ok(())
    }
}

/// Parses a `config/listremotes` response body into remote names.
///
/// A missing or `null` `remotes` field means no remotes are
/// configured; anything else that does not match the response shape
/// is an error.
fn parse_list_remotes(body: &str) -> Result<Vec<String>> {
    let resp: ListRemotesResponse =
        serde_json::from_str(body).map_err(|e| LazyFileError::RcloneApi {
            endpoint: commands::LIST_REMOTES,
            message: format!("unexpected response format: {}", e),
        })?;
    Ok(resp.remotes.unwrap_or_default())
}

/// Parses an `operations/list` response body into file items.
///
/// A missing or `null` `list` field is a valid empty directory;
/// anything that does not match the response shape is an error.
fn parse_list_files(body: &str) -> Result<Vec<FileItem>> {
    let resp: ListFilesResponse =
        serde_json::from_str(body).map_err(|e| LazyFileError::RcloneApi {
            endpoint: commands::LIST_FILES,
            message: format!("unexpected response format: {}", e),
        })?;
    Ok(resp.list.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_file_list() {
        let body = r#"{"list":[{"Name":"a.txt","Size":10,"ModTime":"2024-01-01T00:00:00Z","IsDir":false}]}"#;
        let items = parse_list_files(body).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name(), "a.txt");
    }

    #[test]
    fn treats_null_list_as_empty() {
        let items = parse_list_files(r#"{"list":null}"#).unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn treats_missing_list_as_empty() {
        let items = parse_list_files("{}").unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn rejects_malformed_body() {
        let err = parse_list_files("not json").unwrap_err();
        assert!(matches!(
            err,
            LazyFileError::RcloneApi {
                endpoint: commands::LIST_FILES,
                ..
            }
        ));
    }

    #[test]
    fn rejects_wrong_item_shape() {
        let err = parse_list_files(r#"{"list":[{"unexpected":true}]}"#).unwrap_err();
        assert!(matches!(err, LazyFileError::RcloneApi { .. }));
    }

    #[test]
    fn parses_remote_names() {
        let remotes = parse_list_remotes(r#"{"remotes":["gdrive","s3"]}"#).unwrap();
        assert_eq!(remotes, vec!["gdrive".to_string(), "s3".to_string()]);
    }

    #[test]
    fn treats_null_remotes_as_empty() {
        let remotes = parse_list_remotes(r#"{"remotes":null}"#).unwrap();
        assert!(remotes.is_empty());
    }

    #[test]
    fn treats_missing_remotes_as_empty() {
        let remotes = parse_list_remotes("{}").unwrap();
        assert!(remotes.is_empty());
    }

    #[test]
    fn rejects_malformed_remotes_body() {
        let err = parse_list_remotes(r#"{"remotes":"oops"}"#).unwrap_err();
        assert!(matches!(
            err,
            LazyFileError::RcloneApi {
                endpoint: commands::LIST_REMOTES,
                ..
            }
        ));
    }
}
