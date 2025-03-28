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
use chrono;

/// Security level for MCP operations.
///
/// This enumeration defines the various security levels supported by the MCP system,
/// from low to critical. These levels are used to specify the required security
/// for different operations and resources, enabling fine-grained security control.
///
/// # Ordering
///
/// Security levels form a total ordering where:
/// Low < Standard < High < Critical
///
/// This allows for easy comparison of security requirements.
///
/// # Usage
///
/// ```
/// use mcp::types::SecurityLevel;
///
/// // Function that requires a minimum security level
/// fn secure_operation(level: SecurityLevel) -> bool {
///     if level >= SecurityLevel::High {
///         // Perform high-security operation
///         true
///     } else {
///         // Reject with insufficient security
///         false
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash, Default)]
pub enum SecurityLevel {
    /// Low security level: Minimal security requirements for non-sensitive operations
    Low = 0,
    /// Standard security level: Default for most operations
    #[default]
    Standard = 5,
    /// High security level: For sensitive operations requiring stronger security
    High = 10,
    /// Critical security level: Maximum security for the most sensitive operations
    Critical = 15,
}

/// Encryption format for secure communications.
///
/// This enumeration defines the supported encryption algorithms for
/// securing communications within the MCP system. The appropriate
/// format should be selected based on security requirements and
/// performance considerations.
///
/// # Security Considerations
///
/// - `None`: Provides no encryption and should only be used for non-sensitive data
/// - `Aes256Gcm`: Provides strong security with reasonable performance
/// - `ChaCha20Poly1305`: Alternative that may be faster on systems without AES hardware acceleration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, Hash)]
pub enum EncryptionFormat {
    /// No encryption: Data is transmitted in plaintext
    #[default]
    None,
    /// AES-256-GCM encryption: Industry standard authenticated encryption
    Aes256Gcm,
    /// ChaCha20-Poly1305 encryption: Modern stream cipher with authentication
    ChaCha20Poly1305,
}

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
        }
    }
}

/// Compression format for MCP communications.
///
/// This enumeration defines the supported compression algorithms for
/// reducing the size of data transmitted within the MCP system.
/// Compression can improve performance by reducing bandwidth usage
/// and transmission time.
///
/// # Performance Considerations
///
/// - `None`: No compression, fastest but uses the most bandwidth
/// - `Gzip`: Widely supported, good balance of compression ratio and speed
/// - `Zstd`: Modern format with better compression/speed tradeoff than Gzip
/// - `Lz4`: Fast compression algorithm with good ratio
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseStatus {
    /// Success: The operation completed successfully
    Success,
    /// Error: The operation failed
    Error,
    /// Pending: The operation is still being processed
    Pending,
}

/// Message ID for MCP communications.
///
/// This is a wrapper around a String that uniquely identifies a message in the MCP system.
/// Using a dedicated type (rather than a plain String) provides type safety and makes
/// the API more expressive.
///
/// # Recommended ID Formats
///
/// While any string can be used, it's recommended to use one of these approaches:
/// - UUIDs: Ensures global uniqueness (e.g., "550e8400-e29b-41d4-a716-446655440000")
/// - Structured IDs: Combines source and timestamp (e.g., "mcp-client-1234567890")
/// - Sequential IDs with prefix: (e.g., "msg-123456")
///
/// # Examples
///
/// ```
/// use mcp::types::MessageId;
/// use uuid::Uuid;
///
/// // Using a UUID
/// let id1 = MessageId(Uuid::new_v4().to_string());
///
/// // Using a structured ID
/// let id2 = MessageId(format!("client-{}-{}", "user123", chrono::Utc::now().timestamp()));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(pub String);

impl MessageId {
    /// Creates a new message ID using a randomly generated UUID.
    ///
    /// This is a convenience method for creating a new message ID
    /// with a randomly generated UUID, which ensures global uniqueness
    /// without requiring coordination between components.
    ///
    /// # Returns
    ///
    /// A new `MessageId` with a random UUID as its value
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp::types::MessageId;
    ///
    /// let id = MessageId::new();
    /// println!("New message ID: {}", id.0);
    /// ```
    #[must_use] pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
    
    /// Creates a new message ID with a custom prefix and a randomly generated UUID.
    ///
    /// This method creates a message ID that combines a custom prefix with a UUID,
    /// which can be useful for identifying the source or type of message.
    ///
    /// # Arguments
    ///
    /// * `prefix` - A prefix to add to the UUID
    ///
    /// # Returns
    ///
    /// A new `MessageId` with the format "{prefix}-{uuid}"
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp::types::MessageId;
    ///
    /// let id = MessageId::with_prefix("cmd");
    /// // Would result in something like: "cmd-550e8400-e29b-41d4-a716-446655440000"
    /// ```
    #[must_use] pub fn with_prefix(prefix: &str) -> Self {
        Self(format!("{}-{}", prefix, uuid::Uuid::new_v4()))
    }
}

/// Core message structure for MCP communications.
///
/// This structure represents a message in the Machine Context Protocol (MCP),
/// which is used for communication between components in the system.
///
/// # Fields
///
/// - `id`: Unique identifier for the message
/// - `type_`: Type of the message (Command, Response, Event, Error, etc.)
/// - `payload`: Message payload as JSON value
/// - `metadata`: Optional metadata about the message
/// - `security`: Security-related metadata
/// - `timestamp`: Time when the message was created
/// - `version`: Protocol version used by the message
/// - `trace_id`: Optional trace ID for distributed tracing
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
    pub security: SecurityMetadata,
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
    /// This constructor sets default values for all other fields:
    /// - Generates a new unique message ID
    /// - Sets default security metadata
    /// - Sets the current timestamp
    /// - Uses the default protocol version
    /// - No trace ID (set to None)
    ///
    /// # Arguments
    ///
    /// * `type_` - The message type
    /// * `payload` - The message payload as a JSON value
    ///
    /// # Returns
    ///
    /// A new `MCPMessage` instance with the specified type and payload
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp::types::{MCPMessage, MessageType};
    /// use serde_json::json;
    ///
    /// let message = MCPMessage::new(
    ///     MessageType::Command,
    ///     json!({"command": "status"})
    /// );
    /// ```
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
    /// This constructor allows full control over all message fields,
    /// which is useful for creating messages with specific requirements.
    ///
    /// # Arguments
    ///
    /// * `id` - The message ID
    /// * `type_` - The message type
    /// * `payload` - The message payload
    /// * `metadata` - Optional message metadata
    /// * `security` - Security metadata
    /// * `timestamp` - Message creation timestamp
    /// * `version` - Protocol version
    /// * `trace_id` - Optional trace ID for distributed tracing
    ///
    /// # Returns
    ///
    /// A new `MCPMessage` instance with all fields specified
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp::types::{MCPMessage, MessageId, MessageType, SecurityMetadata, ProtocolVersion};
    /// use serde_json::json;
    /// use chrono::Utc;
    ///
    /// let message = MCPMessage::with_details(
    ///     MessageId("custom-id".to_string()),
    ///     MessageType::Command,
    ///     json!({"command": "status"}),
    ///     Some(json!({"source": "cli"})),
    ///     SecurityMetadata::default(),
    ///     Utc::now(),
    ///     ProtocolVersion::new(1, 0),
    ///     Some("trace-123".to_string())
    /// );
    /// ```
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

/// Security metadata for messages
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecurityMetadata {
    /// Security level for the message
    pub security_level: SecurityLevel,
    /// Optional encryption information
    pub encryption_info: Option<EncryptionInfo>,
    /// Optional digital signature
    pub signature: Option<String>,
    /// Optional authentication token
    pub auth_token: Option<String>,
    /// Optional permissions
    pub permissions: Option<Vec<String>>,
    /// Optional roles
    pub roles: Option<Vec<String>>,
}

/// Encryption information for secure communications
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EncryptionInfo {
    /// Encryption format used
    pub format: EncryptionFormat,
    /// Optional encryption key ID
    pub key_id: Option<String>,
    /// Optional initialization vector
    pub iv: Option<Vec<u8>>,
    /// Optional additional authenticated data
    pub aad: Option<Vec<u8>>,
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
    #[must_use]
    pub fn new() -> Self {
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
    #[must_use]
    pub fn new() -> Self {
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
    #[must_use]
    pub fn new() -> Self {
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

/// User role within the MCP system.
///
/// This enumeration defines the possible roles that users can have in the system,
/// which determines their permissions and access levels. Roles form the foundation
/// of the role-based access control (RBAC) system used by MCP.
///
/// # Access Control
///
/// Each role typically has different permission levels:
/// - Admin: Full system access
/// - User: Standard permissions for normal operations
/// - Guest: Limited, read-only access
/// - Custom: Specialized roles with specific permissions
///
/// # Examples
///
/// ```
/// use mcp::types::UserRole;
///
/// fn check_permission(role: &UserRole, operation: &str) -> bool {
///     match role {
///         UserRole::Admin => true,  // Admins can do anything
///         UserRole::User => {
///             // Users can perform standard operations
///             matches!(operation, "read" | "write" | "update")
///         },
///         UserRole::Guest => {
///             // Guests can only read
///             operation == "read"
///         },
///         UserRole::Custom(role_name) => {
///             // Custom logic for specific roles
///             matches!(role_name.as_str(), "moderator" | "analyst")
///         },
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserRole {
    /// Administrator role: Full system access
    Admin,
    /// User role: Standard access for normal operations
    User,
    /// Guest role: Limited access, typically read-only
    Guest,
    /// Custom role: Role with specific permissions
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
            Self::Admin => write!(f, "Admin"),
            Self::User => write!(f, "User"),
            Self::Guest => write!(f, "Guest"),
            Self::Custom(role) => write!(f, "Custom({role})"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_level_ordering() {
        assert!(SecurityLevel::Standard < SecurityLevel::High);
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
            type_: MessageType::Command,
            payload: serde_json::json!({"command": "test", "data": [1, 2, 3]}),
            metadata: None,
            security: SecurityMetadata::default(),
            timestamp: chrono::Utc::now(),
            version: ProtocolVersion::default(),
            trace_id: None,
        };

        // Test the fields that actually exist
        assert_eq!(message.type_, MessageType::Command);
        assert_eq!(message.id.0, "test-123");
        assert!(message.payload.is_object());
    }
}
