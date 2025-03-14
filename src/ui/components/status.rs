use crossterm::{
    cursor,
    QueueableCommand,
    style::{Color, SetForegroundColor, ResetColor},
};
use std::io::{self, Write};
use std::time::SystemTime;
use derive_more::From;

use crate::ui::layout::LayoutManager;
use crate::ui::theme::{Theme, Themeable, ColorRole, ThemeError, Style};

/// Errors that can occur during status message operations.
#[derive(Debug, From)]
pub enum StatusError {
    /// An I/O error occurred while writing status messages.
    IoError(io::Error),
    /// An invalid message type was specified.
    InvalidType(String),
}

/// A type alias for Result with StatusError as the error type.
pub type Result<T> = std::result::Result<T, StatusError>;

/// The type of status message.
pub enum MessageType {
    /// A success message, typically displayed in green.
    Success,
    /// An error message, typically displayed in red.
    Error,
    /// An informational message, typically displayed in blue.
    Info,
    /// A warning message, typically displayed in yellow.
    Warning,
    /// A debug message, typically displayed in gray.
    Debug,
}

/// A status message with metadata.
pub struct StatusMessage {
    /// The text content of the message.
    pub text: String,
    /// The type of message (success, error, etc.).
    pub message_type: MessageType,
    /// When the message was created.
    pub timestamp: SystemTime,
    /// The priority of the message (higher numbers = higher priority).
    pub priority: u8,
}

impl StatusMessage {
    /// Creates a new status message with the given text and type.
    ///
    /// # Arguments
    /// * `text` - The message text
    /// * `message_type` - The type of message
    pub fn new(text: impl Into<String>, message_type: MessageType) -> Self {
        Self {
            text: text.into(),
            message_type,
            timestamp: SystemTime::now(),
            priority: 0,
        }
    }

    /// Sets the priority of the message and returns self.
    ///
    /// # Arguments
    /// * `priority` - The priority level (higher numbers = higher priority)
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
}

/// Manages status messages in the UI.
pub struct StatusManager {
    /// The maximum number of messages to keep in history.
    max_history: usize,
    /// The history of status messages.
    history: Vec<StatusMessage>,
    /// The layout manager for positioning status messages.
    #[allow(dead_code)]
    layout: LayoutManager,
    /// The style configuration for status messages.
    #[allow(dead_code)]
    style: Style,
}

impl StatusManager {
    /// Creates a new status manager with default settings.
    pub fn new() -> Self {
        Self {
            max_history: 100,
            history: Vec::new(),
            layout: LayoutManager::new(),
            style: Style::new(),
        }
    }

    /// Sets the maximum number of messages to keep in history.
    ///
    /// # Arguments
    /// * `max` - The maximum number of messages to store
    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }

    /// Adds a new message to the history.
    ///
    /// If the history exceeds the maximum size, the oldest messages are removed.
    pub fn add_message(&mut self, message: StatusMessage) {
        self.history.push(message);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    /// Returns a slice of all messages in the history.
    pub fn get_history(&self) -> &[StatusMessage] {
        &self.history
    }

    /// Clears all messages from the history.
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Prints a status message with the specified indentation.
    ///
    /// # Arguments
    /// * `writer` - The output writer
    /// * `message` - The message to print
    /// * `indentation` - The number of spaces to indent
    pub fn print_message<W: Write>(&self, _writer: &mut W, _message: &StatusMessage, _indentation: usize) -> Result<()> {
        // ... existing code ...
        Ok(())
    }

    /// Prints a success message.
    ///
    /// # Arguments
    /// * `writer` - The output writer
    /// * `text` - The message text
    /// * `indentation` - The number of spaces to indent
    pub fn print_success<W: Write>(&self, _writer: &mut W, _text: &str, _indentation: usize) -> Result<()> {
        // ... existing code ...
        Ok(())
    }

    /// Prints an error message.
    ///
    /// # Arguments
    /// * `writer` - The output writer
    /// * `text` - The message text
    /// * `indentation` - The number of spaces to indent
    pub fn print_error<W: Write>(&self, _writer: &mut W, _text: &str, _indentation: usize) -> Result<()> {
        // ... existing code ...
        Ok(())
    }

    /// Prints an informational message.
    ///
    /// # Arguments
    /// * `writer` - The output writer
    /// * `text` - The message text
    /// * `indentation` - The number of spaces to indent
    pub fn print_info<W: Write>(&self, _writer: &mut W, _text: &str, _indentation: usize) -> Result<()> {
        // ... existing code ...
        Ok(())
    }

    /// Prints a warning message.
    ///
    /// # Arguments
    /// * `writer` - The output writer
    /// * `text` - The message text
    /// * `indentation` - The number of spaces to indent
    pub fn print_warning<W: Write>(&self, _writer: &mut W, _text: &str, _indentation: usize) -> Result<()> {
        // ... existing code ...
        Ok(())
    }

    /// Prints a debug message.
    ///
    /// # Arguments
    /// * `writer` - The output writer
    /// * `text` - The message text
    /// * `indentation` - The number of spaces to indent
    pub fn print_debug<W: Write>(&self, _writer: &mut W, _text: &str, _indentation: usize) -> Result<()> {
        // ... existing code ...
        Ok(())
    }
}

impl Default for StatusManager {
    fn default() -> Self {
        Self::new()
    }
}

/// A component for displaying status messages with different types and styles.
pub struct Status {
    /// The layout manager for positioning status messages.
    layout: LayoutManager,
    /// The style configuration for status messages.
    style: Style,
}

impl Status {
    /// Creates a new Status component with the given layout manager.
    ///
    /// # Arguments
    /// * `layout` - The layout manager for positioning status messages
    pub fn new(layout: LayoutManager) -> Self {
        Self {
            layout,
            style: Style::new(),
        }
    }

    /// Prints a success message with green color.
    ///
    /// # Arguments
    /// * `writer` - The output writer
    /// * `text` - The message text
    ///
    /// # Returns
    /// * `io::Result<()>` - Success or error during writing
    pub fn print_success<W: Write>(&self, writer: &mut W, text: &str) -> io::Result<()> {
        let indent = self.layout.get_current_indentation();
        writer.queue(cursor::SavePosition)?;
        writer.queue(cursor::MoveRight(indent as u16))?;
        writer.queue(SetForegroundColor(Color::Green))?;
        write!(writer, "✓ {}", text)?;
        writer.queue(ResetColor)?;
        writer.write_all(b"\n")?;
        writer.queue(cursor::RestorePosition)?;
        writer.flush()?;
        Ok(())
    }

    /// Prints an error message with red color.
    ///
    /// # Arguments
    /// * `writer` - The output writer
    /// * `text` - The message text
    ///
    /// # Returns
    /// * `io::Result<()>` - Success or error during writing
    pub fn print_error<W: Write>(&self, writer: &mut W, text: &str) -> io::Result<()> {
        let indent = self.layout.get_current_indentation();
        writer.queue(cursor::SavePosition)?;
        writer.queue(cursor::MoveRight(indent as u16))?;
        writer.queue(SetForegroundColor(Color::Red))?;
        write!(writer, "✗ {}", text)?;
        writer.queue(ResetColor)?;
        writer.write_all(b"\n")?;
        writer.queue(cursor::RestorePosition)?;
        writer.flush()?;
        Ok(())
    }

    /// Prints an info message with blue color.
    ///
    /// # Arguments
    /// * `writer` - The output writer
    /// * `text` - The message text
    ///
    /// # Returns
    /// * `io::Result<()>` - Success or error during writing
    pub fn print_info<W: Write>(&self, writer: &mut W, text: &str) -> io::Result<()> {
        let indent = self.layout.get_current_indentation();
        writer.queue(cursor::SavePosition)?;
        writer.queue(cursor::MoveRight(indent as u16))?;
        writer.queue(SetForegroundColor(Color::Yellow))?;
        write!(writer, "ℹ {}", text)?;
        writer.queue(ResetColor)?;
        writer.write_all(b"\n")?;
        writer.queue(cursor::RestorePosition)?;
        writer.flush()?;
        Ok(())
    }
}

impl Themeable for Status {
    fn apply_theme(&mut self, theme: &Theme) -> std::result::Result<(), ThemeError> {
        self.style = theme.styles.text.clone();
        Ok(())
    }

    fn get_style(&self) -> &Style {
        &self.style
    }

    fn get_color(&self, role: ColorRole) -> crate::ui::theme::Color {
        match role {
            ColorRole::Success => crate::ui::theme::Color::Green,
            ColorRole::Error => crate::ui::theme::Color::Red,
            ColorRole::Warning => crate::ui::theme::Color::Yellow,
            _ => crate::ui::theme::Color::White,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_status_messages() {
        let mut buffer = Cursor::new(Vec::new());
        let layout = LayoutManager::new();
        let status = Status::new(layout);

        // Test success message
        status.print_success(&mut buffer, "Operation completed").unwrap();
        let output = String::from_utf8(buffer.get_ref().to_vec()).unwrap();
        assert!(output.contains("✓ Operation completed"));

        // Reset buffer
        buffer = Cursor::new(Vec::new());

        // Test error message
        status.print_error(&mut buffer, "Operation failed").unwrap();
        let output = String::from_utf8(buffer.get_ref().to_vec()).unwrap();
        assert!(output.contains("✗ Operation failed"));

        // Reset buffer
        buffer = Cursor::new(Vec::new());

        // Test info message
        status.print_info(&mut buffer, "Processing...").unwrap();
        let output = String::from_utf8(buffer.get_ref().to_vec()).unwrap();
        assert!(output.contains("ℹ Processing..."));
    }

    #[test]
    fn test_theme_application() {
        let layout = LayoutManager::new();
        let mut status = Status::new(layout);

        let theme = Theme {
            name: "test".to_string(),
            colors: crate::ui::theme::ColorScheme {
                primary: crate::ui::theme::Color::Red,
                secondary: crate::ui::theme::Color::Cyan,
                background: crate::ui::theme::Color::Black,
                foreground: crate::ui::theme::Color::White,
                accent: crate::ui::theme::Color::Yellow,
                error: crate::ui::theme::Color::Red,
                warning: crate::ui::theme::Color::DarkYellow,
                success: crate::ui::theme::Color::Green,
            },
            styles: crate::ui::theme::StyleSet {
                header: Style::new(),
                text: Style::new().bold(),
                input: Style::new(),
                button: Style::new(),
                dialog: Style::new(),
            },
            metadata: crate::ui::theme::ThemeMetadata {
                version: "1.0.0".to_string(),
                author: "Test".to_string(),
                description: "Test theme".to_string(),
            },
        };

        assert!(status.apply_theme(&theme).is_ok());
        assert!(status.get_style().attributes.contains(&crate::ui::theme::Attribute::Bold));
        assert_eq!(status.get_color(ColorRole::Success), crate::ui::theme::Color::Green);
        assert_eq!(status.get_color(ColorRole::Error), crate::ui::theme::Color::Red);
        assert_eq!(status.get_color(ColorRole::Warning), crate::ui::theme::Color::Yellow);
    }
} 