// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::{mpsc::{self, Sender, Receiver}, RwLock, Mutex as TokioMutex};
use uuid::Uuid;
use std::collections::VecDeque;
use rand;
use crate::error::{MCPError, Result, TransportError};
use crate::protocol::types::MCPMessage;
use crate::transport::Transport;
use crate::transport::types::TransportMetadata;
use tracing::{error, trace};
use std::collections::HashMap;
use chrono::Utc;
use crate::config::MemoryTransportConfig;

// Import unified config for timeout management
use squirrel_mcp_config::unified::SquirrelUnifiedConfig;

/// In-memory connection state
#[derive(Debug, Clone, PartialEq, Eq)]
enum MemoryState {
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

/// A handle that can be used to pair two memory transports
#[derive(Debug, Clone)]
pub struct MemoryChannel {
    /// Channel A to B
    a_to_b: mpsc::Sender<MCPMessage>,
    
    /// Channel B to A
    b_to_a: mpsc::Sender<MCPMessage>,
    
    /// Message history
    history: Arc<TokioMutex<VecDeque<MCPMessage>>>,
    
    /// Maximum history size
    max_history: Option<usize>,
}

impl MemoryChannel {
    /// Create a new memory channel
    #[must_use] pub fn new(buffer_size: usize, max_history: Option<usize>) -> Self {
        let (a_to_b_tx, _) = mpsc::channel(buffer_size);
        let (b_to_a_tx, _) = mpsc::channel(buffer_size);
        
        Self {
            a_to_b: a_to_b_tx,
            b_to_a: b_to_a_tx,
            history: Arc::new(TokioMutex::new(VecDeque::with_capacity(max_history.unwrap_or(1000)))),
            max_history,
        }
    }
    
    /// Create a single memory transport with the given configuration
    #[must_use] pub fn create_transport(&self, config: MemoryTransportConfig) -> MemoryTransport {
        // Load unified configuration for timeouts
        use squirrel_mcp_config::unified::ConfigLoader;
        let unified_config = Arc::new(
            ConfigLoader::load()
                .map(|c| c.into_config())
                .unwrap_or_default()
        );
        
        // Create message channels
        let (out_tx, _out_rx) = mpsc::channel(config.buffer_size);
        let (_in_tx, in_rx) = mpsc::channel(config.buffer_size);
        
        // Create peer sender - this will be updated when connecting to a peer
        let (peer_tx, _peer_rx) = mpsc::channel(1);
        
        let mut metadata = TransportMetadata {
            connection_id: Uuid::new_v4().to_string(),
            remote_address: None,
            local_address: None,
            encryption_format: Some(config.encryption),
            compression_format: Some(config.compression),
            connected_at: Utc::now(),
            last_activity: Utc::now(),
            additional_info: HashMap::new(),
        };
        
        // Add memory-specific metadata
        metadata.additional_info.insert("transport_type".to_string(), "memory".to_string());
        metadata.additional_info.insert("memory_path".to_string(), format!("memory://{}", config.name));
        
        MemoryTransport {
            config: config.clone(),
            unified_config,
            state: Arc::new(RwLock::new(MemoryState::Disconnected)),
            outgoing_channel: out_tx,
            incoming_channel: Arc::new(TokioMutex::new(in_rx)),
            peer_sender: Arc::new(peer_tx),
            connection_id: Uuid::new_v4().to_string(),
            history: Arc::new(TokioMutex::new(VecDeque::with_capacity(config.max_history.unwrap_or(1000)))),
            max_history: config.max_history,
            metadata,
        }
    }
    
    /// Create a pair of memory transports directly
    #[must_use] pub fn create_transport_pair(self, config_a: Option<MemoryTransportConfig>, config_b: Option<MemoryTransportConfig>) -> (MemoryTransport, MemoryTransport) {
        let config_a = config_a.unwrap_or_default();
        let config_b = config_b.unwrap_or_default();
        
        // Load unified configuration for timeouts (shared between both transports)
        use squirrel_mcp_config::unified::ConfigLoader;
        let unified_config = Arc::new(
            ConfigLoader::load()
                .map(|c| c.into_config())
                .unwrap_or_default()
        );
        
        // Create channels for A -> B communication (Use bounded)
        let (a_to_b_tx, a_to_b_rx) = mpsc::channel(config_a.buffer_size);
        
        // Create channels for B -> A communication (Use bounded)
        let (b_to_a_tx, b_to_a_rx) = mpsc::channel(config_b.buffer_size);
        
        // Create metadata for transport A
        let mut metadata_a = TransportMetadata {
            connection_id: Uuid::new_v4().to_string(),
            remote_address: None,
            local_address: None,
            encryption_format: Some(config_a.encryption),
            compression_format: Some(config_a.compression),
            connected_at: Utc::now(),
            last_activity: Utc::now(),
            additional_info: HashMap::new(),
        };
        
        // Add memory-specific metadata for A
        metadata_a.additional_info.insert("transport_type".to_string(), "memory".to_string());
        metadata_a.additional_info.insert("memory_path".to_string(), format!("memory://{}", config_a.name));
        
        // Create metadata for transport B
        let mut metadata_b = TransportMetadata {
            connection_id: Uuid::new_v4().to_string(),
            remote_address: None,
            local_address: None,
            encryption_format: Some(config_b.encryption),
            compression_format: Some(config_b.compression),
            connected_at: Utc::now(),
            last_activity: Utc::now(),
            additional_info: HashMap::new(),
        };
        
        // Add memory-specific metadata for B
        metadata_b.additional_info.insert("transport_type".to_string(), "memory".to_string());
        metadata_b.additional_info.insert("memory_path".to_string(), format!("memory://{}", config_b.name));
        
        // Create transport A
        let transport_a = MemoryTransport {
            config: config_a.clone(),
            unified_config: Arc::clone(&unified_config),
            state: Arc::new(RwLock::new(MemoryState::Disconnected)),
            outgoing_channel: a_to_b_tx.clone(),
            incoming_channel: Arc::new(TokioMutex::new(b_to_a_rx)),
            peer_sender: Arc::new(b_to_a_tx.clone()),
            connection_id: Uuid::new_v4().to_string(),
            history: self.history.clone(),
            max_history: self.max_history,
            metadata: metadata_a,
        };
        
        // Create transport B
        let transport_b = MemoryTransport {
            config: config_b.clone(),
            unified_config,
            state: Arc::new(RwLock::new(MemoryState::Disconnected)),
            outgoing_channel: b_to_a_tx.clone(),
            incoming_channel: Arc::new(TokioMutex::new(a_to_b_rx)),
            peer_sender: Arc::new(a_to_b_tx.clone()),
            connection_id: Uuid::new_v4().to_string(),
            history: self.history.clone(),
            max_history: self.max_history,
            metadata: metadata_b,
        };
        
        (transport_a, transport_b)
    }
    
    /// Create a pair of memory transports with default configuration
    #[must_use] pub fn create_pair() -> (MemoryTransport, MemoryTransport) {
        let channel = Self::new(100, Some(100));
        
        let config_a = MemoryTransportConfig {
            name: format!("client-{}", Uuid::new_v4()),
            ..Default::default()
        };
        
        let config_b = MemoryTransportConfig {
            name: format!("server-{}", Uuid::new_v4()),
            ..Default::default()
        };
        
        channel.create_transport_pair(Some(config_a), Some(config_b))
    }
    
    /// Retrieve the message history
    pub async fn get_history(&self) -> Vec<MCPMessage> {
        let history = self.history.lock().await;
        history.iter().cloned().collect()
    }
    
    /// Clear the message history
    pub async fn clear_history(&self) {
        let mut history = self.history.lock().await;
        history.clear();
    }
    
    /// Create a pair of Arc-wrapped transports
    #[must_use] pub fn create_pair_arc() -> (Arc<dyn Transport>, Arc<dyn Transport>) {
        let channel = Self::new(100, Some(100));
        
        let config_a = MemoryTransportConfig {
            name: format!("client-{}", Uuid::new_v4()),
            ..Default::default()
        };
        
        let config_b = MemoryTransportConfig {
            name: format!("server-{}", Uuid::new_v4()),
            ..Default::default()
        };
        
        let (transport_a, transport_b) = channel.create_transport_pair(Some(config_a), Some(config_b));
        
        (Arc::new(transport_a), Arc::new(transport_b))
    }

    /// Create a pair of Arc-wrapped transports that are already connected
    #[must_use] pub fn create_connected_pair_arc() -> (Arc<dyn Transport>, Arc<dyn Transport>) {
        let channel = Self::new(100, Some(100));
        
        let config_a = MemoryTransportConfig {
            name: format!("client-{}", Uuid::new_v4()),
            ..Default::default()
        };
        
        let config_b = MemoryTransportConfig {
            name: format!("server-{}", Uuid::new_v4()),
            ..Default::default()
        };
        
        let (mut transport_a, mut transport_b) = channel.create_transport_pair(Some(config_a), Some(config_b));
        
        // Connect the transports before wrapping in Arc
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                transport_a.connect().await.expect("Failed to connect transport A");
                transport_b.connect().await.expect("Failed to connect transport B");
            })
        });
        
        (Arc::new(transport_a), Arc::new(transport_b))
    }
}

/// In-memory transport implementation
#[derive(Debug)]
pub struct MemoryTransport {
    /// Transport configuration
    config: MemoryTransportConfig,
    
    /// Unified system configuration (for timeouts, etc.)
    unified_config: Arc<SquirrelUnifiedConfig>,
    
    /// Current connection state
    state: Arc<RwLock<MemoryState>>,
    
    /// Outgoing message channel (Use standard Sender)
    outgoing_channel: Sender<MCPMessage>,
    
    /// Incoming message channel (Use standard Receiver wrapped in Tokio Mutex)
    incoming_channel: Arc<TokioMutex<Receiver<MCPMessage>>>,
    
    /// Sender to the peer transport (Use standard Sender)
    peer_sender: Arc<Sender<MCPMessage>>,
    
    /// Connection ID
    connection_id: String,
    
    /// Message history
    history: Arc<TokioMutex<VecDeque<MCPMessage>>>,
    
    /// Maximum history size
    max_history: Option<usize>,
    
    /// Transport metadata
    metadata: TransportMetadata,
}

impl MemoryTransport {
    /// Creates a new memory transport layer.
    #[must_use]
    pub fn new(config: &MemoryTransportConfig) -> Self {
        // Load unified configuration for timeouts
        use squirrel_mcp_config::unified::ConfigLoader;
        let unified_config = Arc::new(
            ConfigLoader::load()
                .map(|c| c.into_config())
                .unwrap_or_default()
        );
        
        Self::new_with_unified_config(config, unified_config)
    }
    
    /// Creates a new memory transport layer with provided unified config.
    #[must_use]
    pub fn new_with_unified_config(config: &MemoryTransportConfig, unified_config: Arc<SquirrelUnifiedConfig>) -> Self {
        // Create communication channels (Standard channels)
        let (out_tx, _out_rx) = mpsc::channel(config.buffer_size);
        let (_incoming_tx, incoming_rx): (Sender<MCPMessage>, Receiver<MCPMessage>) = mpsc::channel(config.buffer_size);
        
        // Create peer sender - this will be updated when connecting to a peer
        let (peer_tx, _peer_rx): (Sender<MCPMessage>, Receiver<MCPMessage>) = mpsc::channel(1);
        
        let mut metadata = TransportMetadata {
            connection_id: Uuid::new_v4().to_string(),
            remote_address: None,
            local_address: None,
            encryption_format: Some(config.encryption),
            compression_format: Some(config.compression),
            connected_at: Utc::now(),
            last_activity: Utc::now(),
            additional_info: HashMap::new(),
        };
        
        // Add memory-specific metadata
        metadata.additional_info.insert("transport_type".to_string(), "memory".to_string());
        metadata.additional_info.insert("memory_path".to_string(), format!("memory://{}", config.name));
        
        Self {
            config: config.clone(),
            unified_config,
            state: Arc::new(RwLock::new(MemoryState::Disconnected)),
            outgoing_channel: out_tx,
            incoming_channel: Arc::new(TokioMutex::new(incoming_rx)),
            peer_sender: Arc::new(peer_tx),
            connection_id: Uuid::new_v4().to_string(),
            history: Arc::new(TokioMutex::new(VecDeque::with_capacity(config.max_history.unwrap_or(1000)))),
            max_history: config.max_history,
            metadata,
        }
    }
    
    /// Add a message to the history, respecting max_history size
    async fn add_to_history(&self, message: MCPMessage) {
        if let Some(max) = self.max_history {
            let mut history = self.history.lock().await;
            while history.len() >= max {
                history.pop_front(); // Remove oldest message
            }
            history.push_back(message);
        }
    }
    
    /// Simulate network latency
    async fn simulate_latency(&self) {
        if let Some(latency) = self.config.simulated_latency_ms {
            tokio::time::sleep(tokio::time::Duration::from_millis(latency)).await;
        }
    }
}

#[async_trait]
impl Transport for MemoryTransport {
    async fn send_message(&self, message: MCPMessage) -> Result<()> {
        trace!("MemoryTransport [{}] sending message to peer", self.connection_id);
        
        // Simulate network latency before sending
        self.simulate_latency().await;
        
        let peer_sender = self.peer_sender.clone();
        peer_sender.send(message).await.map_err(|e| {
            MCPError::Transport(format!("Failed to send message: {}", e).into())
        })?;
        Ok(())
    }
    
    async fn receive_message(&self) -> Result<MCPMessage> {
        // Check if connected
        if !self.is_connected().await {
            return Err(MCPError::Transport(TransportError::ConnectionClosed("Not connected".to_string())).into());
        }

        // Acquire the Tokio Mutex lock asynchronously
        let mut rx_guard = self.incoming_channel.lock().await;
        
        // Receive the message asynchronously with a timeout
        let receive_future = rx_guard.recv();
        
        // Use unified config timeout (environment-aware)
        let timeout = self.unified_config.timeouts.operation_timeout();
        match tokio::time::timeout(timeout, receive_future).await {
            Ok(Some(message)) => {
                // Simulate latency for receiving
                self.simulate_latency().await;
                
                // Add received message to history
                self.add_to_history(message.clone()).await;
                
                Ok(message)
            }
            Ok(None) => {
                // Channel closed
                let state = self.state.read().await;
                error!("MemoryTransport [{}]: Incoming channel closed. State: {:?}", self.connection_id, *state);
                Err(MCPError::Transport(TransportError::ConnectionClosed("Channel closed".to_string())).into())
            }
            Err(_) => {
                // Timeout occurred
                error!("MemoryTransport [{}]: Receive operation timed out after 5 seconds", self.connection_id);
                Err(MCPError::Transport(TransportError::Timeout("Receive operation timed out".to_string())).into())
            }
        }
    }
    
    async fn connect(&mut self) -> Result<()> {
        // Get current state
        let state = {
            let state = self.state.read().await;
            state.clone()
        };
        
        // Already connected?
        if state == MemoryState::Connected {
            return Ok(());
        }
        
        // Only allow connecting from Disconnected state
        if state != MemoryState::Disconnected {
            return Err(MCPError::Transport(format!("Cannot connect from state: {state:?}").into()));
        }
        
        // Update state
        {
            let mut state = self.state.write().await;
            *state = MemoryState::Connecting;
        }
        
        // Simulate latency if configured
        self.simulate_latency().await;
        
        // Simulate random failures if configured
        if self.config.simulate_failures && rand::random::<f32>() < 0.1 {
            // 10% chance of failure
            // Update state directly without creating a temporary variable
            *self.state.write().await = MemoryState::Failed("Simulated random failure".to_string());
            
            return Err(MCPError::Transport(TransportError::ConnectionFailed("Simulated random connection failure".to_string())).into());
        }
        
        // Update state to connected
        {
            let mut state = self.state.write().await;
            *state = MemoryState::Connected;
        }
        
        Ok(())
    }

    async fn disconnect(&self) -> Result<()> {
        // Update state to disconnecting
        {
            let mut state = self.state.write().await;
            *state = MemoryState::Disconnecting;
        }
        
        // Simulate network latency
        self.simulate_latency().await;
        
        // Update state to disconnected
        {
            let mut state = self.state.write().await;
            *state = MemoryState::Disconnected;
        }
        
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        let state = self.state.read().await;
        matches!(*state, MemoryState::Connected)
    }

    async fn get_metadata(&self) -> crate::transport::types::TransportMetadata {
        self.metadata.clone()
    }

    // Add placeholder implementation for send_raw
    async fn send_raw(&self, _bytes: &[u8]) -> crate::error::Result<()> {
        // Sending raw bytes doesn't make sense for memory transport?
        // Or should we try to deserialize to MCPMessage first?
        error!("send_raw is not supported for MemoryTransport");
        Err(MCPError::UnsupportedOperation("send_raw not supported".to_string()).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use crate::protocol::types::MessageType;
    
    #[tokio::test]
    async fn test_memory_transport_create() {
        // Create a channel
        let channel = MemoryChannel::new(100, Some(10));
        
        // Create config
        let config = MemoryTransportConfig {
            name: "test".to_string(),
            ..Default::default()
        };
        
        // Create a transport
        let transport = channel.create_transport(config);
        
        // Verify initial state
        assert!(!transport.is_connected().await);
        
        // Check metadata
        let metadata = transport.get_metadata().await;
        assert_eq!(metadata.additional_info.get("transport_type").unwrap_or(&"".to_string()), "memory");
        assert!(metadata.additional_info.get("memory_path").map_or(false, |p| p.contains("memory://test")));
    }
    
    #[tokio::test]
    async fn test_memory_transport_pair() {
        // Create a channel
        let channel = MemoryChannel::new(100, Some(10));
        
        // Create channels for direct communication
        let (a_to_b_tx, a_to_b_rx) = mpsc::channel(100);
        let (b_to_a_tx, b_to_a_rx) = mpsc::channel(100);
        
        // Create config for both sides
        let config_a = MemoryTransportConfig {
            name: "client".to_string(),
            ..Default::default()
        };
        
        let config_b = MemoryTransportConfig {
            name: "server".to_string(),
            ..Default::default()
        };
        
        // Create metadata for transport A
        let mut metadata_a = TransportMetadata {
            connection_id: Uuid::new_v4().to_string(),
            remote_address: None,
            local_address: None,
            encryption_format: Some(config_a.encryption),
            compression_format: Some(config_a.compression),
            connected_at: Utc::now(),
            last_activity: Utc::now(),
            additional_info: HashMap::new(),
        };
        
        // Add memory-specific metadata for A
        metadata_a.additional_info.insert("transport_type".to_string(), "memory".to_string());
        metadata_a.additional_info.insert("memory_path".to_string(), format!("memory://{}", config_a.name));
        
        // Create metadata for transport B
        let mut metadata_b = TransportMetadata {
            connection_id: Uuid::new_v4().to_string(),
            remote_address: None,
            local_address: None,
            encryption_format: Some(config_b.encryption),
            compression_format: Some(config_b.compression),
            connected_at: Utc::now(),
            last_activity: Utc::now(),
            additional_info: HashMap::new(),
        };
        
        // Add memory-specific metadata for B
        metadata_b.additional_info.insert("transport_type".to_string(), "memory".to_string());
        metadata_b.additional_info.insert("memory_path".to_string(), format!("memory://{}", config_b.name));
        
        // Create transport A (client)
        let mut client = MemoryTransport {
            config: config_a.clone(),
            state: Arc::new(RwLock::new(MemoryState::Disconnected)),
            outgoing_channel: a_to_b_tx.clone(),
            incoming_channel: Arc::new(TokioMutex::new(b_to_a_rx)),
            peer_sender: Arc::new(a_to_b_tx.clone()),
            connection_id: Uuid::new_v4().to_string(),
            history: channel.history.clone(),
            max_history: channel.max_history,
            metadata: metadata_a,
        };
        
        // Create transport B (server)
        let mut server = MemoryTransport {
            config: config_b.clone(),
            state: Arc::new(RwLock::new(MemoryState::Disconnected)),
            outgoing_channel: b_to_a_tx.clone(),
            incoming_channel: Arc::new(TokioMutex::new(a_to_b_rx)),
            peer_sender: Arc::new(b_to_a_tx.clone()),
            connection_id: Uuid::new_v4().to_string(),
            history: channel.history.clone(),
            max_history: channel.max_history,
            metadata: metadata_b,
        };
        
        // Connect both sides
        client.connect().await.expect("should succeed");
        server.connect().await.expect("should succeed");
        
        // Check that both are connected
        assert!(client.is_connected().await);
        assert!(server.is_connected().await);
        
        // Send message from client to server
        let client_msg = MCPMessage::new(
            MessageType::Command,
            serde_json::json!({ "action": "test", "value": 42 }),
        );
        
        client.send_message(client_msg.clone()).await.expect("should succeed");
        
        // Receive on server side with timeout (increase timeout to 10 seconds)
        let received = tokio::time::timeout(
            Duration::from_secs(10), 
            server.receive_message()
        ).await.expect("should succeed").expect("should succeed");
        
        // Verify contents
        assert_eq!(received.id, client_msg.id);
        assert_eq!(received.type_, client_msg.type_);
        assert_eq!(
            received.payload.get("action").and_then(|v| v.as_str()),
            Some("test")
        );
        assert_eq!(
            received.payload.get("value").and_then(|v| v.as_i64()),
            Some(42)
        );
        
        // Send response
        let server_msg = MCPMessage::new(
            MessageType::Response,
            serde_json::json!({ "result": "ok" }),
        );
        
        server.send_message(server_msg.clone()).await.expect("should succeed");
        
        // Receive on client side (increase timeout to 10 seconds)
        let received = tokio::time::timeout(
            Duration::from_secs(10), 
            client.receive_message()
        ).await.expect("should succeed").expect("should succeed");
        
        // Verify message contents
        assert_eq!(received.id, server_msg.id);
        assert_eq!(received.type_, server_msg.type_);
        
        // Check history (which should have both messages)
        let history = channel.get_history().await;
        assert_eq!(history.len(), 2);
    }
    
    #[tokio::test]
    async fn test_memory_transport_with_latency() {
        // Create a channel
        let channel = MemoryChannel::new(100, Some(10));
        
        // Create channels for direct communication
        let (a_to_b_tx, a_to_b_rx) = mpsc::channel(100);
        let (b_to_a_tx, b_to_a_rx) = mpsc::channel(100);
        
        // Create config for both sides with higher latency to ensure it's measurable
        let config_a = MemoryTransportConfig {
            name: "client".to_string(),
            simulated_latency_ms: Some(100), // Increase latency to be more reliably measurable
            ..Default::default()
        };
        
        let config_b = MemoryTransportConfig {
            name: "server".to_string(),
            ..Default::default()
        };
        
        // Create metadata for transport A
        let mut metadata_a = TransportMetadata {
            connection_id: Uuid::new_v4().to_string(),
            remote_address: None,
            local_address: None,
            encryption_format: Some(config_a.encryption),
            compression_format: Some(config_a.compression),
            connected_at: Utc::now(),
            last_activity: Utc::now(),
            additional_info: HashMap::new(),
        };
        
        // Add memory-specific metadata for A
        metadata_a.additional_info.insert("transport_type".to_string(), "memory".to_string());
        metadata_a.additional_info.insert("memory_path".to_string(), format!("memory://{}", config_a.name));
        
        // Create metadata for transport B
        let mut metadata_b = TransportMetadata {
            connection_id: Uuid::new_v4().to_string(),
            remote_address: None,
            local_address: None,
            encryption_format: Some(config_b.encryption),
            compression_format: Some(config_b.compression),
            connected_at: Utc::now(),
            last_activity: Utc::now(),
            additional_info: HashMap::new(),
        };
        
        // Add memory-specific metadata for B
        metadata_b.additional_info.insert("transport_type".to_string(), "memory".to_string());
        metadata_b.additional_info.insert("memory_path".to_string(), format!("memory://{}", config_b.name));
        
        // Create transport A (client) with simulated latency
        let mut client = MemoryTransport {
            config: config_a.clone(),
            state: Arc::new(RwLock::new(MemoryState::Disconnected)),
            outgoing_channel: a_to_b_tx.clone(),
            incoming_channel: Arc::new(TokioMutex::new(b_to_a_rx)),
            peer_sender: Arc::new(a_to_b_tx.clone()),
            connection_id: Uuid::new_v4().to_string(),
            history: channel.history.clone(),
            max_history: channel.max_history,
            metadata: metadata_a,
        };
        
        // Create transport B (server)
        let mut server = MemoryTransport {
            config: config_b.clone(),
            state: Arc::new(RwLock::new(MemoryState::Disconnected)),
            outgoing_channel: b_to_a_tx.clone(),
            incoming_channel: Arc::new(TokioMutex::new(a_to_b_rx)),
            peer_sender: Arc::new(b_to_a_tx.clone()),
            connection_id: Uuid::new_v4().to_string(),
            history: channel.history.clone(),
            max_history: channel.max_history,
            metadata: metadata_b,
        };
        
        // Connect both sides
        client.connect().await.expect("should succeed");
        server.connect().await.expect("should succeed");
        
        // Test direct latency application
        let start = tokio::time::Instant::now();
        client.simulate_latency().await;
        let direct_latency = start.elapsed();
        
        // Verify latency is working directly
        assert!(direct_latency >= tokio::time::Duration::from_millis(100),
               "Direct latency test failed: expected at least 100ms, got {:?}", direct_latency);
        
        // Create a message to send
        let client_msg = MCPMessage::new(
            MessageType::Command,
            serde_json::json!({ "test": "latency" }),
        );
        
        // Measure the time it takes to send and receive
        let start = tokio::time::Instant::now();
        
        // Send the message (should apply latency internally)
        client.send_message(client_msg.clone()).await.expect("should succeed");
        
        // Receive with extended timeout
        let _ = tokio::time::timeout(
            Duration::from_secs(10),
            server.receive_message()
        ).await.expect("should succeed").expect("should succeed");
        
        let elapsed = start.elapsed();
        
        // The simulated latency should be visible in the elapsed time
        assert!(elapsed >= tokio::time::Duration::from_millis(100), 
                "Expected at least 100ms latency, but got {:?}", elapsed);
        
        // Let's also confirm that the config has the expected latency value
        assert_eq!(client.config.simulated_latency_ms, Some(100),
                 "Client config has wrong latency value: {:?}", client.config.simulated_latency_ms);
    }

    #[tokio::test]
    async fn test_memory_transport_creation() {
        let (_transport_a, _transport_b) = MemoryChannel::create_pair();
        // ... existing code ...
    }

    #[tokio::test]
    async fn test_memory_transport_with_custom_config() {
        let channel = MemoryChannel::new(100, Some(10));
        let config_a = MemoryTransportConfig {
            name: "client".to_string(),
            ..Default::default()
        };
        let config_b = MemoryTransportConfig {
            name: "server".to_string(),
            ..Default::default()
        };

        let (_client, _server) = channel.create_transport_pair(Some(config_a), Some(config_b));
        // ... existing code ...
    }

    #[tokio::test]
    async fn test_memory_transport_with_custom_config_bidirectional() {
        let channel = MemoryChannel::new(100, Some(10));
        let config_a = MemoryTransportConfig {
            name: "client".to_string(),
            ..Default::default()
        };
        let config_b = MemoryTransportConfig {
            name: "server".to_string(),
            ..Default::default()
        };

        let (_client, _server) = channel.create_transport_pair(Some(config_a), Some(config_b));
        // ... existing code ...
    }
} 