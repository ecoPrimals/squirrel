// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Message handling for MCP protocol
//!
//! This module handles message serialization, deserialization, and processing
//! for the MCP protocol communication.

use super::connection::ConnectionManager;
use super::types::McpMessage;
use tracing::{debug, info, warn};

/// Message handler for MCP protocol
///
/// Handles message serialization, deserialization, and protocol-specific
/// message processing including notifications and responses.
#[derive(Debug)]
pub struct MessageHandler {
    /// Message counter for generating unique IDs
    message_counter: u64,
}

impl MessageHandler {
    /// Create a new message handler
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use squirrel_sdk::communication::mcp::message::MessageHandler;
    ///
    /// let handler = MessageHandler::new();
    /// ```
    pub fn new() -> Self {
        Self { message_counter: 0 }
    }

    /// Send a message through the connection
    ///
    /// Serializes the message and sends it through the WebSocket connection.
    ///
    /// # Arguments
    ///
    /// * `connection` - The connection manager to use for sending
    /// * `message` - The message to send
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the message is sent successfully, or an error if it fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use squirrel_sdk::communication::mcp::message::MessageHandler;
    /// use squirrel_sdk::communication::mcp::connection::ConnectionManager;
    /// use squirrel_sdk::communication::mcp::types::McpMessage;
    /// use squirrel_sdk::config::McpClientConfig;
    /// use serde_json::json;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = McpClientConfig::default();
    /// let mut connection = ConnectionManager::new(config.clone());
    /// let mut handler = MessageHandler::new();
    ///
    /// let message = McpMessage {
    ///     id: "test-id".to_string(),
    ///     message_type: "ping".to_string(),
    ///     payload: json!({}),
    ///     timestamp: "2024-01-01T00:00:00Z".to_string(),
    /// };
    ///
    /// // connection.establish_connection(&config).await?;
    /// // handler.send_message(&mut connection, &message).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_message(
        &mut self,
        connection: &mut ConnectionManager,
        message: &McpMessage,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let message_json = serde_json::to_string(message)?;
        debug!("Sending MCP message: {}", message_json);

        connection.send_message(&message_json).await?;
        Ok(())
    }

    /// Handle an incoming message
    ///
    /// Deserializes and processes incoming messages from the WebSocket connection.
    ///
    /// # Arguments
    ///
    /// * `message_json` - The raw message JSON string
    ///
    /// # Returns
    ///
    /// Returns the deserialized message or an error if parsing fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use squirrel_sdk::communication::mcp::message::MessageHandler;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut handler = MessageHandler::new();
    /// let message_json = r#"{"id":"test","message_type":"ping","payload":{},"timestamp":"2024-01-01T00:00:00Z"}"#;
    ///
    /// let message = handler.handle_incoming_message(message_json).await?;
    /// assert_eq!(message.id, "test");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn handle_incoming_message(
        &mut self,
        message_json: &str,
    ) -> Result<McpMessage, Box<dyn std::error::Error + Send + Sync>> {
        let message: McpMessage = serde_json::from_str(message_json)?;
        debug!("Received MCP message: {:?}", message);
        Ok(message)
    }

    /// Handle a notification message
    ///
    /// Processes notification messages that don't require a response.
    ///
    /// # Arguments
    ///
    /// * `message` - The notification message
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the notification is processed successfully.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use squirrel_sdk::communication::mcp::message::MessageHandler;
    /// use squirrel_sdk::communication::mcp::types::McpMessage;
    /// use serde_json::json;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut handler = MessageHandler::new();
    /// let notification = McpMessage {
    ///     id: "notification-1".to_string(),
    ///     message_type: "notification".to_string(),
    ///     payload: json!({"type": "resource_updated"}),
    ///     timestamp: "2024-01-01T00:00:00Z".to_string(),
    /// };
    ///
    /// handler.handle_notification(&notification).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn handle_notification(
        &mut self,
        message: &McpMessage,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match message.message_type.as_str() {
            "ping" => {
                debug!("Received ping from server");
                // Ping handling is typically done at the client level
                // to send pong responses
            }
            "notification" => {
                debug!("Received notification: {:?}", message.payload);
                info!("MCP notification received: {}", message.payload);

                // Extract notification type and content
                if let Some(notification_type) = message.payload.get("type") {
                    match notification_type.as_str() {
                        Some("resource_updated") => {
                            debug!("Resource updated notification");
                        }
                        Some("tool_changed") => {
                            debug!("Tool changed notification");
                        }
                        Some("server_status") => {
                            debug!("Server status notification");
                        }
                        _ => {
                            debug!("Unknown notification type: {}", notification_type);
                        }
                    }
                }
            }
            _ => {
                warn!("Received unknown message type: {}", message.message_type);
            }
        }
        Ok(())
    }

    /// Generate a unique message ID
    ///
    /// Creates a unique message ID for outgoing messages.
    ///
    /// # Returns
    ///
    /// A unique string ID for the message.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use squirrel_sdk::communication::mcp::message::MessageHandler;
    ///
    /// let mut handler = MessageHandler::new();
    /// let id1 = handler.generate_message_id();
    /// let id2 = handler.generate_message_id();
    /// assert_ne!(id1, id2);
    /// ```
    pub fn generate_message_id(&mut self) -> String {
        self.message_counter += 1;
        format!("msg_{}", self.message_counter)
    }

    /// Create a ping message
    ///
    /// Creates a ping message for connection health checks.
    ///
    /// # Returns
    ///
    /// A ping message ready to be sent.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use squirrel_sdk::communication::mcp::message::MessageHandler;
    ///
    /// let mut handler = MessageHandler::new();
    /// let ping = handler.create_ping_message();
    /// assert_eq!(ping.message_type, "ping");
    /// ```
    pub fn create_ping_message(&mut self) -> McpMessage {
        McpMessage {
            id: self.generate_message_id(),
            message_type: "ping".to_string(),
            payload: serde_json::json!({
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Create a pong response message
    ///
    /// Creates a pong response message in reply to a ping.
    ///
    /// # Arguments
    ///
    /// * `ping_id` - The ID of the ping message being responded to
    ///
    /// # Returns
    ///
    /// A pong response message.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use squirrel_sdk::communication::mcp::message::MessageHandler;
    ///
    /// let mut handler = MessageHandler::new();
    /// let pong = handler.create_pong_message("ping-123");
    /// assert_eq!(pong.message_type, "pong");
    /// ```
    pub fn create_pong_message(&mut self, ping_id: &str) -> McpMessage {
        McpMessage {
            id: format!("pong_{}", ping_id),
            message_type: "pong".to_string(),
            payload: serde_json::json!({
                "ping_id": ping_id,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

impl Default for MessageHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_message_handler_creation() {
        let handler = MessageHandler::new();
        assert_eq!(handler.message_counter, 0);
    }

    #[test]
    fn test_generate_message_id() {
        let mut handler = MessageHandler::new();
        let id1 = handler.generate_message_id();
        let id2 = handler.generate_message_id();

        assert_eq!(id1, "msg_1");
        assert_eq!(id2, "msg_2");
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_create_ping_message() {
        let mut handler = MessageHandler::new();
        let ping = handler.create_ping_message();

        assert_eq!(ping.message_type, "ping");
        assert_eq!(ping.id, "msg_1");
        assert!(ping.payload.get("timestamp").is_some());
    }

    #[test]
    fn test_create_pong_message() {
        let mut handler = MessageHandler::new();
        let pong = handler.create_pong_message("ping-123");

        assert_eq!(pong.message_type, "pong");
        assert_eq!(pong.id, "pong_ping-123");
        assert_eq!(pong.payload["ping_id"], "ping-123");
        assert!(pong.payload.get("timestamp").is_some());
    }

    #[tokio::test]
    async fn test_handle_incoming_message() {
        let mut handler = MessageHandler::new();
        let message_json = r#"
        {
            "id": "test-id",
            "message_type": "ping",
            "payload": {"timestamp": "2024-01-01T00:00:00Z"},
            "timestamp": "2024-01-01T00:00:00Z"
        }
        "#;

        let message = handler.handle_incoming_message(message_json).await.unwrap();
        assert_eq!(message.id, "test-id");
        assert_eq!(message.message_type, "ping");
    }

    #[tokio::test]
    async fn test_handle_invalid_message() {
        let mut handler = MessageHandler::new();
        let invalid_json = r#"{"invalid": json}"#;

        let result = handler.handle_incoming_message(invalid_json).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_notification() {
        let mut handler = MessageHandler::new();

        // Test ping notification
        let ping_message = McpMessage {
            id: "ping-123".to_string(),
            message_type: "ping".to_string(),
            payload: json!({}),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        let result = handler.handle_notification(&ping_message).await;
        assert!(result.is_ok());

        // Test notification with type
        let notification_message = McpMessage {
            id: "notification-123".to_string(),
            message_type: "notification".to_string(),
            payload: json!({
                "type": "resource_updated",
                "resource": "test-resource"
            }),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        let result = handler.handle_notification(&notification_message).await;
        assert!(result.is_ok());

        // Test unknown notification
        let unknown_message = McpMessage {
            id: "unknown-123".to_string(),
            message_type: "unknown_type".to_string(),
            payload: json!({}),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        let result = handler.handle_notification(&unknown_message).await;
        assert!(result.is_ok());
    }
}
