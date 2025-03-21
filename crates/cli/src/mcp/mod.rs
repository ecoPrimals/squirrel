//! Machine Context Protocol (MCP) implementation
//!
//! This module provides MCP protocol support for the CLI, allowing structured
//! communication between machines or between machines and humans.

mod server;
mod protocol;

pub use server::MCPServer;
pub use protocol::{MCPMessage, MCPMessageType, MCPResult, MCPError}; 