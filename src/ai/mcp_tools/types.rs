use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::SystemTime;

/// Message compression format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionFormat {
    None,
    Gzip,
    Zstd,
    Lz4,
}

/// Message encryption format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncryptionFormat {
    None,
    ChaCha20Poly1305,
    Aes256Gcm,
}

/// Tool lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolState {
    /// Tool is registered but not initialized
    Registered,
    /// Tool is initializing
    Initializing,
    /// Tool is ready for use
    Ready,
    /// Tool is processing a request
    Processing,
    /// Tool is paused
    Paused,
    /// Tool is shutting down
    ShuttingDown,
    /// Tool has encountered an error
    Error,
    /// Tool has been unregistered
    Unregistered,
}

/// Tool lifecycle event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolEvent {
    /// Tool has been registered
    Registered {
        tool_id: String,
        timestamp: SystemTime,
    },
    /// Tool initialization started
    InitializationStarted {
        tool_id: String,
        timestamp: SystemTime,
    },
    /// Tool initialization completed
    InitializationCompleted {
        tool_id: String,
        timestamp: SystemTime,
    },
    /// Tool started processing
    ProcessingStarted {
        tool_id: String,
        request_id: String,
        timestamp: SystemTime,
    },
    /// Tool completed processing
    ProcessingCompleted {
        tool_id: String,
        request_id: String,
        timestamp: SystemTime,
    },
    /// Tool paused
    Paused {
        tool_id: String,
        timestamp: SystemTime,
    },
    /// Tool resumed
    Resumed {
        tool_id: String,
        timestamp: SystemTime,
    },
    /// Tool encountered an error
    Error {
        tool_id: String,
        error: String,
        timestamp: SystemTime,
    },
    /// Tool shutdown started
    ShutdownStarted {
        tool_id: String,
        timestamp: SystemTime,
    },
    /// Tool shutdown completed
    ShutdownCompleted {
        tool_id: String,
        timestamp: SystemTime,
    },
    /// Tool unregistered
    Unregistered {
        tool_id: String,
        timestamp: SystemTime,
    },
}

/// Tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    /// Tool identifier
    pub id: String,
    /// Tool name
    pub name: String,
    /// Tool version
    pub version: String,
    /// Tool description
    pub description: String,
    /// Tool capabilities
    pub capabilities: Vec<String>,
    /// Tool dependencies
    pub dependencies: Vec<String>,
    /// Maximum concurrent operations
    pub max_concurrent_operations: usize,
    /// Resource limits
    pub resource_limits: ResourceLimits,
    /// Security requirements
    pub security_requirements: SecurityRequirements,
}

/// Tool resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in bytes
    pub max_memory: u64,
    /// Maximum CPU usage percentage
    pub max_cpu_percent: u8,
    /// Maximum storage usage in bytes
    pub max_storage: u64,
    /// Maximum network bandwidth in bytes per second
    pub max_bandwidth: u64,
}

/// Tool security requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequirements {
    /// Required security level
    pub security_level: SecurityLevel,
    /// Required permissions
    pub required_permissions: Vec<String>,
    /// Required encryption
    pub encryption_required: bool,
    /// Required authentication
    pub authentication_required: bool,
}

/// Message metadata for compression and encryption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    pub compression: CompressionFormat,
    pub encryption: EncryptionFormat,
    pub version: String,
    pub timestamp: u64,
}

/// Represents a command in the Machine Context Protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPCommand {
    /// The name of the command
    pub name: String,
    /// The arguments for the command
    pub args: Vec<String>,
    /// Optional metadata for the command
    pub metadata: Option<MessageMetadata>,
}

/// Represents a response from a Machine Context Protocol command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPResponse {
    /// Whether the command was successful
    pub success: bool,
    /// The response data
    pub data: Option<serde_json::Value>,
    /// Any error message if the command failed
    pub error: Option<String>,
    /// Response metadata
    pub metadata: Option<MessageMetadata>,
}

/// Errors that can occur in the Machine Context Protocol
#[derive(Debug)]
pub enum MCPError {
    /// Command not found
    CommandNotFound(String),
    /// Invalid arguments
    InvalidArguments(String),
    /// Protocol error
    ProtocolError(String),
    /// Serialization error
    SerializationError(serde_json::Error),
    /// IO error
    IoError(std::io::Error),
    /// Security error
    SecurityError(String),
    /// Authentication error
    AuthenticationError(String),
    /// Authorization error
    AuthorizationError(String),
    /// Encryption error
    EncryptionError(String),
    /// Token error
    TokenError(String),
    /// Compression error
    CompressionError(String),
    /// Version error
    VersionError(String),
    /// Tool error
    ToolError(String),
    /// Resource limit exceeded
    ResourceLimitExceeded(String),
    /// Dependency error
    DependencyError(String),
}

impl fmt::Display for MCPError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MCPError::CommandNotFound(cmd) => write!(f, "Command not found: {}", cmd),
            MCPError::InvalidArguments(msg) => write!(f, "Invalid arguments: {}", msg),
            MCPError::ProtocolError(msg) => write!(f, "Protocol error: {}", msg),
            MCPError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            MCPError::IoError(e) => write!(f, "IO error: {}", e),
            MCPError::SecurityError(msg) => write!(f, "Security error: {}", msg),
            MCPError::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
            MCPError::AuthorizationError(msg) => write!(f, "Authorization error: {}", msg),
            MCPError::EncryptionError(msg) => write!(f, "Encryption error: {}", msg),
            MCPError::TokenError(msg) => write!(f, "Token error: {}", msg),
            MCPError::CompressionError(msg) => write!(f, "Compression error: {}", msg),
            MCPError::VersionError(msg) => write!(f, "Version error: {}", msg),
            MCPError::ToolError(msg) => write!(f, "Tool error: {}", msg),
            MCPError::ResourceLimitExceeded(msg) => write!(f, "Resource limit exceeded: {}", msg),
            MCPError::DependencyError(msg) => write!(f, "Dependency error: {}", msg),
        }
    }
}

impl std::error::Error for MCPError {}

impl From<serde_json::Error> for MCPError {
    fn from(err: serde_json::Error) -> Self {
        MCPError::SerializationError(err)
    }
}

impl From<std::io::Error> for MCPError {
    fn from(err: std::io::Error) -> Self {
        MCPError::IoError(err)
    }
} 