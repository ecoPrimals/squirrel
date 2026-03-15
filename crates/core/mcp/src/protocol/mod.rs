// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! MCP Protocol Implementation
//!
//! Core protocol types and handlers for Machine Context Protocol.

pub mod types;
#[cfg(feature = "websocket")]
pub mod websocket;

#[cfg(all(test, feature = "websocket"))]
mod websocket_tests;

// Re-export commonly used types
pub use types::*;
#[cfg(feature = "websocket")]
pub use websocket::*;

use crate::error::Result;

/// Core MCP protocol trait
pub trait MCPProtocol: Send + Sync {
    /// Handles an incoming MCP message and returns the response.
    fn handle_message(
        &self,
        message: MCPMessage,
    ) -> impl std::future::Future<Output = Result<MCPMessage>> + Send;
    /// Returns the protocol version supported by this implementation.
    fn get_version(&self) -> impl std::future::Future<Output = ProtocolVersion> + Send;
}

/// Simple MCP protocol implementation
pub struct SimpleMCPProtocol;

impl MCPProtocol for SimpleMCPProtocol {
    async fn handle_message(&self, message: MCPMessage) -> Result<MCPMessage> {
        // Echo back the message with response type
        Ok(MCPMessage::new(MessageType::Response, message.payload))
    }

    async fn get_version(&self) -> ProtocolVersion {
        ProtocolVersion::default()
    }
}

impl Default for SimpleMCPProtocol {
    fn default() -> Self {
        Self
    }
}
