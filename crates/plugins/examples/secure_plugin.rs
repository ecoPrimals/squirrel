//! # Secure Plugin Example
//!
//! This example demonstrates how to create and use plugins with security features
//! like signature verification. It shows:
//!
//! * Creating a plugin with the builder pattern
//! * Setting up signature verification
//! * Registering a plugin with a signature
//! * Executing commands from a plugin
//! * Working with directory-based plugin discovery
//!
//! The example is a simplified demonstration of the plugin security system.
//!
//! ## NOTE: WORK IN PROGRESS
//!
//! This example is currently a work in progress and has known issues:
//! - There are potential hanging problems due to async operations
//! - The signature verification implementation is incomplete
//! - Error handling needs improvement
//!
//! A more robust implementation of real-world signature verification
//! will be added in a later development stage. For now, use the
//! `simple_secure_plugin.rs` example for basic plugin functionality.

use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

use squirrel_plugins::{
    PluginMetadata, PluginManager,
    CommandsPlugin, CommandsPluginBuilder,
    security::signature::{
        SignatureVerifier, SignatureAlgorithm, SignatureScope, SignatureVerifierConfig
    },
};

/// Example of creating and using a plugin with signature verification
#[tokio::main]
async fn main() -> Result<()> {
    // Create a plugin manager
    let plugin_manager = PluginManager::new();
    
    // Initialize the plugin manager
    println!("Initializing plugin manager...");
    plugin_manager.init().await?;
    println!("Plugin manager initialized successfully");
    
    // Enable dev mode in the security manager by directly accessing the signature verifier
    println!("Creating signature verifier with dev mode enabled...");
    let temp_dir = tempfile::tempdir()?;
    let signature_verifier = SignatureVerifier::with_storage_dir(temp_dir.path().to_path_buf());
    
    // Configure the signature verifier to use dev mode
    let mut config = SignatureVerifierConfig::default();
    config.dev_mode = true;
    config.allow_unsigned_in_dev_mode = true;
    config.require_signatures = false;
    signature_verifier.set_config(config).await?;
    
    println!("Signature verifier configured for dev mode");
    
    // Create a plugin with the builder pattern
    let metadata = PluginMetadata::new(
        "example-secure-plugin",
        "1.0.0",
        "An example of a secure plugin",
        "Squirrel Team"
    ).with_capability("commands");
    
    let plugin = CommandsPluginBuilder::new(metadata.clone())
        .with_command_fn(
            "secure-hello",
            "A secure hello command",
            |args| Box::pin(async move {
                let name = match args.get("name") {
                    Some(name) => name.as_str().unwrap_or("secure world"),
                    None => "secure world",
                };
                
                Ok(json!({
                    "message": format!("Hello, {}! This is a secure command.", name)
                }))
            }),
        )
        .build();
    
    // Generate a "signature" for our plugin
    // Note: In a real application, this would be done using proper cryptographic signing
    let private_key = vec![0u8; 32]; // Placeholder for a real private key
    let signature = signature_verifier.sign_plugin(
        &metadata,
        None, // No binary path for this example
        &private_key,
        SignatureAlgorithm::Ed25519,
        "Example Signer",
        SignatureScope::Metadata
    ).await?;
    
    // For demonstration purposes, we'll turn on dev mode in the plugin manager's security manager
    // to allow our placeholder signature to be accepted
    println!("Note: Using dev mode to bypass signature verification for this example");
    println!("      In a real application, you'd use proper cryptographic signatures");
    
    // Register our plugin - note that the signature verification will fail with our placeholder
    // but we're showing the code pattern for how it would work with real signatures
    let plugin_arc = Arc::new(plugin);
    if let Err(e) = plugin_manager.register_plugin_with_signature(
        plugin_arc.clone(), 
        Some(signature.signature.clone())
    ).await {
        println!("Expected error with placeholder signature: {}", e);
        println!("Registering without signature verification for this example...");
        
        // Fall back to registering without signature verification
        plugin_manager.register_plugin(plugin_arc).await?;
    }
    
    // Execute a command from our plugin
    let plugin_id = metadata.id;
    println!("Retrieving plugin with ID: {}", plugin_id);
    
    // Add a timeout to the get_plugin operation
    let plugin = match timeout(Duration::from_secs(5), plugin_manager.get_plugin(plugin_id)).await {
        Ok(result) => {
            match result {
                Ok(plugin) => {
                    println!("Plugin retrieved successfully");
                    plugin
                },
                Err(e) => {
                    println!("Error retrieving plugin: {}", e);
                    return Ok(());
                }
            }
        },
        Err(_) => {
            println!("Timeout while retrieving plugin");
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
    
    // Execute the command with a timeout
    println!("Executing command 'secure-hello'...");
    let result = match timeout(
        Duration::from_secs(5),
        commands_plugin.execute_command("secure-hello", json!({ "name": "secure user" }))
    ).await {
        Ok(result) => {
            match result {
                Ok(output) => {
                    println!("Command executed successfully");
                    output
                },
                Err(e) => {
                    println!("Error executing command: {}", e);
                    return Ok(());
                }
            }
        },
        Err(_) => {
            println!("Timeout while executing command");
            return Ok(());
        }
    };
    
    println!("Command result: {}", result);
    
    // Create a directory-based example
    println!("Creating directory for plugin example...");
    let plugin_dir = temp_dir.path().join("plugins");
    std::fs::create_dir_all(&plugin_dir)?;
    
    // Create a simplified plugin file
    let plugin_metadata = r#"{
        "id": "123e4567-e89b-12d3-a456-426614174000",
        "name": "example-dir-plugin",
        "version": "1.0.0",
        "description": "An example plugin loaded from a directory",
        "author": "Squirrel Team",
        "capabilities": ["commands"]
    }"#;
    
    println!("Writing plugin metadata file...");
    let plugin_path = plugin_dir.join("example-dir-plugin.json");
    std::fs::write(&plugin_path, plugin_metadata)?;
    
    // Create a signature file
    println!("Writing signature file...");
    let sig_file_path = plugin_dir.join("example-dir-plugin.sig");
    // In a real application, this would be a proper signature
    std::fs::write(&sig_file_path, "example-signature")?;
    
    println!("Loading plugins from directory: {}", plugin_dir.display());
    
    // In a real application, you'd load plugins from the directory using:
    // match timeout(
    //     Duration::from_secs(10),
    //     plugin_manager.load_plugins(plugin_dir.to_str().unwrap())
    // ).await {
    //     Ok(result) => {
    //         match result {
    //             Ok(ids) => println!("Loaded {} plugins from directory", ids.len()),
    //             Err(e) => println!("Failed to load plugins: {}", e),
    //         }
    //     },
    //     Err(_) => println!("Timeout while loading plugins from directory"),
    // }
    
    // This would normally load plugins, but for this example it's just a demonstration
    // since we're not actually implementing the discovery mechanism for this example
    println!("To fully implement this example, you would:");
    println!("1. Create plugin implementation files in the directory");
    println!("2. Sign the plugins with a proper signing mechanism");
    println!("3. Use PluginDiscovery to find and load the plugins");
    println!("4. Register the discovered plugins with the PluginManager");
    
    println!("\nExample complete!");
    
    Ok(())
} 