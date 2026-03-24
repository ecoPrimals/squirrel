// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Federation Network Tests
//!
//! Comprehensive tests for federation network functionality including
//! configuration, peer management, message handling, and connection management.

#[cfg(test)]
mod tests {
    use super::super::{
        DataOperation, FederationError, FederationNetworkManager, MockNetworkConnection,
        NetworkConfig, NetworkConnection, NetworkMessage, NetworkProtocol, NodeInfo, PeerInfo,
        PeerStatus,
    };
    use chrono::Utc;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::RwLock;
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

        let _network = FederationNetworkManager::new(config, node_info);
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

        let network = FederationNetworkManager::new(config, node_info);

        let peer_info = PeerInfo {
            id: Uuid::new_v4(),
            address: "127.0.0.1:8080".parse().expect("should succeed"),
            last_seen: Utc::now(),
            status: PeerStatus::Connected,
            latency: Some(Duration::from_millis(50)),
        };

        network.add_peer(peer_info.clone()).await.expect("should succeed");

        let stats = network.get_stats().await;
        assert_eq!(stats.peer_count, 1);

        network.remove_peer(peer_info.id).await.expect("should succeed");

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

        let network = FederationNetworkManager::new(config, node_info);

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
            .expect("should succeed");
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

        let network = FederationNetworkManager::new(config, node_info);

        network.start().await.expect("should succeed");

        network.stop().await.expect("should succeed");
    }

    #[tokio::test]
    async fn test_network_double_start() {
        let config = NetworkConfig::default();
        let node_info = NodeInfo {
            id: Uuid::new_v4(),
            name: "test-node".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec!["test".to_string()],
            endpoints: vec!["http://localhost:8080".to_string()],
            metadata: HashMap::new(),
        };

        let network = FederationNetworkManager::new(config, node_info);

        network.start().await.expect("should succeed");
        let result = network.start().await;
        assert!(result.is_err());

        match result.unwrap_err() {
            FederationError::AlreadyRunning(_) => {}
            _ => unreachable!("Expected AlreadyRunning error"),
        }
    }

    #[tokio::test]
    async fn test_multiple_peers() {
        let config = NetworkConfig::default();
        let node_info = NodeInfo {
            id: Uuid::new_v4(),
            name: "test-node".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec!["test".to_string()],
            endpoints: vec!["http://localhost:8080".to_string()],
            metadata: HashMap::new(),
        };

        let network = FederationNetworkManager::new(config, node_info);

        // Add multiple peers
        for i in 0..5 {
            let peer = PeerInfo {
                id: Uuid::new_v4(),
                address: format!("127.0.0.1:808{}", i).parse().expect("should succeed"),
                last_seen: Utc::now(),
                status: PeerStatus::Connected,
                latency: Some(Duration::from_millis(50 + i * 10)),
            };
            network.add_peer(peer).await.expect("should succeed");
        }

        let stats = network.get_stats().await;
        assert_eq!(stats.peer_count, 5);
    }

    #[tokio::test]
    async fn test_peer_status_changes() {
        let config = NetworkConfig::default();
        let node_info = NodeInfo {
            id: Uuid::new_v4(),
            name: "test-node".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec!["test".to_string()],
            endpoints: vec!["http://localhost:8080".to_string()],
            metadata: HashMap::new(),
        };

        let network = FederationNetworkManager::new(config, node_info);

        let peer_id = Uuid::new_v4();
        let peer = PeerInfo {
            id: peer_id,
            address: "127.0.0.1:8080".parse().expect("should succeed"),
            last_seen: Utc::now(),
            status: PeerStatus::Connected,
            latency: Some(Duration::from_millis(50)),
        };

        network.add_peer(peer).await.expect("should succeed");
    }

    #[tokio::test]
    async fn test_network_config_default() {
        let config = NetworkConfig::default();

        assert_eq!(config.port, 8080);
        assert!(config.encryption_enabled);
        assert!(config.max_connections > 0);
        assert!(config.connection_timeout > 0);
        assert!(config.heartbeat_interval > 0);
        assert!(config.max_message_size > 0);
        assert!(config.discovery_timeout > 0);
    }

    /// Tests legacy protocol handling: Grpc enum variant retained for compatibility.
    #[tokio::test]
    async fn test_network_protocol_variants() {
        let http = NetworkProtocol::Http;
        let grpc = NetworkProtocol::Grpc; // Legacy: tests Grpc variant (protocol stack is JSON-RPC + tarpc)
        let websocket = NetworkProtocol::WebSocket;
        let custom = NetworkProtocol::Custom("my-protocol".to_string());

        match http {
            NetworkProtocol::Http => {}
            _ => unreachable!("Expected Http"),
        }

        match grpc {
            NetworkProtocol::Grpc => {}
            _ => unreachable!("Expected Grpc"),
        }

        match websocket {
            NetworkProtocol::WebSocket => {}
            _ => unreachable!("Expected WebSocket"),
        }

        match custom {
            NetworkProtocol::Custom(ref name) => assert_eq!(name, "my-protocol"),
            _ => unreachable!("Expected Custom"),
        }
    }

    #[tokio::test]
    async fn test_peer_status_variants() {
        assert_eq!(PeerStatus::Connected, PeerStatus::Connected);
        assert_eq!(PeerStatus::Disconnected, PeerStatus::Disconnected);
        assert_eq!(PeerStatus::Connecting, PeerStatus::Connecting);

        assert!(PeerStatus::Connected != PeerStatus::Disconnected);
    }

    #[tokio::test]
    async fn test_network_message_types() {
        let node_id = Uuid::new_v4();

        // Test Discovery message
        let node_info = NodeInfo {
            id: node_id,
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec![],
            endpoints: vec![],
            metadata: HashMap::new(),
        };
        let discovery = NetworkMessage::Discovery {
            node_id,
            node_info: node_info.clone(),
        };
        match discovery {
            NetworkMessage::Discovery { .. } => {}
            _ => unreachable!("Expected Discovery"),
        }

        // Test DiscoveryResponse message
        let discovery_response = NetworkMessage::DiscoveryResponse {
            node_id,
            node_info: node_info.clone(),
            peers: vec![],
        };
        match discovery_response {
            NetworkMessage::DiscoveryResponse { .. } => {}
            _ => unreachable!("Expected DiscoveryResponse"),
        }

        // Test ConsensusVote message
        let vote = NetworkMessage::ConsensusVote {
            proposal_id: Uuid::new_v4(),
            vote: true,
            node_id,
            timestamp: Utc::now(),
        };
        match vote {
            NetworkMessage::ConsensusVote { .. } => {}
            _ => unreachable!("Expected ConsensusVote"),
        }

        // Test DataSync message
        let sync = NetworkMessage::DataSync {
            operation: DataOperation::Sync,
            data: bytes::Bytes::from_static(b"sync data"),
            checksum: "abc123".to_string(),
        };
        match sync {
            NetworkMessage::DataSync { .. } => {}
            _ => unreachable!("Expected DataSync"),
        }

        // Test HealthCheck message
        let health = NetworkMessage::HealthCheck {
            node_id,
            timestamp: Utc::now(),
        };
        match health {
            NetworkMessage::HealthCheck { .. } => {}
            _ => unreachable!("Expected HealthCheck"),
        }

        // Test Federation message
        let federation = NetworkMessage::Federation {
            message_type: "test".to_string(),
            payload: bytes::Bytes::from_static(b"test payload"),
            sender: node_id,
            recipient: Some(node_id),
        };
        match federation {
            NetworkMessage::Federation { .. } => {}
            _ => unreachable!("Expected Federation"),
        }
    }

    #[tokio::test]
    async fn test_mock_connection() {
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
            .expect("should succeed");

        let (received_peer, received_msg) = connection.receive_message().await.expect("should succeed");
        assert_eq!(received_peer, peer_id);

        match received_msg {
            NetworkMessage::HealthCheck { .. } => {}
            _ => unreachable!("Expected HealthCheck message"),
        }
    }

    #[tokio::test]
    async fn test_mock_connection_close() {
        let peer_id = Uuid::new_v4();
        let connection = MockNetworkConnection::new(peer_id);

        assert!(connection.is_connected().await);

        connection.close().await.expect("should succeed");

        assert!(!connection.is_connected().await);
    }

    #[tokio::test]
    async fn test_node_info_structure() {
        let node_info = NodeInfo {
            id: Uuid::new_v4(),
            name: "test-node".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec!["cap1".to_string(), "cap2".to_string()],
            endpoints: vec![
                "http://localhost:8080".to_string(),
                "ws://localhost:8081".to_string(),
            ],
            metadata: HashMap::from([
                ("key1".to_string(), "value1".to_string()),
                ("key2".to_string(), "value2".to_string()),
            ]),
        };

        assert_eq!(node_info.name, "test-node");
        assert_eq!(node_info.version, "1.0.0");
        assert_eq!(node_info.capabilities.len(), 2);
        assert_eq!(node_info.endpoints.len(), 2);
        assert_eq!(node_info.metadata.len(), 2);
    }

    #[tokio::test]
    async fn test_peer_info_with_latency() {
        let peer = PeerInfo {
            id: Uuid::new_v4(),
            address: "127.0.0.1:8080".parse().expect("should succeed"),
            last_seen: Utc::now(),
            status: PeerStatus::Connected,
            latency: Some(Duration::from_millis(25)),
        };

        assert_eq!(peer.status, PeerStatus::Connected);
        assert!(peer.latency.is_some());
        assert_eq!(peer.latency.expect("should succeed").as_millis(), 25);
    }

    #[tokio::test]
    async fn test_peer_info_without_latency() {
        let peer = PeerInfo {
            id: Uuid::new_v4(),
            address: "127.0.0.1:8080".parse().expect("should succeed"),
            last_seen: Utc::now(),
            status: PeerStatus::Disconnected,
            latency: None,
        };

        assert_eq!(peer.status, PeerStatus::Disconnected);
        assert!(peer.latency.is_none());
    }

    #[tokio::test]
    async fn test_network_stats() {
        let config = NetworkConfig::default();
        let node_id = Uuid::new_v4();
        let node_info = NodeInfo {
            id: node_id,
            name: "test-node".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec!["test".to_string()],
            endpoints: vec!["http://localhost:8080".to_string()],
            metadata: HashMap::new(),
        };

        let network = FederationNetworkManager::new(config, node_info);
        let stats = network.get_stats().await;

        assert_eq!(stats.node_id, node_id);
        assert_eq!(stats.peer_count, 0);
        assert_eq!(stats.connection_count, 0);
        assert_eq!(stats.queued_messages, 0);
    }

    #[tokio::test]
    async fn test_multiple_message_handlers() {
        let config = NetworkConfig::default();
        let node_info = NodeInfo {
            id: Uuid::new_v4(),
            name: "test-node".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec!["test".to_string()],
            endpoints: vec!["http://localhost:8080".to_string()],
            metadata: HashMap::new(),
        };

        let network = FederationNetworkManager::new(config, node_info);

        // Register multiple handlers
        network
            .register_handler("type1".to_string(), move |_msg| Ok(()))
            .await
            .expect("should succeed");

        network
            .register_handler("type2".to_string(), move |_msg| Ok(()))
            .await
            .expect("should succeed");

        network
            .register_handler("type3".to_string(), move |_msg| Ok(()))
            .await
            .expect("should succeed");
    }
}
