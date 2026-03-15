// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! WebSocket Transport Implementation for MCP Protocol
//!
//! This module provides WebSocket transport capabilities for the Machine Context Protocol,
//! replacing mock implementations with real WebSocket connections.

use dashmap::DashMap;
use std::sync::Arc;
use std::time::Instant;

use futures_util::SinkExt;
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, RwLock, broadcast, mpsc};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{accept_async, connect_async};
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

use crate::error::{MCPError, Result};
use crate::protocol::types::MCPMessage;
use crate::transport::frame::{Frame, MessageCodec};

/// WebSocket configuration
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    /// Bind address for server
    pub bind_address: String,
    /// Port number
    pub port: u16,
    /// Connection timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum connections
    pub max_connections: usize,
    /// Buffer size for messages
    pub buffer_size: usize,
    /// Connection timeout duration
    pub connection_timeout: std::time::Duration,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            port: 8080,
            timeout_seconds: 30,
            max_connections: 100,
            buffer_size: 1024,
            connection_timeout: std::time::Duration::from_secs(30),
        }
    }
}

/// Connection state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConnectionState {
    /// Connection is being established
    Connecting,
    /// Connection is active
    Connected,
    /// Connection is being closed
    Disconnecting,
    /// Connection is closed
    Disconnected,
    /// Connection failed
    Failed,
}

/// Connection information
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    /// Connection ID
    pub id: String,
    /// Remote address
    pub remote_address: String,
    /// Connection state
    pub state: ConnectionState,
    /// Connection start time
    pub connected_at: Instant,
    /// Last message time
    pub last_message_at: Option<Instant>,
    /// Message count
    pub message_count: u64,
    /// Last ping time
    pub last_ping: Option<Instant>,
    /// Last pong time
    pub last_pong: Option<Instant>,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Bytes received
    pub bytes_received: u64,
    /// Messages sent
    pub messages_sent: u64,
    /// Messages received
    pub messages_received: u64,
}

/// WebSocket transport (placeholder for now)
#[derive(Debug, Clone)]
pub struct WebSocketTransport {
    /// Connection information
    pub connection: ConnectionInfo,
    /// Configuration
    pub config: WebSocketConfig,
}

impl WebSocketTransport {
    /// Create new WebSocket transport
    pub fn new(connection: ConnectionInfo, config: WebSocketConfig) -> Self {
        Self { connection, config }
    }
}

/// Server events for WebSocket connections
#[derive(Debug, Clone)]
pub enum ServerEvent {
    /// New client connected
    ClientConnected(String),
    /// Client disconnected
    ClientDisconnected(String),
    /// Message received from client
    MessageReceived(String, MCPMessage),
    /// Connection error occurred
    ConnectionError(String, String),
}

/// WebSocket server for MCP protocol
#[derive(Debug)]
pub struct WebSocketServer {
    /// Configuration
    config: WebSocketConfig,
    /// Active connections
    connections: Arc<DashMap<String, ConnectionInfo>>,
    /// Connection message senders
    connection_senders: Arc<DashMap<String, mpsc::Sender<MCPMessage>>>,
    /// Message codec
    codec: MessageCodec,
    /// Broadcast sender for server events
    event_sender: broadcast::Sender<ServerEvent>,
}

impl WebSocketServer {
    /// Create new WebSocket server
    pub fn new(config: WebSocketConfig) -> Self {
        let (event_sender, _) = broadcast::channel(1000);

        Self {
            config,
            connections: Arc::new(DashMap::new()),
            connection_senders: Arc::new(DashMap::new()),
            codec: MessageCodec::new(),
            event_sender,
        }
    }

    /// Start the WebSocket server
    #[instrument(skip(self))]
    pub async fn start(&mut self) -> Result<()> {
        let addr = format!("{}:{}", self.config.bind_address, self.config.port);
        let listener = TcpListener::bind(&addr).await.map_err(|e| {
            MCPError::Transport(format!("Failed to bind to {}: {}", addr, e).into())
        })?;

        info!("WebSocket server listening on {}", addr);

        // Accept connections in a loop
        while let Ok((stream, peer_addr)) = listener.accept().await {
            let connection_id = Uuid::new_v4().to_string();

            info!("New connection from {}: {}", peer_addr, connection_id);

            // Handle connection
            let config = Arc::new(self.config.clone()); // Wrap config in Arc to avoid clone in spawn
            let connections = Arc::clone(&self.connections);
            let connection_senders = Arc::clone(&self.connection_senders);
            let codec = Arc::new(self.codec.clone()); // Wrap codec in Arc for sharing
            let event_sender = self.event_sender.clone(); // Broadcast sender is cheap to clone
            let peer_addr_str = peer_addr.to_string(); // Convert once outside spawn
            let connection_id_for_error = connection_id.clone(); // Clone for error message

            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(
                    stream,
                    connection_id,
                    peer_addr_str,
                    config,
                    connections,
                    connection_senders,
                    codec,
                    event_sender,
                )
                .await
                {
                    error!("Connection {} error: {}", connection_id_for_error, e);
                }
            });
        }

        Ok(())
    }

    /// Handle a WebSocket connection
    async fn handle_connection(
        stream: TcpStream,
        connection_id: String,
        peer_addr: String,
        _config: Arc<WebSocketConfig>, // Reserved for future connection configuration
        connections: Arc<DashMap<String, ConnectionInfo>>,
        connection_senders: Arc<DashMap<String, mpsc::Sender<MCPMessage>>>,
        codec: Arc<MessageCodec>,
        event_sender: broadcast::Sender<ServerEvent>,
    ) -> Result<()> {
        // Accept WebSocket connection
        let ws_stream = accept_async(stream)
            .await
            .map_err(|e| MCPError::Transport(format!("WebSocket handshake failed: {e}").into()))?;

        // Create connection info
        let connection_info = ConnectionInfo {
            id: connection_id.clone(),
            remote_address: peer_addr,
            state: ConnectionState::Connected,
            connected_at: Instant::now(),
            last_message_at: None,
            message_count: 0,
            last_ping: None,
            last_pong: None,
            bytes_sent: 0,
            bytes_received: 0,
            messages_sent: 0,
            messages_received: 0,
        };

        // Store connection info
        connections.insert(connection_id.clone(), connection_info);

        // Send connection event
        let _ = event_sender.send(ServerEvent::ClientConnected(connection_id.clone()));

        // Create message channel
        let (msg_tx, mut msg_rx) = mpsc::channel(100);

        // Store message sender
        connection_senders.insert(connection_id.clone(), msg_tx);

        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // Spawn task to handle outgoing messages
        let _connection_id_out = connection_id.clone(); // Reserved for logging/metrics
        let _event_sender_out = event_sender.clone(); // Reserved for connection events
        let codec_out = codec.clone();
        tokio::spawn(async move {
            while let Some(message) = msg_rx.recv().await {
                let frame = match codec_out.encode_message(&message).await {
                    Ok(frame) => frame,
                    Err(e) => {
                        error!("Failed to encode message: {}", e);
                        continue;
                    }
                };

                let ws_message = match serde_json::to_string(&frame) {
                    Ok(json) => Message::Text(json),
                    Err(e) => {
                        error!("Failed to serialize WebSocket frame: {}", e);
                        continue;
                    }
                };
                if let Err(e) = ws_sender.send(ws_message).await {
                    error!("Failed to send WebSocket message: {}", e);
                    break;
                }
            }
        });

        // Handle incoming messages
        while let Some(message) = ws_receiver.next().await {
            let message = match message {
                Ok(msg) => msg,
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
            };

            match message {
                Message::Text(text) => {
                    // Decode frame
                    let frame: Frame = match serde_json::from_str(&text) {
                        Ok(frame) => frame,
                        Err(e) => {
                            warn!("Failed to parse frame: {}", e);
                            continue;
                        }
                    };

                    // Decode message
                    let mcp_message = match codec.decode_message(&frame).await {
                        Ok(message) => message,
                        Err(e) => {
                            warn!("Failed to decode MCP message: {}", e);
                            continue;
                        }
                    };

                    // Send message event
                    let _ = event_sender.send(ServerEvent::MessageReceived(
                        connection_id.clone(),
                        mcp_message,
                    ));
                }
                Message::Close(_) => {
                    info!("Client {} disconnected", connection_id);
                    break;
                }
                _ => {
                    // Ignore other message types
                }
            }
        }

        // Cleanup
        connections.remove(&connection_id);
        connection_senders.remove(&connection_id);

        // Send disconnect event
        let _ = event_sender.send(ServerEvent::ClientDisconnected(connection_id));

        Ok(())
    }

    /// Send message to a specific client
    pub async fn send_to_client(&self, client_id: &str, message: MCPMessage) -> Result<()> {
        if let Some(sender) = self.connection_senders.get(client_id) {
            sender
                .value()
                .send(message)
                .await
                .map_err(|_| MCPError::Transport("Failed to send message to client".into()))?;
            Ok(())
        } else {
            Err(MCPError::Transport(
                format!("Client {} not found", client_id).into(),
            ))
        }
    }

    /// Get active connections
    pub async fn get_connections(&self) -> Vec<ConnectionInfo> {
        self.connections
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Subscribe to server events
    pub fn subscribe(&self) -> broadcast::Receiver<ServerEvent> {
        self.event_sender.subscribe()
    }
}

/// WebSocket client for MCP protocol
#[derive(Debug)]
pub struct WebSocketClient {
    /// Configuration
    config: WebSocketConfig,
    /// Message sender
    message_sender: Arc<Mutex<Option<mpsc::Sender<MCPMessage>>>>,
    /// Connection info
    connection_info: Arc<RwLock<Option<ConnectionInfo>>>,
    /// Message codec
    codec: MessageCodec,
}

impl WebSocketClient {
    /// Create new WebSocket client
    pub fn new(config: WebSocketConfig) -> Self {
        Self {
            config,
            message_sender: Arc::new(Mutex::new(None)),
            connection_info: Arc::new(RwLock::new(None)),
            codec: MessageCodec::new(),
        }
    }

    /// Connect to WebSocket server
    #[instrument(skip(self))]
    pub async fn connect(&self, url: &str) -> Result<()> {
        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| MCPError::Transport(format!("Connection failed: {e}").into()))?;

        let connection_info = ConnectionInfo {
            id: Uuid::new_v4().to_string(),
            remote_address: url.to_string(),
            state: ConnectionState::Connected,
            connected_at: Instant::now(),
            last_message_at: None,
            message_count: 0,
            last_ping: None,
            last_pong: None,
            bytes_sent: 0,
            bytes_received: 0,
            messages_sent: 0,
            messages_received: 0,
        };

        // Store connection info
        {
            let mut info = self.connection_info.write().await;
            *info = Some(connection_info);
        }

        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // Create message channel
        let (msg_tx, mut msg_rx) = mpsc::channel(100);

        // Store message sender
        {
            let mut sender = self.message_sender.lock().await;
            *sender = Some(msg_tx);
        }

        let codec = self.codec.clone();

        // Spawn task to handle outgoing messages
        tokio::spawn(async move {
            while let Some(message) = msg_rx.recv().await {
                let frame = match codec.encode_message(&message).await {
                    Ok(frame) => frame,
                    Err(e) => {
                        error!("Failed to encode message: {}", e);
                        continue;
                    }
                };

                let ws_message = match serde_json::to_string(&frame) {
                    Ok(json) => Message::Text(json),
                    Err(e) => {
                        error!("Failed to serialize WebSocket frame: {}", e);
                        continue;
                    }
                };
                if let Err(e) = ws_sender.send(ws_message).await {
                    error!("Failed to send WebSocket message: {}", e);
                    break;
                }
            }
        });

        // Spawn task to handle incoming messages
        let codec_in = self.codec.clone();
        tokio::spawn(async move {
            while let Some(message) = ws_receiver.next().await {
                let message = match message {
                    Ok(msg) => msg,
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                };

                match message {
                    Message::Text(text) => {
                        // Decode frame
                        let frame: Frame = match serde_json::from_str(&text) {
                            Ok(frame) => frame,
                            Err(e) => {
                                warn!("Failed to parse frame: {}", e);
                                continue;
                            }
                        };

                        // Decode message
                        let _mcp_message = match codec_in.decode_message(&frame).await {
                            Ok(message) => message,
                            Err(e) => {
                                warn!("Failed to decode MCP message: {}", e);
                                continue;
                            }
                        };

                        // FUTURE: [Protocol] Handle received message
                        // Tracking: Requires message routing implementation
                    }
                    Message::Close(_) => {
                        info!("Server disconnected");
                        break;
                    }
                    _ => {
                        // Ignore other message types
                    }
                }
            }
        });

        info!("Connected to WebSocket server: {}", url);
        Ok(())
    }

    /// Send message to server
    pub async fn send(&self, message: MCPMessage) -> Result<()> {
        let sender = self.message_sender.lock().await;
        if let Some(ref sender) = *sender {
            sender
                .send(message)
                .await
                .map_err(|_| MCPError::Transport("Failed to send message".into()))?;
            Ok(())
        } else {
            Err(MCPError::Transport("Not connected".into()))
        }
    }

    /// Send message and wait for response
    pub async fn request(&self, message: MCPMessage) -> Result<MCPMessage> {
        // Create response channel (reserved for future request/response correlation)
        let (_response_tx, mut response_rx) = mpsc::channel(1);

        // Send message (simplified - in real implementation would handle request/response correlation)
        let sender = self.message_sender.lock().await;
        if let Some(ref sender) = *sender {
            sender
                .send(message)
                .await
                .map_err(|e| MCPError::Transport(format!("Failed to send message: {e}").into()))?;
        } else {
            return Err(MCPError::Transport("Not connected".into()));
        }

        // Wait for response
        let response = tokio::time::timeout(self.config.connection_timeout, response_rx.recv())
            .await
            .map_err(|_| MCPError::Transport("Response timeout".into()))?
            .ok_or_else(|| MCPError::Transport("Response channel closed".into()))?;

        Ok(response)
    }

    /// Get connection info
    pub async fn get_connection_info(&self) -> Option<ConnectionInfo> {
        self.connection_info.read().await.clone()
    }

    /// Disconnect from server
    pub async fn disconnect(&self) -> Result<()> {
        // Clear message sender
        {
            let mut sender = self.message_sender.lock().await;
            *sender = None;
        }

        // Clear connection info
        {
            let mut info = self.connection_info.write().await;
            if let Some(ref mut conn_info) = *info {
                conn_info.state = ConnectionState::Disconnected;
            }
        }

        info!("Disconnected from WebSocket server");
        Ok(())
    }
}
