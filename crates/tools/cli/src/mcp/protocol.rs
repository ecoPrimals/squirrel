// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! MCP protocol implementation
//!
//! Defines the message structures and serialization for the Machine Context Protocol.

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// MCP message type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MCPMessageType {
    /// Request message
    Request,

    /// Response message
    Response,

    /// Notification message (one-way)
    Notification,

    /// Error message
    Error,
}

/// MCP error - simplified for CLI operations
#[derive(Debug, Error)]
pub enum MCPError {
    /// Protocol error
    #[error("Protocol error: {0}")]
    ProtocolError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Command error
    #[error("Command error: {0}")]
    CommandError(String),

    /// Connection error
    #[error("Connection error: {0}")]
    ConnectionError(String),
}

/// MCP result type
pub type MCPResult<T> = Result<T, MCPError>;

/// MCP message
///
/// Represents a message in the Machine Context Protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPMessage {
    /// Message ID (for correlation)
    pub id: String,

    /// Message type
    #[serde(rename = "type")]
    pub message_type: MCPMessageType,

    /// Command or topic
    pub command: String,

    /// Message payload
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,

    /// Error message (for error responses)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl MCPMessage {
    /// Create a new request message
    ///
    /// # Arguments
    ///
    /// * `id` - Message ID
    /// * `command` - Command name
    /// * `payload` - Optional payload
    ///
    /// # Returns
    ///
    /// A new request message
    pub fn new_request(id: String, command: String, payload: Option<serde_json::Value>) -> Self {
        Self {
            id,
            message_type: MCPMessageType::Request,
            command,
            payload,
            error: None,
        }
    }

    /// Create a new response message
    ///
    /// # Arguments
    ///
    /// * `id` - Message ID
    /// * `command` - Command name
    /// * `payload` - Optional payload
    ///
    /// # Returns
    ///
    /// A new response message
    pub fn new_response(id: String, command: String, payload: Option<serde_json::Value>) -> Self {
        Self {
            id,
            message_type: MCPMessageType::Response,
            command,
            payload,
            error: None,
        }
    }

    /// Create a new notification message
    ///
    /// # Arguments
    ///
    /// * `id` - Message ID
    /// * `topic` - Notification topic
    /// * `payload` - Optional payload
    ///
    /// # Returns
    ///
    /// A new notification message
    pub fn new_notification(id: String, topic: String, payload: Option<serde_json::Value>) -> Self {
        Self {
            id,
            message_type: MCPMessageType::Notification,
            command: topic,
            payload,
            error: None,
        }
    }

    /// Create a new error message
    ///
    /// # Arguments
    ///
    /// * `id` - Message ID
    /// * `command` - Command name
    /// * `error` - Error message
    ///
    /// # Returns
    ///
    /// A new error message
    pub fn new_error(id: String, command: String, error: String) -> Self {
        Self {
            id,
            message_type: MCPMessageType::Error,
            command,
            payload: None,
            error: Some(error),
        }
    }

    /// Convert the message to JSON
    ///
    /// # Returns
    ///
    /// A Result containing the JSON string or an error
    pub fn to_json(&self) -> MCPResult<String> {
        serde_json::to_string(self).map_err(MCPError::SerializationError)
    }

    /// Parse a message from JSON
    ///
    /// # Arguments
    ///
    /// * `json` - JSON string
    ///
    /// # Returns
    ///
    /// A Result containing the parsed message or an error
    pub fn from_json(json: &str) -> MCPResult<Self> {
        serde_json::from_str(json).map_err(MCPError::SerializationError)
    }
}

impl fmt::Display for MCPMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}] {} {}", self.message_type, self.id, self.command)?;

        if let Some(payload) = &self.payload {
            write!(f, " payload: {}", payload)?;
        }

        if let Some(error) = &self.error {
            write!(f, " error: {}", error)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_message_request_creation() {
        let id = "test-id".to_string();
        let command = "test-command".to_string();
        let payload = Some(json!({"key": "value"}));

        // Use references to avoid unnecessary clones in test
        let message = MCPMessage::new_request(id.clone(), command.clone(), payload.clone());

        assert_eq!(message.id, id);
        assert_eq!(message.message_type, MCPMessageType::Request);
        assert_eq!(message.command, command);
        assert_eq!(message.payload, payload);
        assert!(message.error.is_none());
    }

    #[test]
    fn test_message_response_creation() {
        let id = "test-id".to_string();
        let command = "test-command".to_string();
        let payload = Some(json!({"result": "success"}));

        let message = MCPMessage::new_response(id.clone(), command.clone(), payload.clone());

        assert_eq!(message.id, id);
        assert_eq!(message.message_type, MCPMessageType::Response);
        assert_eq!(message.command, command);
        assert_eq!(message.payload, payload);
        assert!(message.error.is_none());
    }

    #[test]
    fn test_message_notification_creation() {
        let id = "test-id".to_string();
        let topic = "test-topic".to_string();
        let payload = Some(json!({"event": "something-happened"}));

        let message = MCPMessage::new_notification(id.clone(), topic.clone(), payload.clone());

        assert_eq!(message.id, id);
        assert_eq!(message.message_type, MCPMessageType::Notification);
        assert_eq!(message.command, topic);
        assert_eq!(message.payload, payload);
        assert!(message.error.is_none());
    }

    #[test]
    fn test_message_error_creation() {
        let id = "test-id".to_string();
        let command = "test-command".to_string();
        let error = "Something went wrong".to_string();

        let message = MCPMessage::new_error(id.clone(), command.clone(), error.clone());

        assert_eq!(message.id, id);
        assert_eq!(message.message_type, MCPMessageType::Error);
        assert_eq!(message.command, command);
        assert!(message.payload.is_none());
        assert_eq!(message.error, Some(error));
    }

    #[test]
    fn test_message_serialization() {
        let message = MCPMessage::new_request(
            "req-1".to_string(),
            "test".to_string(),
            Some(json!({"arg": "value"})),
        );

        let json = message.to_json().unwrap();
        let parsed = MCPMessage::from_json(&json).unwrap();

        assert_eq!(parsed.id, message.id);
        assert_eq!(parsed.message_type, message.message_type);
        assert_eq!(parsed.command, message.command);
        assert_eq!(parsed.payload, message.payload);
        assert_eq!(parsed.error, message.error);
    }

    #[test]
    fn test_error_message_serialization() {
        let message = MCPMessage::new_error(
            "err-1".to_string(),
            "test".to_string(),
            "Error message".to_string(),
        );

        let json = message.to_json().unwrap();
        let parsed = MCPMessage::from_json(&json).unwrap();

        assert_eq!(parsed.id, message.id);
        assert_eq!(parsed.message_type, message.message_type);
        assert_eq!(parsed.command, message.command);
        assert_eq!(parsed.payload, message.payload);
        assert_eq!(parsed.error, message.error);
    }
}
