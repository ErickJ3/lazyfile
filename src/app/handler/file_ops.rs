//! File operation handling (delete, mkdir, copy, move).

use super::Handler;
use crate::app::state::{ActiveModal, App};
use crate::error::Result;
use crate::ui::FileOperationsModal;
use crossterm::event::{KeyCode, KeyEvent};
use tracing::{debug, info};

impl Handler {
    /// Opens the delete file/directory modal.
    pub(super) fn handle_delete_file(app: &mut App) {
        if let Some(item) = app.files.get(app.files_selected) {
            let file_name = item.name().to_string();
            let modal = if item.is_dir() {
                debug!(
                    file = %file_name,
                    "opening delete directory modal"
                );
                FileOperationsModal::delete_directory(file_name)
            } else {
                debug!(
                    file = %file_name,
                    "opening delete file modal"
                );
                FileOperationsModal::delete_file(file_name)
            };
            app.modal = Some(ActiveModal::FileOperation(modal));
        }
    }

    /// Opens the mkdir modal.
    pub(super) fn handle_mkdir(app: &mut App) {
        let path = if app.current_path.is_empty() {
            "/".to_string()
        } else {
            app.current_path.clone()
        };
        debug!(path = %path, "opening mkdir modal");
        app.modal = Some(ActiveModal::FileOperation(FileOperationsModal::mkdir(path)));
    }

    /// Opens the copy file modal.
    pub(super) fn handle_copy_file(app: &mut App) {
        if let Some(item) = app.files.get(app.files_selected) {
            let file_name = item.name().to_string();
            let current_path = app.current_path.clone();
            debug!(file = %file_name, "opening copy file modal");
            app.modal = Some(ActiveModal::FileOperation(FileOperationsModal::copy(
                file_name,
                current_path,
            )));
        }
    }

    /// Opens the move file modal.
    pub(super) fn handle_move_file(app: &mut App) {
        if let Some(item) = app.files.get(app.files_selected) {
            let file_name = item.name().to_string();
            let current_path = app.current_path.clone();
            debug!(file = %file_name, "opening move file modal");
            app.modal = Some(ActiveModal::FileOperation(FileOperationsModal::move_file(
                file_name,
                current_path,
            )));
        }
    }

    /// Handles keyboard input in file operations modal.
    pub(super) async fn handle_file_operations_key(app: &mut App, key: KeyEvent) -> Result<()> {
        if let Some(ActiveModal::FileOperation(ref mut modal)) = app.modal {
            match key.code {
                KeyCode::Esc => {
                    debug!("closing file operations modal");
                    app.modal = None;
                }
                KeyCode::Char(c) if modal.needs_input() => {
                    modal.input_char(c);
                }
                KeyCode::Backspace if modal.needs_input() => {
                    modal.backspace();
                }
                KeyCode::Enter => {
                    Self::handle_file_operations_submit(app).await?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Handles file operations modal submission.
    async fn handle_file_operations_submit(app: &mut App) -> Result<()> {
        let Some(ActiveModal::FileOperation(modal)) = app.modal.take() else {
            return Ok(());
        };

        if !modal.is_valid() {
            app.modal = Some(ActiveModal::FileOperation(FileOperationsModal {
                error: Some("Input is required".to_string()),
                ..modal
            }));
            return Ok(());
        }

        let Some(ref remote) = app.current_remote else {
            return Ok(());
        };
        let remote = remote.clone();

        match modal.operation {
            crate::ui::FileOperationType::DeleteFile => {
                info!(file = %modal.file_name, "deleting file");
                if let Err(e) = app.client.delete_file(&remote, &modal.file_name).await {
                    app.modal = Some(ActiveModal::FileOperation(FileOperationsModal {
                        error: Some(format!("Error: {}", e)),
                        ..modal
                    }));
                    return Ok(());
                }
            }
            crate::ui::FileOperationType::DeleteDirectory => {
                info!(
                    dir = %modal.file_name,
                    "purging directory"
                );
                if let Err(e) = app.client.purge(&remote, &modal.file_name).await {
                    app.modal = Some(ActiveModal::FileOperation(FileOperationsModal {
                        error: Some(format!("Error: {}", e)),
                        ..modal
                    }));
                    return Ok(());
                }
            }
            crate::ui::FileOperationType::Mkdir => {
                let new_path = if modal.current_path == "/" {
                    format!("/{}", modal.input)
                } else {
                    format!("{}/{}", modal.current_path, modal.input)
                };
                info!(path = %new_path, "creating directory");
                if let Err(e) = app.client.mkdir(&remote, &new_path).await {
                    app.modal = Some(ActiveModal::FileOperation(FileOperationsModal {
                        error: Some(format!("Error: {}", e)),
                        ..modal
                    }));
                    return Ok(());
                }
            }
            crate::ui::FileOperationType::Copy => {
                info!(
                    src = %modal.file_name,
                    dst = %modal.input,
                    "copying file"
                );
                if let Err(e) = app
                    .client
                    .copy_file(&remote, &modal.file_name, &remote, &modal.input)
                    .await
                {
                    app.modal = Some(ActiveModal::FileOperation(FileOperationsModal {
                        error: Some(format!("Error: {}", e)),
                        ..modal
                    }));
                    return Ok(());
                }
            }
            crate::ui::FileOperationType::Move => {
                info!(
                    src = %modal.file_name,
                    dst = %modal.input,
                    "moving file"
                );
                if let Err(e) = app
                    .client
                    .move_file(&remote, &modal.file_name, &remote, &modal.input)
                    .await
                {
                    app.modal = Some(ActiveModal::FileOperation(FileOperationsModal {
                        error: Some(format!("Error: {}", e)),
                        ..modal
                    }));
                    return Ok(());
                }
            }
        }

        app.load_files().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::Panel;
    use crate::rclone::{FileItem, RcloneClient};
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn create_test_client() -> RcloneClient {
        RcloneClient::new("localhost", 5572).expect("default reqwest client config is valid")
    }

    fn create_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn create_file_item(name: &str, is_dir: bool) -> FileItem {
        FileItem {
            name: name.to_string(),
            size: if is_dir { 0 } else { 100 },
            mod_time: "2024-01-01T00:00:00Z".to_string(),
            is_dir,
        }
    }

    #[tokio::test]
    async fn test_open_delete_file_modal() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = Panel::Files;
        app.files = vec![create_file_item("test.txt", false)];
        app.files_selected = 0;
        assert!(app.file_operations_modal().is_none());

        let key = create_key_event(KeyCode::Char('x'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.file_operations_modal().is_some());
        let modal = app.file_operations_modal().unwrap();
        assert_eq!(modal.operation, crate::ui::FileOperationType::DeleteFile);
        assert_eq!(modal.file_name, "test.txt");
    }

    #[tokio::test]
    async fn test_open_delete_directory_modal() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = Panel::Files;
        app.files = vec![create_file_item("mydir", true)];
        app.files_selected = 0;

        let key = create_key_event(KeyCode::Char('x'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.file_operations_modal().is_some());
        let modal = app.file_operations_modal().unwrap();
        assert_eq!(
            modal.operation,
            crate::ui::FileOperationType::DeleteDirectory
        );
        assert_eq!(modal.file_name, "mydir");
    }

    #[tokio::test]
    async fn test_delete_file_no_files() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = Panel::Files;
        app.files = vec![];

        let key = create_key_event(KeyCode::Char('x'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.file_operations_modal().is_none());
    }

    #[tokio::test]
    async fn test_delete_file_key_ignored_in_remotes_panel() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = Panel::Remotes;
        app.files = vec![create_file_item("test.txt", false)];

        let key = create_key_event(KeyCode::Char('x'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.file_operations_modal().is_none());
    }

    #[tokio::test]
    async fn test_open_mkdir_modal() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = Panel::Files;
        app.current_path = "/some/path".to_string();

        let key = create_key_event(KeyCode::Char('n'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.file_operations_modal().is_some());
        let modal = app.file_operations_modal().unwrap();
        assert_eq!(modal.operation, crate::ui::FileOperationType::Mkdir);
        assert_eq!(modal.current_path, "/some/path");
    }

    #[tokio::test]
    async fn test_open_mkdir_modal_empty_path() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = Panel::Files;
        app.current_path = String::new();

        let key = create_key_event(KeyCode::Char('n'));
        Handler::handle_key(&mut app, key).await.unwrap();

        let modal = app.file_operations_modal().unwrap();
        assert_eq!(modal.current_path, "/");
    }

    #[tokio::test]
    async fn test_mkdir_key_ignored_in_remotes_panel() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = Panel::Remotes;

        let key = create_key_event(KeyCode::Char('n'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.file_operations_modal().is_none());
    }

    #[tokio::test]
    async fn test_open_copy_modal() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = Panel::Files;
        app.current_path = "/current".to_string();
        app.files = vec![create_file_item("source.txt", false)];
        app.files_selected = 0;

        let key = create_key_event(KeyCode::Char('c'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.file_operations_modal().is_some());
        let modal = app.file_operations_modal().unwrap();
        assert_eq!(modal.operation, crate::ui::FileOperationType::Copy);
        assert_eq!(modal.file_name, "source.txt");
        assert_eq!(modal.current_path, "/current");
    }

    #[tokio::test]
    async fn test_copy_no_files() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = Panel::Files;
        app.files = vec![];

        let key = create_key_event(KeyCode::Char('c'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.file_operations_modal().is_none());
    }

    #[tokio::test]
    async fn test_copy_key_ignored_in_remotes_panel() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = Panel::Remotes;
        app.files = vec![create_file_item("source.txt", false)];

        let key = create_key_event(KeyCode::Char('c'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.file_operations_modal().is_none());
    }

    #[tokio::test]
    async fn test_open_move_modal() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = Panel::Files;
        app.current_path = "/current".to_string();
        app.files = vec![create_file_item("source.txt", false)];
        app.files_selected = 0;

        let key = create_key_event(KeyCode::Char('m'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.file_operations_modal().is_some());
        let modal = app.file_operations_modal().unwrap();
        assert_eq!(modal.operation, crate::ui::FileOperationType::Move);
        assert_eq!(modal.file_name, "source.txt");
    }

    #[tokio::test]
    async fn test_move_no_files() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = Panel::Files;
        app.files = vec![];

        let key = create_key_event(KeyCode::Char('m'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.file_operations_modal().is_none());
    }

    #[tokio::test]
    async fn test_move_key_ignored_in_remotes_panel() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = Panel::Remotes;
        app.files = vec![create_file_item("source.txt", false)];

        let key = create_key_event(KeyCode::Char('m'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.file_operations_modal().is_none());
    }

    #[tokio::test]
    async fn test_file_ops_modal_input_char() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.modal = Some(ActiveModal::FileOperation(FileOperationsModal::mkdir(
            "/".to_string(),
        )));

        let key = create_key_event(KeyCode::Char('t'));
        Handler::handle_key(&mut app, key).await.unwrap();

        let modal = app.file_operations_modal().unwrap();
        assert_eq!(modal.input, "t");
    }

    #[tokio::test]
    async fn test_file_ops_modal_backspace() {
        let client = create_test_client();
        let mut app = App::new(client);
        let mut modal = FileOperationsModal::mkdir("/".to_string());
        modal.input = "test".to_string();
        app.modal = Some(ActiveModal::FileOperation(modal));

        let key = create_key_event(KeyCode::Backspace);
        Handler::handle_key(&mut app, key).await.unwrap();

        let modal = app.file_operations_modal().unwrap();
        assert_eq!(modal.input, "tes");
    }

    #[tokio::test]
    async fn test_file_ops_modal_escape_closes() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.modal = Some(ActiveModal::FileOperation(FileOperationsModal::mkdir(
            "/".to_string(),
        )));

        let key = create_key_event(KeyCode::Esc);
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.file_operations_modal().is_none());
    }

    #[tokio::test]
    async fn test_file_ops_modal_delete_no_input_needed() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.modal = Some(ActiveModal::FileOperation(
            FileOperationsModal::delete_file("test.txt".to_string()),
        ));

        let key = create_key_event(KeyCode::Char('x'));
        Handler::handle_key(&mut app, key).await.unwrap();

        let modal = app.file_operations_modal().unwrap();
        assert!(modal.input.is_empty());
    }
}
