//! Example usage of the Plugin-Core adapter
//!
//! This module demonstrates how to use the Plugin-Core adapter in applications.

use std::sync::Arc;
use std::path::PathBuf;
use anyhow::Result;

use crate::plugin_core::{PluginCoreAdapter, PluginCoreConfig};

/// Example application function that demonstrates how to use the Plugin-Core adapter
pub async fn example_usage() -> Result<()> {
    // Create a custom configuration
    let config = PluginCoreConfig {
        auto_initialize_plugins: true,
        require_core_registration: false,
        plugin_directory: PathBuf::from("./plugins"),
        verify_signatures: false,
    };
    
    // Create the adapter with the configuration
    let mut adapter = PluginCoreAdapter::with_config(config);
    
    // Initialize the adapter
    adapter.initialize().await?;
    
    // Load plugins from the configured directory
    let plugin_ids = adapter.load_plugins().await?;
    println!("Loaded {} plugins", plugin_ids.len());
    
    // Get the status of each plugin
    for id in plugin_ids {
        let status = adapter.get_plugin_status(id).await?;
        println!("Plugin {}: {:?}", id, status);
    }
    
    // Get the status of the core component
    let core_status = adapter.get_core_status().await?;
    println!(
        "Core status: {}, Uptime: {}s, Memory: {}MB",
        core_status.status, core_status.uptime, core_status.memory_usage
    );
    
    // Get all plugins as a vector
    let plugins = adapter.get_all_plugins().await?;
    for plugin in &plugins {
        let metadata = plugin.metadata();
        println!(
            "Plugin: {} (v{})",
            metadata.name, metadata.version
        );
    }
    
    // Shutdown all plugins
    adapter.shutdown_all_plugins().await?;
    println!("All plugins shut down successfully");
    
    Ok(())
}

/// Example of using the adapter with dependency injection
pub async fn example_with_di(adapter: Arc<PluginCoreAdapter>) -> Result<()> {
    // This example shows how to use the adapter when it's provided through dependency injection
    
    // Check if the adapter is initialized
    if !adapter.is_initialized() {
        println!("Adapter not initialized");
        return Ok(());
    }
    
    // Get the core status
    let core_status = adapter.get_core_status().await?;
    println!("Core status: {}", core_status.status);
    
    // Get all plugins
    let plugins = adapter.get_all_plugins().await?;
    println!("Found {} plugins", plugins.len());
    
    Ok(())
} 