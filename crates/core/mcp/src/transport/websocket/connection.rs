// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Connection handling for WebSocket transport.

use crate::error::{MCPError, Result, TransportError};
use crate::transport::websocket::config::WebSocketConfig;
use crate::transport::websocket::types::WebSocketState;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};
use tracing::{error, info, warn};

/// Establish WebSocket connection
///
/// Connects to the WebSocket server and sets up the connection infrastructure.
///
/// # Arguments
///
/// * `config` - WebSocket configuration
/// * `connection_state` - Shared connection state
/// * `peer_addr` - Shared peer address storage
/// * `local_addr` - Shared local address storage
/// * `ws_sender` - WebSocket sender channel (will be updated)
/// * `reader_rx` - Reader receiver channel (will be updated)
///
/// # Returns
///
/// Result containing the established socket or an error
pub async fn establish_connection(
    config: &WebSocketConfig,
    connection_state: Arc<Mutex<WebSocketState>>,
    peer_addr: Arc<Mutex<Option<std::net::SocketAddr>>>,
    local_addr: Arc<Mutex<Option<std::net::SocketAddr>>>,
) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
    {
        let mut state = connection_state.lock().await;
        if *state != WebSocketState::Disconnected {
            warn!(
                "WebSocket connect called while not disconnected ({:?})",
                *state
            );
            return Err(MCPError::Transport(TransportError::ConnectionFailed(
                "Already connected or connecting".to_string(),
            )));
        }
        *state = WebSocketState::Connecting;
    }

    info!("Connecting to WebSocket URL: {}", config.url);
    let connection_result = connect_async(&config.url).await;

    let (socket, response) = match connection_result {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to connect to {}: {}", config.url, e);
            *connection_state.lock().await = WebSocketState::Failed(e.to_string());
            return Err(MCPError::Transport(TransportError::ConnectionError(
                format!("Failed to connect to {}: {}", config.url, e),
            )));
        }
    };
    info!(
        "WebSocket connection established. Response: {:?}",
        response.status()
    );

    let (peer, local) = if let MaybeTlsStream::Plain(tcp) = socket.get_ref() {
        // For plain TCP connections
        (tcp.peer_addr().ok(), tcp.local_addr().ok())
    } else {
        // For all TLS connections (regardless of implementation)
        warn!("Could not determine peer/local address from TLS WebSocket stream.");
        (None, None)
    };

    *peer_addr.lock().await = peer;
    *local_addr.lock().await = local;

    Ok(socket)
}
