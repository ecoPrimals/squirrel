//! WebSocket transport implementation for MCP.

// This module provides a WebSocket-based transport implementation
// for Machine Context Protocol (MCP) communication. It supports
// bidirectional message passing over WebSocket connections.

use crate::error::{Result, TransportError, MCPError};
use crate::transport::{Transport, TransportMetadata};
use crate::transport::types::ConnectionState;
use crate::protocol::MCPMessage;
use crate::security::types::EncryptionFormat;
use crate::types::CompressionFormat;
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt, stream::{SplitSink, SplitStream}};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};
use tracing::{debug, warn, error, info};
use log;
use std::ops::DerefMut;
use crate::message::Message;
use crate::transport::frame::{MessageCodec};

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
        
        let metadata = TransportMetadata {
            transport_type: "websocket".to_string(),
            peer_addr: None,
            local_addr: None,
            encryption: config.encryption,
            compression: config.compression,
            connected_at: chrono::Utc::now(),
            state: ConnectionState::Disconnected,
            protocol_version: "unknown".to_string(),
            additional_metadata: Default::default(),
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
            metadata: Arc::new(Mutex::new(metadata)),
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
    ///
    /// # Returns
    ///
    /// Result indicating success or error
    async fn start_websocket_task(
        &self,
        socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
        msg_tx: mpsc::Sender<MCPMessage>,
        mut socket_rx: mpsc::Receiver<SocketCommand>
    ) -> Result<()> {
        let (mut write, mut read) = socket.split();
        let state = self.connection_state.clone();
        
        // Clone for the reader task
        let read_state = state.clone();
        let read_msg_tx = msg_tx;
        
        // Start reader task
        tokio::spawn(async move {
            while let Some(result) = read.next().await {
                match result {
                    Ok(Message::Text(text)) => {
                        // Parse as JSON
                        match serde_json::from_str::<MCPMessage>(&text) {
                            Ok(message) => {
                                if let Err(e) = read_msg_tx.send(message).await {
                                    eprintln!("Failed to forward message to channel: {e}");
                                    break;
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to parse message: {e}");
                                continue;
                            }
                        }
                    }
                    Ok(Message::Binary(bin)) => {
                        // Parse as binary JSON
                        match serde_json::from_slice::<MCPMessage>(&bin) {
                            Ok(message) => {
                                if let Err(e) = read_msg_tx.send(message).await {
                                    eprintln!("Failed to forward message to channel: {e}");
                                    break;
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to parse binary message: {e}");
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
                        break;
                    }
                    Ok(Message::Frame(_)) => {
                        // Handle unexpected frame types if necessary
                        warn!("Received unexpected frame type");
                    }
                    Err(e) => {
                        // Error reading from socket
                        eprintln!("Error reading from WebSocket: {e}");
                        break;
                    }
                }
            }
            
            // Update state to disconnected
            let mut current_state = read_state.lock().await;
            *current_state = WebSocketState::Disconnected;
        });
        
        // Start writer task
        tokio::spawn(async move {
            while let Some(cmd) = socket_rx.recv().await {
                match cmd {
                    SocketCommand::Send(message) => {
                        // Serialize to JSON
                        let json = match serde_json::to_string(&message) {
                            Ok(j) => j,
                            Err(e) => {
                                eprintln!("Failed to serialize message: {e}");
                                continue;
                            }
                        };
                        
                        // Send as text message
                        if let Err(e) = write.send(Message::Text(json)).await {
                            eprintln!("Failed to send message: {e}");
                            break;
                        }
                    }
                    SocketCommand::Close => {
                        // Close the connection gracefully
                        if let Err(e) = write.close().await {
                            eprintln!("Error closing WebSocket: {e}");
                        }
                        break;
                    }
                }
            }
            
            // Update state to disconnected
            let mut current_state = state.lock().await;
            *current_state = WebSocketState::Disconnected;
        });
        
        Ok(())
    }

    /// Check if the transport is connected (implementation moved here)
    async fn is_connected_impl(&self) -> bool {
        let state_guard = self.connection_state.lock().await;
        state_guard.is_connected()
    }

    /// Placeholder for internal message sending logic
    async fn send_internal(&self, _message: Message) -> Result<()> {
        // TODO: Implement actual sending via ws_sender channel to the background task
        Ok(())
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
        // Serialize the message
        let serialized_message = match serde_json::to_string(&message) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to serialize MCPMessage: {}", e);
                return Err(MCPError::Serialization(e.to_string()));
            }
        };

        let ws_message = Message::Text(serialized_message);

        self.send_internal(ws_message).await
    }
    
    /// Receive a message from the WebSocket transport
    ///
    /// Waits for and receives the next MCP message from the WebSocket connection.
    ///
    /// # Returns
    ///
    /// Result containing the received message or an error
    async fn receive_message(&self) -> Result<MCPMessage> {
        log::trace!("Attempting to receive message from reader channel...");
        let mut reader_guard = self.reader_rx.lock().await;
        
        if let Some(ref mut rx) = *reader_guard {
            match rx.recv().await {
                Some(mcp_message) => {
                    log::debug!("Received message via channel: ID {}", mcp_message.id.0);
                    Ok(mcp_message)
                }
                None => {
                    log::warn!("Reader channel closed while trying to receive message.");
                     // Update state to reflect potential disconnection
                     let mut state_guard = self.connection_state.lock().await;
                     if *state_guard == WebSocketState::Connected {
                         *state_guard = WebSocketState::Disconnected; // Assume disconnected if channel closed
                     }
                    Err(TransportError::ConnectionClosed("Reader channel closed".to_string()).into())
                }
            }
        } else {
            log::error!("Reader channel (reader_rx) is None. Cannot receive message.");
            Err(TransportError::NotConnected("Reader channel unavailable".to_string()).into())
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
        // Update state to connecting
        {
            let mut state = self.connection_state.lock().await;
            *state = WebSocketState::Connecting;
        }
        
        // Connect to the WebSocket server
        let connection = connect_async(&self.config.url).await
            .map_err(|e| TransportError::ConnectionFailed(format!(
                "Failed to connect to {}: {}", 
                self.config.url, e
            )))?;
        
        let (socket, _) = connection;
        
        // Create new channels for the connection
        let (msg_tx, msg_rx) = mpsc::channel::<MCPMessage>(100);
        let (socket_tx, socket_rx) = mpsc::channel::<SocketCommand>(100);
        
        // Update the socket sender
        self.ws_sender = Some(socket_tx);
        
        // Update the reader
        {
            let mut reader_guard = self.reader_rx.lock().await;
            *reader_guard = Some(msg_rx);
        }
        
        // Start WebSocket task
        self.start_websocket_task(socket, msg_tx, socket_rx).await?;
        
        // Update state to connected
        {
            let mut state = self.connection_state.lock().await;
            *state = WebSocketState::Connected;
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
        // Update state to disconnecting
        {
            let mut state = self.connection_state.lock().await;
            *state = WebSocketState::Disconnecting;
        }
        
        // Send close command to socket task if it exists
        if let Some(sender) = &self.ws_sender {
            let _ = sender.send(SocketCommand::Close).await;
        }
        
        // Wait a bit for tasks to finish gracefully
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // Reset the reader channel
        {
            let mut reader_guard = self.reader_rx.lock().await;
            *reader_guard = None;
        }
        
        // Update state to disconnected (if not already done by tasks)
        {
            let mut state = self.connection_state.lock().await;
            if *state == WebSocketState::Disconnecting {
                *state = WebSocketState::Disconnected;
            }
        }
        
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
    fn get_metadata(&self) -> TransportMetadata {
        async_std::task::block_on(async { self.metadata.lock().await.clone() })
    }
}

async fn handle_connection(
    peer: SocketAddr,
    stream: TcpStream,
    // ... existing code ...
) -> Result<()> {
    // ... existing code ...
    Ok(())
}

async fn process_socket(
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    msg_tx: mpsc::Sender<MCPMessage>,
    mut socket_rx: mpsc::Receiver<SocketCommand>,
    control_tx: mpsc::Sender<ControlMessage>,
    state: Arc<Mutex<WebSocketState>>,
    peer: SocketAddr,
) {
    // ... existing code ...
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
        let metadata = transport.get_metadata();
        assert_eq!(metadata.transport_type, "websocket");
        assert_eq!(metadata.peer_addr, "ws://localhost:9001");
    }
} 