// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Mock Implementations for Testing
//!
//! This module provides mock implementations for testing the task server functionality.

use clap;
use std::collections::HashMap;

use super::types::SimpleCommand;

/// Mock command implementation for testing
#[derive(Debug)]
pub struct MockCommand {
    name: String,
    description: String,
}

impl MockCommand {
    /// Create a new mock command.
    #[must_use]
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
        }
    }

    /// Clone this command into a boxed trait object.
    #[must_use]
    pub fn clone_box(&self) -> Box<dyn SimpleCommand> {
        Box::new(self.clone())
    }
}

impl Clone for MockCommand {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            description: self.description.clone(),
        }
    }
}

impl SimpleCommand for MockCommand {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn execute(&self, args: &[String]) -> Result<String, String> {
        Ok(format!(
            "Mock command '{}' executed with args: {:?}",
            self.name, args
        ))
    }

    fn parser(&self) -> clap::Command {
        clap::Command::new("mock_command")
            .about("Mock command for testing")
            .arg(
                clap::Arg::new("args")
                    .help("Arguments for the command")
                    .num_args(0..)
                    .value_name("ARGS"),
            )
    }

    fn clone_box(&self) -> Box<dyn SimpleCommand> {
        Box::new(self.clone())
    }
}

/// Mock command registry for testing
#[derive(Debug)]
pub struct MockCommandRegistry {
    commands: HashMap<String, MockCommand>,
}

impl MockCommandRegistry {
    /// Create a new mock command registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    /// Register a command in the mock registry
    pub fn register(&mut self, command: MockCommand) -> Result<(), String> {
        self.commands.insert(command.name().to_string(), command);
        Ok(())
    }

    /// Get list of command names.
    #[must_use]
    pub fn command_names(&self) -> Vec<String> {
        self.commands.keys().cloned().collect()
    }

    /// Get a command by name
    pub fn get(&mut self, name: &str) -> Option<MockCommand> {
        self.commands.get(name).cloned()
    }

    /// List all commands with their descriptions.
    #[must_use]
    pub fn list_commands(&self) -> Vec<(String, String)> {
        self.commands
            .values()
            .map(|cmd| (cmd.name().to_string(), cmd.description().to_string()))
            .collect()
    }

    /// Get help for a specific command
    pub fn get_help(&self, name: &str) -> Result<String, String> {
        if let Some(command) = self.commands.get(name) {
            Ok(command.help())
        } else {
            Err(format!("Command '{name}' not found"))
        }
    }
}

impl Default for MockCommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}
