// Example demonstrating the monitoring plugin system
//
// This example shows how to use the plugin system to collect metrics
// from both default and custom plugins.

use std::sync::Arc;
use anyhow::Result;
use tokio::time::sleep;
use std::time::Duration;

// Import necessary components from the monitoring crate
use squirrel_monitoring::plugins::{
    PluginManager, 
    PluginRegistry, 
    create_default_plugin_manager,
    MonitoringPlugin,
};
use squirrel_monitoring::plugins::examples::CustomMetricsPlugin;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("Running plugin examples...");
    
    // Run examples
    example_default_plugins().await?;
    example_custom_plugins().await?;
    
    println!("Plugin examples completed successfully.");
    Ok(())
}

/// Example using default plugins
async fn example_default_plugins() -> Result<()> {
    println!("\n=== Default Plugins Example ===");
    
    // Create a default plugin manager with all built-in plugins
    let plugin_manager = create_default_plugin_manager().await?;
    
    // Collect metrics from all plugins
    println!("Collecting metrics from default plugins...");
    let metrics = plugin_manager.collect_metrics().await?;
    
    // Print metrics
    println!("Default plugin count: {}", metrics.len());
    for (plugin_name, plugin_metrics) in metrics {
        println!("Plugin {}: {}", plugin_name, plugin_metrics);
    }
    
    Ok(())
}

/// Example using custom plugins
async fn example_custom_plugins() -> Result<()> {
    println!("\n=== Custom Plugins Example ===");
    
    // Create plugin registry and manager
    let plugin_manager = PluginManager::new();
    
    // Create a custom plugin
    let custom_plugin = Arc::new(CustomMetricsPlugin::new());
    
    // Get plugin information
    let plugin_id = custom_plugin.metadata().id;
    
    println!("Created custom plugin: {}", custom_plugin.metadata().name);
    
    // Register the plugin
    println!("Registering custom plugin...");
    plugin_manager.register_plugin(custom_plugin).await?;
    
    // Initialize the plugin
    println!("Initializing custom plugin...");
    plugin_manager.initialize_plugin(plugin_id).await?;
    
    // Collect metrics
    println!("Collecting metrics from all plugins...");
    let metrics = plugin_manager.collect_metrics().await?;
    println!("Initial metrics: {:?}", metrics);
    
    // Wait a bit
    println!("Waiting for metrics to change...");
    sleep(Duration::from_secs(1)).await;
    
    // Collect metrics again
    let updated_metrics = plugin_manager.collect_metrics().await?;
    println!("Updated metrics: {:?}", updated_metrics);
    
    // Disable the plugin
    println!("Disabling custom plugin...");
    plugin_manager.disable_plugin(plugin_id)?;
    
    // Try to collect metrics while disabled
    println!("Attempting to collect metrics while plugin is disabled...");
    let disabled_metrics = plugin_manager.collect_metrics().await?;
    println!("Metrics after disabling: {:?}", disabled_metrics);
    
    // Re-enable the plugin
    println!("Re-enabling custom plugin...");
    plugin_manager.enable_plugin(plugin_id)?;
    
    // Collect metrics after re-enabling
    println!("Collecting metrics after re-enabling...");
    let final_metrics = plugin_manager.collect_metrics().await?;
    println!("Final metrics: {:?}", final_metrics);
    
    Ok(())
} 