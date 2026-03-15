// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use crate::commands::registry::CommandRegistry;
use async_trait::async_trait;
use clap::{Arg, Command as ClapCommand};
use squirrel_commands::Command;
use std::sync::Arc;

use crate::plugins::error::PluginError;
use crate::plugins::plugin::Plugin;
use crate::plugins::state::PluginState;

/// Example command provided by the example plugin
#[derive(Debug, Clone)]
pub struct ExampleCommand;

impl Command for ExampleCommand {
    fn name(&self) -> &'static str {
        "example"
    }

    fn description(&self) -> &'static str {
        "An example command provided by the example plugin"
    }

    fn execute(&self, _args: &[String]) -> squirrel_commands::CommandResult<String> {
        Ok("Example command executed successfully!".to_string())
    }

    fn parser(&self) -> clap::Command {
        ClapCommand::new("example")
            .about("An example command provided by the example plugin")
            .arg(
                Arg::new("message")
                    .help("Optional message to display")
                    .required(false),
            )
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Example plugin implementation
pub struct ExamplePlugin {
    name: String,
    version: String,
    description: String,
    state: PluginState,
    commands: Vec<Arc<dyn Command>>,
}

impl Default for ExamplePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl ExamplePlugin {
    /// Create a new instance of the example plugin
    pub fn new() -> Self {
        let commands = vec![Arc::new(ExampleCommand) as Arc<dyn Command>];

        Self {
            name: "example-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "An example plugin for the Squirrel CLI".to_string(),
            state: PluginState::Created,
            commands,
        }
    }

    /// Handle state transition
    #[allow(dead_code)] // State machine infrastructure for plugin lifecycle
    fn transition_to(&mut self, new_state: PluginState) -> Result<(), PluginError> {
        if !PluginState::is_valid_transition(self.state, new_state) {
            return Err(PluginError::ValidationError(format!(
                "Invalid state transition from {:?} to {:?}",
                self.state, new_state
            )));
        }

        self.state = new_state;
        Ok(())
    }
}

#[async_trait]
impl Plugin for ExamplePlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn description(&self) -> Option<&str> {
        Some(&self.description)
    }

    async fn initialize(&self) -> Result<(), PluginError> {
        // In a real plugin, we might load configuration, connect to services, etc.
        tracing::info!("Example plugin initializing...");
        Ok(())
    }

    async fn start(&self) -> Result<(), PluginError> {
        // In a real plugin, we might start background workers, connect to services, etc.
        tracing::info!("Example plugin starting...");
        Ok(())
    }

    fn register_commands(&self, registry: &CommandRegistry) -> Result<(), PluginError> {
        for command in &self.commands {
            match registry.register(command.name(), command.clone()) {
                Ok(_) => tracing::info!("Registered command: {}", command.name()),
                Err(e) => {
                    return Err(PluginError::RegisterError(format!(
                        "Failed to register command {}: {}",
                        command.name(),
                        e
                    )));
                }
            }
        }

        Ok(())
    }

    fn commands(&self) -> Vec<Arc<dyn Command>> {
        self.commands.clone()
    }

    async fn execute(&self, args: &[String]) -> Result<String, PluginError> {
        if args.is_empty() {
            return Ok("Example plugin: No command specified".to_string());
        }

        match args[0].as_str() {
            "status" => Ok(format!("Example plugin status: {}", self.state)),
            "info" => Ok(format!(
                "Example plugin: {} v{}\n{}",
                self.name,
                self.version,
                self.description.as_str()
            )),
            _ => Err(PluginError::Unknown(format!(
                "Unknown plugin command: {}",
                args[0]
            ))),
        }
    }

    async fn stop(&self) -> Result<(), PluginError> {
        // In a real plugin, we might stop background workers, close connections, etc.
        tracing::info!("Example plugin stopping...");
        Ok(())
    }

    async fn cleanup(&self) -> Result<(), PluginError> {
        // In a real plugin, we might close connections, save state, etc.
        tracing::info!("Example plugin cleaning up...");
        Ok(())
    }
}

/// Plugin factory for creating the example plugin
pub struct ExamplePluginFactory;

impl crate::plugins::plugin::PluginFactory for ExamplePluginFactory {
    fn create(&self) -> Result<Arc<dyn Plugin>, PluginError> {
        Ok(Arc::new(ExamplePlugin::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_example_plugin() {
        let plugin = ExamplePlugin::new();

        // Test initialization
        assert!(plugin.initialize().await.is_ok());

        // Test command registration
        let mut registry = CommandRegistry::new();
        assert!(plugin.register_commands(&mut registry).is_ok());

        // Test plugin execution
        let result = plugin.execute(&["status".to_string()]).await;
        assert!(result.is_ok());

        // Test cleanup
        assert!(plugin.cleanup().await.is_ok());
    }
}
