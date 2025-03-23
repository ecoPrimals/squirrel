//! Plugin command for the Squirrel CLI
//!
//! This module implements a command to manage plugins in the Squirrel CLI.

use clap::{Command as ClapCommand, Arg, ArgAction, ArgMatches};
use async_trait::async_trait;
use tracing::{debug, warn, error};

use squirrel_commands::{Command, CommandError, CommandResult};
use crate::commands::context::CommandContext;
use crate::plugins::{state::get_plugin_manager, PluginStatus};
use crate::formatter::Factory as FormatterFactory;

/// Command to manage plugins
#[derive(Debug, Clone)]
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
    /// Get the command name
    fn name(&self) -> &str {
        "plugin"
    }
    
    /// Get the command description
    fn description(&self) -> &str {
        "Manage plugins for the Squirrel CLI"
    }
    
    /// Execute the command with the given arguments
    fn execute(&self, _args: &[String]) -> CommandResult<String> {
        // For now, we'll just return the help text
        // The actual execution will be handled by the ExecutionContext
        Ok(self.help())
    }
    
    /// Returns the command parser
    fn parser(&self) -> ClapCommand {
        ClapCommand::new("plugin")
            .about("Manage plugins for the Squirrel CLI")
            .subcommand(
                ClapCommand::new("list")
                    .about("List installed plugins")
                    .arg(
                        Arg::new("format")
                            .long("format")
                            .short('f')
                            .value_name("FORMAT")
                            .help("Output format (text, json, yaml)")
                            .default_value("text")
                    )
            )
            .subcommand(
                ClapCommand::new("info")
                    .about("Show information about a plugin")
                    .arg(
                        Arg::new("name")
                            .help("Name of the plugin")
                            .required(true)
                    )
                    .arg(
                        Arg::new("format")
                            .long("format")
                            .short('f')
                            .value_name("FORMAT")
                            .help("Output format (text, json, yaml)")
                            .default_value("text")
                    )
            )
            .subcommand(
                ClapCommand::new("enable")
                    .about("Enable a plugin")
                    .arg(
                        Arg::new("name")
                            .help("Name of the plugin")
                            .required(true)
                    )
            )
            .subcommand(
                ClapCommand::new("disable")
                    .about("Disable a plugin")
                    .arg(
                        Arg::new("name")
                            .help("Name of the plugin")
                            .required(true)
                    )
            )
            .subcommand(
                ClapCommand::new("install")
                    .about("Install a plugin")
                    .arg(
                        Arg::new("path")
                            .help("Path to the plugin directory or file")
                            .required(true)
                    )
            )
            .subcommand(
                ClapCommand::new("uninstall")
                    .about("Uninstall a plugin")
                    .arg(
                        Arg::new("name")
                            .help("Name of the plugin")
                            .required(true)
                    )
                    .arg(
                        Arg::new("force")
                            .long("force")
                            .short('f')
                            .help("Force uninstallation even if plugin is enabled")
                            .action(ArgAction::SetTrue)
                    )
            )
            .subcommand(
                ClapCommand::new("reload")
                    .about("Reload all plugins")
            )
    }
    
    /// Clone the command into a new box
    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Extension to Command for async execution with CommandContext
#[async_trait]
pub trait AsyncCommand {
    /// Execute the command asynchronously with a command context
    async fn execute_async(&self, context: &CommandContext) -> Result<String, CommandError>;
}

#[async_trait]
impl AsyncCommand for PluginCommand {
    /// Execute the command
    async fn execute_async(&self, context: &CommandContext) -> Result<String, CommandError> {
        let matches = context.matches();
        
        match matches.subcommand() {
            Some(("list", sub_matches)) => self.list_plugins(sub_matches).await,
            Some(("info", sub_matches)) => self.plugin_info(sub_matches).await,
            Some(("enable", sub_matches)) => self.enable_plugin(sub_matches).await,
            Some(("disable", sub_matches)) => self.disable_plugin(sub_matches).await,
            Some(("install", sub_matches)) => self.install_plugin(sub_matches).await,
            Some(("uninstall", sub_matches)) => self.uninstall_plugin(sub_matches).await,
            Some(("reload", sub_matches)) => self.reload_plugins(sub_matches).await,
            _ => {
                // Show help
                Ok(self.parser().render_help().to_string())
            }
        }
    }
}

impl PluginCommand {
    /// List installed plugins
    async fn list_plugins(&self, matches: &ArgMatches) -> Result<String, CommandError> {
        // Get the plugin manager
        let plugin_manager_arc = get_plugin_manager();
        let plugin_manager = plugin_manager_arc.lock().map_err(|_| 
            CommandError::ExecutionError("Failed to lock plugin manager".to_string()))?;
        
        // Get the list of plugins
        let plugins = plugin_manager.list_plugins();
        
        // Create formatted output
        let format_default = "text".to_string();
        let format = matches.get_one::<String>("format").unwrap_or(&format_default);
        let formatter = FormatterFactory::create_formatter(format)
            .map_err(|e| CommandError::ExecutionError(e.to_string()))?;
        
        // Format the plugin data
        let plugin_data: Vec<_> = plugins.iter().map(|p| {
            let metadata = p.metadata();
            serde_json::json!({
                "name": metadata.name,
                "version": metadata.version,
                "description": metadata.description,
                "author": metadata.author,
                "status": format!("{:?}", p.status()),
            })
        }).collect();
        
        formatter.format(&plugin_data)
            .map_err(|e| CommandError::ExecutionError(e.to_string()))
    }
    
    /// Show information about a plugin
    async fn plugin_info(&self, matches: &ArgMatches) -> Result<String, CommandError> {
        // Get the plugin name
        let name = matches.get_one::<String>("name")
            .ok_or_else(|| CommandError::ValidationError("Plugin name is required".to_string()))?;
        
        // Get the plugin manager
        let plugin_manager_arc = get_plugin_manager();
        let plugin_manager = plugin_manager_arc.lock().map_err(|_| 
            CommandError::ExecutionError("Failed to lock plugin manager".to_string()))?;
        
        // Get the plugin
        let plugin = plugin_manager.get_plugin(name).map_err(|e| 
            CommandError::ExecutionError(e.to_string()))?;
        
        // Create formatted output
        let format_default = "text".to_string();
        let format = matches.get_one::<String>("format").unwrap_or(&format_default);
        let formatter = FormatterFactory::create_formatter(format)
            .map_err(|e| CommandError::ExecutionError(e.to_string()))?;
        
        // Format the plugin data
        let metadata = plugin.metadata();
        let plugin_data = serde_json::json!({
            "name": metadata.name,
            "version": metadata.version,
            "description": metadata.description,
            "author": metadata.author,
            "homepage": metadata.homepage,
            "path": plugin.path().display().to_string(),
            "status": format!("{:?}", plugin.status()),
        });
        
        formatter.format(&plugin_data)
            .map_err(|e| CommandError::ExecutionError(e.to_string()))
    }
    
    /// Enable a plugin
    async fn enable_plugin(&self, matches: &ArgMatches) -> Result<String, CommandError> {
        // Get the plugin name
        let name = matches.get_one::<String>("name")
            .ok_or_else(|| CommandError::ValidationError("Plugin name is required".to_string()))?;
        
        // Get the plugin manager
        let plugin_manager_arc = get_plugin_manager();
        let mut plugin_manager = plugin_manager_arc.lock()
            .map_err(|_| CommandError::ExecutionError("Failed to lock plugin manager".to_string()))?;
        
        // Load the plugin
        match plugin_manager.load_plugin(name) {
            Ok(()) => {
                // Start the plugin
                if let Err(e) = plugin_manager.start_plugins() {
                    warn!("Failed to start plugin {}: {}", name, e);
                    return Err(CommandError::ExecutionError(format!("Failed to start plugin {}: {}", name, e)));
                }
                
                Ok(format!("Plugin {} enabled successfully", name))
            }
            Err(e) => {
                error!("Failed to enable plugin {}: {}", name, e);
                Err(CommandError::ExecutionError(format!("Failed to enable plugin {}: {}", name, e)))
            }
        }
    }
    
    /// Disable a plugin
    async fn disable_plugin(&self, matches: &ArgMatches) -> Result<String, CommandError> {
        // Get the plugin name
        let name = matches.get_one::<String>("name")
            .ok_or_else(|| CommandError::ValidationError("Plugin name is required".to_string()))?;
        
        // Get the plugin manager
        let plugin_manager_arc = get_plugin_manager();
        let mut plugin_manager = plugin_manager_arc.lock()
            .map_err(|_| CommandError::ExecutionError("Failed to lock plugin manager".to_string()))?;
        
        // Check if plugin exists and is enabled
        let plugin = plugin_manager.get_plugin(name).map_err(|e| 
            CommandError::ExecutionError(e.to_string()))?;
        if !matches!(plugin.status(), PluginStatus::Enabled) {
            return Err(CommandError::ExecutionError(format!("Plugin {} is not enabled", name)));
        }
        
        // Disable the plugin
        // For now, just set the status to Disabled
        // In a real implementation, we would unload the plugin
        let plugin = plugin_manager.get_plugin_mut(name).map_err(|e| 
            CommandError::ExecutionError(e.to_string()))?;
        plugin.set_status(PluginStatus::Disabled);
        
        Ok(format!("Plugin {} disabled successfully", name))
    }
    
    /// Install a plugin
    async fn install_plugin(&self, matches: &ArgMatches) -> Result<String, CommandError> {
        // Get the plugin path
        let path = matches.get_one::<String>("path")
            .ok_or_else(|| CommandError::ValidationError("Plugin path is required".to_string()))?;
        
        // In a real implementation, this would copy/install the plugin
        // to the appropriate directory and then load it
        // For now, just return a message
        
        Ok(format!("Plugin installation from {} is not implemented yet", path))
    }
    
    /// Uninstall a plugin
    async fn uninstall_plugin(&self, matches: &ArgMatches) -> Result<String, CommandError> {
        // Get the plugin name
        let name = matches.get_one::<String>("name")
            .ok_or_else(|| CommandError::ValidationError("Plugin name is required".to_string()))?;
        
        // Get the force flag
        let force = matches.get_flag("force");
        
        // Get the plugin manager
        let plugin_manager_arc = get_plugin_manager();
        let mut plugin_manager = plugin_manager_arc.lock().map_err(|_| 
            CommandError::ExecutionError("Failed to lock plugin manager".to_string()))?;
        
        // Check if plugin exists
        let plugin = plugin_manager.get_plugin(name).map_err(|e| 
            CommandError::ExecutionError(e.to_string()))?;
        
        // Check if plugin is enabled and force is not set
        if matches!(plugin.status(), PluginStatus::Enabled) && !force {
            return Err(CommandError::ExecutionError(
                format!("Plugin {} is enabled. Use --force to uninstall anyway", name)));
        }
        
        // Remove the plugin
        match plugin_manager.remove_plugin(name) {
            Ok(()) => {
                Ok(format!("Plugin {} uninstalled successfully", name))
            }
            Err(e) => {
                error!("Failed to uninstall plugin {}: {}", name, e);
                Err(CommandError::ExecutionError(format!("Failed to uninstall plugin {}: {}", name, e)))
            }
        }
    }
    
    /// Reload all plugins
    async fn reload_plugins(&self, _matches: &ArgMatches) -> Result<String, CommandError> {
        // Get the plugin manager
        let plugin_manager_arc = get_plugin_manager();
        let mut plugin_manager = plugin_manager_arc.lock().map_err(|_| 
            CommandError::ExecutionError("Failed to lock plugin manager".to_string()))?;
        
        // Unload all plugins
        if let Err(e) = plugin_manager.unload_plugins() {
            warn!("Failed to unload plugins: {}", e);
            // Continue anyway
        }
        
        // Get the list of plugins
        let plugins: Vec<_> = plugin_manager.list_plugins()
            .iter()
            .map(|p| p.metadata().name.clone())
            .collect();
        
        // Load each plugin
        let mut success_count = 0;
        let mut failure_count = 0;
        
        for name in plugins {
            match plugin_manager.load_plugin(&name) {
                Ok(()) => {
                    debug!("Plugin {} reloaded successfully", name);
                    success_count += 1;
                }
                Err(e) => {
                    warn!("Failed to reload plugin {}: {}", name, e);
                    failure_count += 1;
                }
            }
        }
        
        // Start plugins
        if let Err(e) = plugin_manager.start_plugins() {
            warn!("Failed to start plugins: {}", e);
            // Continue anyway
        }
        
        Ok(format!("Reloaded {} plugins ({} succeeded, {} failed)", 
                  success_count + failure_count, success_count, failure_count))
    }
} 