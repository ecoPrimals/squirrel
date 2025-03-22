//! Command executor for CLI commands
//!
//! This module provides functionality for executing commands in the CLI.

use clap::ArgMatches;
use std::sync::Arc;
use tracing::{debug, error, info};

use squirrel_commands::{CommandError, CommandRegistry};
use crate::formatter::Factory as FormatterFactory;
use crate::commands::context::CommandContext;

/// Context for command execution
#[derive(Debug)]
pub struct ExecutionContext {
    /// Registry of available commands
    registry: Arc<CommandRegistry>,
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new(registry: Arc<CommandRegistry>) -> Self {
        Self {
            registry,
        }
    }

    /// Execute a command with the given arguments
    ///
    /// # Arguments
    ///
    /// * `command_name` - The name of the command to execute
    /// * `matches` - The parsed command-line arguments
    pub async fn execute_command(&self, command_name: &str, matches: ArgMatches) -> Result<(), CommandError> {
        debug!("Executing command: {}", command_name);
        
        // Create command context
        let context = CommandContext::new(matches);
        
        // Get the command from the registry
        let command = self.registry.get_command(command_name)?;
        
        // Extract args for the base Command trait
        let args: Vec<String> = context.matches().get_many("args")
            .map(|v| v.cloned().collect())
            .unwrap_or_default();
            
        // Execute the command
        match command.execute(&args) {
            Ok(output) => {
                // Determine output format from context flags
                let format = if context.matches().get_flag("json") {
                    "json"
                } else if context.matches().get_flag("yaml") {
                    "yaml"
                } else if context.matches().get_flag("table") {
                    "table"
                } else {
                    "text"
                };

                // Create formatter and format output
                let formatter = FormatterFactory::create_formatter(format)
                    .map_err(|e| CommandError::ExecutionError(e.to_string()))?;
                
                // Print the output
                println!("{}", formatter.format(&output).map_err(|e| CommandError::ExecutionError(e.to_string()))?);
                
                info!("Command '{}' executed successfully", command_name);
                Ok(())
            }
            Err(err) => {
                error!("Command '{}' execution failed: {}", command_name, err);
                Err(err)
            }
        }
    }
} 