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
use crate::protocol::types::MessageType;
use crate::security::types::SecurityLevel;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Map;
use squirrel_core::error::SquirrelError as CoreError;
use uuid;

// Import the moved error types
use crate::error::connection::ConnectionError;
use crate::error::protocol_err::ProtocolError;
use crate::error::security_err::SecurityError;
use crate::error::session::SessionError;
use crate::error::context_err::ContextError;
use crate::error::alert::AlertError;
 // Keep this? MCPError doesn't use it directly yet.
use crate::protocol::adapter_wire::WireFormatError;

// Add import for ClientError

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
#[derive(Debug, Clone)]
pub enum MCPError {
    /// Error originating from the MCP transport layer
    Transport(crate::error::transport::TransportError),
    
    /// Protocol errors
    Protocol(ProtocolError),
    
    /// Security errors
    Security(SecurityError),
    
    /// Connection errors
    Connection(ConnectionError),
    
    /// Session errors
    Session(SessionError),
    
    /// Context errors
    Context(ContextError),
    
    /// Client errors
    Client(crate::error::client::ClientError),
    
    /// Message router errors
    MessageRouter(crate::message_router::MessageRouterError),
    
    /// Plugin system errors
    Plugin(crate::error::plugin::PluginError),
    
    /// Serialization errors
    Serialization(String),
    
    /// Deserialization errors
    Deserialization(String),
    
    /// Invalid message errors
    InvalidMessage(String),
    
    /// State errors
    State(String),
    
    /// Authorization errors
    Authorization(String),
    
    /// Unsupported operation errors
    UnsupportedOperation(String),
    
    /// Circuit breaker errors
    CircuitBreaker(String),
    
    /// IO errors - Use string representation since `std::io::Error` is not Clone
    IoDetail(String),
    
    /// JSON errors (`serde_json`) - Use string representation since `serde_json::Error` is not Clone
    SerdeJsonDetail(String),
    
    /// Squirrel core errors - Use string representation since `SquirrelError` is not Clone
    SquirrelDetail(String),
    
    /// Persistence errors - Use string representation since `PersistenceError` is not Clone
    PersistenceDetail(String),
    
    /// Alert errors
    Alert(AlertError),
    
    /// Storage errors
    Storage(String),
    
    /// Not initialized errors
    NotInitialized(String),
    
    /// General errors
    General(String),
    
    /// Network errors
    Network(String),
    
    /// Already in progress errors
    AlreadyInProgress(String),
    
    /// Monitoring system errors
    Monitoring(String),
    
    /// Not connected errors
    NotConnected(String),
    
    /// Timeout errors
    Timeout(String),
    
    /// Remote errors
    Remote(String),
    
    /// Configuration errors
    Configuration(String),
    
    /// Unexpected errors
    Unexpected(String),
    
    /// Version mismatch errors
    VersionMismatch(String),
    
    /// Unsupported errors
    Unsupported(String),
    
    /// Invalid argument errors
    InvalidArgument(String),
    
    /// Not found errors
    NotFound(String),
    
    /// Not implemented errors
    NotImplemented(String),
    
    /// Not authorized errors
    NotAuthorized(String),
    
    /// Invalid state errors
    InvalidState(String),
    
    /// Invalid operation errors
    InvalidOperation(String),
    
    /// Internal system error
    InternalError(String),
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
            | Self::Connection(ConnectionError::Timeout(_) | ConnectionError::Reset)
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
    #[must_use] pub fn code_str(&self) -> &'static str {
        match self {
            Self::Transport(_) => "MCP-001",
            Self::Protocol(_) => "MCP-002",
            Self::Security(_) => "MCP-003",
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
            Self::IoDetail(_) => "MCP-016",
            Self::SerdeJsonDetail(_) => "MCP-017",
            Self::SquirrelDetail(_) => "MCP-018",
            Self::PersistenceDetail(_) => "MCP-019",
            Self::Alert(_) => "MCP-020",
            Self::Storage(_) => "MCP-021",
            Self::NotInitialized(_) => "MCP-022",
            Self::General(_) => "MCP-023",
            Self::Network(_) => "MCP-024",
            Self::AlreadyInProgress(_) => "MCP-025",
            Self::Monitoring(_) => "MCP-026",
            Self::NotConnected(_) => "MCP-027",
            Self::Timeout(_) => "MCP-028",
            Self::Remote(_) => "MCP-029",
            Self::Configuration(_) => "MCP-030",
            Self::Unexpected(_) => "MCP-031",
            Self::VersionMismatch(_) => "MCP-033",
            Self::Unsupported(_) => "MCP-034",
            Self::InvalidArgument(_) => "MCP-035",
            Self::NotFound(_) => "MCP-036",
            Self::NotImplemented(_) => "MCP-037",
            Self::NotAuthorized(_) => "MCP-038",
            Self::InvalidState(_) => "MCP-039",
            Self::InvalidOperation(_) => "MCP-040",
            Self::InternalError(_) => "MCP-041",
            Self::Plugin(_) => "MCP-032",
        }
    }

    /// For backwards compatibility with existing code
    #[must_use] pub fn error_code(&self) -> String {
        self.code_str().to_string()
    }

    /// Returns a string representation of the general error category.
    pub fn category_str(&self) -> &'static str {
        match self {
            MCPError::Transport(_) => "TRANSPORT",
            MCPError::Protocol(_) => "PROTOCOL",
            MCPError::Security(_) => "SECURITY",
            MCPError::Network(_) => "NETWORK",
            MCPError::Serialization(_) => "SERIALIZATION",
            MCPError::Authorization(_) => "AUTHORIZATION",
            MCPError::Configuration(_) => "CONFIGURATION",
            MCPError::InvalidArgument(_) => "INVALID_ARGUMENT",
            MCPError::NotFound(_) => "NOT_FOUND",
            MCPError::Connection(_) => "CONNECTION",
            MCPError::Session(_) => "SESSION",
            MCPError::Context(_) => "CONTEXT",
            MCPError::Client(_) => "CLIENT",
            MCPError::MessageRouter(_) => "MESSAGE_ROUTER",
            MCPError::Deserialization(_) => "DESERIALIZATION",
            MCPError::InvalidMessage(_) => "INVALID_MESSAGE",
            MCPError::State(_) => "STATE",
            MCPError::UnsupportedOperation(_) => "UNSUPPORTED_OPERATION",
            MCPError::CircuitBreaker(_) => "CIRCUIT_BREAKER",
            MCPError::IoDetail(_) => "IO",
            MCPError::SerdeJsonDetail(_) => "SERDE_JSON",
            MCPError::SquirrelDetail(_) => "SQUIRREL",
            MCPError::PersistenceDetail(_) => "PERSISTENCE",
            MCPError::Alert(_) => "ALERT",
            MCPError::Storage(_) => "STORAGE",
            MCPError::NotInitialized(_) => "NOT_INITIALIZED",
            MCPError::General(_) => "GENERAL",
            MCPError::AlreadyInProgress(_) => "ALREADY_IN_PROGRESS",
            MCPError::Monitoring(_) => "MONITORING",
            MCPError::NotConnected(_) => "NOT_CONNECTED",
            MCPError::Timeout(_) => "TIMEOUT",
            MCPError::Remote(_) => "REMOTE",
            MCPError::Unexpected(_) => "UNEXPECTED",
            MCPError::Plugin(_) => "PLUGIN",
            MCPError::VersionMismatch(_) => "VERSION_MISMATCH",
            MCPError::Unsupported(_) => "UNSUPPORTED",
            MCPError::NotImplemented(_) => "NOT_IMPLEMENTED",
            MCPError::NotAuthorized(_) => "NOT_AUTHORIZED",
            MCPError::InvalidState(_) => "INVALID_STATE",
            MCPError::InvalidOperation(_) => "INVALID_OPERATION",
            Self::InternalError(_) => "INTERNAL_ERROR",
        }
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
            MCPError::Serialization(e) => Self::MCP(format!("Serialization error: {e}")),
            MCPError::Deserialization(e) => Self::MCP(format!("Deserialization error: {e}")),
            MCPError::InvalidMessage(e) => Self::MCP(format!("Invalid message: {e}")),
            MCPError::State(e) => Self::MCP(format!("State error: {e}")),
            MCPError::Authorization(e) => Self::MCP(format!("Authorization error: {e}")),
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
            MCPError::Client(e) => Self::MCP(format!("Client error: {e}")),
            MCPError::MessageRouter(e) => Self::MCP(format!("Message router error: {e}")),
            MCPError::Plugin(e) => Self::MCP(format!("Plugin error: {e}")),
            MCPError::InvalidOperation(e) => Self::MCP(format!("Invalid operation: {e}")),
            MCPError::InternalError(e) => Self::MCP(format!("Internal error: {e}")),
            MCPError::UnsupportedOperation(e) => Self::MCP(format!("Unsupported operation: {e}")),
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
        Self::Session(err)
    }
}

// Remove duplicate imports that are causing errors
// pub use crate::error::{
//     transport::TransportError, 
//     security_err::SecurityError,
//     protocol_err::ProtocolError,
//     session::SessionError,
// };

// Add missing From implementations for various error types
impl From<SecurityError> for MCPError {
    fn from(err: SecurityError) -> Self {
        Self::Security(err)
    }
}

impl From<ContextError> for MCPError {
    fn from(err: ContextError) -> Self {
        Self::Context(err)
    }
}

impl From<ConnectionError> for MCPError {
    fn from(err: ConnectionError) -> Self {
        Self::Connection(err)
    }
}

// Comment out implementation that uses ClientError since it's causing issues
// impl From<ClientError> for MCPError {
//     fn from(err: ClientError) -> Self {
//         Self::Client(err)
//     }
// }

impl From<crate::message_router::MessageRouterError> for MCPError {
    fn from(err: crate::message_router::MessageRouterError) -> Self {
        Self::MessageRouter(err)
    }
}

impl From<crate::error::plugin::PluginError> for MCPError {
    fn from(err: crate::error::plugin::PluginError) -> Self {
        Self::Plugin(err)
    }
}

pub type Result<T, E = MCPError> = std::result::Result<T, E>;

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
            MCPError::IoDetail(e) => Self::MCP(format!("IO error: {e}")),
            MCPError::SerdeJsonDetail(e) => Self::MCP(format!("JSON error: {e}")),
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
impl std::error::Error for MCPError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Transport(e) => Some(e),
            Self::Protocol(e) => Some(e),
            Self::Security(e) => Some(e),
            Self::Connection(e) => Some(e),
            Self::Session(e) => Some(e),
            Self::Client(e) => Some(e),
            Self::Alert(e) => Some(e),
            _ => None,
        }
    }
}

// Add the Display trait implementation for MCPError
impl std::fmt::Display for MCPError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transport(e) => write!(f, "Transport error: {}", e),
            Self::Protocol(e) => write!(f, "Protocol error: {}", e),
            Self::Security(e) => write!(f, "Security error: {}", e),
            Self::Connection(e) => write!(f, "Connection error: {}", e),
            Self::Session(e) => write!(f, "Session error: {}", e),
            Self::Context(e) => write!(f, "Context error: {}", e),
            Self::Client(e) => write!(f, "Client error: {}", e),
            Self::MessageRouter(e) => write!(f, "Message router error: {}", e),
            Self::Plugin(e) => write!(f, "Plugin error: {}", e),
            Self::Serialization(e) => write!(f, "Serialization error: {}", e),
            Self::Deserialization(e) => write!(f, "Deserialization error: {}", e),
            Self::InvalidMessage(e) => write!(f, "Invalid message: {}", e),
            Self::State(e) => write!(f, "State error: {}", e),
            Self::Authorization(e) => write!(f, "Authorization error: {}", e),
            Self::UnsupportedOperation(e) => write!(f, "Unsupported operation: {}", e),
            Self::CircuitBreaker(e) => write!(f, "Circuit breaker error: {}", e),
            Self::IoDetail(e) => write!(f, "IO error: {}", e),
            Self::SerdeJsonDetail(e) => write!(f, "JSON error: {}", e),
            Self::SquirrelDetail(e) => write!(f, "Squirrel core error: {}", e),
            Self::PersistenceDetail(e) => write!(f, "Persistence error: {}", e),
            Self::Alert(e) => write!(f, "Alert error: {}", e),
            Self::Storage(e) => write!(f, "Storage error: {}", e),
            Self::NotInitialized(e) => write!(f, "Not initialized: {}", e),
            Self::General(e) => write!(f, "General error: {}", e),
            Self::Network(e) => write!(f, "Network error: {}", e),
            Self::AlreadyInProgress(e) => write!(f, "Already in progress: {}", e),
            Self::Monitoring(e) => write!(f, "Monitoring error: {}", e),
            Self::NotConnected(e) => write!(f, "Not connected: {}", e),
            Self::Timeout(e) => write!(f, "Timeout: {}", e),
            Self::Remote(e) => write!(f, "Remote error: {}", e),
            Self::Configuration(e) => write!(f, "Configuration error: {}", e),
            Self::Unexpected(e) => write!(f, "Unexpected error: {}", e),
            Self::VersionMismatch(e) => write!(f, "Version mismatch: {}", e),
            Self::Unsupported(e) => write!(f, "Unsupported: {}", e),
            Self::InvalidArgument(e) => write!(f, "Invalid argument: {}", e),
            Self::NotFound(e) => write!(f, "Not found: {}", e),
            Self::NotImplemented(e) => write!(f, "Not implemented: {}", e),
            Self::NotAuthorized(e) => write!(f, "Not authorized: {}", e),
            Self::InvalidState(e) => write!(f, "Invalid state: {}", e),
            Self::InvalidOperation(e) => write!(f, "Invalid operation: {}", e),
            Self::InternalError(e) => write!(f, "Internal error: {}", e),
        }
    }
}
