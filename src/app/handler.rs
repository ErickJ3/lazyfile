//! Keyboard event handling.

use super::state::{App, Panel};
use crate::error::Result;
use crate::ui::{ConfirmModal, CreateRemoteModal, CreateRemoteMode, FileOperationsModal};
use crossterm::event::{KeyCode, KeyEvent};
use std::collections::HashMap;
use tracing::{debug, info};

/// Handles keyboard input events.
pub struct Handler;

impl Handler {
    /// Process a keyboard event and update app state.
    ///
    /// # Arguments
    /// * `app` - Mutable reference to the application state
    /// * `key` - The keyboard event to handle
    ///
    /// # Errors
    /// Returns error if rclone API calls fail.
    pub async fn handle_key(app: &mut App, key: KeyEvent) -> Result<()> {
        // If file operations modal is open, handle it
        if app.file_operations_modal.is_some() {
            return Self::handle_file_operations_key(app, key).await;
        }

        // If confirmation modal is open, handle it
        if app.confirm_modal.is_some() {
            return Self::handle_confirm_key(app, key).await;
        }

        // If create/edit modal is open, handle it
        if app.create_remote_modal.is_some() {
            return Self::handle_modal_key(app, key).await;
        }

        match key.code {
            KeyCode::Char('q') => {
                info!("Quit requested");
                app.running = false;
            }
            KeyCode::Char('a') if matches!(app.focused_panel, Panel::Remotes) => {
                debug!("Opening create remote modal");
                app.create_remote_modal = Some(CreateRemoteModal::new(CreateRemoteMode::Create));
            }
            KeyCode::Char('d') if matches!(app.focused_panel, Panel::Remotes) => {
                Self::handle_delete_remote(app);
            }
            KeyCode::Char('e') if matches!(app.focused_panel, Panel::Remotes) => {
                Self::handle_edit_remote(app).await?;
            }
            KeyCode::Char('x') if matches!(app.focused_panel, Panel::Files) => {
                Self::handle_delete_file(app);
            }
            KeyCode::Char('n') if matches!(app.focused_panel, Panel::Files) => {
                Self::handle_mkdir(app);
            }
            KeyCode::Char('c') if matches!(app.focused_panel, Panel::Files) => {
                Self::handle_copy_file(app);
            }
            KeyCode::Char('m') if matches!(app.focused_panel, Panel::Files) => {
                Self::handle_move_file(app);
            }
            KeyCode::Char('j') | KeyCode::Down => {
                app.navigate_down();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.navigate_up();
            }
            KeyCode::Tab => {
                app.switch_panel();
            }
            KeyCode::Enter => {
                Self::handle_enter(app).await?;
            }
            KeyCode::Backspace => {
                Self::handle_backspace(app).await?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle keyboard input while modal is open.
    async fn handle_modal_key(app: &mut App, key: KeyEvent) -> Result<()> {
        if let Some(ref mut modal) = app.create_remote_modal {
            match key.code {
                KeyCode::Esc => {
                    debug!("Closing create remote modal");
                    app.create_remote_modal = None;
                }
                KeyCode::Tab => {
                    modal.next_field();
                }
                KeyCode::BackTab => {
                    modal.prev_field();
                }
                KeyCode::Char(c) => {
                    modal.input_char(c);
                    modal.error = None;
                }
                KeyCode::Backspace => {
                    modal.backspace();
                    modal.error = None;
                }
                KeyCode::Enter => {
                    Self::handle_modal_submit(app).await?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Handle modal submission.
    async fn handle_modal_submit(app: &mut App) -> Result<()> {
        if let Some(modal) = app.create_remote_modal.take() {
            if !modal.is_valid() {
                app.create_remote_modal = Some(CreateRemoteModal {
                    error: Some("Name and Type are required".to_string()),
                    ..modal
                });
                return Ok(());
            }

            let mut params = HashMap::new();
            if !modal.path.is_empty() {
                params.insert("path".to_string(), modal.path.clone());
            }

            let name = modal.name.clone();
            let remote_type = modal.remote_type.clone();
            let mode = modal.mode;

            match mode {
                CreateRemoteMode::Create => {
                    info!("Creating remote: {}", name);
                    if let Err(e) = app.client.create_remote(&name, &remote_type, params).await {
                        app.create_remote_modal = Some(CreateRemoteModal {
                            error: Some(format!("Error: {}", e)),
                            ..modal
                        });
                        return Ok(());
                    }
                }
                CreateRemoteMode::Edit => {
                    info!("Updating remote: {}", name);
                    if let Err(e) = app.client.update_remote(&name, params).await {
                        app.create_remote_modal = Some(CreateRemoteModal {
                            error: Some(format!("Error: {}", e)),
                            ..modal
                        });
                        return Ok(());
                    }
                }
            }

            app.load_remotes().await?;
        }
        Ok(())
    }

    /// Handle delete remote - open confirmation modal.
    fn handle_delete_remote(app: &mut App) {
        if let Some(remote) = app.remotes.get(app.remotes_selected) {
            debug!("Opening delete confirmation for: {}", remote);
            app.pending_delete_remote = Some(remote.clone());
            app.confirm_modal = Some(ConfirmModal::new(
                "Delete Remote",
                format!("Delete '{}'?", remote),
            ));
        }
    }

    /// Handle confirmation modal input.
    async fn handle_confirm_key(app: &mut App, key: KeyEvent) -> Result<()> {
        if let Some(ref mut modal) = app.confirm_modal {
            match key.code {
                KeyCode::Esc => {
                    debug!("Cancelling delete");
                    app.confirm_modal = None;
                    app.pending_delete_remote = None;
                }
                KeyCode::Tab | KeyCode::Right | KeyCode::Left => {
                    modal.toggle();
                }
                KeyCode::Char(c) if c == 'y' || c == 'n' => {
                    let confirmed = c == 'y';
                    if confirmed != modal.is_confirmed() {
                        modal.toggle();
                    }
                }
                KeyCode::Enter => {
                    if modal.is_confirmed()
                        && let Some(remote) = app.pending_delete_remote.take()
                    {
                        info!("Deleting remote: {}", remote);
                        app.client.delete_remote(&remote).await?;
                        app.load_remotes().await?;
                    }
                    app.confirm_modal = None;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Handle edit remote.
    async fn handle_edit_remote(app: &mut App) -> Result<()> {
        if let Some(remote) = app.remotes.get(app.remotes_selected) {
            info!("Editing remote: {}", remote);
            let modal = CreateRemoteModal::new(CreateRemoteMode::Edit)
                .with_name(remote.clone())
                .with_type("local".to_string());
            app.create_remote_modal = Some(modal);
        }
        Ok(())
    }

    /// Handle Enter key: select remote or open directory.
    async fn handle_enter(app: &mut App) -> Result<()> {
        match app.focused_panel {
            Panel::Remotes => {
                if let Some(remote) = app.remotes.get(app.remotes_selected) {
                    info!("Selecting remote: {}", remote);
                    app.current_remote = Some(remote.clone());
                    app.current_path = String::new();
                    app.load_files().await?;
                    app.focused_panel = Panel::Files;
                }
            }
            Panel::Files => {
                if let Some(item) = app.files.get(app.files_selected)
                    && item.is_dir()
                {
                    let name = item.name();
                    debug!("Opening directory: {}", name);
                    if app.current_path.is_empty() {
                        app.current_path = name.to_string();
                    } else {
                        app.current_path = format!("{}/{}", app.current_path, name);
                    }
                    app.load_files().await?;
                }
            }
        }
        Ok(())
    }

    /// Handle Backspace key: go to parent directory or back to remotes.
    async fn handle_backspace(app: &mut App) -> Result<()> {
        match app.focused_panel {
            Panel::Files => {
                if !app.current_path.is_empty() {
                    if let Some(last_slash) = app.current_path.rfind('/') {
                        debug!("Going back from {}", app.current_path);
                        app.current_path.truncate(last_slash);
                    } else {
                        app.current_path.clear();
                    }
                    app.load_files().await?;
                } else {
                    info!("Going back to remotes");
                    app.current_remote = None;
                    app.focused_panel = Panel::Remotes;
                    app.files.clear();
                }
            }
            Panel::Remotes => {}
        }
        Ok(())
    }

    /// Handle delete file operation.
    fn handle_delete_file(app: &mut App) {
        if let Some(item) = app.files.get(app.files_selected) {
            let file_name = item.name().to_string();
            if item.is_dir() {
                debug!("Opening delete directory modal for: {}", file_name);
                app.file_operations_modal = Some(FileOperationsModal::delete_directory(file_name));
            } else {
                debug!("Opening delete file modal for: {}", file_name);
                app.file_operations_modal = Some(FileOperationsModal::delete_file(file_name));
            }
        }
    }

    /// Handle mkdir operation.
    fn handle_mkdir(app: &mut App) {
        let path = if app.current_path.is_empty() {
            "/".to_string()
        } else {
            app.current_path.clone()
        };
        debug!("Opening mkdir modal for path: {}", path);
        app.file_operations_modal = Some(FileOperationsModal::mkdir(path));
    }

    /// Handle copy file operation.
    fn handle_copy_file(app: &mut App) {
        if let Some(item) = app.files.get(app.files_selected) {
            let file_name = item.name().to_string();
            let current_path = app.current_path.clone();
            debug!("Opening copy file modal for: {}", file_name);
            app.file_operations_modal = Some(FileOperationsModal::copy(file_name, current_path));
        }
    }

    /// Handle move file operation.
    fn handle_move_file(app: &mut App) {
        if let Some(item) = app.files.get(app.files_selected) {
            let file_name = item.name().to_string();
            let current_path = app.current_path.clone();
            debug!("Opening move file modal for: {}", file_name);
            app.file_operations_modal =
                Some(FileOperationsModal::move_file(file_name, current_path));
        }
    }

    /// Handle keyboard input in file operations modal.
    async fn handle_file_operations_key(app: &mut App, key: KeyEvent) -> Result<()> {
        if let Some(ref mut modal) = app.file_operations_modal {
            match key.code {
                KeyCode::Esc => {
                    debug!("Closing file operations modal");
                    app.file_operations_modal = None;
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

    /// Handle file operations modal submission.
    async fn handle_file_operations_submit(app: &mut App) -> Result<()> {
        if let Some(modal) = app.file_operations_modal.take() {
            if !modal.is_valid() {
                app.file_operations_modal = Some(FileOperationsModal {
                    error: Some("Input is required".to_string()),
                    ..modal
                });
                return Ok(());
            }

            let remote = if let Some(ref r) = app.current_remote {
                r.clone()
            } else {
                return Ok(());
            };

            match modal.operation {
                crate::ui::FileOperationType::DeleteFile => {
                    info!("Deleting file: {}", modal.file_name);
                    if let Err(e) = app.client.delete_file(&remote, &modal.file_name).await {
                        app.file_operations_modal = Some(FileOperationsModal {
                            error: Some(format!("Error: {}", e)),
                            ..modal
                        });
                        return Ok(());
                    }
                }
                crate::ui::FileOperationType::DeleteDirectory => {
                    info!("Purging directory: {}", modal.file_name);
                    if let Err(e) = app.client.purge(&remote, &modal.file_name).await {
                        app.file_operations_modal = Some(FileOperationsModal {
                            error: Some(format!("Error: {}", e)),
                            ..modal
                        });
                        return Ok(());
                    }
                }
                crate::ui::FileOperationType::Mkdir => {
                    let new_path = if modal.current_path == "/" {
                        format!("/{}", modal.input)
                    } else {
                        format!("{}/{}", modal.current_path, modal.input)
                    };
                    info!("Creating directory: {}", new_path);
                    if let Err(e) = app.client.mkdir(&remote, &new_path).await {
                        app.file_operations_modal = Some(FileOperationsModal {
                            error: Some(format!("Error: {}", e)),
                            ..modal
                        });
                        return Ok(());
                    }
                }
                crate::ui::FileOperationType::Copy => {
                    info!("Copying file from {} to {}", modal.file_name, modal.input);
                    if let Err(e) = app
                        .client
                        .copy_file(&remote, &modal.file_name, &remote, &modal.input)
                        .await
                    {
                        app.file_operations_modal = Some(FileOperationsModal {
                            error: Some(format!("Error: {}", e)),
                            ..modal
                        });
                        return Ok(());
                    }
                }
                crate::ui::FileOperationType::Move => {
                    info!("Moving file from {} to {}", modal.file_name, modal.input);
                    if let Err(e) = app
                        .client
                        .move_file(&remote, &modal.file_name, &remote, &modal.input)
                        .await
                    {
                        app.file_operations_modal = Some(FileOperationsModal {
                            error: Some(format!("Error: {}", e)),
                            ..modal
                        });
                        return Ok(());
                    }
                }
            }

            app.load_files().await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rclone::{FileItem, NavigationItem, RcloneClient};
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn create_test_client() -> RcloneClient {
        RcloneClient::new("localhost", 5572)
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
    async fn test_quit_key() {
        let client = create_test_client();
        let mut app = App::new(client);
        assert!(app.running);

        let key = create_key_event(KeyCode::Char('q'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(!app.running);
    }

    #[tokio::test]
    async fn test_quit_from_files_panel() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = Panel::Files;
        assert!(app.running);

        let key = create_key_event(KeyCode::Char('q'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(!app.running);
    }

    #[tokio::test]
    async fn test_navigate_down_with_j() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.remotes = vec!["remote1".to_string(), "remote2".to_string()];
        app.focused_panel = Panel::Remotes;
        assert_eq!(app.remotes_selected, 0);

        let key = create_key_event(KeyCode::Char('j'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert_eq!(app.remotes_selected, 1);
    }

    #[tokio::test]
    async fn test_navigate_down_with_arrow() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.remotes = vec!["remote1".to_string(), "remote2".to_string()];
        app.focused_panel = Panel::Remotes;

        let key = create_key_event(KeyCode::Down);
        Handler::handle_key(&mut app, key).await.unwrap();

        assert_eq!(app.remotes_selected, 1);
    }

    #[tokio::test]
    async fn test_navigate_up_with_k() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.remotes = vec!["remote1".to_string(), "remote2".to_string()];
        app.remotes_selected = 1;
        app.focused_panel = Panel::Remotes;

        let key = create_key_event(KeyCode::Char('k'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert_eq!(app.remotes_selected, 0);
    }

    #[tokio::test]
    async fn test_navigate_up_with_arrow() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.remotes = vec!["remote1".to_string(), "remote2".to_string()];
        app.remotes_selected = 1;
        app.focused_panel = Panel::Remotes;

        let key = create_key_event(KeyCode::Up);
        Handler::handle_key(&mut app, key).await.unwrap();

        assert_eq!(app.remotes_selected, 0);
    }

    #[tokio::test]
    async fn test_navigate_files_panel_down() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.files = vec![
            NavigationItem::File(create_file_item("file1.txt", false)),
            NavigationItem::File(create_file_item("file2.txt", false)),
        ];
        app.focused_panel = Panel::Files;

        let key = create_key_event(KeyCode::Char('j'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert_eq!(app.files_selected, 1);
    }

    #[tokio::test]
    async fn test_navigate_files_panel_up() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.files = vec![
            NavigationItem::File(create_file_item("file1.txt", false)),
            NavigationItem::File(create_file_item("file2.txt", false)),
        ];
        app.files_selected = 1;
        app.focused_panel = Panel::Files;

        let key = create_key_event(KeyCode::Char('k'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert_eq!(app.files_selected, 0);
    }

    #[tokio::test]
    async fn test_switch_panel_with_tab() {
        let client = create_test_client();
        let mut app = App::new(client);
        assert_eq!(app.focused_panel, Panel::Remotes);

        let key = create_key_event(KeyCode::Tab);
        Handler::handle_key(&mut app, key).await.unwrap();

        assert_eq!(app.focused_panel, Panel::Files);
    }

    #[tokio::test]
    async fn test_switch_panel_back_with_tab() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = Panel::Files;

        let key = create_key_event(KeyCode::Tab);
        Handler::handle_key(&mut app, key).await.unwrap();

        assert_eq!(app.focused_panel, Panel::Remotes);
    }

    #[tokio::test]
    async fn test_switch_panel_multiple_times() {
        let client = create_test_client();
        let mut app = App::new(client);

        for i in 0..10 {
            let key = create_key_event(KeyCode::Tab);
            Handler::handle_key(&mut app, key).await.unwrap();

            let expected = if i % 2 == 0 {
                Panel::Files
            } else {
                Panel::Remotes
            };
            assert_eq!(app.focused_panel, expected);
        }
    }

    #[tokio::test]
    async fn test_open_create_remote_modal() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = Panel::Remotes;
        assert!(app.create_remote_modal.is_none());

        let key = create_key_event(KeyCode::Char('a'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.create_remote_modal.is_some());
        let modal = app.create_remote_modal.as_ref().unwrap();
        assert_eq!(modal.mode, CreateRemoteMode::Create);
    }

    #[tokio::test]
    async fn test_create_remote_key_ignored_in_files_panel() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = Panel::Files;

        let key = create_key_event(KeyCode::Char('a'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.create_remote_modal.is_none());
    }

    #[tokio::test]
    async fn test_open_delete_remote_modal() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = Panel::Remotes;
        app.remotes = vec!["test_remote".to_string()];
        app.remotes_selected = 0;
        assert!(app.confirm_modal.is_none());

        let key = create_key_event(KeyCode::Char('d'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.confirm_modal.is_some());
        assert_eq!(app.pending_delete_remote, Some("test_remote".to_string()));
    }

    #[tokio::test]
    async fn test_delete_remote_no_remotes() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = Panel::Remotes;
        app.remotes = vec![];

        let key = create_key_event(KeyCode::Char('d'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.confirm_modal.is_none());
        assert!(app.pending_delete_remote.is_none());
    }

    #[tokio::test]
    async fn test_delete_remote_key_ignored_in_files_panel() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = Panel::Files;
        app.remotes = vec!["test_remote".to_string()];

        let key = create_key_event(KeyCode::Char('d'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.confirm_modal.is_none());
    }

    #[tokio::test]
    async fn test_open_delete_file_modal() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = Panel::Files;
        app.files = vec![NavigationItem::File(create_file_item("test.txt", false))];
        app.files_selected = 0;
        assert!(app.file_operations_modal.is_none());

        let key = create_key_event(KeyCode::Char('x'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.file_operations_modal.is_some());
        let modal = app.file_operations_modal.as_ref().unwrap();
        assert_eq!(modal.operation, crate::ui::FileOperationType::DeleteFile);
        assert_eq!(modal.file_name, "test.txt");
    }

    #[tokio::test]
    async fn test_open_delete_directory_modal() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = Panel::Files;
        app.files = vec![NavigationItem::File(create_file_item("mydir", true))];
        app.files_selected = 0;

        let key = create_key_event(KeyCode::Char('x'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.file_operations_modal.is_some());
        let modal = app.file_operations_modal.as_ref().unwrap();
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

        assert!(app.file_operations_modal.is_none());
    }

    #[tokio::test]
    async fn test_delete_file_key_ignored_in_remotes_panel() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = Panel::Remotes;
        app.files = vec![NavigationItem::File(create_file_item("test.txt", false))];

        let key = create_key_event(KeyCode::Char('x'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.file_operations_modal.is_none());
    }

    #[tokio::test]
    async fn test_open_mkdir_modal() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = Panel::Files;
        app.current_path = "/some/path".to_string();

        let key = create_key_event(KeyCode::Char('n'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.file_operations_modal.is_some());
        let modal = app.file_operations_modal.as_ref().unwrap();
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

        let modal = app.file_operations_modal.as_ref().unwrap();
        assert_eq!(modal.current_path, "/");
    }

    #[tokio::test]
    async fn test_mkdir_key_ignored_in_remotes_panel() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = Panel::Remotes;

        let key = create_key_event(KeyCode::Char('n'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.file_operations_modal.is_none());
    }

    #[tokio::test]
    async fn test_open_copy_modal() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = Panel::Files;
        app.current_path = "/current".to_string();
        app.files = vec![NavigationItem::File(create_file_item("source.txt", false))];
        app.files_selected = 0;

        let key = create_key_event(KeyCode::Char('c'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.file_operations_modal.is_some());
        let modal = app.file_operations_modal.as_ref().unwrap();
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

        assert!(app.file_operations_modal.is_none());
    }

    #[tokio::test]
    async fn test_copy_key_ignored_in_remotes_panel() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = Panel::Remotes;
        app.files = vec![NavigationItem::File(create_file_item("source.txt", false))];

        let key = create_key_event(KeyCode::Char('c'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.file_operations_modal.is_none());
    }

    #[tokio::test]
    async fn test_open_move_modal() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = Panel::Files;
        app.current_path = "/current".to_string();
        app.files = vec![NavigationItem::File(create_file_item("source.txt", false))];
        app.files_selected = 0;

        let key = create_key_event(KeyCode::Char('m'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.file_operations_modal.is_some());
        let modal = app.file_operations_modal.as_ref().unwrap();
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

        assert!(app.file_operations_modal.is_none());
    }

    #[tokio::test]
    async fn test_move_key_ignored_in_remotes_panel() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = Panel::Remotes;
        app.files = vec![NavigationItem::File(create_file_item("source.txt", false))];

        let key = create_key_event(KeyCode::Char('m'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.file_operations_modal.is_none());
    }

    #[tokio::test]
    async fn test_modal_input_char() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.create_remote_modal = Some(CreateRemoteModal::new(CreateRemoteMode::Create));

        let key = create_key_event(KeyCode::Char('a'));
        Handler::handle_key(&mut app, key).await.unwrap();

        let modal = app.create_remote_modal.as_ref().unwrap();
        assert_eq!(modal.name, "a");
    }

    #[tokio::test]
    async fn test_modal_backspace() {
        let client = create_test_client();
        let mut app = App::new(client);
        let mut modal = CreateRemoteModal::new(CreateRemoteMode::Create);
        modal.name = "test".to_string();
        app.create_remote_modal = Some(modal);

        let key = create_key_event(KeyCode::Backspace);
        Handler::handle_key(&mut app, key).await.unwrap();

        let modal = app.create_remote_modal.as_ref().unwrap();
        assert_eq!(modal.name, "tes");
    }

    #[tokio::test]
    async fn test_modal_escape_closes() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.create_remote_modal = Some(CreateRemoteModal::new(CreateRemoteMode::Create));

        let key = create_key_event(KeyCode::Esc);
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.create_remote_modal.is_none());
    }

    #[tokio::test]
    async fn test_modal_tab_navigation() {
        let client = create_test_client();
        let mut app = App::new(client);
        let modal = CreateRemoteModal::new(CreateRemoteMode::Create);
        app.create_remote_modal = Some(modal);

        let key = create_key_event(KeyCode::Tab);
        Handler::handle_key(&mut app, key).await.unwrap();

        let modal = app.create_remote_modal.as_ref().unwrap();
        assert_eq!(modal.focus_field, crate::ui::RemoteField::Type);
    }

    #[tokio::test]
    async fn test_modal_back_tab_navigation() {
        let client = create_test_client();
        let mut app = App::new(client);
        let mut modal = CreateRemoteModal::new(CreateRemoteMode::Create);
        modal.focus_field = crate::ui::RemoteField::Type;
        app.create_remote_modal = Some(modal);

        let key = create_key_event(KeyCode::BackTab);
        Handler::handle_key(&mut app, key).await.unwrap();

        let modal = app.create_remote_modal.as_ref().unwrap();
        assert_eq!(modal.focus_field, crate::ui::RemoteField::Name);
    }

    #[tokio::test]
    async fn test_confirm_modal_escape_closes() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.confirm_modal = Some(ConfirmModal::new("Test", "Test message".to_string()));
        app.pending_delete_remote = Some("test".to_string());

        let key = create_key_event(KeyCode::Esc);
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.confirm_modal.is_none());
        assert!(app.pending_delete_remote.is_none());
    }

    #[tokio::test]
    async fn test_confirm_modal_tab_toggles() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.confirm_modal = Some(ConfirmModal::new("Test", "Test message".to_string()));

        assert!(!app.confirm_modal.as_ref().unwrap().is_confirmed());

        let key = create_key_event(KeyCode::Tab);
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.confirm_modal.as_ref().unwrap().is_confirmed());
    }

    #[tokio::test]
    async fn test_confirm_modal_left_right_toggle() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.confirm_modal = Some(ConfirmModal::new("Test", "Test message".to_string()));

        let key = create_key_event(KeyCode::Right);
        Handler::handle_key(&mut app, key).await.unwrap();
        assert!(app.confirm_modal.as_ref().unwrap().is_confirmed());

        let key = create_key_event(KeyCode::Left);
        Handler::handle_key(&mut app, key).await.unwrap();
        assert!(!app.confirm_modal.as_ref().unwrap().is_confirmed());
    }

    #[tokio::test]
    async fn test_confirm_modal_y_key() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.confirm_modal = Some(ConfirmModal::new("Test", "Test message".to_string()));

        let key = create_key_event(KeyCode::Char('y'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.confirm_modal.as_ref().unwrap().is_confirmed());
    }

    #[tokio::test]
    async fn test_confirm_modal_n_key() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.confirm_modal = Some(ConfirmModal::new("Test", "Test message".to_string()));
        app.confirm_modal.as_mut().unwrap().toggle();

        let key = create_key_event(KeyCode::Char('n'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(!app.confirm_modal.as_ref().unwrap().is_confirmed());
    }

    #[tokio::test]
    async fn test_file_ops_modal_input_char() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.file_operations_modal = Some(FileOperationsModal::mkdir("/".to_string()));

        let key = create_key_event(KeyCode::Char('t'));
        Handler::handle_key(&mut app, key).await.unwrap();

        let modal = app.file_operations_modal.as_ref().unwrap();
        assert_eq!(modal.input, "t");
    }

    #[tokio::test]
    async fn test_file_ops_modal_backspace() {
        let client = create_test_client();
        let mut app = App::new(client);
        let mut modal = FileOperationsModal::mkdir("/".to_string());
        modal.input = "test".to_string();
        app.file_operations_modal = Some(modal);

        let key = create_key_event(KeyCode::Backspace);
        Handler::handle_key(&mut app, key).await.unwrap();

        let modal = app.file_operations_modal.as_ref().unwrap();
        assert_eq!(modal.input, "tes");
    }

    #[tokio::test]
    async fn test_file_ops_modal_escape_closes() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.file_operations_modal = Some(FileOperationsModal::mkdir("/".to_string()));

        let key = create_key_event(KeyCode::Esc);
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.file_operations_modal.is_none());
    }

    #[tokio::test]
    async fn test_file_ops_modal_delete_no_input_needed() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.file_operations_modal = Some(FileOperationsModal::delete_file("test.txt".to_string()));

        let key = create_key_event(KeyCode::Char('x'));
        Handler::handle_key(&mut app, key).await.unwrap();

        let modal = app.file_operations_modal.as_ref().unwrap();
        assert!(modal.input.is_empty());
    }

    #[tokio::test]
    async fn test_file_operations_modal_has_priority() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.file_operations_modal = Some(FileOperationsModal::mkdir("/".to_string()));
        app.confirm_modal = Some(ConfirmModal::new("Test", "message".to_string()));
        app.create_remote_modal = Some(CreateRemoteModal::new(CreateRemoteMode::Create));

        let key = create_key_event(KeyCode::Esc);
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.file_operations_modal.is_none());
        assert!(app.confirm_modal.is_some());
        assert!(app.create_remote_modal.is_some());
    }

    #[tokio::test]
    async fn test_confirm_modal_has_second_priority() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.confirm_modal = Some(ConfirmModal::new("Test", "message".to_string()));
        app.create_remote_modal = Some(CreateRemoteModal::new(CreateRemoteMode::Create));

        let key = create_key_event(KeyCode::Esc);
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.confirm_modal.is_none());
        assert!(app.create_remote_modal.is_some());
    }

    #[tokio::test]
    async fn test_unknown_key_does_nothing() {
        let client = create_test_client();
        let mut app = App::new(client);
        let initial_state = app.running;
        let initial_panel = app.focused_panel;

        let key = create_key_event(KeyCode::Char('z'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert_eq!(app.running, initial_state);
        assert_eq!(app.focused_panel, initial_panel);
        assert!(app.create_remote_modal.is_none());
        assert!(app.file_operations_modal.is_none());
    }

    #[tokio::test]
    async fn test_function_keys_ignored() {
        let client = create_test_client();
        let mut app = App::new(client);

        for i in 1..=12 {
            let key = create_key_event(KeyCode::F(i));
            let result = Handler::handle_key(&mut app, key).await;
            assert!(result.is_ok());
        }

        assert!(app.running);
    }

    #[tokio::test]
    async fn test_navigate_with_empty_list() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.remotes = vec![];
        app.files = vec![];

        let key = create_key_event(KeyCode::Char('j'));
        Handler::handle_key(&mut app, key).await.unwrap();
        assert_eq!(app.remotes_selected, 0);

        app.focused_panel = Panel::Files;
        Handler::handle_key(&mut app, key).await.unwrap();
        assert_eq!(app.files_selected, 0);
    }

    #[tokio::test]
    async fn test_navigate_up_at_zero() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.remotes = vec!["remote".to_string()];
        app.remotes_selected = 0;

        let key = create_key_event(KeyCode::Char('k'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert_eq!(app.remotes_selected, 0);
    }

    #[tokio::test]
    async fn test_navigate_down_at_max() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.remotes = vec!["remote1".to_string(), "remote2".to_string()];
        app.remotes_selected = 1;

        let key = create_key_event(KeyCode::Char('j'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert_eq!(app.remotes_selected, 1);
    }

    #[tokio::test]
    async fn test_multiple_rapid_keypresses() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.remotes = vec![
            "r1".to_string(),
            "r2".to_string(),
            "r3".to_string(),
            "r4".to_string(),
            "r5".to_string(),
        ];

        for _ in 0..10 {
            let key = create_key_event(KeyCode::Char('j'));
            Handler::handle_key(&mut app, key).await.unwrap();
        }

        assert_eq!(app.remotes_selected, 4);
    }

    #[tokio::test]
    async fn test_modal_input_clears_error() {
        let client = create_test_client();
        let mut app = App::new(client);
        let mut modal = CreateRemoteModal::new(CreateRemoteMode::Create);
        modal.error = Some("Previous error".to_string());
        app.create_remote_modal = Some(modal);

        let key = create_key_event(KeyCode::Char('a'));
        Handler::handle_key(&mut app, key).await.unwrap();

        let modal = app.create_remote_modal.as_ref().unwrap();
        assert!(modal.error.is_none());
    }
}
