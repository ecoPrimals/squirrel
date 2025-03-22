//! Plugin management command
//!
//! This module implements a command for managing plugins in the Squirrel CLI.

use clap::{Command as ClapCommand};
use squirrel_commands::{Command, CommandResult};

use crate::plugins::PluginStatus;
use crate::plugins::state::get_plugin_manager;

/// Plugin management command
#[derive(Clone)]
pub struct PluginCommand;

impl Default for PluginCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginCommand {
    /// Create a new plugin command
    pub fn new() -> Self {
        Self
    }
}

impl Command for PluginCommand {
    fn name(&self) -> &'static str {
        "plugin"
    }

    fn description(&self) -> &'static str {
        "Manage Squirrel plugins"
    }
    
    fn parser(&self) -> ClapCommand {
        ClapCommand::new("plugin")
            .about("Manage Squirrel plugins")
            .subcommand_required(false)
    }
    
    fn clone_box(&self) -> Box<dyn Command + 'static> {
        Box::new(self.clone())
    }

    fn execute(&self, args: &[String]) -> CommandResult<String> {
        // Simple command parsing
        if args.is_empty() {
            return list_plugins();
        }
        
        match args[0].as_str() {
            "list" => list_plugins(),
            "info" => {
                if args.len() < 2 {
                    Ok("Usage: plugin info <name>".to_string())
                } else {
                    plugin_info(&args[1])
                }
            },
            "install" => {
                if args.len() < 2 {
                    Ok("Usage: plugin install <path>".to_string())
                } else {
                    install_plugin(&args[1])
                }
            },
            "uninstall" => {
                if args.len() < 2 {
                    Ok("Usage: plugin uninstall <name>".to_string())
                } else {
                    uninstall_plugin(&args[1])
                }
            },
            "enable" => {
                if args.len() < 2 {
                    Ok("Usage: plugin enable <name>".to_string())
                } else {
                    enable_plugin(&args[1])
                }
            },
            "disable" => {
                if args.len() < 2 {
                    Ok("Usage: plugin disable <name>".to_string())
                } else {
                    disable_plugin(&args[1])
                }
            },
            _ => Ok(format!("Unknown plugin subcommand: {}", args[0]))
        }
    }
}

/// List all installed plugins
fn list_plugins() -> CommandResult<String> {
    let plugin_manager = get_plugin_manager();
    let mut result = String::new();
    
    // Create a scope for the manager lock
    let plugins = {
        // Lock the plugin manager to access its methods
        let manager = match plugin_manager.lock() {
            Ok(manager) => manager,
            Err(err) => return Ok(format!("Failed to access plugin manager: {}", err)),
        };
        
        // Clone the plugins to release the lock
        manager.list_plugins().into_iter().cloned().collect::<Vec<_>>()
    };
    
    if plugins.is_empty() {
        result.push_str("No plugins installed.\n");
        return Ok(result);
    }
    
    result.push_str("Installed plugins:\n");
    for plugin in plugins {
        let status = match plugin.status() {
            PluginStatus::Installed => "installed",
            PluginStatus::Enabled => "enabled",
            PluginStatus::Disabled => "disabled",
            PluginStatus::Failed(_err) => "failed",
            #[cfg(test)]
            PluginStatus::Custom(_) => "custom",
        };
        
        result.push_str(&format!("  {} (v{}) - {}\n", 
            plugin.metadata().name, 
            plugin.metadata().version, 
            status));
            
        if let Some(desc) = &plugin.metadata().description {
            result.push_str(&format!("    {}\n", desc));
        }
    }
    
    Ok(result)
}

/// Display detailed information about a plugin
fn plugin_info(name: &str) -> CommandResult<String> {
    let plugin_manager = get_plugin_manager();
    let mut result = String::new();
    
    // Lock the plugin manager to access its methods
    let plugin = {
        let manager = match plugin_manager.lock() {
            Ok(manager) => manager,
            Err(err) => return Ok(format!("Failed to access plugin manager: {}", err)),
        };
        
        match manager.get_plugin(name) {
            Ok(plugin) => plugin.clone(),
            Err(err) => return Ok(format!("Failed to get plugin: {}", err)),
        }
    };
    
    result.push_str(&format!("Plugin: {}\n", plugin.metadata().name));
    result.push_str(&format!("Version: {}\n", plugin.metadata().version));
    
    if let Some(desc) = &plugin.metadata().description {
        result.push_str(&format!("Description: {}\n", desc));
    }
    
    if let Some(author) = &plugin.metadata().author {
        result.push_str(&format!("Author: {}\n", author));
    }
    
    if let Some(homepage) = &plugin.metadata().homepage {
        result.push_str(&format!("Homepage: {}\n", homepage));
    }
    
    result.push_str(&format!("Status: {:?}\n", plugin.status()));
    result.push_str(&format!("Path: {:?}\n", plugin.path()));
    
    Ok(result)
}

/// Install a plugin from a directory or archive
fn install_plugin(path: &str) -> CommandResult<String> {
    let result = format!("Installing plugin from {}...\nNot implemented yet.", path);
    
    // TODO: Implement plugin installation logic
    // - Verify plugin structure
    // - Extract if archive
    // - Read metadata
    // - Create Plugin instance
    // - Add to PluginManager
    
    Ok(result)
}

/// Uninstall a plugin
fn uninstall_plugin(name: &str) -> CommandResult<String> {
    let plugin_manager = get_plugin_manager();
    
    // Lock the plugin manager to access its methods
    let result = {
        let mut manager = match plugin_manager.lock() {
            Ok(manager) => manager,
            Err(err) => return Ok(format!("Failed to access plugin manager: {}", err)),
        };
        
        match manager.remove_plugin(name) {
            Ok(_) => format!("Plugin '{}' uninstalled successfully.", name),
            Err(err) => format!("Failed to uninstall plugin '{}': {}", name, err),
        }
    };
    
    Ok(result)
}

/// Enable a plugin
fn enable_plugin(name: &str) -> CommandResult<String> {
    let plugin_manager = get_plugin_manager();
    
    // Lock the plugin manager to access its methods
    let result = {
        let mut manager = match plugin_manager.lock() {
            Ok(manager) => manager,
            Err(err) => return Ok(format!("Failed to access plugin manager: {}", err)),
        };
        
        match manager.get_plugin_mut(name) {
            Ok(plugin) => {
                plugin.set_status(PluginStatus::Enabled);
                format!("Plugin '{}' enabled.", name)
            },
            Err(err) => format!("Failed to enable plugin '{}': {}", name, err),
        }
    };
    
    Ok(result)
}

/// Disable a plugin
fn disable_plugin(name: &str) -> CommandResult<String> {
    let plugin_manager = get_plugin_manager();
    
    // Lock the plugin manager to access its methods
    let result = {
        let mut manager = match plugin_manager.lock() {
            Ok(manager) => manager,
            Err(err) => return Ok(format!("Failed to access plugin manager: {}", err)),
        };
        
        match manager.get_plugin_mut(name) {
            Ok(plugin) => {
                plugin.set_status(PluginStatus::Disabled);
                format!("Plugin '{}' disabled.", name)
            },
            Err(err) => format!("Failed to disable plugin '{}': {}", name, err),
        }
    };
    
    Ok(result)
} 