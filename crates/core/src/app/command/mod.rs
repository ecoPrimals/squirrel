/// Command handling functionality for the application
///
/// This module provides the core command processing components that 
/// enable the application to handle incoming commands, execute them,
/// and process them through pre and post hooks.
use crate::error::{Result, SquirrelError};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::pin::Pin;
use std::future::Future;
use std::fmt::Debug;
use async_trait::async_trait;

use serde_json;

/// Adapter module for command handling
pub mod adapter;
pub use adapter::{CommandHandlerAdapter, create_handler_adapter, create_handler_adapter_with_handler};

/// A command that can be processed by the system
///
/// Commands represent actions that can be executed by the application.
/// Each command has a type, parameters, and metadata to describe its behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    /// The type identifier for the command
    pub command_type: String,
    /// Parameters that control the command's behavior
    pub parameters: serde_json::Value,
    /// Additional metadata associated with the command
    pub metadata: HashMap<String, String>,
}

/// Handles and dispatches commands to appropriate processors
///
/// The `CommandHandler` manages a registry of command processors and
/// routes incoming commands to the appropriate processor based on the command type.
#[derive(Debug, Clone)]
pub struct CommandHandler {
    /// Map of command types to their processors
    handlers: Arc<RwLock<HashMap<String, Box<dyn CommandProcessor>>>>,
}

impl CommandHandler {
    /// Creates a new command handler with an empty set of handlers
    ///
    /// # Returns
    /// A new `CommandHandler` instance
    #[must_use]
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Creates a new command handler with dependencies
    ///
    /// # Arguments
    /// * `handlers` - Map of command types to their processors
    ///
    /// # Returns
    /// A new `CommandHandler` instance with the provided handlers
    #[must_use]
    pub fn with_dependencies(
        handlers: Arc<RwLock<HashMap<String, Box<dyn CommandProcessor>>>>,
    ) -> Self {
        Self { handlers }
    }

    /// Registers a command processor for the specified command type.
    ///
    /// # Arguments
    /// * `command_type` - The type of command to register a handler for
    /// * `handler` - The processor implementation to handle the command
    ///
    /// # Errors
    /// This function will not produce errors, but returns a Result type for consistency
    /// with the async interface.
    pub async fn register_handler(&self, command_type: &str, handler: Box<dyn CommandProcessor>) -> Result<()> {
        let mut handlers = self.handlers.write().await;
        handlers.insert(command_type.to_string(), handler);
        Ok(())
    }

    /// Handles the given command by dispatching it to the appropriate processor.
    ///
    /// # Arguments
    /// * `command` - The command to process
    ///
    /// # Errors
    /// Returns an error if:
    /// * No handler is registered for the command type
    /// * The command processor encounters an error during processing
    pub async fn handle(&self, command: Command) -> Result<()> {
        let handlers = self.handlers.read().await;
        if let Some(handler) = handlers.get(&command.command_type) {
            let process_future = handler.process(&command);
            Pin::from(process_future).await
        } else {
            Err(SquirrelError::Other(format!("Command not found: {}", command.command_type)))
        }
    }
}

impl Default for CommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CommandProcessor for CommandHandler {
    fn process(&self, command: &Command) -> Box<dyn Future<Output = Result<()>> + Send + '_> {
        Box::new(self.handle(command.clone()))
    }
}

/// Default implementation of `CommandProcessor`
#[derive(Debug)]
pub struct DefaultCommandProcessor;

impl Default for DefaultCommandProcessor {
    fn default() -> Self {
        Self
    }
}

impl DefaultCommandProcessor {
    /// Creates a new `DefaultCommandProcessor`
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl CommandProcessor for DefaultCommandProcessor {
    fn process(&self, _command: &Command) -> Box<dyn Future<Output = Result<()>> + Send + '_> {
        Box::new(async move {
            // Default implementation just returns success
            Ok(())
        })
    }
}

/// Processes commands of a specific type
///
/// Command processors implement the specific logic for handling
/// different types of commands in the system.
#[async_trait]
pub trait CommandProcessor: Debug + Send + Sync {
    /// Processes a command and returns a future with the result
    ///
    /// # Arguments
    /// * `command` - The command to process
    ///
    /// # Returns
    /// A boxed future that will resolve to a Result indicating success or failure
    fn process(&self, command: &Command) -> Box<dyn Future<Output = Result<()>> + Send + '_>;
}

/// Provides pre and post processing hooks for commands
///
/// `CommandHook` allows for registering processors that will run before
/// and after command execution, enabling cross-cutting concerns like
/// logging, validation, and side effects.
#[derive(Debug, Clone)]
pub struct CommandHook {
    /// Processors that run before command execution
    pre_hooks: Arc<RwLock<Vec<Box<dyn CommandProcessor>>>>,
    /// Processors that run after command execution
    post_hooks: Arc<RwLock<Vec<Box<dyn CommandProcessor>>>>,
}

impl CommandHook {
    /// Creates a new command hook with empty pre and post hook lists
    ///
    /// # Returns
    /// A new `CommandHook` instance
    #[must_use] pub fn new() -> Self {
        Self {
            pre_hooks: Arc::new(RwLock::new(Vec::new())),
            post_hooks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Adds a pre-execution hook for commands.
    ///
    /// # Arguments
    /// * `hook` - The command processor to execute before command processing
    ///
    /// # Errors
    /// This function will not produce errors, but returns a Result type for consistency
    /// with the async interface.
    pub async fn add_pre_hook(&self, hook: Box<dyn CommandProcessor>) -> Result<()> {
        let mut pre_hooks = self.pre_hooks.write().await;
        pre_hooks.push(hook);
        Ok(())
    }

    /// Adds a post-execution hook for commands.
    ///
    /// # Arguments
    /// * `hook` - The command processor to execute after command processing
    ///
    /// # Errors
    /// This function will not produce errors, but returns a Result type for consistency
    /// with the async interface.
    pub async fn add_post_hook(&self, hook: Box<dyn CommandProcessor>) -> Result<()> {
        let mut post_hooks = self.post_hooks.write().await;
        post_hooks.push(hook);
        Ok(())
    }

    /// Executes all pre-hooks for the given command.
    ///
    /// # Arguments
    /// * `command` - The command about to be processed
    ///
    /// # Errors
    /// Returns an error if any of the pre-hooks fails during execution.
    pub async fn execute_pre_hooks(&self, command: &Command) -> Result<()> {
        let pre_hooks = self.pre_hooks.read().await;
        for hook in pre_hooks.iter() {
            let process_future = hook.process(command);
            Pin::from(process_future).await?;
        }
        Ok(())
    }

    /// Executes all post-hooks for the given command.
    ///
    /// # Arguments
    /// * `command` - The command that was processed
    ///
    /// # Errors
    /// Returns an error if any of the post-hooks fails during execution.
    pub async fn execute_post_hooks(&self, command: &Command) -> Result<()> {
        let post_hooks = self.post_hooks.read().await;
        for hook in post_hooks.iter() {
            let process_future = hook.process(command);
            Pin::from(process_future).await?;
        }
        Ok(())
    }
}

impl Default for CommandHook {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory for creating command handlers
#[derive(Debug, Default)]
pub struct CommandHandlerFactory;

impl CommandHandlerFactory {
    /// Creates a new command handler factory
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Creates a new command handler
    #[must_use]
    pub fn create(&self) -> Arc<CommandHandler> {
        Arc::new(CommandHandler::new())
    }

    /// Creates a new command handler with dependencies
    #[must_use]
    pub fn create_with_dependencies(
        &self,
        handlers: Arc<RwLock<HashMap<String, Box<dyn CommandProcessor>>>>,
    ) -> Arc<CommandHandler> {
        Arc::new(CommandHandler::with_dependencies(handlers))
    }

    /// Creates a new command handler adapter
    #[must_use]
    pub fn create_adapter(&self) -> Arc<CommandHandlerAdapter> {
        create_handler_adapter()
    }

    /// Creates a new command handler adapter with an existing handler
    #[must_use]
    pub fn create_adapter_with_handler(&self, _handler: Arc<CommandHandler>) -> Arc<CommandHandlerAdapter> {
        Arc::new(CommandHandlerAdapter::new())
    }
} 