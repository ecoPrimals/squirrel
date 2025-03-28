use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::{mpsc, RwLock, Mutex};
use uuid::Uuid;
use std::collections::VecDeque;
use rand;

use crate::error::transport::TransportError;
use crate::types::{MCPMessage, EncryptionFormat, CompressionFormat};
use super::{Transport, TransportMetadata};
use crate::error::MCPError;

/// Configuration for the in-memory transport
#[derive(Debug, Clone)]
pub struct MemoryTransportConfig {
    /// Transport name, useful for debugging
    pub name: String,
    
    /// Channel buffer size
    pub buffer_size: usize,
    
    /// Maximum message count in history
    pub max_history: Option<usize>,
    
    /// Simulate latency in milliseconds
    pub simulated_latency_ms: Option<u64>,
    
    /// Simulate random connection failures
    pub simulate_failures: bool,
    
    /// Encryption format (for metadata only)
    pub encryption: EncryptionFormat,
    
    /// Compression format (for metadata only)
    pub compression: CompressionFormat,
}

impl Default for MemoryTransportConfig {
    fn default() -> Self {
        Self {
            name: "memory".to_string(),
            buffer_size: 100,
            max_history: Some(1000),
            simulated_latency_ms: None,
            simulate_failures: false,
            encryption: EncryptionFormat::None,
            compression: CompressionFormat::None,
        }
    }
}

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
    history: Arc<Mutex<VecDeque<MCPMessage>>>,
    
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
            history: Arc::new(Mutex::new(VecDeque::new())),
            max_history,
        }
    }
    
    /// Create a single memory transport with the given configuration
    #[must_use] pub fn create_transport(&self, config: MemoryTransportConfig) -> MemoryTransport {
        // Create message channels
        let (out_tx, _) = mpsc::channel(config.buffer_size);
        let (_, in_rx) = mpsc::channel(config.buffer_size);
        
        // Create peer sender - this will be updated when connecting to a peer
        let (peer_tx, _) = mpsc::channel(1);
        
        let metadata = TransportMetadata {
            transport_type: "memory".to_string(),
            remote_address: format!("memory://{}", config.name),
            local_address: None,
            encryption: config.encryption,
            compression: config.compression,
        };
        
        MemoryTransport {
            config: config,
            state: Arc::new(RwLock::new(MemoryState::Disconnected)),
            outgoing_channel: out_tx,
            incoming_channel: Arc::new(Mutex::new(in_rx)),
            peer_sender: Arc::new(peer_tx),
            connection_id: Uuid::new_v4().to_string(),
            history: self.history.clone(),
            max_history: self.max_history,
            metadata,
        }
    }
    
    /// Create a pair of memory transports directly
    #[must_use] pub fn create_transport_pair(self, config_a: Option<MemoryTransportConfig>, config_b: Option<MemoryTransportConfig>) -> (MemoryTransport, MemoryTransport) {
        let config_a = config_a.unwrap_or_default();
        let config_b = config_b.unwrap_or_default();
        
        // Create channels for A -> B communication
        let (a_to_b_tx, a_to_b_rx) = mpsc::channel(config_a.buffer_size);
        
        // Create channels for B -> A communication
        let (b_to_a_tx, b_to_a_rx) = mpsc::channel(config_b.buffer_size);
        
        // Create transport A
        let transport_a = MemoryTransport {
            config: config_a.clone(),
            state: Arc::new(RwLock::new(MemoryState::Disconnected)),
            outgoing_channel: a_to_b_tx.clone(),
            incoming_channel: Arc::new(Mutex::new(a_to_b_rx)),
            peer_sender: Arc::new(a_to_b_tx),
            connection_id: Uuid::new_v4().to_string(),
            history: self.history.clone(),
            max_history: self.max_history,
            metadata: TransportMetadata {
                transport_type: "memory".to_string(),
                remote_address: format!("memory://{}", config_a.name),
                local_address: None,
                encryption: config_a.encryption,
                compression: config_a.compression,
            },
        };
        
        // Create transport B
        let transport_b = MemoryTransport {
            config: config_b.clone(),
            state: Arc::new(RwLock::new(MemoryState::Disconnected)),
            outgoing_channel: b_to_a_tx.clone(),
            incoming_channel: Arc::new(Mutex::new(b_to_a_rx)),
            peer_sender: Arc::new(b_to_a_tx),
            connection_id: Uuid::new_v4().to_string(),
            history: self.history.clone(),
            max_history: self.max_history,
            metadata: TransportMetadata {
                transport_type: "memory".to_string(),
                remote_address: format!("memory://{}", config_b.name),
                local_address: None,
                encryption: config_b.encryption,
                compression: config_b.compression,
            },
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
    
    /// Get a reference to the message history
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
}

/// In-memory transport implementation
pub struct MemoryTransport {
    /// Transport configuration
    config: MemoryTransportConfig,
    
    /// Current connection state
    state: Arc<RwLock<MemoryState>>,
    
    /// Outgoing message channel
    outgoing_channel: mpsc::Sender<MCPMessage>,
    
    /// Incoming message channel
    incoming_channel: Arc<Mutex<mpsc::Receiver<MCPMessage>>>,
    
    /// Sender to the peer transport
    peer_sender: Arc<mpsc::Sender<MCPMessage>>,
    
    /// Connection ID
    connection_id: String,
    
    /// Message history
    history: Arc<Mutex<VecDeque<MCPMessage>>>,
    
    /// Maximum history size
    max_history: Option<usize>,
    
    /// Transport metadata
    metadata: TransportMetadata,
}

impl MemoryTransport {
    /// Create a new memory transport
    #[must_use] pub fn new(config: MemoryTransportConfig) -> Self {
        // Create message channels
        let (out_tx, _) = mpsc::channel(config.buffer_size);
        let (_, in_rx) = mpsc::channel(config.buffer_size);
        
        // Create peer sender - this will be updated when connecting to a peer
        let (peer_tx, _) = mpsc::channel(1);
        
        let metadata = TransportMetadata {
            transport_type: "memory".to_string(),
            remote_address: format!("memory://{}", config.name),
            local_address: None,
            encryption: config.encryption,
            compression: config.compression,
        };
        
        Self {
            config: config.clone(),
            state: Arc::new(RwLock::new(MemoryState::Disconnected)),
            outgoing_channel: out_tx,
            incoming_channel: Arc::new(Mutex::new(in_rx)),
            peer_sender: Arc::new(peer_tx),
            connection_id: Uuid::new_v4().to_string(),
            history: Arc::new(Mutex::new(VecDeque::new())),
            max_history: config.max_history,
            metadata,
        }
    }
    
    /// Add a message to the history
    async fn add_to_history(&self, message: MCPMessage) {
        let mut history = self.history.lock().await;
        
        // Add the message to history
        history.push_back(message);
        
        // Enforce maximum history size
        if let Some(max) = self.max_history {
            while history.len() > max {
                history.pop_front();
            }
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
    async fn send_message(&self, message: MCPMessage) -> crate::error::Result<()> {
        if !self.is_connected().await {
            return Err(MCPError::Transport(TransportError::ConnectionClosed("Not connected".to_string()).into()));
        }
        
        // Clone the message for history
        let message_copy = message.clone();
        
        // Add to history before sending
        self.add_to_history(message_copy).await;
        
        // Simulate latency
        self.simulate_latency().await;
        
        // Simulate random failures
        if self.config.simulate_failures && rand::random::<f32>() < 0.05 {
            // 5% chance of failure
            return Err(MCPError::Transport(TransportError::ConnectionClosed("Simulated random send failure".to_string()).into()));
        }
        
        // Send to peer
        let peer_sender = self.peer_sender.clone();
        peer_sender.send(message).await
            .map_err(|_| MCPError::Transport(TransportError::ConnectionClosed("Failed to send message to peer".to_string()).into()))
    }
    
    async fn receive_message(&self) -> crate::error::Result<MCPMessage> {
        // Check if connected
        if !self.is_connected().await {
            return Err(MCPError::Transport(TransportError::ConnectionClosed("Not connected".to_string()).into()));
        }
        
        // Get a message from the incoming channel
        let mut rx = self.incoming_channel.lock().await;
        match rx.recv().await {
            Some(message) => {
                // Simulate latency for receiving
                self.simulate_latency().await;
                
                // Add received message to history
                self.add_to_history(message.clone()).await;
                
                Ok(message)
            },
            None => Err(MCPError::Transport(TransportError::ConnectionClosed("Channel closed".to_string()).into())),
        }
    }
    
    async fn connect(&mut self) -> crate::error::Result<()> {
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
            return Err(MCPError::Transport(TransportError::ConnectionFailed(
                format!("Cannot connect from state: {state:?}")
            ).into()));
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
            let mut state = self.state.write().await;
            *state = MemoryState::Failed("Simulated random failure".to_string());
            
            return Err(MCPError::Transport(TransportError::ConnectionFailed(
                "Simulated random connection failure".to_string()
            ).into()));
        }
        
        // Update state to connected
        {
            let mut state = self.state.write().await;
            *state = MemoryState::Connected;
        }
        
        Ok(())
    }

    async fn disconnect(&self) -> crate::error::Result<()> {
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
        *state == MemoryState::Connected
    }

    fn get_metadata(&self) -> TransportMetadata {
        self.metadata.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;
    use std::time::Duration;
    use crate::types::MessageType;
    
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
        let metadata = transport.get_metadata();
        assert_eq!(metadata.transport_type, "memory");
        assert!(metadata.remote_address.contains("memory://test"));
    }
    
    #[tokio::test]
    async fn test_memory_transport_pair() {
        // Create a pair of transports
        let (transport_a, transport_b) = MemoryChannel::create_pair();
        
        // Connect both sides
        transport_a.connect().await.unwrap();
        transport_b.connect().await.unwrap();
        
        // Check that both are connected
        assert!(transport_a.is_connected().await);
        assert!(transport_b.is_connected().await);
        
        // Send a message from A to B
        let message = MCPMessage::new(
            MessageType::Command,
            serde_json::json!({
                "action": "test",
                "value": 42
            })
        );
        
        transport_a.send_message(message.clone()).await.unwrap();
        
        // Receive the message on B
        let received = transport_b.receive_message().await.unwrap();
        
        // Verify contents
        assert_eq!(received.id, message.id);
        assert_eq!(received.type_, message.type_);
        assert_eq!(
            received.payload.get("action").and_then(|v| v.as_str()),
            Some("test")
        );
        assert_eq!(
            received.payload.get("value").and_then(|v| v.as_i64()),
            Some(42)
        );
    }
    
    #[tokio::test]
    async fn test_memory_transport_with_latency() {
        // Create a channel
        let channel = MemoryChannel::new(100, Some(10));
        
        // Create configs
        let config_a = MemoryTransportConfig {
            name: "client".to_string(),
            simulated_latency_ms: Some(50), // 50ms latency
            ..Default::default()
        };
        
        let config_b = MemoryTransportConfig {
            name: "server".to_string(),
            ..Default::default()
        };
        
        // Create transport pair
        let (client, server) = channel.create_transport_pair(Some(config_a), Some(config_b));
        
        // Connect both
        client.connect().await.unwrap();
        server.connect().await.unwrap();
        
        // Send message with timing
        let start = tokio::time::Instant::now();
        
        client.send_message(MCPMessage::new(
            MessageType::Command,
            serde_json::json!({ "test": "latency" })
        )).await.unwrap();
        
        // Receive should take at least the simulated latency
        let _ = server.receive_message().await.unwrap();
        let elapsed = start.elapsed();
        
        assert!(elapsed >= tokio::time::Duration::from_millis(50));
    }
} 