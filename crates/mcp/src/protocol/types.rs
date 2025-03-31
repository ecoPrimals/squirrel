//! Protocol-related types for MCP

use serde::{Deserialize, Serialize};
use uuid; // Needed for MessageId
use chrono; // Needed for MCPMessage
use serde_json; // Needed for MCPMessage

// Imports for types moved from other modules (will likely need adjustment later)
use crate::security::types::SecurityMetadata; 

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
            _ => Err(format!("Unknown message type: {}", s)),
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
    /// Creates a new message ID using a randomly generated UUID.
    ///
    /// ... (doc comment) ...
    #[must_use] pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
    
    /// Creates a new message ID with a custom prefix and a randomly generated UUID.
    ///
    /// ... (doc comment) ...
    #[must_use] pub fn with_prefix(prefix: &str) -> Self {
        Self(format!("{}-{}", prefix, uuid::Uuid::new_v4()))
    }
}

/// Protocol version information
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ProtocolVersion {
    /// Major protocol version
    pub major: u16,
    /// Minor protocol version
    pub minor: u16,
}

impl ProtocolVersion {
    /// Create a new protocol version
    pub fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }
    
    /// Returns the version as a string (e.g., "1.0")
    pub fn version_string(&self) -> String {
        format!("{}.{}", self.major, self.minor)
    }
}

impl std::fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// Represents the header part of an MCP message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub id: MessageId,
    pub message_type: MessageType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: ProtocolVersion,
    pub security: SecurityMetadata,
    // Add other fields often found in headers if needed
    // pub priority: crate::message::MessagePriority, // Example
    // pub correlation_id: Option<MessageId>, // Example
    // pub sequence_number: Option<u64>, // Example
    pub metadata: Option<serde_json::Value>, // Generic metadata
}

/// Core message structure for MCP communications.
///
/// This structure represents a message in the Machine Context Protocol (MCP),
/// which is used for communication between components in the system.
///
/// ... (doc comment) ...
#[derive(Debug, Clone, Serialize)]
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
    #[must_use] pub fn new(type_: MessageType, payload: serde_json::Value) -> Self {
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
    #[must_use] pub const fn with_details(
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

/// Represents a response to a command message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResponse {
    pub request_id: MessageId,
    pub status: String, // e.g., "Success", "Failure"
    pub details: Option<serde_json::Value>,
}

/// Trait for components that handle specific MCP messages.
#[async_trait::async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle_message(&self, message: &MCPMessage) -> Result<Option<MCPMessage>>;
}

// Need imports for trait
use async_trait::async_trait;
use crate::error::{MCPError, Result};

// Placeholder type aliases for results specific to protocol operations
pub type ValidationResult = Result<()>; // Placeholder for validation result
pub type RoutingResult = Result<()>; // Placeholder for routing result

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MessageMetadata; // Import from crate::types if still there, adjust if moved
    use uuid::Uuid;

    #[test]
    fn test_protocol_version_default() {
        let version = ProtocolVersion::default();
        assert_eq!(version.major, 0);
        assert_eq!(version.minor, 1);
        assert_eq!(version.patch, 0);
        assert_eq!(version.version_string(), "mcp/0.1.0");
    }

    #[test]
    fn test_message_id_new() {
        let id1 = MessageId::new();
        let id2 = MessageId::new();
        assert_ne!(id1.0, id2.0);
        // Basic check if it's a valid UUID format
        assert!(Uuid::parse_str(&id1.0).is_ok());
    }

    // Test moved from src/types.rs
    #[test]
    fn test_message_creation() {
        let message = MCPMessage {
            id: MessageId::new(),
            version: ProtocolVersion::default(),
            type_: MessageType::Request, // Assuming Request is still a valid variant
            payload: serde_json::to_vec(&serde_json::json!({ "data": "test" })).unwrap(), // Example payload as Vec<u8>
            metadata: MessageMetadata::default(),
            // Removed auth_token, session_token as they are not direct fields in MCPMessage
            // status and error_message are part of MCPResponse, not MCPMessage
        };
        assert_eq!(message.version.version_string(), "mcp/0.1.0");
        assert_eq!(message.type_, MessageType::Request);
        assert_eq!(message.payload, serde_json::to_vec(&serde_json::json!({ "data": "test" })).unwrap());
    }

    // Test moved from src/types.rs
    #[test]
    fn test_defaults() {
        assert_eq!(CompressionFormat::default(), CompressionFormat::None);
        // Add more default tests for protocol types here if needed
    }

    // Add other tests specific to protocol types here...
}

// Other protocol-related types will go here. 