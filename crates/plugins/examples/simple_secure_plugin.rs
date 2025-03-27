//! # Simple Secure Plugin Example
//!
//! A simplified example that demonstrates the core functionality
//! of creating and using a plugin with security features.

use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

use squirrel_plugins::{
    PluginMetadata, PluginManager,
    CommandsPlugin, CommandsPluginBuilder,
};

/// Example of creating and using a plugin with security features
#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting simple plugin example");
    
    // Create a plugin manager with debug output
    let plugin_manager = PluginManager::new();
    println!("Plugin manager created");
    
    // Initialize the plugin manager
    match timeout(Duration::from_secs(3), plugin_manager.init()).await {
        Ok(result) => {
            result?;
            println!("Plugin manager initialized");
        },
        Err(_) => {
            println!("Timeout initializing plugin manager");
            return Ok(());
        }
    }
    
    // Create a simple plugin with a command
    let metadata = PluginMetadata::new(
        "simple-plugin",
        "1.0.0",
        "A simple plugin example",
        "Squirrel Team"
    ).with_capability("commands");
    
    let plugin = CommandsPluginBuilder::new(metadata.clone())
        .with_command_fn(
            "hello",
            "A hello command",
            |args| Box::pin(async move {
                let name = match args.get("name") {
                    Some(name) => name.as_str().unwrap_or("world"),
                    None => "world",
                };
                
                Ok(json!({
                    "message": format!("Hello, {}!", name)
                }))
            }),
        )
        .build();
    
    println!("Plugin created");
    
    // Register the plugin without signature verification
    let plugin_arc = Arc::new(plugin);
    match timeout(
        Duration::from_secs(3),
        plugin_manager.register_plugin(plugin_arc)
    ).await {
        Ok(result) => {
            result?;
            println!("Plugin registered");
        },
        Err(_) => {
            println!("Timeout registering plugin");
            return Ok(());
        }
    }
    
    // Get the plugin and execute the command
    let plugin_id = metadata.id;
    println!("Retrieving plugin with ID: {}", plugin_id);
    
    let plugin = match timeout(
        Duration::from_secs(3),
        plugin_manager.get_plugin(plugin_id)
    ).await {
        Ok(result) => {
            match result {
                Ok(plugin) => {
                    println!("Plugin retrieved");
                    plugin
                },
                Err(e) => {
                    println!("Error retrieving plugin: {}", e);
                    return Ok(());
                }
            }
        },
        Err(_) => {
            println!("Timeout retrieving plugin");
            return Ok(());
        }
    };
    
    // Cast to CommandsPlugin
    let commands_plugin = match plugin.as_any().downcast_ref::<CommandsPlugin>() {
        Some(cp) => {
            println!("Successfully cast to CommandsPlugin");
            cp
        },
        None => {
            println!("Plugin is not a CommandsPlugin");
            return Ok(());
        }
    };
    
    // Execute the command
    println!("Executing command 'hello'...");
    
    match timeout(
        Duration::from_secs(3),
        commands_plugin.execute_command("hello", json!({ "name": "user" }))
    ).await {
        Ok(result) => {
            match result {
                Ok(output) => {
                    println!("Command executed successfully");
                    println!("Result: {}", output);
                },
                Err(e) => {
                    println!("Error executing command: {}", e);
                }
            }
        },
        Err(_) => {
            println!("Timeout executing command");
        }
    }
    
    println!("Example complete!");
    Ok(())
} 