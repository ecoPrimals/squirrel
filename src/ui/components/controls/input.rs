use std::time::Duration;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use thiserror::Error;
use ratatui::layout::Position;

/// Module providing input handling functionality for the terminal user interface.
/// This includes keyboard event handling, input modes, and text input management.

#[derive(Error, Debug)]
pub enum InputError {
    /// An IO error occurred during input operations.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    /// A timeout occurred while waiting for input.
    #[error("Input timeout")]
    Timeout,
    /// The input mode specified was invalid.
    #[error("Invalid input mode: {0}")]
    InvalidMode(String),
}

/// A type alias for Result with InputError as the error type.
pub type Result<T> = std::result::Result<T, InputError>;

/// Represents different modes of input handling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    /// Normal mode for navigation and commands.
    Normal,
    /// Insert mode for text input.
    Insert,
    /// Command mode for entering commands.
    Command,
    /// Search mode for searching content.
    Search,
}

/// Represents a keyboard input event with its associated mode and modifiers.
pub struct InputEvent {
    /// The key code of the input event.
    pub code: KeyCode,
    /// Any modifier keys (Ctrl, Alt, Shift) that were pressed.
    pub modifiers: KeyModifiers,
    /// The current input mode when the event occurred.
    pub mode: InputMode,
}

/// Represents the result of handling an input event.
pub enum InputResult {
    /// The input was handled successfully.
    Handled,
    /// The input resulted in a submission with the given string.
    Submit(String),
    /// The input operation was cancelled.
    Cancel,
    /// The input was not handled.
    Ignored,
}

/// Handles keyboard input events and manages input state.
pub struct InputHandler {
    /// The current input mode (normal, insert, command, or search).
    mode: InputMode,
    /// The timeout duration for input operations.
    timeout: Duration,
    /// Whether raw mode is enabled for direct input handling.
    raw_mode: bool,
    /// The current input value.
    value: String,
    /// The current cursor position in the input value.
    cursor_position: usize,
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl InputHandler {
    /// Creates a new InputHandler with default settings.
    pub fn new() -> Self {
        Self {
            mode: InputMode::Normal,
            timeout: Duration::from_millis(100),
            raw_mode: false,
            value: String::new(),
            cursor_position: 0,
        }
    }

    /// Sets a timeout duration for input operations.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the initial input mode.
    pub fn with_mode(mut self, mode: InputMode) -> Self {
        self.mode = mode;
        self
    }

    /// Enables raw mode for terminal input.
    pub fn enable_raw_mode(&mut self) -> Result<()> {
        if !self.raw_mode {
            enable_raw_mode()?;
            self.raw_mode = true;
        }
        Ok(())
    }

    /// Disables raw mode for terminal input.
    pub fn disable_raw_mode(&mut self) -> Result<()> {
        if self.raw_mode {
            disable_raw_mode()?;
            self.raw_mode = false;
        }
        Ok(())
    }

    /// Sets the current input mode.
    pub fn set_mode(&mut self, mode: InputMode) {
        self.mode = mode;
    }

    /// Gets the current input mode.
    pub fn get_mode(&self) -> InputMode {
        self.mode
    }

    /// Waits for a key press with timeout if configured.
    pub fn wait_for_key(&mut self) -> Result<InputEvent> {
        if !self.raw_mode {
            self.enable_raw_mode()?;
        }

        if event::poll(self.timeout)? {
            if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                return Ok(InputEvent {
                    code,
                    modifiers,
                    mode: self.mode,
                });
            }
        }

        Err(InputError::Timeout)
    }

    /// Waits for a key press without timeout.
    pub fn wait_for_key_blocking(&mut self) -> Result<InputEvent> {
        if !self.raw_mode {
            self.enable_raw_mode()?;
        }

        if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
            return Ok(InputEvent {
                code,
                modifiers,
                mode: self.mode,
            });
        }

        Err(InputError::Timeout)
    }

    /// Checks if the given event is a navigation key event.
    pub fn is_navigation_key(&self, event: &InputEvent) -> bool {
        matches!(
            event.code,
            KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right |
            KeyCode::Home | KeyCode::End | KeyCode::PageUp | KeyCode::PageDown
        )
    }

    /// Handles mode changes based on input events.
    pub fn handle_mode_change(&mut self, event: &InputEvent) -> Option<InputMode> {
        match (self.mode, event.code, event.modifiers) {
            (InputMode::Normal, KeyCode::Char('i'), _) => {
                self.mode = InputMode::Insert;
                Some(InputMode::Insert)
            }
            (InputMode::Normal, KeyCode::Char(':'), _) => {
                self.mode = InputMode::Command;
                Some(InputMode::Command)
            }
            (InputMode::Normal, KeyCode::Char('/'), _) => {
                self.mode = InputMode::Search;
                Some(InputMode::Search)
            }
            (_, KeyCode::Esc, _) => {
                self.mode = InputMode::Normal;
                Some(InputMode::Normal)
            }
            _ => None,
        }
    }

    /// Handles an input event and returns the result of the handling.
    pub fn handle_input(&mut self, event: Event) -> InputResult {
        match event {
            Event::Key(key) => match (key.code, key.modifiers) {
                (KeyCode::Char(c), _) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.handle_char(c);
                    InputResult::Handled
                }
                (KeyCode::Backspace, _) => {
                    self.handle_backspace();
                    InputResult::Handled
                }
                (KeyCode::Delete, _) => {
                    self.handle_delete();
                    InputResult::Handled
                }
                (KeyCode::Left, _) => {
                    self.handle_left();
                    InputResult::Handled
                }
                (KeyCode::Right, _) => {
                    self.handle_right();
                    InputResult::Handled
                }
                (KeyCode::Home, _) => {
                    self.handle_home();
                    InputResult::Handled
                }
                (KeyCode::End, _) => {
                    self.handle_end();
                    InputResult::Handled
                }
                (KeyCode::Enter, _) => InputResult::Submit(self.value.clone()),
                (KeyCode::Char('c' | 'd'), KeyModifiers::CONTROL) | (KeyCode::Esc, _) => InputResult::Cancel,
                _ => InputResult::Ignored,
            },
            _ => InputResult::Ignored,
        }
    }

    /// Handles a character input event by inserting it at the current cursor position.
    fn handle_char(&mut self, c: char) {
        self.value.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    /// Handles a backspace key event by removing the character before the cursor.
    fn handle_backspace(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.value.remove(self.cursor_position);
        }
    }

    /// Handles a delete key event by removing the character at the cursor position.
    fn handle_delete(&mut self) {
        if self.cursor_position < self.value.len() {
            self.value.remove(self.cursor_position);
        }
    }

    /// Moves the cursor one position to the left if possible.
    fn handle_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Moves the cursor one position to the right if possible.
    fn handle_right(&mut self) {
        if self.cursor_position < self.value.len() {
            self.cursor_position += 1;
        }
    }

    /// Moves the cursor to the start of the input field.
    fn handle_home(&mut self) {
        self.cursor_position = 0;
    }

    /// Moves the cursor to the end of the input field.
    fn handle_end(&mut self) {
        self.cursor_position = self.value.len();
    }

    /// Gets the current input value.
    pub fn get_value(&self) -> &str {
        &self.value
    }

    /// Gets the current cursor position.
    pub fn get_cursor_position(&self) -> usize {
        self.cursor_position
    }

    /// Clears the current input value.
    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor_position = 0;
    }

    /// Checks if the given key event represents a command key
    pub fn is_command_key(&self, event: &KeyEvent) -> bool {
        matches!(event.modifiers, KeyModifiers::CONTROL)
    }
}

impl Drop for InputHandler {
    fn drop(&mut self) {
        let _ = self.disable_raw_mode();
    }
}

/// Represents an input field in the UI.
pub struct Field {
    /// The label text displayed next to the input field.
    label: String,
    /// The current value of the input field.
    value: String,
    /// The position of the input field in the UI.
    position: Position,
    /// Whether the input field currently has focus.
    is_focused: bool,
    /// Whether the input field should mask its value (for passwords).
    is_password: bool,
}

impl Field {
    /// Creates a new input field with the given label.
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            value: String::new(),
            position: Position::default(),
            is_focused: false,
            is_password: false,
        }
    }

    /// Creates a new password input field with the given label.
    pub fn new_password(label: &str) -> Self {
        Self {
            label: label.to_string(),
            value: String::new(),
            position: Position::default(),
            is_focused: false,
            is_password: true,
        }
    }

    /// Gets the label of the field.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Sets the position of the input field.
    pub fn set_position(&mut self, x: u16, y: u16) {
        self.position = Position::new(x, y);
    }

    /// Gets the current value of the input field.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Sets the value of the input field.
    pub fn set_value(&mut self, value: &str) {
        self.value = value.to_string();
    }

    /// Sets the focus state of the input field.
    pub fn set_focus(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    /// Returns whether the input field is focused.
    #[must_use]
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    /// Returns whether the input field is a password field.
    #[must_use]
    pub fn is_password(&self) -> bool {
        self.is_password
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_mode_changes() {
        let mut handler = InputHandler::new();
        assert_eq!(handler.get_mode(), InputMode::Normal);

        handler.set_mode(InputMode::Insert);
        assert_eq!(handler.get_mode(), InputMode::Insert);

        handler.set_mode(InputMode::Command);
        assert_eq!(handler.get_mode(), InputMode::Command);

        handler.set_mode(InputMode::Search);
        assert_eq!(handler.get_mode(), InputMode::Search);
    }

    #[test]
    fn test_command_key_detection() {
        let handler = InputHandler::new();
        
        let ctrl_c = KeyEvent::new(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL,
        );
        assert!(handler.is_command_key(&ctrl_c));

        let normal_key = KeyEvent::new(
            KeyCode::Char('a'),
            KeyModifiers::NONE,
        );
        assert!(!handler.is_command_key(&normal_key));
    }

    #[test]
    fn test_navigation_key_detection() {
        let handler = InputHandler::new();
        
        let up_key = InputEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            mode: InputMode::Normal,
        };
        assert!(handler.is_navigation_key(&up_key));

        let normal_key = InputEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::NONE,
            mode: InputMode::Normal,
        };
        assert!(!handler.is_navigation_key(&normal_key));
    }

    #[test]
    fn test_mode_change_handling() {
        let mut handler = InputHandler::new();
        
        let insert_key = InputEvent {
            code: KeyCode::Char('i'),
            modifiers: KeyModifiers::NONE,
            mode: InputMode::Normal,
        };
        assert_eq!(handler.handle_mode_change(&insert_key), Some(InputMode::Insert));

        let esc_key = InputEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            mode: InputMode::Insert,
        };
        assert_eq!(handler.handle_mode_change(&esc_key), Some(InputMode::Normal));
    }

    #[test]
    fn test_key_handling() {
        let handler = InputHandler::new();
        let normal_key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        
        assert!(handler.is_command_key(&ctrl_c));
        assert!(!handler.is_command_key(&normal_key));
    }

    #[test]
    fn test_input_handling() {
        let _handler = InputHandler::new();
        let _normal_input = InputEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::NONE,
            mode: InputMode::Normal,
        };
        // ... rest of the test ...
    }
} 