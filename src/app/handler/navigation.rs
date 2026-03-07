//! Navigation handling (enter directory, go back).

use super::Handler;
use crate::app::state::{App, Panel};
use crate::error::Result;
use tracing::{debug, info};

impl Handler {
    /// Handles Enter key: select remote or open directory.
    pub(super) async fn handle_enter(app: &mut App) -> Result<()> {
        match app.focused_panel {
            Panel::Remotes => {
                if let Some(remote) = app.remotes.get(app.remotes_selected) {
                    info!(remote = %remote, "selecting remote");
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
                    debug!(dir = name, "opening directory");
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

    /// Handles Backspace key: go to parent directory or back to
    /// remotes.
    pub(super) async fn handle_backspace(app: &mut App) -> Result<()> {
        match app.focused_panel {
            Panel::Files => {
                if !app.current_path.is_empty() {
                    if let Some(last_slash) = app.current_path.rfind('/') {
                        debug!(
                            path = %app.current_path,
                            "going back"
                        );
                        app.current_path.truncate(last_slash);
                    } else {
                        app.current_path.clear();
                    }
                    app.load_files().await?;
                } else {
                    info!("going back to remotes");
                    app.current_remote = None;
                    app.focused_panel = Panel::Remotes;
                    app.files.clear();
                }
            }
            Panel::Remotes => {}
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
            crate::rclone::FileItem {
                name: "file1.txt".to_string(),
                size: 100,
                mod_time: "".to_string(),
                is_dir: false,
            },
            crate::rclone::FileItem {
                name: "file2.txt".to_string(),
                size: 100,
                mod_time: "".to_string(),
                is_dir: false,
            },
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
            crate::rclone::FileItem {
                name: "file1.txt".to_string(),
                size: 100,
                mod_time: "".to_string(),
                is_dir: false,
            },
            crate::rclone::FileItem {
                name: "file2.txt".to_string(),
                size: 100,
                mod_time: "".to_string(),
                is_dir: false,
            },
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
}
