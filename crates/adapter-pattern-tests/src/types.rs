// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_error_display_not_found() {
        let err = CommandError::NotFound("cmd".to_string());
        let s = err.to_string();
        assert!(s.contains("Command not found"));
        assert!(s.contains("cmd"));
    }

    #[test]
    fn test_command_error_display_execution_failed() {
        let err = CommandError::ExecutionFailed("failed".to_string());
        let s = err.to_string();
        assert!(s.contains("Execution failed"));
        assert!(s.contains("failed"));
    }

    #[test]
    fn test_command_error_display_authentication_failed() {
        let err = CommandError::AuthenticationFailed("bad auth".to_string());
        let s = err.to_string();
        assert!(s.contains("Authentication failed"));
        assert!(s.contains("bad auth"));
    }

    #[test]
    fn test_command_error_display_authorization_failed() {
        let err = CommandError::AuthorizationFailed("no perm".to_string());
        let s = err.to_string();
        assert!(s.contains("Authorization failed"));
        assert!(s.contains("no perm"));
    }

    #[test]
    fn test_command_error_display_other() {
        let err = CommandError::Other("misc".to_string());
        let s = err.to_string();
        assert!(s.contains("Error"));
        assert!(s.contains("misc"));
    }

    #[test]
    fn test_command_error_is_std_error() {
        let err = CommandError::NotFound("x".to_string());
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_test_command_new() {
        let cmd = TestCommand::new("name", "desc", "result");
        assert_eq!(cmd.name(), "name");
        assert_eq!(cmd.description(), "desc");
    }

    #[test]
    fn test_test_command_execute_empty_args() {
        let cmd = TestCommand::new("echo", "Echo", "output");
        let result = cmd.execute(vec![]);
        assert!(result.is_ok());
        assert_eq!(result.expect("should succeed"), "output");
    }

    #[test]
    fn test_test_command_execute_with_args() {
        let cmd = TestCommand::new("echo", "Echo", "Echo");
        let result = cmd.execute(vec!["a".to_string(), "b".to_string()]);
        assert!(result.is_ok());
        let s = result.expect("should succeed");
        assert!(s.contains('a'));
        assert!(s.contains('b'));
    }

    #[test]
    fn test_test_command_clone() {
        let cmd = TestCommand::new("x", "y", "z");
        let cloned = cmd.clone();
        assert_eq!(cloned.name(), cmd.name());
    }

    #[test]
    fn test_auth_variants() {
        drop(Auth::User("u".to_string(), "p".to_string()));
        drop(Auth::Token("token".to_string()));
        drop(Auth::ApiKey("key".to_string()));
        drop(Auth::None);
    }

    #[test]
    fn test_auth_clone() {
        let a = Auth::User("u".to_string(), "p".to_string());
        let b = a.clone();
        match (&a, &b) {
            (Auth::User(u1, p1), Auth::User(u2, p2)) => {
                assert_eq!(u1, u2);
                assert_eq!(p1, p2);
            }
            _ => unreachable!("expected User variant"),
        }
    }

    #[test]
    fn test_user_role_variants() {
        assert_eq!(UserRole::Admin, UserRole::Admin);
        assert_eq!(UserRole::PowerUser, UserRole::PowerUser);
        assert_eq!(UserRole::RegularUser, UserRole::RegularUser);
        assert_eq!(UserRole::Guest, UserRole::Guest);
    }

    #[test]
    fn test_user_role_partial_eq() {
        assert!(UserRole::Admin != UserRole::Guest);
        assert!(UserRole::PowerUser == UserRole::PowerUser);
    }

    #[test]
    fn test_auth_user_debug() {
        let user = AuthUser {
            username: "test".to_string(),
            roles: vec![UserRole::Admin],
        };
        let s = format!("{user:?}");
        assert!(!s.is_empty());
    }

    #[test]
    fn test_command_log_entry() {
        let entry = CommandLogEntry {
            command: "cmd".to_string(),
            args: vec!["a".to_string()],
            user: Some("user".to_string()),
            timestamp: std::time::SystemTime::now(),
            success: true,
            message: "ok".to_string(),
        };
        let s = format!("{entry:?}");
        assert!(s.contains("cmd"));
        let cloned = entry.clone();
        assert_eq!(cloned.command, entry.command);
    }
}
