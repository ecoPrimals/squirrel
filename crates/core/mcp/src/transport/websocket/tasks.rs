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

/// Run the WebSocket reader loop until connection closes or error
async fn run_reader_task(
    mut read: futures_util::stream::SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    read_msg_tx: mpsc::Sender<MCPMessage>,
    read_state: Arc<Mutex<WebSocketState>>,
) {
    while let Some(result) = read.next().await {
        match result {
            Ok(Message::Text(text)) => {
                if let Ok(message) = serde_json::from_str::<MCPMessage>(&text) {
                    if read_msg_tx.send(message).await.is_err() {
                        error!("Failed to forward message to channel");
                        break;
                    }
                } else if let Err(e) = serde_json::from_str::<MCPMessage>(&text) {
                    error!("Failed to parse message: {}", e);
                }
            }
            Ok(Message::Binary(bin)) => {
                if let Ok(message) = serde_json::from_slice::<MCPMessage>(&bin) {
                    if read_msg_tx.send(message).await.is_err() {
                        error!("Failed to forward message to channel");
                        break;
                    }
                } else if let Err(e) = serde_json::from_slice::<MCPMessage>(&bin) {
                    error!("Failed to parse binary message: {}", e);
                }
            }
            Ok(Message::Ping(_) | Message::Pong(_)) => debug!("Received ping/pong"),
            Ok(Message::Close(_)) => {
                info!("WebSocket connection closed by peer.");
                break;
            }
            Ok(Message::Frame(_)) => warn!("Received unexpected WebSocket frame type"),
            Err(e) => {
                error!("Error reading from WebSocket: {}", e);
                break;
            }
        }
    }
    info!("WebSocket reader task finished.");
    let mut current_state = read_state.lock().await;
    if *current_state != WebSocketState::Disconnected {
        *current_state = WebSocketState::Disconnected;
        info!("WebSocket state set to Disconnected by reader task.");
    }
}

/// Run the WebSocket writer loop until channel closes or error
async fn run_writer_task(
    mut write: futures_util::stream::SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    mut socket_rx: mpsc::Receiver<SocketCommand>,
    write_state: Arc<Mutex<WebSocketState>>,
) {
    while let Some(command) = socket_rx.recv().await {
        match command {
            SocketCommand::Send(message) => match serde_json::to_string(&message) {
                Ok(json) => {
                    if write.send(Message::Text(json)).await.is_err() {
                        error!("WebSocket: Failed to send message");
                        break;
                    }
                }
                Err(e) => error!("WebSocket: Failed to serialize message: {}", e),
            },
            SocketCommand::SendRaw(bytes) => {
                if write.send(Message::Binary(bytes)).await.is_err() {
                    error!("WebSocket: Failed to send raw bytes");
                    break;
                }
            }
            SocketCommand::Ping => {
                if write.send(Message::Ping(vec![])).await.is_err() {
                    warn!("WebSocket: Failed to send ping");
                } else {
                    trace!("WebSocket: Sent ping frame");
                }
            }
            SocketCommand::Close => {
                info!("WebSocket writer task received Close command.");
                if let Err(e) = write.close().await {
                    error!("Error closing WebSocket: {}", e);
                }
                break;
            }
        }
    }
    info!("WebSocket writer task finished.");
    let mut current_state = write_state.lock().await;
    if *current_state != WebSocketState::Disconnected {
        *current_state = WebSocketState::Disconnected;
        info!("WebSocket state set to Disconnected by writer task.");
    }
}

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
    socket_rx: mpsc::Receiver<SocketCommand>,
    state_clone: Arc<Mutex<WebSocketState>>,
) -> Result<()> {
    let (write, read) = socket.split();
    let read_state = state_clone.clone();
    tokio::spawn(run_reader_task(read, msg_tx, read_state));
    tokio::spawn(run_writer_task(write, socket_rx, state_clone));
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
