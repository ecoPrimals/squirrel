use std::io::{self, Write};
use crossterm::{
    style::{Color, ResetColor, SetForegroundColor},
    QueueableCommand,
};
use std::time::SystemTime;
use derive_more;
use crate::ui::layout::LayoutManager;
use crate::ui::theme::Style;

/// Error types that can occur during status message operations.
#[derive(derive_more::From, Debug)]
pub enum StatusError {
    /// An I/O error occurred during status message operations.
    IoError(io::Error),
    /// The message type was invalid or unsupported.
    InvalidType(String),
}

/// A type alias for Result with StatusError as the error type.
pub type Result<T> = std::result::Result<T, StatusError>;

/// Types of status messages that can be displayed.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MessageType {
    /// A success message indicating a completed operation.
    Success,
    /// An error message indicating a failed operation.
    Error,
    /// An informational message.
    Info,
    /// A warning message.
    Warning,
    /// A debug message for development purposes.
    Debug,
}

/// A status message with associated metadata.
#[derive(Clone)]
pub struct StatusMessage {
    /// The text content of the message.
    pub text: String,
    /// The type of message.
    pub message_type: MessageType,
    /// When the message was created.
    pub timestamp: SystemTime,
    /// Priority level of the message (higher numbers = higher priority).
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

    /// Sets the priority level of the message.
    /// 
    /// # Arguments
    /// * `priority` - Priority level (higher numbers = higher priority)
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
}

/// Manages a collection of status messages and handles their display.
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
            layout: LayoutManager::new(),
            style: Style::default(),
            history: Vec::new(),
            max_history: 100,
        }
    }

    /// Sets the maximum number of messages to keep in history.
    /// 
    /// # Arguments
    /// * `max` - Maximum number of messages to store
    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }

    /// Adds a new message to the history.
    /// 
    /// # Arguments
    /// * `message` - The status message to add
    pub fn add_message(&mut self, message: StatusMessage) {
        self.history.push(message);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    /// Gets the current message history.
    pub fn get_history(&self) -> &[StatusMessage] {
        &self.history
    }

    /// Clears all messages from the history.
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Prints a message with the specified indentation.
    /// 
    /// # Arguments
    /// * `writer` - The writer to output to
    /// * `message` - The message to print
    /// * `indentation` - Number of spaces to indent
    pub fn print_message<W: std::io::Write>(&self, writer: &mut W, message: &StatusMessage, indentation: usize) -> Result<()> {
        let (color, symbol) = match message.message_type {
            MessageType::Success => (Color::Green, "✓"),
            MessageType::Error => (Color::Red, "✗"),
            MessageType::Info => (Color::Blue, "ℹ"),
            MessageType::Warning => (Color::Yellow, "⚠"),
            MessageType::Debug => (Color::Magenta, "⚙"),
        };

        writer.queue(SetForegroundColor(color)).map_err(StatusError::IoError)?;
        write!(writer, "{}{} ", " ".repeat(indentation), symbol).map_err(StatusError::IoError)?;
        writer.queue(ResetColor).map_err(StatusError::IoError)?;
        writeln!(writer, "{}\n", message.text).map_err(StatusError::IoError)?;
        writer.flush().map_err(StatusError::IoError)?;
        Ok(())
    }

    /// Prints a success message.
    /// 
    /// # Arguments
    /// * `writer` - The writer to output to
    /// * `text` - The message text
    /// * `indentation` - Number of spaces to indent
    pub fn print_success<W: Write>(&self, writer: &mut W, text: &str, indentation: usize) -> Result<()> {
        let message = StatusMessage::new(text, MessageType::Success);
        self.print_message(writer, &message, indentation)
    }

    /// Prints an error message.
    /// 
    /// # Arguments
    /// * `writer` - The writer to output to
    /// * `text` - The message text
    /// * `indentation` - Number of spaces to indent
    pub fn print_error<W: Write>(&self, writer: &mut W, text: &str, indentation: usize) -> Result<()> {
        let message = StatusMessage::new(text, MessageType::Error);
        self.print_message(writer, &message, indentation)
    }

    /// Prints an info message.
    /// 
    /// # Arguments
    /// * `writer` - The writer to output to
    /// * `text` - The message text
    /// * `indentation` - Number of spaces to indent
    pub fn print_info<W: Write>(&self, writer: &mut W, text: &str, indentation: usize) -> Result<()> {
        let message = StatusMessage::new(text, MessageType::Info);
        self.print_message(writer, &message, indentation)
    }

    /// Prints a warning message.
    /// 
    /// # Arguments
    /// * `writer` - The writer to output to
    /// * `text` - The message text
    /// * `indentation` - Number of spaces to indent
    pub fn print_warning<W: Write>(&self, writer: &mut W, text: &str, indentation: usize) -> Result<()> {
        let message = StatusMessage::new(text, MessageType::Warning);
        self.print_message(writer, &message, indentation)
    }

    /// Prints a debug message.
    /// 
    /// # Arguments
    /// * `writer` - The writer to output to
    /// * `text` - The message text
    /// * `indentation` - Number of spaces to indent
    pub fn print_debug<W: Write>(&self, writer: &mut W, text: &str, indentation: usize) -> Result<()> {
        let message = StatusMessage::new(text, MessageType::Debug);
        self.print_message(writer, &message, indentation)
    }

    /// Gets all messages since a specific timestamp.
    /// 
    /// # Returns
    /// 
    /// Returns a vector of messages that were added after the specified timestamp.
    #[must_use]
    pub fn get_messages_since(&self, timestamp: SystemTime) -> Vec<StatusMessage> {
        self.history
            .iter()
            .filter(|msg| msg.timestamp >= timestamp)
            .cloned()
            .collect()
    }

    /// Gets all messages of a specific type.
    /// 
    /// # Returns
    /// 
    /// Returns a vector of messages that match the specified message type.
    #[must_use]
    pub fn get_messages_by_type(&self, msg_type: MessageType) -> Vec<StatusMessage> {
        self.history
            .iter()
            .filter(|msg| msg.message_type == msg_type)
            .cloned()
            .collect()
    }

    /// Gets all high priority messages.
    /// 
    /// # Returns
    /// 
    /// Returns a vector of messages marked as high priority.
    #[must_use]
    pub fn get_high_priority_messages(&self, min_priority: u8) -> Vec<StatusMessage> {
        self.history
            .iter()
            .filter(|msg| msg.priority >= min_priority)
            .cloned()
            .collect()
    }

    /// Gets the most recent message.
    /// 
    /// # Returns
    /// 
    /// Returns the most recent message if any exists.
    #[must_use]
    pub fn get_latest_message(&self) -> Option<StatusMessage> {
        self.history.last().cloned()
    }

    /// Gets all messages in the status manager.
    /// 
    /// # Returns
    /// 
    /// Returns a vector of all messages.
    #[must_use]
    pub fn get_all_messages(&self) -> Vec<StatusMessage> {
        self.history.clone()
    }
}

impl Default for StatusManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestWriter {
        inner: Vec<u8>,
    }

    impl TestWriter {
        fn new() -> Self {
            Self { inner: Vec::new() }
        }

        fn contents(&self) -> String {
            String::from_utf8_lossy(&self.inner).to_string()
        }

        fn contains_stripped(&self, text: &str) -> bool {
            let content = self.contents();
            let stripped = strip_ansi_escapes::strip(content.as_bytes());
            String::from_utf8_lossy(&stripped).contains(text)
        }
    }

    impl Write for TestWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.inner.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_message_creation() {
        let message = StatusMessage::new("Test message", MessageType::Success)
            .with_priority(1);
        assert_eq!(message.text, "Test message");
        assert_eq!(message.message_type, MessageType::Success);
        assert_eq!(message.priority, 1);
    }

    #[test]
    fn test_message_printing() {
        let mut writer = TestWriter::new();
        let manager = StatusManager::new();

        manager.print_success(&mut writer, "Success message", 0).unwrap();
        assert!(writer.contains_stripped("✓ Success message"));

        manager.print_error(&mut writer, "Error message", 0).unwrap();
        assert!(writer.contains_stripped("✗ Error message"));

        manager.print_info(&mut writer, "Info message", 0).unwrap();
        assert!(writer.contains_stripped("ℹ Info message"));
    }

    #[test]
    fn test_history_management() {
        let mut manager = StatusManager::new().with_max_history(2);
        
        let msg1 = StatusMessage::new("First", MessageType::Info);
        let msg2 = StatusMessage::new("Second", MessageType::Success);
        let msg3 = StatusMessage::new("Third", MessageType::Error);

        manager.add_message(msg1);
        manager.add_message(msg2);
        manager.add_message(msg3);

        let history = manager.get_history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].text, "Second");
        assert_eq!(history[1].text, "Third");
    }

    #[test]
    fn test_message_filtering() {
        let mut manager = StatusManager::new();
        
        let msg1 = StatusMessage::new("High priority", MessageType::Error)
            .with_priority(2);
        let msg2 = StatusMessage::new("Low priority", MessageType::Info)
            .with_priority(0);

        manager.add_message(msg1);
        manager.add_message(msg2);

        let high_priority = manager.get_high_priority_messages(2);
        assert_eq!(high_priority.len(), 1);
        assert_eq!(high_priority[0].text, "High priority");

        let info_messages = manager.get_messages_by_type(MessageType::Info);
        assert_eq!(info_messages.len(), 1);
        assert_eq!(info_messages[0].text, "Low priority");
    }

    #[test]
    fn test_message_history_clearing() {
        let mut manager = StatusManager::new();
        
        manager.add_message(StatusMessage::new("Test", MessageType::Info));
        assert_eq!(manager.get_history().len(), 1);

        manager.clear_history();
        assert_eq!(manager.get_history().len(), 0);
    }
} 