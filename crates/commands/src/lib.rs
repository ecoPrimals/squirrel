//! Command system for Squirrel
//!
//! This crate provides functionality for command registration, validation,
//! and execution within the Squirrel system.

use std::sync::Arc;
use thiserror::Error;
use anyhow::Result;

/// Built-in commands
pub mod builtin;

/// Command execution hooks
pub mod hooks;

/// Command lifecycle management
pub mod lifecycle;

/// Resource management for commands
pub mod resources;

/// Command validation
pub mod validation;

/// Command history system
pub mod history;

/// Command suggestions system
pub mod suggestions;

/// Command authentication and authorization system
pub mod auth;

/// Command registry
mod registry;
pub use registry::{Command, CommandRegistry, CommandResult};

/// Command errors
#[derive(Debug, Error)]
pub enum CommandError {
    /// Error during command registration
    #[error("Registration error: {0}")]
    RegistrationError(String),
    
    /// Error during command execution
    #[error("Execution error: {0}")]
    ExecutionError(String),
    
    /// Error during command validation
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    /// Error during command resource management
    #[error("Resource error: {0}")]
    ResourceError(String),
    
    /// Error when command is not found
    #[error("Command not found: {0}")]
    CommandNotFound(String),
    
    /// Error related to command registry operations
    #[error("Registry error: {0}")]
    RegistryError(String),
    
    /// Error when attempting to register a command that already exists
    #[error("Command already exists: {0}")]
    CommandAlreadyExists(String),
    
    /// Error related to authentication
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    
    /// Error related to authorization
    #[error("Authorization error: {0}")]
    AuthorizationError(String),
}

/// Command factory for creating command registries
mod factory;
pub use factory::{CommandRegistryFactory, create_command_registry, create_command_registry_with_plugin};

/// Re-export common types from the core crate
pub use squirrel_core::error::Result;

/// Adapter module
pub mod adapter;

#[cfg(test)]
mod tests;

/// Register the command system as a plugin
pub async fn register_plugin(
    registry: &(impl squirrel_interfaces::plugins::PluginRegistry + ?Sized)
) -> Result<String> {
    use adapter::plugins::create_commands_plugin_adapter;
    use factory::create_command_registry;
    
    // Create a command registry
    let cmd_registry = create_command_registry()?;
    
    // Create the plugin adapter
    let plugin = create_commands_plugin_adapter(cmd_registry);
    
    // Initialize the plugin
    plugin.initialize().await?;
    
    // Register the plugin
    registry.register_plugin(plugin).await
} 