// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! MCP (Model Context Protocol) client functionality
//!
//! This module provides a comprehensive client implementation for the Model Context Protocol,
//! including connection management, message handling, and protocol operations.
//!
//! ## Key Components
//!
//! - **Types**: Core data structures for MCP messages, capabilities, and state
//! - **Client**: Main client implementation with connection lifecycle management
//! - **Connection**: Unix socket IPC (JSON-RPC) or browser WebSocket (WASM)
//! - **Message**: Message serialization, deserialization, and processing
//! - **Operations**: High-level MCP operations (tools, resources, prompts)
//!
//! ## Examples
//!
//! ```text
//! use squirrel_sdk::communication::mcp::McpClient;
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut client = McpClient::new();
//! client.connect().await?;
//!
//! let tools = client.list_tools().await?;
//! println!("Available tools: {:?}", tools);
//!
//! client.disconnect().await?;
//! # Ok(())
//! # }
//! ```ignore

pub mod client;
pub mod client_types;
pub mod connection;
pub mod message;
pub mod operations;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export main types for public API
pub use self::client::McpClient;
pub use self::client_types::{
    AiMcpMessage, ClientContext, MessageCategory, MessageResponse, ProcessedPayload,
    ProcessingStrategy,
};
pub use self::types::{
    ConnectionState, McpCapabilities, McpMessage, McpPrompt, McpResource, McpTool,
};

// Re-export configuration
pub use crate::config::McpClientConfig;

// Re-export for backward compatibility
pub use self::client::McpClient as Client;
