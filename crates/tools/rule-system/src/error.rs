// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Error types for the rule system

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur in rule system operations
#[derive(Debug, Error)]
pub enum RuleSystemError {
    /// Rule directory not found
    #[error("Rule directory not found: {0}")]
    DirectoryNotFound(PathBuf),

    /// Rule file not found
    #[error("Rule file not found: {0}")]
    FileNotFound(PathBuf),

    /// Rule parsing error
    #[error("Failed to parse rule: {0}")]
    ParseError(#[from] RuleParserError),

    /// Rule validation error
    #[error("Failed to validate rule: {0}")]
    ValidationError(#[from] RuleValidationError),

    /// Rule repository error
    #[error("Rule repository error: {0}")]
    RepositoryError(#[from] RuleRepositoryError),

    /// Rule manager error
    #[error("Rule manager error: {0}")]
    ManagerError(#[from] RuleManagerError),

    /// Rule evaluator error
    #[error("Rule evaluator error: {0}")]
    EvaluatorError(#[from] RuleEvaluatorError),

    /// Rule action error
    #[error("Rule action error: {0}")]
    ActionError(#[from] RuleActionError),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Rule not found
    #[error("Rule not found: {0}")]
    RuleNotFound(String),

    /// Rule already exists
    #[error("Rule already exists: {0}")]
    RuleAlreadyExists(String),

    /// File watcher error
    #[error("File watcher error: {0}")]
    WatcherError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

/// Errors that can occur during rule parsing
#[derive(Debug, Error)]
pub enum RuleParserError {
    /// Invalid frontmatter format
    #[error("Invalid frontmatter format: {0}")]
    InvalidFrontmatter(String),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Invalid field value
    #[error("Invalid field value for {field}: {reason}")]
    InvalidFieldValue {
        /// Field name
        field: String,
        /// Reason for invalid value
        reason: String,
    },

    /// Invalid section format
    #[error("Invalid section format: {0}")]
    InvalidSection(String),

    /// YAML parsing error
    #[error("YAML parsing error: {0}")]
    YamlError(String),

    /// TOML parsing error
    #[error("TOML parsing error: {0}")]
    TomlError(String),

    /// JSON parsing error
    #[error("JSON parsing error: {0}")]
    JsonError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Other parsing error
    #[error("Other parsing error: {0}")]
    Other(String),
}

/// Errors that can occur during rule validation
#[derive(Debug, Error)]
pub enum RuleValidationError {
    /// Rule ID is invalid
    #[error("Invalid rule ID: {0}")]
    InvalidId(String),

    /// Rule name is invalid
    #[error("Invalid rule name: {0}")]
    InvalidName(String),

    /// Rule version is invalid
    #[error("Invalid rule version: {0}")]
    InvalidVersion(String),

    /// Rule category is invalid
    #[error("Invalid rule category: {0}")]
    InvalidCategory(String),

    /// Rule pattern is invalid
    #[error("Invalid rule pattern: {0}")]
    InvalidPattern(String),

    /// Rule condition is invalid
    #[error("Invalid rule condition: {0}")]
    InvalidCondition(String),

    /// Rule action is invalid
    #[error("Invalid rule action: {0}")]
    InvalidAction(String),

    /// Circular dependency detected
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    /// Missing dependency
    #[error("Missing dependency: {0}")]
    MissingDependency(String),

    /// Other validation error
    #[error("Other validation error: {0}")]
    Other(String),
}

/// Errors that can occur in rule repository operations
#[derive(Debug, Error)]
pub enum RuleRepositoryError {
    /// Rule not found
    #[error("Rule not found: {0}")]
    RuleNotFound(String),

    /// Rule already exists
    #[error("Rule already exists: {0}")]
    RuleAlreadyExists(String),

    /// Rule parsing error
    #[error("Failed to parse rule: {0}")]
    ParseError(#[from] RuleParserError),

    /// Directory not found
    #[error("Directory not found: {0}")]
    DirectoryNotFound(PathBuf),

    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    /// File watcher error
    #[error("File watcher error: {0}")]
    WatcherError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Other repository error
    #[error("Other repository error: {0}")]
    Other(String),
}

/// Errors that can occur in rule manager operations
#[derive(Debug, Error)]
pub enum RuleManagerError {
    /// Repository error
    #[error("Repository error: {0}")]
    RepositoryError(#[from] RuleRepositoryError),

    /// Rule not found
    #[error("Rule not found: {0}")]
    RuleNotFound(String),

    /// Dependency resolution error
    #[error("Dependency resolution error: {0}")]
    DependencyError(String),

    /// Plugin error
    #[error("Plugin error: {0}")]
    PluginError(String),

    /// Other manager error
    #[error("Other manager error: {0}")]
    Other(String),
}

/// Errors that can occur during rule evaluation
#[derive(Debug, Error)]
pub enum RuleEvaluatorError {
    /// Rule not found
    #[error("Rule not found: {0}")]
    RuleNotFound(String),

    /// Manager error
    #[error("Manager error: {0}")]
    ManagerError(#[from] RuleManagerError),

    /// Condition evaluation error
    #[error("Condition evaluation error: {0}")]
    ConditionError(String),

    /// Context error
    #[error("Context error: {0}")]
    ContextError(String),

    /// Other evaluator error
    #[error("Other evaluator error: {0}")]
    Other(String),
}

/// Errors that can occur during rule action execution
#[derive(Debug, Error)]
pub enum RuleActionError {
    /// Action not found
    #[error("Action not found: {0}")]
    ActionNotFound(String),

    /// Context modification error
    #[error("Context modification error: {0}")]
    ContextModificationError(String),

    /// External action error
    #[error("External action error: {0}")]
    ExternalActionError(String),

    /// Plugin error
    #[error("Plugin error: {0}")]
    PluginError(String),

    /// Other action error
    #[error("Other action error: {0}")]
    Other(String),
}

/// Result type for rule system operations
pub type RuleSystemResult<T> = std::result::Result<T, RuleSystemError>;
