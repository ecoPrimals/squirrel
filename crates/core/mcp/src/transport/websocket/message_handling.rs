// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Message handling for WebSocket transport.
#![allow(dead_code)] // WebSocket transport awaiting activation

use crate::error::{MCPError, Result};
use crate::protocol::types::MCPMessage;
use crate::transport::websocket::types::SocketCommand;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::tungstenite::protocol::Message;
use tracing::{debug, error, warn};

/// Handle received WebSocket message
pub async fn handle_received_message(message: Message) -> Result<Option<MCPMessage>> {
    match message {
        Message::Text(text) => {
            // Parse as JSON
            match serde_json::from_str::<MCPMessage>(&text) {
                Ok(msg) => Ok(Some(msg)),
                Err(e) => {
                    error!("Failed to parse text message: {}", e);
                    Err(MCPError::Serialization(e.to_string()))
                }
            }
        }
        Message::Binary(bin) => {
            // Parse as binary JSON
            match serde_json::from_slice::<MCPMessage>(&bin) {
                Ok(msg) => Ok(Some(msg)),
                Err(e) => {
                    error!("Failed to parse binary message: {}", e);
                    Err(MCPError::Serialization(e.to_string()))
                }
            }
        }
        Message::Ping(_) => {
            debug!("Received ping");
            Ok(None) // Pong sent automatically by tungstenite
        }
        Message::Pong(_) => {
            debug!("Received pong");
            Ok(None)
        }
        Message::Close(_) => {
            debug!("Received close frame");
            Ok(None)
        }
        Message::Frame(_) => {
            warn!("Received unexpected frame type");
            Ok(None)
        }
    }
}

/// Buffer a message for later sending
///
/// Adds a message to the buffer for sending after reconnection.
/// Implements a circular buffer strategy (oldest messages dropped if buffer full).
///
/// # Arguments
///
/// * `message` - The message to buffer
/// * `message_buffer` - The message buffer to add to
///
/// # Returns
///
/// Result indicating if the message was buffered or if buffer is full
pub async fn buffer_message(
    message: MCPMessage,
    message_buffer: Arc<Mutex<Vec<MCPMessage>>>,
) -> Result<()> {
    const MAX_BUFFER_SIZE: usize = 1000; // Limit buffer to prevent memory exhaustion

    let mut buffer = message_buffer.lock().await;

    if buffer.len() >= MAX_BUFFER_SIZE {
        // Drop oldest message (circular buffer strategy)
        buffer.remove(0);
        warn!("Message buffer full, dropped oldest message");
    }

    buffer.push(message);
    debug!("Buffered message ({} in buffer)", buffer.len());

    Ok(())
}

/// Drain buffered messages after reconnection
///
/// Sends all buffered messages that accumulated during disconnection.
///
/// # Arguments
///
/// * `message_buffer` - The message buffer to drain
/// * `ws_sender` - The WebSocket sender channel
///
/// # Returns
///
/// Result indicating success or failure
pub async fn drain_message_buffer(
    message_buffer: Arc<Mutex<Vec<MCPMessage>>>,
    ws_sender: &mpsc::Sender<SocketCommand>,
) -> Result<()> {
    let messages: Vec<MCPMessage> = {
        let mut buffer = message_buffer.lock().await;
        let msgs = buffer.clone();
        buffer.clear();
        msgs
    };

    if messages.is_empty() {
        return Ok(());
    }

    debug!(
        "Draining {} buffered messages after reconnection",
        messages.len()
    );

    for (i, message) in messages.into_iter().enumerate() {
        if ws_sender.send(SocketCommand::Send(message)).await.is_err() {
            warn!("Failed to send buffered message {}: channel closed", i + 1);
            // Continue trying to send remaining messages
        } else {
            debug!("Sent buffered message {}", i + 1);
        }
    }

    Ok(())
}
