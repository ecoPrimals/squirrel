// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Protocol-related types for MCP

use chrono; // Needed for MCPMessage
use serde::{Deserialize, Serialize};
use serde_json; // Needed for MCPMessage
use std::time::SystemTime;

use crate::error::Result;
#[cfg(feature = "websocket")]
use tokio_tungstenite::tungstenite::Message;
use uuid; // Needed for MessageId

/// Message type for MCP communications.
///
/// This enumeration defines the different types of messages that can be
/// exchanged within the MCP system. Each type serves a specific purpose
/// in the communication protocol.
///
/// # Usage
///
/// The message type determines how a message is processed by the system:
/// - Command messages trigger actions in the system
/// - Response messages return results from commands
/// - Event messages notify about system changes
/// - Error messages indicate problems
/// - Setup messages are used during protocol initialization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageType {
    /// Command message: Requests an action to be performed
    Command,
    /// Response message: Returns results from a command
    Response,
    /// Event message: One-way notification without expected response
    Event,
    /// Error message: Indicates a problem occurred
    Error,
    /// Setup message: Used for protocol initialization and negotiation
    Setup,
    /// Heartbeat message: Used for connection health monitoring
    Heartbeat,
    /// Sync message: Used for state synchronization
    Sync,
    /// Unknown message type
    Unknown,
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Command => write!(f, "Command"),
            Self::Response => write!(f, "Response"),
            Self::Event => write!(f, "Event"),
            Self::Error => write!(f, "Error"),
            Self::Setup => write!(f, "Setup"),
            Self::Heartbeat => write!(f, "Heartbeat"),
            Self::Sync => write!(f, "Sync"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

impl std::str::FromStr for MessageType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "command" => Ok(Self::Command),
            "response" => Ok(Self::Response),
            "event" => Ok(Self::Event),
            "error" => Ok(Self::Error),
            "setup" => Ok(Self::Setup),
            "heartbeat" => Ok(Self::Heartbeat),
            "sync" => Ok(Self::Sync),
            "unknown" => Ok(Self::Unknown),
            _ => Err(format!("Unknown message type: {s}")),
        }
    }
}

/// Message ID for MCP communications.
///
/// This is a wrapper around a String that uniquely identifies a message in the MCP system.
/// Using a dedicated type (rather than a plain String) provides type safety and makes
/// the API more expressive.
///
/// ... (doc comment) ...
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct MessageId(pub String);

impl MessageId {
    /// Create a new random message ID.
    #[must_use]
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    /// Check if the message ID is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Creates a new message ID with a custom prefix and a randomly generated UUID.
    ///
    /// ... (doc comment) ...
    #[must_use]
    pub fn with_prefix(prefix: &str) -> Self {
        Self(format!("{}-{}", prefix, uuid::Uuid::new_v4()))
    }
}

/// Protocol version information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ProtocolVersion {
    /// Major protocol version
    pub major: u16,
    /// Minor protocol version
    pub minor: u16,
}

impl ProtocolVersion {
    /// Create a new protocol version
    #[must_use]
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    /// Returns the version as a string (e.g., "1.0")
    #[must_use]
    pub fn version_string(&self) -> String {
        format!("{}.{}", self.major, self.minor)
    }
}

impl std::fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        Self { major: 1, minor: 0 }
    }
}

/// Represents the header part of an MCP message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    /// Unique identifier for the message
    pub id: MessageId,
    /// Type of the message
    pub message_type: MessageType,
    /// Timestamp when the message was created
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Protocol version used by the message
    pub version: ProtocolVersion,
    /// Security-related metadata
    pub security: SecurityMetadata,
    /// Optional generic metadata
    pub metadata: Option<serde_json::Value>,
}

/// Core message structure for MCP communications.
///
/// This structure represents a message in the Machine Context Protocol (MCP),
/// which is used for communication between components in the system.
///
/// ... (doc comment) ...
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPMessage {
    /// Unique identifier for the message
    pub id: MessageId,
    /// Type of the message (Command, Response, Event, Error)
    pub type_: MessageType,
    /// Message payload as JSON value
    pub payload: serde_json::Value,
    /// Optional metadata about the message
    pub metadata: Option<serde_json::Value>,
    /// Security-related metadata
    pub security: SecurityMetadata, // Needs definition from security::types
    /// Timestamp when the message was created
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Protocol version used by the message
    pub version: ProtocolVersion,
    /// Optional trace ID for distributed tracing
    pub trace_id: Option<String>,
}

impl MCPMessage {
    /// Creates a new `MCPMessage` with the specified message type and payload.
    ///
    /// ... (doc comment) ...
    #[must_use]
    pub fn new(type_: MessageType, payload: serde_json::Value) -> Self {
        Self {
            id: MessageId::new(),
            type_,
            payload,
            metadata: None,
            security: SecurityMetadata::default(),
            timestamp: chrono::Utc::now(),
            version: ProtocolVersion::default(),
            trace_id: None,
        }
    }

    /// Creates a new `MCPMessage` with all fields specified.
    ///
    /// ... (doc comment) ...
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "Full MCP message constructor mirrors protocol fields"
    )]
    pub const fn with_details(
        id: MessageId,
        type_: MessageType,
        payload: serde_json::Value,
        metadata: Option<serde_json::Value>,
        security: SecurityMetadata,
        timestamp: chrono::DateTime<chrono::Utc>,
        version: ProtocolVersion,
        trace_id: Option<String>,
    ) -> Self {
        Self {
            id,
            type_,
            payload,
            metadata,
            security,
            timestamp,
            version,
            trace_id,
        }
    }

    /// Extracts the command name from the message payload.
    #[must_use]
    pub fn command(&self) -> String {
        self.payload.get("command").map_or_else(
            || "unknown".to_string(),
            |cmd| cmd.as_str().unwrap_or("unknown").to_string(),
        )
    }
}

impl Default for MCPMessage {
    fn default() -> Self {
        Self {
            id: MessageId::new(),
            type_: MessageType::Command,
            payload: serde_json::Value::Null,
            metadata: None,
            security: SecurityMetadata::default(),
            timestamp: chrono::Utc::now(),
            version: ProtocolVersion::default(),
            trace_id: None,
        }
    }
}

#[cfg(feature = "websocket")]
impl TryFrom<Message> for MCPMessage {
    type Error = crate::error::MCPError;

    fn try_from(message: Message) -> std::result::Result<Self, Self::Error> {
        let json_str = match message {
            Message::Text(text) => text,
            Message::Binary(data) => String::from_utf8(data).map_err(|e| {
                crate::error::MCPError::Transport(
                    format!("Invalid UTF-8 in binary message: {e}").into(),
                )
            })?,
            Message::Ping(_) | Message::Pong(_) | Message::Close(_) => {
                return Err(crate::error::MCPError::Transport(
                    "Cannot convert control message to MCPMessage".into(),
                ));
            }
            Message::Frame(_) => {
                return Err(crate::error::MCPError::Transport(
                    "Cannot convert raw frame to MCPMessage".into(),
                ));
            }
        };

        let mcp_message: Self = serde_json::from_str(&json_str).map_err(|e| {
            crate::error::MCPError::Transport(format!("Failed to parse JSON message: {e}").into())
        })?;

        Ok(mcp_message)
    }
}

#[cfg(feature = "websocket")]
impl From<MCPMessage> for Message {
    fn from(mcp_message: MCPMessage) -> Self {
        let json_str = serde_json::to_string(&mcp_message)
            .unwrap_or_else(|_| r#"{"error": "Failed to serialize message"}"#.to_string());
        Self::Text(json_str)
    }
}

/// Represents a response to a command message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResponse {
    /// ID of the request this response corresponds to
    pub request_id: MessageId,
    /// Response status (e.g., "Success", "Failure")
    pub status: String,
    /// Optional response details
    pub details: Option<serde_json::Value>,
}

/// Trait for components that handle specific MCP messages.
#[expect(
    async_fn_in_trait,
    reason = "MCP message handlers are implemented as concrete types; not used as dyn MessageHandler"
)]
pub trait MessageHandler: Send + Sync {
    /// Handles an incoming MCP message and optionally returns a response.
    async fn handle_message(&self, message: &MCPMessage) -> Result<Option<MCPMessage>>;
}

/// Result type for protocol validation operations
pub type ValidationResult = Result<()>;
/// Result type for protocol routing operations
pub type RoutingResult = Result<()>;

#[cfg(test)]
mod tests {
    use super::*;
    // Import from crate::types if still there, adjust if moved

    use crate::types::CompressionFormat;

    #[test]
    fn test_protocol_version_default() {
        let version = ProtocolVersion::default();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 0);
        assert_eq!(version.version_string(), "1.0");
    }

    #[test]
    fn test_message_id_new() {
        let id = MessageId::new();
        assert!(!id.0.is_empty(), "Generated ID should not be empty");

        let id2 = MessageId::new();
        assert_ne!(id.0, id2.0, "Generated IDs should be unique");
    }

    #[test]
    fn test_message_creation() {
        let msg = MCPMessage::new(
            MessageType::Command,
            serde_json::json!({
                "action": "test",
                "value": 42
            }),
        );

        assert_eq!(msg.type_, MessageType::Command);
        assert_eq!(msg.payload["action"], "test");
        assert_eq!(msg.payload["value"], 42);
    }

    // Test moved from src/types.rs
    #[test]
    fn test_defaults() {
        assert_eq!(CompressionFormat::default(), CompressionFormat::None);
        // Add more default tests for protocol types here if needed
    }

    #[test]
    fn message_type_display_and_from_str() {
        for (mt, s) in [
            (MessageType::Command, "command"),
            (MessageType::Response, "RESPONSE"),
            (MessageType::Event, "Event"),
            (MessageType::Error, "error"),
            (MessageType::Setup, "setup"),
            (MessageType::Heartbeat, "heartbeat"),
            (MessageType::Sync, "sync"),
            (MessageType::Unknown, "unknown"),
        ] {
            assert_eq!(mt.to_string(), format!("{mt:?}").replace('"', ""));
            let parsed: MessageType = s.parse().expect("parse");
            assert_eq!(parsed, mt);
        }
        assert!("nope".parse::<MessageType>().is_err());
    }

    #[test]
    fn message_id_prefix_and_empty() {
        let empty = MessageId(String::new());
        assert!(empty.is_empty());
        let p = MessageId::with_prefix("pre");
        assert!(p.0.starts_with("pre-"));
        assert!(!p.is_empty());
    }

    #[test]
    fn mcp_message_command_extraction() {
        let m = MCPMessage::new(
            MessageType::Command,
            serde_json::json!({ "command": "ping" }),
        );
        assert_eq!(m.command(), "ping");

        let m2 = MCPMessage::new(MessageType::Command, serde_json::json!({}));
        assert_eq!(m2.command(), "unknown");

        let m3 = MCPMessage::new(MessageType::Command, serde_json::json!({ "command": 42 }));
        assert_eq!(m3.command(), "unknown");
    }

    #[test]
    fn mcp_message_with_details_round_trip() {
        let id = MessageId("fixed-id".into());
        let ts = chrono::DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
            .expect("should succeed")
            .with_timezone(&chrono::Utc);
        let msg = MCPMessage::with_details(
            id,
            MessageType::Response,
            serde_json::json!({ "a": 1 }),
            Some(serde_json::json!({ "meta": true })),
            SecurityMetadata {
                version: "1".into(),
                timestamp: std::time::UNIX_EPOCH,
            },
            ts,
            ProtocolVersion::new(2, 3),
            Some("trace".into()),
        );
        let json = serde_json::to_string(&msg).expect("ser");
        let back: MCPMessage = serde_json::from_str(&json).expect("de");
        assert_eq!(back.id.0, "fixed-id");
        assert_eq!(back.version.major, 2);
        assert_eq!(back.trace_id.as_deref(), Some("trace"));
    }

    #[test]
    fn header_serde_round_trip() {
        let h = Header {
            id: MessageId("h1".into()),
            message_type: MessageType::Event,
            timestamp: chrono::DateTime::parse_from_rfc3339("2021-06-15T12:00:00Z")
                .expect("should succeed")
                .with_timezone(&chrono::Utc),
            version: ProtocolVersion::new(1, 2),
            security: SecurityMetadata {
                version: "1.0".into(),
                timestamp: std::time::UNIX_EPOCH,
            },
            metadata: Some(serde_json::json!({ "k": "v" })),
        };
        let json = serde_json::to_string(&h).expect("should succeed");
        let back: Header = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(back.id.0, "h1");
        assert_eq!(back.message_type, MessageType::Event);
    }
}

// Other protocol-related types will go here.

/// Security metadata for MCP messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMetadata {
    /// Security metadata schema version
    pub version: String,
    /// Timestamp when the metadata was created
    pub timestamp: SystemTime,
}

impl Default for SecurityMetadata {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            timestamp: SystemTime::now(),
        }
    }
}
