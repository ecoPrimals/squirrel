use std::sync::Arc;
use async_trait::async_trait;
use clap::{Command as ClapCommand, Arg};

use crate::error::AdapterError;
use commands::{Command, CommandResult};
use crate::commands::error::CommandError;
use crate::commands::adapter::AdapterResult;

/// A trait representing a command interface used for testing
#[async_trait]
pub trait TestCommand: Send + Sync {
    /// Returns the name of the command
    fn name(&self) -> &str;
    
    /// Returns the description of the command
    fn description(&self) -> &str;
    
    /// Executes the command with the given arguments
    async fn execute(&self, args: Vec<String>) -> Result<String, AdapterError>;
    
    /// Returns the clap Command parser for this command
    fn parser(&self) -> ClapCommand;
}

/// Simple test command for testing purpose
#[derive(Debug, Clone)]
pub struct SimpleTestCommand {
    name: String,
    description: String,
}

impl SimpleTestCommand {
    /// Create a new simple test command
    pub fn new(name: String, description: String) -> Self {
        Self { name, description }
    }
}

#[async_trait]
impl TestCommand for SimpleTestCommand {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    async fn execute(&self, args: Vec<String>) -> Result<String, AdapterError> {
        if args.is_empty() {
            return Ok("This is a test command".to_string());
        }
        
        let first_arg = &args[0];
        Ok(format!("Test command received: {}", first_arg))
    }
    
    fn parser(&self) -> ClapCommand {
        let name = "test";
        let description = "Simple test command";
        
        ClapCommand::new(name)
            .about(description)
            .arg(Arg::new("input")
                .help("Input parameter")
                .required(false)
                .index(1))
    }
}

impl Command for SimpleTestCommand {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn parser(&self) -> ClapCommand {
        let name = "test";
        let description = "Simple test command";
        
        ClapCommand::new(name)
            .about(description)
            .arg(Arg::new("input")
                .help("Input parameter")
                .required(false)
                .index(1))
    }
    
    fn execute(&self, args: &[String]) -> CommandResult<String> {
        if args.is_empty() {
            return Ok("This is a test command".to_string());
        }
        
        let first_arg = &args[0];
        Ok(format!("Test command received: {}", first_arg))
    }
    
    fn help(&self) -> String {
        "Simple test command used for testing purposes".to_string()
    }
    
    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Simple adapter test command for testing adapter functionality
pub struct SimpleAdapterTestCommand {
    name: String,
    description: String,
    result: String,
}

impl SimpleAdapterTestCommand {
    /// Create a new simple adapter test command
    pub fn new(name: String, description: String, result: String) -> Self {
        Self { name, description, result }
    }
    
    /// Returns the name of the command
    fn name(&self) -> &str {
        &self.name
    }
}

#[async_trait]
impl crate::commands::adapter::CommandAdapterTrait for SimpleAdapterTestCommand {
    async fn execute_command(&self, command: &str, args: Vec<String>) -> AdapterResult<String> {
        if command == self.name() {
            if args.is_empty() {
                Ok(self.result.clone())
            } else {
                let first_arg = &args[0];
                Ok(format!("Test command received: {}", first_arg))
            }
        } else {
            Err(crate::commands::adapter::error::AdapterError::NotFound(command.to_string()))
        }
    }
    
    async fn get_help(&self, command: &str) -> AdapterResult<String> {
        if command == self.name() {
            Ok(format!("Help for {}: {}", self.name, self.description))
        } else {
            Err(crate::commands::adapter::error::AdapterError::NotFound(command.to_string()))
        }
    }
    
    async fn list_commands(&self) -> AdapterResult<Vec<String>> {
        Ok(vec![self.name.clone()])
    }
}

/// Factory function to create a test command
pub fn create_test_command(name: &str, description: &str, _result: &str) -> Arc<dyn TestCommand> {
    Arc::new(SimpleTestCommand::new(name.to_string(), description.to_string()))
} 