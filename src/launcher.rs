//! Application init

use crate::app::{App, Handler};
use crate::error::{LazyFileError, Result};
use crate::ui::{Layout, LoginModal, LoginModalWidget};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;
use std::io;

/// Initialize
fn setup_terminal() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    Ok(())
}

/// Restore to normal state
fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

/// Main
async fn run_app(app: &mut App) -> Result<()> {
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    while app.running {
        terminal.draw(|f| ui_render(f, app))?;

        if crossterm::event::poll(std::time::Duration::from_millis(200))?
            && let Event::Key(key) = event::read()?
        {
            Handler::handle_key(app, key).await?;
        }
    }

    tracing::debug!("Application exiting");
    Ok(())
}

/// Render the UI frame.
fn ui_render(f: &mut Frame, app: &App) {
    let rects = Layout::split(f.area());

    crate::ui::HelpWidget::render(f, rects.help);

    crate::ui::RemoteListWidget::render(
        f,
        rects.remotes,
        &app.remotes,
        app.remotes_selected,
        matches!(app.focused_panel, crate::app::state::Panel::Remotes),
    );

    crate::ui::FileListWidget::render(
        f,
        rects.files,
        &app.files,
        app.files_selected,
        matches!(app.focused_panel, crate::app::state::Panel::Files),
    );

    crate::ui::StatusBarWidget::render(
        f,
        rects.status,
        app.current_remote.as_deref(),
        &app.current_path,
        true,
    );

    // Render confirmation modal if open
    if let Some(ref modal) = app.confirm_modal {
        crate::ui::ConfirmWidget::render(f, f.area(), modal);
    }

    // Render create/edit modal if open
    if let Some(ref modal) = app.create_remote_modal {
        crate::ui::CreateRemoteWidget::render(f, f.area(), modal);
    }

    // Render login modal if open
    if let Some(ref modal) = app.login_modal {
        LoginModalWidget::render(modal, f.area(), f.buffer_mut());
    }
}

/// Start app.
pub async fn start(mut app: App) -> Result<()> {
    setup_terminal()?;

    // Try to load remotes initially
    match app.load_remotes().await {
        Ok(_) => {
            tracing::debug!("Remotes loaded successfully");
        }
        Err(LazyFileError::Unauthorized) => {
            tracing::debug!("Authentication required to load remotes");
            app.login_modal = Some(LoginModal::new_basic());
        }
        Err(e) => {
            if app.auth_manager.should_require_auth_on_startup() {
                app.login_modal = Some(LoginModal::new_basic());
            } else {
                restore_terminal()?;
                return Err(e);
            }
        }
    }

    let res = run_app(&mut app).await;
    restore_terminal()?;

    res
}
