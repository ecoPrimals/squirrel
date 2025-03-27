//! Command system for Squirrel
//!
//! This crate provides functionality for command registration, validation,
//! and execution within the Squirrel system.

use thiserror::Error;
use squirrel_interfaces::plugins::Plugin;
use std::sync::Arc;

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

/// Command transaction system for reliable execution
pub mod transaction;

/// Command journaling system for persistent logging and recovery
pub mod journal;

/// Command observability system for tracing and metrics
pub mod observability;

/// Command registry
mod registry;
pub use registry::{Command, CommandRegistry, CommandResult};

/// Command errors
#[derive(Debug, Error, Clone)]
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
    
    /// Error related to transaction operations
    #[error("Transaction error: {0}")]
    TransactionError(#[from] transaction::TransactionError),
    
    /// Error related to journal operations
    #[error("Journal error: {0}")]
    JournalError(#[from] journal::JournalError),
    
    /// Error related to observability operations
    #[error("Observability error: {0}")]
    ObservabilityError(String),
}

/// Command factory for creating command registries
mod factory;
pub use factory::{CommandRegistryFactory, create_command_registry, create_command_registry_with_plugin};

/// Adapter module
pub mod adapter;

#[cfg(test)]
mod tests;

/// Register the command system as a plugin
///
/// Creates a command registry and registers it with the plugin registry
///
/// # Arguments
///
/// * `registry` - The plugin registry to register the plugin with
///
/// # Returns
///
/// A Result containing the plugin ID or an error
pub async fn register_plugin<T>(
    registry: &mut T,
) -> std::result::Result<String, Box<dyn std::error::Error>> 
where
    T: squirrel_interfaces::plugins::PluginRegistry + ?Sized
{
    use crate::factory::create_command_registry;
    
    // Create the command registry
    let cmd_registry = match create_command_registry() {
        Ok(registry) => registry,
        Err(e) => return Err(format!("Failed to create command registry: {}", e).into()),
    };
    
    // Create the plugin adapter directly as a Plugin trait object
    let plugin = Arc::new(adapter::plugins::CommandsPluginAdapter::new(cmd_registry));
    
    // Initialize the plugin
    plugin.initialize().await?;
    
    // Register the plugin with the registry
    let plugin_id = plugin.metadata().id.clone();
    registry.register_plugin(plugin).await?;
    
    Ok(plugin_id)
} 