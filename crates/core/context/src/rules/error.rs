// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Error types for the rules module
use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Result type for rule operations
pub type Result<T> = std::result::Result<T, RuleError>;

/// Error type for rule operations
#[derive(Debug, Error)]
pub enum RuleError {
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    /// Serialization/Deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    /// Invalid rule format
    #[error("Invalid rule format: {0}")]
    InvalidFormat(String),
    /// Rule not found
    #[error("Rule not found: {0}")]
    NotFound(String),
    /// Rule already exists
    #[error("Rule already exists: {0}")]
    AlreadyExists(String),
    /// Directory operation error
    #[error("Directory operation error: {0}")]
    DirectoryError(String),
    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),
    /// Plugin error
    #[error("Plugin error: {0}")]
    PluginError(String),
    /// Plugin not found
    #[error("Plugin not found: {0}")]
    PluginNotFound(String),
    /// Evaluation error
    #[error("Evaluation error: {0}")]
    EvaluationError(String),
    /// Action execution error
    #[error("Action execution error: {0}")]
    ActionExecutionError(String),
    /// Invalid path error
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    /// Invalid type error
    #[error("Invalid type: {0}")]
    InvalidType(String),
    /// Rule parse error
    #[error("Parse error: {0}")]
    ParseError(String),
    /// Rule dependency error
    #[error("Dependency error: {0}")]
    DependencyError(String),
    /// Rule circular dependency error
    #[error("Circular dependency error: {0}")]
    CircularDependencyError(String),
    /// Rule validation error
    #[error("Rule validation error for '{rule_id}': {}", errors.join(", "))]
    RuleValidationError {
        /// Rule ID
        rule_id: String,
        /// List of validation errors
        errors: Vec<String>,
    },
    /// Path not found in file system
    #[error("Path not found: {}", .0.display())]
    PathNotFound(PathBuf),
}

impl From<serde_json::Error> for RuleError {
    fn from(error: serde_json::Error) -> Self {
        RuleError::SerializationError(error.to_string())
    }
}

impl From<serde_yaml_ng::Error> for RuleError {
    fn from(error: serde_yaml_ng::Error) -> Self {
        RuleError::SerializationError(error.to_string())
    }
}

#[cfg(test)]
#[expect(
    clippy::unnecessary_literal_unwrap,
    reason = "Tests intentionally create Result types"
)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_io_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err = RuleError::from(io_err);
        assert!(err.to_string().contains("IO error"));
        assert!(err.to_string().contains("file not found"));
    }

    #[test]
    fn test_serialization_error() {
        let err = RuleError::SerializationError("invalid json".to_string());
        assert_eq!(err.to_string(), "Serialization error: invalid json");
    }

    #[test]
    fn test_invalid_format() {
        let err = RuleError::InvalidFormat("bad syntax".to_string());
        assert_eq!(err.to_string(), "Invalid rule format: bad syntax");
    }

    #[test]
    fn test_not_found() {
        let err = RuleError::NotFound("rule-123".to_string());
        assert_eq!(err.to_string(), "Rule not found: rule-123");
    }

    #[test]
    fn test_already_exists() {
        let err = RuleError::AlreadyExists("rule-456".to_string());
        assert_eq!(err.to_string(), "Rule already exists: rule-456");
    }

    #[test]
    fn test_directory_error() {
        let err = RuleError::DirectoryError("cannot create".to_string());
        assert_eq!(err.to_string(), "Directory operation error: cannot create");
    }

    #[test]
    fn test_validation_error() {
        let err = RuleError::ValidationError("invalid field".to_string());
        assert_eq!(err.to_string(), "Validation error: invalid field");
    }

    #[test]
    fn test_plugin_error() {
        let err = RuleError::PluginError("plugin failed".to_string());
        assert_eq!(err.to_string(), "Plugin error: plugin failed");
    }

    #[test]
    fn test_plugin_not_found() {
        let err = RuleError::PluginNotFound("my-plugin".to_string());
        assert_eq!(err.to_string(), "Plugin not found: my-plugin");
    }

    #[test]
    fn test_evaluation_error() {
        let err = RuleError::EvaluationError("condition failed".to_string());
        assert_eq!(err.to_string(), "Evaluation error: condition failed");
    }

    #[test]
    fn test_action_execution_error() {
        let err = RuleError::ActionExecutionError("action failed".to_string());
        assert_eq!(err.to_string(), "Action execution error: action failed");
    }

    #[test]
    fn test_invalid_path() {
        let err = RuleError::InvalidPath("/bad/path".to_string());
        assert_eq!(err.to_string(), "Invalid path: /bad/path");
    }

    #[test]
    fn test_invalid_type() {
        let err = RuleError::InvalidType("expected string".to_string());
        assert_eq!(err.to_string(), "Invalid type: expected string");
    }

    #[test]
    fn test_parse_error() {
        let err = RuleError::ParseError("syntax error".to_string());
        assert_eq!(err.to_string(), "Parse error: syntax error");
    }

    #[test]
    fn test_dependency_error() {
        let err = RuleError::DependencyError("missing dependency".to_string());
        assert_eq!(err.to_string(), "Dependency error: missing dependency");
    }

    #[test]
    fn test_circular_dependency_error() {
        let err = RuleError::CircularDependencyError("A -> B -> A".to_string());
        assert_eq!(err.to_string(), "Circular dependency error: A -> B -> A");
    }

    #[test]
    fn test_rule_validation_error_single() {
        let err = RuleError::RuleValidationError {
            rule_id: "test-rule".to_string(),
            errors: vec!["error1".to_string()],
        };
        assert_eq!(
            err.to_string(),
            "Rule validation error for 'test-rule': error1"
        );
    }

    #[test]
    fn test_rule_validation_error_multiple() {
        let err = RuleError::RuleValidationError {
            rule_id: "test-rule".to_string(),
            errors: vec![
                "error1".to_string(),
                "error2".to_string(),
                "error3".to_string(),
            ],
        };
        let msg = err.to_string();
        assert!(msg.contains("test-rule"));
        assert!(msg.contains("error1"));
        assert!(msg.contains("error2"));
        assert!(msg.contains("error3"));
    }

    #[test]
    fn test_path_not_found() {
        let path = PathBuf::from("/missing/file.txt");
        let err = RuleError::PathNotFound(path);
        assert!(err.to_string().contains("Path not found"));
        assert!(err.to_string().contains("missing"));
    }

    #[test]
    fn test_from_serde_json() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let rule_err: RuleError = json_err.into();
        assert!(matches!(rule_err, RuleError::SerializationError(_)));
    }

    #[test]
    fn test_debug_format() {
        let err = RuleError::NotFound("test".to_string());
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("NotFound"));
    }

    #[test]
    fn test_result_type_ok() {
        let result: Result<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.expect("ok"), 42);
    }

    #[test]
    fn test_result_type_err() {
        let result: Result<i32> = Err(RuleError::NotFound("test".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_error_trait() {
        let err = RuleError::NotFound("test".to_string());
        // Test that it implements std::error::Error
        let _: &dyn std::error::Error = &err;
    }
}
