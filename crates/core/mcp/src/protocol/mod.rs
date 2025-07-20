//! MCP Protocol Implementation
//!
//! Core protocol types and handlers for Machine Context Protocol.

pub mod types;
pub mod websocket;

#[cfg(test)]
mod websocket_tests;

// Re-export commonly used types
pub use types::*;
pub use websocket::*;

use crate::error::Result;
use async_trait::async_trait;

/// Core MCP protocol trait
#[async_trait]
pub trait MCPProtocol: Send + Sync {
    async fn handle_message(&self, message: MCPMessage) -> Result<MCPMessage>;
    async fn get_version(&self) -> ProtocolVersion;
}

/// Simple MCP protocol implementation
pub struct SimpleMCPProtocol;

#[async_trait]
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
