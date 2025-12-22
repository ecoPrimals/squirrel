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
pub struct FederationNetworkManager {
    config: NetworkConfig,
    node_id: Uuid,
    node_info: NodeInfo,
    peers: Arc<RwLock<HashMap<Uuid, PeerInfo>>>,
    connections: Arc<RwLock<HashMap<Uuid, Arc<dyn NetworkConnection>>>>,
    message_handlers: Arc<RwLock<HashMap<String, MessageHandler>>>,
    message_queue: Arc<RwLock<Vec<QueuedMessage>>>,
    running: Arc<RwLock<bool>>,
}

impl FederationNetworkManager {
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
