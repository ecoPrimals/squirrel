//! Plugin Adapter implementation
//!
//! This module contains the implementation of the CommandsPluginAdapter, which
//! adapts the command registry for use with a plugin system interface.

use std::sync::Arc;
use async_trait::async_trait;

use crate::command::MockCommand;
use crate::error::AdapterResult;
use super::registry::{CommandRegistryAdapter, MockAdapter};

/// Adapter for integrating commands with a plugin system
///
/// This adapter transforms the command registry interface into one that
/// can be used by a plugin system, providing a unified way to discover
/// and execute commands.
#[derive(Debug)]
pub struct CommandsPluginAdapter {
    /// The underlying command registry adapter
    adapter: CommandRegistryAdapter,
    
    /// Plugin identifier
    plugin_id: String,
    
    /// Plugin version
    version: String,
}

impl CommandsPluginAdapter {
    /// Creates a new plugin adapter with default settings
    pub fn new() -> Self {
        Self {
            adapter: CommandRegistryAdapter::new(),
            plugin_id: "commands".to_string(),
            version: "1.0.0".to_string(),
        }
    }
    
    /// Creates a plugin adapter with custom metadata
    pub fn with_metadata(plugin_id: &str, version: &str) -> Self {
        Self {
            adapter: CommandRegistryAdapter::new(),
            plugin_id: plugin_id.to_string(),
            version: version.to_string(),
        }
    }
    
    /// Creates a plugin adapter with an existing registry adapter
    pub fn with_adapter(adapter: CommandRegistryAdapter) -> Self {
        Self {
            adapter,
            plugin_id: "commands".to_string(),
            version: "1.0.0".to_string(),
        }
    }
    
    /// Checks if the adapter is initialized
    pub fn is_initialized(&self) -> bool {
        self.adapter.is_initialized()
    }
    
    /// Gets the plugin ID
    pub fn plugin_id(&self) -> &str {
        &self.plugin_id
    }
    
    /// Gets the plugin version
    pub fn version(&self) -> &str {
        &self.version
    }
    
    /// Registers a command with the registry
    ///
    /// # Arguments
    ///
    /// * `command` - The command to register
    ///
    /// # Returns
    ///
    /// * `Ok(())` if registration succeeded
    /// * `Err(AdapterError)` if registration failed
    pub fn register_command(&self, command: Arc<dyn MockCommand + Send + Sync>) -> AdapterResult<()> {
        self.adapter.register_command(command)
    }
    
    /// Executes a command through the plugin interface
    ///
    /// # Arguments
    ///
    /// * `command` - The name of the command to execute
    /// * `args` - The arguments to pass to the command
    ///
    /// # Returns
    ///
    /// * `Ok(String)` containing the command output
    /// * `Err(AdapterError)` if execution failed
    pub async fn execute_command(&self, command: &str, args: Vec<String>) -> AdapterResult<String> {
        self.adapter.execute(command, args).await
    }
    
    /// Gets help information for a command
    ///
    /// # Arguments
    ///
    /// * `command` - The name of the command to get help for
    ///
    /// # Returns
    ///
    /// * `Ok(String)` containing the help information
    /// * `Err(AdapterError)` if getting help failed
    pub async fn get_command_help(&self, command: &str) -> AdapterResult<String> {
        self.adapter.get_help(command).await
    }
    
    /// Gets all available commands
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` containing command names
    /// * `Err(AdapterError)` if getting commands failed
    pub async fn get_commands(&self) -> AdapterResult<Vec<String>> {
        self.adapter.list_commands().await
    }
}

#[async_trait]
impl MockAdapter for CommandsPluginAdapter {
    async fn execute(&self, command: &str, args: Vec<String>) -> AdapterResult<String> {
        self.execute_command(command, args).await
    }
    
    async fn get_help(&self, command: &str) -> AdapterResult<String> {
        self.get_command_help(command).await
    }
} 