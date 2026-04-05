// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Transport layer for MCP.
//!
//! Simplified transport layer focusing on core functionality.

pub mod frame;
pub mod framing;
/// Transport types and metadata
pub mod types;
#[cfg(feature = "websocket")]
pub mod websocket;

// Re-export core types
pub use types::*;
#[cfg(feature = "websocket")]
pub use websocket::WebSocketTransport;

// Simplified transport trait
use crate::error::Result;
use crate::protocol::MCPMessage;

/// Trait for MCP message transport implementations
#[expect(
    async_fn_in_trait,
    reason = "internal trait — all impls are Send + Sync"
)]
pub trait Transport: Send + Sync {
    /// Sends an MCP message over the transport
    async fn send_message(&self, message: MCPMessage) -> Result<()>;
    /// Receives an MCP message from the transport
    async fn receive_message(&self) -> Result<MCPMessage>;
    /// Establishes connection to the transport endpoint
    async fn connect(&mut self) -> Result<()>;
    /// Closes the transport connection
    async fn disconnect(&self) -> Result<()>;
    /// Returns whether the transport is currently connected
    async fn is_connected(&self) -> bool;
    /// Returns metadata about the transport connection
    async fn get_metadata(&self) -> types::TransportMetadata;
    /// Sends raw bytes over the transport
    async fn send_raw(&self, bytes: &[u8]) -> Result<()>;
}

/// Simple transport implementation for testing (no-op operations)
pub struct SimpleTransport;

impl Transport for SimpleTransport {
    async fn send_message(&self, _message: MCPMessage) -> Result<()> {
        Ok(())
    }

    async fn receive_message(&self) -> Result<MCPMessage> {
        Ok(MCPMessage::default())
    }

    async fn connect(&mut self) -> Result<()> {
        Ok(())
    }

    async fn disconnect(&self) -> Result<()> {
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        true
    }

    async fn get_metadata(&self) -> types::TransportMetadata {
        types::TransportMetadata {
            connection_id: "simple".to_string(),
            remote_address: None,
            local_address: None,
            connected_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            encryption_format: None,
            compression_format: None,
            additional_info: std::collections::HashMap::new(),
        }
    }

    async fn send_raw(&self, _bytes: &[u8]) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{SimpleTransport, Transport};
    use crate::protocol::MCPMessage;

    #[tokio::test]
    async fn simple_transport_exercises_trait_methods() {
        let mut t = SimpleTransport;
        t.connect().await.expect("should succeed");
        assert!(t.is_connected().await);
        let m = t.get_metadata().await;
        assert_eq!(m.connection_id, "simple");
        t.send_message(MCPMessage::default())
            .await
            .expect("should succeed");
        let _msg = t.receive_message().await.expect("should succeed");
        t.send_raw(b"bytes").await.expect("should succeed");
        t.disconnect().await.expect("should succeed");
    }
}
