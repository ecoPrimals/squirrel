//! WebSocket module for real-time communication with clients.
//!
//! This module handles WebSocket connections, message processing, and broadcasting
//! to support real-time features in the Squirrel platform.

pub mod commands;
pub mod error;
mod handler;
mod manager;
mod models;

#[cfg(test)]
mod tests;

pub use commands::CommandHandler;
pub use error::WebSocketError;
pub use handler::ws_handler;
pub use manager::ConnectionManager;
pub use models::{ChannelCategory, Subscription, WebSocketCommand, WebSocketEvent, WebSocketResponse};

/// Initialize the WebSocket module
pub fn init() -> ConnectionManager {
    ConnectionManager::new()
}

/// Helper function to create a channel ID based on category and channel name
pub fn make_channel_id(category: ChannelCategory, channel: &str) -> String {
    format!("{}:{}", category.as_str(), channel)
} 