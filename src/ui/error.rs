use std::io;
use thiserror::Error;
use crate::ui::components::registry::ComponentId;
use std::fmt;

/// Error types that can occur during UI operations.
#[derive(Debug, Error)]
pub enum ComponentError {
    /// An I/O error occurred during UI operations.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// A theme error occurred.
    #[error("Theme error: {0}")]
    Theme(String),

    /// A lock error occurred.
    #[error("Lock error: {0}")]
    Lock(String),

    /// A component-specific error occurred.
    #[error("Component error: {0}")]
    Component(String),

    /// A layout error occurred.
    #[error("Layout error: {0}")]
    Layout(String),

    /// An event error occurred.
    #[error("Event error: {0}")]
    Event(String),

    /// An unknown error occurred.
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<ComponentError> for io::Error {
    fn from(error: ComponentError) -> Self {
        io::Error::new(io::ErrorKind::Other, error.to_string())
    }
}

#[derive(Debug, Error)]
pub enum UiError {
    /// A component error occurred.
    #[error("Component error: {0}")]
    Component(#[from] ComponentError),

    /// A theme error occurred.
    #[error("Theme error: {0}")]
    Theme(String),

    /// A layout error occurred.
    #[error("Layout error: {0}")]
    Layout(String),

    /// An event error occurred.
    #[error("Event error: {0}")]
    Event(String),

    /// A terminal error occurred.
    #[error("Terminal error: {0}")]
    Terminal(String),

    /// An unknown error occurred.
    #[error("Unknown error: {0}")]
    Unknown(String),

    /// A component was not found.
    #[error("Component not found: {0}")]
    ComponentNotFound(ComponentId),

    /// A component operation failed.
    #[error("Component operation failed: {0}")]
    ComponentOperation(String),

    /// An input error occurred.
    #[error("Input error: {0}")]
    Input(String),

    /// A rendering error occurred.
    #[error("Rendering error: {0}")]
    Render(String),

    /// A state error occurred.
    #[error("State error: {0}")]
    State(String),

    /// A validation error occurred.
    #[error("Validation error: {0}")]
    Validation(String),

    /// A focus error occurred.
    #[error("Focus error: {0}")]
    Focus(String),

    /// A scroll error occurred.
    #[error("Scroll error: {0}")]
    Scroll(String),

    /// A resize error occurred.
    #[error("Resize error: {0}")]
    Resize(String),

    /// A theme application error occurred.
    #[error("Theme application error: {0}")]
    ThemeApplication(String),

    /// An event handling error occurred.
    #[error("Event handling error: {0}")]
    EventHandling(String),
}

/// Result type for UI operations.
pub type UiResult<T> = Result<T, UiError>;

impl From<&str> for UiError {
    fn from(s: &str) -> Self {
        UiError::Component(ComponentError::Unknown(s.to_string()))
    }
}

impl From<String> for UiError {
    fn from(s: String) -> Self {
        UiError::Component(ComponentError::Unknown(s))
    }
}

impl From<UiError> for io::Error {
    fn from(error: UiError) -> Self {
        io::Error::new(io::ErrorKind::Other, error.to_string())
    }
}

impl From<ComponentError> for UiError {
    fn from(error: ComponentError) -> Self {
        UiError::Component(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion() {
        let error = UiError::from("test error");
        assert!(matches!(error, UiError::Component(_)));

        let error = UiError::from(String::from("test error"));
        assert!(matches!(error, UiError::Component(_)));

        let io_error = io::Error::new(io::ErrorKind::Other, "test error");
        let error = UiError::from(io_error);
        assert!(matches!(error, UiError::Io(_)));
    }

    #[test]
    fn test_error_display() {
        let error = UiError::Component("test error".to_string());
        assert_eq!(error.to_string(), "Component error: test error");

        let error = UiError::Layout("invalid layout".to_string());
        assert_eq!(error.to_string(), "Layout error: invalid layout");

        let error = UiError::Theme("invalid theme".to_string());
        assert_eq!(error.to_string(), "Theme error: invalid theme");
    }
} 