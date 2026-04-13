// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Federation Network Manager
//!
//! Core network management logic for federation operations including
//! peer management, message handling, and background tasks.

use super::network_connection::NetworkConnection;
use super::network_types::{
    NetworkConfig, NetworkMessage, NetworkStats, NodeInfo, PeerInfo, PeerStatus, QueuedMessage,
};
use super::{FederationError, FederationResult};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;
use uuid::Uuid;

/// Message handler function type
type MessageHandler = Box<dyn Fn(NetworkMessage) -> FederationResult<()> + Send + Sync>;

/// Federation network manager implementation
pub struct FederationNetworkManager<C: NetworkConnection> {
    config: NetworkConfig,
    node_id: Uuid,
    node_info: NodeInfo,
    peers: Arc<RwLock<HashMap<Uuid, PeerInfo>>>,
    connections: Arc<RwLock<HashMap<Uuid, Arc<C>>>>,
    message_handlers: Arc<RwLock<HashMap<String, MessageHandler>>>,
    message_queue: Arc<RwLock<Vec<QueuedMessage>>>,
    running: Arc<RwLock<bool>>,
}

impl<C: NetworkConnection + 'static> FederationNetworkManager<C> {
    /// Create a new federation network manager
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
                    if connection
                        .send_message(*peer_id, health_check.clone())
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
    use crate::federation::network_connection::MockNetworkConnection;
    use crate::federation::network_types::PeerStatus;

    fn test_config() -> NetworkConfig {
        NetworkConfig {
            heartbeat_interval: 60,
            discovery_timeout: 1,
            ..NetworkConfig::default()
        }
    }

    fn test_node_info() -> NodeInfo {
        NodeInfo {
            id: Uuid::new_v4(),
            name: "test-node".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec!["test".to_string()],
            endpoints: vec![],
            metadata: HashMap::new(),
        }
    }

    fn test_peer_info() -> PeerInfo {
        PeerInfo {
            id: Uuid::new_v4(),
            address: "127.0.0.1:9000".parse().expect("valid addr"),
            last_seen: Utc::now(),
            status: PeerStatus::Connected,
            latency: None,
            capabilities: vec![],
            reliability: 1.0,
        }
    }

    #[tokio::test]
    async fn test_new_creates_empty_manager() {
        let mgr =
            FederationNetworkManager::<MockNetworkConnection>::new(test_config(), test_node_info());
        let stats = mgr.get_stats().await;
        assert_eq!(stats.peer_count, 0);
        assert_eq!(stats.connection_count, 0);
        assert_eq!(stats.queued_messages, 0);
    }

    #[tokio::test]
    async fn test_add_and_remove_peer() {
        let mgr =
            FederationNetworkManager::<MockNetworkConnection>::new(test_config(), test_node_info());
        let peer = test_peer_info();
        let peer_id = peer.id;

        mgr.add_peer(peer).await.expect("add peer");
        let stats = mgr.get_stats().await;
        assert_eq!(stats.peer_count, 1);

        mgr.remove_peer(peer_id).await.expect("remove peer");
        let stats = mgr.get_stats().await;
        assert_eq!(stats.peer_count, 0);
    }

    #[tokio::test]
    async fn test_add_multiple_peers() {
        let mgr =
            FederationNetworkManager::<MockNetworkConnection>::new(test_config(), test_node_info());

        for _ in 0..5 {
            mgr.add_peer(test_peer_info()).await.expect("add peer");
        }

        let stats = mgr.get_stats().await;
        assert_eq!(stats.peer_count, 5);
    }

    #[tokio::test]
    async fn test_register_handler() {
        let mgr =
            FederationNetworkManager::<MockNetworkConnection>::new(test_config(), test_node_info());
        mgr.register_handler("test_type".to_string(), |_msg| Ok(()))
            .await
            .expect("register handler");
    }

    #[tokio::test]
    async fn test_send_to_peer_not_found() {
        let mgr =
            FederationNetworkManager::<MockNetworkConnection>::new(test_config(), test_node_info());
        let msg = NetworkMessage::HealthCheck {
            node_id: Uuid::new_v4(),
            timestamp: Utc::now(),
        };
        let result = mgr.send_to_peer(Uuid::new_v4(), msg).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            FederationError::PeerNotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_broadcast_empty_connections_succeeds() {
        let mgr =
            FederationNetworkManager::<MockNetworkConnection>::new(test_config(), test_node_info());
        let msg = NetworkMessage::HealthCheck {
            node_id: Uuid::new_v4(),
            timestamp: Utc::now(),
        };
        let result = mgr.broadcast(msg).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_start_and_stop() {
        let mgr =
            FederationNetworkManager::<MockNetworkConnection>::new(test_config(), test_node_info());

        mgr.start().await.expect("start");
        assert!(*mgr.running.read().await);

        mgr.stop().await.expect("stop");
        assert!(!*mgr.running.read().await);
    }

    #[tokio::test]
    async fn test_start_twice_returns_error() {
        let mgr =
            FederationNetworkManager::<MockNetworkConnection>::new(test_config(), test_node_info());

        mgr.start().await.expect("first start");
        let result = mgr.start().await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            FederationError::AlreadyRunning(_)
        ));

        mgr.stop().await.expect("stop");
    }

    #[tokio::test]
    async fn test_get_stats_returns_node_id() {
        let info = test_node_info();
        let node_id = info.id;
        let mgr = FederationNetworkManager::<MockNetworkConnection>::new(test_config(), info);

        let stats = mgr.get_stats().await;
        assert_eq!(stats.node_id, node_id);
    }

    #[tokio::test]
    async fn test_remove_nonexistent_peer_is_ok() {
        let mgr =
            FederationNetworkManager::<MockNetworkConnection>::new(test_config(), test_node_info());
        let result = mgr.remove_peer(Uuid::new_v4()).await;
        assert!(result.is_ok());
    }
}
