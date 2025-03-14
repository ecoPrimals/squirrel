use std::fmt;
use thiserror::Error;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use crate::mcp::protocol::{MessageType, SecurityLevel};
use std::io;

#[derive(Error, Debug)]
pub enum MCPError {
    #[error("Port error: {kind}")]
    Port {
        kind: PortErrorKind,
        context: ErrorContext,
        port: u16,
    },
    #[error("Security error: {0}")]
    SecurityError(SecurityError),
    #[error("Connection error: {0}")]
    ConnectionError(ConnectionError),
    #[error("Protocol error: {0}")]
    ProtocolError(ProtocolError),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Message error: {kind}")]
    Message {
        kind: MessageErrorKind,
        context: ErrorContext,
    },
    #[error("Tool error: {kind}")]
    Tool {
        kind: ToolErrorKind,
        context: ErrorContext,
    },
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),
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

#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    #[error("Decryption error: {0}")]
    DecryptionError(String),
    #[error("Invalid token: {0}")]
    InvalidToken(String),
}

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("Connection timeout")]
    Timeout,
    #[error("Connection closed")]
    Closed,
    #[error("Connection refused")]
    Refused,
    #[error("Connection reset")]
    Reset,
    #[error("Connection error: {0}")]
    Other(String),
}

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Invalid message format: {0}")]
    InvalidFormat(String),
    #[error("Protocol error: {0}")]
    Protocol(String),
    #[error("Security error: {0}")]
    Security(String),
    #[error("State error: {0}")]
    State(String),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageErrorKind {
    InvalidPayload,
    InvalidMetadata,
    DeserializationFailed,
    ValidationFailed,
    Timeout,
    UnexpectedResponse,
}

impl fmt::Display for MessageErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageErrorKind::InvalidPayload => write!(f, "Invalid payload"),
            MessageErrorKind::InvalidMetadata => write!(f, "Invalid metadata"),
            MessageErrorKind::DeserializationFailed => write!(f, "Deserialization failed"),
            MessageErrorKind::ValidationFailed => write!(f, "Validation failed"),
            MessageErrorKind::Timeout => write!(f, "Timeout"),
            MessageErrorKind::UnexpectedResponse => write!(f, "Unexpected response"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolErrorKind {
    NotFound,
    InitializationFailed,
    ExecutionFailed,
    ValidationFailed,
    ResourceExhausted,
    Timeout,
}

impl fmt::Display for ToolErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolErrorKind::NotFound => write!(f, "Tool not found"),
            ToolErrorKind::InitializationFailed => write!(f, "Tool initialization failed"),
            ToolErrorKind::ExecutionFailed => write!(f, "Tool execution failed"),
            ToolErrorKind::ValidationFailed => write!(f, "Tool validation failed"),
            ToolErrorKind::ResourceExhausted => write!(f, "Resource exhausted"),
            ToolErrorKind::Timeout => write!(f, "Timeout"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub timestamp: DateTime<Utc>,
    pub operation: String,
    pub component: String,
    pub message_type: Option<MessageType>,
    pub details: serde_json::Value,
    pub severity: ErrorSeverity,
    pub is_recoverable: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Critical,
    Error,
    Warning,
    Info,
}

impl ErrorContext {
    pub fn new(operation: impl Into<String>, component: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            operation: operation.into(),
            component: component.into(),
            message_type: None,
            details: serde_json::Value::Null,
            severity: ErrorSeverity::Info,
            is_recoverable: true,
        }
    }

    pub fn with_message_type(mut self, message_type: MessageType) -> Self {
        self.message_type = Some(message_type);
        self
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = details;
        self
    }

    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn not_recoverable(mut self) -> Self {
        self.is_recoverable = false;
        self
    }
}

impl MCPError {
    pub fn is_recoverable(&self) -> bool {
        match self {
            MCPError::Port { .. } => false,
            MCPError::SecurityError(_) => false,
            MCPError::ConnectionError(_) => false,
            MCPError::ProtocolError(_) => false,
            MCPError::InternalError(_) => false,
            MCPError::Message { context, .. } => context.is_recoverable,
            MCPError::Tool { context, .. } => context.is_recoverable,
            MCPError::Io(_) => false,
            MCPError::SerdeJson(_) => false,
        }
    }

    pub fn severity(&self) -> ErrorSeverity {
        match self {
            MCPError::Port { .. } => ErrorSeverity::Error,
            MCPError::SecurityError(_) => ErrorSeverity::Error,
            MCPError::ConnectionError(_) => ErrorSeverity::Error,
            MCPError::ProtocolError(_) => ErrorSeverity::Error,
            MCPError::InternalError(_) => ErrorSeverity::Error,
            MCPError::Message { context, .. } => context.severity,
            MCPError::Tool { context, .. } => context.severity,
            MCPError::Io(_) => ErrorSeverity::Error,
            MCPError::SerdeJson(_) => ErrorSeverity::Error,
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

    pub async fn handle_error<F, T>(&self, operation: F) -> Result<T, MCPError>
    where
        F: Fn() -> Result<T, MCPError> + Send + Sync,
    {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < self.max_retries {
            match operation() {
                Ok(result) => return Ok(result),
                Err(e) if e.is_recoverable() => {
                    attempts += 1;
                    last_error = Some(e);
                    tokio::time::sleep(self.retry_delay).await;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }

        Err(last_error.unwrap())
    }
}