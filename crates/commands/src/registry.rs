//! Command registry for managing and executing commands
//!
//! This module provides the core types and interfaces for the command system.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use crate::CommandError;

/// Type alias for command operation results
pub type CommandResult<T> = Result<T, CommandError>;

/// Trait for commands to implement
pub trait Command: Send + Sync {
    /// Returns the name of the command
    fn name(&self) -> &str;
    
    /// Returns a description of what the command does
    fn description(&self) -> &str;
    
    /// Executes the command with the given arguments
    fn execute(&self, args: &[String]) -> CommandResult<String>;
    
    /// Returns help text for the command
    fn help(&self) -> String {
        format!("{}: {}", self.name(), self.description())
    }
}

/// Registry for storing and executing commands
pub struct CommandRegistry {
    /// Map of command names to command implementations
    commands: Mutex<HashMap<String, Box<dyn Command>>>,
}

// Manual implementation of Debug for CommandRegistry
impl fmt::Debug for CommandRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Try to acquire the lock - if we can't, just show a placeholder
        match self.commands.try_lock() {
            Ok(commands) => {
                let command_names: Vec<_> = commands.keys().collect();
                f.debug_struct("CommandRegistry")
                    .field("commands", &command_names)
                    .finish()
            }
            Err(_) => {
                f.debug_struct("CommandRegistry")
                    .field("commands", &"<locked>")
                    .finish()
            }
        }
    }
}

impl CommandRegistry {
    /// Creates a new command registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            commands: Mutex::new(HashMap::new()),
        }
    }
    
    /// Registers a command with the registry
    /// 
    /// # Arguments
    /// 
    /// * `command` - The command to register
    /// 
    /// # Errors
    /// 
    /// Returns an error if a command with the same name already exists
    pub fn register(&self, command: Box<dyn Command>) -> CommandResult<()> {
        let name = command.name().to_string();
        let mut commands = self.commands.lock().map_err(|_| {
            CommandError::RegistrationError("Failed to acquire lock on command registry".to_string())
        })?;
        
        if commands.contains_key(&name) {
            return Err(CommandError::RegistrationError(format!("Command '{}' already exists", name)));
        }
        
        commands.insert(name, command);
        Ok(())
    }
    
    /// Executes a command by name with the given arguments
    /// 
    /// # Arguments
    /// 
    /// * `name` - The name of the command to execute
    /// * `args` - The arguments to pass to the command
    /// 
    /// # Errors
    /// 
    /// Returns an error if the command does not exist or if execution fails
    pub fn execute(&self, name: &str, args: &[String]) -> CommandResult<String> {
        // We need to execute within the lock scope since we can't clone Box<dyn Command>
        let commands = self.commands.lock().map_err(|e| {
            CommandError::ExecutionError(format!("Failed to acquire lock on command registry: {}", e))
        })?;
        
        let command = commands.get(name)
            .ok_or_else(|| CommandError::ExecutionError(format!("Command '{}' not found", name)))?;
        
        // Execute the command while holding the lock
        command.execute(args)
    }
    
    /// Returns a list of all registered command names
    /// 
    /// # Errors
    /// 
    /// Returns an error if the command registry cannot be locked
    pub fn list_commands(&self) -> CommandResult<Vec<String>> {
        let commands = self.commands.lock().map_err(|e| {
            CommandError::ExecutionError(format!("Failed to acquire lock on command registry: {}", e))
        })?;
        
        // Clone the keys to release the lock as soon as possible
        Ok(commands.keys().cloned().collect())
    }
    
    /// Returns help text for a command
    /// 
    /// # Arguments
    /// 
    /// * `name` - The name of the command to get help for
    /// 
    /// # Errors
    /// 
    /// Returns an error if the command does not exist
    pub fn get_help(&self, name: &str) -> CommandResult<String> {
        // Get the help text while holding the lock
        let help_text = {
            let commands = self.commands.lock().map_err(|e| {
                CommandError::ExecutionError(format!("Failed to acquire lock on command registry: {}", e))
            })?;
            
            let command = commands.get(name)
                .ok_or_else(|| CommandError::ExecutionError(format!("Command '{}' not found", name)))?;
            
            command.help()
        };
        
        Ok(help_text)
    }
    
    /// Retrieves a command by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the command to retrieve
    ///
    /// # Errors
    ///
    /// Returns an error if the command does not exist
    pub fn get_command(&self, name: &str) -> CommandResult<Arc<dyn Command>> {
        let commands = self.commands.lock().map_err(|_| {
            CommandError::ExecutionError("Failed to acquire lock on command registry".to_string())
        })?;
        
        let _command = commands.get(name).ok_or_else(|| {
            CommandError::ExecutionError(format!("Command '{}' not found", name))
        })?;
        
        // This is not implemented correctly - we can't easily clone a Box<dyn Command>
        // into an Arc<dyn Command> without potentially causing memory issues
        // For now, let's return an error
        Err(CommandError::ExecutionError("Command retrieval not implemented".to_string()))
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[derive(Debug, Clone)]
    struct TestCommand;
    
    impl Command for TestCommand {
        fn name(&self) -> &str {
            "test"
        }
        
        fn description(&self) -> &str {
            "A test command"
        }
        
        fn execute(&self, _args: &[String]) -> CommandResult<String> {
            Ok("Test command executed".to_string())
        }
    }
    
    #[test]
    fn test_register_command() {
        let registry = CommandRegistry::new();
        let result = registry.register(Box::new(TestCommand));
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_register_duplicate_command() {
        let registry = CommandRegistry::new();
        registry.register(Box::new(TestCommand)).unwrap();
        let result = registry.register(Box::new(TestCommand));
        assert!(result.is_err());
    }
    
    #[test]
    fn test_execute_command() {
        let registry = CommandRegistry::new();
        registry.register(Box::new(TestCommand)).unwrap();
        let result = registry.execute("test", &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Test command executed");
    }
    
    #[test]
    fn test_execute_nonexistent_command() {
        let registry = CommandRegistry::new();
        let result = registry.execute("nonexistent", &[]);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_list_commands() {
        let registry = CommandRegistry::new();
        registry.register(Box::new(TestCommand)).unwrap();
        let result = registry.list_commands();
        assert!(result.is_ok());
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], "test");
    }
    
    #[test]
    fn test_get_help() {
        let registry = CommandRegistry::new();
        registry.register(Box::new(TestCommand)).unwrap();
        let result = registry.get_help("test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test: A test command");
    }
} 