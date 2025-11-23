//! Keyboard event handling.

use super::state::{App, Panel};
use crate::auth::Credentials;
use crate::error::{LazyFileError, Result};
use crate::ui::{ConfirmModal, CreateRemoteModal, CreateRemoteMode, LoginField, LoginModal};
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
        // If login modal is open, handle it with priority
        if app.login_modal.is_some() {
            return Self::handle_login_key(app, key).await;
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
                    match app.load_files().await {
                        Ok(_) => {
                            app.focused_panel = Panel::Files;
                        }
                        Err(LazyFileError::Unauthorized) => {
                            debug!("Authentication required to access remote");
                            app.login_modal = Some(LoginModal::new_basic());
                            app.current_remote = None;
                        }
                        Err(e) => return Err(e),
                    }
                }
            }
            Panel::Files => {
                if let Some(item) = app.files.get(app.files_selected)
                    && item.is_dir()
                {
                    let name = item.name();
                    debug!("Opening directory: {}", name);
                    if app.current_path.is_empty() {
                        app.current_path = format!("/{}", name);
                    } else {
                        app.current_path = format!("{}/{}", app.current_path, name);
                    }
                    match app.load_files().await {
                        Ok(_) => {}
                        Err(LazyFileError::Unauthorized) => {
                            debug!("Authentication required to access directory");
                            app.login_modal = Some(LoginModal::new_basic());
                            // Revert path change
                            if let Some(last_slash) = app.current_path.rfind('/') {
                                app.current_path.truncate(last_slash);
                            } else {
                                app.current_path.clear();
                            }
                        }
                        Err(e) => return Err(e),
                    }
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

    /// Handle keyboard input while login modal is open.
    async fn handle_login_key(app: &mut App, key: KeyEvent) -> Result<()> {
        if let Some(ref mut modal) = app.login_modal {
            match key.code {
                KeyCode::Esc => {
                    debug!("Closing login modal");
                    app.login_modal = None;
                }
                KeyCode::Tab => {
                    modal.next_field();
                }
                KeyCode::BackTab => {
                    modal.prev_field();
                }
                KeyCode::Char('l') if matches!(modal.focus_field, LoginField::Password) => {
                    // Toggle password masking with 'l'
                    modal.toggle_password_visibility();
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
                    Self::handle_login_submit(app).await?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Handle login modal submission.
    async fn handle_login_submit(app: &mut App) -> Result<()> {
        if let Some(ref modal) = app.login_modal.clone() {
            if !modal.is_valid() {
                if let Some(ref mut login_modal) = app.login_modal {
                    login_modal.error = Some("All fields are required".to_string());
                }
                return Ok(());
            }

            // Create credentials from modal input
            let credentials = match modal.auth_type {
                crate::auth::CredentialsType::Basic => {
                    Credentials::basic(modal.username.clone(), modal.password.clone(), None)
                }
                crate::auth::CredentialsType::Bearer => {
                    Credentials::bearer(modal.password.clone(), None)
                }
            };

            // Set credentials in client
            app.client.set_credentials(credentials.clone());

            // Try to set in auth manager
            if let Err(e) = app.auth_manager.set_daemon_credentials(credentials) {
                if let Some(ref mut login_modal) = app.login_modal {
                    login_modal.error = Some(format!("Failed to save credentials: {}", e));
                }
                return Ok(());
            }

            // Try to reload remotes to verify auth
            match app.load_remotes().await {
                Ok(_) => {
                    info!("Authentication successful");
                    app.login_modal = None;
                }
                Err(e) => {
                    if let Some(ref mut login_modal) = app.login_modal {
                        login_modal.error = Some(format!("Authentication failed: {}", e));
                    }
                    // Clear credentials on auth failure
                    app.client.clear_credentials();
                }
            }
        }
        Ok(())
    }
}
