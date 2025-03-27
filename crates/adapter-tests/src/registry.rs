//! Command Registry implementation
//!
//! This module contains the implementation of the MockCommandRegistry, which
//! manages the registration and execution of commands.

use std::collections::HashMap;
use std::sync::Arc;
use std::fmt;

use crate::command::MockCommand;
use crate::error::{AdapterError, AdapterResult};

/// Registry for commands
///
/// This registry stores commands and provides methods for registering,
/// executing, and retrieving information about commands.
pub struct MockCommandRegistry {
    /// Map of command names to command instances
    commands: HashMap<String, Arc<dyn MockCommand + Send + Sync>>,
}

impl fmt::Debug for MockCommandRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MockCommandRegistry")
            .field("command_count", &self.commands.len())
            .field("command_names", &self.commands.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl MockCommandRegistry {
    /// Creates a new empty command registry
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }
    
    /// Creates a registry with predefined commands
    pub fn with_commands(commands: Vec<Arc<dyn MockCommand + Send + Sync>>) -> Self {
        let mut registry = Self::new();
        for cmd in commands {
            // Clone the Arc reference to avoid ownership issues
            let name = cmd.name().to_string();
            let _ = registry.register(&name, cmd.clone());
        }
        registry
    }
    
    /// Registers a command with the registry
    ///
    /// # Arguments
    ///
    /// * `name` - The name to register the command under
    /// * `command` - The command to register
    ///
    /// # Returns
    ///
    /// * `Ok(())` if registration succeeded
    /// * `Err(AdapterError)` if registration failed
    pub fn register(&mut self, name: &str, command: Arc<dyn MockCommand + Send + Sync>) -> AdapterResult<()> {
        self.commands.insert(name.to_string(), command);
        Ok(())
    }
    
    /// Executes a command with the given arguments
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the command to execute
    /// * `args` - The arguments to pass to the command
    ///
    /// # Returns
    ///
    /// * `Ok(String)` containing the command output
    /// * `Err(AdapterError)` if execution failed
    pub fn execute(&self, name: &str, args: Vec<String>) -> AdapterResult<String> {
        match self.commands.get(name) {
            Some(cmd) => cmd.execute(args),
            None => Err(AdapterError::NotFound(name.to_string())),
        }
    }
    
    /// Gets help information for a command
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the command to get help for
    ///
    /// # Returns
    ///
    /// * `Ok(String)` containing the help information
    /// * `Err(AdapterError)` if getting help failed
    pub fn get_help(&self, name: &str) -> AdapterResult<String> {
        match self.commands.get(name) {
            Some(cmd) => Ok(format!("{}: {}", cmd.name(), cmd.description())),
            None => Err(AdapterError::NotFound(name.to_string())),
        }
    }
    
    /// Lists all registered commands
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` containing command names
    /// * `Err(AdapterError)` if listing failed
    pub fn list_commands(&self) -> AdapterResult<Vec<String>> {
        Ok(self.commands.keys().cloned().collect())
    }
    
    /// Gets a reference to a command by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the command to get
    ///
    /// # Returns
    ///
    /// * `Some(&Arc<dyn MockCommand + Send + Sync>)` if the command exists
    /// * `None` if the command does not exist
    pub fn get_command(&self, name: &str) -> Option<&Arc<dyn MockCommand + Send + Sync>> {
        self.commands.get(name)
    }
    
    /// Checks if a command exists in the registry
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the command to check
    ///
    /// # Returns
    ///
    /// * `true` if the command exists
    /// * `false` if the command does not exist
    pub fn has_command(&self, name: &str) -> bool {
        self.commands.contains_key(name)
    }
    
    /// Gets the number of registered commands
    pub fn command_count(&self) -> usize {
        self.commands.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::TestCommand;

    #[test]
    fn test_registry_operations() {
        let mut registry = MockCommandRegistry::new();
        
        // Register a command
        let test_cmd = TestCommand::new("test", "A test command", "Test result");
        registry.register("test", Arc::new(test_cmd)).unwrap();
        
        // Check command exists
        assert!(registry.has_command("test"));
        assert_eq!(registry.command_count(), 1);
        
        // Execute command
        let result = registry.execute("test", vec![]).unwrap();
        assert_eq!(result, "Test result");
        
        // Get help
        let help = registry.get_help("test").unwrap();
        assert_eq!(help, "test: A test command");
        
        // List commands
        let commands = registry.list_commands().unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], "test");
    }
    
    #[test]
    fn test_registry_with_commands() {
        let cmds = vec![
            Arc::new(TestCommand::hello()) as Arc<dyn MockCommand + Send + Sync>,
            Arc::new(TestCommand::echo()) as Arc<dyn MockCommand + Send + Sync>,
        ];
        
        let registry = MockCommandRegistry::with_commands(cmds);
        
        assert_eq!(registry.command_count(), 2);
        assert!(registry.has_command("hello"));
        assert!(registry.has_command("echo"));
    }
} 