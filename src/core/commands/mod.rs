//! Command system for the Squirrel project
//!
//! This module provides the command system functionality, which allows
//! executing commands and managing their lifecycle. Commands can be
//! validated, executed, and monitored through various hooks.

mod lifecycle;

use std::fmt;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;

use crate::core::error::{Error, CommandError};

/// The type of command
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommandType {
    /// System command
    System,
    /// User command
    User,
    /// Custom command
    Custom(String),
}

/// A command in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    /// The name of the command
    pub name: String,
    /// The type of command
    pub command_type: CommandType,
    /// The arguments for the command
    pub args: Vec<String>,
    /// The environment variables for the command
    pub env: HashMap<String, String>,
}

/// The result of executing a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    /// Whether the command was successful
    pub success: bool,
    /// The output of the command
    pub output: String,
    /// The error message if the command failed
    pub error: Option<String>,
}

/// The main command handler
#[derive(Debug, Default)]
pub struct CommandHandler {
    /// The command handlers
    handlers: HashMap<String, Box<dyn CommandExecutor>>,
}

impl CommandHandler {
    /// Create a new command handler
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a command handler
    pub fn register(&mut self, name: impl Into<String>, handler: Box<dyn CommandExecutor>) {
        self.handlers.insert(name.into(), handler);
    }

    /// Execute a command
    pub async fn execute(&self, command: Command) -> Result<CommandResult, Error> {
        let handler = self
            .handlers
            .get(&command.name)
            .ok_or_else(|| CommandError::Handler(format!("No handler for command: {}", command.name)))?;

        handler.execute(command).await
    }
}

impl Default for CommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CommandType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandType::System => write!(f, "System"),
            CommandType::User => write!(f, "User"),
            CommandType::Custom(s) => write!(f, "Custom({})", s),
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Command(name={}, type={}, args={}, env={})",
            self.name,
            self.command_type,
            self.args.len(),
            self.env.len()
        )
    }
}

impl fmt::Display for CommandResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CommandResult(success={}, output={}, error={})",
            self.success,
            self.output,
            self.error.as_deref().unwrap_or("None")
        )
    }
}

/// Trait for command executors
#[async_trait]
pub trait CommandExecutor: Send + Sync {
    /// Execute a command
    async fn execute(&self, command: Command) -> Result<CommandResult, Error>;
}

/// Trait for command validators
pub trait CommandValidator: Send + Sync {
    /// Validate a command
    fn validate(&self, command: &Command) -> Result<(), Error>;
}

/// Trait for command hooks
pub trait CommandHook: Send + Sync {
    /// Called before a command is executed
    fn pre_execute(&self, command: &Command) -> Result<(), Error>;
    /// Called after a command is executed
    fn post_execute(&self, command: &Command, result: &CommandResult) -> Result<(), Error>;
}

// Re-export lifecycle types
pub use lifecycle::{LifecycleStage, CommandLifecycle, CommandHook, LoggingHook}; 