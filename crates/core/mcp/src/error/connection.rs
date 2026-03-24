// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use thiserror::Error;

/// Errors related to MCP connection operations
///
/// This enum represents errors that can occur when establishing or maintaining
/// network connections within the MCP system, including failures, timeouts, and
/// connection limit issues.
#[derive(Debug, Clone, Error)]
pub enum ConnectionError {
    /// Error that occurs when a connection cannot be established
    ///
    /// This can happen due to network issues, incorrect configuration,
    /// or when the remote endpoint is unavailable.
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    /// Error that occurs when a connection operation exceeds the time limit
    ///
    /// This happens when the connection process takes longer than the
    /// specified timeout period in milliseconds.
    #[error("Connection timeout after {0}ms")]
    Timeout(u64),

    /// Error that occurs when a connection is closed unexpectedly
    ///
    /// This can happen due to network issues, remote endpoint closure,
    /// or other connection disruptions.
    #[error("Connection closed: {0}")]
    Closed(String),

    /// Error that occurs when a connection is reset by the peer
    ///
    /// This typically happens when the remote endpoint forcibly
    /// closes the connection.
    #[error("Connection reset")]
    Reset,

    /// Error that occurs when a connection is refused by the remote endpoint
    ///
    /// This typically happens when the remote service is not running,
    /// or is configured to reject the connection.
    #[error("Connection refused")]
    Refused,

    /// Error that occurs when the network is unreachable
    ///
    /// This can happen due to network configuration issues, firewalls,
    /// or physical network disconnection.
    #[error("Network unreachable")]
    Unreachable,

    /// Error that occurs when too many concurrent connections are active
    ///
    /// This can happen when the system reaches its maximum connection capacity
    /// as defined by resource limits or configuration.
    #[error("Too many connections")]
    TooManyConnections,

    /// Error that occurs when a connection limit is reached for a specific reason
    ///
    /// This provides more context about why a connection limit was reached,
    /// such as per-user limits or rate limiting.
    #[error("Connection limit reached: {0}")]
    LimitReached(String),

    /// Error reported by the remote peer
    #[error("Remote connection error: {0}")]
    RemoteError(String),
}
