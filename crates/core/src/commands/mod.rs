use crate::error::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Command not found: {0}")]
    NotFound(String),
    #[error("Invalid command: {0}")]
    Invalid(String),
    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Command not found: {0}")]
    CommandNotFound(String),
}

#[async_trait]
pub trait Command: Send + Sync + Debug {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    async fn execute(&self, args: Vec<String>) -> Result<()>;
    fn clone_box(&self) -> Box<dyn Command + Send + Sync>;
}

impl Clone for Box<dyn Command + Send + Sync> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[async_trait]
impl Command for Box<dyn Command + Send + Sync> {
    fn name(&self) -> &str {
        (**self).name()
    }
    
    fn description(&self) -> &str {
        (**self).description()
    }
    
    async fn execute(&self, args: Vec<String>) -> Result<()> {
        (**self).execute(args).await
    }
    
    fn clone_box(&self) -> Box<dyn Command + Send + Sync> {
        (**self).clone_box()
    }
}

/// A registry for storing and managing commands
#[derive(Debug)]
pub struct CommandRegistry {
    commands: RwLock<HashMap<String, Box<dyn Command + Send + Sync>>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register_command(&self, command: Box<dyn Command + Send + Sync>) -> Result<()> {
        let name = command.name().to_string();
        let mut commands = self.commands.write().await;
        commands.insert(name, command);
        Ok(())
    }

    pub async fn get_command(&self, name: &str) -> Result<Box<dyn Command + Send + Sync>> {
        let commands = self.commands.read().await;
        if let Some(cmd) = commands.get(name) {
            Ok(cmd.clone_box())
        } else {
            Err(CommandError::NotFound(name.to_string()).into())
        }
    }

    pub async fn list_commands(&self) -> Result<Vec<String>> {
        let commands = self.commands.read().await;
        Ok(commands.keys().cloned().collect())
    }

    pub async fn handle(&self, name: &str, args: Vec<String>) -> Result<()> {
        let command = self.get_command(name).await?;
        command.execute(args).await
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct DefaultCommandManager {
    registry: Arc<CommandRegistry>,
}

impl DefaultCommandManager {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(CommandRegistry::new()),
        }
    }
}

impl Default for DefaultCommandManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
pub trait CommandManager: Send + Sync {
    async fn register_command(&self, command: Box<dyn Command + Send + Sync>) -> Result<()>;
    async fn unregister_command(&self, name: &str) -> Result<()>;
    async fn execute_command(&self, name: &str, args: Vec<String>) -> Result<()>;
    async fn get_command(&self, name: &str) -> Option<Box<dyn Command + Send + Sync>>;
    async fn get_commands(&self) -> Vec<Box<dyn Command + Send + Sync>>;
    async fn get_command_names(&self) -> Result<Vec<String>>;
}

#[async_trait]
impl CommandManager for DefaultCommandManager {
    async fn register_command(&self, command: Box<dyn Command + Send + Sync>) -> Result<()> {
        self.registry.register_command(command).await
    }

    async fn unregister_command(&self, name: &str) -> Result<()> {
        let mut commands = self.registry.commands.write().await;
        commands.remove(name);
        Ok(())
    }

    async fn execute_command(&self, name: &str, args: Vec<String>) -> Result<()> {
        self.registry.handle(name, args).await
    }

    async fn get_command(&self, name: &str) -> Option<Box<dyn Command + Send + Sync>> {
        self.registry.get_command(name).await.ok()
    }

    async fn get_commands(&self) -> Vec<Box<dyn Command + Send + Sync>> {
        let commands = self.registry.commands.read().await;
        commands.values()
            .map(Command::clone_box)
            .collect()
    }

    async fn get_command_names(&self) -> Result<Vec<String>> {
        self.registry.list_commands().await
    }
}

#[async_trait]
pub trait CommandHandler: Send + Sync {
    async fn handle(&self, command_name: &str, args: Vec<String>) -> Result<()>;
    async fn register_command(&self, command: Box<dyn Command + Send + Sync>) -> Result<()>;
    async fn unregister_command(&self, name: &str) -> Result<()>;
    async fn get_command(&self, name: &str) -> Option<Box<dyn Command + Send + Sync>>;
    async fn get_commands(&self) -> Vec<Box<dyn Command + Send + Sync>>;
    async fn get_command_names(&self) -> Result<Vec<String>>;
}

#[derive(Debug)]
pub struct DefaultCommandHandler {
    commands: Arc<RwLock<HashMap<String, Box<dyn Command + Send + Sync>>>>,
}

impl DefaultCommandHandler {
    pub fn new() -> Self {
        Self {
            commands: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn handle(&self, command_name: &str, args: Vec<String>) -> Result<()> {
        let commands_guard = self.commands.read().await;
        let command = commands_guard.get(command_name).ok_or_else(|| {
            CommandError::CommandNotFound(command_name.to_string())
        })?;
        
        command.execute(args).await
    }
}

impl Default for DefaultCommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CommandHandler for DefaultCommandHandler {
    async fn register_command(&self, command: Box<dyn Command + Send + Sync>) -> Result<()> {
        let name = command.name().to_string();
        let mut commands = self.commands.write().await;
        commands.insert(name, command);
        Ok(())
    }

    async fn unregister_command(&self, name: &str) -> Result<()> {
        let mut commands = self.commands.write().await;
        commands.remove(name);
        Ok(())
    }

    async fn handle(&self, command_name: &str, args: Vec<String>) -> Result<()> {
        let commands_guard = self.commands.read().await;
        let command = commands_guard.get(command_name).ok_or_else(|| {
            CommandError::CommandNotFound(command_name.to_string())
        })?;
        
        command.execute(args).await
    }

    async fn get_command(&self, name: &str) -> Option<Box<dyn Command + Send + Sync>> {
        let commands = self.commands.read().await;
        commands.get(name).map(Command::clone_box)
    }

    async fn get_commands(&self) -> Vec<Box<dyn Command + Send + Sync>> {
        let commands = self.commands.read().await;
        commands.values()
            .map(Command::clone_box)
            .collect()
    }

    async fn get_command_names(&self) -> Result<Vec<String>> {
        let commands = self.commands.read().await;
        Ok(commands.keys().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Removing unused imports
    // use std::future::Future;
    // use futures::future::BoxFuture;

    #[derive(Debug)]
    struct TestCommand {
        name: String,
        description: String,
    }

    #[async_trait]
    impl Command for TestCommand {
        async fn execute(&self, _args: Vec<String>) -> Result<()> {
            Ok(())
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            &self.description
        }

        fn clone_box(&self) -> Box<dyn Command + Send + Sync> {
            Box::new(TestCommand {
                name: self.name.clone(),
                description: self.description.clone(),
            })
        }
    }

    #[tokio::test]
    #[ignore] // Temporarily disabled
    async fn test_command_registry() {
        let registry = CommandRegistry::new();

        // Register a test command
        let command = Box::new(TestCommand {
            name: "test".to_string(),
            description: "Test command".to_string(),
        });
        registry.register_command(command).await.unwrap();

        // Get command
        let cmd = registry.get_command("test").await.unwrap();
        assert_eq!(cmd.name(), "test");
        assert_eq!(cmd.description(), "Test command");

        // List commands
        let commands = registry.list_commands().await.unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], "test");

        // Execute command
        registry.handle("test", vec![]).await.unwrap();
    }
} 