//! Remote modal handling (create, edit, delete, confirm).

use super::Handler;
use crate::app::state::{ActiveModal, App};
use crate::error::Result;
use crate::ui::{ConfirmModal, CreateRemoteModal, CreateRemoteMode};
use crossterm::event::{KeyCode, KeyEvent};
use std::collections::HashMap;
use tracing::{debug, info};

impl Handler {
    /// Handles keyboard input while the create/edit modal is open.
    pub(super) async fn handle_modal_key(app: &mut App, key: KeyEvent) -> Result<()> {
        if let Some(ActiveModal::CreateRemote(ref mut modal)) = app.modal {
            match key.code {
                KeyCode::Esc => {
                    debug!("closing create remote modal");
                    app.modal = None;
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

    /// Handles modal submission.
    async fn handle_modal_submit(app: &mut App) -> Result<()> {
        let Some(ActiveModal::CreateRemote(modal)) = app.modal.take() else {
            return Ok(());
        };

        if !modal.is_valid() {
            app.modal = Some(ActiveModal::CreateRemote(CreateRemoteModal {
                error: Some("Name and Type are required".to_string()),
                ..modal
            }));
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
                info!(remote = %name, "creating remote");
                if let Err(e) = app.client.create_remote(&name, &remote_type, params).await {
                    app.modal = Some(ActiveModal::CreateRemote(CreateRemoteModal {
                        error: Some(format!("Error: {}", e)),
                        ..modal
                    }));
                    return Ok(());
                }
            }
            CreateRemoteMode::Edit => {
                info!(remote = %name, "updating remote");
                if let Err(e) = app.client.update_remote(&name, params).await {
                    app.modal = Some(ActiveModal::CreateRemote(CreateRemoteModal {
                        error: Some(format!("Error: {}", e)),
                        ..modal
                    }));
                    return Ok(());
                }
            }
        }

        app.load_remotes().await?;
        Ok(())
    }

    /// Opens the edit remote modal.
    pub(super) async fn handle_edit_remote(app: &mut App) -> Result<()> {
        if let Some(remote) = app.remotes.get(app.remotes_selected) {
            info!(remote = %remote, "editing remote");
            let modal = CreateRemoteModal::new(CreateRemoteMode::Edit)
                .with_name(remote.clone())
                .with_type("local".to_string());
            app.modal = Some(ActiveModal::CreateRemote(modal));
        }
        Ok(())
    }

    /// Opens the delete remote confirmation modal.
    pub(super) fn handle_delete_remote(app: &mut App) {
        if let Some(remote) = app.remotes.get(app.remotes_selected) {
            debug!(remote = %remote, "opening delete confirmation");
            app.modal = Some(ActiveModal::ConfirmDeleteRemote {
                remote: remote.clone(),
                modal: ConfirmModal::new("Delete Remote", format!("Delete '{}'?", remote)),
            });
        }
    }

    /// Handles confirmation modal input.
    pub(super) async fn handle_confirm_key(app: &mut App, key: KeyEvent) -> Result<()> {
        if let Some(ActiveModal::ConfirmDeleteRemote { ref mut modal, .. }) = app.modal {
            match key.code {
                KeyCode::Esc => {
                    debug!("cancelling delete");
                    app.modal = None;
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
                    // Enter always closes the modal; the delete only runs
                    // when "Yes" is selected.
                    let confirmed = modal.is_confirmed();
                    if let Some(ActiveModal::ConfirmDeleteRemote { remote, .. }) = app.modal.take()
                        && confirmed
                    {
                        info!(remote = %remote, "deleting remote");
                        app.client.delete_remote(&remote).await?;
                        app.load_remotes().await?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rclone::RcloneClient;
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

    #[tokio::test]
    async fn test_open_create_remote_modal() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = crate::app::state::Panel::Remotes;
        assert!(app.create_remote_modal().is_none());

        let key = create_key_event(KeyCode::Char('a'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.create_remote_modal().is_some());
        let modal = app.create_remote_modal().unwrap();
        assert_eq!(modal.mode, CreateRemoteMode::Create);
    }

    #[tokio::test]
    async fn test_create_remote_key_ignored_in_files_panel() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = crate::app::state::Panel::Files;

        let key = create_key_event(KeyCode::Char('a'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.create_remote_modal().is_none());
    }

    #[tokio::test]
    async fn test_open_delete_remote_modal() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = crate::app::state::Panel::Remotes;
        app.remotes = vec!["test_remote".to_string()];
        app.remotes_selected = 0;
        assert!(app.confirm_modal().is_none());

        let key = create_key_event(KeyCode::Char('d'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.confirm_modal().is_some());
        assert_eq!(app.pending_delete_remote(), Some("test_remote"));
    }

    #[tokio::test]
    async fn test_delete_remote_no_remotes() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = crate::app::state::Panel::Remotes;
        app.remotes = vec![];

        let key = create_key_event(KeyCode::Char('d'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.confirm_modal().is_none());
        assert!(app.pending_delete_remote().is_none());
    }

    #[tokio::test]
    async fn test_delete_remote_key_ignored_in_files_panel() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = crate::app::state::Panel::Files;
        app.remotes = vec!["test_remote".to_string()];

        let key = create_key_event(KeyCode::Char('d'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.confirm_modal().is_none());
    }

    #[tokio::test]
    async fn test_modal_input_char() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.modal = Some(ActiveModal::CreateRemote(CreateRemoteModal::new(
            CreateRemoteMode::Create,
        )));

        let key = create_key_event(KeyCode::Char('a'));
        Handler::handle_key(&mut app, key).await.unwrap();

        let modal = app.create_remote_modal().unwrap();
        assert_eq!(modal.name, "a");
    }

    #[tokio::test]
    async fn test_modal_backspace() {
        let client = create_test_client();
        let mut app = App::new(client);
        let mut modal = CreateRemoteModal::new(CreateRemoteMode::Create);
        modal.name = "test".to_string();
        app.modal = Some(ActiveModal::CreateRemote(modal));

        let key = create_key_event(KeyCode::Backspace);
        Handler::handle_key(&mut app, key).await.unwrap();

        let modal = app.create_remote_modal().unwrap();
        assert_eq!(modal.name, "tes");
    }

    #[tokio::test]
    async fn test_modal_escape_closes() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.modal = Some(ActiveModal::CreateRemote(CreateRemoteModal::new(
            CreateRemoteMode::Create,
        )));

        let key = create_key_event(KeyCode::Esc);
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.create_remote_modal().is_none());
    }

    #[tokio::test]
    async fn test_modal_tab_navigation() {
        let client = create_test_client();
        let mut app = App::new(client);
        let modal = CreateRemoteModal::new(CreateRemoteMode::Create);
        app.modal = Some(ActiveModal::CreateRemote(modal));

        let key = create_key_event(KeyCode::Tab);
        Handler::handle_key(&mut app, key).await.unwrap();

        let modal = app.create_remote_modal().unwrap();
        assert_eq!(modal.focus_field, crate::ui::RemoteField::Type);
    }

    #[tokio::test]
    async fn test_modal_back_tab_navigation() {
        let client = create_test_client();
        let mut app = App::new(client);
        let mut modal = CreateRemoteModal::new(CreateRemoteMode::Create);
        modal.focus_field = crate::ui::RemoteField::Type;
        app.modal = Some(ActiveModal::CreateRemote(modal));

        let key = create_key_event(KeyCode::BackTab);
        Handler::handle_key(&mut app, key).await.unwrap();

        let modal = app.create_remote_modal().unwrap();
        assert_eq!(modal.focus_field, crate::ui::RemoteField::Name);
    }

    #[tokio::test]
    async fn test_confirm_modal_escape_closes() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.modal = Some(ActiveModal::ConfirmDeleteRemote {
            remote: "test".to_string(),
            modal: ConfirmModal::new("Test", "Test message".to_string()),
        });

        let key = create_key_event(KeyCode::Esc);
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.confirm_modal().is_none());
        assert!(app.pending_delete_remote().is_none());
    }

    #[tokio::test]
    async fn test_confirm_modal_tab_toggles() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.modal = Some(ActiveModal::ConfirmDeleteRemote {
            remote: "test".to_string(),
            modal: ConfirmModal::new("Test", "Test message".to_string()),
        });

        assert!(!app.confirm_modal().unwrap().is_confirmed());

        let key = create_key_event(KeyCode::Tab);
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.confirm_modal().unwrap().is_confirmed());
    }

    #[tokio::test]
    async fn test_confirm_modal_left_right_toggle() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.modal = Some(ActiveModal::ConfirmDeleteRemote {
            remote: "test".to_string(),
            modal: ConfirmModal::new("Test", "Test message".to_string()),
        });

        let key = create_key_event(KeyCode::Right);
        Handler::handle_key(&mut app, key).await.unwrap();
        assert!(app.confirm_modal().unwrap().is_confirmed());

        let key = create_key_event(KeyCode::Left);
        Handler::handle_key(&mut app, key).await.unwrap();
        assert!(!app.confirm_modal().unwrap().is_confirmed());
    }

    #[tokio::test]
    async fn test_confirm_modal_y_key() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.modal = Some(ActiveModal::ConfirmDeleteRemote {
            remote: "test".to_string(),
            modal: ConfirmModal::new("Test", "Test message".to_string()),
        });

        let key = create_key_event(KeyCode::Char('y'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.confirm_modal().unwrap().is_confirmed());
    }

    #[tokio::test]
    async fn test_confirm_modal_n_key() {
        let client = create_test_client();
        let mut app = App::new(client);
        let mut modal = ConfirmModal::new("Test", "Test message".to_string());
        modal.toggle();
        app.modal = Some(ActiveModal::ConfirmDeleteRemote {
            remote: "test".to_string(),
            modal,
        });

        let key = create_key_event(KeyCode::Char('n'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(!app.confirm_modal().unwrap().is_confirmed());
    }

    #[tokio::test]
    async fn test_modal_input_clears_error() {
        let client = create_test_client();
        let mut app = App::new(client);
        let mut modal = CreateRemoteModal::new(CreateRemoteMode::Create);
        modal.error = Some("Previous error".to_string());
        app.modal = Some(ActiveModal::CreateRemote(modal));

        let key = create_key_event(KeyCode::Char('a'));
        Handler::handle_key(&mut app, key).await.unwrap();

        let modal = app.create_remote_modal().unwrap();
        assert!(modal.error.is_none());
    }

    #[tokio::test]
    async fn test_open_modal_keys_ignored_while_confirm_open() {
        // 'a' opens the create-remote modal only when no modal is open;
        // with the confirmation active the key must not replace it.
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = crate::app::state::Panel::Remotes;
        app.modal = Some(ActiveModal::ConfirmDeleteRemote {
            remote: "test".to_string(),
            modal: ConfirmModal::new("Test", "Test message".to_string()),
        });

        let key = create_key_event(KeyCode::Char('a'));
        Handler::handle_key(&mut app, key).await.unwrap();

        assert!(app.confirm_modal().is_some());
        assert!(app.create_remote_modal().is_none());
    }
}
