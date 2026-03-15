// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Background tasks for WebSocket transport.

use crate::error::Result;
use crate::protocol::types::MCPMessage;
use crate::transport::websocket::types::{SocketCommand, WebSocketState};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::{Mutex, mpsc};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, trace, warn};

/// Start the WebSocket reader and writer tasks
///
/// Creates and starts the background tasks for handling the WebSocket connection.
/// This includes a reader task for incoming messages and a writer task for outgoing messages.
///
/// # Arguments
///
/// * `socket` - The established WebSocket connection
/// * `msg_tx` - Sender for forwarding received messages
/// * `socket_rx` - Receiver for commands to the socket task
/// * `state_clone` - Cloned state to update from tasks
///
/// # Returns
///
/// Result indicating success or error
pub async fn start_websocket_task(
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    msg_tx: mpsc::Sender<MCPMessage>,
    mut socket_rx: mpsc::Receiver<SocketCommand>,
    state_clone: Arc<Mutex<WebSocketState>>,
) -> Result<()> {
    let (mut write, mut read) = socket.split();

    // Clone for the reader task
    let read_state = state_clone.clone();
    let read_msg_tx = msg_tx;

    // Start reader task
    tokio::spawn(async move {
        while let Some(result) = read.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    // Parse as JSON
                    match serde_json::from_str::<MCPMessage>(&text) {
                        Ok(message) => {
                            if read_msg_tx.send(message).await.is_err() {
                                error!("Failed to forward message to channel");
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse message: {}", e);
                            continue;
                        }
                    }
                }
                Ok(Message::Binary(bin)) => {
                    // Parse as binary JSON
                    match serde_json::from_slice::<MCPMessage>(&bin) {
                        Ok(message) => {
                            if read_msg_tx.send(message).await.is_err() {
                                error!("Failed to forward message to channel");
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse binary message: {}", e);
                            continue;
                        }
                    }
                }
                Ok(Message::Ping(_) | Message::Pong(_)) => {
                    // Handle ping/pong, maybe log or ignore
                    debug!("Received ping/pong");
                }
                Ok(Message::Close(_)) => {
                    // Connection closed by the server
                    info!("WebSocket connection closed by peer.");
                    break;
                }
                Ok(Message::Frame(_)) => {
                    // Handle unexpected frame types if necessary
                    warn!("Received unexpected WebSocket frame type");
                }
                Err(e) => {
                    // Error reading from socket
                    error!("Error reading from WebSocket: {}", e);
                    break;
                }
            }
        }

        // Update state to disconnected
        info!("WebSocket reader task finished.");
        let mut current_state = read_state.lock().await;
        if *current_state != WebSocketState::Disconnected {
            *current_state = WebSocketState::Disconnected;
            info!("WebSocket state set to Disconnected by reader task.");
        }
    });

    // Start writer task
    let write_state = state_clone;
    tokio::spawn(async move {
        while let Some(command) = socket_rx.recv().await {
            match command {
                SocketCommand::Send(message) => {
                    // Serialize to JSON
                    let json = match serde_json::to_string(&message) {
                        Ok(j) => j,
                        Err(e) => {
                            error!("WebSocket: Failed to serialize message: {}", e);
                            continue;
                        }
                    };

                    // Send as text message
                    if let Err(e) = write.send(Message::Text(json)).await {
                        error!("WebSocket: Failed to send message: {}", e);
                        break;
                    }
                }
                SocketCommand::SendRaw(bytes) => {
                    // Send as binary message
                    if let Err(e) = write.send(Message::Binary(bytes)).await {
                        error!("WebSocket: Failed to send raw bytes: {}", e);
                        break;
                    }
                }
                SocketCommand::Ping => {
                    // Send ping frame
                    if let Err(e) = write.send(Message::Ping(vec![])).await {
                        warn!("WebSocket: Failed to send ping: {}", e);
                        // Don't break on ping failure, connection might recover
                    } else {
                        trace!("WebSocket: Sent ping frame");
                    }
                }
                SocketCommand::Close => {
                    // Close the connection gracefully
                    info!("WebSocket writer task received Close command.");
                    if let Err(e) = write.close().await {
                        error!("Error closing WebSocket: {}", e);
                    }
                    break;
                }
            }
        }

        // Update state to disconnected
        info!("WebSocket writer task finished.");
        let mut current_state = write_state.lock().await;
        if *current_state != WebSocketState::Disconnected {
            *current_state = WebSocketState::Disconnected;
            info!("WebSocket state set to Disconnected by writer task.");
        }
    });

    Ok(())
}

/// Start keepalive ping task
///
/// Starts a background task that sends periodic ping frames to keep
/// the connection alive and detect disconnections early.
///
/// # Arguments
///
/// * `sender` - The WebSocket sender channel
/// * `state` - The connection state
/// * `ping_interval_secs` - Interval between pings in seconds
pub fn start_keepalive_task(
    sender: Option<mpsc::Sender<SocketCommand>>,
    state: Arc<Mutex<WebSocketState>>,
    ping_interval_secs: u64,
) {
    let interval = Duration::from_secs(ping_interval_secs);

    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);

        loop {
            ticker.tick().await;

            // Check if still connected
            {
                let current_state = state.lock().await;
                if !current_state.is_connected() {
                    debug!("Keepalive task stopping - not connected");
                    break;
                }
            }

            // Send ping
            if let Some(ref tx) = sender {
                if let Err(e) = tx.send(SocketCommand::Ping).await {
                    warn!("Keepalive ping failed: {}", e);
                    break;
                }
                trace!("Sent keepalive ping");
            } else {
                break;
            }
        }

        info!("Keepalive task terminated");
    });
}
