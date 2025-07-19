//! Federation Network Module
//!
//! This module handles networking and communication between federation nodes.
//! It provides secure, reliable communication channels for the federation
//! with support for multiple protocols and encryption.

use super::{FederationError, FederationResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;
use uuid::Uuid;

/// Federation network configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Network protocol to use
    pub protocol: NetworkProtocol,
    /// Port for federation communication
    pub port: u16,
    /// Encryption settings
    pub encryption_enabled: bool,
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Heartbeat interval in seconds
    pub heartbeat_interval: u64,
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Node discovery timeout in seconds
    pub discovery_timeout: u64,
}

/// Network protocols for federation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkProtocol {
    /// HTTP/HTTPS protocol
    Http,
    /// gRPC protocol
    Grpc,
    /// WebSocket protocol
    WebSocket,
    /// Custom protocol
    Custom(String),
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            protocol: NetworkProtocol::Http,
            port: 8080,
            encryption_enabled: true,
            max_connections: 1000,
            connection_timeout: 30,
            heartbeat_interval: 10,
            max_message_size: 1024 * 1024, // 1MB
            discovery_timeout: 5,
        }
    }
}

/// Federation network message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// Node discovery request
    Discovery { node_id: Uuid, node_info: NodeInfo },
    /// Node discovery response
    DiscoveryResponse {
        node_id: Uuid,
        node_info: NodeInfo,
        peers: Vec<PeerInfo>,
    },
    /// Consensus voting message
    ConsensusVote {
        proposal_id: Uuid,
        vote: bool,
        node_id: Uuid,
        timestamp: DateTime<Utc>,
    },
    /// Data synchronization message
    DataSync {
        operation: DataOperation,
        data: Vec<u8>,
        checksum: String,
    },
    /// Health check message
    HealthCheck {
        node_id: Uuid,
        timestamp: DateTime<Utc>,
    },
    /// Generic federation message
    Federation {
        message_type: String,
        payload: Vec<u8>,
        sender: Uuid,
        recipient: Option<Uuid>,
    },
}

/// Node information for network discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub endpoints: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// Peer information for network management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: Uuid,
    pub address: SocketAddr,
    pub last_seen: DateTime<Utc>,
    pub status: PeerStatus,
    pub latency: Option<Duration>,
}

/// Peer connection status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PeerStatus {
    Connected,
    Disconnected,
    Connecting,
    Error(String),
}

/// Data operation types for synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataOperation {
    Create,
    Read,
    Update,
    Delete,
    Sync,
}

/// Network connection interface
#[async_trait]
pub trait NetworkConnection: Send + Sync {
    /// Send a message to a peer
    async fn send_message(&self, peer_id: Uuid, message: NetworkMessage) -> FederationResult<()>;

    /// Receive a message from the network
    async fn receive_message(&self) -> FederationResult<(Uuid, NetworkMessage)>;

    /// Check if connection is alive
    async fn is_connected(&self) -> bool;

    /// Close the connection
    async fn close(&self) -> FederationResult<()>;
}

/// Federation network manager
pub struct FederationNetwork {
    config: NetworkConfig,
    node_id: Uuid,
    node_info: NodeInfo,
    peers: Arc<RwLock<HashMap<Uuid, PeerInfo>>>,
    connections: Arc<RwLock<HashMap<Uuid, Arc<dyn NetworkConnection>>>>,
    message_handlers: Arc<RwLock<HashMap<String, MessageHandler>>>,
    message_queue: Arc<RwLock<Vec<QueuedMessage>>>,
    running: Arc<RwLock<bool>>,
}

/// Message handler function type
type MessageHandler = Box<dyn Fn(NetworkMessage) -> FederationResult<()> + Send + Sync>;

/// Queued message for processing
#[derive(Debug, Clone)]
struct QueuedMessage {
    message: NetworkMessage,
    sender: Uuid,
    timestamp: DateTime<Utc>,
    retry_count: u32,
}

impl FederationNetwork {
    /// Create a new federation network
    pub fn new(config: NetworkConfig, node_info: NodeInfo) -> Self {
        Self {
            config,
            node_id: node_info.id,
            node_info,
            peers: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
            message_handlers: Arc::new(RwLock::new(HashMap::new())),
            message_queue: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the federation network
    pub async fn start(&self) -> FederationResult<()> {
        let mut running = self.running.write().await;
        if *running {
            return Err(FederationError::AlreadyRunning(
                "Network already running".to_string(),
            ));
        }
        *running = true;
        drop(running);

        // Start background tasks
        self.start_heartbeat_task().await;
        self.start_message_processing_task().await;
        self.start_peer_discovery_task().await;

        Ok(())
    }

    /// Stop the federation network
    pub async fn stop(&self) -> FederationResult<()> {
        let mut running = self.running.write().await;
        *running = false;

        // Close all connections
        let connections = self.connections.read().await;
        for connection in connections.values() {
            let _ = connection.close().await;
        }

        Ok(())
    }

    /// Register a message handler
    pub async fn register_handler<F>(
        &self,
        message_type: String,
        handler: F,
    ) -> FederationResult<()>
    where
        F: Fn(NetworkMessage) -> FederationResult<()> + Send + Sync + 'static,
    {
        let mut handlers = self.message_handlers.write().await;
        handlers.insert(message_type, Box::new(handler));
        Ok(())
    }

    /// Send a message to a specific peer
    pub async fn send_to_peer(
        &self,
        peer_id: Uuid,
        message: NetworkMessage,
    ) -> FederationResult<()> {
        let connections = self.connections.read().await;
        if let Some(connection) = connections.get(&peer_id) {
            connection.send_message(peer_id, message).await
        } else {
            Err(FederationError::PeerNotFound(peer_id.to_string()))
        }
    }

    /// Broadcast a message to all peers
    pub async fn broadcast(&self, message: NetworkMessage) -> FederationResult<()> {
        let connections = self.connections.read().await;
        let mut errors = Vec::new();

        for (peer_id, connection) in connections.iter() {
            if let Err(e) = connection.send_message(*peer_id, message.clone()).await {
                errors.push((*peer_id, e));
            }
        }

        if !errors.is_empty() {
            return Err(FederationError::BroadcastFailed(format!(
                "Failed to send to {} peers",
                errors.len()
            )));
        }

        Ok(())
    }

    /// Discover peers in the network
    pub async fn discover_peers(&self) -> FederationResult<Vec<PeerInfo>> {
        let discovery_msg = NetworkMessage::Discovery {
            node_id: self.node_id,
            node_info: self.node_info.clone(),
        };

        // Send discovery message to known peers
        let _ = self.broadcast(discovery_msg).await;

        // Wait for discovery timeout
        sleep(Duration::from_secs(self.config.discovery_timeout)).await;

        // Return current peer list
        let peers = self.peers.read().await;
        Ok(peers.values().cloned().collect())
    }

    /// Add a new peer to the network
    pub async fn add_peer(&self, peer_info: PeerInfo) -> FederationResult<()> {
        let mut peers = self.peers.write().await;
        peers.insert(peer_info.id, peer_info);
        Ok(())
    }

    /// Remove a peer from the network
    pub async fn remove_peer(&self, peer_id: Uuid) -> FederationResult<()> {
        let mut peers = self.peers.write().await;
        peers.remove(&peer_id);

        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.remove(&peer_id) {
            let _ = connection.close().await;
        }

        Ok(())
    }

    /// Get network statistics
    pub async fn get_stats(&self) -> NetworkStats {
        let peers = self.peers.read().await;
        let connections = self.connections.read().await;
        let queue = self.message_queue.read().await;

        NetworkStats {
            peer_count: peers.len(),
            connection_count: connections.len(),
            queued_messages: queue.len(),
            node_id: self.node_id,
            uptime: Utc::now(), // This would be calculated from start time
        }
    }

    /// Start heartbeat task for peer monitoring
    async fn start_heartbeat_task(&self) {
        let peers = Arc::clone(&self.peers);
        let connections = Arc::clone(&self.connections);
        let running = Arc::clone(&self.running);
        let node_id = self.node_id;
        let interval = self.config.heartbeat_interval;

        tokio::spawn(async move {
            while *running.read().await {
                let health_check = NetworkMessage::HealthCheck {
                    node_id,
                    timestamp: Utc::now(),
                };

                let conn_map = connections.read().await;
                for (peer_id, connection) in conn_map.iter() {
                    if let Err(_) = connection
                        .send_message(*peer_id, health_check.clone())
                        .await
                    {
                        // Mark peer as disconnected
                        let mut peer_map = peers.write().await;
                        if let Some(peer) = peer_map.get_mut(peer_id) {
                            peer.status = PeerStatus::Disconnected;
                        }
                    }
                }
                drop(conn_map);

                sleep(Duration::from_secs(interval)).await;
            }
        });
    }

    /// Start message processing task
    async fn start_message_processing_task(&self) {
        let handlers = Arc::clone(&self.message_handlers);
        let queue = Arc::clone(&self.message_queue);
        let running = Arc::clone(&self.running);

        tokio::spawn(async move {
            while *running.read().await {
                let messages = {
                    let mut q = queue.write().await;
                    let messages = q.drain(..).collect::<Vec<_>>();
                    messages
                };

                for queued_msg in messages {
                    let handler_map = handlers.read().await;
                    let message_type = match &queued_msg.message {
                        NetworkMessage::Discovery { .. } => "discovery",
                        NetworkMessage::DiscoveryResponse { .. } => "discovery_response",
                        NetworkMessage::ConsensusVote { .. } => "consensus_vote",
                        NetworkMessage::DataSync { .. } => "data_sync",
                        NetworkMessage::HealthCheck { .. } => "health_check",
                        NetworkMessage::Federation { message_type, .. } => message_type,
                    };

                    if let Some(handler) = handler_map.get(message_type) {
                        let _ = handler(queued_msg.message);
                    }
                    drop(handler_map);
                }

                sleep(Duration::from_millis(100)).await;
            }
        });
    }

    /// Start peer discovery task
    async fn start_peer_discovery_task(&self) {
        let _peers = Arc::clone(&self.peers);
        let running = Arc::clone(&self.running);

        tokio::spawn(async move {
            while *running.read().await {
                // Periodic peer discovery logic
                sleep(Duration::from_secs(60)).await;
            }
        });
    }
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub peer_count: usize,
    pub connection_count: usize,
    pub queued_messages: usize,
    pub node_id: Uuid,
    pub uptime: DateTime<Utc>,
}

/// Network connection implementation for testing
pub struct MockNetworkConnection {
    peer_id: Uuid,
    connected: Arc<RwLock<bool>>,
    message_queue: Arc<RwLock<Vec<NetworkMessage>>>,
}

impl MockNetworkConnection {
    pub fn new(peer_id: Uuid) -> Self {
        Self {
            peer_id,
            connected: Arc::new(RwLock::new(true)),
            message_queue: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl NetworkConnection for MockNetworkConnection {
    async fn send_message(&self, _peer_id: Uuid, message: NetworkMessage) -> FederationResult<()> {
        if !*self.connected.read().await {
            return Err(FederationError::ConnectionClosed(
                "Connection closed".to_string(),
            ));
        }

        let mut queue = self.message_queue.write().await;
        queue.push(message);
        Ok(())
    }

    async fn receive_message(&self) -> FederationResult<(Uuid, NetworkMessage)> {
        let mut queue = self.message_queue.write().await;
        if let Some(message) = queue.pop() {
            Ok((self.peer_id, message))
        } else {
            Err(FederationError::NoMessagesAvailable(
                "No messages".to_string(),
            ))
        }
    }

    async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }

    async fn close(&self) -> FederationResult<()> {
        let mut connected = self.connected.write().await;
        *connected = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_network_creation() {
        let config = NetworkConfig::default();
        let node_info = NodeInfo {
            id: Uuid::new_v4(),
            name: "test-node".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec!["test".to_string()],
            endpoints: vec!["http://localhost:8080".to_string()],
            metadata: HashMap::new(),
        };

        let network = FederationNetwork::new(config, node_info);
        assert!(!*network.running.read().await);
    }

    #[tokio::test]
    async fn test_peer_management() {
        let config = NetworkConfig::default();
        let node_info = NodeInfo {
            id: Uuid::new_v4(),
            name: "test-node".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec!["test".to_string()],
            endpoints: vec!["http://localhost:8080".to_string()],
            metadata: HashMap::new(),
        };

        let network = FederationNetwork::new(config, node_info);

        let peer_info = PeerInfo {
            id: Uuid::new_v4(),
            address: "127.0.0.1:8080".parse().unwrap(),
            last_seen: Utc::now(),
            status: PeerStatus::Connected,
            latency: Some(Duration::from_millis(50)),
        };

        network.add_peer(peer_info.clone()).await.unwrap();

        let stats = network.get_stats().await;
        assert_eq!(stats.peer_count, 1);

        network.remove_peer(peer_info.id).await.unwrap();

        let stats = network.get_stats().await;
        assert_eq!(stats.peer_count, 0);
    }

    #[tokio::test]
    async fn test_message_handling() {
        let config = NetworkConfig::default();
        let node_info = NodeInfo {
            id: Uuid::new_v4(),
            name: "test-node".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec!["test".to_string()],
            endpoints: vec!["http://localhost:8080".to_string()],
            metadata: HashMap::new(),
        };

        let network = FederationNetwork::new(config, node_info);

        let handler_called = Arc::new(RwLock::new(false));
        let handler_called_clone = Arc::clone(&handler_called);

        network
            .register_handler("test".to_string(), move |_msg| {
                let handler_called_clone = handler_called_clone.clone();
                tokio::spawn(async move {
                    let mut called = handler_called_clone.write().await;
                    *called = true;
                });
                Ok(())
            })
            .await
            .unwrap();

        // Test that handler was registered
        let handlers = network.message_handlers.read().await;
        assert!(handlers.contains_key("test"));
    }
}
