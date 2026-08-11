// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Factory for creating command registries.
//!
//! This module provides functionality for creating and configuring command registries.

use std::{fmt, sync::Arc, time::Instant};

use crate::error::CommandError;
use crate::{
    builtin::{EchoCommand, ExitCommand, HelpCommand, HistoryCommand, KillCommand, VersionCommand},
    history::CommandHistory,
    registry::{CommandRegistry, CommandResult},
};

use squirrel_interfaces::plugins::CommandsPlugin;
use tracing::{debug, info};

/// Type alias for a registry and plugin tuple
pub type RegistryWithPlugin = (Arc<CommandRegistry>, Arc<CommandRegistryPluginAdapter>);

/// The command registry factory trait
///
/// Implementations of this trait are responsible for creating and configuring
/// command registries.
pub trait CommandRegistryFactory: fmt::Debug {
    /// Create a new command registry
    fn create_registry(&self) -> Result<Arc<CommandRegistry>, CommandError>;

    /// Register built-in commands in the provided registry
    fn register_builtin_commands(
        &self,
        registry: &Arc<CommandRegistry>,
    ) -> Result<(), CommandError>;
}

/// Create a command registry with built-in commands
pub fn create_command_registry() -> Result<Arc<CommandRegistry>, CommandError> {
    debug!("Factory: Creating command registry using DefaultCommandRegistryFactory");
    let factory = DefaultCommandRegistryFactory::new();
    factory.create_registry()
}

/// Create a command registry with plugin adapter
///
/// This function creates a command registry and a plugin adapter in one call,
/// allowing the commands to be used both directly and through the plugin system.
///
/// # Returns
///
/// A tuple containing the command registry and the plugin adapter
pub fn create_command_registry_with_plugin() -> Result<RegistryWithPlugin, CommandError> {
    let registry = create_command_registry()?;
    let adapter = Arc::new(CommandRegistryPluginAdapter::new(Arc::clone(&registry)));
    Ok((registry, adapter))
}

/// Bridges `CommandRegistry` into the `CommandsPlugin` trait interface.
#[derive(Debug)]
pub struct CommandRegistryPluginAdapter {
    registry: Arc<CommandRegistry>,
    metadata: squirrel_interfaces::plugins::PluginMetadata,
}

impl CommandRegistryPluginAdapter {
    fn new(registry: Arc<CommandRegistry>) -> Self {
        Self {
            registry,
            metadata: squirrel_interfaces::plugins::PluginMetadata::new(
                "command-registry",
                env!("CARGO_PKG_VERSION"),
                "Built-in command registry plugin adapter",
                "ecoPrimals",
            ),
        }
    }
}

impl squirrel_interfaces::plugins::Plugin for CommandRegistryPluginAdapter {
    fn metadata(&self) -> &squirrel_interfaces::plugins::PluginMetadata {
        &self.metadata
    }
}

impl CommandsPlugin for CommandRegistryPluginAdapter {
    fn get_available_commands(&self) -> Vec<squirrel_interfaces::plugins::CommandMetadata> {
        let names = self.registry.list_commands().unwrap_or_default();

        names
            .into_iter()
            .map(|name| {
                let id = name.clone();
                squirrel_interfaces::plugins::CommandMetadata {
                    id,
                    name,
                    description: String::new(),
                    input_schema: serde_json::json!({"type": "array", "items": {"type": "string"}}),
                    output_schema: serde_json::json!({"type": "string"}),
                    permissions: Vec::new(),
                }
            })
            .collect()
    }

    fn get_command_metadata(
        &self,
        command_id: &str,
    ) -> Option<squirrel_interfaces::plugins::CommandMetadata> {
        let cmd = self.registry.get_command(command_id).ok()?;
        Some(squirrel_interfaces::plugins::CommandMetadata {
            id: command_id.to_string(),
            name: cmd.name().to_string(),
            description: cmd.description().to_string(),
            input_schema: serde_json::json!({"type": "array", "items": {"type": "string"}}),
            output_schema: serde_json::json!({"type": "string"}),
            permissions: Vec::new(),
        })
    }

    async fn execute_command(
        &self,
        command_id: &str,
        input: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let args: Vec<String> = match input {
            serde_json::Value::Array(arr) => arr
                .into_iter()
                .map(|v| v.as_str().unwrap_or_default().to_string())
                .collect(),
            serde_json::Value::String(s) => vec![s],
            _ => Vec::new(),
        };

        let result = self.registry.execute(command_id, &args)?;

        Ok(serde_json::Value::String(result))
    }

    fn get_command_help(&self, command_id: &str) -> Option<String> {
        let cmd = self.registry.get_command(command_id).ok()?;
        Some(cmd.help())
    }
}

/// The default command registry factory
///
/// Creates a command registry with basic built-in commands like help and version.
/// This implementation uses a deadlock-safe approach for command registration.
#[derive(Debug)]
pub struct DefaultCommandRegistryFactory;

impl DefaultCommandRegistryFactory {
    /// Creates a new instance of the default factory
    #[must_use]
    pub fn new() -> Self {
        debug!("Factory: Creating new DefaultCommandRegistryFactory instance");
        Self
    }
}

impl Default for DefaultCommandRegistryFactory {
    /// Creates a default instance of the command registry factory
    ///
    /// This is equivalent to calling `DefaultCommandRegistryFactory::new()`.
    ///
    /// # Returns
    ///
    /// A new instance of the default command registry factory
    fn default() -> Self {
        debug!("Factory: Creating DefaultCommandRegistryFactory using default implementation");
        Self
    }
}

impl CommandRegistryFactory for DefaultCommandRegistryFactory {
    fn create_registry(&self) -> Result<Arc<CommandRegistry>, CommandError> {
        debug!("Factory: Creating command registry");
        let start = Instant::now();

        let registry = Arc::new(CommandRegistry::new());
        self.register_builtin_commands(&registry)?;

        let duration = start.elapsed();
        info!("Factory: Command registry created in {:?}", duration);
        Ok(registry)
    }

    fn register_builtin_commands(
        &self,
        registry: &Arc<CommandRegistry>,
    ) -> Result<(), CommandError> {
        debug!("Factory: Registering built-in commands");
        let start = Instant::now();

        let history = Arc::new(CommandHistory::new()?);

        registry.add_post_hook(Arc::new(
            |cmd_name: &str, result: &CommandResult<String>| {
                debug!(
                    "History post-hook executed for command '{}' with result: {:?}",
                    cmd_name,
                    result.is_ok()
                );
            },
        ))?;

        registry.register("version", Arc::new(VersionCommand::new()))?;
        registry.register("echo", Arc::new(EchoCommand::new()))?;
        registry.register("exit", Arc::new(ExitCommand::new()))?;
        registry.register("kill", Arc::new(KillCommand::new()))?;
        registry.register(
            "history",
            Arc::new(HistoryCommand::new(Arc::clone(&history))),
        )?;
        registry.set_resource("command_history", Box::new(Arc::clone(&history)))?;

        let help_command = HelpCommand::new(Arc::clone(registry));
        registry.register("help", Arc::new(help_command))?;

        let duration = start.elapsed();
        info!("Factory: Built-in commands registered in {:?}", duration);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, CommandResult};
    use tracing::info;

    #[test]
    fn test_create_command_registry() -> Result<(), CommandError> {
        let _ = tracing_subscriber::fmt::try_init();
        let registry = create_command_registry()?;

        let commands = registry.list_commands()?;

        assert!(commands.contains(&"version".to_string()));
        assert!(commands.contains(&"help".to_string()));
        assert!(commands.contains(&"echo".to_string()));
        assert!(commands.contains(&"exit".to_string()));
        assert!(commands.contains(&"kill".to_string()));

        let cmd = registry.get_command("version")?;
        let version_result = cmd.execute(&[]);
        assert!(version_result.is_ok());
        assert!(version_result.expect("should succeed").contains("Version"));

        let cmd = registry.get_command("echo")?;
        let echo_result = cmd.execute(&["hello".to_string(), "world".to_string()]);
        assert!(echo_result.is_ok());
        assert_eq!(echo_result.expect("should succeed"), "Echo: hello world");

        Ok(())
    }

    #[test]
    fn test_default_factory() -> Result<(), CommandError> {
        let _ = tracing_subscriber::fmt::try_init();
        let factory = DefaultCommandRegistryFactory;
        let registry = factory.create_registry()?;

        let commands = registry.list_commands()?;

        assert!(commands.contains(&"version".to_string()));
        assert!(commands.contains(&"help".to_string()));
        assert!(commands.contains(&"echo".to_string()));
        assert!(commands.contains(&"exit".to_string()));
        assert!(commands.contains(&"kill".to_string()));

        let mut help_texts = std::collections::HashMap::new();
        for cmd_name in ["version", "echo", "help"] {
            if let Ok(help) = registry.get_help(cmd_name) {
                help_texts.insert(cmd_name.to_string(), help);
            }
        }

        assert!(help_texts.contains_key("version"));
        assert!(help_texts.contains_key("echo"));
        assert!(help_texts.contains_key("help"));

        Ok(())
    }

    #[test]
    fn test_factory_with_custom_commands() -> Result<(), CommandError> {
        #[derive(Debug, Clone)]
        struct CustomCommand;

        impl Command for CustomCommand {
            fn name(&self) -> &str {
                "custom"
            }
            fn description(&self) -> &str {
                "A custom test command"
            }
            fn execute(&self, _args: &[String]) -> CommandResult<String> {
                Ok("Custom command executed".to_string())
            }
            fn parser(&self) -> clap::Command {
                clap::Command::new("custom").about("A custom test command")
            }
            fn clone_box(&self) -> Box<dyn Command> {
                Box::new(self.clone())
            }
        }

        let _ = tracing_subscriber::fmt::try_init();
        let factory = DefaultCommandRegistryFactory;
        let registry = factory.create_registry()?;

        registry.register("custom", Arc::new(CustomCommand))?;

        let cmd = registry.get_command("custom")?;
        let result = cmd.execute(&[]);
        assert!(result.is_ok());
        assert_eq!(result.expect("should succeed"), "Custom command executed");

        Ok(())
    }

    #[test]
    fn test_create_command_registry_with_plugin_produces_adapter() {
        let result = create_command_registry_with_plugin();
        assert!(result.is_ok());
        let (registry, adapter) = result.expect("tested above");
        let cmds = adapter.get_available_commands();
        assert!(!cmds.is_empty());

        let reg_cmds = registry.list_commands().expect("ok");
        assert_eq!(cmds.len(), reg_cmds.len());
    }

    #[tokio::test]
    async fn test_plugin_adapter_execute_command() {
        let (_registry, adapter) = create_command_registry_with_plugin().expect("ok");
        let result = adapter
            .execute_command("echo", serde_json::json!(["hello", "world"]))
            .await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_plugin_adapter_get_command_help() {
        let (_registry, adapter) = create_command_registry_with_plugin().expect("ok");
        let help = adapter.get_command_help("echo");
        assert!(help.is_some());
    }

    #[test]
    fn test_plugin_adapter_get_command_metadata() {
        let (_registry, adapter) = create_command_registry_with_plugin().expect("ok");
        let meta = adapter.get_command_metadata("echo");
        assert!(meta.is_some());
        let meta = meta.expect("tested above");
        assert_eq!(meta.id, "echo");
    }
}
