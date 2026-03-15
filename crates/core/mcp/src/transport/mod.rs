// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Transport layer for MCP.
//!
//! Simplified transport layer focusing on core functionality.

pub mod frame;
/// Transport types and metadata
pub mod types;
// pub mod tcp;      // Commented out due to compilation issues
#[cfg(feature = "websocket")]
pub mod websocket; // Re-enabled for Web-MCP integration
// pub mod stdio;    // Commented out due to compilation issues

// Re-export core types
pub use types::*;
#[cfg(feature = "websocket")]
pub use websocket::WebSocketTransport;

// Simplified transport trait
use crate::error::Result;
use crate::protocol::MCPMessage;
use async_trait::async_trait;

/// Trait for MCP message transport implementations
#[async_trait]
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

#[async_trait]
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
