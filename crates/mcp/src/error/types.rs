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
    /// Transport errors
    Transport(TransportError),
    
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
    
    /// IO errors - Use string representation since std::io::Error is not Clone
    IoDetail(String),
    
    /// JSON errors (serde_json) - Use string representation since serde_json::Error is not Clone
    SerdeJsonDetail(String),
    
    /// Squirrel core errors - Use string representation since SquirrelError is not Clone
    SquirrelDetail(String),
    
    /// Persistence errors - Use string representation since PersistenceError is not Clone
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
}

/// Error kinds for port-related errors.
///
/// These represent different ways in which network port operations can fail,
/// such as when a port is not available, access is denied, or the port is
/// outside the valid range.
#[derive(Debug, Clone, Error)]
pub enum PortErrorKind {
    /// The requested port is not available
    #[error("Port {0} is not available")]
    NotAvailable(u16),
    
    /// Access to the requested port was denied
    #[error("Access to port {0} was denied")]
    AccessDenied(u16),
    
    /// The port is outside the valid range
    #[error("Port {0} is outside the valid range")]
    InvalidRange(u16),
    
    /// The port is already in use
    #[error("Port {0} is already in use")]
    InUse(u16),
    
    /// A generic port-related error
    #[error("Port error: {0}")]
    Other(String),
}

impl std::fmt::Display for MCPError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MCPError::Context(err) => write!(f, "Context error: {err}"),
            MCPError::Protocol(err) => write!(f, "Protocol error: {err}"),
            MCPError::Security(err) => write!(f, "Security error: {err}"),
            MCPError::Connection(err) => write!(f, "Connection error: {err}"),
            MCPError::Session(err) => write!(f, "Session error: {err}"),
            MCPError::Transport(err) => write!(f, "Transport error: {err}"),
            MCPError::Client(err) => write!(f, "Client error: {err}"),
            MCPError::MessageRouter(err) => write!(f, "Message router error: {err}"),
            MCPError::Serialization(msg) => write!(f, "Serialization error: {msg}"),
            MCPError::Deserialization(err) => write!(f, "Deserialization error: {err}"),
            MCPError::InvalidMessage(err) => write!(f, "Invalid message: {err}"),
            MCPError::State(err) => write!(f, "State error: {err}"),
            MCPError::Authorization(err) => write!(f, "Authorization error: {err}"),
            MCPError::UnsupportedOperation(err) => write!(f, "Unsupported operation: {err}"),
            MCPError::CircuitBreaker(err) => write!(f, "Circuit breaker error: {err}"),
            MCPError::IoDetail(err) => write!(f, "IO error: {err}"),
            MCPError::SerdeJsonDetail(err) => write!(f, "Serde JSON error: {err}"),
            MCPError::SquirrelDetail(err) => write!(f, "Squirrel error: {err}"),
            MCPError::PersistenceDetail(err) => write!(f, "Persistence error: {err}"),
            MCPError::Alert(err) => write!(f, "Alert error: {err}"),
            MCPError::Storage(err) => write!(f, "Storage error: {err}"),
            MCPError::NotInitialized(err) => write!(f, "Not initialized: {err}"),
            MCPError::General(err) => write!(f, "MCP error: {err}"),
            MCPError::Network(err) => write!(f, "Network error: {err}"),
            MCPError::AlreadyInProgress(err) => write!(f, "Already in progress: {err}"),
            MCPError::Monitoring(err) => write!(f, "Monitoring error: {err}"),
            MCPError::NotConnected(err) => write!(f, "Not connected: {err}"),
            MCPError::Timeout(err) => write!(f, "Timeout: {err}"),
            MCPError::Remote(err) => write!(f, "Remote error: {err}"),
            MCPError::Configuration(err) => write!(f, "Configuration error: {err}"),
            MCPError::Unexpected(err) => write!(f, "Unexpected error: {err}"),
            MCPError::VersionMismatch(err) => write!(f, "Version mismatch: {err}"),
            MCPError::Unsupported(err) => write!(f, "Unsupported: {err}"),
            MCPError::InvalidArgument(err) => write!(f, "Invalid argument: {err}"),
            MCPError::NotFound(err) => write!(f, "Not found: {err}"),
            MCPError::NotImplemented(err) => write!(f, "Not implemented: {err}"),
            MCPError::NotAuthorized(err) => write!(f, "Not authorized: {err}"),
            MCPError::InvalidState(err) => write!(f, "Invalid state error: {err}"),
        }
    }
}

/// String-based error wrapper.
///
/// This is a newtype wrapper around String that implements `std::error::Error`,
/// allowing string errors to be used with the error machinery.
#[derive(Debug)]
pub struct StringError(pub String);

impl std::fmt::Display for StringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for StringError {}

impl MCPError {
    /// Creates an error from an MCP error message
    pub fn from_message(message: &crate::message::Message) -> Self {
        // Extract error details from message metadata
        let error_type = message.metadata.get("error_type")
            .map(|s| s.as_str())
            .unwrap_or("unknown");

        // Create appropriate error based on type
        match error_type {
            "transport" => MCPError::Transport(crate::error::transport::TransportError::ProtocolError(message.content.clone()).into()),
            "protocol" => MCPError::Protocol(ProtocolError::InvalidFormat(message.content.clone())),
            "security" => MCPError::Security(SecurityError::AuthenticationFailed(message.content.clone())),
            "connection" => MCPError::Connection(ConnectionError::Closed(message.content.clone())),
            "session" => MCPError::Session(SessionError::InvalidSession(message.content.clone())),
            "context" => MCPError::Context(ContextError::NotFound(uuid::Uuid::new_v4())),
            "client" => MCPError::Client(crate::error::client::ClientError::RemoteError(message.content.clone())),
            _ => MCPError::Remote(message.content.clone()),
        }
    }

    #[must_use]
    pub fn is_recoverable(&self) -> bool {
        match self {
            // Recoverable errors
            MCPError::Security(
                SecurityError::AuthenticationFailed(_) | SecurityError::TokenExpired,
            )
            | MCPError::Connection(ConnectionError::Timeout(_) | ConnectionError::Reset) => true,

            // Default case - all other errors are non-recoverable
            _ => false,
        }
    }

    #[must_use]
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            MCPError::Connection(
                ConnectionError::ConnectionFailed(_)
                | ConnectionError::Closed(_)
                | ConnectionError::Reset
                | ConnectionError::Refused
                | ConnectionError::Unreachable
                | ConnectionError::TooManyConnections
                | ConnectionError::LimitReached(_),
            ) => ErrorSeverity::High,

            MCPError::Protocol(ProtocolError::InvalidVersion(_)) => ErrorSeverity::High,

            // All other errors are low severity
            _ => ErrorSeverity::Low,
        }
    }

    /// Returns a string error code for this error.
    ///
    /// The error code consists of a category prefix and a numeric code, e.g., "MCP-001".
    /// This can be used for error tracking and reporting.
    pub fn error_code(&self) -> String {
        match self {
            MCPError::Transport(_) => "MCP-001",
            MCPError::Protocol(_) => "MCP-002",
            MCPError::Security(_) => "MCP-003",
            MCPError::Connection(_) => "MCP-004",
            MCPError::Session(_) => "MCP-005",
            MCPError::Context(_) => "MCP-006",
            MCPError::Serialization(_) => "MCP-007",
            MCPError::Deserialization(_) => "MCP-008",
            MCPError::InvalidMessage(_) => "MCP-009",
            MCPError::State(_) => "MCP-010",
            MCPError::Authorization(_) => "MCP-011",
            MCPError::UnsupportedOperation(_) => "MCP-012",
            MCPError::CircuitBreaker(_) => "MCP-013",
            MCPError::IoDetail(_) => "MCP-014",
            MCPError::SerdeJsonDetail(_) => "MCP-015",
            MCPError::SquirrelDetail(_) => "MCP-016",
            MCPError::PersistenceDetail(_) => "MCP-017",
            MCPError::Alert(_) => "MCP-018",
            MCPError::Storage(_) => "MCP-019",
            MCPError::NotInitialized(_) => "MCP-020",
            MCPError::General(_) => "MCP-021",
            MCPError::Network(_) => "MCP-022",
            MCPError::AlreadyInProgress(_) => "MCP-023",
            MCPError::Monitoring(_) => "MCP-024",
            MCPError::NotConnected(_) => "MCP-025",
            MCPError::Timeout(_) => "MCP-026",
            MCPError::Remote(_) => "MCP-027",
            MCPError::Configuration(_) => "MCP-028",
            MCPError::Unexpected(_) => "MCP-029",
            MCPError::VersionMismatch(_) => "MCP-030",
            MCPError::Unsupported(_) => "MCP-031",
            MCPError::InvalidArgument(_) => "MCP-032",
            MCPError::NotFound(_) => "MCP-033",
            MCPError::NotImplemented(_) => "MCP-034",
            MCPError::NotAuthorized(_) => "MCP-035",
            MCPError::InvalidState(_) => "MCP-036",
            MCPError::Client(_) => "MCP-037",
            MCPError::MessageRouter(_) => "MCP-038",
        }
        .to_string()
    }
}

impl std::error::Error for MCPError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MCPError::Context(err) => Some(err),
            MCPError::Protocol(err) => Some(err),
            MCPError::Security(err) => Some(err),
            MCPError::IoDetail(_) => None,
            MCPError::SerdeJsonDetail(_) => None,
            MCPError::Connection(err) => Some(err),
            MCPError::Session(err) => Some(err),
            MCPError::Transport(err) => Some(err),
            MCPError::Client(err) => Some(err),
            MCPError::MessageRouter(err) => Some(err),
            MCPError::SquirrelDetail(_) => None,
            MCPError::PersistenceDetail(_) => None,
            MCPError::Alert(err) => Some(err),
            MCPError::Storage(_) => None,
            MCPError::NotInitialized(_) => None,
            MCPError::General(_) => None,
            MCPError::Network(_) => None,
            MCPError::AlreadyInProgress(_) => None,
            MCPError::Monitoring(_) => None,
            MCPError::Serialization(_) => None,
            MCPError::Deserialization(_) => None,
            MCPError::InvalidMessage(_) => None,
            MCPError::State(_) => None,
            MCPError::Authorization(_) => None,
            MCPError::UnsupportedOperation(_) => None,
            MCPError::CircuitBreaker(_) => None,
            MCPError::NotConnected(_) => None,
            MCPError::Timeout(_) => None,
            MCPError::Remote(_) => None,
            MCPError::Configuration(_) => None,
            MCPError::Unexpected(_) => None,
            MCPError::VersionMismatch(_) => None,
            MCPError::Unsupported(_) => None,
            MCPError::InvalidArgument(_) => None,
            MCPError::NotFound(_) => None,
            MCPError::NotImplemented(_) => None,
            MCPError::NotAuthorized(_) => None,
            MCPError::InvalidState(_) => None,
        }
    }
}

// Add From implementations for various error types
impl From<std::io::Error> for MCPError {
    fn from(err: std::io::Error) -> Self {
        MCPError::IoDetail(err.to_string())
    }
}

impl From<serde_json::Error> for MCPError {
    fn from(err: serde_json::Error) -> Self {
        MCPError::SerdeJsonDetail(err.to_string())
    }
}

#[derive(Debug)]
pub struct ErrorHandler {
    /// Maximum number of retry attempts
    /// This defines how many times the handler will retry an operation before giving up
    max_retries: u32,
    /// Delay between retry attempts
    /// Specifies how long to wait between retry attempts
    retry_delay: std::time::Duration,
    /// Context information for errors
    /// Contains metadata and context about the errors being handled
    error_context: ErrorContext,
}

impl ErrorHandler {
    pub fn new(
        max_retries: u32,
        retry_delay: std::time::Duration,
        operation: impl Into<String>,
        component: impl Into<String>,
    ) -> Self {
        Self {
            max_retries,
            retry_delay,
            error_context: ErrorContext::new(operation, component),
        }
    }

    /// Handles operation errors with automatic retries
    ///
    /// # Arguments
    /// * `operation` - A closure that returns a `CoreResult<T>`
    ///
    /// # Returns
    /// * `CoreResult<T>` - The result of the operation or the last error encountered
    ///
    /// # Errors
    /// Returns an error if the operation failed after all retry attempts or
    /// if the error is not recoverable
    pub async fn handle_error<F, T>(&mut self, operation: F) -> CoreResult<T>
    where
        F: Fn() -> CoreResult<T> + Send + Sync,
    {
        loop {
            match operation() {
                Ok(result) => return Ok(result),
                Err(error) => {
                    self.error_context.increment_retry_count();

                    if !error.is_recoverable() || self.error_context.retry_count >= self.max_retries
                    {
                        return Err(error);
                    }

                    tokio::time::sleep(self.retry_delay).await;
                }
            }
        }
    }

    #[must_use]
    pub fn error_context(&self) -> &ErrorContext {
        &self.error_context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_context() {
        let context = ErrorContext::new("test_op", "test_component")
            .with_message_type(MessageType::Command)
            .with_severity(ErrorSeverity::High)
            .with_error_code("TEST-001")
            .with_source_location("test.rs:42");

        assert_eq!(context.operation, "test_op");
        assert_eq!(context.component, "test_component");
        assert_eq!(context.message_type, Some(MessageType::Command));
        assert_eq!(context.severity, ErrorSeverity::High);
        assert_eq!(context.error_code, "TEST-001");
        assert_eq!(context.source_location, Some("test.rs:42".to_string()));
    }

    #[test]
    fn test_error_recovery() {
        let version_mismatch = MCPError::Protocol(ProtocolError::InvalidVersion(
            "Version mismatch".to_string(),
        ));
        assert!(!version_mismatch.is_recoverable());
        assert_eq!(version_mismatch.severity(), ErrorSeverity::High);

        let timeout = MCPError::Connection(ConnectionError::Timeout(5000));
        assert!(timeout.is_recoverable());
        assert_eq!(timeout.severity(), ErrorSeverity::Low);
    }
}

/// Transport-related errors (simplified version)
/// 
/// This is a simplified version of the `TransportError` from the 
/// `crates/mcp/src/error/transport.rs` module. 
///
/// **DEPRECATED**: Use `crate::error::transport::TransportError` instead.
/// This exists for backward compatibility and will be removed in a future release.
#[deprecated(
    since = "0.5.0",
    note = "Please use crate::error::transport::TransportError instead"
)]
#[derive(Debug, Clone, Error)]
pub enum TransportError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Invalid frame: {0}")]
    InvalidFrame(String),
    #[error("Timeout: {0}")]
    Timeout(String),
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    #[error("Connection closed: {0}")]
    ConnectionClosed(String),
    #[error("I/O error: {0}")]
    IoError(String),
}

// Add conversion from canonical error type to this simplified one
impl From<crate::error::transport::TransportError> for TransportError {
    fn from(err: crate::error::transport::TransportError) -> Self {
        match err {
            crate::error::transport::TransportError::ConnectionFailed(msg) => 
                TransportError::ConnectionFailed(msg),
            crate::error::transport::TransportError::InvalidFrame(msg) => 
                TransportError::InvalidFrame(msg),
            crate::error::transport::TransportError::Timeout(msg) => 
                TransportError::Timeout(msg),
            crate::error::transport::TransportError::ProtocolError(msg) => 
                TransportError::ProtocolError(msg),
            crate::error::transport::TransportError::ConnectionClosed(msg) => 
                TransportError::ConnectionClosed(msg),
            crate::error::transport::TransportError::IoError(e) => 
                TransportError::IoError(e.to_string()),
            crate::error::transport::TransportError::SecurityError(msg) => 
                TransportError::ConnectionFailed(format!("Security error: {}", msg)),
            crate::error::transport::TransportError::SerializationError(e) => 
                TransportError::InvalidFrame(format!("Serialization error: {}", e)),
            crate::error::transport::TransportError::ConfigurationError(msg) => 
                TransportError::ConnectionFailed(format!("Configuration error: {}", msg)),
            crate::error::transport::TransportError::UnsupportedOperation(msg) => 
                TransportError::ProtocolError(format!("Unsupported operation: {}", msg)),
        }
    }
}

// Update conversion from MCPError to TransportError
impl From<MCPError> for TransportError {
    fn from(err: MCPError) -> Self {
        match err {
            MCPError::Transport(e) => e,
            MCPError::Protocol(e) => TransportError::ProtocolError(e.to_string()),
            MCPError::Connection(e) => TransportError::ConnectionFailed(e.to_string()),
            MCPError::Timeout(msg) => TransportError::Timeout(msg),
            MCPError::NotConnected(msg) => TransportError::ConnectionClosed(msg),
            _ => TransportError::ConnectionFailed(format!("Error converted from MCPError: {}", err)),
        }
    }
}

// Add conversion from this simplified error type to canonical one
impl From<TransportError> for crate::error::transport::TransportError {
    fn from(err: TransportError) -> Self {
        match err {
            TransportError::ConnectionFailed(msg) => 
                crate::error::transport::TransportError::ConnectionFailed(msg),
            TransportError::InvalidFrame(msg) => 
                crate::error::transport::TransportError::InvalidFrame(msg),
            TransportError::Timeout(msg) => 
                crate::error::transport::TransportError::Timeout(msg),
            TransportError::ProtocolError(msg) => 
                crate::error::transport::TransportError::ProtocolError(msg),
            TransportError::ConnectionClosed(msg) => 
                crate::error::transport::TransportError::ConnectionClosed(msg),
            TransportError::IoError(msg) => 
                crate::error::transport::TransportError::IoError(std::io::Error::new(std::io::ErrorKind::Other, msg)),
        }
    }
}

// Add implementation to directly use canonical error type in MCPError
impl From<crate::error::transport::TransportError> for MCPError {
    fn from(err: crate::error::transport::TransportError) -> Self {
        MCPError::Transport(TransportError::from(err))
    }
}

/// Connection-related errors
#[derive(Debug, Clone, Error)]
pub enum ConnectionError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Connection timeout after {0}ms")]
    Timeout(u64),
    #[error("Connection closed: {0}")]
    Closed(String),
    #[error("Connection reset")]
    Reset,
    #[error("Connection refused")]
    Refused,
    #[error("Network unreachable")]
    Unreachable,
    #[error("Too many connections")]
    TooManyConnections,
    #[error("Connection limit reached: {0}")]
    LimitReached(String),
}

/// Protocol-related errors
#[derive(Debug, Clone, Error)]
pub enum ProtocolError {
    #[error("Invalid protocol version: {0}")]
    InvalidVersion(String),
    #[error("Invalid protocol state: {0}")]
    InvalidState(String),
    #[error("Invalid message format: {0}")]
    InvalidFormat(String),
    #[error("Protocol negotiation failed: {0}")]
    NegotiationFailed(String),
    #[error("Protocol handshake failed: {0}")]
    HandshakeFailed(String),
    #[error("Protocol synchronization failed: {0}")]
    SyncFailed(String),
    #[error("Protocol capability not supported: {0}")]
    UnsupportedCapability(String),
    #[error("Protocol configuration error: {0}")]
    ConfigurationError(String),
    #[error("Protocol already initialized")]
    ProtocolAlreadyInitialized,
    #[error("Protocol not initialized")]
    ProtocolNotInitialized,
    #[error("Protocol not ready")]
    ProtocolNotReady,
    #[error("Failed to serialize state: {0}")]
    StateSerialization(String),
    #[error("Failed to deserialize state: {0}")]
    StateDeserialization(String),
    #[error("Handler already exists for message type: {0}")]
    HandlerAlreadyExists(String),
    #[error("No handler found for message type: {0}")]
    HandlerNotFound(String),
    #[error("Invalid payload: {0}")]
    InvalidPayload(String),
    #[error("Message too large: {0}")]
    MessageTooLarge(String),
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),
    #[error("Message timeout: {0}")]
    MessageTimeout(String),
    #[error("Invalid security metadata: {0}")]
    InvalidSecurityMetadata(String),
    #[error("Message validation failed: {0}")]
    ValidationFailed(String),
    #[error("Recovery failed: {0}")]
    RecoveryFailed(String),
    #[error("Wire format error: {0}")]
    Wire(String),
}

/// Security-related errors
#[derive(Debug, Clone, Error)]
pub enum SecurityError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),
    #[error("Invalid credentials: {0}")]
    InvalidCredentials(String),
    #[error("Token expired")]
    TokenExpired,
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    #[error("Invalid role: {0}")]
    InvalidRole(String),
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Internal security error: {0}")]
    InternalError(String),
    #[error("Invalid security level: required {required:?}, provided {provided:?}")]
    InvalidSecurityLevel {
        required: SecurityLevel,
        provided: SecurityLevel,
    },
    #[error("System error: {0}")]
    System(String),
    #[error("Invalid permission format: {0}")]
    InvalidPermissionFormat(String),
    #[error("Invalid action in permission: {0}")]
    InvalidActionInPermission(String),
    #[error("Error creating role: {0}")]
    ErrorCreatingRole(String),
    #[error("RBAC error: {0}")]
    RBACError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Duplicate ID error: {0}")]
    DuplicateIDError(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Policy violation: {0}")]
    PolicyViolation(String),
}

/// Context-related errors
#[derive(Debug, Clone)]
pub enum ContextError {
    /// Context not found
    NotFound(uuid::Uuid),
    /// Context validation error
    ValidationError(String),
    /// Context synchronization error
    SyncError(String),
}

impl std::fmt::Display for ContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "Context not found: {}", id),
            Self::ValidationError(msg) => write!(f, "Context validation error: {}", msg),
            Self::SyncError(msg) => write!(f, "Context sync error: {}", msg),
        }
    }
}

impl std::error::Error for ContextError {}

// Fix the implementation using CoreErrorCode - we'll comment it out for now
// impl From<&MCPError> for CoreErrorCode {
//    fn from(err: &MCPError) -> Self {
//        ...
//    }
// }

// Implementation for session error
#[derive(Debug, Clone, Error)]
pub enum SessionError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),
    #[error("Session timeout: {0}")]
    Timeout(String),
    #[error("Invalid session: {0}")]
    InvalidSession(String),
    #[error("Session not found: {0}")]
    NotFound(String),
    #[error("Session validation error: {0}")]
    Validation(String),
    #[error("Internal session error: {0}")]
    InternalError(String),
}

// Add the persistence error implementation
impl From<squirrel_core::error::PersistenceError> for MCPError {
    fn from(err: squirrel_core::error::PersistenceError) -> Self {
        MCPError::PersistenceDetail(err.to_string())
    }
}

// Restore the AlertError definition
#[derive(Debug, Clone)]
pub struct AlertError(String);

impl AlertError {
    pub fn new(message: String) -> Self {
        Self(message)
    }
}

impl std::fmt::Display for AlertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Alert error: {}", self.0)
    }
}

impl std::error::Error for AlertError {}

// Add implementation for From<SessionError> for MCPError
impl From<SessionError> for MCPError {
    fn from(err: SessionError) -> Self {
        MCPError::Session(err)
    }
}

/// Provides additional context information for errors.
///
/// This struct contains metadata about an error, including when it occurred,
/// what operation was being performed, which component raised the error,
/// and other relevant contextual information.
///
/// # Examples
///
/// ```
/// use mcp::error::{ErrorContext, ErrorSeverity};
/// use chrono::Utc;
/// use serde_json::Map;
///
/// let context = ErrorContext::new("read_file", "file_system")
///     .with_severity(ErrorSeverity::Error)
///     .with_error_code("FS-001");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// When the error occurred
    pub timestamp: DateTime<Utc>,
    /// Operation being performed when the error occurred
    pub operation: String,
    /// Component where the error occurred
    pub component: String,
    /// Type of message being processed, if applicable
    pub message_type: Option<MessageType>,
    /// Additional details about the error
    pub details: Map<String, serde_json::Value>,
    /// Severity level of the error
    pub severity: ErrorSeverity,
    /// Whether the error can be recovered from
    pub is_recoverable: bool,
    /// Number of retry attempts made so far
    pub retry_count: u32,
    /// Unique error code for identification
    pub error_code: String,
    /// Code location where the error occurred
    pub source_location: Option<String>,
}

impl ErrorContext {
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

    #[must_use]
    pub fn with_message_type(mut self, message_type: MessageType) -> Self {
        self.message_type = Some(message_type);
        self
    }

    #[must_use]
    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }

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

    pub fn increment_retry_count(&mut self) {
        self.retry_count += 1;
    }
}

// Add the implementation for From<MCPError> for SquirrelError
impl From<MCPError> for CoreError {
    fn from(err: MCPError) -> Self {
        match err {
            MCPError::Transport(e) => CoreError::MCP(format!("Transport error: {}", e)),
            MCPError::Protocol(e) => CoreError::MCP(format!("Protocol error: {}", e)),
            MCPError::Security(e) => CoreError::MCP(format!("Security error: {}", e)),
            MCPError::Connection(e) => CoreError::MCP(format!("Connection error: {}", e)),
            MCPError::Session(e) => CoreError::MCP(format!("Session error: {}", e)),
            MCPError::Context(e) => CoreError::MCP(format!("Context error: {}", e)),
            MCPError::Client(e) => CoreError::MCP(format!("Client error: {}", e)),
            MCPError::MessageRouter(e) => CoreError::MCP(format!("Message router error: {}", e)),
            MCPError::Serialization(e) => CoreError::MCP(format!("Serialization error: {}", e)),
            MCPError::Deserialization(e) => CoreError::MCP(format!("Deserialization error: {}", e)),
            MCPError::InvalidMessage(e) => CoreError::MCP(format!("Invalid message: {}", e)),
            MCPError::State(e) => CoreError::MCP(format!("State error: {}", e)),
            MCPError::Authorization(e) => CoreError::MCP(format!("Authorization error: {}", e)),
            MCPError::UnsupportedOperation(e) => CoreError::MCP(format!("Unsupported operation: {}", e)),
            MCPError::CircuitBreaker(e) => CoreError::MCP(format!("Circuit breaker error: {}", e)),
            MCPError::IoDetail(e) => CoreError::MCP(format!("IO error: {}", e)),
            MCPError::SerdeJsonDetail(e) => CoreError::MCP(format!("Serde JSON error: {}", e)),
            MCPError::SquirrelDetail(e) => CoreError::MCP(format!("Squirrel error: {}", e)),
            MCPError::PersistenceDetail(e) => CoreError::Persistence(squirrel_core::error::PersistenceError::Storage(e.to_string())),
            MCPError::Alert(e) => CoreError::MCP(format!("Alert error: {}", e)),
            MCPError::Storage(e) => CoreError::MCP(format!("Storage error: {}", e)),
            MCPError::NotInitialized(e) => CoreError::MCP(format!("Not initialized: {}", e)),
            MCPError::General(e) => CoreError::MCP(format!("General error: {}", e)),
            MCPError::Network(e) => CoreError::MCP(format!("Network error: {}", e)),
            MCPError::AlreadyInProgress(e) => CoreError::MCP(format!("Already in progress: {}", e)),
            MCPError::Monitoring(e) => CoreError::MCP(format!("Monitoring error: {}", e)),
            MCPError::NotConnected(e) => CoreError::MCP(format!("Not connected: {}", e)),
            MCPError::Timeout(e) => CoreError::MCP(format!("Timeout: {}", e)),
            MCPError::Remote(e) => CoreError::MCP(format!("Remote error: {}", e)),
            MCPError::Configuration(e) => CoreError::MCP(format!("Configuration error: {}", e)),
            MCPError::Unexpected(e) => CoreError::MCP(format!("Unexpected error: {}", e)),
            MCPError::VersionMismatch(e) => CoreError::MCP(format!("Version mismatch: {}", e)),
            MCPError::Unsupported(e) => CoreError::MCP(format!("Unsupported: {}", e)),
            MCPError::InvalidArgument(e) => CoreError::MCP(format!("Invalid argument: {}", e)),
            MCPError::NotFound(e) => CoreError::MCP(format!("Not found: {}", e)),
            MCPError::NotImplemented(e) => CoreError::MCP(format!("Not implemented: {}", e)),
            MCPError::NotAuthorized(e) => CoreError::MCP(format!("Not authorized: {}", e)),
            MCPError::InvalidState(e) => CoreError::MCP(format!("Invalid state: {}", e)),
        }
    }
}
