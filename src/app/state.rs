//! Application state management.

use crate::error::Result;
use crate::rclone::{NavigationItem, RcloneClient};
use crate::ui::{ConfirmModal, CreateRemoteModal};
use tracing::{debug, info};

/// Represents the focused panel in the UI.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Panel {
    /// Remote list on the left.
    Remotes,
    /// Files list on the right.
    Files,
}

/// Main application state.
#[derive(Debug)]
pub struct App {
    /// RcloneClient for API communication.
    pub client: RcloneClient,
    /// List of configured remotes.
    pub remotes: Vec<String>,
    /// Currently selected remote.
    pub current_remote: Option<String>,
    /// Current path within the remote.
    pub current_path: String,
    /// Files and directories in current path.
    pub files: Vec<NavigationItem>,
    /// Selected index in remotes list.
    pub remotes_selected: usize,
    /// Selected index in files list.
    pub files_selected: usize,
    /// Currently focused panel.
    pub focused_panel: Panel,
    /// Whether the app should continue running.
    pub running: bool,
    /// Modal for creating/editing remotes.
    pub create_remote_modal: Option<CreateRemoteModal>,
    /// Confirmation modal for delete operations.
    pub confirm_modal: Option<ConfirmModal>,
    /// Remote name being deleted (used for confirmation).
    pub pending_delete_remote: Option<String>,
}

impl App {
    /// Create a new App instance.
    pub fn new(client: RcloneClient) -> Self {
        Self {
            client,
            remotes: Vec::new(),
            current_remote: None,
            current_path: String::new(),
            files: Vec::new(),
            remotes_selected: 0,
            files_selected: 0,
            focused_panel: Panel::Remotes,
            running: true,
            create_remote_modal: None,
            confirm_modal: None,
            pending_delete_remote: None,
        }
    }

    /// Load remotes from rclone daemon.
    pub async fn load_remotes(&mut self) -> Result<()> {
        debug!("Loading remotes");
        self.remotes = self.client.list_remotes().await?;
        self.remotes_selected = 0;
        info!("Loaded {} remotes", self.remotes.len());
        Ok(())
    }

    /// Load files from current remote and path.
    pub async fn load_files(&mut self) -> Result<()> {
        if let Some(ref remote) = self.current_remote {
            debug!("Loading files from {}:{}", remote, self.current_path);
            let items = self.client.list_files(remote, &self.current_path).await?;
            self.files = items.into_iter().map(NavigationItem::File).collect();
            info!("Loaded {} files", self.files.len());
        }
        self.files_selected = 0;
        Ok(())
    }

    /// Move selection down in focused panel.
    pub fn navigate_down(&mut self) {
        match self.focused_panel {
            Panel::Remotes => {
                if self.remotes_selected < self.remotes.len().saturating_sub(1) {
                    self.remotes_selected += 1;
                    debug!("Navigate down in remotes: {}", self.remotes_selected);
                }
            }
            Panel::Files => {
                if self.files_selected < self.files.len().saturating_sub(1) {
                    self.files_selected += 1;
                    debug!("Navigate down in files: {}", self.files_selected);
                }
            }
        }
    }

    /// Move selection up in focused panel.
    pub fn navigate_up(&mut self) {
        match self.focused_panel {
            Panel::Remotes => {
                if self.remotes_selected > 0 {
                    self.remotes_selected -= 1;
                    debug!("Navigate up in remotes: {}", self.remotes_selected);
                }
            }
            Panel::Files => {
                if self.files_selected > 0 {
                    self.files_selected -= 1;
                    debug!("Navigate up in files: {}", self.files_selected);
                }
            }
        }
    }

    /// Switch focus between remotes and files panels.
    pub fn switch_panel(&mut self) {
        self.focused_panel = match self.focused_panel {
            Panel::Remotes => {
                debug!("Switching focus to Files");
                Panel::Files
            }
            Panel::Files => {
                debug!("Switching focus to Remotes");
                Panel::Remotes
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rclone::FileItem;

    fn create_test_client() -> RcloneClient {
        RcloneClient::new("localhost", 5572)
    }

    #[test]
    fn test_app_new() {
        let client = create_test_client();
        let app = App::new(client);

        assert!(app.remotes.is_empty());
        assert!(app.current_remote.is_none());
        assert_eq!(app.current_path, "");
        assert!(app.files.is_empty());
        assert_eq!(app.remotes_selected, 0);
        assert_eq!(app.files_selected, 0);
        assert_eq!(app.focused_panel, Panel::Remotes);
        assert!(app.running);
        assert!(app.create_remote_modal.is_none());
        assert!(app.confirm_modal.is_none());
        assert!(app.pending_delete_remote.is_none());
    }

    #[test]
    fn test_navigate_down_remotes() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.remotes = vec!["remote1".to_string(), "remote2".to_string()];
        app.focused_panel = Panel::Remotes;

        app.navigate_down();
        assert_eq!(app.remotes_selected, 1);

        app.navigate_down();
        assert_eq!(app.remotes_selected, 1); // stays at max
    }

    #[test]
    fn test_navigate_down_files() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.files = vec![
            NavigationItem::File(FileItem {
                name: "file1".to_string(),
                size: 100,
                mod_time: "".to_string(),
                is_dir: false,
            }),
            NavigationItem::File(FileItem {
                name: "file2".to_string(),
                size: 200,
                mod_time: "".to_string(),
                is_dir: false,
            }),
        ];
        app.focused_panel = Panel::Files;

        app.navigate_down();
        assert_eq!(app.files_selected, 1);
    }

    #[test]
    fn test_navigate_up_remotes() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.remotes = vec!["remote1".to_string(), "remote2".to_string()];
        app.remotes_selected = 1;
        app.focused_panel = Panel::Remotes;

        app.navigate_up();
        assert_eq!(app.remotes_selected, 0);

        app.navigate_up();
        assert_eq!(app.remotes_selected, 0); // stays at min
    }

    #[test]
    fn test_navigate_up_files() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.files = vec![NavigationItem::File(FileItem {
            name: "file1".to_string(),
            size: 100,
            mod_time: "".to_string(),
            is_dir: false,
        })];
        app.files_selected = 1;
        app.focused_panel = Panel::Files;

        app.navigate_up();
        assert_eq!(app.files_selected, 0);
    }

    #[test]
    fn test_switch_panel_to_files() {
        let client = create_test_client();
        let mut app = App::new(client);
        assert_eq!(app.focused_panel, Panel::Remotes);

        app.switch_panel();
        assert_eq!(app.focused_panel, Panel::Files);
    }

    #[test]
    fn test_switch_panel_to_remotes() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.focused_panel = Panel::Files;

        app.switch_panel();
        assert_eq!(app.focused_panel, Panel::Remotes);
    }

    #[test]
    fn test_switch_panel_multiple_times() {
        let client = create_test_client();
        let mut app = App::new(client);

        for _ in 0..4 {
            assert_eq!(app.focused_panel, Panel::Remotes);
            app.switch_panel();
            assert_eq!(app.focused_panel, Panel::Files);
            app.switch_panel();
        }
    }
}
