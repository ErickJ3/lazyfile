//! Create/Edit remote modal widget.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph},
};

/// Modal state for creating/editing remotes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CreateRemoteMode {
    /// Creating a new remote
    Create,
    /// Editing an existing remote
    Edit,
}

/// Create/Edit remote modal state.
#[derive(Debug, Clone)]
pub struct CreateRemoteModal {
    pub mode: CreateRemoteMode,
    pub name: String,
    pub remote_type: String,
    pub path: String,
    pub focus_field: RemoteField,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RemoteField {
    Name,
    Type,
    Path,
}

impl CreateRemoteModal {
    pub fn new(mode: CreateRemoteMode) -> Self {
        Self {
            mode,
            name: String::new(),
            remote_type: String::from("local"),
            path: String::new(),
            focus_field: RemoteField::Name,
            error: None,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_type(mut self, remote_type: String) -> Self {
        self.remote_type = remote_type;
        self
    }

    pub fn next_field(&mut self) {
        self.focus_field = match self.focus_field {
            RemoteField::Name => RemoteField::Type,
            RemoteField::Type => RemoteField::Path,
            RemoteField::Path => RemoteField::Name,
        };
    }

    pub fn prev_field(&mut self) {
        self.focus_field = match self.focus_field {
            RemoteField::Name => RemoteField::Path,
            RemoteField::Type => RemoteField::Name,
            RemoteField::Path => RemoteField::Type,
        };
    }

    pub fn input_char(&mut self, c: char) {
        match self.focus_field {
            RemoteField::Name => self.name.push(c),
            RemoteField::Type => self.remote_type.push(c),
            RemoteField::Path => self.path.push(c),
        }
    }

    pub fn backspace(&mut self) {
        match self.focus_field {
            RemoteField::Name => {
                self.name.pop();
            }
            RemoteField::Type => {
                self.remote_type.pop();
            }
            RemoteField::Path => {
                self.path.pop();
            }
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && !self.remote_type.is_empty()
    }
}

pub struct CreateRemoteWidget;

impl CreateRemoteWidget {
    pub fn render(f: &mut Frame, area: Rect, modal: &CreateRemoteModal) {
        // Render backdrop to darken background
        let backdrop = Block::default().style(Style::default().bg(Color::Black));
        f.render_widget(Clear, area);
        f.render_widget(backdrop.style(Style::default().bg(Color::DarkGray)), area);

        // Calculate compact modal size (much smaller)
        let modal_width = 50.min(area.width.saturating_sub(4));
        let modal_height = 13; // Compact: title + 3 fields + help
        let x = (area.width.saturating_sub(modal_width)) / 2 + area.x;
        let y = (area.height.saturating_sub(modal_height)) / 2 + area.y;

        let modal_area = Rect {
            x,
            y,
            width: modal_width,
            height: modal_height,
        };

        // Clear the modal area
        f.render_widget(Clear, modal_area);

        let block = Block::default()
            .title(match modal.mode {
                CreateRemoteMode::Create => " Create Remote ",
                CreateRemoteMode::Edit => " Edit Remote ",
            })
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        f.render_widget(block, modal_area);

        let inner = Rect {
            x: modal_area.x + 1,
            y: modal_area.y + 1,
            width: modal_area.width.saturating_sub(2),
            height: modal_area.height.saturating_sub(2),
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Min(1),
            ])
            .split(inner);

        // Name field
        Self::render_field(
            f,
            chunks[0],
            "Name",
            &modal.name,
            modal.focus_field == RemoteField::Name,
        );

        // Type field
        Self::render_field(
            f,
            chunks[1],
            "Type",
            &modal.remote_type,
            modal.focus_field == RemoteField::Type,
        );

        // Path field
        Self::render_field(
            f,
            chunks[2],
            "Path",
            &modal.path,
            modal.focus_field == RemoteField::Path,
        );

        // Error or help text (single line, smaller font)
        let help_text = if let Some(ref error) = modal.error {
            error.clone()
        } else {
            "Tab: Next | Enter: Save | Esc: Cancel".to_string()
        };

        let style = if modal.error.is_some() {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Gray)
        };

        let help = Paragraph::new(help_text).style(style);
        f.render_widget(help, chunks[3]);
    }

    fn render_field(f: &mut Frame, area: Rect, label: &str, value: &str, focused: bool) {
        let value_display = if value.len() > 30 {
            format!("{}...", &value[..27])
        } else {
            value.to_string()
        };

        let text = format!("{}: {}", label, value_display);
        let paragraph = Paragraph::new(text).style(if focused {
            Style::default().fg(Color::Yellow).bold()
        } else {
            Style::default()
        });

        f.render_widget(paragraph, area);

        // Minimal bottom border for focused field
        if focused && area.height > 1 {
            let bottom_line = "─".repeat(area.width as usize);
            let bottom = Paragraph::new(bottom_line).style(Style::default().fg(Color::Cyan));
            let bottom_area = Rect {
                y: area.y + 1,
                height: 1,
                ..area
            };
            f.render_widget(bottom, bottom_area);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_remote_modal_new() {
        let modal = CreateRemoteModal::new(CreateRemoteMode::Create);
        assert_eq!(modal.mode, CreateRemoteMode::Create);
        assert!(modal.name.is_empty());
        assert_eq!(modal.remote_type, "local");
        assert!(modal.path.is_empty());
        assert_eq!(modal.focus_field, RemoteField::Name);
        assert!(modal.error.is_none());
    }

    #[test]
    fn test_edit_mode_creation() {
        let modal = CreateRemoteModal::new(CreateRemoteMode::Edit);
        assert_eq!(modal.mode, CreateRemoteMode::Edit);
    }

    #[test]
    fn test_with_builders() {
        let modal = CreateRemoteModal::new(CreateRemoteMode::Create)
            .with_name("myremote".to_string())
            .with_type("s3".to_string());

        assert_eq!(modal.name, "myremote");
        assert_eq!(modal.remote_type, "s3");
    }

    #[test]
    fn test_field_navigation() {
        let mut modal = CreateRemoteModal::new(CreateRemoteMode::Create);
        assert_eq!(modal.focus_field, RemoteField::Name);

        modal.next_field();
        assert_eq!(modal.focus_field, RemoteField::Type);

        modal.next_field();
        assert_eq!(modal.focus_field, RemoteField::Path);

        modal.next_field();
        assert_eq!(modal.focus_field, RemoteField::Name);
    }

    #[test]
    fn test_prev_field_navigation() {
        let mut modal = CreateRemoteModal::new(CreateRemoteMode::Create);
        modal.focus_field = RemoteField::Type;

        modal.prev_field();
        assert_eq!(modal.focus_field, RemoteField::Name);

        modal.prev_field();
        assert_eq!(modal.focus_field, RemoteField::Path);
    }

    #[test]
    fn test_input_char_to_name() {
        let mut modal = CreateRemoteModal::new(CreateRemoteMode::Create);
        modal.focus_field = RemoteField::Name;

        modal.input_char('t');
        assert_eq!(modal.name, "t");

        modal.input_char('e');
        modal.input_char('s');
        modal.input_char('t');
        assert_eq!(modal.name, "test");
    }

    #[test]
    fn test_input_char_to_type() {
        let mut modal = CreateRemoteModal::new(CreateRemoteMode::Create);
        modal.focus_field = RemoteField::Type;
        modal.remote_type.clear();

        modal.input_char('s');
        modal.input_char('3');
        assert_eq!(modal.remote_type, "s3");
    }

    #[test]
    fn test_backspace() {
        let mut modal = CreateRemoteModal::new(CreateRemoteMode::Create);
        modal.focus_field = RemoteField::Name;
        modal.name = "test".to_string();

        modal.backspace();
        assert_eq!(modal.name, "tes");

        modal.backspace();
        assert_eq!(modal.name, "te");
    }

    #[test]
    fn test_backspace_empty_string() {
        let mut modal = CreateRemoteModal::new(CreateRemoteMode::Create);
        modal.focus_field = RemoteField::Name;

        modal.backspace();
        assert!(modal.name.is_empty());
    }

    #[test]
    fn test_is_valid() {
        let mut modal = CreateRemoteModal::new(CreateRemoteMode::Create);
        assert!(!modal.is_valid()); // name is empty

        modal.name = "myremote".to_string();
        assert!(modal.is_valid()); // now valid

        modal.remote_type.clear();
        assert!(!modal.is_valid()); // type is empty
    }

    #[test]
    fn test_is_valid_requires_both_fields() {
        let modal =
            CreateRemoteModal::new(CreateRemoteMode::Create).with_name("myremote".to_string());
        assert!(modal.is_valid());

        let modal = CreateRemoteModal::new(CreateRemoteMode::Create).with_type("s3".to_string());
        let mut modal = modal;
        modal.name.clear();
        assert!(!modal.is_valid());
    }
}
