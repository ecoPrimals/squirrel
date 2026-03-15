// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Federation Network Types
//!
//! Type definitions for federation network communication including
//! configuration, messages, nodes, peers, and statistics.

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

/// Queued message for processing
#[derive(Debug, Clone)]
pub(super) struct QueuedMessage {
    pub message: NetworkMessage,
    pub sender: Uuid,
    pub timestamp: DateTime<Utc>,
    pub retry_count: u32,
}
