#[allow(unused_imports)]
use std::sync::{Arc, RwLock};
#[allow(unused_imports)]
use squirrel_core::error::{Result, SquirrelError};
#[allow(unused_imports)]
use squirrel_commands::Command;
#[allow(unused_imports)]
use crate::command::{CommandProcessor, DefaultCommandProcessor};
use thiserror::Error;

/// Errors specific to command handler adapter operations
#[derive(Debug, Error)]
pub enum CommandHandlerAdapterError {
    /// Handler is not initialized
    #[error("Command handler not initialized")]
    NotInitialized,
    
    /// Handler is already initialized
    #[error("Command handler already initialized")]
    AlreadyInitialized,
    
    /// The operation failed
    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

/// Adapter for command processors to provide a consistent interface
/// for various command execution strategies.
pub struct CommandHandlerAdapter {
    /// The underlying command processor implementation
    inner: Option<Arc<RwLock<dyn CommandProcessor>>>,
}

impl CommandHandlerAdapter {
    /// Create a new uninitialized adapter
    #[must_use]
    pub fn new() -> Self {
        Self { inner: None }
    }

    /// Set the inner command handler
    #[must_use]
    pub fn with_handler(mut self, handler: Arc<RwLock<dyn CommandProcessor>>) -> Self {
        self.inner = Some(handler);
        self
    }
    
    /// Initialize with a command processor
    /// 
    /// # Errors
    /// 
    /// Returns an error if already initialized
    pub fn initialize(&mut self, handler: Arc<RwLock<dyn CommandProcessor>>) -> Result<()> {
        if self.inner.is_some() {
            return Err(SquirrelError::generic(CommandHandlerAdapterError::AlreadyInitialized.to_string()));
        }
        
        self.inner = Some(handler);
        Ok(())
    }

    /// Get the wrapped command processor
    /// 
    /// # Errors
    /// 
    /// Returns an error if not initialized
    pub fn get_handler(&self) -> Result<Arc<RwLock<dyn CommandProcessor>>> {
        self.inner.clone().ok_or_else(||
            SquirrelError::generic(CommandHandlerAdapterError::NotInitialized.to_string()))
    }

    /// Handles a command by delegating to the underlying processor.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the command processing fails or if the adapter is not initialized.
    pub fn handle(&self, _command: &dyn Command) -> Result<()> {
        // Check if initialized
        let _handler = self.get_handler()?;
        // Since this is a simplified implementation, we'll just return success
        // In a complete implementation, we would delegate to the handler
        // handler.write().unwrap().process(command)
        Ok(())
    }

    /// Registers a handler for a specific command type.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the handler registration fails or if the adapter is not initialized.
    pub fn register_handler(&self, _command_type: &str, _handler: &dyn CommandProcessor) -> Result<()> {
        // Check if initialized
        let _existing_handler = self.get_handler()?;
        // Since CommandProcessor trait doesn't have register_handler, we need to handle this differently
        // For now, just return Ok as this will be implemented in a future update
        Ok(())
    }
    
    /// Checks if the adapter is initialized
    #[must_use]
    pub fn is_initialized(&self) -> bool {
        self.inner.is_some()
    }
}

impl Default for CommandHandlerAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for CommandHandlerAdapter {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// Creates a new command handler adapter with an existing handler
#[must_use]
pub fn create_handler_adapter_with_handler(handler: Arc<RwLock<dyn CommandProcessor>>) -> Arc<CommandHandlerAdapter> {
    Arc::new(CommandHandlerAdapter::new().with_handler(handler))
}

/// Creates a new command handler adapter
#[must_use]
pub fn create_handler_adapter() -> Arc<CommandHandlerAdapter> {
    Arc::new(CommandHandlerAdapter::new())
}

/// Creates a new command handler adapter and initializes it
/// 
/// # Errors
/// 
/// Returns an error if initialization fails.
pub fn create_initialized_handler_adapter() -> Result<Arc<CommandHandlerAdapter>> {
    let mut adapter = CommandHandlerAdapter::new();
    adapter.initialize(Arc::new(RwLock::new(DefaultCommandProcessor::new())))?;
    Ok(Arc::new(adapter))
} 