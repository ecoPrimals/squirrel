//! Error types for MCP Core
//!
//! This module re-exports the canonical MCPError from squirrel_mcp_core.
//! The unified error system is maintained in crates/core/mcp/src/error/types.rs

// Re-export canonical MCPError from core MCP
pub use squirrel_mcp_core::error::MCPError;

/// Result type alias for MCP operations
pub type Result<T, E = MCPError> = std::result::Result<T, E>;
