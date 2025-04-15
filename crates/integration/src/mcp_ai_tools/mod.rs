//! MCP-AI Tools integration
//!
//! This module provides integration between the MCP system and AI tools,
//! allowing for seamless usage of AI capabilities within the MCP ecosystem.

// Module declarations
pub mod adapter;
pub mod config;
pub mod types;

// Re-exports from the adapter module
pub use adapter::{
    McpAiToolsAdapter, McpAiToolsAdapterError,
    create_mcp_ai_tools_adapter, create_mcp_ai_tools_adapter_with_config,
};

// Re-exports from the config module
pub use config::{McpAiToolsConfig, ProviderSettings};

// Re-exports from the types module
pub use types::{AiMessageType, AiToolInvocation, AiToolResponse, AiToolResponseStatus}; 