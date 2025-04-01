//! Common types used throughout the MCP system.
//!
//! This module contains the core data structures and enumerations that define the Machine Context Protocol (MCP).
//! It includes message types, security levels, protocol versioning, and other fundamental structures
//! that are used across the MCP system for communication between components.
//!
//! # Core Message Types
//!
//! The primary message types in this module include:
//! - `MCPMessage`: The core message structure for MCP communications
//! - `CommandRequestMessage`: For issuing commands to the system
//! - `CommandResponseMessage`: For returning results from commands
//! - `EventMessage`: For publishing one-way event notifications
//! - `HandshakeMessage`: For establishing and negotiating connections
//!
//! # Identity Types
//!
//! MCP uses several wrapper types to provide type safety for identifiers:
//! - `MessageId`: Uniquely identifies messages within the system
//! - `UserId`: Identifies users across the system
//! - `AccountId`: Identifies accounts within the system
//! - `SessionToken`: Represents an active user session
//! - `AuthToken`: Used for authentication purposes
//!
//! # Protocol Configuration
//!
//! The protocol configuration is managed through:
//! - `ProtocolVersion`: Represents the protocol version with major and minor components
//! - `ProtocolState`: Tracks the current state of the protocol (ready, error, etc.)
//!
//! # Security
//!
//! Security-related types include:
//! - `SecurityLevel`: Defines the required security level for operations
//! - `EncryptionFormat`: Specifies encryption algorithms for secure communications
//! - `CompressionFormat`: Defines compression methods for efficient data transfer
//! - `UserRole`: Defines access control roles for authorization
//!
//! # Examples
//!
//! Creating a basic MCP message:
//!
//! ```
//! use mcp::types::{MCPMessage, MessageId, MessageType};
//! use serde_json::json;
//!
//! let message = MCPMessage {
//!     id: MessageId("msg123".to_string()),
//!     message_type: MessageType::Command,
//!     payload: json!({
//!         "command": "execute",
//!         "parameters": {
//!             "tool": "file_reader",
//!             "args": ["path/to/file"]
//!         }
//!     }),
//! };
//! ```
//!
//! Processing a response:
//!
//! ```
//! use mcp::types::{MCPResponse, ResponseStatus};
//! use serde_json::json;
//!
//! // Assuming we received a response from the system
//! fn process_response(response: MCPResponse) {
//!     match response.status {
//!         ResponseStatus::Success => {
//!             // Process successful response
//!             println!("Operation succeeded!");
//!         },
//!         ResponseStatus::Error => {
//!             // Handle error
//!             println!("Error: {}", response.error_message.unwrap_or_default());
//!         },
//!         ResponseStatus::Pending => {
//!             // Operation is still in progress
//!             println!("Operation is pending, check back later");
//!         },
//!     }
//! }
//! ```

use serde::{Deserialize, Serialize};
use uuid;
use serde_json;
use crate::protocol::types::MessageId;

/// Compression format for efficient data transfer.
///
/// Defines the supported compression algorithms that can be used to reduce
/// the size of MCP messages before transmission. The choice of format
/// depends on the trade-off between compression ratio and CPU usage.
///
/// # Options
///
/// - `None`: No compression is applied
/// - `Gzip`: Standard gzip compression
/// - `Zstd`: Modern, fast compression algorithm
/// - `Lz4`: Very fast compression, lower compression ratio
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, Hash)]
pub enum CompressionFormat {
    /// No compression: Data is transmitted uncompressed
    #[default]
    None,
    /// Gzip compression: Standard compression format with good compatibility
    Gzip,
    /// Zstandard compression: Modern compression with excellent performance
    Zstd,
    /// LZ4 compression: Fast compression algorithm with good ratio
    Lz4,
}

/// Message metadata for MCP messages.
///
/// This structure provides additional context for messages transmitted
/// through the MCP system. It includes timing information and routing
/// details that can be used for debugging, auditing, and message delivery.
///
/// # Usage
///
/// Message metadata is attached to all MCP messages and can be used to:
/// - Track message flow through the system
/// - Measure performance and latency
/// - Debug message routing issues
/// - Implement message TTL (time-to-live) logic
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageMetadata {
    /// Timestamp of the message (Unix timestamp in milliseconds)
    pub timestamp: u64,
    /// Source of the message (sender identifier)
    pub source: String,
    /// Destination of the message (recipient identifier)
    pub destination: String,
}

/// Status of a response message.
///
/// This enumeration represents the possible states of a response
/// to a command or request. It indicates whether the operation
/// succeeded, failed, or is still in progress.
///
/// # Usage
///
/// Response status is used to determine how to handle a response:
/// - Success: The operation completed successfully
/// - Error: The operation failed
/// - Pending: The operation is still being processed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ResponseStatus {
    /// Success: The operation completed successfully
    #[default]
    Success,
    /// Error: The operation failed
    Error,
    /// Pending: The operation is still being processed
    Pending,
}

/// Command Request Message for executing commands via MCP.
///
/// This message type is used to request execution of a command with optional arguments.
///
/// # Examples
///
/// ```
/// use mcp::types::{CommandRequestMessage, MessageId};
///
/// let request = CommandRequestMessage {
///     id: MessageId("cmd123".to_string()),
///     command: "read_file".to_string(),
///     args: Some(serde_json::json!({"path": "example.txt"})),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRequestMessage {
    /// Unique identifier for the message
    pub id: MessageId,
    /// Command to execute
    pub command: String,
    /// Optional arguments for the command
    pub args: Option<serde_json::Value>,
}

/// Command Response Message for returning command execution results.
///
/// This message type is sent in response to a `CommandRequestMessage` and
/// includes the status of the command execution and optional result data.
///
/// # Examples
///
/// ```
/// use mcp::types::{CommandResponseMessage, MessageId};
///
/// let response = CommandResponseMessage {
///     id: MessageId("resp123".to_string()),
///     command_id: MessageId("cmd123".to_string()),
///     status: "success".to_string(),
///     result: Some(serde_json::json!({"content": "file contents..."})),
/// };
/// ```
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

/// Event Message for publishing events in the MCP system.
///
/// This message type is used for one-way event notifications that don't require responses.
///
/// # Examples
///
/// ```
/// use mcp::types::{EventMessage, MessageId};
///
/// let event = EventMessage {
///     id: MessageId("evt123".to_string()),
///     event_type: "file_change".to_string(),
///     data: serde_json::json!({
///         "path": "example.txt",
///         "operation": "write"
///     }),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMessage {
    /// Unique identifier for the event
    pub id: MessageId,
    /// Type of event
    pub event_type: String,
    /// Event data
    pub data: serde_json::Value,
}

/// Handshake Message for establishing MCP connections.
///
/// This message is sent during connection initialization to negotiate
/// protocol version and capabilities.
///
/// # Examples
///
/// ```
/// use mcp::types::{HandshakeMessage, MessageId, ProtocolVersion};
///
/// let handshake = HandshakeMessage {
///     id: MessageId("hs123".to_string()),
///     version: ProtocolVersion::new(1, 0),
///     capabilities: vec!["compression".to_string(), "encryption".to_string()],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeMessage {
    /// Unique identifier for the handshake
    pub id: MessageId,
    /// Protocol version
    pub version: ProtocolVersion,
    /// Supported capabilities
    pub capabilities: Vec<String>,
}

/// MCP Response structure for replying to requests.
///
/// This structure provides a standardized response format for MCP communications,
/// including status information, payload data, and optional error messages.
///
/// # Examples
///
/// ```
/// use mcp::types::{MCPResponse, ResponseStatus, MessageMetadata};
///
/// let response = MCPResponse {
///     protocol_version: "1.0".to_string(),
///     message_id: "req123".to_string(),
///     status: ResponseStatus::Success,
///     payload: serde_json::to_vec(&serde_json::json!({"result": "ok"})).unwrap(),
///     error_message: None,
///     metadata: MessageMetadata::default(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MCPResponse {
    /// Protocol version
    #[serde(default = "default_protocol_version")]
    pub protocol_version: String,
    /// Message ID this is responding to
    #[serde(default = "default_message_id")]
    pub message_id: MessageId,
    /// Status of the response
    #[serde(default)]
    pub status: ResponseStatus,
    /// Response payload
    #[serde(default)]
    pub payload: Vec<serde_json::Value>,
    /// Error message if status is Error
    #[serde(default)]
    pub error_message: Option<String>,
    /// Response metadata
    #[serde(default)]
    pub metadata: MessageMetadata,
}

/// Helper functions for serde default
fn default_protocol_version() -> String {
    "1.0".to_string()
}

fn default_message_id() -> MessageId {
    MessageId("none".to_string()) // Or MessageId::new() if appropriate
}

/// MCP Command structure for executing operations.
///
/// This structure represents a command to be executed within the MCP system,
/// with an identifier and binary parameters.
///
/// # Examples
///
/// ```
/// use mcp::types::MCPCommand;
///
/// let command = MCPCommand {
///     command: "execute_tool".to_string(),
///     parameters: serde_json::to_vec(&serde_json::json!({"tool": "file_reader"})).unwrap(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPCommand {
    /// Command identifier
    pub command: String,
    /// Command parameters
    pub parameters: Vec<u8>,
}

/// Protocol version for MCP communications.
///
/// This structure represents the version of the MCP protocol using
/// semantic versioning principles with major and minor components.
/// It's used for version negotiation during connection establishment
/// and ensures compatibility between communicating components.
///
/// # Semantic Versioning
///
/// The protocol follows semantic versioning:
/// - Major version changes indicate breaking changes
/// - Minor version changes indicate backward-compatible additions
///
/// Components can negotiate the highest mutually supported version.
///
/// # Examples
///
/// ```
/// use mcp::types::ProtocolVersion;
///
/// let v1_0 = ProtocolVersion::new(1, 0);
/// let v1_1 = ProtocolVersion::new(1, 1);
/// let v2_0 = ProtocolVersion::new(2, 0);
///
/// // Components can check compatibility
/// fn is_compatible(client: &ProtocolVersion, server: &ProtocolVersion) -> bool {
///     // Example compatibility rule: same major version, client minor <= server minor
///     client.major == server.major && client.minor <= server.minor
/// }
///
/// assert!(is_compatible(&v1_0, &v1_1)); // Compatible: client is v1.0, server is v1.1
/// assert!(!is_compatible(&v2_0, &v1_1)); // Incompatible: different major versions
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProtocolVersion {
    /// Major version number
    pub major: u32,
    /// Minor version number
    pub minor: u32,
}

impl ProtocolVersion {
    /// Creates a new protocol version
    #[must_use]
    pub const fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }

    /// Returns the version as a string
    #[must_use]
    pub fn version_string(&self) -> String {
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

/// Protocol state for MCP operations.
///
/// This enumeration defines the possible states of the MCP protocol,
/// from uninitialized to fully operational. It is used to track and
/// manage the lifecycle of protocol connections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolState {
    /// Protocol is uninitialized: Initial state before setup
    Uninitialized,
    /// Protocol is initializing: Setup in progress
    Initializing,
    /// Protocol is initialized: Basic setup complete
    Initialized,
    /// Protocol is ready: Fully operational
    Ready,
    /// Protocol is shutting down: Termination in progress
    ShuttingDown,
    /// Protocol is in an error state: Problem detected
    Error,
    /// Protocol is closed: Fully terminated
    Closed,
}

impl Default for ProtocolState {
    fn default() -> Self {
        Self::Uninitialized
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
    #[must_use]
    pub fn new() -> Self {
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
