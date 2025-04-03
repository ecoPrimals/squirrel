//! Squirrel Integration Library
//!
//! This crate provides integration between various Squirrel components.

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(clippy::undocumented_unsafe_blocks)]

pub mod error;
pub mod types;

// Context-MCP integration
pub mod context_mcp;

// AI agent integration
pub mod ai_agent;

// MCP AI tools integration
pub mod mcp_ai_tools;

// Re-export from the context_mcp module
pub use context_mcp::{
    ContextMcpAdapter, ContextMcpAdapterConfig, ContextMcpError, 
    find_contexts_by_tags, CircuitBreaker,
    // Enhancement types
    ContextEnhancementType, ContextEnhancementOptions, ContextAiEnhancementOptions,
    apply_ai_enhancement,
    // Sync types
    SyncOptions, SyncResult, SyncStatus
};

// Re-export from the ai_agent module
pub use ai_agent::{
    AIAgentConfig, AIAgentAdapter,
    create_ai_agent_adapter, create_ai_agent_adapter_with_config,
};

// Re-export from the mcp_ai_tools module
pub use mcp_ai_tools::{
    McpAiToolsAdapter,
    McpAiToolsConfig,
    adapter::ToolHandler,
    adapter::ToolHandlerV2,
    adapter::ToolCallbacks,
    McpAiToolsAdapterError,
    create_mcp_ai_tools_adapter, create_mcp_ai_tools_adapter_with_config,
    AiMessageType, AiToolInvocation, AiToolResponse, AiToolResponseStatus,
};

// Skip these tests for now
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
} 