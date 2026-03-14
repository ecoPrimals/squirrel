// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Federation Network Module
//!
//! This module handles networking and communication between federation nodes.
//! It provides secure, reliable communication channels for the federation
//! with support for multiple protocols and encryption.
//!
//! ## Architecture
//!
//! The network module is organized into focused sub-modules:
//!
//! - **`types`**: Core type definitions (Config, Messages, PeerInfo, Stats)
//! - **`core`**: Core manager and NetworkConnection trait
//! - **`peers`**: Peer lifecycle management
//! - **`messaging`**: Message handling and routing
//! - **`tasks`**: Background tasks (heartbeat, discovery, processing)
//!
//! This structure provides clear separation of concerns with each module
//! focused on a specific aspect of network management.

mod core;
mod messaging;
mod peers;
mod tasks;
mod types;

// Re-export public API
pub use core::{FederationNetwork, NetworkConnection};
#[cfg(any(test, feature = "testing"))]
pub use peers::MockNetworkConnection;
pub use types::{
    DataOperation, NetworkConfig, NetworkMessage, NetworkProtocol, NetworkStats, NodeInfo,
    PeerInfo, PeerStatus,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use uuid::Uuid;

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
        let stats = network.get_stats().await;
        assert_eq!(stats.peer_count, 0);
        assert_eq!(stats.connection_count, 0);
    }

    #[tokio::test]
    async fn test_add_peer() {
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
            status: PeerStatus::Connected,
            last_seen: chrono::Utc::now(),
            capabilities: vec!["test".to_string()],
            reliability: 1.0,
        };

        network.add_peer(peer_info.clone()).await.unwrap();

        let peers = network.get_peers().await;
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].id, peer_info.id);
    }

    #[tokio::test]
    async fn test_remove_peer() {
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
            status: PeerStatus::Connected,
            last_seen: chrono::Utc::now(),
            capabilities: vec!["test".to_string()],
            reliability: 1.0,
        };

        network.add_peer(peer_info.clone()).await.unwrap();
        assert_eq!(network.get_peers().await.len(), 1);

        network.remove_peer(peer_info.id).await.unwrap();
        assert_eq!(network.get_peers().await.len(), 0);
    }

    #[tokio::test]
    async fn test_network_start_stop() {
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

        // Start network
        network.start().await.unwrap();

        // Stop network
        network.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_mock_connection() {
        use chrono::Utc;

        let peer_id = Uuid::new_v4();
        let connection = MockNetworkConnection::new(peer_id);

        assert!(connection.is_connected().await);

        let message = NetworkMessage::HealthCheck {
            node_id: peer_id,
            timestamp: Utc::now(),
        };

        connection
            .send_message(peer_id, message.clone())
            .await
            .unwrap();

        let (received_peer_id, received_message) = connection.receive_message().await.unwrap();
        assert_eq!(received_peer_id, peer_id);

        connection.close().await.unwrap();
        assert!(!connection.is_connected().await);
    }
}

