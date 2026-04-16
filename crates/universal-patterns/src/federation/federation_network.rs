// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Federation Network Module
//!
//! This module handles networking and communication between federation nodes.
//! It provides secure, reliable communication channels for the federation
//! with support for multiple protocols and encryption.

use super::network_types::QueuedMessage;
use super::{FederationError, FederationResult};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;
use uuid::Uuid;

#[cfg(any(test, feature = "testing"))]
pub use super::network_connection::MockNetworkConnection;
pub use super::network_connection::NetworkConnection;
pub use super::network_types::{
    DataOperation, NetworkConfig, NetworkMessage, NetworkProtocol, NetworkStats, NodeInfo,
    PeerInfo, PeerStatus,
};

/// Message handler function type
type MessageHandler = Box<dyn Fn(NetworkMessage) -> FederationResult<()> + Send + Sync>;

/// Federation network manager
pub struct FederationNetwork<C: NetworkConnection> {
    config: NetworkConfig,
    pub(super) node_id: Uuid,
    node_info: NodeInfo,
    pub(super) peers: Arc<RwLock<HashMap<Uuid, PeerInfo>>>,
    pub(super) connections: Arc<RwLock<HashMap<Uuid, Arc<C>>>>,
    pub(super) message_handlers: Arc<RwLock<HashMap<String, MessageHandler>>>,
    pub(super) message_queue: Arc<RwLock<Vec<QueuedMessage>>>,
    pub(super) running: Arc<RwLock<bool>>,
}

impl<C: NetworkConnection + 'static> FederationNetwork<C> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use universal_constants::builders::localhost_http;
    use universal_constants::network::get_service_port;

    fn localhost_ws_url() -> String {
        localhost_http(get_service_port("websocket"))
    }

    fn peer_addr_websocket() -> std::net::SocketAddr {
        format!("127.0.0.1:{}", get_service_port("websocket"))
            .parse()
            .expect("should succeed")
    }

    fn make_node_info() -> NodeInfo {
        NodeInfo {
            id: Uuid::new_v4(),
            name: "test-node".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec!["test".to_string()],
            endpoints: vec![localhost_ws_url()],
            metadata: HashMap::new(),
        }
    }

    fn make_network() -> FederationNetwork<MockNetworkConnection> {
        FederationNetwork::new(NetworkConfig::default(), make_node_info())
    }

    #[tokio::test]
    async fn test_network_creation() {
        let network = FederationNetwork::<MockNetworkConnection>::new(
            NetworkConfig::default(),
            make_node_info(),
        );
        assert!(!*network.running.read().await);
    }

    #[tokio::test]
    async fn test_network_config_default() {
        let config = NetworkConfig::default();
        assert_eq!(
            config.port,
            universal_constants::network::get_service_port("federation")
        );
        assert!(config.encryption_enabled);
        assert!(matches!(config.protocol, NetworkProtocol::Http));
        assert_eq!(config.max_connections, 1000);
    }

    #[tokio::test]
    async fn test_peer_management() {
        let network = make_network();

        let peer_info = PeerInfo {
            id: Uuid::new_v4(),
            address: peer_addr_websocket(),
            last_seen: Utc::now(),
            status: PeerStatus::Connected,
            latency: Some(Duration::from_millis(50)),
            capabilities: vec![],
            reliability: 1.0,
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
                address: peer_addr_websocket(),
                last_seen: Utc::now(),
                status: PeerStatus::Connected,
                latency: None,
                capabilities: vec![],
                reliability: 1.0,
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
        let base = get_service_port("websocket");
        for i in 0..10 {
            let net = Arc::clone(&network);
            handles.push(tokio::spawn(async move {
                let peer_info = PeerInfo {
                    id: Uuid::new_v4(),
                    address: format!("127.0.0.1:{}", base + i)
                        .parse()
                        .expect("should succeed"),
                    last_seen: Utc::now(),
                    status: PeerStatus::Connected,
                    latency: None,
                    capabilities: vec![],
                    reliability: 1.0,
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
            let payload = bytes::Bytes::from(payload);
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
