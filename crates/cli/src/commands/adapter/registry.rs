use std::sync::Arc;
use async_trait::async_trait;
use log::debug;
use commands::{Command, CommandRegistry};
use tokio::sync::Mutex;

use crate::commands::adapter::error::{AdapterError, AdapterResult};
use crate::commands::adapter::CommandAdapterTrait;
use crate::commands::executor::{LockTimer, with_registry};

/// Adapter for the CommandRegistry that provides a simplified interface and handles locks
/// in a safe manner for async operations.
#[derive(Debug)]
pub struct CommandRegistryAdapter {
    registry: Arc<Mutex<CommandRegistry>>,
}

impl CommandRegistryAdapter {
    /// Create a new registry adapter
    pub fn new(registry: Arc<Mutex<CommandRegistry>>) -> Self {
        debug!("Creating new registry adapter");
        Self { registry }
    }
    
    /// Get the underlying registry
    pub fn get_registry(&self) -> Arc<Mutex<CommandRegistry>> {
        self.registry.clone()
    }
    
    /// Register a command in the registry
    pub async fn register_command(&self, command_name: &str, command: Arc<dyn Command>) -> AdapterResult<()> {
        debug!("Registering command: {}", command_name);
        
        let result = with_registry(&self.registry, "register_command", |registry| {
            registry.register(command_name, command)
        }).await;
        
        // Convert the CommandError to AdapterError
        match result {
            Ok(()) => Ok(()),
            Err(err) => Err(AdapterError::from(err)),
        }
    }
    
    /// Check if a command exists in the registry
    pub async fn command_exists(&self, command: &str) -> AdapterResult<bool> {
        debug!("Checking if command exists: {}", command);
        
        let exists = with_registry(&self.registry, "command_exists", |registry| {
            registry.get_command(command).is_ok()
        }).await;
        
        debug!("Command '{}' exists: {}", command, exists);
        Ok(exists)
    }
}

#[async_trait]
impl CommandAdapterTrait for CommandRegistryAdapter {
    async fn execute_command(&self, command: &str, args: Vec<String>) -> AdapterResult<String> {
        debug!("Executing command: {} with args: {:?}", command, args);
        
        // Get command then release the lock before execution
        let cmd = {
            let timer = LockTimer::new("execute_command_get_command");
            let registry = self.registry.lock().await;
            let cmd = registry.get_command(command)?;
            let cmd_clone = cmd.clone_box();
            drop(registry);
            drop(timer);
            cmd_clone
        };
        
        // Execute without holding the lock
        let result = cmd.execute(&args)?;
        Ok(result)
    }
    
    async fn list_commands(&self) -> AdapterResult<Vec<String>> {
        debug!("Listing commands");
        
        let result = with_registry(&self.registry, "list_commands", |registry| {
            registry.list_commands()
        }).await;
        
        // Convert the CommandError to AdapterError
        match result {
            Ok(commands) => Ok(commands),
            Err(err) => Err(AdapterError::from(err)),
        }
    }
    
    async fn get_help(&self, command: &str) -> AdapterResult<String> {
        debug!("Getting help for command: {}", command);
        
        // Get command then release the lock
        let cmd = {
            let timer = LockTimer::new("get_help_get_command");
            let registry = self.registry.lock().await;
            let cmd = registry.get_command(command)?;
            let cmd_clone = cmd.clone_box();
            drop(registry);
            drop(timer);
            cmd_clone
        };
        
        // Generate help text without holding the lock
        Ok(format!("{}: {}", cmd.name(), cmd.description()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use commands::{Command, CommandError, CommandResult};
    
    // Test command for registry adapter tests
    #[derive(Debug, Clone)]
    struct TestCommand {
        name: String,
        description: String,
        result: String,
    }
    
    impl TestCommand {
        fn new(name: &str, description: &str, result: &str) -> Self {
            Self {
                name: name.to_string(),
                description: description.to_string(),
                result: result.to_string(),
            }
        }
    }
    
    impl Command for TestCommand {
        fn name(&self) -> &str {
            &self.name
        }
        
        fn description(&self) -> &str {
            &self.description
        }
        
        fn parser(&self) -> clap::Command {
            // Use hard-coded strings for the test command to avoid lifetime issues
            clap::Command::new("test_command")
                .about("Test command for unit tests")
        }
        
        fn execute(&self, args: &[String]) -> CommandResult<String> {
            if args.is_empty() {
                Ok(self.result.clone())
            } else {
                Ok(format!("Test command executed with args: {:?}", args))
            }
        }
        
        fn clone_box(&self) -> Box<dyn Command> {
            Box::new(self.clone())
        }
    }
    
    #[tokio::test]
    async fn test_register_and_execute_command() {
        // Create a new registry and adapter
        let registry = Arc::new(Mutex::new(CommandRegistry::new()));
        let adapter = CommandRegistryAdapter::new(registry);
        
        // Register a test command
        let test_command = TestCommand::new("test_command", "Test command for unit tests", "Test command executed successfully");
        adapter.register_command("test_command", Arc::new(test_command)).await.unwrap();
        
        // Check if command exists
        assert!(adapter.command_exists("test_command").await.unwrap());
        
        // Execute the command
        let result = adapter.execute_command("test_command", vec![]).await.unwrap();
        assert_eq!(result, "Test command executed successfully");
        
        // Execute with arguments
        let result = adapter.execute_command("test_command", vec!["arg1".to_string(), "arg2".to_string()]).await.unwrap();
        assert_eq!(result, "Test command executed with args: [\"arg1\", \"arg2\"]");
        
        // Get help for the command
        let help = adapter.get_help("test_command").await.unwrap();
        assert_eq!(help, "test_command: Test command for unit tests");
        
        // List commands
        let commands = adapter.list_commands().await.unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], "test_command");
    }
    
    #[tokio::test]
    async fn test_nonexistent_command() {
        // Create a new registry and adapter
        let registry = Arc::new(Mutex::new(CommandRegistry::new()));
        let adapter = CommandRegistryAdapter::new(registry);
        
        // Check if command exists
        assert!(!adapter.command_exists("nonexistent").await.unwrap());
        
        // Try to execute nonexistent command
        let result = adapter.execute_command("nonexistent", vec![]).await;
        assert!(result.is_err());
        
        // Try to get help for nonexistent command
        let help = adapter.get_help("nonexistent").await;
        assert!(help.is_err());
    }
} 