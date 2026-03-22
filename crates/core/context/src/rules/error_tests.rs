// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for the rules error module

use super::error::{Result, RuleError};
use std::io;
use std::path::PathBuf;

#[test]
fn test_io_error_creation() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let rule_err = RuleError::from(io_err);
    assert!(matches!(rule_err, RuleError::IoError(_)));
}

#[test]
fn test_io_error_display() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let rule_err = RuleError::from(io_err);
    let display = format!("{}", rule_err);
    assert!(display.contains("IO error"));
}

#[test]
fn test_serialization_error_creation() {
    let err = RuleError::SerializationError("invalid JSON".to_string());
    assert!(matches!(err, RuleError::SerializationError(_)));
}

#[test]
fn test_serialization_error_display() {
    let err = RuleError::SerializationError("invalid JSON".to_string());
    let display = format!("{}", err);
    assert_eq!(display, "Serialization error: invalid JSON");
}

#[test]
fn test_serde_json_error_conversion() {
    let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
    let rule_err = RuleError::from(json_err);
    assert!(matches!(rule_err, RuleError::SerializationError(_)));
}

#[test]
fn test_serde_yaml_ng_error_conversion() {
    let yaml_err = serde_yaml_ng::from_str::<serde_yaml_ng::Value>("invalid: [").unwrap_err();
    let rule_err = RuleError::from(yaml_err);
    assert!(matches!(rule_err, RuleError::SerializationError(_)));
}

#[test]
fn test_invalid_format_error() {
    let err = RuleError::InvalidFormat("missing required field".to_string());
    let display = format!("{}", err);
    assert_eq!(display, "Invalid rule format: missing required field");
}

#[test]
fn test_not_found_error() {
    let err = RuleError::NotFound("rule123".to_string());
    let display = format!("{}", err);
    assert_eq!(display, "Rule not found: rule123");
}

#[test]
fn test_already_exists_error() {
    let err = RuleError::AlreadyExists("rule456".to_string());
    let display = format!("{}", err);
    assert_eq!(display, "Rule already exists: rule456");
}

#[test]
fn test_directory_error() {
    let err = RuleError::DirectoryError("cannot create directory".to_string());
    let display = format!("{}", err);
    assert_eq!(
        display,
        "Directory operation error: cannot create directory"
    );
}

#[test]
fn test_validation_error() {
    let err = RuleError::ValidationError("invalid pattern".to_string());
    let display = format!("{}", err);
    assert_eq!(display, "Validation error: invalid pattern");
}

#[test]
fn test_plugin_error() {
    let err = RuleError::PluginError("plugin initialization failed".to_string());
    let display = format!("{}", err);
    assert_eq!(display, "Plugin error: plugin initialization failed");
}

#[test]
fn test_plugin_not_found_error() {
    let err = RuleError::PluginNotFound("my-plugin".to_string());
    let display = format!("{}", err);
    assert_eq!(display, "Plugin not found: my-plugin");
}

#[test]
fn test_evaluation_error() {
    let err = RuleError::EvaluationError("condition failed".to_string());
    let display = format!("{}", err);
    assert_eq!(display, "Evaluation error: condition failed");
}

#[test]
fn test_action_execution_error() {
    let err = RuleError::ActionExecutionError("action timeout".to_string());
    let display = format!("{}", err);
    assert_eq!(display, "Action execution error: action timeout");
}

#[test]
fn test_invalid_path_error() {
    let err = RuleError::InvalidPath("/invalid/path".to_string());
    let display = format!("{}", err);
    assert_eq!(display, "Invalid path: /invalid/path");
}

#[test]
fn test_invalid_type_error() {
    let err = RuleError::InvalidType("expected string, got number".to_string());
    let display = format!("{}", err);
    assert_eq!(display, "Invalid type: expected string, got number");
}

#[test]
fn test_parse_error() {
    let err = RuleError::ParseError("syntax error at line 5".to_string());
    let display = format!("{}", err);
    assert_eq!(display, "Parse error: syntax error at line 5");
}

#[test]
fn test_dependency_error() {
    let err = RuleError::DependencyError("missing dependency: rule789".to_string());
    let display = format!("{}", err);
    assert_eq!(display, "Dependency error: missing dependency: rule789");
}

#[test]
fn test_circular_dependency_error() {
    let err = RuleError::CircularDependencyError("rule1 -> rule2 -> rule1".to_string());
    let display = format!("{}", err);
    assert_eq!(
        display,
        "Circular dependency error: rule1 -> rule2 -> rule1"
    );
}

#[test]
fn test_rule_validation_error_single() {
    let err = RuleError::RuleValidationError {
        rule_id: "test-rule".to_string(),
        errors: vec!["missing name".to_string()],
    };
    let display = format!("{}", err);
    assert_eq!(
        display,
        "Rule validation error for 'test-rule': missing name"
    );
}

#[test]
fn test_rule_validation_error_multiple() {
    let err = RuleError::RuleValidationError {
        rule_id: "test-rule".to_string(),
        errors: vec![
            "missing name".to_string(),
            "invalid pattern".to_string(),
            "missing action".to_string(),
        ],
    };
    let display = format!("{}", err);
    assert_eq!(
        display,
        "Rule validation error for 'test-rule': missing name, invalid pattern, missing action"
    );
}

#[test]
fn test_rule_validation_error_empty() {
    let err = RuleError::RuleValidationError {
        rule_id: "test-rule".to_string(),
        errors: vec![],
    };
    let display = format!("{}", err);
    assert_eq!(display, "Rule validation error for 'test-rule': ");
}

#[test]
fn test_path_not_found_error() {
    let path = PathBuf::from("/nonexistent/file.json");
    let err = RuleError::PathNotFound(path.clone());
    let display = format!("{}", err);
    assert!(display.contains("Path not found"));
    assert!(display.contains("/nonexistent/file.json"));
}

#[test]
fn test_error_trait_implementation() {
    let err = RuleError::NotFound("test".to_string());
    let _: &dyn std::error::Error = &err;
}

#[test]
fn test_debug_implementation() {
    let err = RuleError::NotFound("test".to_string());
    let debug = format!("{:?}", err);
    assert!(debug.contains("NotFound"));
}

#[test]
fn test_result_type_ok() {
    let result: Result<i32> = Ok(42);
    assert!(result.is_ok());
    if let Ok(value) = result {
        assert_eq!(value, 42);
    }
}

#[test]
fn test_result_type_err() {
    let result: Result<i32> = Err(RuleError::NotFound("test".to_string()));
    assert!(result.is_err());
}

#[test]
fn test_error_pattern_matching() {
    let errors = vec![
        RuleError::NotFound("test".to_string()),
        RuleError::AlreadyExists("test".to_string()),
        RuleError::ValidationError("test".to_string()),
    ];

    for err in errors {
        match err {
            RuleError::NotFound(_) => { /* expected */ }
            RuleError::AlreadyExists(_) => { /* expected */ }
            RuleError::ValidationError(_) => { /* expected */ }
            _ => panic!("Unexpected error variant"),
        }
    }
}

#[test]
fn test_all_error_variants_display() {
    // Ensure all error variants have Display implementation
    let errors = vec![
        RuleError::IoError(io::Error::from(io::ErrorKind::Other)),
        RuleError::SerializationError("test".to_string()),
        RuleError::InvalidFormat("test".to_string()),
        RuleError::NotFound("test".to_string()),
        RuleError::AlreadyExists("test".to_string()),
        RuleError::DirectoryError("test".to_string()),
        RuleError::ValidationError("test".to_string()),
        RuleError::PluginError("test".to_string()),
        RuleError::PluginNotFound("test".to_string()),
        RuleError::EvaluationError("test".to_string()),
        RuleError::ActionExecutionError("test".to_string()),
        RuleError::InvalidPath("test".to_string()),
        RuleError::InvalidType("test".to_string()),
        RuleError::ParseError("test".to_string()),
        RuleError::DependencyError("test".to_string()),
        RuleError::CircularDependencyError("test".to_string()),
        RuleError::RuleValidationError {
            rule_id: "test".to_string(),
            errors: vec!["error".to_string()],
        },
        RuleError::PathNotFound(PathBuf::from("/test")),
    ];

    for err in errors {
        let display = format!("{}", err);
        assert!(!display.is_empty(), "Display should not be empty");
    }
}
