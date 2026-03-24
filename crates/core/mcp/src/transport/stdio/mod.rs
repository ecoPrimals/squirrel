// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::{mpsc, RwLock};
use tokio::io::{stdin, stdout, AsyncBufReadExt, AsyncWriteExt, BufReader};
use uuid::Uuid;
use tokio::sync::watch;
use std::time::Duration;
use crate::protocol::types::{MCPMessage, MessageId, MessageType, ProtocolVersion};
use crate::protocol::adapter_wire::{WireProtocolVersion};
use crate::transport::Transport;
use crate::transport::types::TransportMetadata;
use crate::protocol::domain_objects::EncryptionFormat;
use crate::protocol::types::{SecurityMetadata};
use crate::error::{MCPError, Result, TransportError};
use crate::types::CompressionFormat;
use serde_json;
use std::str::FromStr;
use tracing::{error, info, warn};
use tokio::sync::Mutex as TokioMutex;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use futures_util::TryFutureExt;

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
#[derive(Debug)]
pub struct StdioTransport {
    /// Transport configuration
    config: StdioConfig,
    
    /// Current connection state
    state: Arc<RwLock<StdioState>>,
    
    /// Command channel for sending commands to the stdio task
    command_sender: mpsc::Sender<MCPMessage>,
    
    /// Command channel for receiving commands
    command_receiver: Arc<TokioMutex<mpsc::Receiver<MCPMessage>>>,
    
    /// Message channel for receiving messages
    message_rx: watch::Receiver<Option<MCPMessage>>,
    message_tx: watch::Sender<Option<MCPMessage>>,
    
    /// Connection ID
    connection_id: String,
    
    /// Transport metadata
    metadata: TransportMetadata,

    /// Last message sent timestamp
    last_message_sent: AtomicU64,
}

impl StdioTransport {
    /// Create a new stdio transport
    #[must_use] pub fn new(config: &StdioConfig) -> Self {
        let (command_sender, command_receiver) = mpsc::channel(100);
        let (message_tx, message_rx) = watch::channel(None);
        
        let connection_id = Uuid::new_v4().to_string();
        
        let transport_metadata = TransportMetadata {
            connection_id: Uuid::new_v4().to_string(),
            remote_address: None,
            local_address: None,
            encryption_format: Some(config.encryption),
            compression_format: Some(config.compression),
            connected_at: Utc::now(),
            last_activity: Utc::now(),
            additional_info: HashMap::new(),
        };
        
        Self {
            config: config.clone(),
            state: Arc::new(RwLock::new(StdioState::Disconnected)),
            command_sender,
            command_receiver: Arc::new(TokioMutex::new(command_receiver)),
            message_rx,
            message_tx,
            connection_id,
            metadata: transport_metadata,
            last_message_sent: AtomicU64::new(0),
        }
    }
    
    /// Start the reader task for stdin
    async fn start_reader_task(&self) -> Result<()> {
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
                        info!("StdioTransport: stdin closed.");
                        break;
                    }
                    Ok(_) => {
                        // Trim whitespace/newline
                        let trimmed_buffer = buffer.trim();
                        if trimmed_buffer.is_empty() {
                            continue;
                        }

                        // Attempt to parse the JSON
                        match serde_json::from_str::<serde_json::Value>(trimmed_buffer) {
                            Ok(json) => {
                                // Extract fields safely
                                let id_str = json.get("id").and_then(|v| v.as_str()).unwrap_or_default();
                                let type_str = json.get("type").and_then(|v| v.as_str()).unwrap_or("Unknown");
                                let version_str = json.get("version").and_then(|v| v.as_str()).unwrap_or("1.0"); // Default version if missing

                                // Parse MessageType
                                let message_type = match type_str.parse::<MessageType>() {
                                    Ok(mt) => mt,
                                    Err(_) => {
                                        warn!("StdioTransport: Received message with unknown type: {}", type_str);
                                        MessageType::Unknown // Handle unknown types gracefully
                                    }
                                };

                                // Parse major/minor from version_str or default
                                let version = match version_str.split_once('.') {
                                    Some((major_str, minor_str)) => {
                                        let major = major_str.parse::<u16>().unwrap_or(1);
                                        let minor = minor_str.parse::<u16>().unwrap_or(0);
                                        crate::protocol::types::ProtocolVersion { major, minor }
                                    }
                                    None => {
                                        warn!("Invalid protocol version format '{}', using default 1.0", version_str);
                                        crate::protocol::types::ProtocolVersion::default()
                                    }
                                };

                                let message = MCPMessage {
                                    id: crate::protocol::types::MessageId(id_str.to_string()),
                                    type_: message_type, // Use parsed type
                                    payload: json.get("payload").cloned().unwrap_or(serde_json::Value::Null),
                                    metadata: json.get("metadata").cloned(), // Use metadata from message if present
                                    security: SecurityMetadata::default(), // Default security for now
                                    timestamp: chrono::Utc::now(),
                                    // Use parsed or default version
                                    version,
                                    trace_id: json.get("trace_id").and_then(|v| v.as_str()).map(String::from), // Optional trace_id
                                };

                                // Send to message channel
                                if message_tx.send(Some(message)).is_err() {
                                    error!("StdioTransport: Failed to send received message to internal channel.");
                                    break; // Channel closed, exit task
                                }
                            }
                            Err(e) => {
                                error!("StdioTransport: Failed to deserialize message from stdin: {}. Line: '{}'", e, trimmed_buffer);
                                // Optionally, send an error event back if possible/needed,
                                // but primarily handle errors by logging in stdio transport.
                                // Consider sending TransportEvent::Error if a general error channel existed.
                            }
                        }
                    }
                    Err(e) => {
                        error!("StdioTransport: Error reading from stdin: {}", e);
                        break;
                    }
                }
            }
            
            // Update state when task completes
            let mut current_state = state.write().await;
            if *current_state != StdioState::Disconnecting {
                 *current_state = StdioState::Disconnected;
                 info!("StdioTransport: Reader task finished, state set to Disconnected.");
            } else {
                 info!("StdioTransport: Reader task finished during disconnection.");
            }
        });
        
        Ok(())
    }
    
    /// Start the writer task for stdout
    async fn start_writer_task(&self) -> Result<()> {
        let state = self.state.clone();
        let use_ndjson = self.config.use_ndjson;
        let command_receiver = self.command_receiver.clone();
        
        tokio::spawn(async move {
            let mut stdout = stdout();
            let mut rx = command_receiver.lock().await;
            
            while let Some(message) = rx.recv().await {
                // Convert message to JSON
                let json = match serde_json::to_string(&message) {
                    Ok(j) => j,
                    Err(e) => {
                        error!("StdioTransport: Failed to serialize message for stdout: {}", e);
                        continue; // Skip malformed messages
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
                    error!("StdioTransport: Failed to write to stdout: {}", e);
                    break;
                }
                
                // Ensure output is flushed
                if let Err(e) = stdout.flush().await {
                    error!("StdioTransport: Failed to flush stdout: {}", e);
                    break;
                }
            }
            
            // Update the state when the task completes
            let mut current_state = state.write().await;
            if *current_state != StdioState::Disconnecting {
                 *current_state = StdioState::Disconnected;
                 info!("StdioTransport: Writer task finished, state set to Disconnected.");
            } else {
                 info!("StdioTransport: Writer task finished during disconnection.");
            }
        });
        
        Ok(())
    }

    async fn process_line(&self, line: String, msg_tx: &mpsc::Sender<MCPMessage>) -> Result<()> {
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(MCPError::Transport(TransportError::ProtocolError("Invalid message format".to_string())).into());
        }
        let header = parts[0];
        let payload_str = parts[1];

        let header_parts: Vec<&str> = header.split(',').collect();
        if header_parts.len() < 3 {
            return Err(MCPError::Transport(TransportError::ProtocolError("Invalid header format".to_string())).into());
        }

        let message_id = MessageId(header_parts[0].to_string());
        let message_type_str = header_parts[1];
        let version_str = header_parts[2];

        let message_type = MessageType::from_str(message_type_str)
            .map_err(|e| -> crate::error::MCPError {
                MCPError::Transport(format!("Invalid message type: {}", message_type_str).into())
            })?;

        // Correctly handle ProtocolVersion parsing and conversion
        let version = match WireProtocolVersion::from_str(version_str) {
            Ok(wire_version) => ProtocolVersion::try_from(wire_version)
                .map_err(MCPError::Protocol)?,
            Err(_) => {
                warn!("Invalid protocol version string '{}', defaulting to 1.0", version_str);
                // Use the default from crate::protocol::types::ProtocolVersion
                ProtocolVersion::default()
            }
        };

        let payload = serde_json::from_str(payload_str)
            .map_err(|e| MCPError::Serialization(e.to_string()))?;

        let mcp_message = MCPMessage {
            id: message_id,
            type_: message_type,
            version, // Use the parsed/defaulted version
            payload,
            // Fill other fields as needed, e.g., timestamp, security
            timestamp: Utc::now(),
            security: Default::default(),
            metadata: None,
            trace_id: None,
        };

        // Send the parsed message via the channel
        msg_tx.send(mcp_message)
            .await
            .map_err(|e| -> crate::error::MCPError {
                MCPError::Transport(format!("Failed to send parsed message to internal channel: {}", e).into())
            })?;

        self.last_message_sent.store(
            Utc::now().timestamp_millis()
                .try_into()
                .unwrap_or(0), // Fallback to 0 if conversion fails
            Ordering::SeqCst
        )
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn send_message(&self, message: MCPMessage) -> crate::error::Result<()> {
        // Use the command_sender to send messages
        self.command_sender.send(message)
            .await
            .map_err(|e| -> crate::error::MCPError {
                MCPError::Transport(format!("Failed to send message: {}", e).into())
            })?;
        
        self.last_message_sent.store(
            Utc::now().timestamp_millis()
                .try_into()
                .unwrap_or(0), // Fallback to 0 if conversion fails
            Ordering::SeqCst
        );
        Ok(())
    }
    
    async fn receive_message(&self) -> crate::error::Result<MCPMessage> {
        let mut rx = self.message_rx.clone();
        
        // Check connection state before waiting
        if !self.is_connected().await {
             return Err(MCPError::Transport(TransportError::ConnectionError("Not connected".to_string())).into());
        }

        loop {
            // Wait for the next message
            if rx.changed().await.is_err() {
                 // Sender dropped, connection likely closed
                 let state = self.state.read().await;
                 error!("StdioTransport: Message watch channel closed. State: {:?}", *state);
                 return Err(MCPError::Transport(TransportError::ConnectionError("Watch channel closed".to_string())).into());
            }

            // Get the current value
            if let Some(message) = &*rx.borrow() {
                // Consume the message by sending None back? No, watch channel doesn't work like that.
                // The receiver just observes the latest value.
                return Ok(message.clone());
            }
            // If none, check connection state again before sleeping
             if !self.is_connected().await {
                 return Err(MCPError::Transport(TransportError::ConnectionError("Disconnected while waiting for message".to_string())).into());
            }

            // Small sleep to prevent busy-waiting if changed() fires but value is still None (shouldn't happen often)
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }
    
    async fn connect(&mut self) -> crate::error::Result<()> {
        info!("StdioTransport: Connecting...");
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
            if *state == StdioState::Connecting {
                *state = StdioState::Connected;
                info!("StdioTransport: Connected successfully.");
            } else {
                 warn!("StdioTransport: State changed during connection sequence: {:?}. Not setting to Connected.", *state);
                 // Properly wrapped error
                 return Err(MCPError::Transport(format!("Connection failed during startup. Final state: {:?}", *state).into()).into());
            }
        }
        
        Ok(())
    }
    
    async fn disconnect(&self) -> crate::error::Result<()> {
        info!("StdioTransport: Disconnecting...");
        // Update the state to disconnecting
        let mut state = self.state.write().await;
        if *state == StdioState::Disconnected || *state == StdioState::Disconnecting {
            info!("StdioTransport: Already disconnected or disconnecting.");
            return Ok(()); // Already disconnected or in progress
        }
        *state = StdioState::Disconnecting;
        drop(state); // Release lock before potentially long operations

        // Closing stdin/stdout isn't directly possible/reliable from here.
        // The tasks should detect closure or state changes.

        // Wait briefly for tasks to potentially shut down based on state change.
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Final state update
        let mut state = self.state.write().await;
        *state = StdioState::Disconnected;
        info!("StdioTransport: Disconnected.");

        Ok(())
    }
    
    async fn is_connected(&self) -> bool {
        let state = self.state.read().await;
        *state == StdioState::Connected
    }
    
    async fn get_metadata(&self) -> crate::transport::types::TransportMetadata {
        self.metadata.clone()
    }

    // Update send_raw to return UnsupportedOperation
    async fn send_raw(&self, _bytes: &[u8]) -> crate::error::Result<()> {
        error!("send_raw is not supported for StdioTransport");
        Err(MCPError::UnsupportedOperation("send_raw not supported for StdioTransport".to_string()).into())
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        let (command_sender, command_receiver) = mpsc::channel(100);
        let (message_tx, message_rx) = watch::channel(None);
        let metadata = TransportMetadata {
            connection_id: Uuid::new_v4().to_string(),
            remote_address: None,
            local_address: None,
            encryption_format: Some(EncryptionFormat::None),
            compression_format: Some(CompressionFormat::None),
            connected_at: Utc::now(),
            last_activity: Utc::now(),
            additional_info: HashMap::new(),
        };
        Self {
            config: StdioConfig {
                max_message_size: 10 * 1024 * 1024, // 10MB
                encryption: EncryptionFormat::None,
                compression: CompressionFormat::None,
                use_ndjson: true,
                buffer_size: 8 * 1024, // 8KB
            },
            state: Arc::new(RwLock::new(StdioState::Disconnected)),
            command_sender,
            command_receiver: Arc::new(TokioMutex::new(command_receiver)),
            message_rx,
            message_tx,
            connection_id: Uuid::new_v4().to_string(),
            metadata,
            last_message_sent: AtomicU64::new(0),
        }
    }
}

#[cfg(test)]
mod tests {
    // ... tests ...
}