/// Command handling functionality for the application
///
/// This module provides the core command processing components that 
/// enable the application to handle incoming commands, execute them,
/// and process them through pre and post hooks.
use crate::error::{Result, CoreError};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::fmt::Debug;
use async_trait::async_trait;
use squirrel_commands::Command;


/// Adapter module for command handling
pub mod adapter;
pub use adapter::{CommandHandlerAdapter, create_handler_adapter, create_handler_adapter_with_handler};

/// A command handler that processes commands
#[derive(Debug)]
pub struct CommandHandler {
    /// Map of command types to their processors
    handlers: Arc<RwLock<HashMap<String, Box<dyn CommandProcessor>>>>,
}

impl CommandHandler {
    /// Creates a new `CommandHandler` with default settings
    #[must_use]
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Creates a new `CommandHandler` with dependencies
    #[must_use]
    pub fn with_dependencies(handlers: Arc<RwLock<HashMap<String, Box<dyn CommandProcessor>>>>) -> Self {
        Self { handlers }
    }
    
    /// Registers a command processor for a specific command type
    /// 
    /// # Errors
    /// 
    /// Returns an error if the registration fails
    pub async fn register(&self, command_type: String, processor: Box<dyn CommandProcessor>) -> Result<()> {
        let mut handlers = self.handlers.write().await;
        handlers.insert(command_type, processor);
        Ok(())
    }
    
    /// Handles a command by routing it to the appropriate processor
    /// 
    /// # Errors
    /// 
    /// Returns an error if the command handling fails
    pub async fn handle(&self, command: &dyn Command) -> Result<()> {
        let handlers = self.handlers.read().await;
        let command_name = command.name();
        if let Some(processor) = handlers.get(command_name) {
            processor.process(command).await
        } else {
            Err(CoreError::Command(format!("Command not found: {command_name}")))
        }
    }
}

/// Default implementation
impl Default for CommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Processes a command and returns a result
#[async_trait]
pub trait CommandProcessor: Send + Sync + Debug {
    /// Process a command asynchronously
    /// 
    /// # Errors
    /// 
    /// Returns an error if the command processing fails
    async fn process(&self, command: &dyn Command) -> Result<()>;
}

/// A default command processor that logs commands but doesn't do anything else
#[derive(Debug)]
pub struct DefaultCommandProcessor;

impl DefaultCommandProcessor {
    /// Create a new default command processor
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultCommandProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CommandProcessor for DefaultCommandProcessor {
    async fn process(&self, _command: &dyn Command) -> Result<()> {
        // Implementation left empty for documentation purposes
        Ok(())
    }
}

/// A trait for command hooks that can be executed before or after command processing
#[async_trait]
pub trait CommandHook: Send + Sync + Debug {
    /// Execute the hook
    /// 
    /// # Errors
    /// 
    /// Returns an error if the hook execution fails
    async fn execute(&self, command: &dyn Command) -> Result<()>;
}

/// A command processor with pre and post hooks
#[derive(Debug)]
pub struct HookedCommandProcessor {
    /// The underlying processor
    processor: Box<dyn CommandProcessor>,
    /// Hooks to execute before processing
    pre_hooks: Vec<Box<dyn CommandHook>>,
    /// Hooks to execute after processing
    post_hooks: Vec<Box<dyn CommandHook>>,
}

impl HookedCommandProcessor {
    /// Creates a new `HookedCommandProcessor`
    #[must_use]
    pub fn new(processor: Box<dyn CommandProcessor>) -> Self {
        Self {
            processor,
            pre_hooks: Vec::new(),
            post_hooks: Vec::new(),
        }
    }
    
    /// Adds a pre-hook to the processor
    pub fn add_pre_hook(&mut self, hook: Box<dyn CommandHook>) {
        self.pre_hooks.push(hook);
    }
    
    /// Adds a post-hook to the processor
    pub fn add_post_hook(&mut self, hook: Box<dyn CommandHook>) {
        self.post_hooks.push(hook);
    }
    
    /// Execute all pre-hooks
    /// 
    /// # Errors
    /// 
    /// Returns an error if any pre-hook execution fails
    pub async fn execute_pre_hooks(&self, command: &dyn Command) -> Result<()> {
        for hook in &self.pre_hooks {
            hook.execute(command).await?;
        }
        Ok(())
    }
    
    /// Execute all post-hooks
    /// 
    /// # Errors
    /// 
    /// Returns an error if any post-hook execution fails
    pub async fn execute_post_hooks(&self, command: &dyn Command) -> Result<()> {
        for hook in &self.post_hooks {
            hook.execute(command).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl CommandProcessor for HookedCommandProcessor {
    async fn process(&self, command: &dyn Command) -> Result<()> {
        self.execute_pre_hooks(command).await?;
        let result = self.processor.process(command).await;
        if result.is_ok() {
            self.execute_post_hooks(command).await?;
        }
        result
    }
}

/// Provides pre and post processing hooks for commands
///
/// Allows for registering processors that will run before
/// and after command execution, enabling cross-cutting concerns like
/// logging, validation, and side effects.
#[derive(Debug)]
pub struct CommandHookImpl {
    /// Processors that run before command execution
    pre_hooks: Arc<RwLock<Vec<Box<dyn CommandProcessor>>>>,
    /// Processors that run after command execution
    post_hooks: Arc<RwLock<Vec<Box<dyn CommandProcessor>>>>,
}

impl CommandHookImpl {
    /// Creates a new command hook with empty pre and post hook lists
    ///
    /// # Returns
    /// A new instance
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
    pub async fn execute_pre_hooks(&self, command: &dyn Command) -> Result<()> {
        let pre_hooks = self.pre_hooks.read().await;
        for hook in pre_hooks.iter() {
            hook.process(command).await?;
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
    pub async fn execute_post_hooks(&self, command: &dyn Command) -> Result<()> {
        let post_hooks = self.post_hooks.read().await;
        for hook in post_hooks.iter() {
            hook.process(command).await?;
        }
        Ok(())
    }
}

impl Default for CommandHookImpl {
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
    pub fn create_adapter_with_handler(_handler: &Arc<CommandHandler>) -> Arc<CommandHandlerAdapter> {
        Arc::new(CommandHandlerAdapter::new())
    }
} 