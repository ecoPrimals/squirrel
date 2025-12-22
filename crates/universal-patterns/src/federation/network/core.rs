//! Federation Network Core Manager
//!
//! Core federation network manager providing the main struct,
//! constructor, lifecycle management, and public API.

use super::types::{NetworkConfig, NodeInfo, NetworkMessage, PeerInfo};
use super::super::{FederationError, FederationResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Network connection trait for communication abstraction
#[async_trait]
pub trait NetworkConnection: Send + Sync {
    /// Send a message to a peer
    async fn send_message(&self, peer_id: Uuid, message: NetworkMessage) -> FederationResult<()>;

    /// Receive a message from the network
    async fn receive_message(&self) -> FederationResult<(Uuid, NetworkMessage)>;

    /// Check if the connection is active
    async fn is_connected(&self) -> bool;

    /// Close the connection
    async fn close(&self) -> FederationResult<()>;
}

/// Message handler function type
pub(super) type MessageHandler =
    Box<dyn Fn(NetworkMessage) -> FederationResult<()> + Send + Sync>;

/// Queued message for processing
#[derive(Debug, Clone)]
pub(super) struct QueuedMessage {
    pub message: NetworkMessage,
    #[allow(dead_code)]
    pub sender: Uuid,
    #[allow(dead_code)]
    pub timestamp: DateTime<Utc>,
    #[allow(dead_code)]
    pub retry_count: u32,
}

/// Federation network manager
pub struct FederationNetwork {
    pub(super) config: NetworkConfig,
    pub(super) node_id: Uuid,
    pub(super) node_info: NodeInfo,
    pub(super) peers: Arc<RwLock<HashMap<Uuid, PeerInfo>>>,
    pub(super) connections: Arc<RwLock<HashMap<Uuid, Arc<dyn NetworkConnection>>>>,
    pub(super) message_handlers: Arc<RwLock<HashMap<String, MessageHandler>>>,
    pub(super) message_queue: Arc<RwLock<Vec<QueuedMessage>>>,
    pub(super) running: Arc<RwLock<bool>>,
}

impl FederationNetwork {
    /// Create a new federation network
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
}

