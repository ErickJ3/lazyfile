//! Files list widget.

use crate::rclone::FileItem;
use crate::ui::styles;
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, List, ListItem},
};

/// Widget for displaying files and directories.
pub struct FileListWidget;

impl FileListWidget {
    /// Render the files list widget.
    ///
    /// # Arguments
    /// * `f` - Frame for rendering
    /// * `area` - Area to render in
    /// * `files` - List of navigation items
    /// * `selected` - Index of selected item
    /// * `focused` - Whether this panel is focused
    pub fn render(f: &mut Frame, area: Rect, files: &[FileItem], selected: usize, focused: bool) {
        let items: Vec<ListItem> = files
            .iter()
            .map(|item| {
                let name = if item.is_dir() {
                    format!("[{}]", item.name())
                } else {
                    item.name().to_string()
                };
                ListItem::new(name)
            })
            .collect();

        let border_style = if focused {
            styles::focused_style()
        } else {
            styles::NORMAL_STYLE
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Files ")
                    .border_style(border_style),
            )
            .style(styles::NORMAL_STYLE)
            .highlight_style(styles::selected_style());

        let mut list_state = ratatui::widgets::ListState::default();
        list_state.select(Some(selected));

        f.render_stateful_widget(list, area, &mut list_state);
    }
}
