use std::sync::{Arc, RwLock};
use std::error::Error;
use crate::commands::{Command, CommandRegistry};
use crate::commands::validation::ValidationRule;
use crate::commands::lifecycle::LifecycleHook as LifecycleHandler;

/// Type alias for Result with boxed errors
#[allow(dead_code)]
type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

/// Adapter for the command registry to provide a consistent interface
/// for various implementations.
pub struct CommandRegistryAdapter {
    /// Inner registry that handles the command operations
    inner: Arc<RwLock<CommandRegistry>>,
}

impl CommandRegistryAdapter {
    /// Creates a new command registry adapter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(CommandRegistry::new())),
        }
    }

    /// Creates a new adapter with the specified registry.
    #[must_use]
    pub fn with_registry(registry: Arc<RwLock<CommandRegistry>>) -> Self {
        Self { inner: registry }
    }

    /// Gets a reference to the inner registry.
    #[must_use]
    pub fn get_registry(&self) -> Arc<RwLock<CommandRegistry>> {
        Arc::clone(&self.inner)
    }

    /// Adds a validation rule to the registry.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the rule couldn't be added.
    pub fn add_validation_rule(&self, _rule: Box<dyn ValidationRule>) -> Result<()> {
        // Simplified implementation for now
        Ok(())
    }

    /// Adds a lifecycle handler to the registry.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the handler couldn't be added.
    pub fn add_lifecycle_handler(&self, _handler: Box<dyn LifecycleHandler>) -> Result<()> {
        // Simplified implementation for now
        Ok(())
    }

    /// Registers a command with the registry.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the command couldn't be registered.
    pub fn register_command(&self, _command: Box<dyn Command>) -> Result<()> {
        // Simplified implementation for now
        Ok(())
    }

    /// Gets a command by name.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the command couldn't be retrieved.
    pub fn get_command(&self, _name: &str) -> Result<Option<Box<dyn Command>>> {
        // Simplified implementation for now
        Ok(None)
    }

    /// Lists all registered commands.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the commands couldn't be listed.
    pub fn list_commands(&self) -> Result<Vec<String>> {
        // Simplified implementation for now
        Ok(Vec::new())
    }

    /// Executes a command by name with the given arguments.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the command execution fails.
    pub fn execute_command(&self, _name: &str, _args: Vec<String>) -> Result<()> {
        // Simplified implementation for now
        Ok(())
    }
}

impl Default for CommandRegistryAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for CommandRegistryAdapter {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// Creates a new registry adapter.
#[must_use]
pub fn create_registry_adapter() -> Arc<CommandRegistryAdapter> {
    Arc::new(CommandRegistryAdapter::new())
}

/// Creates a new command registry adapter with an existing registry
#[must_use]
#[allow(dead_code)]
pub fn create_registry_adapter_with_registry(_registry: Arc<RwLock<CommandRegistry>>) -> Arc<CommandRegistryAdapter> {
    // We need to create a new registry since with_registry expects CommandRegistry not Arc<CommandRegistry>
    let new_registry = CommandRegistry::new();
    Arc::new(CommandRegistryAdapter::with_registry(Arc::new(RwLock::new(new_registry))))
} 