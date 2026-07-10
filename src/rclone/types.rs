//! Data types for rclone API responses.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Response from rclone `config/listremotes` call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRemotesResponse {
    /// Configured remote names. None when no remotes exist
    /// (rclone marshals an empty Go slice as null).
    pub remotes: Option<Vec<String>>,
}

/// Response from rclone `config/dump` call
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDumpResponse {
    #[serde(flatten)]
    pub remotes: HashMap<String, HashMap<String, String>>,
}

/// Parameters for rclone `config/create` call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigCreateRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub remote_type: String,
    #[serde(default)]
    pub parameters: HashMap<String, String>,
}

/// Parameters for rclone `config/update` call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigUpdateRequest {
    pub name: String,
    #[serde(default)]
    pub parameters: HashMap<String, String>,
}

/// Parameters for rclone `config/delete` call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDeleteRequest {
    pub name: String,
}

/// Parameters for rclone `operations/mkdir` call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MkdirRequest {
    pub fs: String,
    pub remote: String,
}

/// Parameters for rclone `operations/deletefile` call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteFileRequest {
    pub fs: String,
    pub remote: String,
}

/// Parameters for rclone `operations/purge` call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurgeRequest {
    pub fs: String,
    pub remote: String,
}

/// Parameters for rclone `operations/copyfile` call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyFileRequest {
    #[serde(rename = "srcFs")]
    pub src_fs: String,
    #[serde(rename = "srcRemote")]
    pub src_remote: String,
    #[serde(rename = "dstFs")]
    pub dst_fs: String,
    #[serde(rename = "dstRemote")]
    pub dst_remote: String,
}

/// Parameters for rclone `operations/movefile` call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveFileRequest {
    #[serde(rename = "srcFs")]
    pub src_fs: String,
    #[serde(rename = "srcRemote")]
    pub src_remote: String,
    #[serde(rename = "dstFs")]
    pub dst_fs: String,
    #[serde(rename = "dstRemote")]
    pub dst_remote: String,
}

/// Parameters for rclone `sync/copy` call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncCopyRequest {
    #[serde(rename = "srcFs")]
    pub src_fs: String,
    #[serde(rename = "dstFs")]
    pub dst_fs: String,
}

/// Represents a file or directory from rclone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileItem {
    /// File or directory name.
    #[serde(rename = "Name")]
    pub name: String,
    /// File size in bytes (0 for directories).
    #[serde(rename = "Size")]
    pub size: i64,
    /// Last modification time.
    #[serde(rename = "ModTime")]
    pub mod_time: String,
    /// True if this is a directory.
    #[serde(rename = "IsDir")]
    pub is_dir: bool,
}

/// Response from rclone `operations/list` call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFilesResponse {
    /// List of files and directories, None if empty.
    pub list: Option<Vec<FileItem>>,
}

impl FileItem {
    /// Returns the display name of the item.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns whether this item is a directory.
    pub fn is_dir(&self) -> bool {
        self.is_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_item_properties() {
        let item = FileItem {
            name: "test.txt".to_string(),
            size: 1024,
            mod_time: "2024-01-01T00:00:00Z".to_string(),
            is_dir: false,
        };

        assert_eq!(item.name, "test.txt");
        assert_eq!(item.size, 1024);
        assert!(!item.is_dir);
    }

    #[test]
    fn test_file_item_directory() {
        let item = FileItem {
            name: "folder".to_string(),
            size: 0,
            mod_time: "2024-01-01T00:00:00Z".to_string(),
            is_dir: true,
        };

        assert!(item.is_dir);
        assert_eq!(item.size, 0);
    }

    #[test]
    fn test_file_item_name_method() {
        let item = FileItem {
            name: "myfile.txt".to_string(),
            size: 100,
            mod_time: "2024-01-01T00:00:00Z".to_string(),
            is_dir: false,
        };

        assert_eq!(item.name(), "myfile.txt");
    }

    #[test]
    fn test_file_item_is_dir_method() {
        let dir_item = FileItem {
            name: "folder".to_string(),
            size: 0,
            mod_time: "2024-01-01T00:00:00Z".to_string(),
            is_dir: true,
        };

        let file_item = FileItem {
            name: "file.txt".to_string(),
            size: 100,
            mod_time: "2024-01-01T00:00:00Z".to_string(),
            is_dir: false,
        };

        assert!(dir_item.is_dir());
        assert!(!file_item.is_dir());
    }

    #[test]
    fn test_list_files_response_empty() {
        let response = ListFilesResponse { list: None };
        assert!(response.list.is_none());
    }

    #[test]
    fn test_list_files_response_with_files() {
        let items = vec![FileItem {
            name: "file1.txt".to_string(),
            size: 100,
            mod_time: "2024-01-01T00:00:00Z".to_string(),
            is_dir: false,
        }];

        let response = ListFilesResponse { list: Some(items) };

        assert!(response.list.is_some());
        assert_eq!(response.list.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_config_create_request() {
        let mut params = HashMap::new();
        params.insert("path".to_string(), "/bucket".to_string());

        let req = ConfigCreateRequest {
            name: "myremote".to_string(),
            remote_type: "s3".to_string(),
            parameters: params,
        };

        assert_eq!(req.name, "myremote");
        assert_eq!(req.remote_type, "s3");
        assert_eq!(req.parameters.get("path").unwrap(), "/bucket");
    }

    #[test]
    fn test_config_delete_request() {
        let req = ConfigDeleteRequest {
            name: "myremote".to_string(),
        };

        assert_eq!(req.name, "myremote");
    }

    #[test]
    fn test_config_update_request() {
        let mut params = HashMap::new();
        params.insert("path".to_string(), "/newpath".to_string());

        let req = ConfigUpdateRequest {
            name: "myremote".to_string(),
            parameters: params,
        };

        assert_eq!(req.name, "myremote");
        assert_eq!(req.parameters.get("path").unwrap(), "/newpath");
    }
}
