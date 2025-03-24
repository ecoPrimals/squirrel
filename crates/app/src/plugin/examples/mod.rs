//! Plugin example implementations
//!
//! This module contains example implementations of various plugin types
//! to demonstrate how to create and use plugins in the Squirrel system.

pub mod example_plugin;
pub mod mcp_plugin_example;
pub mod tool_plugin_example;

pub use example_plugin::{create_example_command_plugin, AdvancedExamplePlugin};
pub use mcp_plugin_example::{create_example_mcp_plugin, AdvancedMcpPlugin};
pub use tool_plugin_example::{create_example_tool_plugin, CodeToolsPlugin};

use crate::plugin::{Plugin, PluginMetadata};
use uuid::Uuid;

/// Create a collection of example plugins for testing or demonstration
#[must_use] pub fn create_example_plugins() -> Vec<Box<dyn Plugin>> {
    let command_plugin = example_plugin::create_example_command_plugin();
    let mcp_plugin = mcp_plugin_example::create_example_mcp_plugin();
    let tool_plugin = tool_plugin_example::create_example_tool_plugin();
    
    vec![
        // Convert command plugin to base Plugin
        command_plugin.clone_box(),
        // Unwrap MCP plugin from Arc and get the Plugin implementation
        mcp_plugin.clone_box(),
        // Create a new tool plugin instance rather than trying to upcast
        Box::new(CodeToolsPlugin::new())
    ]
}

/// Create an advanced command plugin with a unique ID
#[must_use] pub fn create_advanced_command_plugin() -> Box<dyn Plugin> {
    Box::new(AdvancedExamplePlugin::new())
}

/// Create an advanced MCP plugin with a unique ID
#[must_use] pub fn create_advanced_mcp_plugin() -> Box<dyn Plugin> {
    Box::new(AdvancedMcpPlugin::new())
}

/// Create an advanced tool plugin with a unique ID
#[must_use] pub fn create_advanced_tool_plugin() -> Box<dyn Plugin> {
    Box::new(CodeToolsPlugin::new())
}

/// Create a set of plugins with dependencies
#[must_use] pub fn create_plugin_dependency_chain() -> Vec<Box<dyn Plugin>> {
    // Create base plugin with no dependencies
    let base_id = Uuid::new_v4();
    let base_metadata = PluginMetadata {
        id: base_id,
        name: "base-plugin".to_string(),
        version: "0.1.0".to_string(),
        description: "Base plugin with no dependencies".to_string(),
        author: "Squirrel Team".to_string(),
        dependencies: vec![],
        capabilities: vec!["command".to_string()],
    };
    let base_plugin = example_plugin::AdvancedExamplePlugin::with_metadata(base_metadata);
    
    // Create mid-level plugin that depends on base
    let mid_id = Uuid::new_v4();
    let mid_metadata = PluginMetadata {
        id: mid_id,
        name: "mid-plugin".to_string(),
        version: "0.1.0".to_string(),
        description: "Mid-level plugin that depends on base".to_string(),
        author: "Squirrel Team".to_string(),
        dependencies: vec![base_id.to_string()],
        capabilities: vec!["tool".to_string()],
    };
    let mid_plugin = tool_plugin_example::CodeToolsPlugin::with_metadata(mid_metadata);
    
    // Create top-level plugin that depends on mid-level
    let top_metadata = PluginMetadata {
        id: Uuid::new_v4(),
        name: "top-plugin".to_string(),
        version: "0.1.0".to_string(),
        description: "Top-level plugin that depends on mid-level".to_string(),
        author: "Squirrel Team".to_string(),
        dependencies: vec![mid_id.to_string()],
        capabilities: vec!["mcp".to_string()],
    };
    let top_plugin = mcp_plugin_example::AdvancedMcpPlugin::with_metadata(top_metadata);
    
    // Return plugins in non-dependency order to test loading order
    vec![
        Box::new(top_plugin),
        Box::new(base_plugin), 
        Box::new(mid_plugin),
    ]
} 