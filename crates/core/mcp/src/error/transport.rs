// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use std::convert::From;
use std::io;
use thiserror::Error;

use super::types::MCPError;
use crate::protocol::types::MCPMessage;

/// Errors that can occur in the transport layer
#[derive(Debug, Error, Clone)]
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
    #[error("I/O error: {0}")]
    IoError(String),

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
    SerializationError(String),

    /// Error due to invalid or incompatible configuration parameters
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Error when an operation is not supported by the current transport
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    /// Error when a send operation fails
    #[error("Send error: {0}")]
    SendError(String),

    /// Error reported by the remote peer
    #[error("Remote transport error: {0}")]
    RemoteError(String),

    /// Error when a connection error occurs
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// Error when reading from a transport fails
    #[error("Read error: {0}")]
    ReadError(String),

    /// Error when writing to a transport fails
    #[error("Write error: {0}")]
    WriteError(String),

    /// Error when framing messages fails
    #[error("Framing error: {0}")]
    FramingError(String),
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

    /// Creates a new `TransportError::ReadError` with the given message
    pub fn read_error<S: Into<String>>(msg: S) -> Self {
        Self::ReadError(msg.into())
    }

    /// Creates a new `TransportError::WriteError` with the given message
    pub fn write_error<S: Into<String>>(msg: S) -> Self {
        Self::WriteError(msg.into())
    }

    /// Creates a new `TransportError::FramingError` with the given message
    pub fn framing_error<S: Into<String>>(msg: S) -> Self {
        Self::FramingError(msg.into())
    }
}

impl From<io::Error> for TransportError {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::ConnectionRefused => Self::ConnectionFailed(err.to_string()),
            io::ErrorKind::ConnectionReset
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::NotConnected
            | io::ErrorKind::UnexpectedEof => Self::ConnectionClosed(err.to_string()),
            io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock => Self::Timeout(err.to_string()),
            io::ErrorKind::InvalidData => Self::InvalidFrame(err.to_string()),
            io::ErrorKind::InvalidInput => Self::ProtocolError(err.to_string()),
            _ => Self::IoError(err.to_string()),
        }
    }
}

impl From<tokio::sync::mpsc::error::SendError<MCPMessage>> for TransportError {
    fn from(err: tokio::sync::mpsc::error::SendError<MCPMessage>) -> Self {
        Self::SendError(format!("Failed to send message: {err}"))
    }
}

impl From<serde_json::Error> for TransportError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError(err.to_string())
    }
}

impl From<MCPError> for TransportError {
    fn from(error: MCPError) -> Self {
        match error {
            // This is a recursive case that should not happen in practice once
            // all code is migrated, but we include it for safety
            MCPError::Transport(transport_error) => transport_error,
            _ => Self::protocol_error(format!("Error converted from MCPError: {error}")),
        }
    }
}

/// Implement `From<String>` for `TransportError` to simplify error handling
impl From<String> for TransportError {
    fn from(msg: String) -> Self {
        Self::ConnectionError(msg)
    }
}

impl From<&str> for TransportError {
    fn from(msg: &str) -> Self {
        Self::ConnectionError(msg.to_string())
    }
}
