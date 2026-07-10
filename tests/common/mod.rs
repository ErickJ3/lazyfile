use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use lazyfile::rclone::RcloneClient;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub const TEST_BASE_DIR: &str = "/tmp/lazyfile_integration_test";
pub const TEST_REMOTE: &str = "lazyfile_test";

pub async fn setup_test_remote(client: &RcloneClient) {
    if let Ok(remotes) = client.list_remotes().await {
        if remotes.contains(&TEST_REMOTE.to_string()) {
            if client.list_files(TEST_REMOTE, "").await.is_ok() {
                return;
            }
            let _ = client.delete_remote(TEST_REMOTE).await;
        }
    }

    let mut params = HashMap::new();
    params.insert("remote".to_string(), "/tmp".to_string());
    client
        .create_remote(TEST_REMOTE, "alias", params)
        .await
        .expect("Failed to create test remote");
}

pub fn get_test_dir(test_name: &str) -> PathBuf {
    let path = PathBuf::from(TEST_BASE_DIR).join(test_name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("Failed to create test directory");
    path
}

pub fn get_remote_path(test_name: &str) -> String {
    format!("lazyfile_integration_test/{}", test_name)
}

pub fn cleanup_test_dir(test_name: &str) {
    let path = PathBuf::from(TEST_BASE_DIR).join(test_name);
    let _ = fs::remove_dir_all(path);
}

pub fn create_test_client() -> RcloneClient {
    RcloneClient::new("localhost", 5572).expect("default reqwest client config is valid")
}

pub fn unique_remote_name(prefix: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{}_{}", prefix, timestamp)
}

pub fn create_key_event(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}
