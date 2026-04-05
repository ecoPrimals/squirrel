// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Peer Management
//!
//! Peer lifecycle management including adding, removing, and tracking
//! peer connections within the federation network.

use super::super::FederationResult;
use super::core::{FederationNetwork, NetworkConnection};
use super::types::{NetworkStats, PeerInfo, PeerStatus};
use chrono::Utc;
use std::net::SocketAddr;
use std::sync::Arc;
use uuid::Uuid;

impl<C: NetworkConnection + 'static> FederationNetwork<C> {
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

    /// Connect to a peer using the `NetworkConnection` provided by `C`.
    ///
    /// Updates the peer's status to `Connected` and stores the connection
    /// handle for subsequent message delivery. The concrete transport (Unix
    /// socket, TCP, in-process channel) is determined by the `C` type parameter.
    pub async fn connect_to_peer(
        &self,
        peer_id: Uuid,
        address: SocketAddr,
    ) -> FederationResult<()> {
        let connection = C::connect(address).await.map_err(|e| {
            super::super::FederationError::NetworkError(format!(
                "Failed to connect to peer {peer_id} at {address}: {e}"
            ))
        })?;

        let mut peers = self.peers.write().await;
        if let Some(peer) = peers.get_mut(&peer_id) {
            peer.status = PeerStatus::Connected;
            peer.last_seen = Utc::now();
        }

        let mut connections = self.connections.write().await;
        connections.insert(peer_id, Arc::new(connection));

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
