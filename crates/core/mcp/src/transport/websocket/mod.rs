// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! WebSocket transport implementation for MCP.
#![allow(dead_code)] // WebSocket transport awaiting activation

// This module provides a WebSocket-based transport implementation
// for Machine Context Protocol (MCP) communication. It supports
// bidirectional message passing over WebSocket connections.

mod config;
mod connection;
mod message_handling;
mod tasks;
mod types;

#[cfg(test)]
mod tests;

use crate::error::{MCPError, Result, TransportError};
use crate::protocol::types::MCPMessage;
use crate::transport::types::TransportMetadata;
use crate::transport::Transport;
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tracing::{error, info, warn};
use uuid::Uuid;

// Re-export public types
pub use config::WebSocketConfig;
pub use types::{ControlMessage, SocketCommand, WebSocketState};

use connection::establish_connection;
use message_handling::{buffer_message, drain_message_buffer};
use tasks::{start_keepalive_task, start_websocket_task};

/// WebSocket transport for MCP communication
///
/// This implementation provides WebSocket-based transport for MCP messages.
/// It handles connection establishment, message sending/receiving, automatic
/// reconnection with exponential backoff, and connection cleanup.
///
/// ## Reconnection Strategy
///
/// When a connection fails, the transport will automatically attempt to reconnect
/// with exponential backoff up to `max_reconnect_attempts`. Pending messages
/// are buffered (up to buffer limit) and sent once reconnection succeeds.
#[derive(Debug)]
pub struct WebSocketTransport {
    /// Transport configuration
    pub(crate) config: WebSocketConfig,

    /// Connection state
    pub(crate) connection_state: Arc<Mutex<WebSocketState>>,

    /// WebSocket sender
    ws_sender: Option<mpsc::Sender<SocketCommand>>,

    /// Receiver from the read task
    reader_rx: Arc<Mutex<Option<mpsc::Receiver<MCPMessage>>>>,

    /// Receiver for control messages
    control_rx: Option<mpsc::Receiver<ControlMessage>>,

    /// Sender for control messages
    control_tx: Option<mpsc::Sender<ControlMessage>>,

    /// Peer address
    pub(crate) peer_addr: Arc<Mutex<Option<SocketAddr>>>,

    /// Local address
    pub(crate) local_addr: Arc<Mutex<Option<SocketAddr>>>,

    /// Transport metadata
    metadata: Arc<Mutex<TransportMetadata>>,

    /// Message buffer for pending sends during reconnection
    pub(crate) message_buffer: Arc<Mutex<Vec<MCPMessage>>>,

    /// Reconnection attempts counter
    pub(crate) reconnection_attempts: Arc<Mutex<u32>>,
}

impl WebSocketTransport {
    /// Create a new WebSocket transport
    ///
    /// Initializes a new WebSocket transport with the given configuration.
    /// The transport starts in a disconnected state and needs to be explicitly
    /// connected before use.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the WebSocket transport
    ///
    /// # Returns
    ///
    /// A new `WebSocketTransport` instance
    #[must_use]
    pub fn new(config: WebSocketConfig) -> Self {
        // Create message channels
        let (_msg_tx, msg_rx) = mpsc::channel(100);
        let (socket_tx, _socket_rx) = mpsc::channel(100);
        let (control_tx, control_rx) = mpsc::channel(100);

        // Create additional info with transport type
        let mut additional_info = HashMap::new();
        additional_info.insert("transport_type".to_string(), "websocket".to_string());
        additional_info.insert("peer_addr".to_string(), config.url.clone());

        let transport_metadata = TransportMetadata {
            connection_id: Uuid::new_v4().to_string(),
            remote_address: config.url.parse().ok(),
            local_address: None,
            encryption_format: Some(config.encryption),
            compression_format: Some(config.compression),
            connected_at: Utc::now(),
            last_activity: Utc::now(),
            additional_info,
        };

        Self {
            config,
            connection_state: Arc::new(Mutex::new(WebSocketState::Disconnected)),
            ws_sender: Some(socket_tx),
            reader_rx: Arc::new(Mutex::new(Some(msg_rx))),
            control_rx: Some(control_rx),
            control_tx: Some(control_tx),
            peer_addr: Arc::new(Mutex::new(None)),
            local_addr: Arc::new(Mutex::new(None)),
            metadata: Arc::new(Mutex::new(transport_metadata)),
            message_buffer: Arc::new(Mutex::new(Vec::new())),
            reconnection_attempts: Arc::new(Mutex::new(0)),
        }
    }

    /// Check if the transport is connected (implementation moved here)
    async fn is_connected_impl(&self) -> bool {
        let state_guard = self.connection_state.lock().await;
        state_guard.is_connected()
    }

    /// Placeholder for internal message sending logic
    #[allow(dead_code)] // Reserved for WebSocket message sending system
    async fn send_internal(
        &self,
        ws_message: tokio_tungstenite::tungstenite::protocol::Message,
    ) -> Result<()> {
        if let Some(sender) = &self.ws_sender {
            match ws_message {
                tokio_tungstenite::tungstenite::protocol::Message::Text(text) => {
                    match serde_json::from_str::<MCPMessage>(&text) {
                        Ok(mcp_message) => {
                            if sender.send(SocketCommand::Send(mcp_message)).await.is_err() {
                                error!("WebSocket: Failed to send command to writer task");
                                return Err(MCPError::Transport(TransportError::ConnectionClosed(
                                    "Writer task channel closed".to_string(),
                                )));
                            }
                        }
                        Err(e) => {
                            error!(
                                "WebSocket: Failed to deserialize text message for sending: {}",
                                e
                            );
                            return Err(MCPError::Serialization(e.to_string()));
                        }
                    }
                }
                tokio_tungstenite::tungstenite::protocol::Message::Binary(_bytes) => {
                    error!("WebSocket: send_internal cannot directly send raw binary via MCPMessage command.");
                    return Err(MCPError::Transport(TransportError::ProtocolError(
                        "Raw binary send via send_internal needs rework".to_string(),
                    )));
                }
                _ => {
                    // Ignore non-data message types
                }
            }
            Ok(())
        } else {
            Err(MCPError::Transport(TransportError::ConnectionFailed(
                "WebSocket sender unavailable".to_string(),
            )))
        }
    }

    /// Attempt to reconnect with exponential backoff
    ///
    /// Implements automatic reconnection with exponential backoff strategy.
    /// Buffers messages during disconnection and sends them after reconnection.
    ///
    /// # Returns
    ///
    /// Result indicating success or failure after all retry attempts
    async fn attempt_reconnection(&mut self) -> Result<()> {
        let max_attempts = self.config.max_reconnect_attempts;
        let mut delay_ms = self.config.reconnect_delay_ms;

        for attempt in 1..=max_attempts {
            // Update reconnection attempt counter
            {
                let mut attempts = self.reconnection_attempts.lock().await;
                *attempts = attempt;
            }

            info!("Reconnection attempt {}/{}", attempt, max_attempts);

            // Try to connect
            match self.connect().await {
                Ok(()) => {
                    info!("✅ Reconnection successful after {} attempts", attempt);

                    // Reset attempt counter
                    {
                        let mut attempts = self.reconnection_attempts.lock().await;
                        *attempts = 0;
                    }

                    // Drain buffered messages
                    if let Some(ref sender) = self.ws_sender {
                        if let Err(e) =
                            drain_message_buffer(self.message_buffer.clone(), sender).await
                        {
                            warn!("Failed to drain message buffer after reconnection: {}", e);
                        }
                    }

                    return Ok(());
                }
                Err(e) => {
                    warn!("Reconnection attempt {} failed: {}", attempt, e);

                    // Exponential backoff (don't sleep on last attempt)
                    if attempt < max_attempts {
                        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                        delay_ms = (delay_ms * 2).min(30000); // Cap at 30 seconds
                    }
                }
            }
        }

        error!("❌ Reconnection failed after {} attempts", max_attempts);
        Err(MCPError::Transport(TransportError::ConnectionFailed(
            format!("Reconnection failed after {} attempts", max_attempts),
        )))
    }

    /// Drain buffered messages after reconnection
    ///
    /// Sends all buffered messages that accumulated during disconnection.
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    async fn drain_message_buffer(&self) -> Result<()> {
        if let Some(ref sender) = self.ws_sender {
            drain_message_buffer(self.message_buffer.clone(), sender).await
        } else {
            Ok(())
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
    ///
    /// # Returns
    ///
    /// Result indicating if the message was buffered or if buffer is full
    pub async fn buffer_message(&self, message: MCPMessage) -> Result<()> {
        buffer_message(message, self.message_buffer.clone()).await
    }

    /// Start keepalive ping task
    ///
    /// Starts a background task that sends periodic ping frames to keep
    /// the connection alive and detect disconnections early.
    fn start_keepalive_task(&self) {
        if let Some(ping_interval_secs) = self.config.ping_interval {
            start_keepalive_task(
                self.ws_sender.clone(),
                self.connection_state.clone(),
                ping_interval_secs,
            );
        }
    }

    /// Get the remote address of the WebSocket connection
    #[allow(dead_code)] // Reserved for connection address tracking
    pub async fn remote_addr(&self) -> std::result::Result<Option<SocketAddr>, MCPError> {
        // Access the peer_addr field instead of trying to access a stream field
        let peer_addr = self.peer_addr.lock().await;
        Ok(*peer_addr)
    }
}

#[async_trait]
impl Transport for WebSocketTransport {
    /// Send a message over the WebSocket transport
    ///
    /// Sends an MCP message over the established WebSocket connection.
    /// If the connection is not available, the message is buffered for
    /// sending after reconnection.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send
    ///
    /// # Returns
    ///
    /// Result indicating success or error
    async fn send_message(&self, message: MCPMessage) -> Result<()> {
        if !self.is_connected().await {
            // Buffer message for sending after reconnection
            warn!("WebSocket not connected, buffering message");
            return self.buffer_message(message).await;
        }

        // Send the message to the write task through the channel
        match self
            .ws_sender
            .as_ref()
            .expect("ws_sender should be set after connection")
            .send(SocketCommand::Send(message))
            .await
        {
            Ok(_) => {
                // Update last activity
                let mut meta = self.metadata.lock().await;
                meta.last_activity = Utc::now();
                Ok(())
            }
            Err(e) => Err(MCPError::Transport(TransportError::SendError(
                e.to_string(),
            ))),
        }
    }

    /// Receive a message from the WebSocket transport
    ///
    /// Waits for and receives the next MCP message from the WebSocket connection.
    ///
    /// # Returns
    ///
    /// Result containing the received message or an error
    async fn receive_message(&self) -> Result<MCPMessage> {
        if !self.is_connected().await {
            return Err(MCPError::Transport(TransportError::ConnectionClosed(
                "Cannot receive message, not connected".to_string(),
            )));
        }
        let mut reader_guard = self.reader_rx.lock().await;

        if let Some(ref mut rx) = *reader_guard {
            let received: Option<MCPMessage> = rx.recv().await;
            match received {
                Some(mcp_message) => Ok(mcp_message),
                None => {
                    error!("Reader channel (reader_rx) is closed. Cannot receive message.");
                    *self.connection_state.lock().await = WebSocketState::Disconnected;
                    Err(MCPError::Transport(TransportError::ConnectionClosed(
                        "Reader channel closed".to_string(),
                    )))
                }
            }
        } else {
            error!("Reader channel (reader_rx) is None. Cannot receive message.");
            Err(MCPError::Transport(TransportError::ConnectionClosed(
                "Reader channel unavailable".to_string(),
            )))
        }
    }

    /// Connect to the WebSocket server
    ///
    /// Establishes a connection to the WebSocket server specified in the configuration.
    /// This method creates the necessary background tasks for handling the connection.
    ///
    /// # Returns
    ///
    /// Result indicating success or error
    async fn connect(&mut self) -> Result<()> {
        {
            let mut state = self.connection_state.lock().await;
            if *state != WebSocketState::Disconnected {
                warn!(
                    "WebSocket connect called while not disconnected ({:?})",
                    *state
                );
                return Ok(());
            }
            *state = WebSocketState::Connecting;
        }

        let socket = establish_connection(
            &self.config,
            self.connection_state.clone(),
            self.peer_addr.clone(),
            self.local_addr.clone(),
        )
        .await?;

        {
            let mut meta = self.metadata.lock().await;
            meta.remote_address = *self.peer_addr.lock().await;
            meta.local_address = *self.local_addr.lock().await;
            meta.connected_at = Utc::now();
            meta.last_activity = Utc::now();
        }

        let (msg_tx, msg_rx) = mpsc::channel::<MCPMessage>(100);
        let (socket_tx, socket_rx) = mpsc::channel::<SocketCommand>(100);

        self.ws_sender = Some(socket_tx);

        {
            let mut reader_guard = self.reader_rx.lock().await;
            *reader_guard = Some(msg_rx);
        }

        start_websocket_task(socket, msg_tx, socket_rx, self.connection_state.clone()).await?;

        {
            let mut state = self.connection_state.lock().await;
            if *state == WebSocketState::Connecting {
                *state = WebSocketState::Connected;
                info!("WebSocket transport connected successfully.");
            } else {
                warn!(
                    "WebSocket state changed during connection ({:?}), not setting to Connected.",
                    *state
                );
            }
        }

        // Start keepalive task after successful connection
        self.start_keepalive_task();

        Ok(())
    }

    /// Disconnect from the WebSocket server
    ///
    /// Closes the connection to the WebSocket server and cleans up resources.
    ///
    /// # Returns
    ///
    /// Result indicating success or error
    async fn disconnect(&self) -> Result<()> {
        {
            let mut state = self.connection_state.lock().await;
            if *state == WebSocketState::Disconnected {
                info!("WebSocket already disconnected.");
                return Ok(());
            }
            *state = WebSocketState::Disconnected;
        }

        if let Some(sender) = &self.ws_sender {
            if let Err(e) = sender.send(SocketCommand::Close).await {
                error!("Failed to send close command to WebSocket task: {}", e);
            }
        }

        {
            let mut reader_guard = self.reader_rx.lock().await;
            *reader_guard = None;
        }

        info!("WebSocket disconnected.");

        Ok(())
    }

    /// Check if the transport is connected
    ///
    /// # Returns
    ///
    /// True if the transport is in the Connected state, false otherwise
    async fn is_connected(&self) -> bool {
        self.is_connected_impl().await
    }

    /// Get transport metadata
    ///
    /// # Returns
    ///
    /// Metadata about this transport connection
    async fn get_metadata(&self) -> crate::transport::types::TransportMetadata {
        // Await the lock future, then clone the guarded data
        let metadata_guard = self.metadata.lock().await;
        metadata_guard.clone()
    }

    /// Send raw bytes over the WebSocket transport
    /// Sends raw bytes as a WebSocket Binary message.
    async fn send_raw(&self, bytes: &[u8]) -> crate::error::Result<()> {
        if !self.is_connected().await {
            return Err(MCPError::Transport(TransportError::ConnectionClosed(
                "Cannot send raw bytes, not connected".to_string(),
            )));
        }

        if let Some(sender) = &self.ws_sender {
            let cmd = SocketCommand::SendRaw(bytes.to_vec());
            if sender.send(cmd).await.is_err() {
                error!("WebSocket: Failed to send raw bytes command to writer task");
                return Err(MCPError::Transport(TransportError::SendError(
                    "Writer task channel closed".to_string(),
                )));
            }
            Ok(())
        } else {
            error!("WebSocket: No sender available for raw bytes");
            Err(MCPError::Transport(TransportError::ConnectionFailed(
                "WebSocket sender unavailable".to_string(),
            )))
        }
    }
}

/// Handle incoming WebSocket connection
#[allow(dead_code)] // Reserved for WebSocket connection handling system
pub async fn handle_connection(_peer: SocketAddr, _stream: TcpStream) -> Result<()> {
    Ok(())
}

/// Process WebSocket socket messages
#[allow(dead_code)] // Reserved for WebSocket message processing system
pub async fn process_socket(
    _socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    _msg_tx: mpsc::Sender<MCPMessage>,
    mut _socket_rx: mpsc::Receiver<SocketCommand>,
    _control_tx: mpsc::Sender<ControlMessage>,
    _state: Arc<Mutex<WebSocketState>>,
    _peer: SocketAddr,
) {
}
