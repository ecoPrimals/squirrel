// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Hello World plugin for Squirrel CLI
//!
//! This is a simple plugin that adds a "hello" command to the CLI.

#![deny(unsafe_code)]
use async_trait::async_trait;
use clap::{Arg, Command as ClapCommand};
use log::{debug, info};
use std::sync::Arc;

use squirrel_cli::plugins::{Plugin, PluginError, PluginFactory};
use squirrel_cli::{Command, CommandContext, CommandResult};
use squirrel_cli::command_adapter::CommandRegistry;

/// Hello command implementation
#[derive(Debug, Clone)]
pub struct HelloCommand;

impl HelloCommand {
    /// Create a new hello command
    pub fn new() -> Self {
        Self
    }


}

#[async_trait]
impl Command for HelloCommand {
    /// Get the command name
    fn name(&self) -> &str {
        "hello"
    }

    /// Get the command description
    fn description(&self) -> &str {
        "Say hello from a plugin"
    }

    /// Execute the command
    async fn execute(&self, context: &CommandContext) -> CommandResult<String> {
        debug!("Executing hello command");

        let matches = context.state.matches();
        let name = matches
            .get_one::<String>("name")
            .unwrap_or(&"world".to_string());

        Ok(format!("Hello, {}! This is coming from a plugin.", name))
    }

    /// Get the command parser
    fn parser(&self) -> ClapCommand {
        ClapCommand::new("hello")
            .about("Hello world command from a plugin")
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("Name to greet")
                    .default_value("world"),
            )
    }

    /// Clone the command as a boxed trait object
    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Hello plugin implementation
pub struct HelloPlugin;

#[async_trait]
impl Plugin for HelloPlugin {
    /// Get the plugin name
    fn name(&self) -> &str {
        "hello"
    }

    /// Get the plugin version
    fn version(&self) -> &str {
        "1.0.0"
    }

    /// Get the plugin description
    fn description(&self) -> Option<&str> {
        Some("A simple hello world plugin")
    }

    /// Initialize the plugin
    async fn initialize(&self) -> Result<(), PluginError> {
        info!("Initializing hello plugin");
        Ok(())
    }

    /// Register commands provided by this plugin
    fn register_commands(&self, registry: &CommandRegistry) -> Result<(), PluginError> {
        info!("Registering hello command");
        let command = HelloCommand::new();
        // Note: Registry is now immutable, so we can't register commands directly
        // This would need to be handled differently in a real implementation
        Ok(())
    }

    /// Return the list of commands provided by this plugin
    fn commands(&self) -> Vec<Arc<dyn Command>> {
        vec![Arc::new(HelloCommand::new())]
    }

    /// Execute plugin functionality
    async fn execute(&self, args: &[String]) -> Result<String, PluginError> {
        debug!("Executing hello plugin with args: {:?}", args);

        // Just return a simple message in this example
        Ok("Hello plugin executed successfully".to_string())
    }

    /// Clean up plugin resources
    async fn cleanup(&self) -> Result<(), PluginError> {
        info!("Cleaning up hello plugin");
        Ok(())
    }
}

/// Hello plugin factory implementation
#[no_mangle]
pub fn create_plugin() -> Result<Arc<dyn Plugin>, PluginError> {
    Ok(Arc::new(HelloPlugin))
}

/// Plugin factory registration
#[no_mangle]
pub fn register_plugin_factory() -> Arc<dyn PluginFactory> {
    struct HelloPluginFactory;

    impl PluginFactory for HelloPluginFactory {
        fn create(&self) -> Result<Arc<dyn Plugin>, PluginError> {
            create_plugin()
        }
    }

    Arc::new(HelloPluginFactory)
}
