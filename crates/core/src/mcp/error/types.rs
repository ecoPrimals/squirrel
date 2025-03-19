use thiserror::Error;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use serde_json::Map;
use crate::mcp::types::{MessageType, SecurityLevel, ProtocolVersion};
use crate::error::{SquirrelError as CoreError, Result as CoreResult};
use crate::mcp::error::context::ErrorSeverity;

#[derive(Debug, Error)]
pub enum MCPError {
    #[error("Protocol error: {0}")]
    Protocol(#[from] ProtocolError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    
    #[error("Invalid message: {0}")]
    InvalidMessage(String),
    
    #[error("Security error: {0}")]
    Security(#[from] SecurityError),
    
    #[error("Event error: {0}")]
    Event(String),
    
    #[error("Connection error: {0}")]
    Connection(#[from] ConnectionError),
    
    #[error("State error: {0}")]
    State(String),
    
    #[error("Not initialized: {0}")]
    NotInitialized(String),
    
    #[error("Already initialized: {0}")]
    AlreadyInitialized(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Sync error: {0}")]
    SyncError(String),
    
    #[error("Version mismatch: expected {expected}, received {received}")]
    VersionMismatch {
        expected: ProtocolVersion,
        received: ProtocolVersion,
    },
    
    #[error("Security level too low: required {required:?}, provided {provided:?}")]
    SecurityLevelTooLow {
        required: SecurityLevel,
        provided: SecurityLevel,
    },
    
    #[error("Unknown message type: {0}")]
    UnknownMessageType(String),
    
    #[error("Message validation failed: {0}")]
    ValidationError(String),
    
    #[error("Message routing failed: {0}")]
    RoutingError(String),
    
    #[error("Handler error: {0}")]
    HandlerError(String),
    
    #[error("Timeout error: {operation} exceeded {timeout_ms}ms")]
    Timeout {
        operation: String,
        timeout_ms: u64,
    },
}

impl From<MCPError> for CoreError {
    fn from(err: MCPError) -> Self {
        match err {
            MCPError::Protocol(e) => CoreError::MCP(format!("Protocol error: {e}")),
            MCPError::Io(e) => CoreError::MCP(format!("IO error: {e}")),
            MCPError::SerdeJson(e) => CoreError::MCP(format!("Serialization error: {e}")),
            MCPError::InvalidMessage(e) | MCPError::Event(e) | MCPError::State(e) => CoreError::MCP(e),
            MCPError::Security(e) => CoreError::MCP(format!("Security error: {e}")),
            MCPError::Connection(e) => CoreError::MCP(format!("Connection error: {e}")),
            MCPError::NotInitialized(e) => CoreError::MCP(format!("Not initialized: {e}")),
            MCPError::AlreadyInitialized(e) => CoreError::MCP(format!("Already initialized: {e}")),
            MCPError::StorageError(e) => CoreError::MCP(format!("Storage error: {e}")),
            MCPError::SyncError(e) => CoreError::MCP(format!("Sync error: {e}")),
            MCPError::VersionMismatch { .. } => CoreError::MCP("Version mismatch".to_string()),
            MCPError::SecurityLevelTooLow { .. } => CoreError::MCP("Security level too low".to_string()),
            MCPError::UnknownMessageType(msg) => CoreError::MCP(format!("Unknown message type: {msg}")),
            MCPError::ValidationError(msg) => CoreError::MCP(format!("Message validation failed: {msg}")),
            MCPError::RoutingError(msg) => CoreError::MCP(format!("Message routing failed: {msg}")),
            MCPError::HandlerError(msg) => CoreError::MCP(format!("Handler error: {msg}")),
            MCPError::Timeout { .. } => CoreError::MCP("Timeout".to_string()),
        }
    }
}

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
    #[error("Invalid security level: required {required:?}, provided {provided:?}")]
    InvalidSecurityLevel {
        required: SecurityLevel,
        provided: SecurityLevel,
    },
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
                              ConnectionError::Reset) |
            MCPError::NotInitialized(_) | 
            MCPError::StorageError(_) |
            MCPError::SyncError(_) | 
            MCPError::HandlerError(_) | 
            MCPError::Timeout { .. } => true,
            
            // Default case - all other errors are non-recoverable
            _ => false,
        }
    }

    #[must_use] pub fn severity(&self) -> ErrorSeverity {
        match self {
            // Critical severity errors
            MCPError::Security(_) | 
            MCPError::VersionMismatch { .. } | 
            MCPError::SecurityLevelTooLow { .. } => ErrorSeverity::Critical,
            
            // High severity errors - general connection errors except timeout
            MCPError::Connection(
                ConnectionError::ConnectionFailed(_) |
                ConnectionError::Closed(_) |
                ConnectionError::Reset |
                ConnectionError::Refused |
                ConnectionError::Unreachable |
                ConnectionError::TooManyConnections |
                ConnectionError::LimitReached(_)
            ) => ErrorSeverity::High,
            
            // High severity errors - other types
            MCPError::Protocol(_) | 
            MCPError::StorageError(_) => ErrorSeverity::High,
            
            // Medium severity errors
            MCPError::Connection(ConnectionError::Timeout(_)) |
            MCPError::NotInitialized(_) |
            MCPError::SyncError(_) |
            MCPError::ValidationError(_) |
            MCPError::HandlerError(_) |
            MCPError::Timeout { .. } => ErrorSeverity::Medium,
            
            // All other errors are low severity
            _ => ErrorSeverity::Low,
        }
    }

    #[must_use] pub fn error_code(&self) -> String {
        match self {
            MCPError::Protocol(_) => "MCP-001",
            MCPError::Security(_) => "MCP-002",
            MCPError::Connection(_) => "MCP-003",
            MCPError::VersionMismatch { .. } => "MCP-004",
            MCPError::SecurityLevelTooLow { .. } => "MCP-005",
            MCPError::ValidationError(_) => "MCP-006",
            MCPError::HandlerError(_) => "MCP-007",
            MCPError::Timeout { .. } => "MCP-008",
            MCPError::NotInitialized(_) => "MCP-009",
            MCPError::AlreadyInitialized(_) => "MCP-010",
            MCPError::StorageError(_) => "MCP-011",
            MCPError::SyncError(_) => "MCP-012",
            _ => "MCP-000",
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
        let version_mismatch = MCPError::VersionMismatch {
            expected: ProtocolVersion::new(1, 0),
            received: ProtocolVersion::new(2, 0),
        };
        assert!(!version_mismatch.is_recoverable());
        assert_eq!(version_mismatch.severity(), ErrorSeverity::Critical);

        let timeout = MCPError::Connection(ConnectionError::Timeout(5000));
        assert!(timeout.is_recoverable());
        assert_eq!(timeout.severity(), ErrorSeverity::Medium);
    }
}