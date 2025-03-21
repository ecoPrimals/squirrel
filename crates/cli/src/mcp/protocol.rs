//! MCP protocol implementation
//!
//! Defines the message structures and serialization for the Machine Context Protocol.

use std::fmt;
use serde::{Serialize, Deserialize};
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

/// MCP error
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
    /// * `id` - Message ID (should match the request ID)
    /// * `command` - Command name (should match the request command)
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
    /// * `id` - Message ID (should match the request ID)
    /// * `command` - Command name (should match the request command)
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
        serde_json::to_string(self)
            .map_err(MCPError::from)
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
        serde_json::from_str(json)
            .map_err(MCPError::from)
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