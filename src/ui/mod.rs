pub mod components;
pub mod events;
pub mod layout;
pub mod theme;
pub mod terminal;
pub mod gui;

// Re-exports
pub use components::{Component, Container};
pub use events::Event;
pub use layout::{Layout, Rect, Size};
pub use theme::{Theme, Themeable, ColorRole, StyleRole, Style};
pub use terminal::Terminal;
pub use gui::Gui;

// Error type for UI operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Component error: {0}")]
    Component(#[from] components::error::ComponentError),
    #[error("Theme error: {0}")]
    Theme(#[from] theme::ThemeError),
    #[error("Terminal error: {0}")]
    Terminal(String),
    #[error("Lock error")]
    Lock,
}

pub type Result<T> = std::result::Result<T, Error>; 