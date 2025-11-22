//! LazyFile - TUI file manager for cloud storage using rclone.

mod app;
mod config;
mod error;
mod rclone;
mod ui;

use app::{App, Handler};
use config::{RCLONE_HOST, RCLONE_PORT};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;
use rclone::RcloneClient;
use std::io;
use ui::Layout;

#[tokio::main]
async fn main() -> error::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    tracing::debug!("Starting LazyFile");

    let client = RcloneClient::new(RCLONE_HOST, RCLONE_PORT);
    let mut app = App::new(client);
    app.load_remotes().await?;

    setup_terminal()?;
    let res = run_app(&mut app).await;
    restore_terminal()?;

    res
}

/// Initialize terminal for TUI.
fn setup_terminal() -> error::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    Ok(())
}

/// Restore terminal to normal state.
fn restore_terminal() -> error::Result<()> {
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

/// Main application loop.
async fn run_app(app: &mut App) -> error::Result<()> {
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

    ui::HelpWidget::render(f, rects.help);

    ui::RemoteListWidget::render(
        f,
        rects.remotes,
        &app.remotes,
        app.remotes_selected,
        matches!(app.focused_panel, app::state::Panel::Remotes),
    );

    ui::FileListWidget::render(
        f,
        rects.files,
        &app.files,
        app.files_selected,
        matches!(app.focused_panel, app::state::Panel::Files),
    );

    ui::StatusBarWidget::render(
        f,
        rects.status,
        app.current_remote.as_deref(),
        &app.current_path,
        true,
    );

    // Render confirmation modal if open
    if let Some(ref modal) = app.confirm_modal {
        ui::ConfirmWidget::render(f, f.area(), modal);
    }

    // Render create/edit modal if open
    if let Some(ref modal) = app.create_remote_modal {
        ui::CreateRemoteWidget::render(f, f.area(), modal);
    }
}
