// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Federation Network Type Definitions
//!
//! Core types for federation networking including configuration, protocols,
//! messages, peer information, and connection states.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
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
    /// Current status of the peer connection
    pub status: PeerStatus,
    /// Last time the peer was seen active
    pub last_seen: DateTime<Utc>,
    /// Capabilities supported by the peer
    pub capabilities: Vec<String>,
    /// Connection reliability score (0.0 to 1.0)
    pub reliability: f64,
}

/// Status of a peer connection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeerStatus {
    /// Peer is connected and active
    Connected,
    /// Peer is disconnected
    Disconnected,
    /// Peer is connecting
    Connecting,
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
    /// Update existing data
    Update,
    /// Delete data
    Delete,
    /// Full synchronization
    FullSync,
}

/// Connection state for network management
#[derive(Debug, Clone)]
pub(super) struct ConnectionState {
    /// Connected peers
    pub peers: HashMap<Uuid, PeerInfo>,
    /// Active connections count
    pub active_connections: usize,
    /// Last heartbeat timestamp
    pub last_heartbeat: Option<DateTime<Utc>>,
    /// Network statistics
    pub stats: NetworkStats,
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

impl ConnectionState {
    /// Create new connection state
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
            active_connections: 0,
            last_heartbeat: None,
            stats: NetworkStats::default(),
        }
    }
}

