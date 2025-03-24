//! Integration tests for the complete plugin system
//!
//! These tests verify that the various components of the plugin system
//! work together correctly, including the registry, manager, and example plugins.

use crate::plugin::{
    Plugin, PluginManager, PluginRegistry, PluginStatus,
    examples::{
        create_example_plugins, create_plugin_dependency_chain,
        create_advanced_command_plugin, create_advanced_mcp_plugin, create_advanced_tool_plugin
    },
    types::{CommandPlugin, ToolPlugin, McpPlugin}
};
use serde_json::json;
use std::sync::Arc;
use tokio::time::Duration;

/// Test the full plugin lifecycle from registration to unloading
#[tokio::test]
async fn test_full_plugin_lifecycle() {
    // Create manager and registry
    let manager = PluginManager::new();
    let registry = PluginRegistry::new();
    
    // Get example plugins
    let plugins = create_example_plugins();
    
    // Register plugins with both manager and registry
    for plugin in &plugins {
        manager.register_plugin(plugin.clone_box()).await.unwrap();
        registry.add_plugin(plugin.as_ref(), "memory://example").await.unwrap();
    }
    
    // Resolve dependencies
    let load_order = registry.resolve_dependencies().await.unwrap();
    
    // Load plugins in the correct order
    for id in load_order {
        manager.load_plugin(id).await.unwrap();
        registry.update_status(id, PluginStatus::Active).await.unwrap();
        registry.mark_used(id).await.unwrap();
    }
    
    // Verify all plugins are loaded
    for plugin in &plugins {
        let id = plugin.metadata().id;
        let status = manager.get_plugin_status(id).await.unwrap();
        assert_eq!(status, PluginStatus::Active);
        
        let registry_status = registry.get_status(id).await.unwrap();
        assert_eq!(registry_status, PluginStatus::Active);
    }
    
    // Test capability-based plugin retrieval
    let command_capabilities = registry.get_plugins_by_capability("command").await;
    assert!(!command_capabilities.is_empty());
    
    let tool_capabilities = registry.get_plugins_by_capability("tool").await;
    assert!(!tool_capabilities.is_empty());
    
    let mcp_capabilities = registry.get_plugins_by_capability("mcp").await;
    assert!(!mcp_capabilities.is_empty());
    
    // Test specialized plugin retrieval from manager
    for id in command_capabilities {
        let command_plugin = manager.get_command_plugin(id).await.unwrap();
        let commands = command_plugin.list_commands();
        assert!(!commands.is_empty());
    }
    
    for id in tool_capabilities {
        let tool_plugin = manager.get_tool_plugin(id).await.unwrap();
        let tools = tool_plugin.list_tools();
        assert!(!tools.is_empty());
    }
    
    for id in mcp_capabilities {
        let mcp_plugin = manager.get_mcp_plugin(id).await.unwrap();
        let extensions = mcp_plugin.get_protocol_extensions();
        assert!(!extensions.is_empty());
    }
    
    // Get plugin catalog entries
    let entries = registry.get_all_catalog_entries().await;
    assert_eq!(entries.len(), plugins.len());
    
    // Unload plugins in reverse order
    let mut reverse_order = load_order;
    reverse_order.reverse();
    
    for id in reverse_order {
        manager.unload_plugin(id).await.unwrap();
        registry.update_status(id, PluginStatus::Unloaded).await.unwrap();
    }
    
    // Verify all plugins are unloaded
    for plugin in &plugins {
        let id = plugin.metadata().id;
        let status = manager.get_plugin_status(id).await.unwrap();
        assert_eq!(status, PluginStatus::Unloaded);
        
        let registry_status = registry.get_status(id).await.unwrap();
        assert_eq!(registry_status, PluginStatus::Unloaded);
    }
}

/// Test dependency chain resolution and loading
#[tokio::test]
async fn test_dependency_resolution() {
    let manager = PluginManager::new();
    let registry = PluginRegistry::new();
    
    // Get plugins with a dependency chain
    let plugins = create_plugin_dependency_chain();
    
    // Register plugins with both manager and registry
    for plugin in &plugins {
        manager.register_plugin(plugin.clone_box()).await.unwrap();
        registry.add_plugin(plugin.as_ref(), "memory://example").await.unwrap();
    }
    
    // Resolve dependencies
    let load_order = registry.resolve_dependencies().await.unwrap();
    
    // Verify correct ordering: base -> mid -> top
    let base_id = registry.get_plugin_id("base-plugin").await.unwrap();
    let mid_id = registry.get_plugin_id("mid-plugin").await.unwrap();
    let top_id = registry.get_plugin_id("top-plugin").await.unwrap();
    
    let base_pos = load_order.iter().position(|&id| id == base_id).unwrap();
    let mid_pos = load_order.iter().position(|&id| id == mid_id).unwrap();
    let top_pos = load_order.iter().position(|&id| id == top_id).unwrap();
    
    // Base should come before mid
    assert!(base_pos < mid_pos);
    // Mid should come before top
    assert!(mid_pos < top_pos);
    
    // Load plugins in the correct order
    for id in load_order {
        manager.load_plugin(id).await.unwrap();
        registry.update_status(id, PluginStatus::Active).await.unwrap();
    }
    
    // Verify all plugins are loaded
    for plugin in &plugins {
        let id = plugin.metadata().id;
        let status = manager.get_plugin_status(id).await.unwrap();
        assert_eq!(status, PluginStatus::Active);
    }
    
    // Test dependencies lookup
    let top_deps = registry.get_dependencies(top_id).await;
    assert_eq!(top_deps.len(), 1);
    assert_eq!(top_deps[0], mid_id);
    
    let mid_deps = registry.get_dependencies(mid_id).await;
    assert_eq!(mid_deps.len(), 1);
    assert_eq!(mid_deps[0], base_id);
    
    let base_deps = registry.get_dependencies(base_id).await;
    assert_eq!(base_deps.len(), 0);
    
    // Test dependents lookup
    let base_dependents = registry.get_dependents(base_id).await;
    assert_eq!(base_dependents.len(), 1);
    assert_eq!(base_dependents[0], mid_id);
    
    let mid_dependents = registry.get_dependents(mid_id).await;
    assert_eq!(mid_dependents.len(), 1);
    assert_eq!(mid_dependents[0], top_id);
    
    let top_dependents = registry.get_dependents(top_id).await;
    assert_eq!(top_dependents.len(), 0);
}

/// Test plugin functionality across types
#[tokio::test]
async fn test_plugin_functionality() {
    // Create specialized plugins
    let command_plugin = create_advanced_command_plugin();
    let tool_plugin = create_advanced_tool_plugin();
    let mcp_plugin = create_advanced_mcp_plugin();
    
    // Register and initialize
    let manager = PluginManager::new();
    
    manager.register_plugin(command_plugin.clone_box()).await.unwrap();
    manager.register_plugin(tool_plugin.clone_box()).await.unwrap();
    manager.register_plugin(mcp_plugin.clone_box()).await.unwrap();
    
    let command_id = command_plugin.metadata().id;
    let tool_id = tool_plugin.metadata().id;
    let mcp_id = mcp_plugin.metadata().id;
    
    // Load plugins
    manager.load_plugin(command_id).await.unwrap();
    manager.load_plugin(tool_id).await.unwrap();
    manager.load_plugin(mcp_id).await.unwrap();
    
    // Test command plugin functionality
    let command_plugin = manager.get_command_plugin(command_id).await.unwrap();
    let result = command_plugin.execute_command("increment", json!({"amount": 5})).await.unwrap();
    assert_eq!(result.get("count").unwrap().as_u64().unwrap(), 5);
    
    // Test tool plugin functionality
    let tool_plugin = manager.get_tool_plugin(tool_id).await.unwrap();
    let code = "fn main() { println!(\"Hello\"); }";
    let result = tool_plugin.execute_tool("analyze", json!({"code": code, "language": "rust"})).await.unwrap();
    assert!(result.get("lines_of_code").is_some());
    
    // Test MCP plugin functionality
    let mcp_plugin = manager.get_mcp_plugin(mcp_id).await.unwrap();
    let message = json!({
        "type": "context-enrich",
        "file_path": "/path/to/file.rs"
    });
    let result = mcp_plugin.handle_message(message).await.unwrap();
    assert_eq!(result.get("status").unwrap().as_str().unwrap(), "success");
    
    // Clean up
    manager.unload_plugin(command_id).await.unwrap();
    manager.unload_plugin(tool_id).await.unwrap();
    manager.unload_plugin(mcp_id).await.unwrap();
}

/// Test plugin recovery functionality
#[tokio::test]
async fn test_plugin_recovery() {
    // Create a simple plugin manager
    let manager = PluginManager::new();
    
    // Register the example plugins
    let plugins = create_example_plugins();
    for plugin in &plugins {
        manager.register_plugin(plugin.clone_box()).await.unwrap();
    }
    
    // Get the first plugin ID
    let plugin_id = plugins[0].metadata().id;
    
    // Test recovery function
    let result = manager.load_plugin_with_recovery(plugin_id).await;
    assert!(result.is_ok());
    
    // Verify the plugin is loaded
    let status = manager.get_plugin_status(plugin_id).await.unwrap();
    assert_eq!(status, PluginStatus::Active);
    
    // Create a plugin that will simulate a failure
    struct FailingPlugin {
        metadata: crate::plugin::PluginMetadata,
        should_fail: bool,
    }
    
    impl Plugin for FailingPlugin {
        fn metadata(&self) -> &crate::plugin::PluginMetadata {
            &self.metadata
        }
        
        fn initialize(&self) -> BoxFuture<'_, crate::error::Result<()>> {
            let should_fail = self.should_fail;
            Box::pin(async move {
                if should_fail {
                    Err(crate::error::SquirrelError::plugin_error("Simulated failure".to_string()))
                } else {
                    Ok(())
                }
            })
        }
        
        fn shutdown(&self) -> BoxFuture<'_, crate::error::Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        
        fn get_state(&self) -> BoxFuture<'_, crate::error::Result<Option<crate::plugin::PluginState>>> {
            Box::pin(async move { Ok(None) })
        }
        
        fn set_state(&self, _state: crate::plugin::PluginState) -> BoxFuture<'_, crate::error::Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        
        fn as_any(&self) -> &dyn Any {
            self
        }
        
        fn clone_box(&self) -> Box<dyn Plugin> {
            Box::new(FailingPlugin {
                metadata: self.metadata.clone(),
                should_fail: self.should_fail,
            })
        }
    }
    
    // Create and register the failing plugin
    let failing_plugin = FailingPlugin {
        metadata: crate::plugin::PluginMetadata {
            id: Uuid::new_v4(),
            name: "failing-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Plugin that simulates failures".to_string(),
            author: "Test".to_string(),
            dependencies: vec![],
            capabilities: vec![],
        },
        should_fail: true,
    };
    
    manager.register_plugin(Box::new(failing_plugin)).await.unwrap();
    
    // Test recovery - this should fail but not panic
    let failing_id = manager.name_to_id.read().await.get("failing-plugin").unwrap().clone();
    let result = manager.load_plugin_with_recovery(failing_id).await;
    assert!(result.is_err());
    
    // Verify the plugin is in failed state
    let status = manager.get_plugin_status(failing_id).await.unwrap();
    assert_eq!(status, PluginStatus::Failed);
} 