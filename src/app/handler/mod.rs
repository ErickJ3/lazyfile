//! Keyboard event handling.

mod file_ops;
mod navigation;
mod remote_modal;

use super::state::{App, Panel};
use crate::error::Result;
use crossterm::event::{KeyCode, KeyEvent};
use tracing::{debug, info};

/// Handles keyboard input events.
pub struct Handler;

impl Handler {
    /// Processes a keyboard event and updates app state.
    ///
    /// # Errors
    /// Returns error if rclone API calls fail.
    pub async fn handle_key(app: &mut App, key: KeyEvent) -> Result<()> {
        if app.file_operations_modal.is_some() {
            return Self::handle_file_operations_key(app, key).await;
        }

        if app.confirm_modal.is_some() {
            return Self::handle_confirm_key(app, key).await;
        }

        if app.create_remote_modal.is_some() {
            return Self::handle_modal_key(app, key).await;
        }

        match key.code {
            KeyCode::Char('q') => {
                info!("quit requested");
                app.running = false;
            }
            KeyCode::Char('a') if matches!(app.focused_panel, Panel::Remotes) => {
                debug!("opening create remote modal");
                app.create_remote_modal = Some(crate::ui::CreateRemoteModal::new(
                    crate::ui::CreateRemoteMode::Create,
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
