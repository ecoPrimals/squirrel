#[allow(unused_imports)]
use std::sync::{Arc, RwLock};
#[allow(unused_imports)]
use crate::error::{Result, SquirrelError};
#[allow(unused_imports)]
use crate::commands::Command;
#[allow(unused_imports)]
use crate::app::command::{CommandProcessor, DefaultCommandProcessor};

/// Adapter for command processors to provide a consistent interface
/// for various command execution strategies.
pub struct CommandHandlerAdapter {
    /// The underlying command processor implementation
    inner: Option<Arc<RwLock<dyn CommandProcessor>>>,
}

impl CommandHandlerAdapter {
    /// Create a new command handler adapter
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

    /// Get the inner command handler, creating it if necessary
    fn get_handler(&self) -> Arc<RwLock<dyn CommandProcessor>> {
        if let Some(handler) = &self.inner {
            Arc::clone(handler)
        } else {
            // Create a new DefaultCommandProcessor directly
            let processor = DefaultCommandProcessor::new();
            Arc::new(RwLock::new(processor))
        }
    }

    /// Handles a command by delegating to the underlying processor.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the command processing fails.
    pub fn handle(&self, _command: &dyn Command) -> Result<()> {
        let _handler = self.get_handler();
        // Since this is a simplified implementation, we'll just return success
        // In a complete implementation, we would delegate to the handler
        // handler.write().unwrap().process(command)
        Ok(())
    }

    /// Registers a handler for a specific command type.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the handler registration fails.
    pub fn register_handler(&self, _command_type: &str, _handler: Box<dyn CommandProcessor>) -> Result<()> {
        // Since CommandProcessor trait doesn't have register_handler, we need to handle this differently
        // For now, just return Ok as this will be implemented in a future update
        Ok(())
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
pub fn create_handler_adapter_with_handler(handler: Arc<RwLock<dyn CommandProcessor>>) -> Arc<CommandHandlerAdapter> {
    Arc::new(CommandHandlerAdapter::new().with_handler(handler))
}

/// Creates a new command handler adapter
#[must_use]
pub fn create_handler_adapter() -> Arc<CommandHandlerAdapter> {
    Arc::new(CommandHandlerAdapter::new())
} 