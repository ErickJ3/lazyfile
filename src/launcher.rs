//! Application init

use crate::app::{App, Handler};
use crate::error::Result;
use crate::ui::Layout;
use crossterm::event::{self, Event};
use ratatui::{DefaultTerminal, Frame};

/// Main
async fn run_app(terminal: &mut DefaultTerminal, app: &mut App) -> Result<()> {
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
        app.connected,
    );

    if let Some(ref modal) = app.modal {
        match modal {
            crate::app::ActiveModal::FileOperation(m) => {
                crate::ui::FileOperationsWidget::render(f, f.area(), m);
            }
            crate::app::ActiveModal::ConfirmDeleteRemote { modal: m, .. } => {
                crate::ui::ConfirmWidget::render(f, f.area(), m);
            }
            crate::app::ActiveModal::CreateRemote(m) => {
                crate::ui::CreateRemoteWidget::render(f, f.area(), m);
            }
        }
    }
}

/// Start app.
pub async fn start(mut app: App) -> Result<()> {
    // try_init/try_restore keep setup errors in the Result chain
    // instead of panicking. Mouse capture is intentionally not
    // enabled so the terminal keeps native text selection.
    let mut terminal = ratatui::try_init()?;
    let res = run_app(&mut terminal, &mut app).await;
    let restored = ratatui::try_restore();

    // An app error takes precedence over a restore error.
    res.and(restored.map_err(Into::into))
}
