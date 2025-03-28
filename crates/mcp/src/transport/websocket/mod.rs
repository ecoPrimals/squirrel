// WebSocket transport implementation for MCP
//
// This module provides a WebSocket-based transport implementation
// for Machine Context Protocol (MCP) communication. It supports
// bidirectional message passing over WebSocket connections.

use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, WebSocketStream, MaybeTlsStream};
use tokio_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;

use crate::error::transport::TransportError;
use crate::types::{MCPMessage, EncryptionFormat, CompressionFormat};
use super::{Transport, TransportMetadata};

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

/// State of the WebSocket connection
///
/// Represents the current state of the WebSocket connection.
#[derive(Debug, Clone, PartialEq, Eq)]
enum WebSocketState {
    /// Not connected
    Disconnected,
    
    /// In the process of connecting
    Connecting,
    
    /// Connected and ready to send/receive
    Connected,
    
    /// In the process of disconnecting
    Disconnecting,
    
    /// Connection has failed
    Failed(String),
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
    connection_state: Arc<tokio::sync::RwLock<WebSocketState>>,
    
    /// WebSocket sender
    ws_sender: Option<mpsc::Sender<SocketCommand>>,
    
    /// Receiver from the read task
    reader_rx: Arc<Mutex<Option<mpsc::Receiver<MCPMessage>>>>,
    
    /// Receiver for control messages
    control_rx: Option<mpsc::Receiver<ControlMessage>>,
    
    /// Sender for control messages
    control_tx: Option<mpsc::Sender<ControlMessage>>,
    
    /// Transport metadata
    metadata: TransportMetadata,
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
    /// A new WebSocketTransport instance
    pub fn new(config: WebSocketConfig) -> Self {
        // Create message channels
        let (_msg_tx, msg_rx) = mpsc::channel(100);
        let (socket_tx, _socket_rx) = mpsc::channel(100);
        let (control_tx, control_rx) = mpsc::channel(100);
        
        let metadata = TransportMetadata {
            transport_type: "websocket".to_string(),
            remote_address: config.url.clone(),
            local_address: None,
            encryption: config.encryption,
            compression: config.compression,
        };
        
        Self {
            config,
            connection_state: Arc::new(tokio::sync::RwLock::new(WebSocketState::Disconnected)),
            ws_sender: Some(socket_tx),
            reader_rx: Arc::new(Mutex::new(Some(msg_rx))),
            control_rx: Some(control_rx),
            control_tx: Some(control_tx),
            metadata,
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
    ) -> Result<(), TransportError> {
        let (mut write, mut read) = socket.split();
        let state = self.connection_state.clone();
        
        // Clone for the reader task
        let read_state = state.clone();
        let read_msg_tx = msg_tx.clone();
        
        // Start reader task
        tokio::spawn(async move {
            while let Some(result) = read.next().await {
                match result {
                    Ok(Message::Text(text)) => {
                        // Parse as JSON
                        match serde_json::from_str::<MCPMessage>(&text) {
                            Ok(message) => {
                                if let Err(e) = read_msg_tx.send(message).await {
                                    eprintln!("Failed to forward message to channel: {}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to parse message: {}", e);
                                continue;
                            }
                        }
                    }
                    Ok(Message::Binary(bin)) => {
                        // Parse as binary JSON
                        match serde_json::from_slice::<MCPMessage>(&bin) {
                            Ok(message) => {
                                if let Err(e) = read_msg_tx.send(message).await {
                                    eprintln!("Failed to forward message to channel: {}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to parse binary message: {}", e);
                                continue;
                            }
                        }
                    }
                    Ok(Message::Ping(_)) => {
                        // Respond to ping (handled automatically by tungstenite)
                    }
                    Ok(Message::Pong(_)) => {
                        // Pong response (could update last activity timestamp)
                    }
                    Ok(Message::Close(_)) => {
                        // Connection closed by the server
                        break;
                    }
                    Ok(Message::Frame(_)) => {
                        // Ignore frame messages
                        continue;
                    }
                    Err(e) => {
                        // Error reading from socket
                        eprintln!("Error reading from WebSocket: {}", e);
                        break;
                    }
                }
            }
            
            // Update state to disconnected
            let mut current_state = read_state.write().await;
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
                                eprintln!("Failed to serialize message: {}", e);
                                continue;
                            }
                        };
                        
                        // Send as text message
                        if let Err(e) = write.send(Message::Text(json)).await {
                            eprintln!("Failed to send message: {}", e);
                            break;
                        }
                    }
                    SocketCommand::Close => {
                        // Close the connection gracefully
                        if let Err(e) = write.close().await {
                            eprintln!("Error closing WebSocket: {}", e);
                        }
                        break;
                    }
                }
            }
            
            // Update state to disconnected
            let mut current_state = state.write().await;
            *current_state = WebSocketState::Disconnected;
        });
        
        Ok(())
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
    async fn send_message(&self, message: MCPMessage) -> Result<(), TransportError> {
        // Check if we're connected
        {
            let state = self.connection_state.read().await;
            if *state != WebSocketState::Connected {
                return Err(TransportError::ConnectionClosed(
                    "Transport is not connected".into()
                ));
            }
        }
        
        // Send the command to the socket task
        if let Some(sender) = &self.ws_sender {
            sender.send(SocketCommand::Send(message)).await
                .map_err(|_| TransportError::ConnectionClosed("Failed to send message to socket task".into()))?;
        } else {
            return Err(TransportError::ConnectionClosed("Socket sender is not initialized".into()));
        }
        
        Ok(())
    }
    
    /// Receive a message from the WebSocket transport
    ///
    /// Waits for and receives the next MCP message from the WebSocket connection.
    ///
    /// # Returns
    ///
    /// Result containing the received message or an error
    async fn receive_message(&self) -> Result<MCPMessage, TransportError> {
        // Check if we're connected
        {
            let state = self.connection_state.read().await;
            if *state != WebSocketState::Connected {
                return Err(TransportError::ConnectionClosed(
                    "Transport is not connected".into()
                ));
            }
        }
        
        // Wait for a message from the receiver
        let mut receiver_guard = self.reader_rx.lock().await;
        if let Some(receiver) = &mut *receiver_guard {
            match receiver.recv().await {
                Some(message) => Ok(message),
                None => Err(TransportError::ConnectionClosed("Message channel closed".into())),
            }
        } else {
            Err(TransportError::ConnectionClosed("Reader channel is not initialized".into()))
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
    async fn connect(&mut self) -> Result<(), TransportError> {
        // Update state to connecting
        {
            let mut state = self.connection_state.write().await;
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
            let mut state = self.connection_state.write().await;
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
    async fn disconnect(&self) -> Result<(), TransportError> {
        // Update state to disconnecting
        {
            let mut state = self.connection_state.write().await;
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
            let mut state = self.connection_state.write().await;
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
        let state = self.connection_state.read().await;
        *state == WebSocketState::Connected
    }
    
    /// Get transport metadata
    ///
    /// # Returns
    ///
    /// Metadata about this transport connection
    fn get_metadata(&self) -> TransportMetadata {
        self.metadata.clone()
    }
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
        assert_eq!(metadata.remote_address, "ws://localhost:9001");
    }
} 