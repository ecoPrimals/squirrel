//! WebSocket Transport Implementation for MCP Protocol
//!
//! This module provides WebSocket transport capabilities for the Machine Context Protocol,
//! replacing mock implementations with real WebSocket connections.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, Mutex, RwLock};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{accept_async, connect_async, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

use crate::error::{MCPError, Result};
use crate::protocol::types::MCPMessage;
use crate::transport::frame::{Frame, MessageCodec};

/// WebSocket connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    /// Server bind address
    pub bind_address: String,
    /// Server port
    pub port: u16,
    /// Maximum message size (bytes)
    pub max_message_size: usize,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Ping interval
    pub ping_interval: Duration,
    /// Pong timeout
    pub pong_timeout: Duration,
    /// Maximum concurrent connections
    pub max_connections: usize,
    /// Enable compression
    pub enable_compression: bool,
    /// Subprotocol
    pub subprotocol: Option<String>,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        use crate::constants::{message_sizes, network, protocol, timeouts};

        Self {
            bind_address: network::DEFAULT_BIND_ADDRESS.to_string(),
            port: network::DEFAULT_WEBSOCKET_PORT,
            max_message_size: message_sizes::DEFAULT_MAX_MESSAGE_SIZE,
            connection_timeout: timeouts::DEFAULT_CONNECTION_TIMEOUT,
            ping_interval: timeouts::DEFAULT_PING_INTERVAL,
            pong_timeout: timeouts::DEFAULT_PONG_TIMEOUT,
            max_connections: network::DEFAULT_MAX_CONNECTIONS,
            enable_compression: true,
            subprotocol: Some(protocol::DEFAULT_MCP_SUBPROTOCOL.to_string()),
        }
    }
}

/// WebSocket connection state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    Connecting,
    Connected,
    Disconnected,
    Error(String),
}

/// WebSocket connection information
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub id: String,
    pub remote_address: String,
    pub state: ConnectionState,
    pub connected_at: Instant,
    pub last_ping: Option<Instant>,
    pub last_pong: Option<Instant>,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
}

/// WebSocket server for MCP protocol
#[derive(Debug)]
pub struct WebSocketServer {
    /// Configuration
    config: WebSocketConfig,
    /// Active connections
    connections: Arc<RwLock<HashMap<String, ConnectionInfo>>>,
    /// Connection message senders
    connection_senders: Arc<RwLock<HashMap<String, mpsc::Sender<MCPMessage>>>>,
    /// Message codec
    codec: MessageCodec,
    /// Broadcast sender for server events
    event_sender: broadcast::Sender<ServerEvent>,
    /// Shutdown signal
    shutdown_tx: Option<mpsc::Sender<()>>,
}

/// WebSocket client for MCP protocol
#[derive(Debug)]
pub struct WebSocketClient {
    /// Configuration
    config: WebSocketConfig,
    /// Connection info
    connection_info: Arc<RwLock<Option<ConnectionInfo>>>,
    /// Message codec
    codec: MessageCodec,
    /// Message sender
    message_sender: Arc<Mutex<Option<mpsc::Sender<MCPMessage>>>>,
    /// Response receivers
    response_receivers: Arc<RwLock<HashMap<String, mpsc::Sender<MCPMessage>>>>,
}

/// Server events for MCP protocol coordination and intelligence
#[derive(Debug, Clone)]
pub enum ServerEvent {
    ConnectionEstablished(String),
    ConnectionClosed(String),
    MessageReceived(String, MCPMessage),
    Error(String, MCPError),
    
    // New MCP coordination events for enhanced intelligence
    /// Serialization error occurred during message processing
    SerializationError(String, String),
    
    /// Connection error occurred during communication
    ConnectionError(String, String),
    
    /// Heartbeat event for connection health monitoring
    /// (connection_id, message_count, last_message_size)
    Heartbeat(String, u64, u64),
}

/// WebSocket connection handler
#[derive(Debug)]
struct ConnectionHandler {
    connection_id: String,
    connection_info: Arc<RwLock<ConnectionInfo>>,
    event_sender: broadcast::Sender<ServerEvent>,
    codec: MessageCodec,
    config: WebSocketConfig,
}

impl WebSocketServer {
    /// Create a new WebSocket server
    pub fn new(config: WebSocketConfig) -> Self {
        let (event_sender, _) = broadcast::channel(1000);

        Self {
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
            connection_senders: Arc::new(RwLock::new(HashMap::new())),
            codec: MessageCodec::new(),
            event_sender,
            shutdown_tx: None,
        }
    }

    /// Start the WebSocket server
    #[instrument(skip(self))]
    pub async fn start(&mut self) -> Result<()> {
        let bind_addr = format!("{}:{}", self.config.bind_address, self.config.port);
        let listener = TcpListener::bind(&bind_addr).await.map_err(|e| {
            MCPError::Transport(format!("Failed to bind to {bind_addr}: {e}").into())
        })?;

        info!("WebSocket server listening on {}", bind_addr);

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        let connections = Arc::clone(&self.connections);
        let connection_senders = Arc::clone(&self.connection_senders);
        let event_sender = self.event_sender.clone();
        let codec = self.codec.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    accept_result = listener.accept() => {
                        match accept_result {
                            Ok((stream, addr)) => {
                                let connection_id = Uuid::new_v4().to_string();

                                // Check connection limit
                                let connection_count = connections.read().await.len();
                                if connection_count >= config.max_connections {
                                    warn!("Connection limit reached, rejecting connection from {}", addr);
                                    continue;
                                }

                                info!("New connection from {}", addr);

                                let connection_info = ConnectionInfo {
                                    id: connection_id.clone(),
                                    remote_address: addr.to_string(),
                                    state: ConnectionState::Connecting,
                                    connected_at: Instant::now(),
                                    last_ping: None,
                                    last_pong: None,
                                    bytes_sent: 0,
                                    bytes_received: 0,
                                    messages_sent: 0,
                                    messages_received: 0,
                                };

                                let connection_info = Arc::new(RwLock::new(connection_info));
                                connections.write().await.insert(connection_id.clone(), connection_info.read().await.clone());

                                // Create message channel for this connection
                                let (message_tx, message_rx) = mpsc::channel(100);
                                connection_senders.write().await.insert(connection_id.clone(), message_tx);

                                let handler = ConnectionHandler {
                                    connection_id: connection_id.clone(),
                                    connection_info: Arc::clone(&connection_info),
                                    event_sender: event_sender.clone(),
                                    codec: codec.clone(),
                                    config: config.clone(),
                                };

                                let connections = Arc::clone(&connections);
                                let connection_senders = Arc::clone(&connection_senders);
                                let event_sender_spawn = event_sender.clone();
                                tokio::spawn(async move {
                                    if let Err(e) = handler.handle_connection(stream, message_rx).await {
                                        error!("Connection error: {}", e);
                                        let _ = event_sender_spawn.send(ServerEvent::Error(connection_id.clone(), e));
                                    }

                                    // Clean up connection and sender
                                    connections.write().await.remove(&connection_id);
                                    connection_senders.write().await.remove(&connection_id);
                                    let _ = event_sender_spawn.send(ServerEvent::ConnectionClosed(connection_id));
                                });
                            }
                            Err(e) => {
                                error!("Failed to accept connection: {}", e);
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Shutdown signal received, stopping server");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the WebSocket server
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(()).await;
        }
        Ok(())
    }

    /// Get active connections
    pub async fn get_connections(&self) -> Vec<ConnectionInfo> {
        self.connections.read().await.values().cloned().collect()
    }

    /// Subscribe to server events
    pub fn subscribe_events(&self) -> broadcast::Receiver<ServerEvent> {
        self.event_sender.subscribe()
    }

    /// Send message to specific connection
    pub async fn send_to_connection(&self, connection_id: &str, message: MCPMessage) -> Result<()> {
        if let Some(sender) = self.connection_senders.read().await.get(connection_id) {
            sender.send(message).await.map_err(|e| {
                MCPError::Transport(
                    format!("Failed to send message to {connection_id}: {e}").into(),
                )
            })?;
        } else {
            return Err(MCPError::Transport(
                format!("Connection {connection_id} not found").into(),
            ));
        }
        Ok(())
    }

    /// Broadcast message to all connections
    pub async fn broadcast_message(&self, message: MCPMessage) -> Result<()> {
        let senders = self.connection_senders.read().await;
        let mut failed_connections = Vec::new();

        for (connection_id, sender) in senders.iter() {
            if let Err(e) = sender.send(message.clone()).await {
                error!(
                    "Failed to send message to connection {}: {}",
                    connection_id, e
                );
                failed_connections.push(connection_id.clone());
            }
        }

        // Clean up failed connections
        if !failed_connections.is_empty() {
            drop(senders); // Release read lock
            let mut senders = self.connection_senders.write().await;
            let mut connections = self.connections.write().await;

            for connection_id in failed_connections {
                senders.remove(&connection_id);
                connections.remove(&connection_id);
                let _ = self
                    .event_sender
                    .send(ServerEvent::ConnectionClosed(connection_id));
            }
        }

        Ok(())
    }
}

impl ConnectionHandler {
    /// Handle a WebSocket connection
    #[instrument(skip(self, stream, message_rx))]
    async fn handle_connection(
        &self,
        stream: TcpStream,
        mut message_rx: mpsc::Receiver<MCPMessage>,
    ) -> Result<()> {
        // Accept WebSocket connection
        let ws_stream = accept_async(stream)
            .await
            .map_err(|e| MCPError::Transport(format!("WebSocket handshake failed: {e}").into()))?;

        // Update connection state
        {
            let mut info = self.connection_info.write().await;
            info.state = ConnectionState::Connected;
        }

        let _ = self.event_sender.send(ServerEvent::ConnectionEstablished(
            self.connection_id.clone(),
        ));

        // Split the WebSocket stream for bidirectional communication
        let (mut ws_sink, ws_stream) = ws_stream.split();

        // Handle incoming and outgoing messages concurrently
        let connection_id = self.connection_id.clone();
        let connection_info = Arc::clone(&self.connection_info);
        let event_sender = self.event_sender.clone();
        let codec = self.codec.clone();
        let config = self.config.clone();

        // Task for handling outgoing messages
        let outgoing_task = tokio::spawn(async move {
            // Initialize MCP coordination metrics
            let mut last_heartbeat = std::time::Instant::now();
            let heartbeat_interval = config.ping_interval;
            let mut message_count = 0u64;
            
            info!("🐿️ Starting MCP outgoing message handler for connection {}", connection_id);
            
            while let Some(message) = message_rx.recv().await {
                let json_message = match serde_json::to_string(&message) {
                    Ok(json) => json,
                    Err(e) => {
                        error!("Failed to serialize message: {}", e);
                        // Notify event system of serialization failure for MCP coordination
                        let _ = event_sender.send(ServerEvent::SerializationError(
                            connection_id.clone(),
                            format!("Serialization failed: {}", e)
                        )).await;
                        continue;
                    }
                };

                let message_len = json_message.len() as u64;
                message_count += 1;

                if let Err(e) = ws_sink.send(Message::Text(json_message)).await {
                    error!("Failed to send message: {}", e);
                    // Notify event system of send failure for MCP coordination
                    let _ = event_sender.send(ServerEvent::ConnectionError(
                        connection_id.clone(),
                        format!("Send failed: {}", e)
                    )).await;
                    break;
                }

                // Update connection stats with MCP-aware tracking
                {
                    let mut info = connection_info.write().await;
                    info.messages_sent += 1;
                    info.bytes_sent += message_len;
                    
                    // Add MCP coordination intelligence
                    if message_count % 100 == 0 {
                        debug!("🐿️ MCP coordination: {} messages sent on connection {}", message_count, connection_id);
                    }
                }
                
                // MCP-aware heartbeat and health monitoring
                if last_heartbeat.elapsed() > heartbeat_interval {
                    // Send heartbeat event for MCP coordination
                    let _ = event_sender.send(ServerEvent::Heartbeat(
                        connection_id.clone(),
                        message_count,
                        message_len
                    )).await;
                    last_heartbeat = std::time::Instant::now();
                }
            }
            
            // Final MCP coordination cleanup
            info!("🐿️ MCP outgoing handler completed for connection {} (sent {} messages)", 
                  connection_id, message_count);
            let _ = event_sender.send(ServerEvent::ConnectionClosed(connection_id)).await;
        });

        // Task for handling incoming messages
        let incoming_task = {
            let connection_id = self.connection_id.clone();
            let connection_info = Arc::clone(&self.connection_info);
            let event_sender = self.event_sender.clone();
            let config = self.config.clone();
            tokio::spawn(async move {
                let handler = ConnectionHandler {
                    connection_id,
                    connection_info,
                    event_sender,
                    codec,
                    config,
                };
                handler.handle_incoming_messages(ws_stream).await
            })
        };

        // Wait for either task to complete
        tokio::select! {
            _ = outgoing_task => {},
            result = incoming_task => {
                if let Err(e) = result {
                    error!("Incoming message handler task error: {}", e);
                } else if let Ok(Err(e)) = result {
                    error!("Incoming message handler error: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Handle incoming WebSocket messages
    #[instrument(skip(self, ws_stream))]
    async fn handle_incoming_messages(
        &self,
        mut ws_stream: futures_util::stream::SplitStream<WebSocketStream<TcpStream>>,
    ) -> Result<()> {
        let mut ping_interval = tokio::time::interval(self.config.ping_interval);

        loop {
            tokio::select! {
                msg = ws_stream.next() => {
                    match msg {
                        Some(Ok(message)) => {
                            if let Err(e) = self.handle_message(message).await {
                                error!("Error handling message: {}", e);
                                let _ = self.event_sender.send(ServerEvent::Error(self.connection_id.clone(), e));
                            }
                        }
                        Some(Err(e)) => {
                            error!("WebSocket error: {}", e);
                            break;
                        }
                        None => {
                            debug!("WebSocket connection closed");
                            break;
                        }
                    }
                }
                _ = ping_interval.tick() => {
                    // Note: We can't send ping here directly since we don't have access to the sink
                    // This would need to be sent through a separate channel if needed
                }
            }
        }

        Ok(())
    }

    /// Handle individual WebSocket message
    async fn handle_message(&self, message: Message) -> Result<()> {
        match message {
            Message::Text(ref text) => {
                self.handle_text_message(text.clone()).await?;
            }
            Message::Binary(ref data) => {
                self.handle_binary_message(data.clone()).await?;
            }
            Message::Ping(ref data) => {
                self.handle_ping(data.to_vec()).await?;
            }
            Message::Pong(ref data) => {
                self.handle_pong(data.to_vec()).await?;
            }
            Message::Close(close_frame) => {
                info!("Received close frame: {:?}", close_frame);
                return Ok(());
            }
            Message::Frame(_) => {
                // Raw frames are handled by the underlying library
            }
        }

        // Update connection stats
        {
            let mut info = self.connection_info.write().await;
            info.messages_received += 1;
            info.bytes_received += message.len() as u64;
        }

        Ok(())
    }

    /// Handle text message
    async fn handle_text_message(&self, text: String) -> Result<()> {
        debug!("Received text message: {}", text);

        // Parse as MCP message
        let mcp_message: MCPMessage = serde_json::from_str(&text)
            .map_err(|e| MCPError::Transport(format!("Invalid MCP message: {e}").into()))?;

        // Send to event handler
        let _ = self.event_sender.send(ServerEvent::MessageReceived(
            self.connection_id.clone(),
            mcp_message,
        ));

        Ok(())
    }

    /// Handle binary message
    async fn handle_binary_message(&self, data: Vec<u8>) -> Result<()> {
        debug!("Received binary message: {} bytes", data.len());

        // Create frame and decode
        let frame = Frame::from_vec(data);
        let mcp_message = self.codec.decode_message(&frame).await?;

        // Send to event handler
        let _ = self.event_sender.send(ServerEvent::MessageReceived(
            self.connection_id.clone(),
            mcp_message,
        ));

        Ok(())
    }

    /// Handle ping message
    async fn handle_ping(&self, _data: Vec<u8>) -> Result<()> {
        debug!("Received ping");

        // Update connection info - pong response is automatically handled by the WebSocket library
        {
            let mut info = self.connection_info.write().await;
            info.last_ping = Some(Instant::now());
        }

        Ok(())
    }

    /// Handle pong message
    async fn handle_pong(&self, _data: Vec<u8>) -> Result<()> {
        debug!("Received pong");

        // Update connection info
        {
            let mut info = self.connection_info.write().await;
            info.last_pong = Some(Instant::now());
        }

        Ok(())
    }
}

impl WebSocketClient {
    /// Create a new WebSocket client
    pub fn new(config: WebSocketConfig) -> Self {
        Self {
            config,
            connection_info: Arc::new(RwLock::new(None)),
            codec: MessageCodec::new(),
            message_sender: Arc::new(Mutex::new(None)),
            response_receivers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Connect to WebSocket server
    #[instrument(skip(self))]
    pub async fn connect(&self, url: &str) -> Result<()> {
        info!("Connecting to WebSocket server: {}", url);

        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| MCPError::Transport(format!("Failed to connect to {url}: {e}").into()))?;

        // Create connection info
        let connection_info = ConnectionInfo {
            id: Uuid::new_v4().to_string(),
            remote_address: url.to_string(),
            state: ConnectionState::Connected,
            connected_at: Instant::now(),
            last_ping: None,
            last_pong: None,
            bytes_sent: 0,
            bytes_received: 0,
            messages_sent: 0,
            messages_received: 0,
        };

        {
            let mut info = self.connection_info.write().await;
            *info = Some(connection_info);
        }

        // Start message handler
        self.start_message_handler(ws_stream).await?;

        info!("Connected to WebSocket server");
        Ok(())
    }

    /// Start message handler
    async fn start_message_handler(
        &self,
        ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    ) -> Result<()> {
        let (ws_sink, mut ws_stream) = ws_stream.split();
        let ws_sink: Arc<
            Mutex<
                futures_util::stream::SplitSink<
                    WebSocketStream<MaybeTlsStream<TcpStream>>,
                    Message,
                >,
            >,
        > = Arc::new(Mutex::new(ws_sink));
        let ws_sink_clone = ws_sink.clone();
        let ws_sink_clone2 = ws_sink.clone();

        let (message_tx, mut message_rx) = mpsc::channel(100);

        // Store message sender
        {
            let mut sender = self.message_sender.lock().await;
            *sender = Some(message_tx);
        }

        let connection_info = Arc::clone(&self.connection_info);
        let codec = self.codec.clone();
        let response_receivers = Arc::clone(&self.response_receivers);

        // Message sending task with MCP coordination intelligence
        let message_sender_task = tokio::spawn(async move {
            let ws_sink = ws_sink_clone;
            let mut local_message_count = 0u64;
            let mut local_bytes_sent = 0u64;
            let task_start_time = std::time::Instant::now();
            
            debug!("🐿️ Starting MCP message sender task with coordination intelligence");
            
            while let Some(message) = message_rx.recv().await {
                let json_message = serde_json::to_string(&message).unwrap_or_default();
                let message_size = json_message.len() as u64;
                
                let mut sink = ws_sink.lock().await;
                if let Err(e) = sink.send(Message::Text(json_message.clone())).await {
                    error!("Failed to send message: {}", e);
                    break;
                } else {
                    // Successfully sent message - update MCP coordination statistics
                    local_message_count += 1;
                    local_bytes_sent += message_size;
                    
                    // Update connection info with actual available fields
                    {
                        let mut info = connection_info.write().await;
                        info.messages_sent += 1;
                        info.bytes_sent += message_size;
                        
                        // MCP coordination intelligence: periodic stats reporting
                        if local_message_count % 50 == 0 {
                            let throughput = local_message_count as f64 / task_start_time.elapsed().as_secs_f64();
                            debug!("🐿️ MCP coordination stats: {} messages, {} bytes, {:.2} msg/sec", 
                                   local_message_count, local_bytes_sent, throughput);
                        }
                    }
                }
                
                // Periodic MCP coordination health reporting
                if local_message_count % 100 == 0 && local_message_count > 0 {
                    let elapsed = task_start_time.elapsed();
                    let throughput = local_message_count as f64 / elapsed.as_secs_f64();
                    info!("🐿️ MCP coordination milestone: {} messages sent, {:.2} MB, {:.2} msg/sec over {:?}", 
                          local_message_count, local_bytes_sent as f64 / 1_048_576.0, throughput, elapsed);
                }
            }
            
            // Final MCP coordination summary
            let final_elapsed = task_start_time.elapsed();
            let final_throughput = if final_elapsed.as_secs_f64() > 0.0 {
                local_message_count as f64 / final_elapsed.as_secs_f64()
            } else { 0.0 };
            
            info!("🐿️ MCP message sender task completed: {} messages, {} bytes, {:.2} msg/sec", 
                  local_message_count, local_bytes_sent, final_throughput);
        });

        // Message receiving task
        let message_receiver_task = tokio::spawn(async move {
            let ws_sink = ws_sink_clone2;
            while let Some(message) = ws_stream.next().await {
                match message {
                    Ok(Message::Text(text)) => {
                        if let Ok(mcp_message) = serde_json::from_str::<MCPMessage>(&text) {
                            // Handle response
                            if let Some(sender) =
                                response_receivers.write().await.remove(&mcp_message.id.0)
                            {
                                let _ = sender.send(mcp_message).await;
                            }
                        }
                    }
                    Ok(Message::Binary(data)) => {
                        let frame = Frame::from_vec(data);
                        if let Ok(mcp_message) = codec.decode_message(&frame).await {
                            // Handle response
                            if let Some(sender) =
                                response_receivers.write().await.remove(&mcp_message.id.0)
                            {
                                let _ = sender.send(mcp_message).await;
                            }
                        }
                    }
                    Ok(Message::Ping(data)) => {
                        let mut sink = ws_sink.lock().await;
                        if let Err(e) = sink.send(Message::Pong(data)).await {
                            tracing::error!("Failed to send pong: {}", e);
                            break;
                        }
                    }
                    Ok(Message::Pong(ref _data)) => {
                        // Handle pong
                    }
                    Ok(Message::Close(_)) => {
                        info!("Connection closed by server");
                        break;
                    }
                    Ok(Message::Frame(_)) => {
                        // Raw frames handled by library
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                }
            }
        });

        // Wait for tasks to complete
        tokio::select! {
            _ = message_sender_task => {},
            _ = message_receiver_task => {},
        }

        Ok(())
    }

    /// Send message and wait for response
    pub async fn send_message(&self, message: MCPMessage) -> Result<MCPMessage> {
        let (response_tx, mut response_rx) = mpsc::channel(1);

        // Store response receiver
        {
            let mut receivers = self.response_receivers.write().await;
            receivers.insert(message.id.0.clone(), response_tx);
        }

        // Send message
        if let Some(sender) = self.message_sender.lock().await.as_ref() {
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

/// WebSocket transport implementation
#[derive(Debug)]
pub struct WebSocketTransport {
    /// Server instance
    server: Option<WebSocketServer>,
    /// Client instance
    client: Option<WebSocketClient>,
    /// Configuration
    config: WebSocketConfig,
}

impl WebSocketTransport {
    /// Create new WebSocket transport
    pub fn new(config: WebSocketConfig) -> Self {
        Self {
            server: None,
            client: None,
            config,
        }
    }

    /// Create server transport
    pub fn server(config: WebSocketConfig) -> Self {
        let server = WebSocketServer::new(config.clone());
        Self {
            server: Some(server),
            client: None,
            config,
        }
    }

    /// Create client transport
    pub fn client(config: WebSocketConfig) -> Self {
        let client = WebSocketClient::new(config.clone());
        Self {
            server: None,
            client: Some(client),
            config,
        }
    }

    /// Start server
    pub async fn start_server(&mut self) -> Result<()> {
        if let Some(ref mut server) = self.server {
            server.start().await?;
        } else {
            return Err(MCPError::Transport("No server configured".into()));
        }
        Ok(())
    }

    /// Connect client
    pub async fn connect_client(&self, url: &str) -> Result<()> {
        if let Some(ref client) = self.client {
            client.connect(url).await?;
        } else {
            return Err(MCPError::Transport("No client configured".into()));
        }
        Ok(())
    }

    /// Send message (client mode)
    pub async fn send_message(&self, message: MCPMessage) -> Result<MCPMessage> {
        if let Some(ref client) = self.client {
            client.send_message(message).await
        } else {
            Err(MCPError::Transport("No client configured".into()))
        }
    }

    /// Subscribe to server events
    pub fn subscribe_events(&self) -> Result<broadcast::Receiver<ServerEvent>> {
        if let Some(ref server) = self.server {
            Ok(server.subscribe_events())
        } else {
            Err(MCPError::Transport("No server configured".into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::types::MessageType;

    #[tokio::test]
    async fn test_websocket_config_default() {
        let config = WebSocketConfig::default();
        assert_eq!(config.bind_address, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.max_connections, 100);
    }

    #[tokio::test]
    async fn test_websocket_server_creation() {
        let config = WebSocketConfig::default();
        let server = WebSocketServer::new(config);

        let connections = server.get_connections().await;
        assert_eq!(connections.len(), 0);
    }

    #[tokio::test]
    async fn test_websocket_client_creation() {
        let config = WebSocketConfig::default();
        let client = WebSocketClient::new(config);

        let connection_info = client.get_connection_info().await;
        assert!(connection_info.is_none());
    }

    #[tokio::test]
    async fn test_websocket_transport_creation() {
        let config = WebSocketConfig::default();
        let transport = WebSocketTransport::new(config);

        assert!(transport.server.is_none());
        assert!(transport.client.is_none());
    }

    #[tokio::test]
    async fn test_server_transport_creation() {
        let config = WebSocketConfig::default();
        let transport = WebSocketTransport::server(config);

        assert!(transport.server.is_some());
        assert!(transport.client.is_none());
    }

    #[tokio::test]
    async fn test_client_transport_creation() {
        let config = WebSocketConfig::default();
        let transport = WebSocketTransport::client(config);

        assert!(transport.server.is_none());
        assert!(transport.client.is_some());
    }

    #[tokio::test]
    async fn test_connection_info_states() {
        let info = ConnectionInfo {
            id: "test-123".to_string(),
            remote_address: "127.0.0.1:8080".to_string(),
            state: ConnectionState::Connected,
            connected_at: Instant::now(),
            last_ping: None,
            last_pong: None,
            bytes_sent: 0,
            bytes_received: 0,
            messages_sent: 0,
            messages_received: 0,
        };

        assert_eq!(info.id, "test-123");
        assert_eq!(info.state, ConnectionState::Connected);
        assert_eq!(info.bytes_sent, 0);
        assert_eq!(info.messages_sent, 0);
    }

    #[tokio::test]
    async fn test_server_event_types() {
        let connection_id = "test-conn-123".to_string();
        let message = MCPMessage::new(MessageType::Command, serde_json::json!({"test": "data"}));

        let events = vec![
            ServerEvent::ConnectionEstablished(connection_id.clone()),
            ServerEvent::MessageReceived(connection_id.clone(), message),
            ServerEvent::ConnectionClosed(connection_id.clone()),
            ServerEvent::Error(
                connection_id.clone(),
                MCPError::Transport("Test error".into()),
            ),
        ];

        assert_eq!(events.len(), 4);
    }
}
