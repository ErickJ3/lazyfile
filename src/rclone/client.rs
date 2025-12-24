//! Rclone JSON-RPC client implementation.

use crate::error::{LazyFileError, Result};
use crate::rclone::types::{
    ConfigCreateRequest, ConfigDeleteRequest, ConfigUpdateRequest, CopyFileRequest,
    DeleteFileRequest, FileItem, MkdirRequest, MoveFileRequest, PurgeRequest,
    SyncCopyRequest,
};
use reqwest::Client;
use std::collections::HashMap;
use tracing::{debug, error, trace};

/// HTTP client for communicating with rclone rc daemon.
#[derive(Debug)]
pub struct RcloneClient {
    base_url: String,
    client: Client,
}

impl RcloneClient {
    /// Create a new RcloneClient.
    ///
    /// # Arguments
    /// * `host` - Host address of rclone daemon (e.g., "localhost")
    /// * `port` - Port number of rclone daemon (e.g., 5572)
    pub fn new(host: &str, port: u16) -> Self {
        let base_url = format!("http://{}:{}", host, port);
        trace!("Creating RcloneClient with base URL: {}", base_url);
        Self {
            base_url,
            client: Client::new(),
        }
    }

    /// List all configured remotes.
    ///
    /// # Errors
    /// Returns error if rclone daemon is unreachable or responds with error.
    pub async fn list_remotes(&self) -> Result<Vec<String>> {
        debug!("Listing remotes");
        let url = format!("{}/config/listremotes", self.base_url);

        let response = self.client.post(&url).send().await?;

        if !response.status().is_success() {
            error!("Failed to list remotes: {}", response.status());
            return Err(LazyFileError::RcloneApi(format!(
                "Failed to list remotes: {}",
                response.status()
            )));
        }

        let body = response.text().await?;
        trace!("Response body: {}", body);
        let json: serde_json::Value = serde_json::from_str(&body)?;

        if let Some(remotes) = json.get("remotes")
            && let Ok(remotes_vec) = serde_json::from_value::<Vec<String>>(remotes.clone())
        {
            debug!("Found {} remotes", remotes_vec.len());
            return Ok(remotes_vec);
        }

        error!("Unexpected response format from rclone");
        Err(LazyFileError::RcloneApi(
            "Unexpected response format from rclone".to_string(),
        ))
    }

    /// List files in a remote path.
    ///
    /// # Arguments
    /// * `remote` - Name of the remote (e.g., "gdrive")
    /// * `path` - Path within the remote (empty string for root)
    ///
    /// # Errors
    /// Returns error if rclone daemon is unreachable or responds with error.
    pub async fn list_files(&self, remote: &str, path: &str) -> Result<Vec<FileItem>> {
        let fs = format!("{}:", remote);
        let remote_path = path.trim_start_matches('/');

        debug!("Listing files in {}:{}", remote, remote_path);
        let url = format!("{}/operations/list", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "fs": fs,
                "remote": remote_path
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            error!("Failed to list files: {}", response.status());
            return Err(LazyFileError::RcloneApi(format!(
                "Failed to list files: {}",
                response.status()
            )));
        }

        let body = response.text().await?;
        trace!("Response body: {}", body);
        let json: serde_json::Value = serde_json::from_str(&body)?;

        if let Ok(list_response) = serde_json::from_value::<super::types::ListFilesResponse>(json) {
            let items = list_response.list.unwrap_or_default();
            debug!("Found {} items", items.len());
            return Ok(items);
        }

        debug!("No items found in response");
        Ok(Vec::new())
    }

    /// Create a new remote configuration.
    ///
    /// # Arguments
    /// * `name` - Name of the remote
    /// * `remote_type` - Type of remote (e.g., "local", "drive", "s3")
    /// * `parameters` - Configuration parameters for the remote
    ///
    /// # Errors
    /// Returns error if rclone daemon is unreachable or responds with error.
    pub async fn create_remote(
        &self,
        name: &str,
        remote_type: &str,
        parameters: HashMap<String, String>,
    ) -> Result<()> {
        debug!("Creating remote: {} (type: {})", name, remote_type);
        let url = format!("{}/config/create", self.base_url);
        let request = ConfigCreateRequest {
            name: name.to_string(),
            remote_type: remote_type.to_string(),
            parameters,
        };

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            error!("Failed to create remote: {}", response.status());
            let body = response.text().await?;
            return Err(LazyFileError::RcloneApi(format!(
                "Failed to create remote: {}",
                body
            )));
        }

        debug!("Remote '{}' created successfully", name);
        Ok(())
    }

    /// Update an existing remote configuration.
    ///
    /// # Arguments
    /// * `name` - Name of the remote to update
    /// * `parameters` - Configuration parameters to update
    ///
    /// # Errors
    /// Returns error if rclone daemon is unreachable or responds with error.
    pub async fn update_remote(
        &self,
        name: &str,
        parameters: HashMap<String, String>,
    ) -> Result<()> {
        debug!("Updating remote: {}", name);
        let url = format!("{}/config/update", self.base_url);
        let request = ConfigUpdateRequest {
            name: name.to_string(),
            parameters,
        };

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            error!("Failed to update remote: {}", response.status());
            let body = response.text().await?;
            return Err(LazyFileError::RcloneApi(format!(
                "Failed to update remote: {}",
                body
            )));
        }

        debug!("Remote '{}' updated successfully", name);
        Ok(())
    }

    /// Delete a remote configuration.
    ///
    /// # Arguments
    /// * `name` - Name of the remote to delete
    ///
    /// # Errors
    /// Returns error if rclone daemon is unreachable or responds with error.
    pub async fn delete_remote(&self, name: &str) -> Result<()> {
        debug!("Deleting remote: {}", name);
        let url = format!("{}/config/delete", self.base_url);
        let request = ConfigDeleteRequest {
            name: name.to_string(),
        };

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            error!("Failed to delete remote: {}", response.status());
            let body = response.text().await?;
            return Err(LazyFileError::RcloneApi(format!(
                "Failed to delete remote: {}",
                body
            )));
        }

        debug!("Remote '{}' deleted successfully", name);
        Ok(())
    }

    /// Create a new directory in a remote.
    ///
    /// # Arguments
    /// * `remote` - Remote name (e.g., "gdrive")
    /// * `path` - Path to create the directory at
    ///
    /// # Errors
    /// Returns error if rclone daemon is unreachable or responds with error.
    pub async fn mkdir(&self, remote: &str, path: &str) -> Result<()> {
        let fs = format!("{}:", remote);
        let remote_path = path.trim_start_matches('/');
        debug!("Creating directory at {}:{}", remote, remote_path);
        let url = format!("{}/operations/mkdir", self.base_url);
        let request = MkdirRequest {
            fs,
            remote: remote_path.to_string(),
        };

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            error!("Failed to create directory: {}", response.status());
            let body = response.text().await?;
            return Err(LazyFileError::RcloneApi(format!(
                "Failed to create directory: {}",
                body
            )));
        }

        debug!("Directory created successfully at {}:{}", remote, remote_path);
        Ok(())
    }

    /// Delete a file from a remote.
    ///
    /// # Arguments
    /// * `remote` - Remote name (e.g., "gdrive")
    /// * `path` - Path to the file to delete
    ///
    /// # Errors
    /// Returns error if rclone daemon is unreachable or responds with error.
    pub async fn delete_file(&self, remote: &str, path: &str) -> Result<()> {
        let fs = format!("{}:", remote);
        let remote_path = path.trim_start_matches('/');
        debug!("Deleting file: {}:{}", remote, remote_path);
        let url = format!("{}/operations/deletefile", self.base_url);
        let request = DeleteFileRequest {
            fs,
            remote: remote_path.to_string(),
        };

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            error!("Failed to delete file: {}", response.status());
            let body = response.text().await?;
            return Err(LazyFileError::RcloneApi(format!(
                "Failed to delete file: {}",
                body
            )));
        }

        debug!("File deleted successfully: {}/{}", remote, path);
        Ok(())
    }

    /// Delete a directory and its contents from a remote.
    ///
    /// # Arguments
    /// * `remote` - Remote name (e.g., "gdrive")
    /// * `path` - Path to the directory to delete
    ///
    /// # Errors
    /// Returns error if rclone daemon is unreachable or responds with error.
    pub async fn purge(&self, remote: &str, path: &str) -> Result<()> {
        let fs = format!("{}:", remote);
        let remote_path = path.trim_start_matches('/');
        debug!("Purging directory: {}:{}", remote, remote_path);
        let url = format!("{}/operations/purge", self.base_url);
        let request = PurgeRequest {
            fs,
            remote: remote_path.to_string(),
        };

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            error!("Failed to purge directory: {}", response.status());
            let body = response.text().await?;
            return Err(LazyFileError::RcloneApi(format!(
                "Failed to purge directory: {}",
                body
            )));
        }

        debug!("Directory purged successfully at {}:{}", remote, remote_path);
        Ok(())
    }

    /// Copy a file between remotes or within the same remote.
    ///
    /// # Arguments
    /// * `src_remote` - Source remote name
    /// * `src_path` - Source file path
    /// * `dst_remote` - Destination remote name
    /// * `dst_path` - Destination file path
    ///
    /// # Errors
    /// Returns error if rclone daemon is unreachable or responds with error.
    pub async fn copy_file(
        &self,
        src_remote: &str,
        src_path: &str,
        dst_remote: &str,
        dst_path: &str,
    ) -> Result<()> {
        let src_path_clean = src_path.trim_start_matches('/');
        let dst_path_clean = dst_path.trim_start_matches('/');
        debug!(
            "Copying file from {}:{} to {}:{}",
            src_remote, src_path_clean, dst_remote, dst_path_clean
        );
        let url = format!("{}/operations/copyfile", self.base_url);
        let request = CopyFileRequest {
            src_fs: format!("{}:", src_remote),
            src_remote: src_path_clean.to_string(),
            dst_fs: format!("{}:", dst_remote),
            dst_remote: dst_path_clean.to_string(),
        };

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            error!("Failed to copy file: {}", response.status());
            let body = response.text().await?;
            return Err(LazyFileError::RcloneApi(format!(
                "Failed to copy file: {}",
                body
            )));
        }

        debug!("File copied successfully");
        Ok(())
    }

    /// Move a file between remotes or within the same remote.
    ///
    /// # Arguments
    /// * `src_remote` - Source remote name
    /// * `src_path` - Source file path
    /// * `dst_remote` - Destination remote name
    /// * `dst_path` - Destination file path
    ///
    /// # Errors
    /// Returns error if rclone daemon is unreachable or responds with error.
    pub async fn move_file(
        &self,
        src_remote: &str,
        src_path: &str,
        dst_remote: &str,
        dst_path: &str,
    ) -> Result<()> {
        let src_path_clean = src_path.trim_start_matches('/');
        let dst_path_clean = dst_path.trim_start_matches('/');
        debug!(
            "Moving file from {}:{} to {}:{}",
            src_remote, src_path_clean, dst_remote, dst_path_clean
        );
        let url = format!("{}/operations/movefile", self.base_url);
        let request = MoveFileRequest {
            src_fs: format!("{}:", src_remote),
            src_remote: src_path_clean.to_string(),
            dst_fs: format!("{}:", dst_remote),
            dst_remote: dst_path_clean.to_string(),
        };

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            error!("Failed to move file: {}", response.status());
            let body = response.text().await?;
            return Err(LazyFileError::RcloneApi(format!(
                "Failed to move file: {}",
                body
            )));
        }

        debug!("File moved successfully");
        Ok(())
    }

    /// Sync/copy a directory between remotes.
    ///
    /// # Arguments
    /// * `src_remote` - Source remote name
    /// * `dst_remote` - Destination remote name
    ///
    /// # Errors
    /// Returns error if rclone daemon is unreachable or responds with error.
    pub async fn sync_copy(&self, src_remote: &str, dst_remote: &str) -> Result<()> {
        debug!(
            "Syncing/copying from {}:/ to {}:/",
            src_remote, dst_remote
        );
        let url = format!("{}/sync/copy", self.base_url);
        let request = SyncCopyRequest {
            src_fs: format!("{}:", src_remote),
            dst_fs: format!("{}:", dst_remote),
        };

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            error!("Failed to sync copy: {}", response.status());
            let body = response.text().await?;
            return Err(LazyFileError::RcloneApi(format!(
                "Failed to sync copy: {}",
                body
            )));
        }

        debug!("Sync copy completed successfully");
        Ok(())
    }
}
