// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Federation Network Connection
//!
//! Connection abstraction and implementations for federation network communication.

use super::FederationResult;
use super::network_types::NetworkMessage;
use std::future::Future;
use uuid::Uuid;

#[cfg(any(test, feature = "testing"))]
use super::FederationError;
#[cfg(any(test, feature = "testing"))]
use std::sync::Arc;
#[cfg(any(test, feature = "testing"))]
use tokio::sync::RwLock;

/// Network connection interface for federation peers.
///
/// Implementations provide the transport layer (Unix socket, TCP, in-process
/// channel, etc.). [`connect`](Self::connect) establishes a new connection to
/// a remote address; the remaining methods operate on the resulting handle.
pub trait NetworkConnection: Send + Sync + Sized {
    /// Establish a connection to `address`.
    fn connect(
        address: std::net::SocketAddr,
    ) -> impl Future<Output = FederationResult<Self>> + Send;

    /// Send a message to a peer
    fn send_message(
        &self,
        peer_id: Uuid,
        message: NetworkMessage,
    ) -> impl Future<Output = FederationResult<()>> + Send;

    /// Receive a message from the network
    fn receive_message(
        &self,
    ) -> impl Future<Output = FederationResult<(Uuid, NetworkMessage)>> + Send;

    /// Check if connection is alive
    fn is_connected(&self) -> impl Future<Output = bool> + Send;

    /// Close the connection
    fn close(&self) -> impl Future<Output = FederationResult<()>> + Send;
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
impl NetworkConnection for MockNetworkConnection {
    async fn connect(_address: std::net::SocketAddr) -> FederationResult<Self> {
        Ok(Self::new(Uuid::new_v4()))
    }

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
