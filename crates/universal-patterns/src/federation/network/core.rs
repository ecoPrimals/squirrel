// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Federation Network Core Manager
//!
//! Core federation network manager providing the main struct,
//! constructor, lifecycle management, and public API.

pub use super::types::QueuedMessage;
pub use crate::federation::network_connection::NetworkConnection;

use super::super::{FederationError, FederationResult};
use super::types::{NetworkConfig, NetworkMessage, NodeInfo, PeerInfo};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Message handler function type
pub(super) type MessageHandler = Box<dyn Fn(NetworkMessage) -> FederationResult<()> + Send + Sync>;

/// Federation network manager
pub struct FederationNetwork<C: NetworkConnection> {
    pub(super) config: NetworkConfig,
    pub(super) node_id: Uuid,
    #[expect(
        dead_code,
        reason = "federation Phase 2 — will be wired when connection state tracking is implemented"
    )]
    pub(super) node_info: NodeInfo,
    pub(super) peers: Arc<RwLock<HashMap<Uuid, PeerInfo>>>,
    pub(super) connections: Arc<RwLock<HashMap<Uuid, Arc<C>>>>,
    pub(super) message_handlers: Arc<RwLock<HashMap<String, MessageHandler>>>,
    pub(super) message_queue: Arc<RwLock<Vec<QueuedMessage>>>,
    pub(super) running: Arc<RwLock<bool>>,
}

impl<C: NetworkConnection + 'static> FederationNetwork<C> {
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
