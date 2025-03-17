use crate::error::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;

use thiserror::Error;

/// Errors that can occur during command operations
///
/// This enum represents the various error conditions that can arise
/// during command registration, retrieval, and execution.
#[derive(Error, Debug)]
pub enum CommandError {
    /// Command not found in the registry
    #[error("Command not found: {0}")]
    NotFound(String),
    
    /// Command is invalid (malformed or contains errors)
    #[error("Invalid command: {0}")]
    Invalid(String),
    
    /// Command execution failed with the given error message
    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),
    
    /// Command not found during execution (duplicate of NotFound for compatibility)
    #[error("Command not found: {0}")]
    CommandNotFound(String),
}

/// A command that can be registered and executed in the system
///
/// Commands represent actions that can be performed by the monitoring system.
/// Each command has a name, description, and execution logic.
#[async_trait]
pub trait Command: Send + Sync + Debug {
    /// Returns the name of the command
    ///
    /// The name is used to identify and invoke the command.
    fn name(&self) -> &str;
    
    /// Returns a human-readable description of the command's purpose
    ///
    /// The description should explain what the command does and how to use it.
    fn description(&self) -> &str;
    
    /// Executes the command with the provided arguments
    ///
    /// # Arguments
    /// * `args` - Vector of string arguments to pass to the command
    ///
    /// # Returns
    /// * `Result<()>` - Success or error result from command execution
    async fn execute(&self, args: Vec<String>) -> Result<()>;
    
    /// Creates a boxed clone of this command
    ///
    /// This method is required to support cloning of trait objects.
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
    /// Creates a new empty command registry
    ///
    /// # Returns
    /// A new CommandRegistry instance with no registered commands
    pub fn new() -> Self {
        Self {
            commands: RwLock::new(HashMap::new()),
        }
    }

    /// Registers a command in the registry
    ///
    /// # Arguments
    /// * `command` - The command to register
    ///
    /// # Returns
    /// * `Result<()>` - Success or an error if registration failed
    pub async fn register_command(&self, command: Box<dyn Command + Send + Sync>) -> Result<()> {
        let mut commands = self.commands.write().await;
        commands.insert(command.name().to_string(), command);
        Ok(())
    }

    /// Retrieves a command by name
    ///
    /// # Arguments
    /// * `name` - The name of the command to retrieve
    ///
    /// # Returns
    /// * `Result<Box<dyn Command + Send + Sync>>` - The command if found, or an error if not found
    ///
    /// # Errors
    /// Returns CommandError::NotFound if the command doesn't exist
    pub async fn get_command(&self, name: &str) -> Result<Box<dyn Command + Send + Sync>> {
        let commands = self.commands.read().await;
        commands.get(name)
            .cloned()
            .ok_or_else(|| CommandError::NotFound(name.to_string()).into())
    }

    /// Lists all registered command names
    ///
    /// # Returns
    /// * `Result<Vec<String>>` - A list of all registered command names
    pub async fn list_commands(&self) -> Result<Vec<String>> {
        let commands = self.commands.read().await;
        Ok(commands.keys().cloned().collect())
    }

    /// Handles the execution of a command by name
    ///
    /// # Arguments
    /// * `name` - The name of the command to execute
    /// * `args` - Arguments to pass to the command
    ///
    /// # Returns
    /// * `Result<()>` - Success or an error if execution failed
    ///
    /// # Errors
    /// Returns CommandError::NotFound if the command doesn't exist
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

/// Default implementation of the command manager
///
/// Provides a concrete implementation of the CommandManager trait.
#[derive(Debug)]
pub struct DefaultCommandManager {
    registry: Arc<CommandRegistry>,
}

impl DefaultCommandManager {
    /// Creates a new command manager with an empty registry
    ///
    /// # Returns
    /// A new DefaultCommandManager instance
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

/// Manages commands in the monitoring system
///
/// This trait defines the operations for registering, retrieving, and executing commands
/// in a thread-safe manner.
#[async_trait]
pub trait CommandManager: Send + Sync {
    /// Registers a command with the command manager
    ///
    /// # Arguments
    /// * `command` - The command to register
    ///
    /// # Returns
    /// * `Result<()>` - Success or an error if registration failed
    async fn register_command(&self, command: Box<dyn Command + Send + Sync>) -> Result<()>;
    
    /// Unregisters a command from the command manager
    ///
    /// # Arguments
    /// * `name` - The name of the command to unregister
    ///
    /// # Returns
    /// * `Result<()>` - Success or an error if unregistration failed
    async fn unregister_command(&self, name: &str) -> Result<()>;
    
    /// Executes a command by name with the given arguments
    ///
    /// # Arguments
    /// * `name` - The name of the command to execute
    /// * `args` - Arguments to pass to the command
    ///
    /// # Returns
    /// * `Result<()>` - Success or an error if execution failed
    async fn execute_command(&self, name: &str, args: Vec<String>) -> Result<()>;
    
    /// Retrieves a command by name
    ///
    /// # Arguments
    /// * `name` - The name of the command to retrieve
    ///
    /// # Returns
    /// * `Option<Box<dyn Command + Send + Sync>>` - The command if found, or None if not
    async fn get_command(&self, name: &str) -> Option<Box<dyn Command + Send + Sync>>;
    
    /// Retrieves all registered commands
    ///
    /// # Returns
    /// * `Vec<Box<dyn Command + Send + Sync>>` - A list of all registered commands
    async fn get_commands(&self) -> Vec<Box<dyn Command + Send + Sync>>;
    
    /// Lists all registered command names
    ///
    /// # Returns
    /// * `Result<Vec<String>>` - A list of all registered command names
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

/// Command handler for processing command requests
///
/// This trait defines the interface for components that can handle commands
/// directly, with functionality for executing, registering, and managing commands.
#[async_trait]
pub trait CommandHandler: Send + Sync {
    /// Handles the execution of a command by name with the given arguments
    ///
    /// # Arguments
    /// * `command_name` - The name of the command to execute
    /// * `args` - Arguments to pass to the command
    ///
    /// # Returns
    /// * `Result<()>` - Success or an error if execution failed
    async fn handle(&self, command_name: &str, args: Vec<String>) -> Result<()>;
    
    /// Registers a command with the handler
    ///
    /// # Arguments
    /// * `command` - The command to register
    ///
    /// # Returns
    /// * `Result<()>` - Success or an error if registration failed
    async fn register_command(&self, command: Box<dyn Command + Send + Sync>) -> Result<()>;
    
    /// Unregisters a command from the handler
    ///
    /// # Arguments
    /// * `name` - The name of the command to unregister
    ///
    /// # Returns
    /// * `Result<()>` - Success or an error if unregistration failed
    async fn unregister_command(&self, name: &str) -> Result<()>;
    
    /// Retrieves a command by name
    ///
    /// # Arguments
    /// * `name` - The name of the command to retrieve
    ///
    /// # Returns
    /// * `Option<Box<dyn Command + Send + Sync>>` - The command if found, or None if not
    async fn get_command(&self, name: &str) -> Option<Box<dyn Command + Send + Sync>>;
    
    /// Retrieves all registered commands
    ///
    /// # Returns
    /// * `Vec<Box<dyn Command + Send + Sync>>` - A list of all registered commands
    async fn get_commands(&self) -> Vec<Box<dyn Command + Send + Sync>>;
    
    /// Lists all registered command names
    ///
    /// # Returns
    /// * `Result<Vec<String>>` - A list of all registered command names
    async fn get_command_names(&self) -> Result<Vec<String>>;
}

/// Default implementation of the command handler
///
/// Provides direct command execution and management functionality.
#[derive(Debug)]
pub struct DefaultCommandHandler {
    commands: Arc<RwLock<HashMap<String, Box<dyn Command + Send + Sync>>>>,
}

impl DefaultCommandHandler {
    /// Creates a new command handler with an empty command set
    ///
    /// # Returns
    /// A new DefaultCommandHandler instance
    pub fn new() -> Self {
        Self {
            commands: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Handles the execution of a command by name
    ///
    /// # Arguments
    /// * `command_name` - The name of the command to execute
    /// * `args` - Arguments to pass to the command
    ///
    /// # Returns
    /// * `Result<()>` - Success or an error if execution failed
    ///
    /// # Errors
    /// Returns CommandError::CommandNotFound if the command doesn't exist
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
    /// Registers a command with the handler
    ///
    /// # Arguments
    /// * `command` - The command to register
    ///
    /// # Returns
    /// * `Result<()>` - Success or an error if registration failed
    async fn register_command(&self, command: Box<dyn Command + Send + Sync>) -> Result<()> {
        let name = command.name().to_string();
        let mut commands = self.commands.write().await;
        commands.insert(name, command);
        Ok(())
    }

    /// Unregisters a command from the handler
    ///
    /// # Arguments
    /// * `name` - The name of the command to unregister
    ///
    /// # Returns
    /// * `Result<()>` - Success or an error if unregistration failed
    async fn unregister_command(&self, name: &str) -> Result<()> {
        let mut commands = self.commands.write().await;
        commands.remove(name);
        Ok(())
    }

    /// Handles the execution of a command by name with the given arguments
    ///
    /// # Arguments
    /// * `command_name` - The name of the command to execute
    /// * `args` - Arguments to pass to the command
    ///
    /// # Returns
    /// * `Result<()>` - Success or an error if execution failed
    ///
    /// # Errors
    /// Returns CommandError::CommandNotFound if the command doesn't exist
    async fn handle(&self, command_name: &str, args: Vec<String>) -> Result<()> {
        let commands_guard = self.commands.read().await;
        let command = commands_guard.get(command_name).ok_or_else(|| {
            CommandError::CommandNotFound(command_name.to_string())
        })?;
        
        command.execute(args).await
    }

    /// Retrieves a command by name
    ///
    /// # Arguments
    /// * `name` - The name of the command to retrieve
    ///
    /// # Returns
    /// * `Option<Box<dyn Command + Send + Sync>>` - The command if found, or None if not
    async fn get_command(&self, name: &str) -> Option<Box<dyn Command + Send + Sync>> {
        let commands = self.commands.read().await;
        commands.get(name).map(Command::clone_box)
    }

    /// Retrieves all registered commands
    ///
    /// # Returns
    /// * `Vec<Box<dyn Command + Send + Sync>>` - A list of all registered commands
    async fn get_commands(&self) -> Vec<Box<dyn Command + Send + Sync>> {
        let commands = self.commands.read().await;
        commands.values()
            .map(Command::clone_box)
            .collect()
    }

    /// Lists all registered command names
    ///
    /// # Returns
    /// * `Result<Vec<String>>` - A list of all registered command names
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