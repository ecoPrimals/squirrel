// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Validation Tests
//!
//! Tests for rule validation including:
//! - Required fields (patterns, conditions, actions)
//! - Field constraints (length, format, range)
//! - Content validation (description, version, priority, ID format)

use crate::rules::error::RuleError;
use crate::rules::parser::RuleParser;

/// Test validation: rule must have at least one pattern
#[test]
fn test_validate_fails_without_patterns() {
    let content = r#"---
id: no-pattern-rule
name: No Pattern Rule
description: This rule has no patterns
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
    assert!(result.is_err(), "Should fail without patterns");

    match result.unwrap_err() {
        RuleError::ParseError(msg) => {
            assert!(
                msg.contains("pattern"),
                "Error should mention pattern requirement"
            );
        }
        _ => panic!("Expected ParseError"),
    }
}

/// Test validation: rule must have at least one condition
#[test]
fn test_validate_fails_without_conditions() {
    let content = r#"---
id: no-condition-rule
name: No Condition Rule
description: This rule has no conditions
patterns:
  - "test_pattern"
---

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
    assert!(result.is_err(), "Should fail without conditions");

    match result.unwrap_err() {
        RuleError::ValidationError(msg) => {
            assert!(
                msg.contains("condition"),
                "Error should mention condition requirement"
            );
        }
        _ => panic!("Expected ValidationError"),
    }
}

/// Test validation: rule must have at least one action
#[test]
fn test_validate_fails_without_actions() {
    let content = r#"---
id: no-action-rule
name: No Action Rule
description: This rule has no actions
patterns:
  - "test_pattern"
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
"#;

    let result = RuleParser::parse_string(content);
    assert!(result.is_err(), "Should fail without actions");

    match result.unwrap_err() {
        RuleError::ValidationError(msg) => {
            assert!(
                msg.contains("action"),
                "Error should mention action requirement"
            );
        }
        _ => panic!("Expected ValidationError"),
    }
}

/// Test validation: empty description
#[test]
fn test_validate_fails_empty_description() {
    let content = r#"---
id: test-rule
name: Test Rule
description: ""
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
    assert!(result.is_err(), "Should fail with empty description");
}

/// Test validation: invalid version (no digits)
#[test]
fn test_validate_fails_version_without_digits() {
    let content = r#"---
id: test-rule
name: Test Rule
description: Test
version: "abc"
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
    assert!(result.is_err(), "Should fail with version without digits");
}

/// Test validation: negative priority
#[test]
fn test_validate_fails_negative_priority() {
    let content = r#"---
id: test-rule
name: Test Rule
description: Test
priority: -1
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
    assert!(result.is_err(), "Should fail with negative priority");

    match result.unwrap_err() {
        RuleError::ValidationError(msg) => {
            assert!(msg.contains("priority"), "Error should mention priority");
        }
        _ => panic!("Expected ValidationError"),
    }
}

/// Test validation: priority too large
#[test]
fn test_validate_fails_priority_too_large() {
    let content = r#"---
id: test-rule
name: Test Rule
description: Test
priority: 10001
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
    assert!(result.is_err(), "Should fail with priority > 10000");
}

/// Test validation: pattern too short (< 2 chars)
#[test]
fn test_validate_fails_pattern_too_short() {
    let content = r#"---
id: test-rule
name: Test Rule
description: Test
patterns:
  - "a"
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
    assert!(result.is_err(), "Should fail with pattern < 2 chars");

    match result.unwrap_err() {
        RuleError::ValidationError(msg) => {
            assert!(
                msg.contains("2 characters"),
                "Error should mention minimum length"
            );
        }
        _ => panic!("Expected ValidationError"),
    }
}

/// Test validation: invalid ID format (special characters)
#[test]
fn test_validate_fails_invalid_id_format() {
    let content = r#"---
id: "test@rule#123"
name: Test Rule
description: Test
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
    assert!(result.is_err(), "Should fail with invalid ID format");

    match result.unwrap_err() {
        RuleError::ValidationError(msg) => {
            assert!(
                msg.contains("alphanumeric"),
                "Error should mention alphanumeric requirement"
            );
        }
        _ => panic!("Expected ValidationError"),
    }
}

/// Test validation: name too long (> 100 chars)
#[test]
fn test_validate_fails_name_too_long() {
    let long_name = "a".repeat(101);
    let content = format!(
        r#"---
id: test-rule
name: "{long_name}"
description: Test
patterns:
  - "test"
---

## Conditions

```json
{{
  "type": "match",
  "config": {{
    "path": "test",
    "pattern": ".*"
  }}
}}
```

## Actions

```json
{{
  "type": "log",
  "config": {{
    "level": "info",
    "message": "test"
  }}
}}
```
"#
    );

    let result = RuleParser::parse_string(&content);
    assert!(result.is_err(), "Should fail with name > 100 chars");

    match result.unwrap_err() {
        RuleError::ValidationError(msg) => {
            assert!(
                msg.contains("100 characters"),
                "Error should mention length limit"
            );
        }
        _ => panic!("Expected ValidationError"),
    }
}

/// Test validation: description too long (> 1000 chars)
#[test]
fn test_validate_fails_description_too_long() {
    let long_desc = "a".repeat(1001);
    let content = format!(
        r#"---
id: test-rule
name: Test Rule
description: "{long_desc}"
patterns:
  - "test"
---

## Conditions

```json
{{
  "type": "match",
  "config": {{
    "path": "test",
    "pattern": ".*"
  }}
}}
```

## Actions

```json
{{
  "type": "log",
  "config": {{
    "level": "info",
    "message": "test"
  }}
}}
```
"#
    );

    let result = RuleParser::parse_string(&content);
    assert!(result.is_err(), "Should fail with description > 1000 chars");
}
