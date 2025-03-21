use std::sync::{Arc, RwLock};
use std::error::Error;
use crate::commands::{Command, CommandRegistry};
use crate::commands::validation::ValidationRule;
use crate::commands::lifecycle::LifecycleHook as LifecycleHandler;
use thiserror::Error;
use crate::history::{HistoryEntry, HistoryReplay};
use crate::{CommandError, CommandResult};

/// Type alias for Result with boxed errors
#[allow(dead_code)]
type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

/// Errors specific to command registry adapter operations
#[derive(Debug, Error)]
pub enum CommandRegistryAdapterError {
    /// Registry is not initialized
    #[error("Command registry not initialized")]
    NotInitialized,
    
    /// Registry is already initialized
    #[error("Command registry already initialized")]
    AlreadyInitialized,
    
    /// The operation failed
    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

/// Adapter for the command registry to provide a consistent interface
/// for various implementations.
pub struct CommandRegistryAdapter {
    /// Inner registry that handles the command operations
    inner: Option<Arc<RwLock<CommandRegistry>>>,
}

impl CommandRegistryAdapter {
    /// Creates a new command registry adapter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: None,
        }
    }

    /// Creates a new adapter with the specified registry.
    #[must_use]
    pub fn with_registry(registry: Arc<RwLock<CommandRegistry>>) -> Self {
        Self { inner: Some(registry) }
    }

    /// Initializes the adapter with a default registry
    /// 
    /// # Errors
    /// 
    /// Returns `CommandRegistryAdapterError::AlreadyInitialized` if the adapter
    /// is already initialized.
    pub fn initialize(&mut self) -> Result<()> {
        if self.inner.is_some() {
            return Err(Box::new(CommandRegistryAdapterError::AlreadyInitialized));
        }
        
        self.inner = Some(Arc::new(RwLock::new(CommandRegistry::new())));
        Ok(())
    }

    /// Gets a reference to the inner registry.
    /// 
    /// # Errors
    /// 
    /// Returns `CommandRegistryAdapterError::NotInitialized` if the adapter
    /// is not initialized.
    pub fn get_registry(&self) -> Result<Arc<RwLock<CommandRegistry>>> {
        self.inner.clone().ok_or_else(|| Box::new(CommandRegistryAdapterError::NotInitialized))
    }

    /// Adds a validation rule to the registry.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the rule couldn't be added or if the adapter is not initialized.
    pub fn add_validation_rule(&self, _rule: Box<dyn ValidationRule>) -> Result<()> {
        if self.inner.is_none() {
            return Err(Box::new(CommandRegistryAdapterError::NotInitialized));
        }
        // Simplified implementation for now
        Ok(())
    }

    /// Adds a lifecycle handler to the registry.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the handler couldn't be added or if the adapter is not initialized.
    pub fn add_lifecycle_handler(&self, _handler: Box<dyn LifecycleHandler>) -> Result<()> {
        if self.inner.is_none() {
            return Err(Box::new(CommandRegistryAdapterError::NotInitialized));
        }
        // Simplified implementation for now
        Ok(())
    }

    /// Registers a command with the registry.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the command couldn't be registered or if the adapter is not initialized.
    pub fn register_command(&self, _command: Box<dyn Command>) -> Result<()> {
        if self.inner.is_none() {
            return Err(Box::new(CommandRegistryAdapterError::NotInitialized));
        }
        // Simplified implementation for now
        Ok(())
    }

    /// Gets a command by name.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the command couldn't be retrieved or if the adapter is not initialized.
    pub fn get_command(&self, _name: &str) -> Result<Option<Box<dyn Command>>> {
        if self.inner.is_none() {
            return Err(Box::new(CommandRegistryAdapterError::NotInitialized));
        }
        // Simplified implementation for now
        Ok(None)
    }

    /// Lists all registered commands.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the commands couldn't be listed or if the adapter is not initialized.
    pub fn list_commands(&self) -> Result<Vec<String>> {
        if self.inner.is_none() {
            return Err(Box::new(CommandRegistryAdapterError::NotInitialized));
        }
        // Simplified implementation for now
        Ok(Vec::new())
    }

    /// Executes a command by name with the given arguments.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the command execution fails or if the adapter is not initialized.
    pub fn execute_command(&self, _name: &str, _args: Vec<String>) -> Result<()> {
        if self.inner.is_none() {
            return Err(Box::new(CommandRegistryAdapterError::NotInitialized));
        }
        // Simplified implementation for now
        Ok(())
    }
    
    /// Checks if the adapter is initialized
    #[must_use]
    pub fn is_initialized(&self) -> bool {
        self.inner.is_some()
    }

    /// Gets the history manager, creating it if needed
    fn get_history(&self) -> CommandResult<Arc<crate::history::CommandHistory>> {
        let registry_lock = self.inner.as_ref().ok_or_else(|| {
            CommandError::RegistryError("Command registry not initialized".to_string())
        })?;
        
        // Check if we have a history manager in resources
        let resource_key = "command_history";
        let history = match registry_lock.read().map_err(|e| {
            CommandError::RegistryError(format!("Failed to acquire lock on registry: {}", e))
        })?.get_resource(resource_key) {
            Ok(Some(history)) => {
                // Try to downcast to CommandHistory
                match history.downcast_ref::<Arc<crate::history::CommandHistory>>() {
                    Some(history) => Arc::clone(history),
                    None => {
                        // Create a new history manager
                        debug!("CommandRegistryAdapter: Creating new history manager");
                        let history = Arc::new(crate::history::CommandHistory::new()?);
                        
                        // Store it in resources
                        registry_lock.write().map_err(|e| {
                            CommandError::RegistryError(format!("Failed to set resource in registry: {}", e))
                        })?.set_resource(resource_key, Box::new(Arc::clone(&history)))?;
                        
                        history
                    }
                }
            },
            _ => {
                // Create a new history manager
                debug!("CommandRegistryAdapter: Creating new history manager");
                let history = Arc::new(crate::history::CommandHistory::new()?);
                
                // Store it in resources
                registry_lock.write().map_err(|e| {
                    CommandError::RegistryError(format!("Failed to set resource in registry: {}", e))
                })?.set_resource(resource_key, Box::new(Arc::clone(&history)))?;
                
                history
            }
        };
        
        Ok(history)
    }
    
    /// Records a command execution in history
    /// 
    /// This should be called after a command is executed to track it in history
    pub fn record_execution(
        &self,
        command: String,
        args: Vec<String>,
        success: bool,
        error_message: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> CommandResult<()> {
        // Get history manager
        let history = self.get_history()?;
        
        // Add entry
        history.add(command, args, success, error_message, metadata).map_err(|e| {
            CommandError::ResourceError(format!("Failed to record command execution: {}", e))
        })
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
            inner: self.inner.clone(),
        }
    }
}

/// Creates a new registry adapter.
#[must_use]
pub fn create_registry_adapter() -> Arc<CommandRegistryAdapter> {
    Arc::new(CommandRegistryAdapter::new())
}

/// Creates a new registry adapter and initializes it
/// 
/// # Errors
/// 
/// Returns an error if initialization fails.
pub fn create_initialized_registry_adapter() -> Result<Arc<CommandRegistryAdapter>> {
    let mut adapter = CommandRegistryAdapter::new();
    adapter.initialize()?;
    Ok(Arc::new(adapter))
}

/// Creates a new command registry adapter with an existing registry
#[must_use]
pub fn create_registry_adapter_with_registry(registry: Arc<RwLock<CommandRegistry>>) -> Arc<CommandRegistryAdapter> {
    Arc::new(CommandRegistryAdapter::with_registry(registry))
}

impl HistoryReplay for CommandRegistryAdapter {
    fn replay(&self, entry: &HistoryEntry) -> CommandResult<String> {
        debug!("CommandRegistryAdapter: Replaying command: {}", entry.command);
        
        let registry_lock = self.inner.as_ref().ok_or_else(|| {
            CommandError::RegistryError("Command registry not initialized".to_string())
        })?;
        
        // Execute the command with the same arguments
        registry_lock.execute(&entry.command, &entry.args)
    }
    
    fn replay_last(&self) -> CommandResult<String> {
        debug!("CommandRegistryAdapter: Replaying last command");
        
        // Get history manager
        let history = self.get_history()?;
        
        // Get the last entry
        let entries = history.get_last(1).map_err(|e| {
            CommandError::ExecutionError(format!("Failed to get last command: {}", e))
        })?;
        
        if entries.is_empty() {
            return Err(CommandError::ExecutionError("No command history available to replay".to_string()));
        }
        
        // Replay the command
        self.replay(&entries[0])
    }
    
    fn replay_last_command(&self, command: &str) -> CommandResult<String> {
        debug!("CommandRegistryAdapter: Replaying last instance of command: {}", command);
        
        // Get history manager
        let history = self.get_history()?;
        
        // Get the last entry for the command
        let entry = history.get_last_for_command(command).map_err(|e| {
            CommandError::ExecutionError(format!("Failed to get last command '{}': {}", command, e))
        })?;
        
        match entry {
            Some(entry) => self.replay(&entry),
            None => Err(CommandError::ExecutionError(format!("No history found for command '{}'", command))),
        }
    }
} 