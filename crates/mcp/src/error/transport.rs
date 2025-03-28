use std::io;
use std::convert::From;
use std::fmt;
use thiserror::Error;

use super::types::MCPError;

/// Errors that can occur in the transport layer
#[derive(Debug, Error)]
pub enum TransportError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Connection closed: {0}")]
    ConnectionClosed(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("IO error: {0}")]
    IoError(std::io::Error),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Invalid frame: {0}")]
    InvalidFrame(String),

    #[error("Security error: {0}")]
    SecurityError(String),

    #[error("Serialization error: {0}")]
    SerializationError(serde_json::Error),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
}

impl TransportError {
    /// Creates a new TransportError::ConnectionFailed with the given message
    pub fn connection_failed<S: Into<String>>(msg: S) -> Self {
        TransportError::ConnectionFailed(msg.into())
    }

    /// Creates a new TransportError::ConnectionClosed with the given message
    pub fn connection_closed<S: Into<String>>(msg: S) -> Self {
        TransportError::ConnectionClosed(msg.into())
    }

    /// Creates a new TransportError::ProtocolError with the given message
    pub fn protocol_error<S: Into<String>>(msg: S) -> Self {
        TransportError::ProtocolError(msg.into())
    }
    
    /// Creates a new TransportError::InvalidFrame with the given message
    pub fn invalid_frame<S: Into<String>>(msg: S) -> Self {
        TransportError::InvalidFrame(msg.into())
    }
    
    /// Creates a new TransportError::SecurityError with the given message
    pub fn security_error<S: Into<String>>(msg: S) -> Self {
        TransportError::SecurityError(msg.into())
    }
    
    /// Creates a new TransportError::Timeout with the given message
    pub fn timeout<S: Into<String>>(msg: S) -> Self {
        TransportError::Timeout(msg.into())
    }
}

impl From<io::Error> for TransportError {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::ConnectionRefused => TransportError::ConnectionFailed(err.to_string()),
            io::ErrorKind::ConnectionReset => TransportError::ConnectionClosed(err.to_string()),
            io::ErrorKind::ConnectionAborted => TransportError::ConnectionClosed(err.to_string()),
            io::ErrorKind::NotConnected => TransportError::ConnectionClosed(err.to_string()),
            io::ErrorKind::TimedOut => TransportError::Timeout(err.to_string()),
            io::ErrorKind::WouldBlock => TransportError::Timeout(err.to_string()),
            io::ErrorKind::InvalidData => TransportError::InvalidFrame(err.to_string()),
            io::ErrorKind::InvalidInput => TransportError::ProtocolError(err.to_string()),
            io::ErrorKind::UnexpectedEof => TransportError::ConnectionClosed(err.to_string()),
            _ => TransportError::IoError(err),
        }
    }
}

impl From<tokio::sync::mpsc::error::SendError<crate::types::MCPMessage>> for TransportError {
    fn from(err: tokio::sync::mpsc::error::SendError<crate::types::MCPMessage>) -> Self {
        TransportError::ProtocolError(format!("Failed to send message: {}", err))
    }
}

impl From<serde_json::Error> for TransportError {
    fn from(err: serde_json::Error) -> Self {
        TransportError::SerializationError(err)
    }
}

impl From<MCPError> for TransportError {
    fn from(error: MCPError) -> Self {
        match error {
            MCPError::Transport(transport_error) => {
                match transport_error {
                    crate::error::types::TransportError::ConnectionFailed(msg) => 
                        TransportError::connection_failed(msg),
                    crate::error::types::TransportError::InvalidFrame(msg) => 
                        TransportError::invalid_frame(msg),
                    crate::error::types::TransportError::Timeout(msg) => 
                        TransportError::timeout(msg),
                    crate::error::types::TransportError::ProtocolError(msg) => 
                        TransportError::protocol_error(msg),
                    crate::error::types::TransportError::ConnectionClosed(msg) => 
                        TransportError::connection_closed(msg),
                    crate::error::types::TransportError::IoError(msg) => 
                        TransportError::IoError(std::io::Error::new(std::io::ErrorKind::Other, msg)),
                }
            },
            _ => TransportError::protocol_error(format!("Error converted from MCPError: {}", error)),
        }
    }
}

// Implement conversion from String to TransportError for convenience
impl From<String> for TransportError {
    fn from(err: String) -> Self {
        TransportError::ProtocolError(err)
    }
}

// Implement conversion from &str to TransportError for convenience
impl From<&str> for TransportError {
    fn from(err: &str) -> Self {
        TransportError::ProtocolError(err.to_string())
    }
} 