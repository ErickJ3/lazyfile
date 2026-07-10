//! File operations modal widget.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
};

/// Type of file operation being performed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileOperationType {
    /// Delete a file
    DeleteFile,
    /// Delete a directory (purge)
    DeleteDirectory,
    /// Create a new directory
    Mkdir,
    /// Copy a file
    Copy,
    /// Move a file
    Move,
}

/// State for file operations modal.
#[derive(Debug, Clone)]
pub struct FileOperationsModal {
    pub operation: FileOperationType,
    pub file_name: String,
    pub current_path: String,
    pub input: String,
    pub error: Option<String>,
}

impl FileOperationsModal {
    /// Create a new file operations modal for delete file.
    pub fn delete_file(file_name: String) -> Self {
        Self {
            operation: FileOperationType::DeleteFile,
            file_name,
            current_path: String::new(),
            input: String::new(),
            error: None,
        }
    }

    /// Create a new file operations modal for delete directory.
    pub fn delete_directory(dir_name: String) -> Self {
        Self {
            operation: FileOperationType::DeleteDirectory,
            file_name: dir_name,
            current_path: String::new(),
            input: String::new(),
            error: None,
        }
    }

    /// Create a new file operations modal for mkdir.
    pub fn mkdir(current_path: String) -> Self {
        Self {
            operation: FileOperationType::Mkdir,
            file_name: String::new(),
            current_path,
            input: String::new(),
            error: None,
        }
    }

    /// Create a new file operations modal for copy.
    pub fn copy(file_name: String, current_path: String) -> Self {
        Self {
            operation: FileOperationType::Copy,
            file_name,
            current_path,
            input: String::new(),
            error: None,
        }
    }

    /// Create a new file operations modal for move.
    pub fn move_file(file_name: String, current_path: String) -> Self {
        Self {
            operation: FileOperationType::Move,
            file_name,
            current_path,
            input: String::new(),
            error: None,
        }
    }

    pub fn input_char(&mut self, c: char) {
        // Bracketed paste can deliver control characters as Char
        // events; they are never valid in a path segment.
        if c.is_control() {
            return;
        }
        self.input.push(c);
        self.error = None;
    }

    pub fn backspace(&mut self) {
        self.input.pop();
    }

    pub fn is_valid(&self) -> bool {
        match self.operation {
            FileOperationType::DeleteFile | FileOperationType::DeleteDirectory => true,
            FileOperationType::Mkdir | FileOperationType::Copy | FileOperationType::Move => {
                !self.input.is_empty()
            }
        }
    }

    pub fn get_title(&self) -> &str {
        match self.operation {
            FileOperationType::DeleteFile => "Delete File",
            FileOperationType::DeleteDirectory => "Delete Directory",
            FileOperationType::Mkdir => "New Directory",
            FileOperationType::Copy => "Copy File",
            FileOperationType::Move => "Move File",
        }
    }

    pub fn get_message(&self) -> String {
        match self.operation {
            FileOperationType::DeleteFile => {
                format!("Delete file '{}'?", self.file_name)
            }
            FileOperationType::DeleteDirectory => {
                format!("Delete directory '{}' and all contents?", self.file_name)
            }
            FileOperationType::Mkdir => "Enter directory name:".to_string(),
            FileOperationType::Copy => {
                format!("Copy '{}' to (relative path):", self.file_name)
            }
            FileOperationType::Move => {
                format!("Move '{}' to (relative path):", self.file_name)
            }
        }
    }

    pub fn needs_input(&self) -> bool {
        matches!(
            self.operation,
            FileOperationType::Mkdir | FileOperationType::Copy | FileOperationType::Move
        )
    }
}

pub struct FileOperationsWidget;

impl FileOperationsWidget {
    pub fn render(f: &mut Frame, area: Rect, modal: &FileOperationsModal) {
        // Render backdrop
        f.render_widget(Clear, area);
        f.render_widget(
            Block::default().style(Style::default().bg(Color::DarkGray)),
            area,
        );

        // Calculate modal size
        let modal_width = 55.min(area.width.saturating_sub(4));
        let modal_height = if modal.needs_input() { 11 } else { 9 };
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
        let border_color = match modal.operation {
            FileOperationType::DeleteFile | FileOperationType::DeleteDirectory => Color::Red,
            _ => Color::Cyan,
        };
        let block = Block::default()
            .title(format!(" {} ", modal.get_title()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));
        f.render_widget(block, modal_area);

        let inner = Rect {
            x: modal_area.x + 1,
            y: modal_area.y + 1,
            width: modal_area.width.saturating_sub(2),
            height: modal_area.height.saturating_sub(2),
        };

        let mut constraints = vec![Constraint::Length(2), Constraint::Length(1)];
        if modal.needs_input() {
            constraints.insert(1, Constraint::Length(3));
        }
        constraints.push(Constraint::Length(1));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner);

        // Message
        let msg_text = modal.get_message();
        let message = Paragraph::new(msg_text.as_str());
        f.render_widget(message, chunks[0]);

        // Input field (if needed)
        let mut message_idx = 1;
        if modal.needs_input() {
            let input_style = Style::default().fg(Color::Cyan).bg(Color::Black);
            let input_text = format!(" {} ", modal.input);
            let input = Paragraph::new(input_text)
                .style(input_style)
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(input, chunks[1]);
            message_idx = 2;
        }

        // Error message
        if let Some(error) = &modal.error {
            let error_para = Paragraph::new(error.as_str()).style(Style::default().fg(Color::Red));
            f.render_widget(error_para, chunks[message_idx]);
        }

        // Help text
        let help_text = if modal.needs_input() {
            "Type: input | Backspace: delete char | Enter: confirm | Esc: cancel"
        } else {
            "Enter: confirm | Esc: cancel"
        };
        let help = Paragraph::new(help_text).style(Style::default().fg(Color::Gray));
        f.render_widget(help, chunks[chunks.len() - 1]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =============================================================================
    // FileOperationType Tests
    // =============================================================================

    #[test]
    fn test_file_operation_type_equality() {
        assert_eq!(FileOperationType::DeleteFile, FileOperationType::DeleteFile);
        assert_eq!(
            FileOperationType::DeleteDirectory,
            FileOperationType::DeleteDirectory
        );
        assert_eq!(FileOperationType::Mkdir, FileOperationType::Mkdir);
        assert_eq!(FileOperationType::Copy, FileOperationType::Copy);
        assert_eq!(FileOperationType::Move, FileOperationType::Move);

        assert_ne!(
            FileOperationType::DeleteFile,
            FileOperationType::DeleteDirectory
        );
        assert_ne!(FileOperationType::Copy, FileOperationType::Move);
    }

    #[test]
    fn test_file_operation_type_clone() {
        let op = FileOperationType::Copy;
        let cloned = op.clone();
        assert_eq!(op, cloned);
    }

    // =============================================================================
    // DeleteFile Modal Tests
    // =============================================================================

    #[test]
    fn test_delete_file_modal_creation() {
        let modal = FileOperationsModal::delete_file("test.txt".to_string());

        assert_eq!(modal.operation, FileOperationType::DeleteFile);
        assert_eq!(modal.file_name, "test.txt");
        assert!(modal.current_path.is_empty());
        assert!(modal.input.is_empty());
        assert!(modal.error.is_none());
    }

    #[test]
    fn test_delete_file_modal_title() {
        let modal = FileOperationsModal::delete_file("test.txt".to_string());
        assert_eq!(modal.get_title(), "Delete File");
    }

    #[test]
    fn test_delete_file_modal_message() {
        let modal = FileOperationsModal::delete_file("important.txt".to_string());
        assert_eq!(modal.get_message(), "Delete file 'important.txt'?");
    }

    #[test]
    fn test_delete_file_does_not_need_input() {
        let modal = FileOperationsModal::delete_file("test.txt".to_string());
        assert!(!modal.needs_input());
    }

    #[test]
    fn test_delete_file_is_always_valid() {
        let modal = FileOperationsModal::delete_file("test.txt".to_string());
        assert!(modal.is_valid());
    }

    // =============================================================================
    // DeleteDirectory Modal Tests
    // =============================================================================

    #[test]
    fn test_delete_directory_modal_creation() {
        let modal = FileOperationsModal::delete_directory("mydir".to_string());

        assert_eq!(modal.operation, FileOperationType::DeleteDirectory);
        assert_eq!(modal.file_name, "mydir");
        assert!(modal.current_path.is_empty());
        assert!(modal.input.is_empty());
        assert!(modal.error.is_none());
    }

    #[test]
    fn test_delete_directory_modal_title() {
        let modal = FileOperationsModal::delete_directory("mydir".to_string());
        assert_eq!(modal.get_title(), "Delete Directory");
    }

    #[test]
    fn test_delete_directory_modal_message() {
        let modal = FileOperationsModal::delete_directory("important_folder".to_string());
        assert_eq!(
            modal.get_message(),
            "Delete directory 'important_folder' and all contents?"
        );
    }

    #[test]
    fn test_delete_directory_does_not_need_input() {
        let modal = FileOperationsModal::delete_directory("mydir".to_string());
        assert!(!modal.needs_input());
    }

    #[test]
    fn test_delete_directory_is_always_valid() {
        let modal = FileOperationsModal::delete_directory("mydir".to_string());
        assert!(modal.is_valid());
    }

    // =============================================================================
    // Mkdir Modal Tests
    // =============================================================================

    #[test]
    fn test_mkdir_modal_creation() {
        let modal = FileOperationsModal::mkdir("/current/path".to_string());

        assert_eq!(modal.operation, FileOperationType::Mkdir);
        assert!(modal.file_name.is_empty());
        assert_eq!(modal.current_path, "/current/path");
        assert!(modal.input.is_empty());
        assert!(modal.error.is_none());
    }

    #[test]
    fn test_mkdir_modal_title() {
        let modal = FileOperationsModal::mkdir("/path".to_string());
        assert_eq!(modal.get_title(), "New Directory");
    }

    #[test]
    fn test_mkdir_modal_message() {
        let modal = FileOperationsModal::mkdir("/path".to_string());
        assert_eq!(modal.get_message(), "Enter directory name:");
    }

    #[test]
    fn test_mkdir_needs_input() {
        let modal = FileOperationsModal::mkdir("/path".to_string());
        assert!(modal.needs_input());
    }

    #[test]
    fn test_mkdir_invalid_when_empty() {
        let modal = FileOperationsModal::mkdir("/path".to_string());
        assert!(!modal.is_valid());
    }

    #[test]
    fn test_mkdir_valid_when_has_input() {
        let mut modal = FileOperationsModal::mkdir("/path".to_string());
        modal.input_char('n');
        modal.input_char('e');
        modal.input_char('w');
        assert!(modal.is_valid());
        assert_eq!(modal.input, "new");
    }

    // =============================================================================
    // Copy Modal Tests
    // =============================================================================

    #[test]
    fn test_copy_modal_creation() {
        let modal = FileOperationsModal::copy("source.txt".to_string(), "/current".to_string());

        assert_eq!(modal.operation, FileOperationType::Copy);
        assert_eq!(modal.file_name, "source.txt");
        assert_eq!(modal.current_path, "/current");
        assert!(modal.input.is_empty());
        assert!(modal.error.is_none());
    }

    #[test]
    fn test_copy_modal_title() {
        let modal = FileOperationsModal::copy("source.txt".to_string(), "/path".to_string());
        assert_eq!(modal.get_title(), "Copy File");
    }

    #[test]
    fn test_copy_modal_message() {
        let modal = FileOperationsModal::copy("myfile.txt".to_string(), "/path".to_string());
        assert_eq!(modal.get_message(), "Copy 'myfile.txt' to (relative path):");
    }

    #[test]
    fn test_copy_needs_input() {
        let modal = FileOperationsModal::copy("source.txt".to_string(), "/path".to_string());
        assert!(modal.needs_input());
    }

    #[test]
    fn test_copy_invalid_when_empty() {
        let modal = FileOperationsModal::copy("source.txt".to_string(), "/path".to_string());
        assert!(!modal.is_valid());
    }

    #[test]
    fn test_copy_valid_when_has_destination() {
        let mut modal = FileOperationsModal::copy("source.txt".to_string(), "/path".to_string());
        modal.input_char('d');
        modal.input_char('e');
        modal.input_char('s');
        modal.input_char('t');
        assert!(modal.is_valid());
        assert_eq!(modal.input, "dest");
    }

    // =============================================================================
    // Move Modal Tests
    // =============================================================================

    #[test]
    fn test_move_modal_creation() {
        let modal =
            FileOperationsModal::move_file("source.txt".to_string(), "/current".to_string());

        assert_eq!(modal.operation, FileOperationType::Move);
        assert_eq!(modal.file_name, "source.txt");
        assert_eq!(modal.current_path, "/current");
        assert!(modal.input.is_empty());
        assert!(modal.error.is_none());
    }

    #[test]
    fn test_move_modal_title() {
        let modal = FileOperationsModal::move_file("source.txt".to_string(), "/path".to_string());
        assert_eq!(modal.get_title(), "Move File");
    }

    #[test]
    fn test_move_modal_message() {
        let modal = FileOperationsModal::move_file("myfile.txt".to_string(), "/path".to_string());
        assert_eq!(modal.get_message(), "Move 'myfile.txt' to (relative path):");
    }

    #[test]
    fn test_move_needs_input() {
        let modal = FileOperationsModal::move_file("source.txt".to_string(), "/path".to_string());
        assert!(modal.needs_input());
    }

    #[test]
    fn test_move_invalid_when_empty() {
        let modal = FileOperationsModal::move_file("source.txt".to_string(), "/path".to_string());
        assert!(!modal.is_valid());
    }

    #[test]
    fn test_move_valid_when_has_destination() {
        let mut modal =
            FileOperationsModal::move_file("source.txt".to_string(), "/path".to_string());
        modal.input_char('n');
        modal.input_char('e');
        modal.input_char('w');
        modal.input_char('_');
        modal.input_char('n');
        modal.input_char('a');
        modal.input_char('m');
        modal.input_char('e');
        assert!(modal.is_valid());
        assert_eq!(modal.input, "new_name");
    }

    // =============================================================================
    // Input Handling Tests
    // =============================================================================

    #[test]
    fn test_input_char_adds_character() {
        let mut modal = FileOperationsModal::mkdir("/path".to_string());
        modal.input_char('a');
        assert_eq!(modal.input, "a");
        modal.input_char('b');
        assert_eq!(modal.input, "ab");
        modal.input_char('c');
        assert_eq!(modal.input, "abc");
    }

    #[test]
    fn test_input_char_clears_error() {
        let mut modal = FileOperationsModal::mkdir("/path".to_string());
        modal.error = Some("Previous error".to_string());
        modal.input_char('a');
        assert!(modal.error.is_none());
    }

    #[test]
    fn test_backspace_removes_character() {
        let mut modal = FileOperationsModal::mkdir("/path".to_string());
        modal.input = "abc".to_string();
        modal.backspace();
        assert_eq!(modal.input, "ab");
        modal.backspace();
        assert_eq!(modal.input, "a");
        modal.backspace();
        assert_eq!(modal.input, "");
    }

    #[test]
    fn test_backspace_on_empty_string() {
        let mut modal = FileOperationsModal::mkdir("/path".to_string());
        modal.backspace(); // Should not panic
        assert_eq!(modal.input, "");
    }

    #[test]
    fn test_input_with_spaces() {
        let mut modal = FileOperationsModal::mkdir("/path".to_string());
        modal.input_char('m');
        modal.input_char('y');
        modal.input_char(' ');
        modal.input_char('d');
        modal.input_char('i');
        modal.input_char('r');
        assert_eq!(modal.input, "my dir");
    }

    #[test]
    fn test_input_with_special_characters() {
        let mut modal = FileOperationsModal::mkdir("/path".to_string());
        modal.input_char('d');
        modal.input_char('i');
        modal.input_char('r');
        modal.input_char('-');
        modal.input_char('1');
        modal.input_char('_');
        modal.input_char('2');
        assert_eq!(modal.input, "dir-1_2");
    }

    // =============================================================================
    // Clone Tests
    // =============================================================================

    #[test]
    fn test_modal_clone() {
        let mut original = FileOperationsModal::copy("file.txt".to_string(), "/path".to_string());
        original.input = "destination".to_string();
        original.error = Some("test error".to_string());

        let cloned = original.clone();

        assert_eq!(cloned.operation, FileOperationType::Copy);
        assert_eq!(cloned.file_name, "file.txt");
        assert_eq!(cloned.current_path, "/path");
        assert_eq!(cloned.input, "destination");
        assert_eq!(cloned.error, Some("test error".to_string()));
    }

    // =============================================================================
    // Edge Cases Tests
    // =============================================================================

    #[test]
    fn test_empty_file_name() {
        let modal = FileOperationsModal::delete_file(String::new());
        assert_eq!(modal.get_message(), "Delete file ''?");
    }

    #[test]
    fn test_file_name_with_special_chars() {
        let modal = FileOperationsModal::delete_file("file<>:\"name.txt".to_string());
        assert_eq!(modal.get_message(), "Delete file 'file<>:\"name.txt'?");
    }

    #[test]
    fn test_unicode_file_name() {
        let modal = FileOperationsModal::delete_file("arquivo_português.txt".to_string());
        assert_eq!(modal.get_message(), "Delete file 'arquivo_português.txt'?");
    }

    #[test]
    fn test_long_file_name() {
        let long_name = "a".repeat(100);
        let modal = FileOperationsModal::delete_file(long_name.clone());
        assert_eq!(modal.file_name, long_name);
    }

    #[test]
    fn test_root_path_mkdir() {
        let modal = FileOperationsModal::mkdir("/".to_string());
        assert_eq!(modal.current_path, "/");
    }

    #[test]
    fn test_empty_path_mkdir() {
        let modal = FileOperationsModal::mkdir(String::new());
        assert!(modal.current_path.is_empty());
    }

    #[test]
    fn test_nested_path_copy() {
        let modal =
            FileOperationsModal::copy("file.txt".to_string(), "/very/deep/nested/path".to_string());
        assert_eq!(modal.current_path, "/very/deep/nested/path");
    }

    #[test]
    fn test_input_char_ignores_control_chars() {
        let mut modal = FileOperationsModal::mkdir("/".to_string());
        modal.input_char('\n');
        modal.input_char('\u{1b}');
        modal.input_char('a');
        assert_eq!(modal.input, "a");
    }
}
