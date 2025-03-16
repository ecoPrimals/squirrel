use crate::error::{Result, SquirrelError};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::future::Future;
use async_trait::async_trait;
use std::fmt::Debug;
use std::pin::Pin;

use serde_json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub command_type: String,
    pub parameters: serde_json::Value,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct CommandHandler {
    handlers: Arc<RwLock<HashMap<String, Box<dyn CommandProcessor>>>>,
}

impl CommandHandler {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
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
            Err(SquirrelError::Command(format!("No handler found for command type: {}", command.command_type)))
        }
    }
}

impl Default for CommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
pub trait CommandProcessor: Debug + Send + Sync {
    fn process(&self, command: &Command) -> Box<dyn Future<Output = Result<()>> + Send + '_>;
}

#[derive(Debug, Clone)]
pub struct CommandHook {
    pre_hooks: Arc<RwLock<Vec<Box<dyn CommandProcessor>>>>,
    post_hooks: Arc<RwLock<Vec<Box<dyn CommandProcessor>>>>,
}

impl CommandHook {
    pub fn new() -> Self {
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