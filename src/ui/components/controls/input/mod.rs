use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal,
};
use std::io::{self, Write};
use std::time::Duration;
use crate::ui::layout::LayoutManager;
use crate::ui::theme::{Theme, Themeable, ColorRole, ThemeError, Style};

/// Represents an input component in the UI.
///
/// # Examples
///
/// ```
/// use crate::ui::components::controls::Input;
/// use std::time::Duration;
///
/// let input = Input::with_timeout(Duration::from_secs(5));
/// if let Ok(line) = input.read_line_with_prompt("Enter your name: ") {
///     println!("Hello, {}", line);
/// }
/// ```
#[derive(Debug)]
pub struct Input {
    /// Optional timeout duration for input operations.
    timeout: Option<Duration>,
    /// The layout manager for positioning the input field.
    layout: LayoutManager,
    /// The style configuration for the input component.
    style: Style,
}

/// Error types that can occur during input operations.
#[derive(Debug, thiserror::Error)]
pub enum InputError {
    /// An I/O error occurred during input handling.
    #[error("IO error: {0}")]
    IO(#[from] io::Error),
    /// A timeout occurred while waiting for input.
    #[error("Input timeout")]
    Timeout,
    /// The input was invalid or could not be processed.
    #[error("Invalid input")]
    InvalidInput,
}

impl Input {
    /// Creates a new input component with no timeout.
    #[must_use]
    pub fn new() -> Self {
        Self {
            timeout: None,
            layout: LayoutManager::new(),
            style: Style::new(),
        }
    }

    /// Creates a new input component with the specified timeout duration.
    ///
    /// # Arguments
    /// * `timeout` - The maximum duration to wait for input
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// let input = Input::with_timeout(Duration::from_secs(5));
    /// ```
    #[must_use]
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            timeout: Some(timeout),
            layout: LayoutManager::new(),
            style: Style::new(),
        }
    }

    /// Waits for a key press and returns the key event.
    ///
    /// If a timeout is set, returns an error if no key is pressed within
    /// the timeout duration.
    ///
    /// # Errors
    ///
    /// Returns `InputError::Timeout` if no key is pressed within the timeout duration.
    /// Returns `InputError::IO` if an I/O error occurs.
    /// Returns `InputError::InvalidInput` if the event is not a key event.
    pub fn wait_for_key(&self) -> Result<KeyEvent, InputError> {
        let _raw = terminal::enable_raw_mode()?;
        let result = match event::poll(self.timeout.unwrap_or(Duration::from_millis(100)))? {
            true => match event::read()? {
                Event::Key(key) => Ok(key),
                _ => Err(InputError::InvalidInput),
            },
            false => Err(InputError::Timeout),
        };
        terminal::disable_raw_mode()?;
        result
    }

    /// Waits for a character input and returns it.
    ///
    /// Returns None if the key pressed was not a character key.
    /// If a timeout is set, returns an error if no key is pressed within
    /// the timeout duration.
    ///
    /// # Errors
    ///
    /// Returns `InputError::Timeout` if no key is pressed within the timeout duration.
    /// Returns `InputError::IO` if an I/O error occurs.
    pub fn wait_for_char(&self) -> Result<Option<char>, InputError> {
        let key = self.wait_for_key()?;
        Ok(match key.code {
            KeyCode::Char(c) => Some(c),
            _ => None,
        })
    }

    /// Reads a line of text from the user, ending with Enter.
    ///
    /// If a timeout is set, returns an error if the line is not completed
    /// within the timeout duration.
    ///
    /// # Errors
    ///
    /// Returns `InputError::Timeout` if input is not completed within the timeout duration.
    /// Returns `InputError::IO` if an I/O error occurs.
    /// Returns `InputError::InvalidInput` if Escape is pressed.
    pub fn read_line(&self) -> Result<String, InputError> {
        let mut input = String::new();
        let _raw = terminal::enable_raw_mode()?;

        loop {
            match event::poll(self.timeout.unwrap_or(Duration::from_millis(100)))? {
                true => {
                    if let Event::Key(key) = event::read()? {
                        match key.code {
                            KeyCode::Enter => break,
                            KeyCode::Char(c) => {
                                input.push(c);
                                print!("{c}");
                                io::stdout().flush()?;
                            }
                            KeyCode::Backspace => {
                                if !input.is_empty() {
                                    input.pop();
                                    print!("\x08 \x08"); // Move back, erase, move back
                                    io::stdout().flush()?;
                                }
                            }
                            KeyCode::Esc => {
                                terminal::disable_raw_mode()?;
                                return Err(InputError::InvalidInput);
                            }
                            _ => {}
                        }
                    }
                }
                false => continue,
            }
        }

        terminal::disable_raw_mode()?;
        println!(); // New line after input
        Ok(input)
    }

    /// Reads a line of text with a prompt displayed before the input.
    ///
    /// # Arguments
    /// * `prompt` - The text to display before the input line
    ///
    /// # Errors
    ///
    /// Returns `InputError::IO` if an I/O error occurs.
    /// Returns other `InputError` variants from `read_line`.
    pub fn read_line_with_prompt(&self, prompt: &str) -> Result<String, InputError> {
        print!("{prompt}");
        io::stdout().flush()?;
        self.read_line()
    }

    /// Checks if the Control key was pressed with the given key event.
    #[must_use]
    pub const fn is_ctrl_pressed(key: KeyEvent) -> bool {
        key.modifiers.contains(KeyModifiers::CONTROL)
    }

    /// Checks if the Alt key was pressed with the given key event.
    #[must_use]
    pub const fn is_alt_pressed(key: KeyEvent) -> bool {
        key.modifiers.contains(KeyModifiers::ALT)
    }

    /// Checks if the Shift key was pressed with the given key event.
    #[must_use]
    pub const fn is_shift_pressed(key: KeyEvent) -> bool {
        key.modifiers.contains(KeyModifiers::SHIFT)
    }
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

impl Themeable for Input {
    fn apply_theme(&mut self, theme: &Theme) -> Result<(), ThemeError> {
        self.style = theme.styles.input.clone();
        Ok(())
    }

    fn get_style(&self) -> &Style {
        &self.style
    }

    fn get_color(&self, role: ColorRole) -> crate::ui::theme::Color {
        match role {
            ColorRole::Primary => crate::ui::theme::Color::Blue,
            _ => crate::ui::theme::Color::White,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_input_creation() {
        let input = Input::new();
        assert_eq!(input.timeout, None);

        let custom_input = Input::with_timeout(Duration::from_secs(1));
        assert_eq!(custom_input.timeout, Some(Duration::from_secs(1)));
    }

    #[test]
    fn test_modifier_detection() {
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(Input::is_ctrl_pressed(key));

        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::ALT);
        assert!(Input::is_alt_pressed(key));

        let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::SHIFT);
        assert!(Input::is_shift_pressed(key));
    }

    #[test]
    fn test_theme_application() {
        let mut input = Input::new();
        let theme = Theme {
            name: "test".to_string(),
            colors: crate::ui::theme::ColorScheme {
                primary: crate::ui::theme::Color::Blue,
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
                text: Style::new(),
                input: Style::new().bold(),
                button: Style::new(),
                dialog: Style::new(),
            },
            metadata: crate::ui::theme::ThemeMetadata {
                version: "1.0.0".to_string(),
                author: "Test".to_string(),
                description: "Test theme".to_string(),
            },
        };

        assert!(input.apply_theme(&theme).is_ok());
        assert!(input.get_style().attributes.contains(&crate::ui::theme::Attribute::Bold));
        assert_eq!(input.get_color(ColorRole::Primary), crate::ui::theme::Color::Blue);
    }
} 