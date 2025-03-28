use std::io;
use std::convert::From;
use thiserror::Error;

use super::types::MCPError;

/// Errors that can occur in the transport layer
#[derive(Debug, Error)]
pub enum TransportError {
    /// Error when a connection could not be established with the remote endpoint
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    /// Error when an existing connection was closed, either by the remote endpoint or locally
    #[error("Connection closed: {0}")]
    ConnectionClosed(String),

    /// Error when an operation timed out waiting for a response or connection
    #[error("Timeout: {0}")]
    Timeout(String),

    /// Error originating from underlying I/O operations
    #[error("IO error: {0}")]
    IoError(std::io::Error),

    /// Error related to the communication protocol, such as invalid message format or sequence
    #[error("Protocol error: {0}")]
    ProtocolError(String),

    /// Error when a frame received or being sent is invalid or malformed
    #[error("Invalid frame: {0}")]
    InvalidFrame(String),

    /// Error related to security mechanisms such as encryption, authentication, or authorization
    #[error("Security error: {0}")]
    SecurityError(String),

    /// Error during serialization or deserialization of messages
    #[error("Serialization error: {0}")]
    SerializationError(serde_json::Error),

    /// Error due to invalid or incompatible configuration parameters
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Error when an operation is not supported by the current transport
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
}

impl TransportError {
    /// Creates a new `TransportError::ConnectionFailed` with the given message
    pub fn connection_failed<S: Into<String>>(msg: S) -> Self {
        Self::ConnectionFailed(msg.into())
    }

    /// Creates a new `TransportError::ConnectionClosed` with the given message
    pub fn connection_closed<S: Into<String>>(msg: S) -> Self {
        Self::ConnectionClosed(msg.into())
    }

    /// Creates a new `TransportError::ProtocolError` with the given message
    pub fn protocol_error<S: Into<String>>(msg: S) -> Self {
        Self::ProtocolError(msg.into())
    }
    
    /// Creates a new `TransportError::InvalidFrame` with the given message
    pub fn invalid_frame<S: Into<String>>(msg: S) -> Self {
        Self::InvalidFrame(msg.into())
    }
    
    /// Creates a new `TransportError::SecurityError` with the given message
    pub fn security_error<S: Into<String>>(msg: S) -> Self {
        Self::SecurityError(msg.into())
    }
    
    /// Creates a new `TransportError::Timeout` with the given message
    pub fn timeout<S: Into<String>>(msg: S) -> Self {
        Self::Timeout(msg.into())
    }
}

impl From<io::Error> for TransportError {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::ConnectionRefused => Self::ConnectionFailed(err.to_string()),
            io::ErrorKind::ConnectionReset => Self::ConnectionClosed(err.to_string()),
            io::ErrorKind::ConnectionAborted => Self::ConnectionClosed(err.to_string()),
            io::ErrorKind::NotConnected => Self::ConnectionClosed(err.to_string()),
            io::ErrorKind::TimedOut => Self::Timeout(err.to_string()),
            io::ErrorKind::WouldBlock => Self::Timeout(err.to_string()),
            io::ErrorKind::InvalidData => Self::InvalidFrame(err.to_string()),
            io::ErrorKind::InvalidInput => Self::ProtocolError(err.to_string()),
            io::ErrorKind::UnexpectedEof => Self::ConnectionClosed(err.to_string()),
            _ => Self::IoError(err),
        }
    }
}

impl From<tokio::sync::mpsc::error::SendError<crate::types::MCPMessage>> for TransportError {
    fn from(err: tokio::sync::mpsc::error::SendError<crate::types::MCPMessage>) -> Self {
        Self::ProtocolError(format!("Failed to send message: {err}"))
    }
}

impl From<serde_json::Error> for TransportError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError(err)
    }
}

impl From<MCPError> for TransportError {
    fn from(error: MCPError) -> Self {
        match error {
            MCPError::Transport(transport_error) => {
                match transport_error {
                    crate::error::types::TransportError::ConnectionFailed(msg) => 
                        Self::connection_failed(msg),
                    crate::error::types::TransportError::InvalidFrame(msg) => 
                        Self::invalid_frame(msg),
                    crate::error::types::TransportError::Timeout(msg) => 
                        Self::timeout(msg),
                    crate::error::types::TransportError::ProtocolError(msg) => 
                        Self::protocol_error(msg),
                    crate::error::types::TransportError::ConnectionClosed(msg) => 
                        Self::connection_closed(msg),
                    crate::error::types::TransportError::IoError(msg) => 
                        Self::IoError(std::io::Error::new(std::io::ErrorKind::Other, msg)),
                }
            },
            _ => Self::protocol_error(format!("Error converted from MCPError: {error}")),
        }
    }
}

// Implement conversion from String to TransportError for convenience
impl From<String> for TransportError {
    fn from(err: String) -> Self {
        Self::ProtocolError(err)
    }
}

// Implement conversion from &str to TransportError for convenience
impl From<&str> for TransportError {
    fn from(err: &str) -> Self {
        Self::ProtocolError(err.to_string())
    }
} 