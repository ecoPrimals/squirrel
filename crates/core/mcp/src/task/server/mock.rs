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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::server::types::SimpleCommand;

    #[test]
    fn mock_command_implements_simple_command_execute_parser_help_clone_box() {
        let cmd = MockCommand::new("alpha", "beta");
        let sc: &dyn SimpleCommand = &cmd;
        assert_eq!(sc.name(), "alpha");
        assert_eq!(sc.description(), "beta");
        assert!(
            sc.execute(&[String::from("z")])
                .expect("should succeed")
                .contains("alpha")
        );
        assert_eq!(sc.parser().get_name(), "mock_command");
        assert!(sc.help().contains("alpha"));
        let boxed = sc.clone_box();
        assert_eq!(boxed.name(), "alpha");
    }

    #[test]
    fn mock_command_debug_and_assoc_clone_box() {
        let c = MockCommand::new("x", "y");
        let dbg = format!("{c:?}");
        assert!(dbg.contains("MockCommand"));
        let b = MockCommand::clone_box(&c);
        assert_eq!(
            b.execute(&[]).expect("should succeed"),
            c.execute(&[]).expect("should succeed")
        );
    }

    #[test]
    fn mock_command_registry_register_list_get_help_and_errors() {
        let mut reg = MockCommandRegistry::new();
        reg.register(MockCommand::new("one", "first"))
            .expect("should succeed");
        let names = reg.command_names();
        assert!(names.contains(&"one".to_string()));
        let listed = reg.list_commands();
        assert!(listed.iter().any(|(n, _)| n == "one"));
        assert_eq!(reg.get("one").expect("should succeed").name(), "one");
        assert_eq!(
            reg.get_help("one").expect("should succeed"),
            MockCommand::new("one", "first").help()
        );
        assert!(reg.get_help("nope").is_err());
        let empty = MockCommandRegistry::default();
        assert!(empty.command_names().is_empty());
    }
}
