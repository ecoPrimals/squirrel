// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Error types for the commands crate
//!
//! This module defines the errors that can occur in the commands crate.

use std::error::Error;

/// Result type for command operations
pub type Result<T> = std::result::Result<T, CommandError>;

/// Error type for command operations
#[derive(Debug, Clone, thiserror::Error)]
pub enum CommandError {
    /// Error parsing command input
    #[error("Input error: {0}")]
    InputError(String),

    /// Error validating command
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Error executing command
    #[error("Execution error: {0}")]
    ExecutionError(String),

    /// Error with resources
    #[error("Resource error: {0}")]
    ResourceError(String),

    /// Error with permissions
    #[error("Permission error: {0}")]
    PermissionError(String),

    /// Error with IO operations
    #[error("IO error: {0}")]
    IoError(String),

    /// Internal command error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Error with serialization/deserialization
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Error with command registry operations
    #[error("Registry error: {0}")]
    RegistryError(String),

    /// Error when a command is not found
    #[error("Command not found: {0}")]
    CommandNotFound(String),

    /// Error when a command already exists
    #[error("Command already exists: {0}")]
    CommandAlreadyExists(String),

    /// Error with journal operations
    #[error("Journal error: {0}")]
    JournalError(#[from] crate::journal::JournalError),

    /// Error during hook execution
    #[error("Hook error: {0}")]
    Hook(String),

    /// Error acquiring lock
    #[error("Lock error: {0}")]
    Lock(String),

    /// Error during lifecycle stage execution
    #[error("Lifecycle error: {0}")]
    Lifecycle(String),

    /// Error when a resource type is not found
    #[error("Resource type not found: {0}")]
    ResourceNotFound(String),

    /// Error when allocation is not found
    #[error("Allocation error: {0}")]
    Allocation(String),

    /// Custom error type
    #[error("Error: {0}")]
    Other(String),
}

impl From<std::io::Error> for CommandError {
    fn from(err: std::io::Error) -> Self {
        CommandError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for CommandError {
    fn from(err: serde_json::Error) -> Self {
        CommandError::SerializationError(err.to_string())
    }
}

impl From<String> for CommandError {
    fn from(err: String) -> Self {
        CommandError::Other(err)
    }
}

impl From<&str> for CommandError {
    fn from(err: &str) -> Self {
        CommandError::Other(err.to_string())
    }
}

impl From<Box<dyn Error + Send + Sync>> for CommandError {
    fn from(err: Box<dyn Error + Send + Sync>) -> Self {
        CommandError::Other(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::CommandError;
    use std::error::Error;
    use std::io;

    #[test]
    fn command_error_display_covers_variants() {
        let cases: Vec<CommandError> = vec![
            CommandError::InputError("i".into()),
            CommandError::ValidationError("v".into()),
            CommandError::ExecutionError("e".into()),
            CommandError::ResourceError("r".into()),
            CommandError::PermissionError("p".into()),
            CommandError::IoError("io".into()),
            CommandError::Internal("in".into()),
            CommandError::SerializationError("s".into()),
            CommandError::RegistryError("reg".into()),
            CommandError::CommandNotFound("c".into()),
            CommandError::CommandAlreadyExists("c".into()),
            CommandError::Hook("h".into()),
            CommandError::Lock("l".into()),
            CommandError::Lifecycle("lf".into()),
            CommandError::ResourceNotFound("rt".into()),
            CommandError::Allocation("a".into()),
            CommandError::Other("o".into()),
        ];
        for err in cases {
            assert!(!err.to_string().is_empty());
            assert!(!format!("{err:?}").is_empty());
        }
    }

    #[test]
    fn from_io_error() {
        let e: CommandError = io::Error::other("x").into();
        assert!(matches!(e, CommandError::IoError(_)));
    }

    #[test]
    fn from_serde_json_error() {
        let e = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
        let c: CommandError = e.into();
        assert!(matches!(c, CommandError::SerializationError(_)));
    }

    #[test]
    fn from_string_and_str() {
        let a: CommandError = "hello".to_string().into();
        assert!(matches!(a, CommandError::Other(_)));
        let b: CommandError = "hello".into();
        assert!(matches!(b, CommandError::Other(_)));
    }

    #[test]
    fn from_box_error() {
        let boxed: Box<dyn Error + Send + Sync> = Box::new(io::Error::other("boxed"));
        let c: CommandError = boxed.into();
        assert!(matches!(c, CommandError::Other(_)));
    }
}
