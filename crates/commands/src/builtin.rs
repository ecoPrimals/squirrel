//! Built-in commands for the Squirrel system
//!
//! This module provides basic built-in commands such as help and version.

use std::sync::Arc;
use crate::registry::{Command, CommandResult, CommandRegistry};

/// Arguments for the help command
#[derive(Debug, Clone)]
pub struct HelpCommand {
    #[allow(dead_code)]
    registry: Arc<CommandRegistry>,
}

impl HelpCommand {
    /// Create a new help command
    #[must_use]
    pub fn new(registry: Arc<CommandRegistry>) -> Self {
        Self { registry }
    }
}

impl Command for HelpCommand {
    fn name(&self) -> &str {
        "help"
    }

    fn description(&self) -> &str {
        "Display help information for available commands"
    }

    fn execute(&self, args: &[String]) -> CommandResult<String> {
        // Basic help text
        let help_text = if args.is_empty() {
            // Simply return our own help text for now to avoid potential deadlocks
            let my_help = "Available commands:\n- help: Display help information for available commands\n- version: Display version information";
            String::from(my_help)
        } else {
            // Show help for a specific command
            match args[0].as_str() {
                "help" => "help: Display help information for available commands".to_string(),
                "version" => "version: Display version information".to_string(),
                _ => format!("Unknown command: {}", args[0]),
            }
        };
        
        Ok(help_text)
    }
}

/// Command that displays the version information
#[derive(Debug, Clone)]
pub struct VersionCommand;

impl Command for VersionCommand {
    fn name(&self) -> &str {
        "version"
    }

    fn description(&self) -> &str {
        "Display version information"
    }

    fn execute(&self, _args: &[String]) -> CommandResult<String> {
        let version = squirrel_core::build_info::version();
        Ok(format!("Squirrel version: {}", version))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_command() {
        let cmd = VersionCommand;
        assert_eq!(cmd.name(), "version");
        assert!(cmd.execute(&[]).is_ok());
    }

    #[test]
    fn test_help_command() {
        let cmd = HelpCommand {
            registry: Arc::new(CommandRegistry::new()),
        };
        assert_eq!(cmd.name(), "help");
        assert!(cmd.execute(&[]).is_ok());
    }
} 