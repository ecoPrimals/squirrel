//! WebSocket transport implementation for MCP.

// This module provides a WebSocket-based transport implementation
// for Machine Context Protocol (MCP) communication. It supports
// bidirectional message passing over WebSocket connections.

use crate::error::{Result, MCPError, TransportError};
use crate::transport::types::TransportMetadata;
use crate::transport::{Transport};
use crate::protocol::MCPMessage;
use crate::security::types::EncryptionFormat;
use crate::types::CompressionFormat;
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};
use tracing::{debug, warn, error, info, trace};
use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;

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
        Self {
            url: "ws://localhost:9001".to_string(),
            max_message_size: 10 * 1024 * 1024, // 10MB
            connection_timeout: 30,
            ping_interval: Some(30),
            encryption: EncryptionFormat::None,
            compression: CompressionFormat::None,
            max_reconnect_attempts: 5,
            reconnect_delay_ms: 1000,
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
    
    /// Close the connection
    Close,
}

/// WebSocket transport for MCP communication
///
/// This implementation provides WebSocket-based transport for MCP messages.
/// It handles connection establishment, message sending/receiving, and
/// connection cleanup.
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
    #[must_use] pub fn new(config: WebSocketConfig) -> Self {
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
                    },
                    SocketCommand::SendRaw(bytes) => {
                        // Send as binary message
                        if let Err(e) = write.send(Message::Binary(bytes)).await {
                            error!("WebSocket: Failed to send raw bytes: {}", e);
                            break;
                        }
                    },
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
    async fn send_internal(&self, ws_message: Message) -> Result<()> {
        if let Some(sender) = &self.ws_sender {
            match ws_message {
                Message::Text(text) => {
                    match serde_json::from_str::<MCPMessage>(&text) {
                        Ok(mcp_message) => {
                            if sender.send(SocketCommand::Send(mcp_message)).await.is_err() {
                                error!("WebSocket: Failed to send command to writer task");
                                return Err(MCPError::Transport(TransportError::ConnectionClosed("Writer task channel closed".to_string())).into());
                            }
                        }
                        Err(e) => {
                            error!("WebSocket: Failed to deserialize text message for sending: {}", e);
                            return Err(MCPError::Serialization(e.to_string()).into());
                        }
                    }
                }
                Message::Binary(_bytes) => {
                     error!("WebSocket: send_internal cannot directly send raw binary via MCPMessage command.");
                     return Err(MCPError::Transport(TransportError::ProtocolError("Raw binary send via send_internal needs rework".to_string())).into());
                }
                _ => {
                    debug!("WebSocket: send_internal ignoring non-data message type");
                }
            }
            Ok(())
        } else {
            Err(MCPError::Transport(TransportError::ConnectionFailed("WebSocket sender unavailable".to_string())).into())
        }
    }

    /// Placeholder for handling received WebSocket messages
    async fn handle_received_message(&self, _message: Message) -> Result<Option<MCPMessage>> {
        // TODO: Implement deserialization and handling of Ping/Pong/Close/Binary/Text
        Ok(None)
    }
}

#[async_trait]
impl Transport for WebSocketTransport {
    /// Send a message over the WebSocket transport
    ///
    /// Sends an MCP message over the established WebSocket connection.
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
            return Err(MCPError::Transport(TransportError::ConnectionClosed("Cannot send message, not connected".to_string())).into());
        }

        // Send the message to the write task through the channel
        match self.ws_sender.as_ref().unwrap().send(SocketCommand::Send(message)).await {
            Ok(_) => Ok(()),
            Err(e) => Err(MCPError::Transport(TransportError::SendError(e.to_string())).into())
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
            return Err(MCPError::Transport(TransportError::ConnectionClosed("Cannot receive message, not connected".to_string())).into());
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
                    Err(MCPError::Transport(TransportError::ConnectionClosed("Reader channel closed".to_string())).into())
                }
            }
        } else {
            error!("Reader channel (reader_rx) is None. Cannot receive message.");
            Err(MCPError::Transport(TransportError::ConnectionClosed("Reader channel unavailable".to_string())).into())
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
                warn!("WebSocket connect called while not disconnected ({:?})", *state);
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
                return Err(MCPError::Transport(TransportError::ConnectionError(format!("Failed to connect to {}: {}", 
                    self.config.url, e))).into());
            }
        };
        info!("WebSocket connection established. Response: {:?}", response.status());
        
        let (peer_addr, local_addr) = match socket.get_ref() {
            #[cfg(feature = "native-tls")] // Keep native-tls handling if feature enabled
            MaybeTlsStream::NativeTls(tls) => (
                tls.get_ref().get_ref().peer_addr().ok(), 
                tls.get_ref().get_ref().local_addr().ok()
            ),
            // Prioritize rustls if enabled
            #[cfg(feature = "rustls-tls-native-roots")] // Or rustls-tls-webpki-roots
            MaybeTlsStream::Rustls(tls) => {
                // Access the underlying TcpStream through the appropriate path for rustls
                // This might involve accessing private fields or using methods if available
                // Assuming `tls.get_ref().0` provides access to the underlying stream structure
                // Adjust based on the actual structure provided by tokio-rustls
                 match tls.get_ref() {
                     // Assuming the underlying stream is accessible via .0 or similar
                     // You might need to inspect the `tokio_rustls::client::TlsStream` structure
                     // For example: `tls.get_ref().io.peer_addr()` if `io` holds the TcpStream
                     // If direct access isn't possible, alternative methods might be needed.
                     // Placeholder: Adapt this based on actual `tokio-rustls` API
                     _ => (None, None) // Default if specific access pattern unclear
                     // Example if underlying TCP is directly accessible:
                     // (tls.get_ref().0.peer_addr().ok(), tls.get_ref().0.local_addr().ok())
                 }
            },
            MaybeTlsStream::Plain(tcp) => (
                tcp.peer_addr().ok(),
                tcp.local_addr().ok()
            ),
            // Handle unexpected variants or cases where no TLS feature is enabled
            _ => {
                 warn!("Could not determine peer/local address from WebSocket stream type.");
                 (None, None)
            },
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
        
        self.start_websocket_task(socket, msg_tx, socket_rx, self.connection_state.clone()).await?;
        
        {
            let mut state = self.connection_state.lock().await;
             if *state == WebSocketState::Connecting {
                *state = WebSocketState::Connected;
                info!("WebSocket transport connected successfully.");
            } else {
                warn!("WebSocket state changed during connection ({:?}), not setting to Connected.", *state);
            }
        }
        
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
            return Err(MCPError::Transport(TransportError::ConnectionClosed("Cannot send raw bytes, not connected".to_string())).into());
        }
        
        if let Some(sender) = &self.ws_sender {
            let cmd = SocketCommand::SendRaw(bytes.to_vec());
            if sender.send(cmd).await.is_err() {
                error!("WebSocket: Failed to send raw bytes command to writer task");
                return Err(MCPError::Transport(TransportError::SendError("Writer task channel closed".to_string())).into());
            }
            Ok(())
        } else {
            error!("WebSocket: No sender available for raw bytes");
            Err(MCPError::Transport(TransportError::ConnectionFailed("WebSocket sender unavailable".to_string())).into())
        }
    }
}

async fn handle_connection(
    _peer: SocketAddr,
    _stream: TcpStream,
) -> Result<()> {
    Ok(())
}

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
        // Create a config
        let config = WebSocketConfig {
            url: "ws://localhost:9001".to_string(),
            ..Default::default()
        };
        
        // Create transport
        let transport = WebSocketTransport::new(config);
        
        // Ensure it starts disconnected
        assert!(!transport.is_connected().await);
        
        // Get metadata
        let metadata = transport.get_metadata().await;
        assert_eq!(metadata.additional_info.get("transport_type").unwrap_or(&"".to_string()), "websocket");
        assert_eq!(metadata.additional_info.get("peer_addr").unwrap_or(&"".to_string()), "ws://localhost:9001");
    }
    
    #[tokio::test]
    async fn test_websocket_transport_send_raw() {
        // Create a config
        let config = WebSocketConfig {
            url: "ws://localhost:9001".to_string(),
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
            let e_str = format!("{:?}", e);
            assert!(e_str.contains("ConnectionClosed") || e_str.contains("SendError"), 
                    "Expected ConnectionClosed or SendError, got: {:?}", e);
        }
    }
} 