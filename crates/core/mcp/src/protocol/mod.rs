// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

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
use std::future::Future;

/// Core MCP protocol trait
pub trait MCPProtocol: Send + Sync {
    fn handle_message(
        &self,
        message: MCPMessage,
    ) -> impl Future<Output = Result<MCPMessage>> + Send;
    fn get_version(&self) -> impl Future<Output = ProtocolVersion> + Send;
}

/// Simple MCP protocol implementation
pub struct SimpleMCPProtocol;

impl MCPProtocol for SimpleMCPProtocol {
    fn handle_message(
        &self,
        message: MCPMessage,
    ) -> impl Future<Output = Result<MCPMessage>> + Send {
        async move {
            // Echo back the message with response type
            Ok(MCPMessage::new(MessageType::Response, message.payload))
        }
    }

    fn get_version(&self) -> impl Future<Output = ProtocolVersion> + Send {
        async move { ProtocolVersion::default() }
    }
}

impl Default for SimpleMCPProtocol {
    fn default() -> Self {
        Self
    }
}
