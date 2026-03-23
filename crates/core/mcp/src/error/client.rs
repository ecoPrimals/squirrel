// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use std::error::Error;
use std::fmt;

/// Errors that can occur in the MCP client
#[derive(Debug, Clone)]
pub enum ClientError {
    /// Client is not connected to the server
    NotConnected(String),

    /// Request timed out
    Timeout(String),

    /// Response channel was closed
    ResponseChannelClosed(String),

    /// Failed to serialize or deserialize a message
    SerializationError(String),

    /// Failed to connect to server
    ConnectionFailed(String),

    /// Invalid message received
    InvalidMessage(String),

    /// Client is already connected
    AlreadyConnected(String),

    /// Error received from remote endpoint
    RemoteError(String),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotConnected(msg) => write!(f, "Client not connected: {msg}"),
            Self::Timeout(msg) => write!(f, "Timeout: {msg}"),
            Self::ResponseChannelClosed(msg) => write!(f, "Response channel closed: {msg}"),
            Self::SerializationError(msg) => write!(f, "Serialization error: {msg}"),
            Self::ConnectionFailed(msg) => write!(f, "Connection failed: {msg}"),
            Self::InvalidMessage(msg) => write!(f, "Invalid message: {msg}"),
            Self::AlreadyConnected(msg) => write!(f, "Already connected: {msg}"),
            Self::RemoteError(msg) => write!(f, "Remote error: {msg}"),
        }
    }
}

impl Error for ClientError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_error_display_all_variants() {
        let cases = vec![
            (
                ClientError::NotConnected("a".into()),
                "Client not connected: a",
            ),
            (ClientError::Timeout("b".into()), "Timeout: b"),
            (
                ClientError::ResponseChannelClosed("c".into()),
                "Response channel closed: c",
            ),
            (
                ClientError::SerializationError("d".into()),
                "Serialization error: d",
            ),
            (
                ClientError::ConnectionFailed("e".into()),
                "Connection failed: e",
            ),
            (
                ClientError::InvalidMessage("f".into()),
                "Invalid message: f",
            ),
            (
                ClientError::AlreadyConnected("g".into()),
                "Already connected: g",
            ),
            (ClientError::RemoteError("h".into()), "Remote error: h"),
        ];
        for (err, want) in cases {
            assert_eq!(err.to_string(), want);
            assert!(std::error::Error::source(&err).is_none());
        }
    }
}
