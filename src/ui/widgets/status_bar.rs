//! Status bar widget.

use crate::ui::styles;
use ratatui::{prelude::*, widgets::Paragraph};

/// Widget for displaying application status.
pub struct StatusBarWidget;

impl StatusBarWidget {
    /// Render the status bar widget.
    ///
    /// # Arguments
    /// * `f` - Frame for rendering
    /// * `area` - Area to render in
    /// * `remote` - Currently selected remote
    /// * `path` - Current path within remote
    /// * `connected` - Connection status
    pub fn render(f: &mut Frame, area: Rect, remote: Option<&str>, path: &str, connected: bool) {
        let status = if connected {
            "Connected"
        } else {
            "Disconnected"
        };

        let display_path = if let Some(r) = remote {
            if path.is_empty() {
                format!("{}:", r)
            } else {
                format!("{}:{}", r, path)
            }
        } else {
            "Select a remote".to_string()
        };

        let text = format!("  {} | {}  ", display_path, status);
        let paragraph = Paragraph::new(text).style(styles::status_bar_style());
        f.render_widget(paragraph, area);
    }
}
