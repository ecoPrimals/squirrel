// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Federation Network Types
//!
//! Type definitions for federation network communication including
//! configuration, messages, nodes, peers, and statistics.

use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;
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
            port: universal_constants::network::get_service_port("federation"),
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
        data: Bytes,
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
        payload: Bytes,
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
    /// Capabilities supported by the peer
    pub capabilities: Vec<String>,
    /// Connection reliability score (0.0 to 1.0)
    pub reliability: f64,
}

/// Peer connection status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeerStatus {
    /// Peer is currently connected
    Connected,
    /// Peer is disconnected
    Disconnected,
    /// Currently attempting to connect to peer
    Connecting,
    /// Connection is in error state with description
    Error(String),
    /// Peer connection failed
    Failed,
    /// Peer is being synchronized
    Syncing,
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
    /// Full synchronization sweep
    FullSync,
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

impl Default for NetworkStats {
    fn default() -> Self {
        Self {
            peer_count: 0,
            connection_count: 0,
            queued_messages: 0,
            node_id: Uuid::nil(),
            uptime: Utc::now(),
        }
    }
}

/// Queued message for processing
#[derive(Debug, Clone)]
pub struct QueuedMessage {
    /// Payload to deliver or process
    pub message: NetworkMessage,
    /// Originating peer id
    pub sender: Uuid,
    /// When the message was queued
    pub timestamp: DateTime<Utc>,
    /// Retry attempts for at-least-once delivery (reserved)
    pub retry_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_config_default() {
        let config = NetworkConfig::default();
        assert!(matches!(config.protocol, NetworkProtocol::Http));
        assert_eq!(
            config.port,
            universal_constants::network::get_service_port("federation")
        );
        assert!(config.encryption_enabled);
        assert_eq!(config.max_connections, 1000);
    }

    #[test]
    fn test_network_protocol_serde() {
        let protocols = vec![
            NetworkProtocol::Http,
            NetworkProtocol::Grpc,
            NetworkProtocol::WebSocket,
            NetworkProtocol::Custom("custom".to_string()),
        ];
        for p in protocols {
            let json = serde_json::to_string(&p).expect("should succeed");
            let decoded: NetworkProtocol = serde_json::from_str(&json).expect("should succeed");
            assert!(std::mem::discriminant(&p) == std::mem::discriminant(&decoded));
        }
    }

    #[test]
    fn test_node_info_creation() {
        let info = NodeInfo {
            id: Uuid::new_v4(),
            name: "test-node".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec!["sync".to_string()],
            endpoints: vec!["http://localhost:8080".to_string()],
            metadata: std::collections::HashMap::new(),
        };
        assert_eq!(info.name, "test-node");
        assert_eq!(info.capabilities.len(), 1);
    }

    #[test]
    fn test_peer_status_serde() {
        let statuses = vec![
            PeerStatus::Connected,
            PeerStatus::Disconnected,
            PeerStatus::Connecting,
            PeerStatus::Error("timeout".to_string()),
        ];
        for s in statuses {
            let json = serde_json::to_string(&s).expect("should succeed");
            let decoded: PeerStatus = serde_json::from_str(&json).expect("should succeed");
            assert_eq!(s, decoded);
        }
    }

    #[test]
    fn test_data_operation_serde() {
        let ops = vec![
            DataOperation::Create,
            DataOperation::Read,
            DataOperation::Update,
            DataOperation::Delete,
            DataOperation::Sync,
        ];
        for op in ops {
            let json = serde_json::to_string(&op).expect("should succeed");
            let decoded: DataOperation = serde_json::from_str(&json).expect("should succeed");
            assert!(std::mem::discriminant(&op) == std::mem::discriminant(&decoded));
        }
    }

    #[test]
    fn test_network_stats_creation() {
        let stats = NetworkStats {
            peer_count: 5,
            connection_count: 3,
            queued_messages: 10,
            node_id: Uuid::new_v4(),
            uptime: Utc::now(),
        };
        assert_eq!(stats.peer_count, 5);
        assert_eq!(stats.queued_messages, 10);
    }
}
