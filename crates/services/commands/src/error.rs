// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Error types for the commands crate
//!
//! This module defines the errors that can occur in the commands crate.

use std::error::Error;
use std::fmt;

/// Result type for command operations
pub type Result<T> = std::result::Result<T, CommandError>;

/// Error type for command operations
#[derive(Debug, Clone)]
pub enum CommandError {
    /// Error parsing command input
    InputError(String),

    /// Error validating command
    ValidationError(String),

    /// Error executing command
    ExecutionError(String),

    /// Error with resources
    ResourceError(String),

    /// Error with permissions
    PermissionError(String),

    /// Error with IO operations
    IoError(String),

    /// Internal command error
    Internal(String),

    /// Error with serialization/deserialization
    SerializationError(String),

    /// Error with command registry operations
    RegistryError(String),

    /// Error when a command is not found
    CommandNotFound(String),

    /// Error when a command already exists
    CommandAlreadyExists(String),

    /// Error with journal operations
    JournalError(crate::journal::JournalError),

    /// Custom error type
    Other(String),
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::InputError(msg) => write!(f, "Input error: {msg}"),
            CommandError::ValidationError(msg) => write!(f, "Validation error: {msg}"),
            CommandError::ExecutionError(msg) => write!(f, "Execution error: {msg}"),
            CommandError::ResourceError(msg) => write!(f, "Resource error: {msg}"),
            CommandError::PermissionError(msg) => write!(f, "Permission error: {msg}"),
            CommandError::IoError(err) => write!(f, "IO error: {err}"),
            CommandError::SerializationError(msg) => write!(f, "Serialization error: {msg}"),
            CommandError::RegistryError(msg) => write!(f, "Registry error: {msg}"),
            CommandError::CommandNotFound(msg) => write!(f, "Command not found error: {msg}"),
            CommandError::CommandAlreadyExists(msg) => {
                write!(f, "Command already exists error: {msg}")
            }
            CommandError::JournalError(err) => write!(f, "Journal error: {err}"),
            CommandError::Internal(msg) => write!(f, "Internal error: {msg}"),
            CommandError::Other(err) => write!(f, "Error: {err}"),
        }
    }
}

impl Error for CommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CommandError::IoError(_) => None,
            CommandError::Other(_) => None,
            _ => None,
        }
    }
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

impl From<crate::journal::JournalError> for CommandError {
    fn from(err: crate::journal::JournalError) -> Self {
        CommandError::JournalError(err)
    }
}
