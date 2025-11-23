//! Login modal widget for authentication.

use crate::auth::CredentialsType;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph},
};

/// Login modal state for rclone RC authentication.
#[derive(Debug, Clone)]
pub struct LoginModal {
    pub auth_type: CredentialsType,
    pub username: String,
    pub password: String,
    pub focus_field: LoginField,
    pub error: Option<String>,
    pub is_password_masked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginField {
    /// For Basic Auth
    Username,
    /// For Basic Auth or Bearer Token
    Password,
    /// Placeholder for Bearer auth (no username)
    Token,
}

impl LoginModal {
    /// Create a new login modal for Basic Auth.
    pub fn new_basic() -> Self {
        Self {
            auth_type: CredentialsType::Basic,
            username: String::new(),
            password: String::new(),
            focus_field: LoginField::Username,
            error: None,
            is_password_masked: true,
        }
    }

    /// Create a new login modal for Bearer Token auth.
    #[allow(dead_code)]
    pub fn new_bearer() -> Self {
        Self {
            auth_type: CredentialsType::Bearer,
            username: String::new(),
            password: String::new(),
            focus_field: LoginField::Token,
            error: None,
            is_password_masked: false,
        }
    }

    /// Move to the next field.
    pub fn next_field(&mut self) {
        self.focus_field = match self.auth_type {
            CredentialsType::Basic => match self.focus_field {
                LoginField::Username => LoginField::Password,
                LoginField::Password => LoginField::Username,
                LoginField::Token => LoginField::Username,
            },
            CredentialsType::Bearer => LoginField::Token,
        };
    }

    /// Move to the previous field.
    pub fn prev_field(&mut self) {
        self.focus_field = match self.auth_type {
            CredentialsType::Basic => match self.focus_field {
                LoginField::Username => LoginField::Password,
                LoginField::Password => LoginField::Username,
                LoginField::Token => LoginField::Password,
            },
            CredentialsType::Bearer => LoginField::Token,
        };
    }

    /// Add a character to the focused field.
    pub fn input_char(&mut self, c: char) {
        match self.focus_field {
            LoginField::Username => self.username.push(c),
            LoginField::Password | LoginField::Token => self.password.push(c),
        }
    }

    /// Remove the last character from the focused field.
    pub fn backspace(&mut self) {
        match self.focus_field {
            LoginField::Username => {
                self.username.pop();
            }
            LoginField::Password | LoginField::Token => {
                self.password.pop();
            }
        }
    }

    /// Check if the form is valid.
    pub fn is_valid(&self) -> bool {
        match self.auth_type {
            CredentialsType::Basic => !self.username.is_empty() && !self.password.is_empty(),
            CredentialsType::Bearer => !self.password.is_empty(),
        }
    }

    /// Toggle password visibility.
    pub fn toggle_password_visibility(&mut self) {
        self.is_password_masked = !self.is_password_masked;
    }

    /// Get the title for the modal.
    pub fn title(&self) -> &str {
        match self.auth_type {
            CredentialsType::Basic => "Login - Basic Authentication",
            CredentialsType::Bearer => "Login - Bearer Token",
        }
    }

    /// Clear all fields.
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.username.clear();
        self.password.clear();
        self.error = None;
    }
}

pub struct LoginModalWidget;

impl LoginModalWidget {
    pub fn render(modal: &LoginModal, area: Rect, buf: &mut Buffer) {
        let width = 50;
        let height = match modal.auth_type {
            CredentialsType::Basic => 12,
            CredentialsType::Bearer => 9,
        };

        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;
        let popup_area = Rect::new(x, y, width, height);

        Clear.render(popup_area, buf);
        Block::default()
            .borders(Borders::ALL)
            .title(modal.title())
            .render(popup_area, buf);

        let inner = Rect {
            x: popup_area.x + 1,
            y: popup_area.y + 1,
            width: popup_area.width.saturating_sub(2),
            height: popup_area.height.saturating_sub(2),
        };

        let mut y_offset = inner.y;

        if let Some(error) = &modal.error {
            let error_text = Paragraph::new(error.as_str()).style(Style::default().fg(Color::Red));
            error_text.render(
                Rect {
                    x: inner.x,
                    y: y_offset,
                    width: inner.width,
                    height: 1,
                },
                buf,
            );
            y_offset += 2;
        }

        if modal.auth_type == CredentialsType::Basic {
            let username_style = if modal.focus_field == LoginField::Username {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Paragraph::new(format!("Username: {}", modal.username))
                .style(username_style)
                .render(
                    Rect {
                        x: inner.x,
                        y: y_offset,
                        width: inner.width,
                        height: 1,
                    },
                    buf,
                );
            y_offset += 2;
        }

        let field_label = match modal.auth_type {
            CredentialsType::Basic => "Password: ",
            CredentialsType::Bearer => "Token: ",
        };

        let password_style =
            if matches!(modal.focus_field, LoginField::Password | LoginField::Token) {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

        let display_password =
            if modal.is_password_masked && matches!(modal.focus_field, LoginField::Password) {
                "*".repeat(modal.password.len())
            } else {
                modal.password.clone()
            };

        Paragraph::new(format!("{}{}", field_label, display_password))
            .style(password_style)
            .render(
                Rect {
                    x: inner.x,
                    y: y_offset,
                    width: inner.width,
                    height: 1,
                },
                buf,
            );
        y_offset += 2;

        let instructions = vec!["Tab/Shift+Tab: Switch fields", "Enter: Login | Esc: Cancel"];

        for instruction in instructions {
            if y_offset < inner.y + inner.height {
                Paragraph::new(instruction)
                    .style(Style::default().dim())
                    .render(
                        Rect {
                            x: inner.x,
                            y: y_offset,
                            width: inner.width,
                            height: 1,
                        },
                        buf,
                    );
                y_offset += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_auth_modal_creation() {
        let modal = LoginModal::new_basic();
        assert_eq!(modal.auth_type, CredentialsType::Basic);
        assert_eq!(modal.focus_field, LoginField::Username);
        assert!(modal.is_password_masked);
    }

    #[test]
    fn test_bearer_token_modal_creation() {
        let modal = LoginModal::new_bearer();
        assert_eq!(modal.auth_type, CredentialsType::Bearer);
        assert_eq!(modal.focus_field, LoginField::Token);
        assert!(!modal.is_password_masked);
    }

    #[test]
    fn test_basic_auth_validation() {
        let mut modal = LoginModal::new_basic();
        assert!(!modal.is_valid());

        modal.username = "user".to_string();
        assert!(!modal.is_valid());

        modal.password = "pass".to_string();
        assert!(modal.is_valid());
    }

    #[test]
    fn test_bearer_token_validation() {
        let mut modal = LoginModal::new_bearer();
        assert!(!modal.is_valid());

        modal.password = "token123".to_string();
        assert!(modal.is_valid());
    }

    #[test]
    fn test_input_char() {
        let mut modal = LoginModal::new_basic();
        modal.input_char('u');
        assert_eq!(modal.username, "u");

        modal.next_field();
        modal.input_char('p');
        assert_eq!(modal.password, "p");
    }

    #[test]
    fn test_backspace() {
        let mut modal = LoginModal::new_basic();
        modal.username = "user".to_string();
        modal.backspace();
        assert_eq!(modal.username, "use");
    }
}
