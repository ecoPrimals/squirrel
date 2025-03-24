//! Galaxy plugin example
//!
//! This module provides an example of how to use the Galaxy plugin.

use std::sync::Arc;
use anyhow::Result;
use serde_json::json;

use crate::manager::{PluginManager, DefaultPluginManager};
use crate::galaxy::{GalaxyPlugin, GalaxyAdapterPlugin, GalaxyAdapterPluginConfig};
use crate::plugin::Plugin;

/// Example function showing how to use the Galaxy plugin
pub async fn galaxy_plugin_example() -> Result<()> {
    // Create the plugin manager
    let mut plugin_manager = DefaultPluginManager::new();
    
    // Create the Galaxy plugin
    let config = GalaxyAdapterPluginConfig {
        api_url: "https://usegalaxy.org/api".to_string(),
        api_key: "your_api_key".to_string(),
        timeout: Some(30),
    };
    
    let galaxy_plugin = GalaxyAdapterPlugin::new(config);
    
    // Register the plugin
    plugin_manager.register_plugin(Arc::new(galaxy_plugin))?;
    
    // Initialize plugins
    plugin_manager.initialize_all_plugins().await?;
    
    // Find the Galaxy plugin
    let plugins = plugin_manager.get_plugins_by_capability("galaxy-integration");
    
    if let Some(plugin) = plugins.first() {
        // Cast to GalaxyPlugin
        if let Some(galaxy_plugin) = plugin.downcast_ref::<dyn GalaxyPlugin>() {
            // Connect to Galaxy
            galaxy_plugin.connect(json!({
                "url": "https://usegalaxy.org/api",
                "key": "your_api_key"
            })).await?;
            
            // Send data to Galaxy
            let result = galaxy_plugin.send_data(json!({
                "history_id": "default",
                "name": "example_dataset",
                "file_type": "txt",
                "content": "Hello, Galaxy!"
            })).await?;
            
            println!("Sent data to Galaxy: {:?}", result);
            
            // Receive data from Galaxy
            let data = galaxy_plugin.receive_data().await?;
            println!("Received data from Galaxy: {:?}", data);
        }
    }
    
    // Shutdown plugins
    plugin_manager.shutdown_all_plugins().await?;
    
    Ok(())
}

/// Example showing how to directly use the Galaxy adapter plugin
pub async fn direct_adapter_example() -> Result<()> {
    // Create the Galaxy plugin
    let config = GalaxyAdapterPluginConfig {
        api_url: "https://usegalaxy.org/api".to_string(),
        api_key: "your_api_key".to_string(),
        timeout: Some(30),
    };
    
    let galaxy_plugin = GalaxyAdapterPlugin::new(config);
    
    // Initialize the plugin
    galaxy_plugin.initialize().await?;
    
    // Use the plugin directly
    let result = galaxy_plugin.send_data(json!({
        "history_id": "default",
        "name": "example_dataset",
        "file_type": "txt",
        "content": "Hello, Galaxy!"
    })).await?;
    
    println!("Sent data to Galaxy: {:?}", result);
    
    // Shutdown the plugin
    galaxy_plugin.shutdown().await?;
    
    Ok(())
} 