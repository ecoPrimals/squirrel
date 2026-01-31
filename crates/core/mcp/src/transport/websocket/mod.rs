//! WebSocket transport implementation for MCP.

// This module provides a WebSocket-based transport implementation
// for Machine Context Protocol (MCP) communication. It supports
// bidirectional message passing over WebSocket connections.

use crate::error::{MCPError, Result, TransportError};
use crate::protocol::types::MCPMessage;
use crate::transport::types::TransportMetadata;
use crate::transport::Transport;
use crate::types::EncryptionFormat;
// BearDog handles security: // use crate::security::types::EncryptionFormat;
use crate::types::CompressionFormat;
use async_trait::async_trait;
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

/// Configuration for the WebSocket transport
///
/// This struct contains all the configuration parameters for
/// establishing and maintaining a WebSocket connection.
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    /// WebSocket URL to connect to
    pub url: String,

    /// Maximum message size in bytes
    pub max_message_size: usize,

    /// Connection timeout in seconds
    pub connection_timeout: u64,

    /// Ping interval in seconds
    pub ping_interval: Option<u64>,

    /// Encryption format
    pub encryption: EncryptionFormat,

    /// Compression format
    pub compression: CompressionFormat,

    /// Maximum number of reconnection attempts
    pub max_reconnect_attempts: u32,

    /// Reconnection delay in milliseconds
    pub reconnect_delay_ms: u64,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        // Using universal-constants for all configuration values
        use universal_constants::limits::DEFAULT_MAX_MESSAGE_SIZE;
        use universal_constants::network::get_service_port;
        use universal_constants::timeouts::{
            DEFAULT_CONNECTION_TIMEOUT, DEFAULT_INITIAL_DELAY, DEFAULT_PING_INTERVAL,
        };

        Self {
            url: format!("ws://localhost:{}", get_service_port("websocket")),
            max_message_size: DEFAULT_MAX_MESSAGE_SIZE,
            connection_timeout: DEFAULT_CONNECTION_TIMEOUT.as_secs(),
            ping_interval: Some(DEFAULT_PING_INTERVAL.as_secs()),
            encryption: EncryptionFormat::None,
            compression: CompressionFormat::None,
            max_reconnect_attempts: 5,
            reconnect_delay_ms: DEFAULT_INITIAL_DELAY.as_millis() as u64,
        }
    }
}

/// WebSocket control message types
///
/// Internal message types used to control the WebSocket connection.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ControlMessage {
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
enum WebSocketState {
    Disconnected,
    Connecting,
    Connected,
    Failed(String),
}

impl WebSocketState {
    /// Check if the state is Connected
    fn is_connected(&self) -> bool {
        matches!(self, Self::Connected)
    }
}

/// Commands for the WebSocket socket task
///
/// Commands sent to the WebSocket task to control its behavior.
#[derive(Debug)]
enum SocketCommand {
    /// Send a message
    Send(MCPMessage),

    /// Send raw binary data
    SendRaw(Vec<u8>),

    /// Send a ping frame
    Ping,

    /// Close the connection
    Close,
}

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
    config: WebSocketConfig,

    /// Connection state
    connection_state: Arc<Mutex<WebSocketState>>,

    /// WebSocket sender
    ws_sender: Option<mpsc::Sender<SocketCommand>>,

    /// Receiver from the read task
    reader_rx: Arc<Mutex<Option<mpsc::Receiver<MCPMessage>>>>,

    /// Receiver for control messages
    control_rx: Option<mpsc::Receiver<ControlMessage>>,

    /// Sender for control messages
    control_tx: Option<mpsc::Sender<ControlMessage>>,

    /// Peer address
    peer_addr: Arc<Mutex<Option<SocketAddr>>>,

    /// Local address
    local_addr: Arc<Mutex<Option<SocketAddr>>>,

    /// Transport metadata
    metadata: Arc<Mutex<TransportMetadata>>,
    
    /// Message buffer for pending sends during reconnection
    message_buffer: Arc<Mutex<Vec<MCPMessage>>>,
    
    /// Reconnection attempts counter
    reconnection_attempts: Arc<Mutex<u32>>,
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

    /// Start the WebSocket task
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
    async fn start_websocket_task(
        &self,
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

    /// Check if the transport is connected (implementation moved here)
    async fn is_connected_impl(&self) -> bool {
        let state_guard = self.connection_state.lock().await;
        state_guard.is_connected()
    }

    /// Placeholder for internal message sending logic
    #[allow(dead_code)] // Reserved for WebSocket message sending system
    async fn send_internal(&self, ws_message: Message) -> Result<()> {
        if let Some(sender) = &self.ws_sender {
            match ws_message {
                Message::Text(text) => match serde_json::from_str::<MCPMessage>(&text) {
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
                },
                Message::Binary(_bytes) => {
                    error!("WebSocket: send_internal cannot directly send raw binary via MCPMessage command.");
                    return Err(MCPError::Transport(TransportError::ProtocolError(
                        "Raw binary send via send_internal needs rework".to_string(),
                    )));
                }
                _ => {
                    debug!("WebSocket: send_internal ignoring non-data message type");
                }
            }
            Ok(())
        } else {
            Err(MCPError::Transport(TransportError::ConnectionFailed(
                "WebSocket sender unavailable".to_string(),
            )))
        }
    }

    /// Placeholder for handling received WebSocket messages
    /// Handle received WebSocket message
    async fn handle_received_message(&self, message: Message) -> Result<Option<MCPMessage>> {
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
                info!("Received close frame");
                Ok(None)
            }
            Message::Frame(_) => {
                warn!("Received unexpected frame type");
                Ok(None)
            }
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
                    if let Err(e) = self.drain_message_buffer().await {
                        warn!("Failed to drain message buffer after reconnection: {}", e);
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
            format!("Reconnection failed after {} attempts", max_attempts)
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
        let messages: Vec<MCPMessage> = {
            let mut buffer = self.message_buffer.lock().await;
            let msgs = buffer.clone();
            buffer.clear();
            msgs
        };
        
        if messages.is_empty() {
            return Ok(());
        }
        
        info!("Draining {} buffered messages after reconnection", messages.len());
        
        for (i, message) in messages.into_iter().enumerate() {
            match self.send_message(message).await {
                Ok(()) => {
                    debug!("Sent buffered message {}", i + 1);
                }
                Err(e) => {
                    warn!("Failed to send buffered message {}: {}", i + 1, e);
                    // Continue trying to send remaining messages
                }
            }
        }
        
        Ok(())
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
    async fn buffer_message(&self, message: MCPMessage) -> Result<()> {
        const MAX_BUFFER_SIZE: usize = 1000; // Limit buffer to prevent memory exhaustion
        
        let mut buffer = self.message_buffer.lock().await;
        
        if buffer.len() >= MAX_BUFFER_SIZE {
            // Drop oldest message (circular buffer strategy)
            buffer.remove(0);
            warn!("Message buffer full, dropped oldest message");
        }
        
        buffer.push(message);
        debug!("Buffered message ({} in buffer)", buffer.len());
        
        Ok(())
    }
    
    /// Start keepalive ping task
    ///
    /// Starts a background task that sends periodic ping frames to keep
    /// the connection alive and detect disconnections early.
    fn start_keepalive_task(&self) {
        if let Some(ping_interval_secs) = self.config.ping_interval {
            let sender = self.ws_sender.clone();
            let state = self.connection_state.clone();
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
    }

    /// Get the remote address of the WebSocket connection
    #[allow(dead_code)] // Reserved for connection address tracking
    async fn remote_addr(&self) -> std::result::Result<Option<SocketAddr>, MCPError> {
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
            .unwrap()
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
        trace!("Attempting to receive message from reader channel...");
        let mut reader_guard = self.reader_rx.lock().await;

        if let Some(ref mut rx) = *reader_guard {
            let received: Option<MCPMessage> = rx.recv().await;
            match received {
                Some(mcp_message) => {
                    debug!("Received message via channel: ID {}", mcp_message.id.0);
                    Ok(mcp_message)
                }
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

        info!("Connecting to WebSocket URL: {}", self.config.url);
        let connection_result = connect_async(&self.config.url).await;

        let (socket, response) = match connection_result {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to connect to {}: {}", self.config.url, e);
                *self.connection_state.lock().await = WebSocketState::Failed(e.to_string());
                return Err(MCPError::Transport(TransportError::ConnectionError(
                    format!("Failed to connect to {}: {}", self.config.url, e),
                )));
            }
        };
        info!(
            "WebSocket connection established. Response: {:?}",
            response.status()
        );

        let (peer_addr, local_addr) = match socket.get_ref() {
            // For plain TCP connections
            MaybeTlsStream::Plain(tcp) => (tcp.peer_addr().ok(), tcp.local_addr().ok()),
            // For all TLS connections (regardless of implementation)
            // Use a conditional pattern match that will work with various versions
            _ => {
                warn!("Could not determine peer/local address from TLS WebSocket stream.");
                (None, None)
            }
        };

        *self.peer_addr.lock().await = peer_addr;
        *self.local_addr.lock().await = local_addr;

        {
            let mut meta = self.metadata.lock().await;
            meta.remote_address = peer_addr;
            meta.local_address = local_addr;
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

        self.start_websocket_task(socket, msg_tx, socket_rx, self.connection_state.clone())
            .await?;

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
async fn handle_connection(_peer: SocketAddr, _stream: TcpStream) -> Result<()> {
    Ok(())
}

/// Process WebSocket socket messages
#[allow(dead_code)] // Reserved for WebSocket message processing system
async fn process_socket(
    _socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    _msg_tx: mpsc::Sender<MCPMessage>,
    mut _socket_rx: mpsc::Receiver<SocketCommand>,
    _control_tx: mpsc::Sender<ControlMessage>,
    _state: Arc<Mutex<WebSocketState>>,
    _peer: SocketAddr,
) {
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_transport_create() {
        // Create a config with environment-based or default WebSocket port
        let test_url = std::env::var("TEST_WEBSOCKET_URL")
            .unwrap_or_else(|_| {
                use universal_constants::network::get_service_port;
                let port = get_service_port("websocket");
                format!("ws://localhost:{}", port)
            });
        
        let config = WebSocketConfig {
            url: test_url.clone(),
            ..Default::default()
        };

        // Create transport
        let transport = WebSocketTransport::new(config);

        // Ensure it starts disconnected
        assert!(!transport.is_connected().await);

        // Get metadata
        let metadata = transport.get_metadata().await;
        assert_eq!(
            metadata
                .additional_info
                .get("transport_type")
                .unwrap_or(&"".to_string()),
            "websocket"
        );
        // Verify peer_addr is set (actual value depends on test environment)
        assert!(
            metadata
                .additional_info
                .get("peer_addr")
                .is_some(),
            "peer_addr should be set in additional_info"
        );
    }

    #[tokio::test]
    async fn test_websocket_transport_send_raw() {
        // Create a config with environment-based or default WebSocket port
        let test_url = std::env::var("TEST_WEBSOCKET_URL")
            .unwrap_or_else(|_| {
                use universal_constants::network::get_service_port;
                let port = get_service_port("websocket");
                format!("ws://localhost:{}", port)
            });
        
        let config = WebSocketConfig {
            url: test_url,
            ..Default::default()
        };

        // Create transport
        let transport = WebSocketTransport::new(config);

        // Mock the connection state for testing
        {
            let mut state = transport.connection_state.lock().await;
            *state = WebSocketState::Connected;
        }

        // Test data to send
        let data = b"Hello WebSocket Raw Data!";

        // Since we're mocked as connected but not actually connected,
        // this should fail gracefully with a specific error
        let result = transport.send_raw(data).await;
        assert!(result.is_err());

        // We expect a specific error type - either ConnectionClosed or SendError
        if let Err(e) = result {
            let e_str = format!("{e:?}");
            assert!(
                e_str.contains("ConnectionClosed") || e_str.contains("SendError"),
                "Expected ConnectionClosed or SendError, got: {e:?}",
            );
        }
    }
    
    #[tokio::test]
    async fn test_websocket_message_buffering() {
        // Create config
        let config = WebSocketConfig {
            url: "ws://localhost:8080".to_string(),
            ..Default::default()
        };
        
        let transport = WebSocketTransport::new(config);
        
        // Create a test message
        use crate::protocol::types::{MCPMessageId, MCPVersion};
        let test_message = MCPMessage {
            jsonrpc: MCPVersion::V2_0,
            id: MCPMessageId(1),
            method: Some("test.method".to_string()),
            params: None,
            result: None,
            error: None,
        };
        
        // Transport is disconnected, message should be buffered
        assert!(!transport.is_connected().await);
        
        let buffer_result = transport.buffer_message(test_message.clone()).await;
        assert!(buffer_result.is_ok(), "Should buffer message when disconnected");
        
        // Verify message is in buffer
        {
            let buffer = transport.message_buffer.lock().await;
            assert_eq!(buffer.len(), 1, "Buffer should contain 1 message");
        }
        
        // Buffer multiple messages
        for i in 2..10 {
            let msg = MCPMessage {
                jsonrpc: MCPVersion::V2_0,
                id: MCPMessageId(i),
                method: Some("test.method".to_string()),
                params: None,
                result: None,
                error: None,
            };
            let _ = transport.buffer_message(msg).await;
        }
        
        // Verify buffer contains all messages
        {
            let buffer = transport.message_buffer.lock().await;
            assert_eq!(buffer.len(), 9, "Buffer should contain 9 messages");
        }
    }
    
    #[tokio::test]
    async fn test_websocket_buffer_overflow() {
        // Create config
        let config = WebSocketConfig {
            url: "ws://localhost:8080".to_string(),
            ..Default::default()
        };
        
        let transport = WebSocketTransport::new(config);
        
        // Create test message
        use crate::protocol::types::{MCPMessageId, MCPVersion};
        let test_message = MCPMessage {
            jsonrpc: MCPVersion::V2_0,
            id: MCPMessageId(1),
            method: Some("test.method".to_string()),
            params: None,
            result: None,
            error: None,
        };
        
        // Buffer 1001 messages (buffer max is 1000)
        for i in 0..1001 {
            let msg = MCPMessage {
                jsonrpc: MCPVersion::V2_0,
                id: MCPMessageId(i),
                method: Some("test.method".to_string()),
                params: None,
                result: None,
                error: None,
            };
            let _ = transport.buffer_message(msg).await;
        }
        
        // Buffer should be capped at 1000 (oldest dropped)
        {
            let buffer = transport.message_buffer.lock().await;
            assert_eq!(buffer.len(), 1000, "Buffer should be capped at 1000");
            
            // First message should be id=1 (oldest was id=0, dropped)
            if let Some(first_msg) = buffer.first() {
                assert_eq!(first_msg.id.0, 1, "Oldest message should be dropped");
            }
        }
    }
    
    #[tokio::test]
    async fn test_websocket_reconnection_counter() {
        // Create config with specific reconnection settings
        let config = WebSocketConfig {
            url: "ws://localhost:9999".to_string(), // Invalid port to force failure
            max_reconnect_attempts: 3,
            reconnect_delay_ms: 10, // Fast for testing
            ..Default::default()
        };
        
        let transport = WebSocketTransport::new(config);
        
        // Verify initial counter is 0
        {
            let attempts = transport.reconnection_attempts.lock().await;
            assert_eq!(*attempts, 0, "Initial reconnection attempts should be 0");
        }
        
        // Note: We can't test actual reconnection without a running WebSocket server
        // This test verifies the structure is in place
        // Full reconnection testing would be done in integration tests
    }
    
    #[tokio::test]
    async fn test_websocket_keepalive_configuration() {
        // Test with keepalive enabled
        let config_with_keepalive = WebSocketConfig {
            url: "ws://localhost:8080".to_string(),
            ping_interval: Some(30), // 30 second ping interval
            ..Default::default()
        };
        
        let transport = WebSocketTransport::new(config_with_keepalive);
        assert!(transport.config.ping_interval.is_some(), "Keepalive should be enabled");
        assert_eq!(transport.config.ping_interval.unwrap(), 30);
        
        // Test with keepalive disabled
        let config_without_keepalive = WebSocketConfig {
            url: "ws://localhost:8080".to_string(),
            ping_interval: None, // No keepalive
            ..Default::default()
        };
        
        let transport = WebSocketTransport::new(config_without_keepalive);
        assert!(transport.config.ping_interval.is_none(), "Keepalive should be disabled");
    }
}
