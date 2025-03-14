use crate::core::error::{Error, Result};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

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

    pub async fn register_handler(&self, command_type: &str, handler: Box<dyn CommandProcessor>) -> Result<()> {
        let mut handlers = self.handlers.write().await;
        handlers.insert(command_type.to_string(), handler);
        Ok(())
    }

    pub async fn handle(&self, command: Command) -> Result<()> {
        let handlers = self.handlers.read().await;
        if let Some(handler) = handlers.get(&command.command_type) {
            handler.process(&command).await
        } else {
            Err(Error::Command(format!("No handler found for command type: {}", command.command_type)))
        }
    }
}

#[async_trait::async_trait]
pub trait CommandProcessor: std::fmt::Debug + Send + Sync {
    async fn process(&self, command: &Command) -> Result<()>;
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

    pub async fn add_pre_hook(&self, hook: Box<dyn CommandProcessor>) -> Result<()> {
        let mut hooks = self.pre_hooks.write().await;
        hooks.push(hook);
        Ok(())
    }

    pub async fn add_post_hook(&self, hook: Box<dyn CommandProcessor>) -> Result<()> {
        let mut hooks = self.post_hooks.write().await;
        hooks.push(hook);
        Ok(())
    }

    pub async fn execute_pre_hooks(&self, command: &Command) -> Result<()> {
        let hooks = self.pre_hooks.read().await;
        for hook in hooks.iter() {
            hook.process(command).await?;
        }
        Ok(())
    }

    pub async fn execute_post_hooks(&self, command: &Command) -> Result<()> {
        let hooks = self.post_hooks.read().await;
        for hook in hooks.iter() {
            hook.process(command).await?;
        }
        Ok(())
    }
} 