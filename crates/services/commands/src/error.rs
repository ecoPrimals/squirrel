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
