//! Common types used throughout the MCP system

use serde::{Serialize, Deserialize};
use uuid;

/// Security level for MCP operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum SecurityLevel {
    /// Standard security level for basic operations
    #[default]
    Standard,
    /// High security level for sensitive operations
    High,
    /// Maximum security level for critical operations
    Maximum,
}

/// Encryption format for secure communications
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EncryptionFormat {
    /// No encryption
    #[default]
    None,
    /// AES-256-GCM encryption
    Aes256Gcm,
    /// ChaCha20-Poly1305 encryption
    ChaCha20Poly1305,
}

/// Message type for MCP communications
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageType {
    /// Command message
    Command,
    /// Response message
    Response,
    /// Event message
    Event,
    /// Error message
    Error,
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageType::Command => write!(f, "Command"),
            MessageType::Response => write!(f, "Response"),
            MessageType::Event => write!(f, "Event"),
            MessageType::Error => write!(f, "Error"),
        }
    }
}

/// Compression format for MCP communications
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CompressionFormat {
    /// No compression
    #[default]
    None,
    /// Gzip compression
    Gzip,
    /// Zstandard compression
    Zstd,
}

/// Message metadata for MCP messages
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageMetadata {
    /// Timestamp of the message
    pub timestamp: u64,
    /// Source of the message
    pub source: String,
    /// Destination of the message
    pub destination: String,
}

/// Status of a response message
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseStatus {
    /// Success
    Success,
    /// Error
    Error,
    /// Pending
    Pending,
}

/// Message ID for MCP communications
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(pub String);

/// MCP Message for communication between components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPMessage {
    /// Unique identifier for the message
    pub id: MessageId,
    /// Type of the message (Command, Response, Event, Error)
    pub message_type: MessageType,
    /// Message payload as JSON value
    pub payload: serde_json::Value,
}

impl Default for MCPMessage {
    fn default() -> Self {
        Self {
            id: MessageId("default".to_string()),
            message_type: MessageType::Command,
            payload: serde_json::json!({}),
        }
    }
}

/// Command Request Message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRequestMessage {
    /// Unique identifier for the message
    pub id: MessageId,
    /// Command to execute
    pub command: String,
    /// Optional arguments for the command
    pub args: Option<serde_json::Value>,
}

/// Command Response Message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResponseMessage {
    /// Unique identifier for the response
    pub id: MessageId,
    /// ID of the command request this is responding to
    pub command_id: MessageId,
    /// Status of the command execution
    pub status: String,
    /// Optional result data from the command
    pub result: Option<serde_json::Value>,
}

/// Event Message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMessage {
    /// Unique identifier for the event
    pub id: MessageId,
    /// Type of event
    pub event_type: String,
    /// Event data
    pub data: serde_json::Value,
}

/// Handshake Message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeMessage {
    /// Unique identifier for the handshake
    pub id: MessageId,
    /// Protocol version
    pub version: ProtocolVersion,
    /// Supported capabilities
    pub capabilities: Vec<String>,
}

/// MCP Response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPResponse {
    /// Protocol version
    pub protocol_version: String,
    /// Message ID this is responding to
    pub message_id: String,
    /// Status of the response
    pub status: ResponseStatus,
    /// Response payload
    pub payload: Vec<u8>,
    /// Error message if status is Error
    pub error_message: Option<String>,
    /// Response metadata
    pub metadata: MessageMetadata,
}

/// MCP Command structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPCommand {
    /// Command identifier
    pub command: String,
    /// Command parameters
    pub parameters: Vec<u8>,
}

/// Protocol version for MCP communications
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProtocolVersion {
    /// Major version number
    pub major: u32,
    /// Minor version number
    pub minor: u32,
}

impl ProtocolVersion {
    /// Creates a new protocol version
    #[must_use] pub fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }

    /// Returns the version as a string
    #[must_use] pub fn version_string(&self) -> String {
        format!("{}.{}", self.major, self.minor)
    }
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        Self { major: 1, minor: 0 }
    }
}

impl std::fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// Protocol state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProtocolState {
    /// Protocol is uninitialized
    Uninitialized,
    /// Protocol is initializing
    Initializing,
    /// Protocol is initialized and ready
    Initialized,
    /// Protocol is shutting down
    ShuttingDown,
    /// Protocol is in an error state
    Error,
}

impl Default for ProtocolState {
    fn default() -> Self {
        Self::Uninitialized
    }
}

/// User ID type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub String);

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl UserId {
    /// Create a new user ID
    #[must_use] pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for UserId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for UserId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Account ID type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccountId(pub String);

impl Default for AccountId {
    fn default() -> Self {
        Self::new()
    }
}

impl AccountId {
    /// Create a new account ID
    #[must_use] pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl std::fmt::Display for AccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for AccountId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for AccountId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Session token type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionToken(pub String);

impl Default for SessionToken {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionToken {
    /// Create a new session token
    #[must_use] pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl std::fmt::Display for SessionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for SessionToken {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SessionToken {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Authentication token type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AuthToken(pub String);

impl Default for AuthToken {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthToken {
    /// Create a new authentication token
    #[must_use] pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl std::fmt::Display for AuthToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for AuthToken {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for AuthToken {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// User role
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserRole {
    /// Administrator role
    Admin,
    /// User role
    User,
    /// Guest role
    Guest,
    /// Custom role
    Custom(String),
}

impl Default for UserRole {
    fn default() -> Self {
        Self::User
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Admin => write!(f, "Admin"),
            UserRole::User => write!(f, "User"),
            UserRole::Guest => write!(f, "Guest"),
            UserRole::Custom(role) => write!(f, "Custom({role})"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_level_ordering() {
        assert!(SecurityLevel::Standard < SecurityLevel::High);
        assert!(SecurityLevel::High < SecurityLevel::Maximum);
    }

    #[test]
    fn test_defaults() {
        assert_eq!(SecurityLevel::default(), SecurityLevel::Standard);
        assert_eq!(EncryptionFormat::default(), EncryptionFormat::None);
        assert_eq!(CompressionFormat::default(), CompressionFormat::None);
    }

    #[test]
    fn test_message_creation() {
        // Create a message ID
        let message_id = MessageId("test-123".to_string());
        
        // Create a message with the correct fields
        let message = MCPMessage {
            id: message_id,
            message_type: MessageType::Command,
            payload: serde_json::json!({"command": "test", "data": [1, 2, 3]}),
        };

        // Test the fields that actually exist
        assert_eq!(message.message_type, MessageType::Command);
        assert_eq!(message.id.0, "test-123");
        assert!(message.payload.is_object());
    }
} 