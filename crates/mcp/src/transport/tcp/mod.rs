use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::{mpsc, RwLock, Mutex};
use tokio::net::{TcpStream, TcpListener};
use tokio::io::{split, AsyncWriteExt};
use uuid::Uuid;
use std::collections::HashMap;
use socket2;
use tracing::{debug, error, info, trace, warn};
use crate::error::{MCPError, Result as MCPResult};

use crate::error::transport::TransportError;
use crate::types::{MCPMessage, EncryptionFormat, CompressionFormat};
use super::{Transport, TransportMetadata};
use super::frame::{FrameReader, FrameWriter, MessageCodec};

/// Configuration for the TCP transport
#[derive(Debug, Clone)]
pub struct TcpTransportConfig {
    /// Remote address to connect to (client mode) or bind address (server mode)
    pub remote_address: String,
    
    /// Local bind address for client connections
    pub local_bind_address: Option<String>,
    
    /// Max message size in bytes
    pub max_message_size: usize,
    
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    
    /// Keep alive interval in seconds
    pub keep_alive_interval: Option<u64>,
    
    /// Encryption format
    pub encryption: EncryptionFormat,
    
    /// Compression format
    pub compression: CompressionFormat,
    
    /// Maximum number of reconnection attempts
    pub max_reconnect_attempts: u32,
    
    /// Reconnection delay in milliseconds
    pub reconnect_delay_ms: u64,
}

impl Default for TcpTransportConfig {
    fn default() -> Self {
        Self {
            remote_address: "127.0.0.1:9000".to_string(),
            local_bind_address: None,
            max_message_size: 10 * 1024 * 1024, // 10MB
            connection_timeout: 30,
            keep_alive_interval: Some(60),
            encryption: EncryptionFormat::None,
            compression: CompressionFormat::None,
            max_reconnect_attempts: 5,
            reconnect_delay_ms: 1000,
        }
    }
}

/// TCP transport connection state
#[derive(Debug, Clone, PartialEq, Eq)]
enum TcpTransportState {
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

/// TCP transport implementation
#[derive(Clone)]
pub struct TcpTransport {
    /// Transport configuration
    config: TcpTransportConfig,
    
    /// Current connection state
    state: Arc<RwLock<TcpTransportState>>,
    
    /// Message channel for sending
    message_sender: Arc<Mutex<mpsc::Sender<MCPMessage>>>,
    
    /// Frame channel for incoming frames
    frame_receiver: Arc<Mutex<Option<mpsc::Receiver<MCPMessage>>>>,
    
    /// Connection ID
    connection_id: String,
    
    /// Transport metadata
    metadata: TransportMetadata,
}

impl TcpTransport {
    /// Create a new TCP transport
    pub fn new(config: TcpTransportConfig) -> Self {
        // Create message channels
        let (msg_tx, _) = mpsc::channel(100);
        
        let metadata = TransportMetadata {
            transport_type: "tcp".to_string(),
            remote_address: config.remote_address.clone(),
            local_address: config.local_bind_address.clone(),
            encryption: config.encryption,
            compression: config.compression,
        };
        
        Self {
            config,
            state: Arc::new(RwLock::new(TcpTransportState::Disconnected)),
            message_sender: Arc::new(Mutex::new(msg_tx)),
            frame_receiver: Arc::new(Mutex::new(None)),
            connection_id: Uuid::new_v4().to_string(),
            metadata,
        }
    }
    
    /// Start the reader task to process incoming messages
    async fn start_reader_task<R>(&self, reader: R) -> Result<(), TransportError> 
    where 
        R: tokio::io::AsyncRead + Unpin + Send + 'static 
    {
        let addr = self.config.remote_address.clone();
        if addr.is_empty() {
            return Err(TransportError::ConnectionFailed(
                "TCP transport requires a remote address".into()
            ));
        }
        
        // Generate a unique stream ID
        let _stream_id = Uuid::new_v4().to_string();
        
        let state = self.state.clone();
        
        // Create a channel for frames
        let (frame_tx, frame_rx) = mpsc::channel::<MCPMessage>(100);
        
        // Lock the frame_receiver and replace it with our receiver
        let mut frame_receiver = self.frame_receiver.lock().await;
        *frame_receiver = Some(frame_rx);
        
        tokio::spawn(async move {
            let mut reader = FrameReader::new(reader);
            let codec = MessageCodec::new();
            
            'reader_loop: loop {
                // Check if we should continue running
                {
                    let current_state = state.read().await;
                    if *current_state != TcpTransportState::Connected {
                        break 'reader_loop;
                    }
                }
                
                // Read a frame
                let frame_result = reader.read_frame().await;
                
                match frame_result {
                    Ok(Some(frame)) => {
                        // Decode the frame to a message
                        match codec.decode_message(&frame).await {
                            Ok(message) => {
                                // Forward the message to the channel
                                if let Err(e) = frame_tx.send(message).await {
                                    eprintln!("Failed to forward message: {}", e);
                                    break 'reader_loop;
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to decode frame: {}", e);
                                continue 'reader_loop;
                            }
                        }
                    }
                    Ok(None) => {
                        // End of stream
                        break 'reader_loop;
                    }
                    Err(e) => {
                        eprintln!("Failed to read frame: {}", e);
                        break 'reader_loop;
                    }
                }
            }
            
            // Update state to disconnected
            let mut current_state = state.write().await;
            *current_state = TcpTransportState::Disconnected;
        });
        
        Ok(())
    }
    
    /// Start the writer task to process outgoing messages
    async fn start_writer_task<W>(&self, writer: W, mut msg_rx: mpsc::Receiver<MCPMessage>) -> Result<(), TransportError> 
    where 
        W: tokio::io::AsyncWrite + Unpin + Send + 'static 
    {
        let mut writer = FrameWriter::new(writer);
        let codec = MessageCodec::new();
        
        // Clone the state for the task
        let state = self.state.clone();
        
        // Spawn writer task
        tokio::spawn(async move {
            loop {
                // Check if we're still connected
                {
                    let current_state = state.read().await;
                    if *current_state != TcpTransportState::Connected {
                        break;
                    }
                }
                
                // Wait for the next message
                match msg_rx.recv().await {
                    Some(message) => {
                        // Encode the message
                        match codec.encode_message(&message).await {
                            Ok(frame) => {
                                // Write the frame
                                if let Err(e) = writer.write_frame(frame).await {
                                    eprintln!("Failed to write frame: {}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to encode message: {}", e);
                                continue;
                            }
                        }
                    }
                    None => {
                        // Channel closed
                        break;
                    }
                }
            }
            
            // Update state to disconnected
            let mut current_state = state.write().await;
            *current_state = TcpTransportState::Disconnected;
        });
        
        Ok(())
    }
}

#[async_trait]
impl Transport for TcpTransport {
    async fn send_message(&self, message: MCPMessage) -> Result<(), TransportError> {
        // Make sure we're connected
        if !self.is_connected().await {
            return Err(TransportError::ConnectionClosed(
                "Not connected".into()
            ));
        }
        
        // Send the message to the channel, which will be picked up by the writer task
        self.message_sender.lock().await.send(message).await.map_err(|e| {
            TransportError::ConnectionClosed(format!("Failed to send message: {}", e))
        })?;
        
        Ok(())
    }
    
    async fn receive_message(&self) -> Result<MCPMessage, TransportError> {
        // Get the receiver from the mutex
        let mut frame_rx = self.frame_receiver.lock().await;
        if frame_rx.is_none() {
            return Err(TransportError::ConnectionClosed(
                "No active stream to receive messages from".into()
            ));
        }
        
        // Wait for a message from the receiver
        match frame_rx.as_mut().unwrap().recv().await {
            Some(message) => Ok(message),
            None => Err(TransportError::ConnectionClosed(
                "Channel closed".into()
            )),
        }
    }
    
    async fn connect(&self) -> Result<(), TransportError> {
        // Update state to connecting
        {
            let mut state = self.state.write().await;
            *state = TcpTransportState::Connecting;
        }
        
        // Connect to the remote host
        let stream = match TcpStream::connect(&self.config.remote_address).await {
            Ok(stream) => stream,
            Err(e) => {
                // Update state to failed
                let mut state = self.state.write().await;
                *state = TcpTransportState::Failed(e.to_string());
                
                return Err(TransportError::ConnectionFailed(format!(
                    "Failed to connect to {}: {}", 
                    self.config.remote_address, e
                )));
            }
        };
        
        // Configure the stream
        stream.set_nodelay(true).map_err(|e| {
            TransportError::ConnectionFailed(format!("Failed to set nodelay: {}", e))
        })?;
        
        if let Some(interval) = self.config.keep_alive_interval {
            // We need to use socket2 to set keepalive on TcpStream
            // First get the std TcpStream, then create socket2::Socket from it
            let socket = socket2::Socket::from(stream.into_std().map_err(|e| {
                TransportError::ConnectionFailed(format!("Failed to get std socket: {}", e))
            })?);
            
            // Set keep-alive
            socket.set_keepalive(true).map_err(|e| {
                TransportError::ConnectionFailed(format!("Failed to set keepalive: {}", e))
            })?;
            
            // Set keep-alive parameters if available on this platform
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            socket.set_tcp_keepalive(&socket2::TcpKeepalive::new().with_time(std::time::Duration::from_secs(interval))).map_err(|e| {
                TransportError::ConnectionFailed(format!("Failed to set keepalive parameters: {}", e))
            })?;
            
            // Convert Socket to std::net::TcpStream explicitly
            let std_stream: std::net::TcpStream = socket.into();
            
            // Convert back to Tokio's TcpStream
            let socket = TcpStream::from_std(std_stream).unwrap();
            
            // Create a new channel for message sending
            let (msg_tx, msg_rx) = mpsc::channel(100);
            
            // Update our message sender with the new sender
            {
                let mut sender = self.message_sender.lock().await;
                *sender = msg_tx;
            }
            
            // Use split to create separate reader and writer parts
            let (read_half, write_half) = tokio::io::split(socket);
            
            // Start the reader task with the reader half
            self.start_reader_task(read_half)
                .await?;
            
            // Start the writer task with the writer half
            self.start_writer_task(write_half, msg_rx).await?;
        } else {
            // No keepalive configured, proceed normally
            // Create a new channel for message sending
            let (msg_tx, msg_rx) = mpsc::channel(100);
            
            // Update our message sender with the new sender
            {
                let mut sender = self.message_sender.lock().await;
                *sender = msg_tx;
            }
            
            // Use split to create separate reader and writer parts
            let (read_half, write_half) = tokio::io::split(stream);
            
            // Start the reader task with the reader half
            self.start_reader_task(read_half)
                .await?;
            
            // Start the writer task with the writer half
            self.start_writer_task(write_half, msg_rx).await?;
        }
        
        // Update state to connected
        {
            let mut state = self.state.write().await;
            *state = TcpTransportState::Connected;
        }
        
        Ok(())
    }
    
    async fn disconnect(&self) -> Result<(), TransportError> {
        // Update state to disconnecting
        {
            let mut state = self.state.write().await;
            *state = TcpTransportState::Disconnecting;
        }
        
        // Wait a bit for tasks to finish gracefully
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // Update state to disconnected
        {
            let mut state = self.state.write().await;
            *state = TcpTransportState::Disconnected;
        }
        
        Ok(())
    }
    
    async fn is_connected(&self) -> bool {
        let state = self.state.read().await;
        *state == TcpTransportState::Connected
    }
    
    fn get_metadata(&self) -> TransportMetadata {
        self.metadata.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_tcp_transport_connect() {
        // Create a config for localhost
        let config = TcpTransportConfig {
            remote_address: "127.0.0.1:9000".to_string(),
            ..Default::default()
        };
        
        // Create transport
        let transport = TcpTransport::new(config);
        
        // Ensure it starts disconnected
        assert!(!transport.is_connected().await);
        
        // Get metadata
        let metadata = transport.get_metadata();
        assert_eq!(metadata.transport_type, "tcp");
        assert_eq!(metadata.remote_address, "127.0.0.1:9000");
    }
} 