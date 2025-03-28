use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::{mpsc, RwLock, Mutex};
use tokio_tungstenite::{connect_async, WebSocketStream, MaybeTlsStream};
use tokio_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use uuid::Uuid;
use tokio_tungstenite::tungstenite::Error as WsError;
use native_tls::TlsConnector;
use futures_util::stream::Stream;
use std::collections::HashMap;

use crate::error::transport::TransportError;
use crate::types::{MCPMessage, EncryptionFormat, CompressionFormat};
use super::{Transport, TransportMetadata};

/// Configuration for the WebSocket transport
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

/// WebSocket connection state
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

/// WebSocket transport implementation
pub struct WebSocketTransport {
    /// Transport configuration
    config: WebSocketConfig,
    
    /// Current connection state
    state: Arc<RwLock<WebSocketState>>,
    
    /// Message channels for sending/receiving
    message_receiver: Arc<Mutex<mpsc::Receiver<MCPMessage>>>,
    message_sender: mpsc::Sender<MCPMessage>,
    
    /// Socket channel sender, wrapped in Arc<Mutex> to allow updating
    socket_sender: Arc<Mutex<mpsc::Sender<SocketCommand>>>,
    
    /// Connection ID
    connection_id: String,
    
    /// Transport metadata
    metadata: TransportMetadata,
}

/// Commands for the WebSocket socket task
enum SocketCommand {
    /// Send a message
    Send(MCPMessage),
    
    /// Close the connection
    Close,
}

impl WebSocketTransport {
    /// Create a new WebSocket transport
    pub fn new(config: WebSocketConfig) -> Self {
        // Create message channels
        let (msg_tx, msg_rx) = mpsc::channel(100);
        let (socket_tx, _) = mpsc::channel(100);
        
        let metadata = TransportMetadata {
            transport_type: "websocket".to_string(),
            remote_address: config.url.clone(),
            local_address: None,
            encryption: config.encryption,
            compression: config.compression,
        };
        
        Self {
            config,
            state: Arc::new(RwLock::new(WebSocketState::Disconnected)),
            message_receiver: Arc::new(Mutex::new(msg_rx)),
            message_sender: msg_tx,
            socket_sender: Arc::new(Mutex::new(socket_tx)),
            connection_id: Uuid::new_v4().to_string(),
            metadata,
        }
    }
    
    /// Start the WebSocket task
    async fn start_websocket_task(
        &self,
        socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
        msg_tx: mpsc::Sender<MCPMessage>,
        mut socket_rx: mpsc::Receiver<SocketCommand>
    ) -> Result<(), TransportError> {
        let (mut write, mut read) = socket.split();
        let state = self.state.clone();
        
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
    async fn send_message(&self, message: MCPMessage) -> Result<(), TransportError> {
        // Check if we're connected
        {
            let state = self.state.read().await;
            if *state != WebSocketState::Connected {
                return Err(TransportError::ConnectionClosed(
                    "Transport is not connected".into()
                ));
            }
        }
        
        // Send the command to the socket task
        let socket_sender = self.socket_sender.lock().await;
        socket_sender.send(SocketCommand::Send(message)).await
            .map_err(|_| TransportError::ConnectionClosed("Failed to send message to socket task".into()))?;
        
        Ok(())
    }
    
    async fn receive_message(&self) -> Result<MCPMessage, TransportError> {
        // Check if we're connected
        {
            let state = self.state.read().await;
            if *state != WebSocketState::Connected {
                return Err(TransportError::ConnectionClosed(
                    "Transport is not connected".into()
                ));
            }
        }
        
        // Wait for a message from the receiver
        let mut receiver = self.message_receiver.lock().await;
        match receiver.recv().await {
            Some(message) => Ok(message),
            None => Err(TransportError::ConnectionClosed("Message channel closed".into())),
        }
    }
    
    async fn connect(&self) -> Result<(), TransportError> {
        // Update state to connecting
        {
            let mut state = self.state.write().await;
            *state = WebSocketState::Connecting;
        }
        
        // Connect to the WebSocket server
        let connection = connect_async(&self.config.url).await
            .map_err(|e| TransportError::ConnectionFailed(format!(
                "Failed to connect to {}: {}", 
                self.config.url, e
            )))?;
        
        let (socket, _) = connection;
        
        // Create new channels for connection
        let (socket_tx, socket_rx) = mpsc::channel::<SocketCommand>(100);
        let (msg_tx, msg_rx) = mpsc::channel::<MCPMessage>(100);
        
        // Update the socket sender and message receiver
        {
            let mut sender = self.socket_sender.lock().await;
            *sender = socket_tx;
        }
        
        {
            let mut receiver = self.message_receiver.lock().await;
            *receiver = msg_rx;
        }
        
        // Start WebSocket task
        self.start_websocket_task(socket, msg_tx, socket_rx).await?;
        
        // Update state to connected
        {
            let mut state = self.state.write().await;
            *state = WebSocketState::Connected;
        }
        
        Ok(())
    }
    
    async fn disconnect(&self) -> Result<(), TransportError> {
        // Update state to disconnecting
        {
            let mut state = self.state.write().await;
            *state = WebSocketState::Disconnecting;
        }
        
        // Send close command to socket task
        let socket_sender = self.socket_sender.lock().await;
        let _ = socket_sender.send(SocketCommand::Close).await;
        
        // Wait a bit for tasks to finish gracefully
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // Update state to disconnected (if not already done by tasks)
        {
            let mut state = self.state.write().await;
            if *state == WebSocketState::Disconnecting {
                *state = WebSocketState::Disconnected;
            }
        }
        
        Ok(())
    }
    
    async fn is_connected(&self) -> bool {
        let state = self.state.read().await;
        *state == WebSocketState::Connected
    }
    
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