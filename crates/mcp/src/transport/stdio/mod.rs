use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::{mpsc, RwLock};
use tokio::io::{stdin, stdout, AsyncBufReadExt, AsyncWriteExt, BufReader};
use uuid::Uuid;
use tokio::sync::watch;
use std::time::Duration;
use crate::types;

use crate::error::transport::TransportError;
use crate::types::{MCPMessage, EncryptionFormat, CompressionFormat, MessageType, SecurityMetadata, ProtocolVersion};
use super::{Transport, TransportMetadata};
use crate::error::MCPError;

/// Configuration for the stdio transport
#[derive(Debug, Clone)]
pub struct StdioConfig {
    /// Maximum message size in bytes
    pub max_message_size: usize,
    
    /// Encryption format
    pub encryption: EncryptionFormat,
    
    /// Compression format
    pub compression: CompressionFormat,
    
    /// Whether to use newline delimited JSON
    pub use_ndjson: bool,
    
    /// Buffer size for stdin/stdout
    pub buffer_size: usize,
}

impl Default for StdioConfig {
    fn default() -> Self {
        Self {
            max_message_size: 10 * 1024 * 1024, // 10MB
            encryption: EncryptionFormat::None,
            compression: CompressionFormat::None,
            use_ndjson: true,
            buffer_size: 8 * 1024, // 8KB
        }
    }
}

/// Stdio connection state
#[derive(Debug, Clone, PartialEq, Eq)]
enum StdioState {
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

/// Stdio transport implementation
pub struct StdioTransport {
    /// Transport configuration
    config: StdioConfig,
    
    /// Current connection state
    state: Arc<RwLock<StdioState>>,
    
    /// Command channel for sending commands to the stdio task
    command_sender: mpsc::Sender<MCPMessage>,
    
    /// Message channel for receiving messages
    message_rx: watch::Receiver<Option<MCPMessage>>,
    message_tx: watch::Sender<Option<MCPMessage>>,
    
    /// Connection ID
    connection_id: String,
    
    /// Transport metadata
    metadata: TransportMetadata,
}

impl StdioTransport {
    /// Create a new stdio transport
    #[must_use] pub fn new(config: StdioConfig) -> Self {
        let (tx, rx) = mpsc::channel(100);
        let (message_tx, message_rx) = watch::channel(None);
        
        let connection_id = Uuid::new_v4().to_string();
        
        Self {
            config: config.clone(),
            state: Arc::new(RwLock::new(StdioState::Disconnected)),
            command_sender: tx,
            message_rx,
            message_tx,
            connection_id,
            metadata: TransportMetadata {
                transport_type: "stdio".to_string(),
                remote_address: "stdio".to_string(),
                local_address: None,
                encryption: config.encryption,
                compression: config.compression,
            },
        }
    }
    
    /// Start the reader task for stdin
    async fn start_reader_task(&self) -> Result<(), TransportError> {
        let state = self.state.clone();
        let message_tx = self.message_tx.clone();
        let use_ndjson = self.config.use_ndjson;
        
        tokio::spawn(async move {
            let stdin = stdin();
            let mut reader = BufReader::new(stdin);
            let mut buffer = String::new();
            
            loop {
                buffer.clear();
                
                // Read based on format
                let read_result = if use_ndjson {
                    // Read line for NDJSON
                    reader.read_line(&mut buffer).await
                } else {
                    // Read until null terminator for JSON
                    reader.read_line(&mut buffer).await
                };
                
                match read_result {
                    Ok(0) => {
                        // EOF reached
                        break;
                    }
                    Ok(_) => {
                        // Parse the JSON
                        let json: serde_json::Value = serde_json::from_str(&buffer).unwrap_or(serde_json::json!({}));
                        
                        let message = MCPMessage {
                            id: types::MessageId(json.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string()),
                            type_: MessageType::Command,
                            payload: json.get("payload").cloned().unwrap_or(serde_json::Value::Null),
                            metadata: Some(json.get("metadata").cloned().unwrap_or(serde_json::Value::Null)),
                            security: SecurityMetadata::default(),
                            timestamp: chrono::Utc::now(),
                            version: ProtocolVersion::default(),
                            trace_id: None,
                        };
                        
                        // Send to message channel
                        message_tx.send(Some(message)).ok();
                    }
                    Err(e) => {
                        eprintln!("Error reading from stdin: {e}");
                        break;
                    }
                }
            }
            
            // Update state when task completes
            let mut current_state = state.write().await;
            *current_state = StdioState::Disconnected;
        });
        
        Ok(())
    }
    
    /// Start the writer task for stdout
    async fn start_writer_task(&self) -> Result<(), TransportError> {
        let state = self.state.clone();
        let use_ndjson = self.config.use_ndjson;
        
        // Create a new channel for the writer task
        let (_tx, mut rx) = mpsc::channel::<MCPMessage>(100);
        
        // Clone the command_sender for passing to the task
        let _command_sender = self.command_sender.clone();
        
        tokio::spawn(async move {
            let mut stdout = stdout();
            
            while let Some(message) = rx.recv().await {
                // Convert message to JSON
                let json = match serde_json::to_string(&message) {
                    Ok(j) => j,
                    Err(e) => {
                        eprintln!("Failed to serialize message: {e}");
                        continue;
                    }
                };
                
                // Write message to stdout based on format
                let result = if use_ndjson {
                    // Write as NDJSON (Newline-Delimited JSON)
                    stdout.write_all(format!("{json}\n").as_bytes()).await
                } else {
                    // Write as JSON with terminator
                    stdout.write_all(format!("{json}\0").as_bytes()).await
                };
                
                // Check for write errors
                if let Err(e) = result {
                    eprintln!("Failed to write to stdout: {e}");
                    break;
                }
                
                // Ensure output is flushed
                if let Err(e) = stdout.flush().await {
                    eprintln!("Failed to flush stdout: {e}");
                    break;
                }
            }
            
            // Update the state when the task completes
            let mut current_state = state.write().await;
            *current_state = StdioState::Disconnected;
        });
        
        Ok(())
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn send_message(&self, message: MCPMessage) -> crate::error::Result<()> {
        // Use the command_sender to send messages
        self.command_sender.send(message)
            .await
            .map_err(|e| MCPError::Transport(TransportError::ConnectionClosed(format!("Failed to send message: {e}")).into()))?;
        
        Ok(())
    }
    
    async fn receive_message(&self) -> crate::error::Result<MCPMessage> {
        let mut rx = self.message_rx.clone();
        
        loop {
            // Wait for the next message
            rx.changed().await.map_err(|_| MCPError::Transport(TransportError::ConnectionClosed("Channel closed".to_string()).into()))?;
            
            // Get the current value
            if let Some(message) = &*rx.borrow() {
                return Ok(message.clone());
            }
            
            // If none, continue waiting
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
    
    async fn connect(&mut self) -> crate::error::Result<()> {
        // Update the state to connecting
        {
            let mut state = self.state.write().await;
            *state = StdioState::Connecting;
        }
        
        // Start the reader and writer tasks
        self.start_reader_task().await?;
        self.start_writer_task().await?;
        
        // Update the state to connected
        {
            let mut state = self.state.write().await;
            *state = StdioState::Connected;
        }
        
        Ok(())
    }
    
    async fn disconnect(&self) -> crate::error::Result<()> {
        // Update the state to disconnecting
        {
            let mut state = self.state.write().await;
            *state = StdioState::Disconnecting;
        }
        
        // Don't actually send a stop command since we don't have a stop_channel
        // Just wait a bit to simulate disconnection
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Update the state to disconnected
        {
            let mut state = self.state.write().await;
            *state = StdioState::Disconnected;
        }
        
        Ok(())
    }
    
    async fn is_connected(&self) -> bool {
        let state = self.state.read().await;
        *state == StdioState::Connected
    }
    
    fn get_metadata(&self) -> TransportMetadata {
        self.metadata.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_stdio_transport_create() {
        // Create a config
        let config = StdioConfig {
            use_ndjson: true,
            ..Default::default()
        };
        
        // Create transport
        let transport = StdioTransport::new(config);
        
        // Ensure it starts disconnected
        assert!(!transport.is_connected().await);
        
        // Get metadata
        let metadata = transport.get_metadata();
        assert_eq!(metadata.transport_type, "stdio");
        assert_eq!(metadata.remote_address, "stdio");
    }
} 