//! MCP (Machine Context Protocol) module for Squirrel web server
//!
//! This module provides interfaces and implementations for communicating with the MCP server.

mod error;
mod client;
mod mock;
mod real;
mod context;
mod event_bridge;
mod types;

// Re-export public items
pub use error::McpError;
pub use client::{McpClient, McpCommandClient};
pub use mock::MockMcpClient;
pub use real::RealMcpClient;
pub use context::{McpContext, ContextManager, McpContextUpdates};
pub use event_bridge::McpEventBridge;
pub use types::{ConnectionStatus, McpClientConfig, McpMessage, WebSocketServerMessage}; 