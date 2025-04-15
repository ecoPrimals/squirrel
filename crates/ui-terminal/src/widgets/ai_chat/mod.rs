//! AI Chat widget module for the terminal UI

pub mod state;
pub mod models;
pub mod messages;
pub mod render;
pub mod tests;
pub mod api;
pub mod widget;

pub use state::AiChatWidgetState;
pub use models::AiModel;
pub use messages::ChatMessage;

// Re-export important structs and functions
pub use render::render;
pub use api::send_message; 