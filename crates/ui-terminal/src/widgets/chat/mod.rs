//! Chat widget module for the terminal UI
//! This module contains the chat widget and related functionality

mod types;
mod message;
mod state;
mod render;
mod persistence;

pub use types::InputMode;
pub use message::ChatMessage;
pub use state::ChatState;
pub use render::render; 