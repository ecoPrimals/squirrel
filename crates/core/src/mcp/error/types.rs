use std::fmt;
use thiserror::Error;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use serde_json::Map;
use crate::mcp::types::{MessageType, SecurityLevel, ProtocolVersion};
use std::io;
use crate::core::error::{CoreError, Result};
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

impl std::error::Error for MCPError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MCPError::Protocol(e) => Some(e),
            MCPError::Io(e) => Some(e),
            MCPError::SerdeJson(e) => Some(e),
            MCPError::Security(e) => Some(e),
            MCPError::Connection(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for MCPError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MCPError::Protocol(e) => write!(f, "Protocol error: {}", e),
            MCPError::Io(e) => write!(f, "IO error: {}", e),
            MCPError::SerdeJson(e) => write!(f, "Serialization error: {}", e),
            MCPError::InvalidMessage(msg) => write!(f, "Invalid message: {}", msg),
            MCPError::Security(e) => write!(f, "Security error: {}", e),
            MCPError::Event(msg) => write!(f, "Event error: {}", msg),
            MCPError::Connection(e) => write!(f, "Connection error: {}", e),
            MCPError::State(msg) => write!(f, "State error: {}", msg),
            MCPError::VersionMismatch { .. } => write!(f, "Version mismatch"),
            MCPError::SecurityLevelTooLow { .. } => write!(f, "Security level too low"),
            MCPError::UnknownMessageType(msg) => write!(f, "Unknown message type: {}", msg),
            MCPError::ValidationError(msg) => write!(f, "Message validation failed: {}", msg),
            MCPError::RoutingError(msg) => write!(f, "Message routing failed: {}", msg),
            MCPError::HandlerError(msg) => write!(f, "Handler error: {}", msg),
            MCPError::Timeout { .. } => write!(f, "Timeout"),
        }
    }
}

impl From<MCPError> for CoreError {
    fn from(err: MCPError) -> Self {
        match err {
            MCPError::Protocol(e) => CoreError::Protocol(e.to_string()),
            MCPError::Io(e) => CoreError::Io(e),
            MCPError::SerdeJson(e) => CoreError::Other(format!("Serialization error: {}", e)),
            MCPError::InvalidMessage(e) => CoreError::Other(e),
            MCPError::Security(e) => CoreError::Security(e.to_string()),
            MCPError::Event(e) => CoreError::Other(e),
            MCPError::Connection(e) => CoreError::Context(e.to_string()),
            MCPError::State(e) => CoreError::Protocol(e),
            MCPError::VersionMismatch { .. } => CoreError::Protocol("Version mismatch".to_string()),
            MCPError::SecurityLevelTooLow { .. } => CoreError::Protocol("Security level too low".to_string()),
            MCPError::UnknownMessageType(msg) => CoreError::Other(format!("Unknown message type: {}", msg)),
            MCPError::ValidationError(msg) => CoreError::Other(format!("Message validation failed: {}", msg)),
            MCPError::RoutingError(msg) => CoreError::Other(format!("Message routing failed: {}", msg)),
            MCPError::HandlerError(msg) => CoreError::Other(format!("Handler error: {}", msg)),
            MCPError::Timeout { .. } => CoreError::Other("Timeout".to_string()),
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

    pub fn with_message_type(mut self, message_type: MessageType) -> Self {
        self.message_type = Some(message_type);
        self
    }

    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_details(mut self, details: Map<String, serde_json::Value>) -> Self {
        self.details = details;
        self
    }

    pub fn with_error_code(mut self, code: impl Into<String>) -> Self {
        self.error_code = code.into();
        self
    }

    pub fn with_source_location(mut self, location: impl Into<String>) -> Self {
        self.source_location = Some(location.into());
        self
    }

    pub fn increment_retry_count(&mut self) {
        self.retry_count += 1;
    }
}

impl MCPError {
    pub fn is_recoverable(&self) -> bool {
        match self {
            MCPError::Protocol(ProtocolError::InvalidVersion(_)) => false,
            MCPError::Protocol(ProtocolError::ConfigurationError(_)) => false,
            MCPError::Security(SecurityError::AuthenticationFailed(_)) => true,
            MCPError::Security(SecurityError::TokenExpired) => true,
            MCPError::Connection(ConnectionError::Timeout(_)) => true,
            MCPError::Connection(ConnectionError::Reset) => true,
            MCPError::VersionMismatch { .. } => false,
            MCPError::SecurityLevelTooLow { .. } => false,
            MCPError::ValidationError(_) => false,
            MCPError::HandlerError(_) => true,
            MCPError::Timeout { .. } => true,
            _ => false,
        }
    }

    pub fn severity(&self) -> ErrorSeverity {
        match self {
            MCPError::Protocol(_) => ErrorSeverity::High,
            MCPError::Security(_) => ErrorSeverity::Critical,
            MCPError::Connection(ConnectionError::Timeout(_)) => ErrorSeverity::Medium,
            MCPError::Connection(_) => ErrorSeverity::High,
            MCPError::VersionMismatch { .. } => ErrorSeverity::Critical,
            MCPError::SecurityLevelTooLow { .. } => ErrorSeverity::Critical,
            MCPError::ValidationError(_) => ErrorSeverity::Medium,
            MCPError::HandlerError(_) => ErrorSeverity::Medium,
            MCPError::Timeout { .. } => ErrorSeverity::Medium,
            _ => ErrorSeverity::Low,
        }
    }

    pub fn error_code(&self) -> String {
        match self {
            MCPError::Protocol(_) => "MCP-001",
            MCPError::Security(_) => "MCP-002",
            MCPError::Connection(_) => "MCP-003",
            MCPError::VersionMismatch { .. } => "MCP-004",
            MCPError::SecurityLevelTooLow { .. } => "MCP-005",
            MCPError::ValidationError(_) => "MCP-006",
            MCPError::HandlerError(_) => "MCP-007",
            MCPError::Timeout { .. } => "MCP-008",
            _ => "MCP-000",
        }.to_string()
    }
}

#[derive(Debug)]
pub struct ErrorHandler {
    max_retries: u32,
    retry_delay: std::time::Duration,
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

    pub async fn handle_error<F, T>(&mut self, operation: F) -> Result<T>
    where
        F: Fn() -> Result<T> + Send + Sync,
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

    pub fn error_context(&self) -> &ErrorContext {
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