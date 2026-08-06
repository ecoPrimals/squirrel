// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use thiserror::Error;

/// Errors that can occur in the MCP client
#[derive(Debug, Clone, Error)]
pub enum ClientError {
    /// Client is not connected to the server
    #[error("Client not connected: {0}")]
    NotConnected(String),

    /// Request timed out
    #[error("Timeout: {0}")]
    Timeout(String),

    /// Response channel was closed
    #[error("Response channel closed: {0}")]
    ResponseChannelClosed(String),

    /// Failed to serialize or deserialize a message
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Failed to connect to server
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    /// Invalid message received
    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    /// Client is already connected
    #[error("Already connected: {0}")]
    AlreadyConnected(String),

    /// Error received from remote endpoint
    #[error("Remote error: {0}")]
    RemoteError(String),
}

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
