// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

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
    Discovery {
        /// ID of the discovering node
        node_id: Uuid,
        /// Information about the discovering node
        node_info: NodeInfo,
    },
    /// Node discovery response
    DiscoveryResponse {
        /// ID of the responding node
        node_id: Uuid,
        /// Information about the responding node
        node_info: NodeInfo,
        /// List of known peer nodes
        peers: Vec<PeerInfo>,
    },
    /// Consensus voting message
    ConsensusVote {
        /// ID of the proposal being voted on
        proposal_id: Uuid,
        /// The vote (true for yes, false for no)
        vote: bool,
        /// ID of the voting node
        node_id: Uuid,
        /// Timestamp when the vote was cast
        timestamp: DateTime<Utc>,
    },
    /// Data synchronization message
    DataSync {
        /// The type of data operation
        operation: DataOperation,
        /// The data being synchronized
        data: Vec<u8>,
        /// Checksum for data integrity verification
        checksum: String,
    },
    /// Health check message
    HealthCheck {
        /// ID of the node performing the health check
        node_id: Uuid,
        /// Timestamp when the health check was performed
        timestamp: DateTime<Utc>,
    },
    /// Generic federation message
    Federation {
        /// Type identifier for the message
        message_type: String,
        /// Message payload data
        payload: Vec<u8>,
        /// ID of the message sender
        sender: Uuid,
        /// ID of the intended recipient (None for broadcast)
        recipient: Option<Uuid>,
    },
}

/// Node information for network discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Unique identifier for the node
    pub id: Uuid,
    /// Human-readable name of the node
    pub name: String,
    /// Version of the node software
    pub version: String,
    /// List of capabilities supported by the node
    pub capabilities: Vec<String>,
    /// Network endpoints for connecting to the node
    pub endpoints: Vec<String>,
    /// Additional metadata about the node
    pub metadata: HashMap<String, String>,
}

/// Peer information for network management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Unique identifier for the peer
    pub id: Uuid,
    /// Network address of the peer
    pub address: SocketAddr,
    /// Timestamp of the last successful communication
    pub last_seen: DateTime<Utc>,
    /// Current connection status of the peer
    pub status: PeerStatus,
    /// Network latency to the peer (if available)
    pub latency: Option<Duration>,
}

/// Peer connection status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PeerStatus {
    /// Peer is currently connected
    Connected,
    /// Peer is disconnected
    Disconnected,
    /// Currently attempting to connect to peer
    Connecting,
    /// Connection is in error state with description
    Error(String),
}

/// Data operation types for synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataOperation {
    /// Create new data
    Create,
    /// Read existing data
    Read,
    /// Update existing data
    Update,
    /// Delete existing data
    Delete,
    /// Synchronize data across nodes
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
    #[expect(dead_code, reason = "Reserved for message attribution and debugging")]
    sender: Uuid,
    #[expect(dead_code, reason = "Reserved for time-based queue management")]
    timestamp: DateTime<Utc>,
    #[expect(dead_code, reason = "Reserved for retry logic implementation")]
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
                let conn_map = connections.read().await;
                for (peer_id, connection) in conn_map.iter() {
                    let health_check = NetworkMessage::HealthCheck {
                        node_id,
                        timestamp: Utc::now(),
                    };
                    if connection
                        .send_message(*peer_id, health_check)
                        .await
                        .is_err()
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
                    q.drain(..).collect::<Vec<_>>()
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
    /// Number of connected peers
    pub peer_count: usize,
    /// Number of active connections
    pub connection_count: usize,
    /// Number of messages waiting to be sent
    pub queued_messages: usize,
    /// ID of the local node
    pub node_id: Uuid,
    /// Time when the node started
    pub uptime: DateTime<Utc>,
}

/// Network connection implementation for testing
#[cfg(any(test, feature = "testing"))]
pub struct MockNetworkConnection {
    peer_id: Uuid,
    connected: Arc<RwLock<bool>>,
    message_queue: Arc<RwLock<Vec<NetworkMessage>>>,
}

#[cfg(any(test, feature = "testing"))]
impl MockNetworkConnection {
    /// Creates a new mock network connection for testing
    pub fn new(peer_id: Uuid) -> Self {
        Self {
            peer_id,
            connected: Arc::new(RwLock::new(true)),
            message_queue: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[cfg(any(test, feature = "testing"))]
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

    fn make_node_info() -> NodeInfo {
        NodeInfo {
            id: Uuid::new_v4(),
            name: "test-node".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec!["test".to_string()],
            endpoints: vec!["http://localhost:8080".to_string()],
            metadata: HashMap::new(),
        }
    }

    fn make_network() -> FederationNetwork {
        FederationNetwork::new(NetworkConfig::default(), make_node_info())
    }

    #[tokio::test]
    async fn test_network_creation() {
        let network = FederationNetwork::new(NetworkConfig::default(), make_node_info());
        assert!(!*network.running.read().await);
    }

    #[tokio::test]
    async fn test_network_config_default() {
        let config = NetworkConfig::default();
        assert_eq!(config.port, 8080);
        assert!(config.encryption_enabled);
        assert!(matches!(config.protocol, NetworkProtocol::Http));
        assert_eq!(config.max_connections, 1000);
    }

    #[tokio::test]
    async fn test_peer_management() {
        let network = make_network();

        let peer_info = PeerInfo {
            id: Uuid::new_v4(),
            address: "127.0.0.1:8080".parse().expect("should succeed"),
            last_seen: Utc::now(),
            status: PeerStatus::Connected,
            latency: Some(Duration::from_millis(50)),
        };

        network
            .add_peer(peer_info.clone())
            .await
            .expect("should succeed");

        let stats = network.get_stats().await;
        assert_eq!(stats.peer_count, 1);

        network
            .remove_peer(peer_info.id)
            .await
            .expect("should succeed");

        let stats = network.get_stats().await;
        assert_eq!(stats.peer_count, 0);
    }

    #[tokio::test]
    async fn test_message_handling() {
        let network = make_network();

        network
            .register_handler("test".to_string(), move |_msg| Ok(()))
            .await
            .expect("should succeed");

        let handlers = network.message_handlers.read().await;
        assert!(handlers.contains_key("test"));
    }

    #[tokio::test]
    async fn test_send_to_peer_not_found() {
        let network = make_network();
        let msg = NetworkMessage::HealthCheck {
            node_id: network.node_id,
            timestamp: Utc::now(),
        };

        let result = network.send_to_peer(Uuid::new_v4(), msg).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            FederationError::PeerNotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_start_already_running() {
        let network = make_network();
        network.start().await.expect("should succeed");

        let result = network.start().await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            FederationError::AlreadyRunning(_)
        ));

        network.stop().await.expect("should succeed");
    }

    #[tokio::test]
    async fn test_stop_clears_running() {
        let network = make_network();
        network.start().await.expect("should succeed");
        assert!(*network.running.read().await);

        network.stop().await.expect("should succeed");
        assert!(!*network.running.read().await);
    }

    #[tokio::test]
    async fn test_broadcast_empty_peers_succeeds() {
        let network = make_network();
        network.start().await.expect("should succeed");

        let msg = NetworkMessage::HealthCheck {
            node_id: network.node_id,
            timestamp: Utc::now(),
        };
        let result = network.broadcast(msg).await;
        assert!(result.is_ok());

        network.stop().await.expect("should succeed");
    }

    #[tokio::test]
    async fn test_discover_peers() {
        let network = make_network();
        network.start().await.expect("should succeed");

        let peers = network.discover_peers().await.expect("should succeed");
        assert!(peers.is_empty());

        network.stop().await.expect("should succeed");
    }

    #[tokio::test]
    async fn test_mock_connection_send_receive() {
        let peer_id = Uuid::new_v4();
        let conn = MockNetworkConnection::new(peer_id);

        let msg = NetworkMessage::HealthCheck {
            node_id: peer_id,
            timestamp: Utc::now(),
        };
        conn.send_message(peer_id, msg.clone())
            .await
            .expect("should succeed");

        let (received_peer, received_msg) = conn.receive_message().await.expect("should succeed");
        assert_eq!(received_peer, peer_id);
        assert!(matches!(received_msg, NetworkMessage::HealthCheck { .. }));
    }

    #[tokio::test]
    async fn test_mock_connection_receive_empty_fails() {
        let conn = MockNetworkConnection::new(Uuid::new_v4());
        let result = conn.receive_message().await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            FederationError::NoMessagesAvailable(_)
        ));
    }

    #[tokio::test]
    async fn test_mock_connection_send_when_disconnected_fails() {
        let peer_id = Uuid::new_v4();
        let conn = MockNetworkConnection::new(peer_id);
        conn.close().await.expect("should succeed");

        let msg = NetworkMessage::HealthCheck {
            node_id: peer_id,
            timestamp: Utc::now(),
        };
        let result = conn.send_message(peer_id, msg).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            FederationError::ConnectionClosed(_)
        ));
    }

    #[tokio::test]
    async fn test_mock_connection_is_connected() {
        let conn = MockNetworkConnection::new(Uuid::new_v4());
        assert!(conn.is_connected().await);
        conn.close().await.expect("should succeed");
        assert!(!conn.is_connected().await);
    }

    #[tokio::test]
    async fn test_network_message_serialization() {
        let msg = NetworkMessage::Discovery {
            node_id: Uuid::new_v4(),
            node_info: make_node_info(),
        };
        let json = serde_json::to_string(&msg).expect("should succeed");
        let _: NetworkMessage = serde_json::from_str(&json).expect("should succeed");
    }

    #[tokio::test]
    async fn test_network_stats_fields() {
        let network = make_network();
        network
            .add_peer(PeerInfo {
                id: Uuid::new_v4(),
                address: "127.0.0.1:8080".parse().expect("should succeed"),
                last_seen: Utc::now(),
                status: PeerStatus::Connected,
                latency: None,
            })
            .await
            .expect("should succeed");

        let stats = network.get_stats().await;
        assert_eq!(stats.peer_count, 1);
        assert_eq!(stats.node_id, network.node_id);
    }

    #[tokio::test]
    async fn test_concurrent_peer_operations() {
        let network = Arc::new(make_network());

        let mut handles = vec![];
        for i in 0..10 {
            let net = Arc::clone(&network);
            handles.push(tokio::spawn(async move {
                let peer_info = PeerInfo {
                    id: Uuid::new_v4(),
                    address: format!("127.0.0.1:{}", 8080 + i)
                        .parse()
                        .expect("should succeed"),
                    last_seen: Utc::now(),
                    status: PeerStatus::Connected,
                    latency: None,
                };
                net.add_peer(peer_info).await
            }));
        }

        for handle in handles {
            assert!(handle.await.expect("should succeed").is_ok());
        }

        let stats = network.get_stats().await;
        assert_eq!(stats.peer_count, 10);
    }
}

#[cfg(test)]
mod proptest_tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::HashMap;

    fn node_info_strategy() -> impl Strategy<Value = NodeInfo> {
        (
            any::<[u8; 16]>().prop_map(Uuid::from_bytes),
            any::<String>(),
            any::<String>(),
            proptest::collection::vec(any::<String>(), 0..4),
            proptest::collection::vec(any::<String>(), 0..4),
        )
            .prop_map(|(id, name, version, caps, endpoints)| NodeInfo {
                id,
                name,
                version,
                capabilities: caps,
                endpoints,
                metadata: HashMap::new(),
            })
    }

    proptest! {
        #[test]
        fn network_message_discovery_round_trip(
            node_id in any::<[u8; 16]>().prop_map(Uuid::from_bytes),
            node_info in node_info_strategy(),
        ) {
            let msg = NetworkMessage::Discovery { node_id, node_info };
            let json = serde_json::to_string(&msg).expect("should succeed");
            let deserialized: NetworkMessage = serde_json::from_str(&json).expect("should succeed");
            if let (NetworkMessage::Discovery { node_id: a, node_info: ai }, NetworkMessage::Discovery { node_id: b, node_info: bi }) = (&msg, &deserialized) {
                prop_assert_eq!(a, b);
                prop_assert_eq!(ai.id, bi.id);
                prop_assert_eq!(&ai.name, &bi.name);
            } else {
                return Err(proptest::test_runner::TestCaseError::reject("variant mismatch"));
            }
        }

        #[test]
        fn network_message_federation_round_trip(
            msg_type in "[a-zA-Z0-9_-]{1,50}",
            payload in proptest::collection::vec(any::<u8>(), 0..256),
            sender in any::<[u8; 16]>().prop_map(Uuid::from_bytes),
        ) {
            let msg = NetworkMessage::Federation {
                message_type: msg_type.clone(),
                payload: payload.clone(),
                sender,
                recipient: None,
            };
            let json = serde_json::to_string(&msg).expect("should succeed");
            let deserialized: NetworkMessage = serde_json::from_str(&json).expect("should succeed");
            if let NetworkMessage::Federation { message_type, payload: p, sender: s, recipient } = deserialized {
                prop_assert_eq!(message_type, msg_type);
                prop_assert_eq!(p, payload);
                prop_assert_eq!(s, sender);
                prop_assert_eq!(recipient, None);
            } else {
                return Err(proptest::test_runner::TestCaseError::reject("variant mismatch"));
            }
        }

        #[test]
        fn network_message_health_check_round_trip(node_id in any::<[u8; 16]>().prop_map(Uuid::from_bytes)) {
            let msg = NetworkMessage::HealthCheck {
                node_id,
                timestamp: Utc::now(),
            };
            let json = serde_json::to_string(&msg).expect("should succeed");
            let deserialized: NetworkMessage = serde_json::from_str(&json).expect("should succeed");
            if let NetworkMessage::HealthCheck { node_id: n, .. } = deserialized {
                prop_assert_eq!(n, node_id);
            } else {
                return Err(proptest::test_runner::TestCaseError::reject("variant mismatch"));
            }
        }
    }
}
