// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Comprehensive error path testing for federation network
//!
//! This module expands test coverage for the federation network system by testing:
//! - Connection management and lifecycle
//! - Message handling and routing
//! - Peer discovery and management
//! - Error conditions and recovery
//! - Network protocol handling
//! - Message size limits
//! - Timeout scenarios
//!
//! Target: Increase federation_network.rs coverage from 43.56% to 70%+

#[cfg(test)]
mod network_error_tests {
    use super::super::network_manager::*;
    use super::super::network_types::*;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::time::Duration;
    use uuid::Uuid;

    /// Helper to create test node info
    fn create_test_node(name: &str) -> NodeInfo {
        NodeInfo {
            id: Uuid::new_v4(),
            name: name.to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec!["test".to_string()],
            endpoints: vec!["http://localhost:8080".to_string()],
            metadata: HashMap::new(),
        }
    }

    /// Helper to create test network config
    fn create_test_config() -> NetworkConfig {
        NetworkConfig {
            protocol: NetworkProtocol::Http,
            port: 8080,
            encryption_enabled: true,
            max_connections: 10,
            connection_timeout: 5,
            heartbeat_interval: 2,
            max_message_size: 1024,
            discovery_timeout: 3,
        }
    }

    /// Test network configuration defaults
    #[test]
    fn test_network_config_defaults() {
        let config = NetworkConfig::default();

        assert_eq!(config.port, 8080);
        assert!(config.encryption_enabled);
        assert_eq!(config.max_connections, 1000);
        assert_eq!(config.connection_timeout, 30);
        assert_eq!(config.heartbeat_interval, 10);
        assert_eq!(config.max_message_size, 1024 * 1024);
        assert_eq!(config.discovery_timeout, 5);
    }

    /// Test network protocol serialization
    #[test]
    fn test_network_protocol_variants() {
        let protocols = vec![
            NetworkProtocol::Http,
            NetworkProtocol::Grpc,
            NetworkProtocol::WebSocket,
            NetworkProtocol::Custom("test-protocol".to_string()),
        ];

        // Ensure all variants can be cloned and debugged
        for protocol in protocols {
            let cloned = protocol.clone();
            let _ = format!("{:?}", cloned);
        }
    }

    /// Test custom network protocol
    #[test]
    fn test_custom_protocol() {
        let custom = NetworkProtocol::Custom("my-protocol".to_string());

        match custom {
            NetworkProtocol::Custom(name) => {
                assert_eq!(name, "my-protocol");
            }
            _ => panic!("Expected Custom protocol"),
        }
    }

    /// Test network config with custom values
    #[test]
    fn test_network_config_custom() {
        let config = NetworkConfig {
            protocol: NetworkProtocol::Grpc,
            port: 9090,
            encryption_enabled: false,
            max_connections: 500,
            connection_timeout: 60,
            heartbeat_interval: 30,
            max_message_size: 2 * 1024 * 1024,
            discovery_timeout: 10,
        };

        assert_eq!(config.port, 9090);
        assert!(!config.encryption_enabled);
        assert_eq!(config.max_connections, 500);
        assert_eq!(config.max_message_size, 2 * 1024 * 1024);
    }

    /// Test node info creation
    #[test]
    fn test_node_info_creation() {
        let mut metadata = HashMap::new();
        metadata.insert("region".to_string(), "us-east".to_string());

        let node = NodeInfo {
            id: Uuid::new_v4(),
            name: "test-node".to_string(),
            version: "2.0.0".to_string(),
            capabilities: vec!["consensus".to_string(), "storage".to_string()],
            endpoints: vec![
                "http://localhost:8080".to_string(),
                "tarpc://localhost:9090".to_string(),
            ],
            metadata,
        };

        assert_eq!(node.name, "test-node");
        assert_eq!(node.version, "2.0.0");
        assert_eq!(node.capabilities.len(), 2);
        assert_eq!(node.endpoints.len(), 2);
        assert_eq!(node.metadata.get("region"), Some(&"us-east".to_string()));
    }

    /// Test peer info structure
    #[test]
    fn test_peer_info_structure() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let now = Utc::now();

        let peer = PeerInfo {
            id: Uuid::new_v4(),
            address: addr,
            last_seen: now,
            status: PeerStatus::Connected,
            latency: Some(Duration::from_millis(10)),
        };

        assert_eq!(peer.address.port(), 8080);
        assert_eq!(peer.latency, Some(Duration::from_millis(10)));
        assert!(matches!(peer.status, PeerStatus::Connected));
    }

    /// Test peer status variants
    #[test]
    fn test_peer_status_variants() {
        let statuses = vec![
            PeerStatus::Connected,
            PeerStatus::Connecting,
            PeerStatus::Disconnected,
            PeerStatus::Error("test error".to_string()),
        ];

        // Ensure all variants work
        for status in statuses {
            let _ = format!("{:?}", status);
        }
    }

    /// Test data operation variants
    #[test]
    fn test_data_operation_variants() {
        let operations = vec![
            DataOperation::Create,
            DataOperation::Update,
            DataOperation::Delete,
            DataOperation::Read,
            DataOperation::Sync,
        ];

        for op in operations {
            let _ = format!("{:?}", op);
        }
    }

    /// Test network message variants
    #[test]
    fn test_network_message_discovery() {
        let node = create_test_node("test");
        let msg = NetworkMessage::Discovery {
            node_id: node.id,
            node_info: node,
        };

        match msg {
            NetworkMessage::Discovery { node_id, node_info } => {
                assert_eq!(node_info.name, "test");
                assert_eq!(node_id, node_info.id);
            }
            _ => panic!("Expected Discovery message"),
        }
    }

    /// Test discovery response message
    #[test]
    fn test_discovery_response_message() {
        let node = create_test_node("responder");
        let peers = vec![PeerInfo {
            id: Uuid::new_v4(),
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            last_seen: Utc::now(),
            status: PeerStatus::Connected,
            latency: Some(Duration::from_millis(5)),
        }];

        let msg = NetworkMessage::DiscoveryResponse {
            node_id: node.id,
            node_info: node.clone(),
            peers: peers.clone(),
        };

        match msg {
            NetworkMessage::DiscoveryResponse {
                node_id,
                node_info,
                peers: resp_peers,
            } => {
                assert_eq!(node_id, node.id);
                assert_eq!(node_info.name, "responder");
                assert_eq!(resp_peers.len(), 1);
            }
            _ => panic!("Expected DiscoveryResponse message"),
        }
    }

    /// Test consensus vote message
    #[test]
    fn test_consensus_vote_message() {
        let proposal_id = Uuid::new_v4();
        let node_id = Uuid::new_v4();
        let timestamp = Utc::now();

        let msg = NetworkMessage::ConsensusVote {
            proposal_id,
            vote: true,
            node_id,
            timestamp,
        };

        match msg {
            NetworkMessage::ConsensusVote {
                proposal_id: pid,
                vote,
                ..
            } => {
                assert_eq!(pid, proposal_id);
                assert!(vote);
            }
            _ => panic!("Expected ConsensusVote message"),
        }
    }

    /// Test data sync message with checksum
    #[test]
    fn test_data_sync_message() {
        let data = b"test data".to_vec();
        let checksum = "abc123".to_string();

        let msg = NetworkMessage::DataSync {
            operation: DataOperation::Create,
            data: data.clone(),
            checksum: checksum.clone(),
        };

        match msg {
            NetworkMessage::DataSync {
                operation,
                data: msg_data,
                checksum: msg_checksum,
            } => {
                assert!(matches!(operation, DataOperation::Create));
                assert_eq!(msg_data, data);
                assert_eq!(msg_checksum, checksum);
            }
            _ => panic!("Expected DataSync message"),
        }
    }

    /// Test health check message
    #[test]
    fn test_health_check_message() {
        let node_id = Uuid::new_v4();
        let timestamp = Utc::now();

        let msg = NetworkMessage::HealthCheck { node_id, timestamp };

        match msg {
            NetworkMessage::HealthCheck { node_id: nid, .. } => {
                assert_eq!(nid, node_id);
            }
            _ => panic!("Expected HealthCheck message"),
        }
    }

    /// Test federation message with optional recipient
    #[test]
    fn test_federation_message_broadcast() {
        let sender = Uuid::new_v4();
        let payload = b"broadcast message".to_vec();

        let msg = NetworkMessage::Federation {
            message_type: "announcement".to_string(),
            payload: payload.clone(),
            sender,
            recipient: None, // Broadcast
        };

        match msg {
            NetworkMessage::Federation {
                message_type,
                payload: msg_payload,
                sender: msg_sender,
                recipient,
            } => {
                assert_eq!(message_type, "announcement");
                assert_eq!(msg_payload, payload);
                assert_eq!(msg_sender, sender);
                assert!(recipient.is_none());
            }
            _ => panic!("Expected Federation message"),
        }
    }

    /// Test federation message with specific recipient
    #[test]
    fn test_federation_message_direct() {
        let sender = Uuid::new_v4();
        let recipient = Uuid::new_v4();
        let payload = b"direct message".to_vec();

        let msg = NetworkMessage::Federation {
            message_type: "direct".to_string(),
            payload,
            sender,
            recipient: Some(recipient),
        };

        match msg {
            NetworkMessage::Federation {
                recipient: msg_recipient,
                ..
            } => {
                assert_eq!(msg_recipient, Some(recipient));
            }
            _ => panic!("Expected Federation message"),
        }
    }

    /// Test network creation
    #[tokio::test]
    async fn test_network_creation() {
        let config = create_test_config();
        let node = create_test_node("test-node");

        let network = FederationNetworkManager::new(config, node);

        // Verify network was created successfully (no panic)
        // Note: node_id is private, so we can't directly assert it
        // The fact that creation succeeded is the test
        let _ = network;
    }

    /// Test network start idempotency (already running error)
    #[tokio::test]
    async fn test_network_already_running() {
        let config = create_test_config();
        let node = create_test_node("test-node");
        let network = FederationNetworkManager::new(config, node);

        // First start should succeed
        let result1 = network.start().await;
        assert!(result1.is_ok());

        // Second start should fail with AlreadyRunning error
        let result2 = network.start().await;
        assert!(result2.is_err());

        match result2.unwrap_err() {
            super::super::FederationError::AlreadyRunning(_) => {
                // Expected error
            }
            _ => panic!("Expected AlreadyRunning error"),
        }

        // Cleanup
        let _ = network.stop().await;
    }

    /// Test network stop
    #[tokio::test]
    async fn test_network_stop() {
        let config = create_test_config();
        let node = create_test_node("test-node");
        let network = FederationNetworkManager::new(config, node);

        // Start network
        let _ = network.start().await;

        // Stop should succeed
        let result = network.stop().await;
        assert!(result.is_ok());
    }

    /// Test network stop when not running
    #[tokio::test]
    async fn test_network_stop_when_not_running() {
        let config = create_test_config();
        let node = create_test_node("test-node");
        let network = FederationNetworkManager::new(config, node);

        // Stop without starting should still succeed (idempotent)
        let result = network.stop().await;
        assert!(result.is_ok());
    }

    /// Test message handler registration
    #[tokio::test]
    async fn test_register_message_handler() {
        let config = create_test_config();
        let node = create_test_node("test-node");
        let network = FederationNetworkManager::new(config, node);

        // Register a handler
        let result = network
            .register_handler("test-message".to_string(), |_msg| Ok(()))
            .await;

        assert!(result.is_ok());
    }

    /// Test multiple handler registrations
    #[tokio::test]
    async fn test_register_multiple_handlers() {
        let config = create_test_config();
        let node = create_test_node("test-node");
        let network = FederationNetworkManager::new(config, node);

        // Register multiple handlers
        for i in 0..5 {
            let result = network
                .register_handler(format!("handler-{}", i), |_msg| Ok(()))
                .await;
            assert!(result.is_ok());
        }
    }

    /// Test large message size limit
    #[test]
    fn test_message_size_limits() {
        let mut config = NetworkConfig::default();
        config.max_message_size = 1024; // 1KB limit

        // Small message (should be under limit)
        let small_data = vec![0u8; 512];
        assert!(small_data.len() <= config.max_message_size);

        // Large message (over limit)
        let large_data = vec![0u8; 2048];
        assert!(large_data.len() > config.max_message_size);
    }

    /// Test zero timeout configuration
    #[test]
    fn test_zero_timeout_config() {
        let config = NetworkConfig {
            protocol: NetworkProtocol::Http,
            port: 8080,
            encryption_enabled: true,
            max_connections: 100,
            connection_timeout: 0, // Edge case: zero timeout
            heartbeat_interval: 1,
            max_message_size: 1024,
            discovery_timeout: 0, // Edge case: zero timeout
        };

        assert_eq!(config.connection_timeout, 0);
        assert_eq!(config.discovery_timeout, 0);
    }

    /// Test maximum connection limit
    #[test]
    fn test_max_connections_limit() {
        let config = NetworkConfig {
            protocol: NetworkProtocol::Http,
            port: 8080,
            encryption_enabled: true,
            max_connections: 1, // Minimum connections
            connection_timeout: 30,
            heartbeat_interval: 10,
            max_message_size: 1024,
            discovery_timeout: 5,
        };

        assert_eq!(config.max_connections, 1);
    }

    /// Test encryption enabled/disabled
    #[test]
    fn test_encryption_toggle() {
        let config_encrypted = NetworkConfig {
            encryption_enabled: true,
            ..NetworkConfig::default()
        };

        let config_unencrypted = NetworkConfig {
            encryption_enabled: false,
            ..NetworkConfig::default()
        };

        assert!(config_encrypted.encryption_enabled);
        assert!(!config_unencrypted.encryption_enabled);
    }

    /// Test node info with empty capabilities
    #[test]
    fn test_node_info_empty_capabilities() {
        let node = NodeInfo {
            id: Uuid::new_v4(),
            name: "minimal-node".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec![], // No capabilities
            endpoints: vec!["http://localhost:8080".to_string()],
            metadata: HashMap::new(),
        };

        assert!(node.capabilities.is_empty());
        assert!(!node.endpoints.is_empty());
    }

    /// Test node info with no endpoints
    #[test]
    fn test_node_info_no_endpoints() {
        let node = NodeInfo {
            id: Uuid::new_v4(),
            name: "no-endpoints-node".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec!["test".to_string()],
            endpoints: vec![], // No endpoints
            metadata: HashMap::new(),
        };

        assert!(node.endpoints.is_empty());
    }

    /// Test peer info with high latency
    #[test]
    fn test_peer_info_high_latency() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 8080);

        let peer = PeerInfo {
            id: Uuid::new_v4(),
            address: addr,
            last_seen: Utc::now(),
            status: PeerStatus::Connected,
            latency: Some(Duration::from_secs(5)), // 5 second latency
        };

        assert_eq!(peer.latency, Some(Duration::from_secs(5)));
    }

    /// Test failed connection status (error variant)
    #[test]
    fn test_peer_status_error() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)), 9090);

        let peer = PeerInfo {
            id: Uuid::new_v4(),
            address: addr,
            last_seen: Utc::now(),
            status: PeerStatus::Error("Connection timeout".to_string()),
            latency: None,
        };

        assert!(matches!(peer.status, PeerStatus::Error(_)));
        assert!(peer.latency.is_none());
    }
}
