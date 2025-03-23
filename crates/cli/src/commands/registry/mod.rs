//! Command registry for CLI commands

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

/// Command registry for CLI commands
pub struct CommandRegistry {
    commands: Arc<Mutex<HashMap<String, Box<dyn super::Command>>>>,
}

impl fmt::Debug for CommandRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CommandRegistry")
            .field("commands", &"<commands>")
            .finish()
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRegistry {
    /// Create a new command registry
    pub fn new() -> Self {
        Self {
            commands: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a command
    pub fn register(&mut self, command: Box<dyn super::Command>) {
        let name = command.name().to_owned();
        let mut commands = self.commands.lock().unwrap();
        commands.insert(name, command);
    }

    /// Get a command by name
    pub fn get(&self, name: &str) -> Option<Box<dyn super::Command>> {
        let commands = self.commands.lock().unwrap();
        commands.get(name).map(|cmd| cmd.clone_box())
    }

    /// Get all command names
    pub fn command_names(&self) -> Vec<String> {
        let commands = self.commands.lock().unwrap();
        commands.keys().cloned().collect()
    }
}

/// CLI interface
pub mod cli {
    use clap::Parser;
    use super::CommandRegistry;

    /// CLI interface
    #[derive(Debug, Parser)]
    #[command(author, version, about, long_about = None)]
    pub struct Cli {
        /// Command to execute
        #[arg(value_name = "COMMAND")]
        pub command: Option<String>,

        /// Command arguments
        #[arg(trailing_var_arg = true)]
        pub args: Vec<String>,
    }

    impl Cli {
        /// Create a new CLI
        pub fn new(_registry: &CommandRegistry) -> Self {
            Self {
                command: None,
                args: Vec::new(),
            }
        }
    }
} 