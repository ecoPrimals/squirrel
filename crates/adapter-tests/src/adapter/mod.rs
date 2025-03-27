//! Adapter module for Command Pattern implementation
//!
//! This module provides interfaces and implementations for the Command Adapter Pattern.
//! The adapters in this module transform the core command registry functionality into 
//! different interfaces for various use cases.

mod registry;
mod mcp;
mod plugin;

// Re-export adapter types
pub use registry::{CommandRegistryAdapter, MockAdapter};
pub use mcp::{McpCommandAdapter, Auth};
pub use plugin::CommandsPluginAdapter;

/// Creates a new command registry adapter with default settings
pub fn create_registry_adapter() -> CommandRegistryAdapter {
    registry::CommandRegistryAdapter::new()
}

/// Creates a new MCP command adapter with default settings
pub fn create_mcp_adapter() -> McpCommandAdapter {
    mcp::McpCommandAdapter::new()
}

/// Creates a new plugin adapter with default settings
pub fn create_plugin_adapter() -> CommandsPluginAdapter {
    plugin::CommandsPluginAdapter::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_adapter_factory_functions() {
        // Test registry adapter creation
        let registry_adapter = create_registry_adapter();
        assert!(registry_adapter.is_initialized());

        // Test MCP adapter creation
        let mcp_adapter = create_mcp_adapter();
        assert!(mcp_adapter.is_initialized());

        // Test plugin adapter creation
        let plugin_adapter = create_plugin_adapter();
        assert!(plugin_adapter.is_initialized());
    }
} 