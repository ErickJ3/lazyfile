//! Remotes list widget.

use crate::ui::styles;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem},
};

/// Widget for displaying list of remotes.
pub struct RemoteListWidget;

impl RemoteListWidget {
    /// Render the remotes list widget.
    ///
    /// # Arguments
    /// * `f` - Frame for rendering
    /// * `area` - Area to render in
    /// * `remotes` - List of remote names
    /// * `selected` - Index of selected remote
    /// * `focused` - Whether this panel is focused
    pub fn render(f: &mut Frame, area: Rect, remotes: &[String], selected: usize, focused: bool) {
        let items: Vec<ListItem> = remotes.iter().map(|r| ListItem::new(r.as_str())).collect();

        let border_style = if focused {
            styles::focused_style()
        } else {
            styles::NORMAL_STYLE
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Remotes ")
                    .border_style(border_style),
            )
            .style(styles::NORMAL_STYLE)
            .highlight_style(styles::selected_style());

        let mut list_state = ratatui::widgets::ListState::default();
        list_state.select(Some(selected));

        f.render_stateful_widget(list, area, &mut list_state);
    }
}
