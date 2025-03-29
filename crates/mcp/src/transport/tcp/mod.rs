// TCP transport implementation for MCP
//
// This module provides a TCP-based transport implementation for Machine Context Protocol
// (MCP) communication. It handles establishing TCP connections, message framing,
// serialization/deserialization, and connection lifecycle management.
//
// The TCP transport is a reliable, stream-oriented transport that ensures messages
// are delivered in order without loss. It uses a framing protocol to preserve message
// boundaries over the byte stream and supports various configurations including:
// - Connection timeouts
// - Keep-alive mechanisms
// - Local address binding
// - Reconnection strategies
// - Encryption and compression options

use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::{mpsc, RwLock, Mutex};
use tokio::net::TcpStream;
use uuid::Uuid;
use socket2;
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio_util::codec::{Framed, LinesCodec, LinesCodecError};
use std::net::SocketAddr;
use std::str::FromStr;
use crate::error::{MCPError, Result, TransportError};
use crate::message::Message;
use crate::protocol::MCPMessage;
use crate::transport::types::ConnectionState;
use crate::security::types::EncryptionFormat;
use crate::types::CompressionFormat;
use super::{Transport, TransportMetadata};
use super::frame::{FrameReader, FrameWriter, MessageCodec};
use crate::security::EncryptionFormat as SecurityEncryptionFormat;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;

/// Configuration for the TCP transport
///
/// This struct contains all the configuration parameters for establishing
/// and maintaining a TCP connection for MCP communication.
#[derive(Debug, Clone)]
pub struct TcpTransportConfig {
    /// Remote address to connect to (client mode) or bind address (server mode)
    pub remote_address: String,
    
    /// Local bind address for client connections
    ///
    /// When specified, outgoing connections will be bound to this local address.
    /// This can be useful for multi-homed systems or when specific network interfaces
    /// need to be used.
    pub local_bind_address: Option<String>,
    
    /// Max message size in bytes
    ///
    /// Messages exceeding this size will be rejected. This helps prevent
    /// excessive memory usage and potential denial-of-service attacks.
    pub max_message_size: usize,
    
    /// Connection timeout in seconds
    ///
    /// Maximum time to wait for the connection to be established.
    /// If the connection cannot be established within this time,
    /// it will fail with a timeout error.
    pub connection_timeout: u64,
    
    /// Keep alive interval in seconds
    ///
    /// If set, TCP keepalive packets will be sent at this interval to maintain
    /// the connection and detect disconnections even when no data is being sent.
    pub keep_alive_interval: Option<u64>,
    
    /// Encryption format
    ///
    /// Specifies how messages should be encrypted during transport.
    /// Default is no encryption (None).
    pub encryption: EncryptionFormat,
    
    /// Compression format
    ///
    /// Specifies how messages should be compressed during transport.
    /// Default is no compression (None).
    pub compression: CompressionFormat,
    
    /// Maximum number of reconnection attempts
    ///
    /// How many times the transport will attempt to reconnect after a connection failure
    /// before giving up. Set to 0 to disable reconnection attempts.
    pub max_reconnect_attempts: u32,
    
    /// Reconnection delay in milliseconds
    ///
    /// Time to wait between reconnection attempts. This may be increased
    /// with each attempt using exponential backoff, depending on the implementation.
    pub reconnect_delay_ms: u64,
}

impl Default for TcpTransportConfig {
    /// Creates a default TCP transport configuration
    ///
    /// Default values:
    /// - Remote address: 127.0.0.1:9000
    /// - Max message size: 10MB
    /// - Connection timeout: 30 seconds
    /// - Keep-alive interval: 60 seconds
    /// - No encryption or compression
    /// - 5 reconnection attempts with 1 second delay
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

impl TcpTransportConfig {
    /// Sets the remote address
    ///
    /// # Arguments
    ///
    /// * `address` - The remote address to connect to
    ///
    /// # Returns
    ///
    /// The updated configuration
    #[must_use]
    pub fn with_remote_address(mut self, address: &str) -> Self {
        self.remote_address = address.to_string();
        self
    }
    
    /// Sets the local bind address
    ///
    /// # Arguments
    ///
    /// * `address` - The local address to bind to
    ///
    /// # Returns
    ///
    /// The updated configuration
    #[must_use]
    pub fn with_local_bind_address(mut self, address: &str) -> Self {
        self.local_bind_address = Some(address.to_string());
        self
    }
    
    /// Sets the maximum message size
    ///
    /// # Arguments
    ///
    /// * `size` - Maximum message size in bytes
    ///
    /// # Returns
    ///
    /// The updated configuration
    #[must_use]
    pub const fn with_max_message_size(mut self, size: usize) -> Self {
        self.max_message_size = size;
        self
    }
    
    /// Sets the connection timeout
    ///
    /// # Arguments
    ///
    /// * `timeout_ms` - Connection timeout in milliseconds
    ///
    /// # Returns
    ///
    /// The updated configuration
    #[must_use]
    pub const fn with_connection_timeout(mut self, timeout_ms: u64) -> Self {
        self.connection_timeout = timeout_ms / 1000;
        if self.connection_timeout == 0 {
            self.connection_timeout = 1; // Minimum 1 second
        }
        self
    }
    
    /// Sets the keep-alive interval
    ///
    /// # Arguments
    ///
    /// * `interval_ms` - Keep-alive interval in milliseconds, or None to disable
    ///
    /// # Returns
    ///
    /// The updated configuration
    #[must_use]
    pub fn with_keep_alive_interval(mut self, interval_ms: Option<u64>) -> Self {
        self.keep_alive_interval = interval_ms.map(|ms| ms / 1000);
        self
    }
    
    /// Sets the encryption format
    ///
    /// # Arguments
    ///
    /// * `encryption_format` - The encryption format to use
    ///
    /// # Returns
    ///
    /// The updated configuration
    #[must_use]
    pub const fn with_encryption(mut self, encryption_format: EncryptionFormat) -> Self {
        self.encryption = encryption_format;
        self
    }
    
    /// Sets the compression format
    ///
    /// # Arguments
    ///
    /// * `compression_format` - The compression format to use
    ///
    /// # Returns
    ///
    /// The updated configuration
    #[must_use]
    pub const fn with_compression(mut self, compression_format: CompressionFormat) -> Self {
        self.compression = compression_format;
        self
    }
    
    /// Sets the maximum number of reconnection attempts
    ///
    /// # Arguments
    ///
    /// * `max_attempts` - Maximum number of reconnection attempts
    ///
    /// # Returns
    ///
    /// The updated configuration
    #[must_use]
    pub const fn with_max_reconnect_attempts(mut self, max_attempts: u32) -> Self {
        self.max_reconnect_attempts = max_attempts;
        self
    }
    
    /// Sets the reconnection delay
    ///
    /// # Arguments
    ///
    /// * `delay_ms` - Reconnection delay in milliseconds
    ///
    /// # Returns
    ///
    /// The updated configuration
    #[must_use]
    pub const fn with_reconnect_delay_ms(mut self, delay_ms: u64) -> Self {
        self.reconnect_delay_ms = delay_ms;
        self
    }
}

/// TCP transport connection state
///
/// Represents the current state of the TCP connection.
#[derive(Debug, Clone, PartialEq, Eq)]
enum TcpTransportState {
    /// Not connected
    ///
    /// The initial state or the state after a disconnection.
    Disconnected,
    
    /// In the process of connecting
    ///
    /// The transport is actively trying to establish a connection.
    Connecting,
    
    /// Connected and ready to send/receive
    ///
    /// The connection is established and messages can be sent and received.
    Connected,
    
    /// In the process of disconnecting
    ///
    /// The transport is actively closing the connection.
    Disconnecting,
    
    /// Connection has failed
    ///
    /// The connection attempt has failed, with an error message.
    Failed(String),
}

/// TCP transport for MCP communication
///
/// Provides TCP-based transport for MCP messages using a framing protocol to
/// preserve message boundaries. The implementation handles connection establishment,
/// reconnection, message serialization, and the full connection lifecycle.
pub struct TcpTransport {
    /// Transport configuration
    config: TcpTransportConfig,
    
    /// Current connection state
    state: Arc<RwLock<TcpTransportState>>,
    
    /// Message channel for sending
    ///
    /// Messages to be sent are passed to the writer task through this channel.
    message_sender: Arc<Mutex<mpsc::Sender<MCPMessage>>>,
    
    /// Frame channel for incoming frames
    ///
    /// Received messages are passed from the reader task through this channel.
    frame_receiver: Arc<Mutex<Option<mpsc::Receiver<MCPMessage>>>>,
    
    /// Connection ID
    ///
    /// Unique identifier for this connection instance.
    connection_id: String,
    
    /// Transport metadata
    ///
    /// Contains information about this transport instance.
    metadata: TransportMetadata,
}

impl TcpTransport {
    /// Creates a new TCP transport with the specified configuration
    ///
    /// This method initializes the transport but does not establish a connection.
    /// You must call `connect()` before sending or receiving messages.
    ///
    /// # Arguments
    ///
    /// * `config` - The TCP transport configuration
    ///
    /// # Returns
    ///
    /// A new TCP transport instance
    #[must_use]
    pub fn new(config: TcpTransportConfig) -> Self {
        // Create message channel for sending
        let (tx, rx) = mpsc::channel(100);
        
        // Clone the receiver for the constructor
        let rx_option = Some(rx);
        
        // Attempt to parse addresses from config
        let peer_addr = SocketAddr::from_str(&config.remote_address).ok();
        let local_addr = config.local_bind_address.as_deref().and_then(|s| SocketAddr::from_str(s).ok());
        
        // Create metadata
        let metadata = TransportMetadata {
            transport_type: "tcp".to_string(),
            peer_addr,
            local_addr,
            encryption: config.encryption,
            compression: config.compression,
            connected_at: chrono::Utc::now(),
            state: ConnectionState::Disconnected,
            protocol_version: "unknown".to_string(),
            additional_metadata: Default::default(),
        };
        
        // Generate a unique connection ID
        let connection_id = Uuid::new_v4().to_string();
        
        Self {
            config,
            state: Arc::new(RwLock::new(TcpTransportState::Disconnected)),
            message_sender: Arc::new(Mutex::new(tx)),
            frame_receiver: Arc::new(Mutex::new(rx_option)),
            connection_id,
            metadata,
        }
    }
    
    /// Start the reader task to process incoming messages
    ///
    /// Creates and starts a background task to read frames from the TCP stream,
    /// decode them into `MCPMessages`, and forward them to the `frame_receiver` channel.
    ///
    /// # Arguments
    ///
    /// * `reader` - An `AsyncRead` implementation (typically a TCP stream)
    ///
    /// # Returns
    ///
    /// Result indicating success or an error
    async fn start_reader_task<R>(&self, reader: R) -> Result<()> 
    where 
        R: tokio::io::AsyncRead + Unpin + Send + 'static 
    {
        let addr = self.config.remote_address.clone();
        if addr.is_empty() {
            return Err(TransportError::ConnectionFailed(
                "TCP transport requires a remote address".to_string()
            ).into());
        }
        
        // Generate a unique stream ID
        let _stream_id = Uuid::new_v4().to_string();
        
        let state = self.state.clone();
        
        // Create a channel for frames
        let (frame_tx, frame_rx) = mpsc::channel::<MCPMessage>(100);
        
        // Lock the frame_receiver and replace it with our receiver
        *self.frame_receiver.lock().await = Some(frame_rx);
        
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
                                    eprintln!("Failed to forward message: {e}");
                                    break 'reader_loop;
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to decode frame: {e}");
                                continue 'reader_loop;
                            }
                        }
                    }
                    Ok(None) => {
                        // End of stream
                        break 'reader_loop;
                    }
                    Err(e) => {
                        eprintln!("Failed to read frame: {e}");
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
    async fn start_writer_task<W>(&self, writer: W, mut msg_rx: mpsc::Receiver<MCPMessage>) -> Result<()> 
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
                                    eprintln!("Failed to write frame: {e}");
                                    break;
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to encode message: {e}");
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
    async fn send_message(&self, message: MCPMessage) -> crate::error::Result<()> {
        // Check if connected
        {
            let state = self.state.read().await;
            if !matches!(*state, TcpTransportState::Connected) {
                return Err(TransportError::connection_closed("Not connected").into());
            }
        }
        
        // Send message to writer task
        let msg_sender = self.message_sender.lock().await;
        match msg_sender.send(message).await {
            Ok(()) => Ok(()),
            Err(e) => Err(TransportError::protocol_error(format!("Failed to send message: {e}")).into())
        }
    }
    
    /// Receives a message from the transport
    async fn receive_message(&self) -> crate::error::Result<MCPMessage> {
        // Check if transport is connected using state directly
        {
            let state = self.state.read().await;
            if *state != TcpTransportState::Connected {
                return Err(TransportError::connection_closed("Not connected").into());
            }
        }
        
        // Lock the frame_receiver and check for a receiver in a single block
        let mut frame_receiver_guard = self.frame_receiver.lock().await;
        
        // Check if we have a receiver and process message in a single code path
        if let Some(receiver) = frame_receiver_guard.as_mut() {
            // Now receive a message (this is safe as we keep the lock)
            if let Some(msg) = receiver.recv().await {
                // Drop the guard as soon as we have the message
                drop(frame_receiver_guard);
                Ok(msg)
            } else {
                // Channel is closed, remove the receiver
                *frame_receiver_guard = None;
                drop(frame_receiver_guard);
                Err(TransportError::connection_closed("Channel closed").into())
            }
        } else {
            // No receiver available
            drop(frame_receiver_guard);
            Err(TransportError::protocol_error("No receiver available").into())
        }
    }
    
    async fn connect(&mut self) -> crate::error::Result<()> {
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
                {
                    *self.state.write().await = TcpTransportState::Failed(e.to_string());
                    
                    return Err(MCPError::Transport(TransportError::connection_failed(format!(
                        "Failed to connect to {}: {}", 
                        self.config.remote_address, e
                    )).into()));
                }
            }
        };
        
        // Configure the stream
        stream.set_nodelay(true).map_err(|e| {
            MCPError::Transport(TransportError::connection_failed(format!("Failed to set nodelay: {e}")).into())
        })?;
        
        if let Some(interval) = self.config.keep_alive_interval {
            // We need to use socket2 to set keepalive on TcpStream
            // First get the std TcpStream, then create socket2::Socket from it
            let socket = socket2::Socket::from(stream.into_std().map_err(|e| {
                MCPError::Transport(TransportError::connection_failed(format!("Failed to get std socket: {e}")).into())
            })?);
            
            // Set keep-alive
            socket.set_keepalive(true).map_err(|e| {
                MCPError::Transport(TransportError::connection_failed(format!("Failed to set keepalive: {e}")).into())
            })?;
            
            // Set keep-alive parameters if available on this platform
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            socket.set_tcp_keepalive(&socket2::TcpKeepalive::new().with_time(std::time::Duration::from_secs(interval))).map_err(|e| {
                MCPError::Transport(TransportError::connection_failed(format!("Failed to set keepalive parameters: {e}")).into())
            })?;
            
            // Convert Socket to std::net::TcpStream explicitly
            let std_stream: std::net::TcpStream = socket.into();
            
            // Convert back to Tokio's TcpStream
            let socket = TcpStream::from_std(std_stream).map_err(|e| {
                MCPError::Transport(TransportError::connection_failed(format!("Failed to convert std socket to tokio socket: {e}")).into())
            })?;
            
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
    
    async fn disconnect(&self) -> crate::error::Result<()> {
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
        matches!(*state, TcpTransportState::Connected)
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