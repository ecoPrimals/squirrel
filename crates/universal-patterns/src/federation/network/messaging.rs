// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Network Messaging
//!
//! Message handling, routing, and broadcasting functionality for
//! the federation network.

use super::core::{FederationNetwork, QueuedMessage};
use super::types::{NetworkMessage};
use super::super::{FederationError, FederationResult};
use chrono::Utc;
use uuid::Uuid;

impl FederationNetwork {
    /// Register a message handler
    pub async fn register_handler<F>(
        &self,
        message_type: String,
        handler: F,
    ) -> FederationResult<()>
    where
        F: Fn(NetworkMessage) -> FederationResult<()> + Send + Sync + 'static,
    {
        let mut handlers = self.message_handlers.write().await;
        handlers.insert(message_type, Box::new(handler));
        Ok(())
    }

    /// Send a message to a specific peer
    pub async fn send_to_peer(
        &self,
        peer_id: Uuid,
        message: NetworkMessage,
    ) -> FederationResult<()> {
        let connections = self.connections.read().await;
        if let Some(connection) = connections.get(&peer_id) {
            connection.send_message(peer_id, message).await?;
            Ok(())
        } else {
            Err(FederationError::PeerNotFound(peer_id))
        }
    }

    /// Broadcast a message to all peers
    pub async fn broadcast(&self, message: NetworkMessage) -> FederationResult<()> {
        let connections = self.connections.read().await;
        for (peer_id, connection) in connections.iter() {
            let _ = connection.send_message(*peer_id, message.clone()).await;
        }
        Ok(())
    }

    /// Queue a message for later processing
    pub(super) async fn queue_message(&self, sender: Uuid, message: NetworkMessage) {
        let mut queue = self.message_queue.write().await;
        queue.push(QueuedMessage {
            message,
            sender,
            timestamp: Utc::now(),
            retry_count: 0,
        });
    }

    /// Process queued messages
    pub(super) async fn process_queued_messages(&self) {
        let mut queue = self.message_queue.write().await;
        let handlers = self.message_handlers.read().await;

        // Process all queued messages
        while let Some(queued) = queue.pop() {
            // Determine message type and call appropriate handler
            if let Some(handler) = handlers.get("default") {
                let _ = handler(queued.message);
            }
        }
    }
}

