// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! WebSocket transport types and enums.

use crate::protocol::types::MCPMessage;

/// WebSocket control message types
///
/// Internal message types used to control the WebSocket connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlMessage {
    /// Shutdown the connection
    Shutdown,
    /// Reconnect to the server
    Reconnect,
    /// Ping the server
    Ping,
    /// Pong response
    Pong,
}

/// Simple state of the WebSocket connection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebSocketState {
    Disconnected,
    Connecting,
    Connected,
    Failed(String),
}

impl WebSocketState {
    /// Check if the state is Connected
    pub fn is_connected(&self) -> bool {
        matches!(self, Self::Connected)
    }
}

/// Commands for the WebSocket socket task
///
/// Commands sent to the WebSocket task to control its behavior.
#[derive(Debug)]
pub enum SocketCommand {
    /// Send a message
    Send(MCPMessage),

    /// Send raw binary data
    SendRaw(Vec<u8>),

    /// Send a ping frame
    Ping,

    /// Close the connection
    Close,
}
