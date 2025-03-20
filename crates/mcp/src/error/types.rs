use thiserror::Error;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use serde_json::Map;
use crate::types::{MessageType, SecurityLevel, ProtocolVersion};
use squirrel_core::error::{SquirrelError as CoreError, Result as CoreResult};
use crate::error::context::ErrorSeverity;
use uuid;

/// MCP specific error types
#[derive(Debug)]
pub enum MCPError {
    /// Context-related errors
    Context(ContextError),
    /// Protocol-related errors
    Protocol(ProtocolError),
    /// Security-related errors
    Security(SecurityError),
    /// IO errors
    Io(std::io::Error),
    /// Serde JSON errors
    SerdeJson(serde_json::Error),
    /// Connection errors
    Connection(ConnectionError),
    /// Storage related errors
    Storage(String),
    /// Not initialized error
    NotInitialized(String),
    /// General MCP errors
    General(String),
    /// Network related errors
    Network(String),
    /// Operation already in progress
    AlreadyInProgress(String),
}

impl std::fmt::Display for MCPError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MCPError::Context(err) => write!(f, "Context error: {err}"),
            MCPError::Protocol(err) => write!(f, "Protocol error: {err}"),
            MCPError::Security(err) => write!(f, "Security error: {err}"),
            MCPError::Io(err) => write!(f, "IO error: {err}"),
            MCPError::SerdeJson(err) => write!(f, "Serde JSON error: {err}"),
            MCPError::Connection(err) => write!(f, "Connection error: {err}"),
            MCPError::Storage(err) => write!(f, "Storage error: {err}"),
            MCPError::NotInitialized(err) => write!(f, "Not initialized: {err}"),
            MCPError::General(err) => write!(f, "MCP error: {err}"),
            MCPError::Network(err) => write!(f, "Network error: {err}"),
            MCPError::AlreadyInProgress(err) => write!(f, "Already in progress: {err}"),
        }
    }
}

// We need to create newtype wrappers for String errors to implement std::error::Error
#[derive(Debug)]
pub struct StringError(pub String);

impl std::fmt::Display for StringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for StringError {}

impl std::error::Error for MCPError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MCPError::Context(err) => Some(err),
            MCPError::Protocol(err) => Some(err),
            MCPError::Security(err) => Some(err),
            MCPError::Io(err) => Some(err),
            MCPError::SerdeJson(err) => Some(err),
            MCPError::Connection(err) => Some(err),
            MCPError::Storage(_) => None,
            MCPError::NotInitialized(_) => None,
            MCPError::General(_) => None,
            MCPError::Network(_) => None,
            MCPError::AlreadyInProgress(_) => None,
        }
    }
}

// Add From implementations for various error types
impl From<std::io::Error> for MCPError {
    fn from(err: std::io::Error) -> Self {
        MCPError::Io(err)
    }
}

impl From<serde_json::Error> for MCPError {
    fn from(err: serde_json::Error) -> Self {
        MCPError::SerdeJson(err)
    }
}

impl From<ConnectionError> for MCPError {
    fn from(err: ConnectionError) -> Self {
        MCPError::Connection(err)
    }
}

impl From<ProtocolError> for MCPError {
    fn from(err: ProtocolError) -> Self {
        MCPError::Protocol(err)
    }
}

impl From<MCPError> for CoreError {
    fn from(err: MCPError) -> Self {
        match err {
            MCPError::Context(e) => CoreError::MCP(format!("Context error: {e}")),
            MCPError::Protocol(e) => CoreError::MCP(format!("Protocol error: {e}")),
            MCPError::Io(e) => CoreError::MCP(format!("IO error: {e}")),
            MCPError::SerdeJson(e) => CoreError::MCP(format!("Serialization error: {e}")),
            MCPError::Security(e) => CoreError::MCP(format!("Security error: {e}")),
            MCPError::Connection(e) => CoreError::MCP(format!("Connection error: {e}")),
            MCPError::Storage(e) => CoreError::MCP(format!("Storage error: {e}")),
            MCPError::NotInitialized(e) => CoreError::MCP(format!("Not initialized: {e}")),
            MCPError::General(e) => CoreError::MCP(format!("MCP error: {e}")),
            MCPError::Network(e) => CoreError::MCP(format!("Network error: {e}")),
            MCPError::AlreadyInProgress(e) => CoreError::MCP(format!("Already in progress: {e}")),
        }
    }
}

impl From<ContextError> for MCPError {
    fn from(err: ContextError) -> Self {
        MCPError::Context(err)
    }
}

impl From<squirrel_core::error::PersistenceError> for MCPError {
    fn from(err: squirrel_core::error::PersistenceError) -> Self {
        MCPError::Storage(format!("Persistence error: {}", err))
    }
}

impl From<SecurityError> for MCPError {
    fn from(err: SecurityError) -> Self {
        MCPError::Security(err)
    }
}

/// Context-related errors
#[derive(Debug)]
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

#[derive(Error, Debug)]
pub enum PortErrorKind {
    #[error("Port {0} is already in use")]
    PortInUse(u16),
    #[error("Port {0} is not allowed")]
    PortNotAllowed(u16),
    #[error("Port {0} is reserved")]
    PortReserved(u16),
    #[error("Port {0} is invalid")]
    InvalidPort(u16),
    #[error("Port {0} is not available")]
    NotAvailable(u16),
    #[error("Port {0} is already in use")]
    AlreadyInUse(u16),
    #[error("Port {0} is out of range")]
    InvalidRange(u16),
    #[error("Access denied for port {0}")]
    AccessDenied(u16),
}

#[derive(Debug, Error)]
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
}

#[derive(Debug, Error)]
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

#[derive(Debug, Error)]
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub timestamp: DateTime<Utc>,
    pub operation: String,
    pub component: String,
    pub message_type: Option<MessageType>,
    pub details: Map<String, serde_json::Value>,
    pub severity: ErrorSeverity,
    pub is_recoverable: bool,
    pub retry_count: u32,
    pub error_code: String,
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

    #[must_use] pub fn with_message_type(mut self, message_type: MessageType) -> Self {
        self.message_type = Some(message_type);
        self
    }

    #[must_use] pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }

    #[must_use] pub fn with_details(mut self, details: Map<String, serde_json::Value>) -> Self {
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

impl MCPError {
    #[must_use] pub fn is_recoverable(&self) -> bool {
        match self {
            // Recoverable errors
            MCPError::Security(SecurityError::AuthenticationFailed(_) | 
                           SecurityError::TokenExpired) |
            MCPError::Connection(ConnectionError::Timeout(_) | 
                              ConnectionError::Reset) => true,
            
            // Default case - all other errors are non-recoverable
            _ => false,
        }
    }

    #[must_use] pub fn severity(&self) -> ErrorSeverity {
        match self {
            MCPError::Connection(
                ConnectionError::ConnectionFailed(_) |
                ConnectionError::Closed(_) |
                ConnectionError::Reset |
                ConnectionError::Refused |
                ConnectionError::Unreachable |
                ConnectionError::TooManyConnections |
                ConnectionError::LimitReached(_)
            ) => ErrorSeverity::High,
            
            MCPError::Protocol(ProtocolError::InvalidVersion(_)) => ErrorSeverity::High,
            
            // All other errors are low severity
            _ => ErrorSeverity::Low,
        }
    }

    #[must_use] pub fn error_code(&self) -> String {
        match self {
            MCPError::Context(_) => "MCP-001",
            MCPError::Protocol(_) => "MCP-002",
            MCPError::Security(_) => "MCP-003",
            MCPError::SerdeJson(_) => "MCP-004",
            MCPError::Io(_) => "MCP-005",
            MCPError::Connection(_) => "MCP-006",
            MCPError::Storage(_) => "MCP-007",
            MCPError::NotInitialized(_) => "MCP-008",
            MCPError::General(_) => "MCP-009",
            MCPError::Network(_) => "MCP-010",
            MCPError::AlreadyInProgress(_) => "MCP-011",
        }.to_string()
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
                    
                    if !error.is_recoverable() || self.error_context.retry_count >= self.max_retries {
                        return Err(error);
                    }
                    
                    tokio::time::sleep(self.retry_delay).await;
                }
            }
        }
    }

    #[must_use] pub fn error_context(&self) -> &ErrorContext {
        &self.error_context
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

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
        let version_mismatch = MCPError::Protocol(ProtocolError::InvalidVersion("Version mismatch".to_string()));
        assert!(!version_mismatch.is_recoverable());
        assert_eq!(version_mismatch.severity(), ErrorSeverity::High);

        let timeout = MCPError::Connection(ConnectionError::Timeout(5000));
        assert!(timeout.is_recoverable());
        assert_eq!(timeout.severity(), ErrorSeverity::Low);
    }
}