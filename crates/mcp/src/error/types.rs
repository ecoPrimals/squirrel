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

    /// Determines the severity level of this error
    ///
    /// Severity levels help prioritize error handling and reporting.
    /// Higher severity errors require more immediate attention or 
    /// may have a greater impact on system functionality.
    ///
    /// # Returns
    ///
    /// The severity level of the error as an ErrorSeverity enum value
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

/// Error handler with retry capabilities
///
/// Provides mechanisms for handling errors, including automatic retry with
/// configurable backoff, error context tracking, and recovery strategies.
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
    /// Creates a new ErrorHandler with the specified retry parameters
    ///
    /// # Arguments
    ///
    /// * `max_retries` - Maximum number of times to retry failed operations
    /// * `retry_delay` - How long to wait between retry attempts
    /// * `operation` - Name or description of the operation being handled
    /// * `component` - Name of the component where the operation is performed
    ///
    /// # Returns
    ///
    /// A new ErrorHandler configured with the specified parameters
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

    /// Gets the current error context
    ///
    /// # Returns
    ///
    /// A reference to the current error context
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
    /// Error when a connection could not be established with the remote endpoint
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    /// Error when a frame received or being sent is invalid or malformed
    #[error("Invalid frame: {0}")]
    InvalidFrame(String),
    
    /// Error when an operation timed out waiting for a response or connection
    #[error("Timeout: {0}")]
    Timeout(String),
    
    /// Error related to the communication protocol, such as invalid message format or sequence
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    
    /// Error when an existing connection was closed, either by the remote endpoint or locally
    #[error("Connection closed: {0}")]
    ConnectionClosed(String),
    
    /// Error originating from underlying I/O operations
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

/// Errors related to MCP connection operations
///
/// This enum represents errors that can occur when establishing or maintaining
/// network connections within the MCP system, including failures, timeouts, and
/// connection limit issues.
#[derive(Debug, Clone, Error)]
pub enum ConnectionError {
    /// Error that occurs when a connection cannot be established
    ///
    /// This can happen due to network issues, incorrect configuration,
    /// or when the remote endpoint is unavailable.
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    /// Error that occurs when a connection operation exceeds the time limit
    ///
    /// This happens when the connection process takes longer than the
    /// specified timeout period in milliseconds.
    #[error("Connection timeout after {0}ms")]
    Timeout(u64),
    
    /// Error that occurs when a connection is closed unexpectedly
    ///
    /// This can happen due to network issues, remote endpoint closure,
    /// or other connection disruptions.
    #[error("Connection closed: {0}")]
    Closed(String),
    
    /// Error that occurs when a connection is reset by the peer
    ///
    /// This typically happens when the remote endpoint forcibly
    /// closes the connection.
    #[error("Connection reset")]
    Reset,
    
    /// Error that occurs when a connection is refused by the remote endpoint
    ///
    /// This typically happens when the remote service is not running,
    /// or is configured to reject the connection.
    #[error("Connection refused")]
    Refused,
    
    /// Error that occurs when the network is unreachable
    ///
    /// This can happen due to network configuration issues, firewalls,
    /// or physical network disconnection.
    #[error("Network unreachable")]
    Unreachable,
    
    /// Error that occurs when too many concurrent connections are active
    ///
    /// This can happen when the system reaches its maximum connection capacity
    /// as defined by resource limits or configuration.
    #[error("Too many connections")]
    TooManyConnections,
    
    /// Error that occurs when a connection limit is reached for a specific reason
    ///
    /// This provides more context about why a connection limit was reached,
    /// such as per-user limits or rate limiting.
    #[error("Connection limit reached: {0}")]
    LimitReached(String),
}

/// Errors related to the MCP protocol
///
/// This enum represents various error conditions that can occur during protocol
/// operations, including version mismatches, invalid states, and message format errors.
#[derive(Debug, Clone, Error)]
pub enum ProtocolError {
    /// Error when the protocol version is invalid or incompatible
    #[error("Invalid protocol version: {0}")]
    InvalidVersion(String),
    
    /// Error when the protocol is in an invalid state for the requested operation
    #[error("Invalid protocol state: {0}")]
    InvalidState(String),
    
    /// Error when a message doesn't conform to the expected format
    #[error("Invalid message format: {0}")]
    InvalidFormat(String),
    
    /// Error when protocol negotiation fails between endpoints
    #[error("Protocol negotiation failed: {0}")]
    NegotiationFailed(String),
    
    /// Error when the protocol handshake process fails
    #[error("Protocol handshake failed: {0}")]
    HandshakeFailed(String),
    
    /// Error when protocol synchronization cannot be established
    #[error("Protocol synchronization failed: {0}")]
    SyncFailed(String),
    
    /// Error when a requested protocol capability is not supported
    #[error("Protocol capability not supported: {0}")]
    UnsupportedCapability(String),
    
    /// Error related to protocol configuration settings
    #[error("Protocol configuration error: {0}")]
    ConfigurationError(String),
    
    /// Error when trying to initialize a protocol that's already initialized
    #[error("Protocol already initialized")]
    ProtocolAlreadyInitialized,
    
    /// Error when using a protocol that hasn't been initialized
    #[error("Protocol not initialized")]
    ProtocolNotInitialized,
    
    /// Error when the protocol is not in a ready state for the operation
    #[error("Protocol not ready")]
    ProtocolNotReady,
    
    /// Error when serializing protocol state
    #[error("Failed to serialize state: {0}")]
    StateSerialization(String),
    
    /// Error when deserializing protocol state
    #[error("Failed to deserialize state: {0}")]
    StateDeserialization(String),
    
    /// Error when a handler already exists for a message type
    #[error("Handler already exists for message type: {0}")]
    HandlerAlreadyExists(String),
    
    /// Error when no handler is found for a message type
    #[error("No handler found for message type: {0}")]
    HandlerNotFound(String),
    
    /// Error when a message payload is invalid
    #[error("Invalid payload: {0}")]
    InvalidPayload(String),
    
    /// Error when a message exceeds the allowed size limit
    #[error("Message too large: {0}")]
    MessageTooLarge(String),
    
    /// Error when a message timestamp is invalid
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),
    
    /// Error when a message operation times out
    #[error("Message timeout: {0}")]
    MessageTimeout(String),
    
    /// Error when security metadata is invalid
    #[error("Invalid security metadata: {0}")]
    InvalidSecurityMetadata(String),
    
    /// Error when message validation fails
    #[error("Message validation failed: {0}")]
    ValidationFailed(String),
    
    /// Error when protocol recovery attempts fail
    #[error("Recovery failed: {0}")]
    RecoveryFailed(String),
    
    /// Error in the wire format encoding/decoding
    #[error("Wire format error: {0}")]
    Wire(String),
}

/// Security-related errors
#[derive(Debug, Clone, Error)]
pub enum SecurityError {
    /// Authentication error that occurs when credentials cannot be verified
    /// or the authentication process fails for any reason
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    /// Authorization error that occurs when a user lacks permissions
    /// to perform the requested operation
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),
    
    /// Error that occurs when provided credentials are invalid,
    /// malformed, or do not match expected format
    #[error("Invalid credentials: {0}")]
    InvalidCredentials(String),
    
    /// Error that occurs when an authentication token has expired
    /// and is no longer valid for use
    #[error("Token expired")]
    TokenExpired,
    
    /// Error that occurs when a token is invalid, corrupted,
    /// or cannot be verified
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    
    /// Error that occurs when a user role does not exist or
    /// is not valid in the current context
    #[error("Invalid role: {0}")]
    InvalidRole(String),
    
    /// Error that occurs during encryption operations, such as
    /// key generation, data encryption, or signature creation
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    
    /// Error that occurs during decryption operations, such as
    /// key retrieval, data decryption, or signature verification
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    
    /// General security error that occurs within the security
    /// subsystem but doesn't fit other specific categories
    #[error("Internal security error: {0}")]
    InternalError(String),
    
    /// Error that occurs when a message is sent with an insufficient
    /// security level for the operation being performed
    #[error("Invalid security level: required {required:?}, provided {provided:?}")]
    InvalidSecurityLevel {
        /// The security level required by the operation or receiver
        required: SecurityLevel,
        /// The security level provided in the message or request
        provided: SecurityLevel,
    },
    
    /// Error that occurs within the underlying system security
    /// infrastructure or OS security mechanisms
    #[error("System error: {0}")]
    System(String),
    
    /// Error that occurs when a permission string has invalid format
    /// or cannot be parsed correctly
    #[error("Invalid permission format: {0}")]
    InvalidPermissionFormat(String),
    
    /// Error that occurs when an action specified in a permission
    /// is not recognized or not supported
    #[error("Invalid action in permission: {0}")]
    InvalidActionInPermission(String),
    
    /// Error that occurs during the creation of a new role
    /// in the security system
    #[error("Error creating role: {0}")]
    ErrorCreatingRole(String),
    
    /// Error related to the Role-Based Access Control system,
    /// such as role assignment or permission checking
    #[error("RBAC error: {0}")]
    RBACError(String),
    
    /// Error that occurs during validation of security-related
    /// data or operations
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    /// Error that occurs when attempting to create a security
    /// entity with an ID that already exists
    #[error("Duplicate ID error: {0}")]
    DuplicateIDError(String),
    
    /// Error that occurs when a security-related entity
    /// could not be found
    #[error("Not found: {0}")]
    NotFound(String),
    
    /// Error that occurs when an operation would violate
    /// a defined security policy
    #[error("Policy violation: {0}")]
    PolicyViolation(String),
}

/// Errors related to MCP context operations
///
/// This enum represents errors that can occur when working with MCP contexts,
/// including context lookup failures, validation errors, and synchronization issues.
#[derive(Debug, Clone)]
pub enum ContextError {
    /// Error that occurs when a context with the specified UUID cannot be found
    ///
    /// This typically happens when trying to access a context that doesn't exist
    /// or has been removed.
    NotFound(uuid::Uuid),
    
    /// Error that occurs when context validation fails
    ///
    /// This can happen when a context contains invalid data or doesn't meet
    /// the required constraints.
    ValidationError(String),
    
    /// Error that occurs during context synchronization
    ///
    /// This can happen when there are issues synchronizing context data
    /// between components or systems.
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

/// Errors related to MCP session operations
///
/// This enum represents errors that can occur when working with MCP sessions,
/// including authentication and authorization failures, timeouts, and validation issues.
#[derive(Debug, Clone, Error)]
pub enum SessionError {
    /// Error that occurs when session authentication fails
    ///
    /// This typically happens when credentials cannot be verified
    /// or the authentication process fails for any reason.
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    /// Error that occurs when session authorization fails
    ///
    /// This typically happens when a user lacks the necessary permissions
    /// to perform a requested operation.
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),
    
    /// Error that occurs when a session times out
    ///
    /// This can happen when a session exceeds its maximum allowed
    /// duration or when there is no activity for a specified period.
    #[error("Session timeout: {0}")]
    Timeout(String),
    
    /// Error that occurs when a session is invalid
    ///
    /// This can happen when a session is malformed, corrupted,
    /// or doesn't meet the required constraints.
    #[error("Invalid session: {0}")]
    InvalidSession(String),
    
    /// Error that occurs when a session cannot be found
    ///
    /// This typically happens when trying to access a session
    /// that doesn't exist or has been removed.
    #[error("Session not found: {0}")]
    NotFound(String),
    
    /// Error that occurs during session validation
    ///
    /// This can happen when session data fails validation checks
    /// or doesn't meet the required constraints.
    #[error("Session validation error: {0}")]
    Validation(String),
    
    /// General internal error within the session management system
    ///
    /// This is used for errors that don't fit into other specific
    /// categories but occur within the session subsystem.
    #[error("Internal session error: {0}")]
    InternalError(String),
}

// Add the persistence error implementation
impl From<squirrel_core::error::PersistenceError> for MCPError {
    fn from(err: squirrel_core::error::PersistenceError) -> Self {
        MCPError::PersistenceDetail(err.to_string())
    }
}

/// Error related to the alert system
///
/// Represents errors that occur within the alert processing system,
/// including notification failures, alert validation errors, and 
/// alert delivery issues.
#[derive(Debug, Clone)]
pub struct AlertError(String);

impl AlertError {
    /// Creates a new AlertError with the specified message
    ///
    /// # Arguments
    ///
    /// * `message` - Description of the alert error
    ///
    /// # Returns
    ///
    /// A new AlertError instance
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
    /// A new ErrorContext with default values
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
    /// The updated ErrorContext for method chaining
    #[must_use]
    pub fn with_message_type(mut self, message_type: MessageType) -> Self {
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
    /// The updated ErrorContext for method chaining
    #[must_use]
    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
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
    /// The updated ErrorContext for method chaining
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
