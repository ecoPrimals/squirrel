//! Command Registry Adapter implementation
//!
//! This module contains the implementation of the CommandRegistryAdapter, which
//! adapts the core command registry to provide a simplified interface for
//! command operations with proper async and thread safety.

use std::sync::{Arc, Mutex};
use async_trait::async_trait;

use crate::command::MockCommand;
use crate::registry::MockCommandRegistry;
use crate::error::{AdapterError, AdapterResult, to_adapter_error};

/// Adapter for command registry operations
///
/// This adapter provides a simplified interface for working with the command registry,
/// handling thread safety and asynchronous operations appropriately.
///
/// # Thread Safety
///
/// This adapter uses Arc<Mutex<>> to ensure thread safety when accessing the 
/// underlying command registry. Care is taken to ensure proper lock scoping
/// and prevent deadlocks in async contexts.
#[derive(Debug)]
pub struct CommandRegistryAdapter {
    /// The underlying command registry protected by a mutex for thread safety
    registry: Arc<Mutex<MockCommandRegistry>>,
}

impl CommandRegistryAdapter {
    /// Creates a new command registry adapter
    ///
    /// This initializes a new command registry and wraps it in an adapter.
    pub fn new() -> Self {
        let registry = MockCommandRegistry::new();
        Self {
            registry: Arc::new(Mutex::new(registry)),
        }
    }

    /// Creates an adapter with an existing command registry
    ///
    /// This is useful for testing or when you need to share a registry.
    pub fn with_registry(registry: MockCommandRegistry) -> Self {
        Self {
            registry: Arc::new(Mutex::new(registry)),
        }
    }

    /// Checks if the adapter is initialized
    ///
    /// All adapter instances created with the constructors are considered initialized.
    pub fn is_initialized(&self) -> bool {
        true
    }
    
    /// Registers a command with the registry
    ///
    /// # Arguments
    ///
    /// * `command` - The command to register, wrapped in Arc
    ///
    /// # Returns
    ///
    /// * `Ok(())` if registration succeeded
    /// * `Err(AdapterError)` if registration failed
    pub fn register_command(&self, command: Arc<dyn MockCommand + Send + Sync>) -> AdapterResult<()> {
        let cmd_name = command.name().to_string();
        
        // Acquire lock in a limited scope to avoid holding it across await points
        let mut registry = self.registry.lock().map_err(|e| to_adapter_error(e))?;
        
        match registry.register(&cmd_name, command) {
            Ok(()) => Ok(()),
            Err(_) => Err(AdapterError::Other(format!("Failed to register command '{}'", cmd_name)))
        }
    }
    
    /// Executes a command asynchronously
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
    pub async fn execute(&self, command: &str, args: Vec<String>) -> AdapterResult<String> {
        // Acquire lock in a limited scope to avoid holding it across await points
        let registry = self.registry.lock().map_err(|e| to_adapter_error(e))?;
        
        match registry.execute(command, args) {
            Ok(result) => Ok(result),
            Err(_) => Err(AdapterError::NotFound(command.to_string()))
        }
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
    pub async fn get_help(&self, command: &str) -> AdapterResult<String> {
        // Acquire lock in a limited scope
        let registry = self.registry.lock().map_err(|e| to_adapter_error(e))?;
        
        match registry.get_help(command) {
            Ok(help) => Ok(help),
            Err(_) => Err(AdapterError::NotFound(command.to_string()))
        }
    }
    
    /// Lists all registered commands
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` containing command names
    /// * `Err(AdapterError)` if listing failed
    pub async fn list_commands(&self) -> AdapterResult<Vec<String>> {
        // Acquire lock in a limited scope
        let registry = self.registry.lock().map_err(|e| to_adapter_error(e))?;
        
        match registry.list_commands() {
            Ok(commands) => Ok(commands),
            Err(_) => Err(AdapterError::Other("Failed to list commands".to_string()))
        }
    }
}

/// MockAdapter trait for abstract adapter interfaces
///
/// This trait defines the basic operations all command adapters should support.
#[async_trait]
pub trait MockAdapter: Send + Sync {
    /// Executes a command with the given arguments
    async fn execute(&self, command: &str, args: Vec<String>) -> AdapterResult<String>;
    
    /// Gets help information for a command
    async fn get_help(&self, command: &str) -> AdapterResult<String>;
}

#[async_trait]
impl MockAdapter for CommandRegistryAdapter {
    async fn execute(&self, command: &str, args: Vec<String>) -> AdapterResult<String> {
        self.execute(command, args).await
    }
    
    async fn get_help(&self, command: &str) -> AdapterResult<String> {
        self.get_help(command).await
    }
}

impl Default for CommandRegistryAdapter {
    fn default() -> Self {
        Self::new()
    }
} 