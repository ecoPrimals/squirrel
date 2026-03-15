// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Advanced Features Tests
//!
//! Tests for advanced parser features including:
//! - Metadata support
//! - Multiple conditions and actions
//! - Custom configuration (category, priority, version)
//! - Error handling for invalid JSON

use crate::rules::error::RuleError;
use crate::rules::parser::RuleParser;

/// Test parsing with metadata in frontmatter
#[test]
fn test_parse_with_metadata() {
    let content = r#"---
id: metadata-rule
name: Metadata Rule
description: Rule with custom metadata
patterns:
  - "test"
metadata:
  author: "test_user"
  created: "2025-11-15"
  tags:
    - "production"
    - "critical"
---

## Conditions

```json
{
  "type": "match",
  "config": {
    "path": "test",
    "pattern": ".*"
  }
}
```

## Actions

```json
{
  "type": "log",
  "config": {
    "level": "info",
    "message": "test"
  }
}
```
"#;

    let result = RuleParser::parse_string(content);
    assert!(result.is_ok(), "Should parse with metadata");

    let rule = result.expect("test: should succeed");
    assert!(
        rule.metadata.get("author").is_some(),
        "Should have author metadata"
    );
}

/// Test parsing with multiple conditions
#[test]
fn test_parse_with_multiple_conditions() {
    let content = r#"---
id: multi-condition-rule
name: Multi Condition Rule
description: Rule with multiple conditions
patterns:
  - "test"
---

## Conditions

```json
{
  "type": "match",
  "config": {
    "path": "test.first",
    "pattern": ".*"
  }
}
```

```json
{
  "type": "exists",
  "config": {
    "path": "test.second"
  }
}
```

## Actions

```json
{
  "type": "log",
  "config": {
    "level": "info",
    "message": "test"
  }
}
```
"#;

    let result = RuleParser::parse_string(content);
    assert!(result.is_ok(), "Should parse with multiple conditions");

    let rule = result.expect("test: should succeed");
    assert_eq!(rule.conditions.len(), 2, "Should have 2 conditions");
}

/// Test parsing with multiple actions
#[test]
fn test_parse_with_multiple_actions() {
    let content = r#"---
id: multi-action-rule
name: Multi Action Rule
description: Rule with multiple actions
patterns:
  - "test"
---

## Conditions

```json
{
  "type": "match",
  "config": {
    "path": "test",
    "pattern": ".*"
  }
}
```

## Actions

```json
{
  "type": "log",
  "config": {
    "level": "info",
    "message": "First action"
  }
}
```

```json
{
  "type": "log",
  "config": {
    "level": "warn",
    "message": "Second action"
  }
}
```
"#;

    let result = RuleParser::parse_string(content);
    assert!(result.is_ok(), "Should parse with multiple actions");

    let rule = result.expect("test: should succeed");
    assert_eq!(rule.actions.len(), 2, "Should have 2 actions");
}

/// Test parsing with invalid JSON in conditions section
#[test]
fn test_parse_fails_invalid_condition_json() {
    let content = r#"---
id: bad-json-rule
name: Bad JSON Rule
description: Rule with invalid condition JSON
patterns:
  - "test"
---

## Conditions

```json
{
  "type": "match",
  "config": {
    "path": "test",
    "pattern": ".*",
  }
}
```

## Actions

```json
{
  "type": "log",
  "config": {
    "level": "info",
    "message": "test"
  }
}
```
"#;

    let result = RuleParser::parse_string(content);
    assert!(result.is_err(), "Should fail with invalid JSON");

    match result.unwrap_err() {
        RuleError::ParseError(msg) => {
            assert!(
                msg.contains("condition"),
                "Error should mention condition parsing"
            );
        }
        _ => panic!("Expected ParseError"),
    }
}

/// Test parsing with invalid JSON in actions section
#[test]
fn test_parse_fails_invalid_action_json() {
    let content = r#"---
id: bad-action-json-rule
name: Bad Action JSON Rule
description: Rule with invalid action JSON
patterns:
  - "test"
---

## Conditions

```json
{
  "type": "match",
  "config": {
    "path": "test",
    "pattern": ".*"
  }
}
```

## Actions

```json
{
  "type": "log",
  "config": {
    "level": "info",
    "message": "test",
  }
}
```
"#;

    let result = RuleParser::parse_string(content);
    assert!(result.is_err(), "Should fail with invalid action JSON");

    match result.unwrap_err() {
        RuleError::ParseError(msg) => {
            assert!(
                msg.contains("action"),
                "Error should mention action parsing"
            );
        }
        _ => panic!("Expected ParseError"),
    }
}

/// Test parsing rule with custom category
#[test]
fn test_parse_with_custom_category() {
    let content = r#"---
id: custom-cat-rule
name: Custom Category Rule
description: Rule with custom category
category: "security"
patterns:
  - "test"
---

## Conditions

```json
{
  "type": "match",
  "config": {
    "path": "test",
    "pattern": ".*"
  }
}
```

## Actions

```json
{
  "type": "log",
  "config": {
    "level": "info",
    "message": "test"
  }
}
```
"#;

    let result = RuleParser::parse_string(content);
    assert!(result.is_ok(), "Should parse with custom category");

    let rule = result.expect("test: should succeed");
    assert_eq!(rule.category, "security");
}

/// Test parsing rule with custom priority
#[test]
fn test_parse_with_custom_priority() {
    let content = r#"---
id: priority-rule
name: Priority Rule
description: Rule with custom priority
priority: 500
patterns:
  - "test"
---

## Conditions

```json
{
  "type": "match",
  "config": {
    "path": "test",
    "pattern": ".*"
  }
}
```

## Actions

```json
{
  "type": "log",
  "config": {
    "level": "info",
    "message": "test"
  }
}
```
"#;

    let result = RuleParser::parse_string(content);
    assert!(result.is_ok(), "Should parse with custom priority");

    let rule = result.expect("test: should succeed");
    assert_eq!(rule.priority, 500);
}

/// Test parsing rule with custom version
#[test]
fn test_parse_with_custom_version() {
    let content = r#"---
id: version-rule
name: Version Rule
description: Rule with custom version
version: "2.3.1"
patterns:
  - "test"
---

## Conditions

```json
{
  "type": "match",
  "config": {
    "path": "test",
    "pattern": ".*"
  }
}
```

## Actions

```json
{
  "type": "log",
  "config": {
    "level": "info",
    "message": "test"
  }
}
```
"#;

    let result = RuleParser::parse_string(content);
    assert!(result.is_ok(), "Should parse with custom version");

    let rule = result.expect("test: should succeed");
    assert_eq!(rule.version, "2.3.1");
}
