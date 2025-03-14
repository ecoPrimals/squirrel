use std::fmt;
use std::io;
use crate::ui::layout;
use crate::ui::theme;

/// Error types that can occur during UI operations.
#[derive(Debug)]
pub enum UiError {
    /// An I/O error occurred during UI operations.
    Io(io::Error),
    /// An error occurred in the layout system.
    Layout(layout::LayoutError),
    /// Attempted to use grid layout features when no grid layout was configured.
    NoGridLayout,
    /// An error occurred while processing theme operations.
    Theme(theme::ThemeError),
    /// An error occurred while processing a component operation.
    Component(String),
    /// The component was in an invalid state for the requested operation.
    InvalidState(String),
    /// A mutex lock operation failed.
    LockError,
    /// Invalid input was provided to a component.
    InvalidInput(String),
}

impl fmt::Display for UiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {}", err),
            Self::Layout(err) => write!(f, "Layout error: {}", err),
            Self::NoGridLayout => write!(f, "No grid layout configured"),
            Self::Theme(err) => write!(f, "Theme error: {}", err),
            Self::Component(msg) => write!(f, "Component error: {}", msg),
            Self::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
            Self::LockError => write!(f, "Failed to acquire lock"),
            Self::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl std::error::Error for UiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Layout(err) => Some(err),
            Self::Theme(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for UiError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<layout::LayoutError> for UiError {
    fn from(err: layout::LayoutError) -> Self {
        Self::Layout(err)
    }
}

impl From<theme::ThemeError> for UiError {
    fn from(err: theme::ThemeError) -> Self {
        Self::Theme(err)
    }
}

impl From<String> for UiError {
    fn from(msg: String) -> Self {
        Self::Component(msg)
    }
}

impl From<&str> for UiError {
    fn from(msg: &str) -> Self {
        Self::Component(msg.to_string())
    }
} 