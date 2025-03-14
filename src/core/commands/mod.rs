//! Command system for the Squirrel project

//!
//! This module provides the command system functionality, which allows
//! executing commands and managing their lifecycle. Commands can be
//! validated, executed, and monitored through various hooks.

pub mod lifecycle;
mod executor;

use std::fmt;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use std::sync::Arc;

use crate::core::error::Result;

// Re-export types
pub use executor::BasicCommandExecutor;
pub use lifecycle::{LifecycleStage, CommandLifecycle, LoggingHook};

/// The type of command
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
pub struct CommandArgs {
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
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

impl Default for CommandResult {
    fn default() -> Self {
        Self {
            success: false,
            output: String::new(),
            error: None,
        }
    }
}

// Base trait for commands
pub trait Command: Send + Sync {
    /// Returns the name of the command
    fn name(&self) -> &str;
    
    /// Returns the description of the command
    fn description(&self) -> &str;

    /// Execute the command asynchronously
    fn execute<'a>(&'a self, args: &'a CommandArgs) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<CommandResult>> + Send + 'a>>;
}

// Base trait for command executors
pub trait CommandExecutor: Send + Sync {
    /// Execute a command asynchronously
    fn execute<'a>(&'a self, command: &'a dyn Command, args: &'a CommandArgs) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<CommandResult>> + Send + 'a>>;
    
    /// Validate command arguments
    fn validate<'a>(&'a self, args: &'a CommandArgs) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>>;
    
    /// Cleanup after command execution
    fn cleanup<'a>(&'a self, args: &'a CommandArgs) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>>;
    
    /// Pre-execute hook
    fn pre_execute<'a>(&'a self, args: &'a CommandArgs) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>>;
    
    /// Post-execute hook
    fn post_execute<'a>(&'a self, args: &'a CommandArgs, result: &'a CommandResult) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>>;
}

pub struct CommandHandler {
    executor: Box<dyn CommandExecutor>,
}

impl CommandHandler {
    pub fn new(executor: Box<dyn CommandExecutor>) -> Self {
        Self { executor }
    }

    pub async fn execute(&self, command: &dyn Command, args: &CommandArgs) -> Result<CommandResult> {
        self.executor.validate(args).await?;
        self.executor.pre_execute(args).await?;
        let result = self.executor.execute(command, args).await?;
        self.executor.post_execute(args, &result).await?;
        self.executor.cleanup(args).await?;
        Ok(result)
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

impl fmt::Display for CommandArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CommandArgs(args={}, env={})", self.args.len(), self.env.len())
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

/// Trait for command validators
pub trait CommandValidator: Send + Sync {
    /// Validate a command
    fn validate<'a>(&'a self, args: &'a CommandArgs) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>>;
}

/// Trait for command hooks
pub trait CommandHook: Send + Sync {
    /// Called before a command is executed
    fn pre_execute<'a>(&'a self, args: &'a CommandArgs) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>>;
    
    /// Called after a command is executed
    fn post_execute<'a>(&'a self, args: &'a CommandArgs, result: &'a CommandResult) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>>;
}

pub struct BasicCommand {
    name: String,
    description: String,
}

impl BasicCommand {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
        }
    }
}

impl Command for BasicCommand {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn execute<'a>(&'a self, args: &'a CommandArgs) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<CommandResult>> + Send + 'a>> {
        Box::pin(async move {
            Ok(CommandResult {
                success: true,
                output: format!("Executed command {} with args {:?}", self.name, args),
                error: None,
            })
        })
    }
}

impl fmt::Display for BasicCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.description)
    }
} 