// Dynamic Plugin Loader
//
// This example application demonstrates how to load and use dynamic plugins.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;
use inquire::{Select, Text};
use serde_json::{json, Value};
use tracing::{debug, error, info, warn};

use squirrel_plugins::interfaces::{CommandsPlugin, ToolPlugin, Plugin};
use squirrel_plugins::management::PluginRegistry;

/// Command line arguments for the dynamic plugin loader
#[derive(Parser, Debug)]
#[clap(author, version, about = "Loads and interacts with dynamic plugins")]
struct Args {
    /// Path to the dynamic plugin library
    #[clap(short, long, value_name = "FILE")]
    plugin_path: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    info!("Starting dynamic plugin loader");
    
    // Parse command line arguments
    let args = Args::parse();
    
    // Create plugin registry
    let registry = Arc::new(PluginRegistry::new());
    
    // Get plugin path from command line or user input
    let plugin_path = if let Some(path) = args.plugin_path {
        path
    } else {
        let path = Text::new("Enter the path to the dynamic plugin library:")
            .with_default("target/release/dynamic_plugin_example.dll")
            .prompt()?;
        PathBuf::from(path)
    };
    
    // Verify the plugin exists
    if !plugin_path.exists() {
        error!("Plugin not found at {}", plugin_path.display());
        anyhow::bail!("Plugin not found: {}", plugin_path.display());
    }
    
    info!("Loading plugin from {}", plugin_path.display());
    
    // Load the plugin
    let plugin_id = registry.register_dynamic_plugin(&plugin_path)
        .await
        .context("Failed to register plugin")?;
    
    info!("Plugin registered with ID: {}", plugin_id);
    
    // Initialize the plugin
    registry.initialize_plugin(plugin_id)
        .await
        .context("Failed to initialize plugin")?;
    
    info!("Plugin initialized");
    
    // Start the plugin
    registry.start_plugin(plugin_id)
        .await
        .context("Failed to start plugin")?;
    
    info!("Plugin started");
    
    // Interactive plugin interaction
    interact_with_plugin(&registry, plugin_id).await?;
    
    // Stop the plugin
    registry.stop_plugin(plugin_id)
        .await
        .context("Failed to stop plugin")?;
    
    info!("Plugin stopped");
    
    // Shutdown the plugin
    registry.shutdown_plugin(plugin_id)
        .await
        .context("Failed to shutdown plugin")?;
    
    info!("Plugin shutdown");
    
    Ok(())
}

/// Interactive session with the plugin
async fn interact_with_plugin(registry: &Arc<PluginRegistry>, plugin_id: uuid::Uuid) -> Result<()> {
    let plugin_type = Select::new(
        "Select plugin type to interact with:",
        vec!["Command Plugin", "Tool Plugin", "Exit"],
    ).prompt()?;
    
    match plugin_type {
        "Command Plugin" => {
            interact_with_command_plugin(registry, plugin_id).await?;
        },
        "Tool Plugin" => {
            interact_with_tool_plugin(registry, plugin_id).await?;
        },
        "Exit" => {
            info!("Exiting plugin interaction");
            return Ok(());
        },
        _ => unreachable!(),
    }
    
    // Recurse to allow multiple interactions
    interact_with_plugin(registry, plugin_id).await
}

/// Interactive session with command plugin
async fn interact_with_command_plugin(registry: &Arc<PluginRegistry>, plugin_id: uuid::Uuid) -> Result<()> {
    // Get the plugin as a command plugin
    let plugin = registry.get_plugin(plugin_id).await
        .context("Failed to get plugin")?;
    
    // Check if the plugin implements CommandsPlugin
    let command_plugin = match plugin.as_any().downcast_ref::<Box<dyn CommandsPlugin>>() {
        Some(p) => p,
        None => {
            // Alternative method: get all command plugins and find ours
            let command_plugins = registry.get_command_plugins().await;
            let mut found_plugin = None;
            
            for plugin in command_plugins {
                if plugin.metadata().id == plugin_id {
                    found_plugin = Some(plugin);
                    break;
                }
            }
            
            match found_plugin {
                Some(p) => p,
                None => {
                    warn!("Plugin does not implement CommandsPlugin trait");
                    anyhow::bail!("Plugin does not implement CommandsPlugin trait");
                }
            }
        }
    };
    
    // Get available commands
    let commands = command_plugin.get_commands();
    if commands.is_empty() {
        warn!("Plugin has no commands");
        anyhow::bail!("Plugin has no commands");
    }
    
    // Create command list for selection
    let command_names: Vec<String> = commands.iter()
        .map(|c| format!("{}: {}", c.name, c.description))
        .collect();
    
    // Let the user select a command
    let selected = Select::new(
        "Select a command to execute:",
        command_names,
    ).prompt()?;
    
    // Extract the command name from the selection
    let command_name = selected.split(':').next().unwrap().trim();
    
    // Get command help
    let help = command_plugin.get_command_help(command_name)
        .context("Command help not available")?;
    
    println!("\n=== Command Help ===");
    println!("Name: {}", help.name);
    println!("Description: {}", help.description);
    println!("Usage: {}", help.usage);
    println!("Examples:");
    for example in &help.examples {
        println!("  {}", example);
    }
    
    // Get command schema
    let schema = command_plugin.get_command_schema(command_name)
        .context("Command schema not available")?;
    
    // Prepare command arguments
    let mut args = json!({});
    
    if let Some(props) = schema["properties"].as_object() {
        for (name, prop) in props {
            let description = prop["description"].as_str().unwrap_or(name);
            let default = prop["default"].as_str();
            
            let is_required = schema["required"].as_array()
                .map(|required| required.iter().any(|r| r.as_str().unwrap_or("") == name))
                .unwrap_or(false);
            
            let prompt = if is_required {
                format!("{} (required):", description)
            } else {
                format!("{} (optional):", description)
            };
            
            let input = if let Some(default_val) = default {
                Text::new(&prompt)
                    .with_default(default_val)
                    .prompt()?
            } else {
                Text::new(&prompt).prompt()?
            };
            
            if !input.is_empty() {
                // Try to parse as different types based on schema
                if let Some("number") = prop["type"].as_str() {
                    if let Ok(num) = input.parse::<f64>() {
                        args[name] = json!(num);
                    } else {
                        args[name] = json!(input);
                    }
                } else if let Some("boolean") = prop["type"].as_str() {
                    if input.to_lowercase() == "true" {
                        args[name] = json!(true);
                    } else if input.to_lowercase() == "false" {
                        args[name] = json!(false);
                    } else {
                        args[name] = json!(input);
                    }
                } else {
                    args[name] = json!(input);
                }
            }
        }
    }
    
    // Execute the command
    println!("\nExecuting command with args: {}", args);
    let result = command_plugin.execute_command(command_name, args).await
        .context("Command execution failed")?;
    
    // Display the result
    println!("\n=== Command Result ===");
    println!("{}", serde_json::to_string_pretty(&result)?);
    
    Ok(())
}

/// Interactive session with tool plugin
async fn interact_with_tool_plugin(registry: &Arc<PluginRegistry>, plugin_id: uuid::Uuid) -> Result<()> {
    // Get all tool plugins
    let tool_plugins = registry.get_tool_plugins().await;
    
    // Find our plugin
    let mut tool_plugin = None;
    for plugin in tool_plugins {
        if plugin.metadata().id == plugin_id {
            tool_plugin = Some(plugin);
            break;
        }
    }
    
    let tool_plugin = match tool_plugin {
        Some(p) => p,
        None => {
            warn!("Plugin does not implement ToolPlugin trait");
            anyhow::bail!("Plugin does not implement ToolPlugin trait");
        }
    };
    
    // Get available tools
    let tools = tool_plugin.get_tools();
    if tools.is_empty() {
        warn!("Plugin has no tools");
        anyhow::bail!("Plugin has no tools");
    }
    
    // Create tool list for selection
    let tool_names: Vec<String> = tools.iter()
        .map(|t| format!("{}: {}", t.name, t.description))
        .collect();
    
    // Let the user select a tool
    let selected = Select::new(
        "Select a tool to execute:",
        tool_names,
    ).prompt()?;
    
    // Extract the tool name from the selection
    let tool_name = selected.split(':').next().unwrap().trim();
    
    // Check tool availability
    let availability = tool_plugin.check_tool_availability(tool_name).await
        .context("Failed to check tool availability")?;
    
    if !availability.available {
        warn!("Tool is not available: {}", availability.reason.unwrap_or_default());
        anyhow::bail!("Tool is not available");
    }
    
    // Get tool metadata
    let metadata = tool_plugin.get_tool_metadata(tool_name)
        .context("Tool metadata not available")?;
    
    println!("\n=== Tool Metadata ===");
    println!("Name: {}", metadata.name);
    println!("Description: {}", metadata.description);
    println!("Version: {}", metadata.version);
    println!("Author: {}", metadata.author);
    
    // Get input schema
    let input_schema = &metadata.schema["input"];
    
    // Prepare tool arguments
    let mut args = json!({});
    
    if let Some(props) = input_schema["properties"].as_object() {
        for (name, prop) in props {
            let description = prop["description"].as_str().unwrap_or(name);
            let default = prop["default"].as_str();
            
            let is_required = input_schema["required"].as_array()
                .map(|required| required.iter().any(|r| r.as_str().unwrap_or("") == name))
                .unwrap_or(false);
            
            let prompt = if is_required {
                format!("{} (required):", description)
            } else {
                format!("{} (optional):", description)
            };
            
            let input = if let Some(default_val) = default {
                Text::new(&prompt)
                    .with_default(default_val)
                    .prompt()?
            } else {
                Text::new(&prompt).prompt()?
            };
            
            if !input.is_empty() {
                // Try to parse as different types based on schema
                if let Some("number") = prop["type"].as_str() {
                    if let Ok(num) = input.parse::<f64>() {
                        args[name] = json!(num);
                    } else {
                        args[name] = json!(input);
                    }
                } else if let Some("boolean") = prop["type"].as_str() {
                    if input.to_lowercase() == "true" {
                        args[name] = json!(true);
                    } else if input.to_lowercase() == "false" {
                        args[name] = json!(false);
                    } else {
                        args[name] = json!(input);
                    }
                } else {
                    args[name] = json!(input);
                }
            }
        }
    }
    
    // Execute the tool
    println!("\nExecuting tool with args: {}", args);
    let result = tool_plugin.execute_tool(tool_name, args).await
        .context("Tool execution failed")?;
    
    // Display the result
    println!("\n=== Tool Result ===");
    println!("{}", serde_json::to_string_pretty(&result)?);
    
    Ok(())
} 