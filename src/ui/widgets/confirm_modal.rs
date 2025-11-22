//! Confirmation modal widget for delete operations.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph},
};

/// Confirmation modal state.
#[derive(Debug, Clone)]
pub struct ConfirmModal {
    pub title: String,
    pub message: String,
    pub selected: ConfirmChoice,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfirmChoice {
    Yes,
    No,
}

impl ConfirmModal {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            selected: ConfirmChoice::No, // Default to No for safety
        }
    }

    pub fn toggle(&mut self) {
        self.selected = match self.selected {
            ConfirmChoice::Yes => ConfirmChoice::No,
            ConfirmChoice::No => ConfirmChoice::Yes,
        };
    }

    pub fn is_confirmed(&self) -> bool {
        self.selected == ConfirmChoice::Yes
    }
}

pub struct ConfirmWidget;

impl ConfirmWidget {
    pub fn render(f: &mut Frame, area: Rect, modal: &ConfirmModal) {
        // Render backdrop
        f.render_widget(Clear, area);
        f.render_widget(
            Block::default().style(Style::default().bg(Color::DarkGray)),
            area,
        );

        // Calculate compact modal size
        let modal_width = 45.min(area.width.saturating_sub(4));
        let modal_height = 9;
        let x = (area.width.saturating_sub(modal_width)) / 2 + area.x;
        let y = (area.height.saturating_sub(modal_height)) / 2 + area.y;

        let modal_area = Rect {
            x,
            y,
            width: modal_width,
            height: modal_height,
        };

        // Clear and draw modal border
        f.render_widget(Clear, modal_area);
        let block = Block::default()
            .title(format!(" {} ", modal.title))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red));
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
                Constraint::Min(2),
                Constraint::Length(2),
                Constraint::Length(1),
            ])
            .split(inner);

        // Message
        let message = Paragraph::new(modal.message.as_str());
        f.render_widget(message, chunks[0]);

        // Buttons
        let button_area = chunks[1];
        let button_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(button_area);

        let yes_style = if modal.selected == ConfirmChoice::Yes {
            Style::default().fg(Color::Black).bg(Color::Red).bold()
        } else {
            Style::default().fg(Color::Red)
        };

        let no_style = if modal.selected == ConfirmChoice::No {
            Style::default().fg(Color::Black).bg(Color::Yellow).bold()
        } else {
            Style::default().fg(Color::Yellow)
        };

        let yes_btn = Paragraph::new(" Yes ")
            .style(yes_style)
            .alignment(Alignment::Center);
        let no_btn = Paragraph::new(" No ")
            .style(no_style)
            .alignment(Alignment::Center);

        f.render_widget(yes_btn, button_chunks[0]);
        f.render_widget(no_btn, button_chunks[1]);

        // Help text
        let help = Paragraph::new("Tab: Switch | Enter: Confirm | Esc: Cancel")
            .style(Style::default().fg(Color::Gray));
        f.render_widget(help, chunks[2]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirm_modal_new() {
        let modal = ConfirmModal::new("Delete", "Delete item?");
        assert_eq!(modal.title, "Delete");
        assert_eq!(modal.message, "Delete item?");
        assert_eq!(modal.selected, ConfirmChoice::No);
    }

    #[test]
    fn test_confirm_modal_default_is_no() {
        let modal = ConfirmModal::new("Delete", "Are you sure?");
        assert!(!modal.is_confirmed());
        assert_eq!(modal.selected, ConfirmChoice::No);
    }

    #[test]
    fn test_toggle_yes_to_no() {
        let mut modal = ConfirmModal::new("Test", "Test?");
        modal.selected = ConfirmChoice::Yes;

        modal.toggle();
        assert_eq!(modal.selected, ConfirmChoice::No);
        assert!(!modal.is_confirmed());
    }

    #[test]
    fn test_toggle_no_to_yes() {
        let mut modal = ConfirmModal::new("Test", "Test?");
        assert_eq!(modal.selected, ConfirmChoice::No);

        modal.toggle();
        assert_eq!(modal.selected, ConfirmChoice::Yes);
        assert!(modal.is_confirmed());
    }

    #[test]
    fn test_is_confirmed_yes() {
        let mut modal = ConfirmModal::new("Test", "Test?");
        modal.selected = ConfirmChoice::Yes;
        assert!(modal.is_confirmed());
    }

    #[test]
    fn test_is_confirmed_no() {
        let modal = ConfirmModal::new("Test", "Test?");
        assert!(!modal.is_confirmed());
    }

    #[test]
    fn test_toggle_multiple_times() {
        let mut modal = ConfirmModal::new("Test", "Test?");

        for _ in 0..4 {
            assert_eq!(modal.selected, ConfirmChoice::No);
            modal.toggle();
            assert_eq!(modal.selected, ConfirmChoice::Yes);
            modal.toggle();
        }
    }

    #[test]
    fn test_string_conversion() {
        let modal = ConfirmModal::new(
            "Delete Remote".to_string(),
            "Delete 'myremote'?".to_string(),
        );
        assert_eq!(modal.title, "Delete Remote");
        assert_eq!(modal.message, "Delete 'myremote'?");
    }

    #[test]
    fn test_str_conversion() {
        let modal = ConfirmModal::new("Test", "Test message");
        assert_eq!(modal.title, "Test");
        assert_eq!(modal.message, "Test message");
    }
}
