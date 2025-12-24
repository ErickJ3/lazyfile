//! Application state management.

use crate::error::Result;
use crate::rclone::{NavigationItem, RcloneClient};
use crate::ui::{ConfirmModal, CreateRemoteModal, FileOperationsModal};
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
    /// Modal for file operations.
    pub file_operations_modal: Option<FileOperationsModal>,
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
            file_operations_modal: None,
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
        assert!(app.file_operations_modal.is_none());
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

    #[test]
    fn test_panel_equality() {
        assert_eq!(Panel::Remotes, Panel::Remotes);
        assert_eq!(Panel::Files, Panel::Files);
        assert_ne!(Panel::Remotes, Panel::Files);
    }

    #[test]
    fn test_panel_clone() {
        let panel = Panel::Remotes;
        let cloned = panel.clone();
        assert_eq!(panel, cloned);
    }

    #[test]
    fn test_panel_copy() {
        let panel = Panel::Files;
        let copied: Panel = panel;
        assert_eq!(panel, copied);
    }

    #[test]
    fn test_navigate_down_empty_remotes() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.remotes = vec![];
        app.focused_panel = Panel::Remotes;

        app.navigate_down();
        assert_eq!(app.remotes_selected, 0);
    }

    #[test]
    fn test_navigate_up_empty_remotes() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.remotes = vec![];
        app.focused_panel = Panel::Remotes;

        app.navigate_up();
        assert_eq!(app.remotes_selected, 0);
    }

    #[test]
    fn test_navigate_down_empty_files() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.files = vec![];
        app.focused_panel = Panel::Files;

        app.navigate_down();
        assert_eq!(app.files_selected, 0);
    }

    #[test]
    fn test_navigate_up_empty_files() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.files = vec![];
        app.focused_panel = Panel::Files;

        app.navigate_up();
        assert_eq!(app.files_selected, 0);
    }

    #[test]
    fn test_navigate_down_single_item() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.remotes = vec!["single".to_string()];
        app.focused_panel = Panel::Remotes;

        app.navigate_down();
        assert_eq!(app.remotes_selected, 0);
    }

    #[test]
    fn test_navigate_up_single_item() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.remotes = vec!["single".to_string()];
        app.focused_panel = Panel::Remotes;

        app.navigate_up();
        assert_eq!(app.remotes_selected, 0);
    }

    #[test]
    fn test_navigate_down_boundary() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.remotes = vec!["r1".to_string(), "r2".to_string(), "r3".to_string()];
        app.remotes_selected = 2;
        app.focused_panel = Panel::Remotes;

        app.navigate_down();
        assert_eq!(app.remotes_selected, 2);
    }

    #[test]
    fn test_navigate_up_boundary() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.remotes = vec!["r1".to_string(), "r2".to_string(), "r3".to_string()];
        app.remotes_selected = 0;
        app.focused_panel = Panel::Remotes;

        app.navigate_up();
        assert_eq!(app.remotes_selected, 0);
    }

    #[test]
    fn test_navigate_many_items() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.remotes = (0..100).map(|i| format!("remote_{}", i)).collect();
        app.focused_panel = Panel::Remotes;

        for _ in 0..50 {
            app.navigate_down();
        }
        assert_eq!(app.remotes_selected, 50);

        for _ in 0..25 {
            app.navigate_up();
        }
        assert_eq!(app.remotes_selected, 25);
    }

    #[test]
    fn test_navigate_files_many_items() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.files = (0..100)
            .map(|i| {
                NavigationItem::File(FileItem {
                    name: format!("file_{}.txt", i),
                    size: i * 100,
                    mod_time: "".to_string(),
                    is_dir: false,
                })
            })
            .collect();
        app.focused_panel = Panel::Files;

        for _ in 0..150 {
            app.navigate_down();
        }
        assert_eq!(app.files_selected, 99);

        for _ in 0..150 {
            app.navigate_up();
        }
        assert_eq!(app.files_selected, 0);
    }

    #[test]
    fn test_app_running_flag() {
        let client = create_test_client();
        let mut app = App::new(client);
        assert!(app.running);

        app.running = false;
        assert!(!app.running);
    }

    #[test]
    fn test_app_current_remote_none_initially() {
        let client = create_test_client();
        let app = App::new(client);
        assert!(app.current_remote.is_none());
    }

    #[test]
    fn test_app_current_path_empty_initially() {
        let client = create_test_client();
        let app = App::new(client);
        assert!(app.current_path.is_empty());
    }

    #[test]
    fn test_app_modals_none_initially() {
        let client = create_test_client();
        let app = App::new(client);
        assert!(app.create_remote_modal.is_none());
        assert!(app.confirm_modal.is_none());
        assert!(app.file_operations_modal.is_none());
        assert!(app.pending_delete_remote.is_none());
    }

    #[test]
    fn test_set_current_remote() {
        let client = create_test_client();
        let mut app = App::new(client);

        app.current_remote = Some("gdrive".to_string());
        assert_eq!(app.current_remote, Some("gdrive".to_string()));

        app.current_remote = None;
        assert!(app.current_remote.is_none());
    }

    #[test]
    fn test_set_current_path() {
        let client = create_test_client();
        let mut app = App::new(client);

        app.current_path = "/some/path".to_string();
        assert_eq!(app.current_path, "/some/path");

        app.current_path = String::new();
        assert!(app.current_path.is_empty());
    }

    #[test]
    fn test_add_remotes() {
        let client = create_test_client();
        let mut app = App::new(client);

        app.remotes.push("remote1".to_string());
        app.remotes.push("remote2".to_string());

        assert_eq!(app.remotes.len(), 2);
        assert_eq!(app.remotes[0], "remote1");
        assert_eq!(app.remotes[1], "remote2");
    }

    #[test]
    fn test_add_files() {
        let client = create_test_client();
        let mut app = App::new(client);

        app.files.push(NavigationItem::File(FileItem {
            name: "file1.txt".to_string(),
            size: 100,
            mod_time: "".to_string(),
            is_dir: false,
        }));

        assert_eq!(app.files.len(), 1);
        assert_eq!(app.files[0].name(), "file1.txt");
    }

    #[test]
    fn test_remotes_selected_initial() {
        let client = create_test_client();
        let app = App::new(client);
        assert_eq!(app.remotes_selected, 0);
    }

    #[test]
    fn test_files_selected_initial() {
        let client = create_test_client();
        let app = App::new(client);
        assert_eq!(app.files_selected, 0);
    }

    #[test]
    fn test_selection_persists_after_switch() {
        let client = create_test_client();
        let mut app = App::new(client);
        app.remotes = vec!["r1".to_string(), "r2".to_string(), "r3".to_string()];
        app.files = vec![
            NavigationItem::File(FileItem {
                name: "f1".to_string(),
                size: 0,
                mod_time: "".to_string(),
                is_dir: false,
            }),
            NavigationItem::File(FileItem {
                name: "f2".to_string(),
                size: 0,
                mod_time: "".to_string(),
                is_dir: false,
            }),
        ];

        app.focused_panel = Panel::Remotes;
        app.navigate_down();
        app.navigate_down();
        assert_eq!(app.remotes_selected, 2);

        app.switch_panel();
        app.navigate_down();
        assert_eq!(app.files_selected, 1);

        app.switch_panel();
        assert_eq!(app.remotes_selected, 2);
    }

    #[test]
    fn test_file_item_size() {
        let file = FileItem {
            name: "large_file.bin".to_string(),
            size: 1024 * 1024 * 1024,
            mod_time: "2024-01-01T00:00:00Z".to_string(),
            is_dir: false,
        };

        assert_eq!(file.size, 1024 * 1024 * 1024);
    }

    #[test]
    fn test_file_item_negative_size() {
        let file = FileItem {
            name: "unknown_size.txt".to_string(),
            size: -1,
            mod_time: "".to_string(),
            is_dir: false,
        };

        assert_eq!(file.size, -1);
    }

    #[test]
    fn test_file_item_mod_time() {
        let file = FileItem {
            name: "file.txt".to_string(),
            size: 0,
            mod_time: "2024-12-24T15:30:00Z".to_string(),
            is_dir: false,
        };

        assert_eq!(file.mod_time, "2024-12-24T15:30:00Z");
    }

    #[test]
    fn test_navigation_item_directory() {
        let dir = NavigationItem::File(FileItem {
            name: "documents".to_string(),
            size: 0,
            mod_time: "".to_string(),
            is_dir: true,
        });

        assert!(dir.is_dir());
        assert_eq!(dir.name(), "documents");
    }

    #[test]
    fn test_navigation_item_file() {
        let file = NavigationItem::File(FileItem {
            name: "readme.md".to_string(),
            size: 1024,
            mod_time: "".to_string(),
            is_dir: false,
        });

        assert!(!file.is_dir());
        assert_eq!(file.name(), "readme.md");
    }

    #[test]
    fn test_navigation_item_with_special_name() {
        let item = NavigationItem::File(FileItem {
            name: "file with spaces & special chars!.txt".to_string(),
            size: 0,
            mod_time: "".to_string(),
            is_dir: false,
        });

        assert_eq!(item.name(), "file with spaces & special chars!.txt");
    }

    #[test]
    fn test_navigation_item_unicode_name() {
        let item = NavigationItem::File(FileItem {
            name: "файл_日本語.txt".to_string(),
            size: 0,
            mod_time: "".to_string(),
            is_dir: false,
        });

        assert_eq!(item.name(), "файл_日本語.txt");
    }
}
