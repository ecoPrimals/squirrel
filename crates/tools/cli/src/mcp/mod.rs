// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Machine Context Protocol (MCP) implementation
//!
//! This module provides MCP protocol support for the CLI, allowing structured
//! communication between machines or between machines and humans.

mod client;
mod client_interactive;
mod client_listener;
mod client_types;
pub mod config; // Changed from `mod config;` to `pub mod config;`
mod protocol;
mod server;

pub use client::MCPClient;
pub use client_types::NotificationCallback;
pub use config::{MCPClientConfig, MCPServerConfig};
pub use protocol::{MCPError, MCPMessage, MCPMessageType, MCPResult};
pub use server::MCPServer;

/// Type alias for a callback function that handles MCP messages
pub type McpCallbackFn = Box<dyn Fn(MCPMessage) -> Result<(), String> + Send + Sync>;

/// Type alias for a map of topic subscriptions to callback functions
pub type SubscriptionMap = std::collections::HashMap<String, Vec<McpCallbackFn>>;

#[cfg(test)]
mod tests;
