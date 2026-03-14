// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Core types for the adapter pattern implementation

use std::fmt::{self, Debug};

/// Result type for command operations
pub type CommandResult<T> = Result<T, CommandError>;

/// Error type for command operations
#[derive(Debug)]
pub enum CommandError {
    /// Command not found in registry
    NotFound(String),
    /// Command execution failed
    ExecutionFailed(String),
    /// Authentication failed
    AuthenticationFailed(String),
    /// Authorization failed (insufficient permissions)
    AuthorizationFailed(String),
    /// Other error
    Other(String),
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(s) => write!(f, "Command not found: {s}"),
            Self::ExecutionFailed(s) => write!(f, "Execution failed: {s}"),
            Self::AuthenticationFailed(s) => write!(f, "Authentication failed: {s}"),
            Self::AuthorizationFailed(s) => write!(f, "Authorization failed: {s}"),
            Self::Other(s) => write!(f, "Error: {s}"),
        }
    }
}

impl std::error::Error for CommandError {}

/// Command trait representing a command interface
pub trait Command: Send + Sync + Debug {
    /// Get the command name
    fn name(&self) -> &str;
    /// Get the command description
    fn description(&self) -> &str;
    /// Execute the command with given arguments
    ///
    /// # Errors
    ///
    /// Returns `CommandError` if execution fails.
    fn execute(&self, args: Vec<String>) -> CommandResult<String>;
}

/// `TestCommand` is a mock command implementation for testing
#[derive(Debug, Clone)]
pub struct TestCommand {
    name: String,
    description: String,
    result: String,
}

impl TestCommand {
    /// Create a new test command
    #[must_use]
    pub fn new(name: &str, description: &str, result: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            result: result.to_string(),
        }
    }
}

impl Command for TestCommand {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn execute(&self, args: Vec<String>) -> CommandResult<String> {
        if args.is_empty() {
            Ok(self.result.clone())
        } else {
            Ok(format!("{} with args: {:?}", self.result, args))
        }
    }
}

/// Authentication type for MCP adapter
#[derive(Debug, Clone)]
pub enum Auth {
    /// Username and password authentication
    User(String, String), // username, password
    /// Token-based authentication
    Token(String), // authentication token
    /// API key authentication
    ApiKey(String), // API key
    /// No authentication
    None,
}

/// User role for permission management
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserRole {
    /// Administrator with full access
    Admin,
    /// Power user with elevated permissions
    PowerUser,
    /// Regular user with standard permissions
    RegularUser,
    /// Guest user with limited access
    Guest,
}

/// Authentication result containing user information
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub(crate) username: String,
    pub(crate) roles: Vec<UserRole>,
}

/// `CommandLogEntry` for command audit logging.
#[derive(Debug, Clone)]
pub struct CommandLogEntry {
    pub(crate) command: String,
    pub(crate) args: Vec<String>,
    pub(crate) user: Option<String>,
    pub(crate) timestamp: std::time::SystemTime,
    pub(crate) success: bool,
    pub(crate) message: String,
}
