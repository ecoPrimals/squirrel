use std::fmt;
use thiserror::Error;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use serde_json::Map;
use crate::mcp::types::{MessageType, SecurityLevel};
use std::io;
use crate::core::error::{CoreError, Result};
use crate::mcp::error::context::ErrorSeverity;

#[derive(Debug)]
pub enum MCPError {
    Protocol(ProtocolError),
    Io(std::io::Error),
    SerdeJson(serde_json::Error),
    InvalidMessage(String),
    Security(SecurityError),
    Event(String),
    Connection(ConnectionError),
    State(String),
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
}

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Connection timeout: {0}")]
    Timeout(String),
    #[error("Connection closed: {0}")]
    Closed(String),
}

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Invalid protocol version: {0}")]
    InvalidVersion(String),
    #[error("Invalid protocol state: {0}")]
    InvalidState(String),
    #[error("Invalid message format: {0}")]
    InvalidFormat(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    timestamp: DateTime<Utc>,
    operation: String,
    component: String,
    message_type: Option<MessageType>,
    details: Map<String, serde_json::Value>,
    severity: crate::mcp::error::context::ErrorSeverity,
    is_recoverable: bool,
}

impl ErrorContext {
    pub fn new(operation: impl Into<String>, component: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            operation: operation.into(),
            component: component.into(),
            message_type: None,
            details: Map::new(),
            severity: crate::mcp::error::context::ErrorSeverity::Low,
            is_recoverable: true,
        }
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    pub fn operation(&self) -> &str {
        &self.operation
    }

    pub fn component(&self) -> &str {
        &self.component
    }

    pub fn message_type(&self) -> Option<&MessageType> {
        self.message_type.as_ref()
    }

    pub fn details(&self) -> &Map<String, serde_json::Value> {
        &self.details
    }

    pub fn severity(&self) -> crate::mcp::error::context::ErrorSeverity {
        self.severity
    }

    pub fn is_recoverable(&self) -> bool {
        self.is_recoverable
    }

    pub fn set_message_type(&mut self, message_type: MessageType) {
        self.message_type = Some(message_type);
    }

    pub fn set_details(&mut self, details: Map<String, serde_json::Value>) {
        self.details = details;
    }

    pub fn set_severity(&mut self, severity: crate::mcp::error::context::ErrorSeverity) {
        self.severity = severity;
    }

    pub fn set_recoverable(&mut self, recoverable: bool) {
        self.is_recoverable = recoverable;
    }
}

impl MCPError {
    pub fn is_recoverable(&self) -> bool {
        match self {
            MCPError::Protocol(_) => false,
            MCPError::Io(_) => false,
            MCPError::SerdeJson(_) => false,
            MCPError::InvalidMessage(_) => false,
            MCPError::Security(_) => false,
            MCPError::Event(_) => false,
            MCPError::Connection(_) => false,
            MCPError::State(_) => false,
        }
    }

    pub fn severity(&self) -> ErrorSeverity {
        match self {
            MCPError::Protocol(_) => ErrorSeverity::High,
            MCPError::Io(_) => ErrorSeverity::High,
            MCPError::SerdeJson(_) => ErrorSeverity::Medium,
            MCPError::InvalidMessage(_) => ErrorSeverity::Medium,
            MCPError::Security(_) => ErrorSeverity::Critical,
            MCPError::Event(_) => ErrorSeverity::Low,
            MCPError::Connection(_) => ErrorSeverity::High,
            MCPError::State(_) => ErrorSeverity::High,
        }
    }
}

#[derive(Debug)]
pub struct ErrorHandler {
    max_retries: u32,
    retry_delay: std::time::Duration,
}

impl ErrorHandler {
    pub fn new(max_retries: u32, retry_delay: std::time::Duration) -> Self {
        Self {
            max_retries,
            retry_delay,
        }
    }

    pub async fn handle_error<F, T>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Result<T> + Send + Sync,
    {
        let mut retries = 0;
        loop {
            match operation() {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if !error.is_recoverable() || retries >= self.max_retries {
                        return Err(error);
                    }
                    retries += 1;
                    tokio::time::sleep(self.retry_delay).await;
                }
            }
        }
    }
}