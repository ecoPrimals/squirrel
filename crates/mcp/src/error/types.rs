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

use crate::error::context::ErrorSeverity;
use crate::types::{MessageType, SecurityLevel};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Map;
use squirrel_core::error::{Result as CoreResult, SquirrelError as CoreError};
use thiserror::Error;
use uuid;

// Import the moved error types
use crate::error::connection::ConnectionError;
use crate::error::protocol_err::ProtocolError;
use crate::error::security_err::SecurityError;
use crate::error::session::SessionError;
use crate::error::context_err::ContextError;
use crate::error::alert::AlertError;
use crate::error::rbac::RBACError;
use crate::error::port::PortErrorKind; // Keep this? MCPError doesn't use it directly yet.
use crate::protocol::adapter_wire::WireFormatError;

/// Main error type for MCP operations.
///
/// This enum represents all possible errors that can occur during MCP operations.
/// It categorizes errors into different types based on their source and nature,
/// providing detailed information about what went wrong.
///
/// # Examples
///
/// ```
/// use mcp::error::{MCPError, Result};
///
/// fn handle_error() -> Result<()> {
///     // Operation that might fail
///     if something_went_wrong {
///         return Err(MCPError::General("Something went wrong".to_string()));
///     }
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone, Error)]
pub enum MCPError {
    /// Error originating from the MCP transport layer
    #[error("Transport error: {0}")]
    Transport(crate::error::transport::TransportError),
    
    /// Protocol errors
    #[error("Protocol error: {0}")]
    Protocol(ProtocolError),
    
    /// Security errors
    #[error("Security error: {0}")]
    Security(SecurityError),
    
    /// Connection errors
    #[error("Connection error: {0}")]
    Connection(ConnectionError),
    
    /// Session errors
    #[error("Session error: {0}")]
    Session(SessionError),
    
    /// Context errors
    #[error("Context error: {0}")]
    Context(ContextError),
    
    /// Client errors
    #[error("Client error: {0}")]
    Client(crate::error::client::ClientError),
    
    /// Message router errors
    #[error("Message router error: {0}")]
    MessageRouter(crate::message_router::MessageRouterError),
    
    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// Deserialization errors
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    
    /// Invalid message errors
    #[error("Invalid message: {0}")]
    InvalidMessage(String),
    
    /// State errors
    #[error("State error: {0}")]
    State(String),
    
    /// Authorization errors
    #[error("Authorization error: {0}")]
    Authorization(String),
    
    /// Unsupported operation errors
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
    
    /// Circuit breaker errors
    #[error("Circuit breaker error: {0}")]
    CircuitBreaker(String),
    
    /// IO errors - Use string representation since `std::io::Error` is not Clone
    #[error("IO error: {0}")]
    IoDetail(String),
    
    /// JSON errors (`serde_json`) - Use string representation since `serde_json::Error` is not Clone
    #[error("JSON error: {0}")]
    SerdeJsonDetail(String),
    
    /// Squirrel core errors - Use string representation since `SquirrelError` is not Clone
    #[error("Squirrel core error: {0}")]
    SquirrelDetail(String),
    
    /// Persistence errors - Use string representation since `PersistenceError` is not Clone
    #[error("Persistence error: {0}")]
    PersistenceDetail(String),
    
    /// Alert errors
    #[error("Alert error: {0}")]
    Alert(AlertError),
    
    /// Storage errors
    #[error("Storage error: {0}")]
    Storage(String),
    
    /// Not initialized errors
    #[error("Not initialized: {0}")]
    NotInitialized(String),
    
    /// General errors
    #[error("General error: {0}")]
    General(String),
    
    /// Network errors
    #[error("Network error: {0}")]
    Network(String),
    
    /// Already in progress errors
    #[error("Already in progress: {0}")]
    AlreadyInProgress(String),
    
    /// Monitoring system errors
    #[error("Monitoring error: {0}")]
    Monitoring(String),
    
    /// Not connected errors
    #[error("Not connected: {0}")]
    NotConnected(String),
    
    /// Timeout errors
    #[error("Timeout: {0}")]
    Timeout(String),
    
    /// Remote errors
    #[error("Remote error: {0}")]
    Remote(String),
    
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    /// Unexpected errors
    #[error("Unexpected error: {0}")]
    Unexpected(String),
    
    /// Version mismatch errors
    #[error("Version mismatch: {0}")]
    VersionMismatch(String),
    
    /// Unsupported errors
    #[error("Unsupported: {0}")]
    Unsupported(String),
    
    /// Invalid argument errors
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    /// Not found errors
    #[error("Not found: {0}")]
    NotFound(String),
    
    /// Not implemented errors
    #[error("Not implemented: {0}")]
    NotImplemented(String),
    
    /// Not authorized errors
    #[error("Not authorized: {0}")]
    NotAuthorized(String),
    
    /// Invalid state errors
    #[error("Invalid state: {0}")]
    InvalidState(String),
}

impl MCPError {
    /// Creates an error from an MCP error message
    #[must_use] pub fn from_message(message: &crate::message::Message) -> Self {
        // Extract error details from message metadata
        let error_type = message.metadata.get("error_type")
            .map_or("unknown", std::string::String::as_str);

        // Create appropriate error based on type
        match error_type {
            "transport" => Self::Transport(crate::error::transport::TransportError::ProtocolError(message.content.clone()).into()),
            "protocol" => Self::Protocol(ProtocolError::InvalidFormat(message.content.clone())),
            "security" => Self::Security(SecurityError::AuthenticationFailed(message.content.clone())),
            "connection" => Self::Connection(ConnectionError::Closed(message.content.clone())),
            "session" => Self::Session(SessionError::InvalidSession(message.content.clone())),
            "context" => Self::Context(ContextError::NotFound(uuid::Uuid::new_v4())),
            "client" => Self::Client(crate::error::client::ClientError::RemoteError(message.content.clone())),
            _ => Self::Remote(message.content.clone()),
        }
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
            Self::Security(
                SecurityError::AuthenticationFailed(_) | SecurityError::TokenExpired,
            )
            | Self::Connection(ConnectionError::Timeout(_) | ConnectionError::Reset) => true,

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

            // All other errors are low severity
            _ => ErrorSeverity::Low,
        }
    }

    /// Returns a string error code for this error.
    ///
    /// The error code consists of a category prefix and a numeric code, e.g., "MCP-001".
    /// This can be used for error tracking and reporting.
    #[must_use] pub fn error_code(&self) -> String {
        match self {
            Self::Transport(_) => "MCP-001",
            Self::Protocol(_) => "MCP-002",
            Self::Security(_) => "MCP-003",
            Self::Connection(_) => "MCP-004",
            Self::Session(_) => "MCP-005",
            Self::Context(_) => "MCP-006",
            Self::Serialization(_) => "MCP-007",
            Self::Deserialization(_) => "MCP-008",
            Self::InvalidMessage(_) => "MCP-009",
            Self::State(_) => "MCP-010",
            Self::Authorization(_) => "MCP-011",
            Self::UnsupportedOperation(_) => "MCP-012",
            Self::CircuitBreaker(_) => "MCP-013",
            Self::IoDetail(_) => "MCP-014",
            Self::SerdeJsonDetail(_) => "MCP-015",
            Self::SquirrelDetail(_) => "MCP-016",
            Self::PersistenceDetail(_) => "MCP-017",
            Self::Alert(_) => "MCP-018",
            Self::Storage(_) => "MCP-019",
            Self::NotInitialized(_) => "MCP-020",
            Self::General(_) => "MCP-021",
            Self::Network(_) => "MCP-022",
            Self::AlreadyInProgress(_) => "MCP-023",
            Self::Monitoring(_) => "MCP-024",
            Self::NotConnected(_) => "MCP-025",
            Self::Timeout(_) => "MCP-026",
            Self::Remote(_) => "MCP-027",
            Self::Configuration(_) => "MCP-028",
            Self::Unexpected(_) => "MCP-029",
            Self::VersionMismatch(_) => "MCP-030",
            Self::Unsupported(_) => "MCP-031",
            Self::InvalidArgument(_) => "MCP-032",
            Self::NotFound(_) => "MCP-033",
            Self::NotImplemented(_) => "MCP-034",
            Self::NotAuthorized(_) => "MCP-035",
            Self::InvalidState(_) => "MCP-036",
            Self::Client(_) => "MCP-037",
            Self::MessageRouter(_) => "MCP-038",
        }
        .to_string()
    }

    /// Returns a short string code representing the error type.
    pub fn code_str(&self) -> &'static str {
        match self {
            MCPError::Network(_) => "NETWORK_ERROR",
            MCPError::Protocol(_) => "PROTOCOL_ERROR",
            MCPError::Serialization(_) => "SERIALIZATION_ERROR",
            MCPError::Transport(_) => "TRANSPORT_ERROR",
            MCPError::Timeout(_) => "TIMEOUT_ERROR",
            MCPError::Configuration(_) => "CONFIG_ERROR",
            MCPError::Io(_) => "IO_ERROR",
            MCPError::State(_) => "STATE_ERROR",
            MCPError::Crypto(_) => "CRYPTO_ERROR",
            MCPError::Authentication(_) => "AUTH_ERROR",
            MCPError::Authorization(_) => "AUTHZ_ERROR",
            MCPError::NotFound(_) => "NOT_FOUND",
            MCPError::InvalidInput(_) => "INVALID_INPUT",
            MCPError::Internal(_) => "INTERNAL_ERROR",
            MCPError::Plugin(_) => "PLUGIN_ERROR",
            MCPError::Resource(_) => "RESOURCE_ERROR",
            MCPError::NotImplemented(_) => "NOT_IMPLEMENTED",
            MCPError::Remote(_) => "REMOTE_ERROR",
            MCPError::Connection(_) => "CONNECTION_ERROR",
            MCPError::Session(_) => "SESSION_ERROR",
            MCPError::Context(_) => "CONTEXT_ERROR",
            MCPError::Client(_) => "CLIENT_ERROR",
            MCPError::Security(_) => "SECURITY_ERROR",
            MCPError::RBAC(_) => "RBAC_ERROR",
            MCPError::Alert(_) => "ALERT_ERROR",
            MCPError::UnsupportedVersion(_) => "UNSUPPORTED_VERSION",
            MCPError::UnsupportedOperation(_) => "UNSUPPORTED_OPERATION",
            MCPError::Core(_) => "CORE_ERROR",
        }
    }
}

impl fmt::Display for MCPError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error_code())
    }
}

// Add From implementations for various error types
impl From<std::io::Error> for MCPError {
    fn from(err: std::io::Error) -> Self {
        Self::IoDetail(err.to_string())
    }
}

impl From<serde_json::Error> for MCPError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerdeJsonDetail(err.to_string())
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

    /// Type of message being processed when the error occurred, if applicable
    ///
    /// This field is optional and only relevant for errors that occur
    /// during message processing.
    pub message_type: Option<MessageType>,

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

// Add the implementation for From<MCPError> for SquirrelError
impl From<MCPError> for CoreError {
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
            MCPError::Serialization(e) => Self::MCP(format!("Serialization error: {e}")),
            MCPError::Deserialization(e) => Self::MCP(format!("Deserialization error: {e}")),
            MCPError::InvalidMessage(e) => Self::MCP(format!("Invalid message: {e}")),
            MCPError::State(e) => Self::MCP(format!("State error: {e}")),
            MCPError::Authorization(e) => Self::MCP(format!("Authorization error: {e}")),
            MCPError::UnsupportedOperation(e) => Self::MCP(format!("Unsupported operation: {e}")),
            MCPError::CircuitBreaker(e) => Self::MCP(format!("Circuit breaker error: {e}")),
            MCPError::IoDetail(e) => Self::MCP(format!("IO error: {e}")),
            MCPError::SerdeJsonDetail(e) => Self::MCP(format!("Serde JSON error: {e}")),
            MCPError::SquirrelDetail(e) => Self::MCP(format!("Squirrel error: {e}")),
            MCPError::PersistenceDetail(e) => Self::Persistence(squirrel_core::error::PersistenceError::Storage(e)),
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
        }
    }
}

// Add implementation to directly use TransportError in MCPError
impl From<crate::error::transport::TransportError> for MCPError {
    fn from(err: crate::error::transport::TransportError) -> Self {
        Self::Transport(err)
    }
}

impl From<WireFormatError> for MCPError {
    fn from(err: WireFormatError) -> Self {
        MCPError::Protocol(ProtocolError::Wire(err.to_string()))
    }
}

impl From<ProtocolError> for MCPError {
    fn from(err: ProtocolError) -> Self {
        MCPError::Protocol(err)
    }
}

impl From<SessionError> for MCPError {
    fn from(err: SessionError) -> Self {
        MCPError::Session(err)
    }
}
