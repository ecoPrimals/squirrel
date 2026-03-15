// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Peer Management
//!
//! Peer lifecycle management including adding, removing, and tracking
//! peer connections within the federation network.

use super::core::{FederationNetwork, NetworkConnection};
use super::types::{NetworkMessage, NetworkStats, PeerInfo, PeerStatus};
use super::super::{FederationResult};
use chrono::Utc;
use std::net::SocketAddr;
use std::sync::Arc;
use uuid::Uuid;

impl FederationNetwork {
    /// Add a peer to the network
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

    /// Get list of all peers
    pub async fn get_peers(&self) -> Vec<PeerInfo> {
        let peers = self.peers.read().await;
        peers.values().cloned().collect()
    }

    /// Get a specific peer by ID
    pub async fn get_peer(&self, peer_id: Uuid) -> Option<PeerInfo> {
        let peers = self.peers.read().await;
        peers.get(&peer_id).cloned()
    }

    /// Connect to a peer
    pub async fn connect_to_peer(
        &self,
        peer_id: Uuid,
        address: SocketAddr,
    ) -> FederationResult<()> {
        // Create connection (implementation would vary by protocol)
        // For now, this is a placeholder that would be implemented
        // based on the actual network protocol being used

        let mut peers = self.peers.write().await;
        if let Some(peer) = peers.get_mut(&peer_id) {
            peer.status = PeerStatus::Connected;
            peer.last_seen = Utc::now();
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
}

/// Mock network connection for testing
#[cfg(any(test, feature = "testing"))]
pub struct MockNetworkConnection {
    peer_id: Uuid,
    connected: Arc<tokio::sync::RwLock<bool>>,
    message_queue: Arc<tokio::sync::RwLock<Vec<NetworkMessage>>>,
}

#[cfg(any(test, feature = "testing"))]
impl MockNetworkConnection {
    /// Creates a new mock network connection for testing
    pub fn new(peer_id: Uuid) -> Self {
        Self {
            peer_id,
            connected: Arc::new(tokio::sync::RwLock::new(true)),
            message_queue: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }
}

#[cfg(any(test, feature = "testing"))]
#[async_trait::async_trait]
impl NetworkConnection for MockNetworkConnection {
    async fn send_message(
        &self,
        _peer_id: Uuid,
        message: NetworkMessage,
    ) -> FederationResult<()> {
        use super::super::FederationError;

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
        use super::super::FederationError;

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

