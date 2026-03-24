// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Federation Network Connection
//!
//! Connection abstraction and implementations for federation network communication.

use super::network_types::NetworkMessage;
use super::{FederationError, FederationResult};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Network connection interface
#[async_trait]
pub trait NetworkConnection: Send + Sync {
    /// Send a message to a peer
    async fn send_message(&self, peer_id: Uuid, message: NetworkMessage) -> FederationResult<()>;

    /// Receive a message from the network
    async fn receive_message(&self) -> FederationResult<(Uuid, NetworkMessage)>;

    /// Check if connection is alive
    async fn is_connected(&self) -> bool;

    /// Close the connection
    async fn close(&self) -> FederationResult<()>;
}

/// Network connection implementation for testing
#[cfg(any(test, feature = "testing"))]
pub struct MockNetworkConnection {
    peer_id: Uuid,
    connected: Arc<RwLock<bool>>,
    message_queue: Arc<RwLock<Vec<NetworkMessage>>>,
}

#[cfg(any(test, feature = "testing"))]
impl MockNetworkConnection {
    /// Creates a new mock network connection for testing
    pub fn new(peer_id: Uuid) -> Self {
        Self {
            peer_id,
            connected: Arc::new(RwLock::new(true)),
            message_queue: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[cfg(any(test, feature = "testing"))]
#[async_trait]
impl NetworkConnection for MockNetworkConnection {
    async fn send_message(&self, _peer_id: Uuid, message: NetworkMessage) -> FederationResult<()> {
        if !*self.connected.read().await {
            return Err(FederationError::ConnectionClosed(
                "Connection closed".to_string(),
            ));
        }

        let mut queue = self.message_queue.write().await;
        queue.push(message);
        Ok(())
    }

    async fn receive_message(&self) -> FederationResult<(Uuid, NetworkMessage)> {
        let mut queue = self.message_queue.write().await;
        if let Some(message) = queue.pop() {
            Ok((self.peer_id, message))
        } else {
            Err(FederationError::NoMessagesAvailable(
                "No messages".to_string(),
            ))
        }
    }

    async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }

    async fn close(&self) -> FederationResult<()> {
        let mut connected = self.connected.write().await;
        *connected = false;
        Ok(())
    }
}
