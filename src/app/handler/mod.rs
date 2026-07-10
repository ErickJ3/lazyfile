//! Keyboard event handling.

mod file_ops;
mod navigation;
mod remote_modal;

use super::state::{ActiveModal, App, Panel};
use crate::error::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use tracing::{debug, info};

/// Handles keyboard input events.
pub struct Handler;

impl Handler {
    /// Processes a keyboard event and updates app state.
    ///
    /// # Errors
    /// Returns error if rclone API calls fail.
    pub async fn handle_key(app: &mut App, key: KeyEvent) -> Result<()> {
        // Terminals with key-release reporting (kitty protocol, Windows
        // console) deliver Release/Repeat events; acting on them would
        // fire every keybinding twice.
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }

        match app.modal {
            Some(ActiveModal::FileOperation(_)) => {
                return Self::handle_file_operations_key(app, key).await;
            }
            Some(ActiveModal::ConfirmDeleteRemote { .. }) => {
                return Self::handle_confirm_key(app, key).await;
            }
            Some(ActiveModal::CreateRemote(_)) => {
                return Self::handle_modal_key(app, key).await;
            }
            None => {}
        }

        match key.code {
            KeyCode::Char('q') => {
                info!("quit requested");
                app.running = false;
            }
            KeyCode::Char('a') if matches!(app.focused_panel, Panel::Remotes) => {
                debug!("opening create remote modal");
                app.modal = Some(ActiveModal::CreateRemote(
                    crate::ui::CreateRemoteModal::new(crate::ui::CreateRemoteMode::Create),
                ));
            }
            KeyCode::Char('e') if matches!(app.focused_panel, Panel::Remotes) => {
                Self::handle_edit_remote(app).await?;
            }
            KeyCode::Char('d') if matches!(app.focused_panel, Panel::Remotes) => {
                Self::handle_delete_remote(app);
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
}
