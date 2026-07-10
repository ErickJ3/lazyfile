//! Terminal layout and area management.

use ratatui::layout::{Constraint, Direction, Rect};

/// Divides terminal into distinct regions.
pub struct Layout;

impl Layout {
    /// Split terminal area into help, content, and status regions.
    ///
    /// Returns `LayoutRects` containing areas for each panel.
    pub fn split(area: Rect) -> LayoutRects {
        let chunks = ratatui::layout::Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(4),
                Constraint::Length(1),
            ])
            .split(area);

        let help_area = chunks[0];
        let content_area = chunks[1];
        let status_area = chunks[2];

        let content_chunks = ratatui::layout::Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(content_area);

        LayoutRects {
            help: help_area,
            remotes: content_chunks[0],
            files: content_chunks[1],
            status: status_area,
        }
    }
}

/// Layout regions for different UI components.
pub struct LayoutRects {
    /// Help text area at top.
    pub help: Rect,
    /// Remotes list area (left).
    pub remotes: Rect,
    /// Files list area (right).
    pub files: Rect,
    /// Status bar area at bottom.
    pub status: Rect,
}
