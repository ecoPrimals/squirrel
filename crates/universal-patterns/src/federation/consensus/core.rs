// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Core Consensus Manager Implementation
//!
//! The main consensus manager struct and its core functionality including
//! node management, quorum checking, and message channel setup.

use super::super::{FederationNode, NodeStatus};
use super::messaging::process_messages;
use super::types::{ConsensusConfig, ConsensusManagerState, ConsensusMessage};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

/// Default consensus manager implementation
pub struct DefaultConsensusManager {
    /// Node configuration
    pub(super) config: ConsensusConfig,
    /// Current node ID
    pub(super) node_id: Uuid,
    /// Current consensus state
    pub(super) state: Arc<RwLock<ConsensusManagerState>>,
    /// Message channel for communication
    pub(super) message_tx: mpsc::UnboundedSender<ConsensusMessage>,
    /// Registered nodes in the federation
    pub(super) nodes: Arc<RwLock<HashMap<Uuid, FederationNode>>>,
}

impl DefaultConsensusManager {
    /// Create a new consensus manager
    #[must_use]
    pub fn new(config: ConsensusConfig, node_id: Uuid) -> Self {
        let (message_tx, message_rx) = mpsc::unbounded_channel();

        let state = Arc::new(RwLock::new(ConsensusManagerState::new()));

        let config_clone = config.clone();
        let state_bg = state.clone();

        let manager = Self {
            config,
            node_id,
            state,
            message_tx,
            nodes: Arc::new(RwLock::new(HashMap::new())),
        };

        // Start message processing task
        let state_clone = state_bg;
        let node_id_clone = node_id;
        tokio::spawn(async move {
            process_messages(state_clone, config_clone, node_id_clone, message_rx).await;
        });

        manager
    }

    /// Register a node in the federation
    pub async fn register_node(&self, node: FederationNode) {
        let mut nodes = self.nodes.write().await;
        nodes.insert(node.id, node);
    }

    /// Remove a node from the federation
    pub async fn remove_node(&self, node_id: Uuid) {
        let mut nodes = self.nodes.write().await;
        nodes.remove(&node_id);
    }

    /// Get active nodes
    pub(super) async fn get_active_nodes(&self) -> Vec<FederationNode> {
        let nodes = self.nodes.read().await;
        nodes
            .values()
            .filter(|node| node.status == NodeStatus::Active)
            .cloned()
            .collect()
    }

    /// Check if we have enough nodes for consensus
    pub(super) async fn has_quorum(&self) -> bool {
        let active_nodes = self.get_active_nodes().await;
        active_nodes.len() >= self.config.min_nodes as usize
    }
}
