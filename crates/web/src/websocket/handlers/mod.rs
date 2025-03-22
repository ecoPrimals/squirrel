//! WebSocket handler trait and implementations
//!
//! This module contains the WebSocket handler trait and implementations
//! for different types of WebSocket handlers.

pub mod commands;

use async_trait::async_trait;
use crate::websocket::{WebSocketMessage, WebSocketContext, error::WebSocketError};

/// WebSocket handler trait
///
/// This trait defines the interface for WebSocket handlers that process
/// messages from clients and return responses.
#[async_trait]
pub trait WebSocketHandler: Send + Sync + 'static {
    /// Handle a WebSocket message
    ///
    /// # Arguments
    ///
    /// * `context` - The WebSocket connection context
    /// * `message` - The WebSocket message to handle
    ///
    /// # Returns
    ///
    /// * `Ok(Some(message))` - A response message to send back to the client
    /// * `Ok(None)` - No response needed
    /// * `Err(error)` - An error occurred while handling the message
    async fn handle_message(
        &self,
        context: &WebSocketContext,
        message: WebSocketMessage,
    ) -> Result<Option<WebSocketMessage>, WebSocketError>;
} 