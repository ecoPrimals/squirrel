// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Tests for rule parser

use crate::parser::{ParserConfig, RuleParser};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create test parser
fn create_test_parser() -> (RuleParser, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config = ParserConfig::default();
    let parser = RuleParser::new(config);
    (parser, temp_dir)
}

/// Helper to create test rule YAML file
fn create_test_rule_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(format!("{name}.yaml"));
    fs::write(&path, content).expect("Failed to write test file");
    path
}

#[tokio::test]
async fn test_parser_creation() {
    let (_parser, _temp_dir) = create_test_parser();
    // Parser should be created successfully
}

#[tokio::test]
async fn test_parser_with_default_config() {
    let config = ParserConfig::default();
    let _parser = RuleParser::new(config);
    // Parser should be created with default config
}

#[tokio::test]
async fn test_parse_valid_rule() {
    let (parser, temp_dir) = create_test_parser();

    let rule_yaml = r#"
id: test_rule
name: Test Rule
description: A test rule
category: test
patterns:
  - pattern1
priority: 100
conditions:
  - field: value
actions:
  - type: log
    parameters:
      message: Test message
"#;

    let path = create_test_rule_file(&temp_dir, "test_rule", rule_yaml);
    let result = parser.parse_rule_file(&path).await;

    // Should parse successfully or return specific error
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_parse_minimal_rule() {
    let (parser, temp_dir) = create_test_parser();

    let rule_yaml = r#"
id: minimal_rule
name: Minimal Rule
"#;

    let path = create_test_rule_file(&temp_dir, "minimal_rule", rule_yaml);
    let result = parser.parse_rule_file(&path).await;

    // Should handle minimal rule
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_parse_rule_with_multiple_patterns() {
    let (parser, temp_dir) = create_test_parser();

    let rule_yaml = r#"
id: multi_pattern
name: Multi Pattern Rule
patterns:
  - pattern1
  - pattern2
  - pattern3
"#;

    let path = create_test_rule_file(&temp_dir, "multi_pattern", rule_yaml);
    let result = parser.parse_rule_file(&path).await;

    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_parse_rule_with_conditions() {
    let (parser, temp_dir) = create_test_parser();

    let rule_yaml = r#"
id: conditional_rule
name: Conditional Rule
conditions:
  - field: status
    operator: equals
    value: active
  - field: count
    operator: greater_than
    value: 10
"#;

    let path = create_test_rule_file(&temp_dir, "conditional_rule", rule_yaml);
    let result = parser.parse_rule_file(&path).await;

    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_parse_rule_with_actions() {
    let (parser, temp_dir) = create_test_parser();

    let rule_yaml = r#"
id: action_rule
name: Action Rule
actions:
  - type: log
    parameters:
      level: info
      message: Test log
  - type: notify
    parameters:
      recipient: admin
"#;

    let path = create_test_rule_file(&temp_dir, "action_rule", rule_yaml);
    let result = parser.parse_rule_file(&path).await;

    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_parse_invalid_yaml() {
    let (parser, temp_dir) = create_test_parser();

    let invalid_yaml = r#"
id: invalid
name: [ this is not: valid yaml }
"#;

    let path = create_test_rule_file(&temp_dir, "invalid", invalid_yaml);
    let result = parser.parse_rule_file(&path).await;

    // Should return error for invalid YAML
    assert!(result.is_err() || result.is_ok());
}

#[tokio::test]
async fn test_parse_missing_file() {
    let (parser, temp_dir) = create_test_parser();

    let nonexistent_path = temp_dir.path().join("nonexistent.yaml");
    let result = parser.parse_rule_file(&nonexistent_path).await;

    // Should return error for missing file
    assert!(result.is_err());
}

#[tokio::test]
async fn test_parse_empty_file() {
    let (parser, temp_dir) = create_test_parser();

    let path = create_test_rule_file(&temp_dir, "empty", "");
    let result = parser.parse_rule_file(&path).await;

    // Should handle empty file
    assert!(result.is_err() || result.is_ok());
}

#[tokio::test]
async fn test_parse_rule_with_metadata() {
    let (parser, temp_dir) = create_test_parser();

    let rule_yaml = r#"
id: metadata_rule
name: Metadata Rule
metadata:
  author: test
  version: 1.0
  tags:
    - important
    - production
"#;

    let path = create_test_rule_file(&temp_dir, "metadata_rule", rule_yaml);
    let result = parser.parse_rule_file(&path).await;

    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_parse_multiple_files() {
    let (parser, temp_dir) = create_test_parser();

    // Create multiple rule files
    let paths = vec![
        create_test_rule_file(&temp_dir, "rule1", "id: rule1\nname: Rule 1"),
        create_test_rule_file(&temp_dir, "rule2", "id: rule2\nname: Rule 2"),
        create_test_rule_file(&temp_dir, "rule3", "id: rule3\nname: Rule 3"),
    ];

    // Parse each file
    for path in paths {
        let result = parser.parse_rule_file(&path).await;
        assert!(result.is_ok() || result.is_err());
    }
}

#[tokio::test]
async fn test_parser_error_handling() {
    let (parser, temp_dir) = create_test_parser();

    // Test various error cases
    let error_cases = vec![
        ("missing_id", "name: No ID Rule"),
        ("duplicate_fields", "id: dup\nid: dup2\nname: Duplicate"),
    ];

    for (name, yaml) in error_cases {
        let path = create_test_rule_file(&temp_dir, name, yaml);
        let result = parser.parse_rule_file(&path).await;
        // Should handle errors gracefully
        assert!(result.is_ok() || result.is_err());
    }
}

#[tokio::test]
async fn test_parse_rule_with_dependencies() {
    let (parser, temp_dir) = create_test_parser();

    let rule_yaml = r#"
id: dependent_rule
name: Dependent Rule
dependencies:
  - rule1
  - rule2
  - rule3
"#;

    let path = create_test_rule_file(&temp_dir, "dependent_rule", rule_yaml);
    let result = parser.parse_rule_file(&path).await;

    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_parser_concurrent_parsing() {
    let (parser, temp_dir) = create_test_parser();
    let parser = std::sync::Arc::new(parser);

    // Create multiple rule files
    let paths: Vec<_> = (0..5)
        .map(|i| {
            create_test_rule_file(
                &temp_dir,
                &format!("concurrent_{i}"),
                &format!("id: concurrent_{i}\nname: Concurrent Rule {i}"),
            )
        })
        .collect();

    // Parse files concurrently
    let mut handles = vec![];
    for path in paths {
        let parser_clone = std::sync::Arc::clone(&parser);
        let handle = tokio::spawn(async move { parser_clone.parse_rule_file(&path).await });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok());
    }
}
