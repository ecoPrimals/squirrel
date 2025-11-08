//! Error type definitions for the MCP system.
//!
//! This module provides a comprehensive error type hierarchy for the Machine Context Protocol (MCP)
//! system. It defines various error types for different categories of errors that can occur during
//! MCP operations, including context errors, protocol errors, security errors, connection errors,
//! and more.
//!
//! # Error Types
//!
//! The central error type is `MCPError`, which is a comprehensive enum that can represent any
//! error that may occur in the MCP system. Specialized error types like `ContextError`,
//! `ProtocolError`, `SecurityError`, and `ConnectionError` provide more detailed error information
//! for specific categories of errors.
//!
//! # Error Context
//!
//! The `ErrorContext` struct provides additional metadata about errors, including:
//! - Timestamp of when the error occurred
//! - Operation that was being performed
//! - Component where the error occurred
//! - Severity of the error
//! - Whether the error is recoverable
//! - Additional details about the error

use crate::protocol::types::MessageType;
// Security types handled by BearDog framework
// use crate::security::types::SecurityLevel;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Map;

/// Error severity levels for categorizing and prioritizing errors.
///
/// Severity levels help determine error handling strategy, logging priority,
/// and whether immediate attention or alerts are required.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Low severity - minimal impact, typically handled automatically
    Low,

    /// Medium severity - moderate impact, may require attention
    Medium,

    /// High severity - significant impact, requires attention
    High,

    /// Critical severity - severe impact, requires immediate attention
    Critical,
}

impl ErrorSeverity {
    /// Check if severity requires immediate attention
    pub fn requires_immediate_attention(&self) -> bool {
        matches!(self, ErrorSeverity::High | ErrorSeverity::Critical)
    }
    
    /// Check if severity should trigger alerts
    pub fn should_alert(&self) -> bool {
        matches!(self, ErrorSeverity::High | ErrorSeverity::Critical)
    }
}

// Add missing types that were moved to other projects
/// Security level placeholder for core MCP functionality
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum SecurityLevel {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

/// Wire format error placeholder for core MCP functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireFormatError {
    pub message: String,
}

impl std::fmt::Display for WireFormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wire format error: {}", self.message)
    }
}

impl std::error::Error for WireFormatError {}

// Import all specialized error types
use crate::error::alert::AlertError;
use crate::error::client::ClientError;
use crate::error::config::ConfigError;
use crate::error::connection::ConnectionError;
use crate::error::context_err::ContextError as ContextErr;
use crate::error::handler::HandlerError;
use crate::error::integration::IntegrationError;
use crate::error::plugin::PluginError;
use crate::error::port::PortErrorKind;
use crate::error::protocol_err::ProtocolError;
use crate::error::rbac::RBACError;
use crate::error::registry::RegistryError;
use crate::error::session::SessionError;
use crate::error::task::TaskError;
use crate::error::tool::ToolError;
use crate::error::transport::TransportError;

/// Main error type for MCP operations.
///
/// This enum represents all possible errors that can occur during MCP operations.
/// It categorizes errors into different types based on their source and nature,
/// providing detailed information about what went wrong.
///
/// # Examples
///
/// ```
/// use squirrel_mcp::error::{MCPError, Result};
///
/// fn handle_error() -> Result<()> {
///     // Operation that might fail
///     if false {
///         return Err(MCPError::General("Something went wrong".to_string()));
///     }
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone, thiserror::Error)]
pub enum MCPError {
    // === Core Layer Errors ===
    
    /// Error originating from the MCP transport layer
    #[error(transparent)]
    Transport(#[from] TransportError),

    /// Error originating from the MCP protocol
    #[error(transparent)]
    Protocol(#[from] ProtocolError),

    /// Error originating from connection management
    #[error(transparent)]
    Connection(#[from] ConnectionError),

    // === Application Layer Errors ===
    
    /// Error originating from context management
    #[error(transparent)]
    Context(#[from] ContextErr),

    /// Error originating from session management
    #[error(transparent)]
    Session(#[from] SessionError),

    /// Error originating from client operations
    #[error(transparent)]
    Client(#[from] ClientError),

    /// Plugin-related errors
    #[error(transparent)]
    Plugin(#[from] PluginError),

    /// Tool execution errors
    #[error(transparent)]
    Tool(#[from] ToolError),
    
    /// Service registry errors
    #[error(transparent)]
    Registry(#[from] RegistryError),
    
    /// Task management errors
    #[error(transparent)]
    Task(#[from] TaskError),
    
    /// Request handler errors
    #[error(transparent)]
    Handler(#[from] HandlerError),

    // === Infrastructure Layer Errors ===

    /// Configuration errors
    #[error(transparent)]
    Config(#[from] ConfigError),

    /// Integration errors
    #[error(transparent)]
    Integration(#[from] IntegrationError),
    
    /// Port allocation errors
    #[error(transparent)]
    Port(#[from] PortErrorKind),

    // === Security Layer Errors ===

    /// Role-based access control errors
    #[error(transparent)]
    RBAC(#[from] RBACError),

    // === Monitoring Layer Errors ===

    /// Alert system errors
    #[error(transparent)]
    Alert(#[from] AlertError),

    /// Resource exhausted error
    ResourceExhausted(String),

    /// Invalid argument error
    InvalidArgument(String),

    /// Not found error
    NotFound(String),

    /// Internal server error
    Internal(String),

    /// Authentication error
    Authentication(String),

    /// Authorization error
    Authorization(String),

    /// Rate limit exceeded
    RateLimit(String),

    /// Timeout error
    Timeout(String),

    /// Configuration error
    Configuration(String),

    /// Validation error
    Validation(String),

    /// Invalid state error
    InvalidState(String),

    /// Invalid operation error
    InvalidOperation(String),

    /// Network error
    Network(String),

    /// IO error
    Io(String),

    /// JSON parsing error
    Json(String),

    /// General error
    General(String),

    /// Generic error (alias for General)
    Generic(String),

    /// Message router error
    MessageRouter(String),

    /// Serialization error
    Serialization(String),

    /// Deserialization error
    Deserialization(String),

    /// Invalid message error
    InvalidMessage(String),

    /// State error
    State(String),

    /// Unsupported operation error
    UnsupportedOperation(String),

    /// Circuit breaker error
    CircuitBreaker(String),

    /// Security error
    Security(String),

    /// Resource error
    Resource(String),

    /// Lifecycle error
    Lifecycle(String),

    /// Wire format error
    WireFormat(String),

    /// Not initialized error
    NotInitialized(String),

    /// Already in progress error
    AlreadyInProgress(String),

    /// Monitoring error
    Monitoring(String),

    /// Not connected error
    NotConnected(String),

    /// Remote error
    Remote(String),

    /// Unexpected error
    Unexpected(String),

    /// Version mismatch error
    VersionMismatch(String),

    /// Unsupported error
    Unsupported(String),

    /// Not implemented error
    NotImplemented(String),

    /// Not authorized error
    NotAuthorized(String),

    /// Internal error
    InternalError(String),

    /// Sync error
    Sync(String),

    /// Already exists error
    AlreadyExists(String),

    /// Invalid request error
    InvalidRequest(String),

    /// Database error
    Database(String),

    /// Operation failed error
    OperationFailed(String),
}

/// Errors related to Authentication and Authorization
#[derive(Debug, Clone, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials provided.")]
    InvalidCredentials,
    #[error("Authentication token is invalid or expired.")]
    InvalidToken,
    #[error("User account is locked or inactive.")]
    AccountLocked,
    #[error("Permission denied for action on resource '{0}'.")]
    PermissionDenied(String), // Holds permission identifier
    #[error("Authorization context is missing or invalid.")]
    MissingContext,
    #[error("External authentication provider error: {0}")]
    ProviderError(String),
    #[error("An internal authentication error occurred: {0}")]
    InternalError(String),
}

impl MCPError {
    /// Creates an error from an MCP error message
    #[must_use]
    pub fn from_message(message: &str) -> Self {
        Self::Generic(message.to_string())
    }

    /// Determines if this error is recoverable
    ///
    /// Recoverable errors can be retried with a reasonable expectation of success.
    /// Examples include temporary network issues, authentication failures that can
    /// be resolved by obtaining new credentials, or timeout errors that might succeed
    /// on a subsequent attempt.
    ///
    /// # Returns
    ///
    /// * `true` - If the error is recoverable
    /// * `false` - If the error is not recoverable
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        match self {
            // Recoverable errors
            Self::Connection(ConnectionError::Timeout(_) | ConnectionError::Reset)
            | Self::UnsupportedOperation(_) => true,

            // Default case - all other errors are non-recoverable
            _ => false,
        }
    }

    /// Determines the severity level of this error
    ///
    /// Severity levels help prioritize error handling and reporting.
    /// Higher severity errors require more immediate attention or
    /// may have a greater impact on system functionality.
    ///
    /// # Returns
    ///
    /// The severity level of the error as an `ErrorSeverity` enum value
    #[must_use]
    pub const fn severity(&self) -> ErrorSeverity {
        match self {
            Self::Connection(
                ConnectionError::ConnectionFailed(_)
                | ConnectionError::Closed(_)
                | ConnectionError::Reset
                | ConnectionError::Refused
                | ConnectionError::Unreachable
                | ConnectionError::TooManyConnections
                | ConnectionError::LimitReached(_),
            ) => ErrorSeverity::High,

            Self::Protocol(ProtocolError::InvalidVersion(_)) => ErrorSeverity::High,
            Self::UnsupportedOperation(_) => ErrorSeverity::Medium,

            // All other errors are low severity
            _ => ErrorSeverity::Low,
        }
    }

    /// Returns a string error code for this error.
    ///
    /// The error code consists of a category prefix and a numeric code, e.g., "MCP-001".
    /// This can be used for error tracking and reporting.
    #[must_use]
    pub fn code_str(&self) -> &'static str {
        match self {
            Self::Transport(_) => "MCP-001",
            Self::Protocol(_) => "MCP-002",
            Self::Connection(_) => "MCP-004",
            Self::Session(_) => "MCP-005",
            Self::Context(_) => "MCP-006",
            Self::Client(_) => "MCP-007",
            Self::MessageRouter(_) => "MCP-008",
            Self::Serialization(_) => "MCP-009",
            Self::Deserialization(_) => "MCP-010",
            Self::InvalidMessage(_) => "MCP-011",
            Self::State(_) => "MCP-012",
            Self::Authorization(_) => "MCP-013",
            Self::UnsupportedOperation(_) => "MCP-014",
            Self::CircuitBreaker(_) => "MCP-015",
            Self::Io(_) => "MCP-016",
            Self::Json(_) => "MCP-017",
            Self::Task(_) => "MCP-018",
            Self::Handler(_) => "MCP-019",
            Self::Plugin(_) => "MCP-020",
            Self::Security(_) => "MCP-021",
            Self::Resource(_) => "MCP-022",
            Self::Validation(_) => "MCP-023",
            Self::Lifecycle(_) => "MCP-024",
            Self::Tool(_) => "MCP-025",
            Self::WireFormat(_) => "MCP-027",
            Self::NotInitialized(_) => "MCP-028",
            Self::General(_) => "MCP-029",
            Self::AlreadyInProgress(_) => "MCP-030",
            Self::Monitoring(_) => "MCP-031",
            Self::NotConnected(_) => "MCP-032",
            Self::Timeout(_) => "MCP-033",
            Self::Remote(_) => "MCP-034",
            Self::Unexpected(_) => "MCP-035",
            Self::VersionMismatch(_) => "MCP-036",
            Self::Unsupported(_) => "MCP-037",
            Self::InvalidArgument(_) => "MCP-038",
            Self::NotFound(_) => "MCP-039",
            Self::NotImplemented(_) => "MCP-040",
            Self::NotAuthorized(_) => "MCP-041",
            Self::InvalidState(_) => "MCP-042",
            Self::InvalidOperation(_) => "MCP-043",
            Self::InternalError(_) => "MCP-044",
            Self::Sync(_) => "MCP-045",
            Self::AlreadyExists(_) => "MCP-046",
            Self::InvalidRequest(_) => "MCP-047",
            Self::Database(_) => "MCP-048",
            Self::OperationFailed(_) => "MCP-049",
            Self::Generic(_) => "MCP-050",
            Self::Configuration(_) => "MCP-051",
            Self::Alert(_) => "MCP-052",
            Self::Network(_) => "MCP-053",
            Self::ResourceExhausted(_) => "MCP-056",
            Self::Internal(_) => "MCP-057",
            Self::Authentication(_) => "MCP-058",
            Self::RateLimit(_) => "MCP-059",
            Self::Registry(_) => "MCP-060",
            Self::Config(_) => "MCP-061",
            Self::Integration(_) => "MCP-062",
            Self::Port(_) => "MCP-063",
            Self::RBAC(_) => "MCP-064",
        }
    }

    /// For backwards compatibility with existing code
    #[must_use]
    pub fn error_code(&self) -> String {
        self.code_str().to_string()
    }

    /// Returns a string representation of the general error category.
    pub fn category_str(&self) -> &'static str {
        match self {
            MCPError::Transport(_) => "TRANSPORT",
            MCPError::Protocol(_) => "PROTOCOL",
            MCPError::Connection(_) => "CONNECTION",
            MCPError::Session(_) => "SESSION",
            MCPError::Context(_) => "CONTEXT",
            MCPError::Client(_) => "CLIENT",
            MCPError::MessageRouter(_) => "MESSAGE_ROUTER",
            MCPError::Serialization(_) => "SERIALIZATION",
            MCPError::Deserialization(_) => "DESERIALIZATION",
            MCPError::InvalidMessage(_) => "INVALID_MESSAGE",
            MCPError::State(_) => "STATE",
            MCPError::Authorization(_) => "AUTHORIZATION",
            MCPError::UnsupportedOperation(_) => "UNSUPPORTED_OPERATION",
            MCPError::CircuitBreaker(_) => "CIRCUIT_BREAKER",
            MCPError::Io(_) => "IO",
            MCPError::Json(_) => "JSON",
            MCPError::Task(_) => "TASK",
            MCPError::Handler(_) => "HANDLER",
            MCPError::Plugin(_) => "PLUGIN",
            MCPError::Security(_) => "SECURITY",
            MCPError::Resource(_) => "RESOURCE",
            MCPError::Validation(_) => "VALIDATION",
            MCPError::Lifecycle(_) => "LIFECYCLE",
            MCPError::Tool(_) => "TOOL",
            MCPError::WireFormat(_) => "WIRE_FORMAT",
            MCPError::NotInitialized(_) => "NOT_INITIALIZED",
            MCPError::General(_) => "GENERAL",
            MCPError::AlreadyInProgress(_) => "ALREADY_IN_PROGRESS",
            MCPError::Monitoring(_) => "MONITORING",
            MCPError::NotConnected(_) => "NOT_CONNECTED",
            MCPError::Timeout(_) => "TIMEOUT",
            MCPError::Remote(_) => "REMOTE",
            MCPError::Unexpected(_) => "UNEXPECTED",
            MCPError::VersionMismatch(_) => "VERSION_MISMATCH",
            MCPError::Unsupported(_) => "UNSUPPORTED",
            MCPError::NotImplemented(_) => "NOT_IMPLEMENTED",
            MCPError::NotAuthorized(_) => "NOT_AUTHORIZED",
            MCPError::InvalidState(_) => "INVALID_STATE",
            MCPError::InvalidOperation(_) => "INVALID_OPERATION",
            Self::InternalError(_) => "INTERNAL_ERROR",
            Self::Sync(_) => "SYNC",
            Self::AlreadyExists(_) => "ALREADY_EXISTS",
            Self::InvalidRequest(_) => "INVALID_REQUEST",
            Self::Database(_) => "DATABASE",
            Self::OperationFailed(_) => "OPERATION_FAILED",
            Self::Generic(_) => "GENERIC",
            Self::Configuration(_) => "CONFIGURATION",
            Self::Alert(_) => "ALERT",
            Self::Network(_) => "NETWORK",
            Self::InvalidArgument(_) => "INVALID_ARGUMENT",
            Self::NotFound(_) => "NOT_FOUND",
            Self::ResourceExhausted(_) => "RESOURCE_EXHAUSTED",
            Self::Internal(_) => "INTERNAL",
            Self::Authentication(_) => "AUTHENTICATION",
            Self::RateLimit(_) => "RATE_LIMIT",
            Self::Registry(_) => "REGISTRY",
            Self::Config(_) => "CONFIG",
            Self::Integration(_) => "INTEGRATION",
            Self::Port(_) => "PORT",
            Self::RBAC(_) => "RBAC",
        }
    }
}

// Add From implementations for various error types
impl From<std::io::Error> for MCPError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<serde_json::Error> for MCPError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err.to_string())
    }
}

/// Error context information for MCP errors
///
/// This struct provides detailed contextual information about errors that occur
/// within the MCP system, including when they occurred, what operation was being
/// performed, and additional metadata to assist with debugging and error handling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Timestamp indicating when the error occurred
    ///
    /// This helps with chronological tracking and correlation of errors
    /// with other system events.
    pub timestamp: DateTime<Utc>,

    /// Description of the operation being performed when the error occurred
    ///
    /// This provides context about what the system was trying to do
    /// when the error was encountered.
    pub operation: String,

    /// Name of the component where the error occurred
    ///
    /// This identifies which part of the system encountered the error,
    /// helping with troubleshooting and error localization.
    pub component: String,

    /// Optional: Type of message being processed when error occurred
    pub message_type: Option<MessageType>,

    /// Optional: Security level context at the time of the error
    pub security_level: Option<SecurityLevel>,

    /// Additional structured details about the error
    ///
    /// This can include any relevant contextual information that might
    /// help with diagnosing and resolving the issue.
    pub details: Map<String, serde_json::Value>,

    /// Severity level of the error
    ///
    /// This indicates how serious the error is, ranging from
    /// informational to critical.
    pub severity: ErrorSeverity,

    /// Indicates whether the error can be recovered from
    ///
    /// If true, the system may attempt to automatically recover
    /// from this error through retry mechanisms or fallbacks.
    pub is_recoverable: bool,

    /// Count of retry attempts made for recoverable errors
    ///
    /// This tracks how many times the system has attempted to
    /// recover from this error.
    pub retry_count: u32,

    /// Unique error code for identification and categorization
    ///
    /// This code can be used for error tracking, documentation,
    /// and reference purposes.
    pub error_code: String,

    /// Code location where the error occurred
    ///
    /// This optional field provides information about the specific
    /// location in the source code where the error originated.
    pub source_location: Option<String>,
}

impl ErrorContext {
    /// Creates a new error context with basic information
    ///
    /// Initializes a new error context with the specified operation and component,
    /// setting default values for all other fields.
    ///
    /// # Arguments
    ///
    /// * `operation` - Description of the operation being performed when the error occurred
    /// * `component` - Name of the component where the error occurred
    ///
    /// # Returns
    ///
    /// A new `ErrorContext` with default values
    pub fn new(operation: impl Into<String>, component: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            operation: operation.into(),
            component: component.into(),
            message_type: None,
            security_level: None,
            details: Map::new(),
            severity: ErrorSeverity::Low,
            is_recoverable: true,
            retry_count: 0,
            error_code: String::new(),
            source_location: None,
        }
    }

    /// Adds a message type to this error context
    ///
    /// # Arguments
    ///
    /// * `message_type` - The type of message being processed when the error occurred
    ///
    /// # Returns
    ///
    /// The updated `ErrorContext` for method chaining
    #[must_use]
    pub const fn with_message_type(mut self, message_type: MessageType) -> Self {
        self.message_type = Some(message_type);
        self
    }

    /// Sets the severity level for this error context
    ///
    /// # Arguments
    ///
    /// * `severity` - The severity level of the error
    ///
    /// # Returns
    ///
    /// The updated `ErrorContext` for method chaining
    #[must_use]
    pub const fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Adds detailed information to this error context
    ///
    /// # Arguments
    ///
    /// * `details` - A map of additional details about the error
    ///
    /// # Returns
    ///
    /// The updated `ErrorContext` for method chaining
    #[must_use]
    pub fn with_details(mut self, details: Map<String, serde_json::Value>) -> Self {
        self.details = details;
        self
    }

    /// Sets the error code for this context
    ///
    /// # Returns
    /// Returns self for method chaining
    #[must_use]
    pub fn with_error_code(mut self, code: impl Into<String>) -> Self {
        self.error_code = code.into();
        self
    }

    /// Sets the source location for this context
    ///
    /// # Returns
    /// Returns self for method chaining
    #[must_use]
    pub fn with_source_location(mut self, location: impl Into<String>) -> Self {
        self.source_location = Some(location.into());
        self
    }

    /// Increments the retry count for this error context
    ///
    /// This is called each time a recovery attempt is made for the error.
    pub fn increment_retry_count(&mut self) {
        self.retry_count += 1;
    }
}

// CoreError implementation removed - not needed for core functionality

// Add implementation to directly use TransportError in MCPError
// From<TransportError> now handled by #[from] attribute
// impl From<crate::error::transport::TransportError> for MCPError {
//     fn from(err: crate::error::transport::TransportError) -> Self {
//         Self::Transport(err)
//     }
// }

impl From<WireFormatError> for MCPError {
    fn from(err: WireFormatError) -> Self {
        MCPError::Protocol(ProtocolError::Wire(err.to_string()))
    }
}

// From<ProtocolError> now handled by #[from] attribute
// impl From<ProtocolError> for MCPError {
//     fn from(err: ProtocolError) -> Self {
//         MCPError::Protocol(err)
//     }
// }

// From<SessionError> now handled by #[from] attribute
// impl From<SessionError> for MCPError {
//     fn from(err: SessionError) -> Self {
//         Self::Session(err)
//     }
// }

// Remove duplicate imports that are causing errors
// pub use crate::error::{
//     transport::TransportError,
//     security_err::SecurityError,
//     protocol_err::ProtocolError,
//     session::SessionError,
// };

// Add missing From implementations for various error types
// From<ConnectionError> now handled by #[from] attribute
// impl From<ConnectionError> for MCPError {
//     fn from(err: ConnectionError) -> Self {
//         Self::Connection(err)
//     }
// }

// From<ContextErr> now handled by #[from] attribute
// impl From<ContextErr> for MCPError {
//     fn from(err: ContextErr) -> Self {
//         Self::Context(err)
//     }
// }

// Comment out implementation that uses ClientError since it's causing issues
// impl From<ClientError> for MCPError {
//     fn from(err: ClientError) -> Self {
//         Self::Client(err)
//     }
// }

// MessageRouter implementation removed - module doesn't exist

// From<PluginError> now handled by #[from] attribute
// impl From<crate::error::plugin::PluginError> for MCPError {
//     fn from(err: crate::error::plugin::PluginError) -> Self {
//         Self::Plugin(err)
//     }
// }

// Implement From<String> for MCPError to handle cases where String is converted to MCPError
impl From<String> for MCPError {
    fn from(msg: String) -> Self {
        MCPError::General(msg)
    }
}

// Implement From<&str> for MCPError for convenience
impl From<&str> for MCPError {
    fn from(msg: &str) -> Self {
        MCPError::General(msg.to_string())
    }
}

// TODO: Re-enable when enhanced module is available
// impl From<crate::enhanced::config_validation::ConfigValidationError> for MCPError {
//     fn from(error: crate::enhanced::config_validation::ConfigValidationError) -> Self {
//         MCPError::Validation(error.to_string())
//     }
// }

/// Error type alias for backward compatibility
///
/// This type alias is provided for backward compatibility with code
/// that refers to `crate::error::Error` instead of `MCPError`.
pub type Error = MCPError;

/// Result type alias for backward compatibility
///
/// This type alias is provided for backward compatibility with code
/// that refers to `crate::error::MCPResult` instead of `Result`.
pub type MCPResult<T> = std::result::Result<T, MCPError>;

// Comment out the From<MCPError> for DomainError implementation until DomainError is defined
/*
impl From<MCPError> for DomainError {
    fn from(err: MCPError) -> Self {
        match err {
            MCPError::Transport(e) => Self::MCP(format!("Transport error: {e}")),
            MCPError::Protocol(e) => Self::MCP(format!("Protocol error: {e}")),
            MCPError::Security(e) => Self::MCP(format!("Security error: {e}")),
            MCPError::Connection(e) => Self::MCP(format!("Connection error: {e}")),
            MCPError::Session(e) => Self::MCP(format!("Session error: {e}")),
            MCPError::Context(e) => Self::MCP(format!("Context error: {e}")),
            MCPError::Client(e) => Self::MCP(format!("Client error: {e}")),
            MCPError::MessageRouter(e) => Self::MCP(format!("Message router error: {e}")),
            MCPError::Plugin(e) => Self::MCP(format!("Plugin error: {e}")),
            MCPError::Serialization(e) => Self::MCP(format!("Serialization error: {e}")),
            MCPError::Deserialization(e) => Self::MCP(format!("Deserialization error: {e}")),
            MCPError::InvalidMessage(e) => Self::MCP(format!("Invalid message: {e}")),
            MCPError::State(e) => Self::MCP(format!("State error: {e}")),
            MCPError::Authorization(e) => Self::MCP(format!("Authorization error: {e}")),
            MCPError::UnsupportedOperation(e) => Self::MCP(format!("Unsupported operation: {e}")),
            MCPError::CircuitBreaker(e) => Self::MCP(format!("Circuit breaker error: {e}")),
            MCPError::Io(e) => Self::MCP(format!("IO error: {e}")),
            MCPError::Json(e) => Self::MCP(format!("JSON error: {e}")),
            MCPError::SquirrelDetail(e) => Self::MCP(format!("Squirrel core error: {e}")),
            MCPError::PersistenceDetail(e) => Self::MCP(format!("Persistence error: {e}")),
            MCPError::Alert(e) => Self::MCP(format!("Alert error: {e}")),
            MCPError::Storage(e) => Self::MCP(format!("Storage error: {e}")),
            MCPError::NotInitialized(e) => Self::MCP(format!("Not initialized: {e}")),
            MCPError::General(e) => Self::MCP(format!("General error: {e}")),
            MCPError::Network(e) => Self::MCP(format!("Network error: {e}")),
            MCPError::AlreadyInProgress(e) => Self::MCP(format!("Already in progress: {e}")),
            MCPError::Monitoring(e) => Self::MCP(format!("Monitoring error: {e}")),
            MCPError::NotConnected(e) => Self::MCP(format!("Not connected: {e}")),
            MCPError::Timeout(e) => Self::MCP(format!("Timeout: {e}")),
            MCPError::Remote(e) => Self::MCP(format!("Remote error: {e}")),
            MCPError::Configuration(e) => Self::MCP(format!("Configuration error: {e}")),
            MCPError::Unexpected(e) => Self::MCP(format!("Unexpected error: {e}")),
            MCPError::VersionMismatch(e) => Self::MCP(format!("Version mismatch: {e}")),
            MCPError::Unsupported(e) => Self::MCP(format!("Unsupported: {e}")),
            MCPError::InvalidArgument(e) => Self::MCP(format!("Invalid argument: {e}")),
            MCPError::NotFound(e) => Self::MCP(format!("Not found: {e}")),
            MCPError::NotImplemented(e) => Self::MCP(format!("Not implemented: {e}")),
            MCPError::NotAuthorized(e) => Self::MCP(format!("Not authorized: {e}")),
            MCPError::InvalidState(e) => Self::MCP(format!("Invalid state: {e}")),
            MCPError::InvalidOperation(e) => Self::MCP(format!("Invalid operation: {e}")),
            MCPError::InternalError(e) => Self::MCP(format!("Internal error: {e}")),
        }
    }
}
*/

// Add the Error trait implementation
/// Canonical Result type for MCP operations
///
/// This is the primary Result type used throughout the MCP system.
/// It provides a convenient alias for Result<T, MCPError>.
///
/// # Examples
///
/// ```ignore
/// use crate::error::{Result, MCPError};
///
/// fn do_something() -> Result<String> {
///     Ok("success".to_string())
/// }
/// ```
pub type Result<T> = std::result::Result<T, MCPError>;

// std::error::Error now handled by thiserror::Error derive
// impl std::error::Error for MCPError {
//     fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
//         match self {
//             Self::Transport(e) => Some(e),
//             Self::Protocol(e) => Some(e),
//             Self::Connection(e) => Some(e),
//             Self::Session(e) => Some(e),
//             Self::Context(e) => Some(e),
//             Self::Client(e) => Some(e),
//             Self::Alert(_e) => None,
//             _ => None,
//         }
//     }
// }

// Add the Display trait implementation for MCPError
impl std::fmt::Display for MCPError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transport(e) => write!(f, "Transport error: {e}"),
            Self::Protocol(e) => write!(f, "Protocol error: {e}"),
            Self::Connection(e) => write!(f, "Connection error: {e}"),
            Self::Session(e) => write!(f, "Session error: {e}"),
            Self::Context(e) => write!(f, "Context error: {e}"),
            Self::Client(e) => write!(f, "Client error: {e}"),
            Self::Alert(e) => write!(f, "Alert error: {e}"),
            _ => write!(f, "{}", self.category_str()),
        }
    }
}

// Tonic gRPC integration removed - using WebSocket + tarpc instead
