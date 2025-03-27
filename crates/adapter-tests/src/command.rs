//! Command definitions and implementations
//!
//! This module contains the definition of the MockCommand trait and
//! implementations of test commands for use in adapter tests.

use std::sync::Arc;

use crate::error::AdapterResult;

/// Trait defining the common interface for all commands
///
/// This trait defines the operations that all commands must support,
/// including execution, help, and metadata.
pub trait MockCommand: Send + Sync {
    /// Gets the command name
    fn name(&self) -> &str;
    
    /// Gets the command description
    fn description(&self) -> &str;
    
    /// Executes the command with the given arguments
    fn execute(&self, args: Vec<String>) -> AdapterResult<String>;
    
    /// Creates a boxed clone of the command
    fn clone_box(&self) -> Box<dyn MockCommand + Send + Sync>;
    
    /// Returns the usage of the command
    fn usage(&self) -> String {
        format!("{}: {}", self.name(), self.description())
    }
}

/// A test command implementation for use in adapter tests
///
/// This command returns a predefined result when executed, optionally
/// including the arguments that were passed to it.
#[derive(Clone, Debug)]
pub struct TestCommand {
    /// The name of the command
    name: String,
    
    /// The description of the command
    description: String,
    
    /// The result to return when executed
    result: String,
}

impl TestCommand {
    /// Creates a new test command
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the command
    /// * `description` - The description of the command
    /// * `result` - The result to return when executed
    pub fn new(name: &str, description: &str, result: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            result: result.to_string(),
        }
    }
    
    /// Creates a simple hello command
    pub fn hello() -> Self {
        Self::new(
            "hello",
            "Says hello to the user",
            "Hello, world!"
        )
    }
    
    /// Creates an echo command
    pub fn echo() -> Self {
        Self::new(
            "echo",
            "Echoes back the arguments",
            "Echo"
        )
    }
    
    /// Creates an admin command
    pub fn admin() -> Self {
        Self::new(
            "admin-cmd",
            "An admin-only command",
            "Admin command result"
        )
    }
}

impl MockCommand for TestCommand {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn execute(&self, args: Vec<String>) -> AdapterResult<String> {
        if args.is_empty() {
            Ok(self.result.clone())
        } else {
            Ok(format!("{} with args: {:?}", self.result, args))
        }
    }
    
    fn clone_box(&self) -> Box<dyn MockCommand + Send + Sync> {
        Box::new(self.clone())
    }
}

/// Converts a TestCommand to an Arc-wrapped MockCommand
pub fn to_arc_command(command: TestCommand) -> Arc<dyn MockCommand + Send + Sync> {
    Arc::new(command)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_execution() {
        let command = TestCommand::new("test", "A test command", "Test result");
        
        // Execute without arguments
        let result = command.execute(vec![]).unwrap();
        assert_eq!(result, "Test result");
        
        // Execute with arguments
        let result = command.execute(vec!["arg1".to_string(), "arg2".to_string()]).unwrap();
        assert_eq!(result, "Test result with args: [\"arg1\", \"arg2\"]");
    }
    
    #[test]
    fn test_command_factory_methods() {
        let hello = TestCommand::hello();
        assert_eq!(hello.name(), "hello");
        assert_eq!(hello.description(), "Says hello to the user");
        
        let echo = TestCommand::echo();
        assert_eq!(echo.name(), "echo");
        assert_eq!(echo.description(), "Echoes back the arguments");
        
        let admin = TestCommand::admin();
        assert_eq!(admin.name(), "admin-cmd");
        assert_eq!(admin.description(), "An admin-only command");
    }
    
    #[test]
    fn test_command_usage() {
        let command = TestCommand::new("test", "A test command", "Test result");
        assert_eq!(command.usage(), "test: A test command");
    }
} 