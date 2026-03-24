// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Basic Parsing Tests
//!
//! Tests for core parsing functionality including:
//! - Valid minimal rules
//! - Multiple patterns
//! - Frontmatter extraction
//! - Required field validation
//! - Default value application

use crate::rules::error::RuleError;
use crate::rules::parser::{FrontmatterParser, RuleParser};

/// Test parsing a valid minimal rule with all required fields
#[test]
fn test_parse_valid_minimal_rule() {
    let content = r#"---
id: test-rule-001
name: Test Rule
description: A minimal test rule
patterns:
  - "test_pattern"
---

## Conditions

```json
{
  "type": "match",
  "config": {
    "path": "test.value",
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
    "message": "Test action executed"
  }
}
```
"#;

    let result = RuleParser::parse_string(content);
    if let Err(ref e) = result {
        eprintln!("Parse error: {:?}", e);
    }
    assert!(
        result.is_ok(),
        "Should successfully parse valid minimal rule"
    );

    let rule = result.expect("test: should succeed");
    assert_eq!(rule.id, "test-rule-001");
    assert_eq!(rule.name, "Test Rule");
    assert_eq!(rule.description, "A minimal test rule");
    assert_eq!(rule.patterns.len(), 1);
    assert_eq!(rule.patterns[0], "test_pattern");
    assert_eq!(rule.conditions.len(), 1);
    assert_eq!(rule.actions.len(), 1);
}

/// Test parsing a rule with multiple patterns
#[test]
fn test_parse_rule_with_multiple_patterns() {
    let content = r#"---
id: multi-pattern-rule
name: Multi Pattern Rule
description: Rule with multiple patterns
patterns:
  - "pattern_one"
  - "pattern_two"
  - "pattern_three"
---

## Conditions

```json
{
  "type": "exists",
  "config": {
    "path": "some.field"
  }
}
```

## Actions

```json
{
  "type": "log",
  "config": {
    "level": "info",
    "message": "Multiple patterns matched"
  }
}
```
"#;

    let result = RuleParser::parse_string(content);
    assert!(result.is_ok(), "Should parse rule with multiple patterns");

    let rule = result.expect("test: should succeed");
    assert_eq!(rule.patterns.len(), 3);
    assert!(rule.patterns.contains(&"pattern_one".to_string()));
    assert!(rule.patterns.contains(&"pattern_two".to_string()));
    assert!(rule.patterns.contains(&"pattern_three".to_string()));
}

/// Test that parsing fails when frontmatter is missing
#[test]
fn test_parse_fails_without_frontmatter() {
    let content = "This is just plain text without frontmatter";

    let result = RuleParser::parse_string(content);
    assert!(result.is_err(), "Should fail without frontmatter");

    match result.unwrap_err() {
        RuleError::ParseError(msg) => {
            assert!(
                msg.contains("frontmatter"),
                "Error should mention frontmatter"
            );
        }
        _ => unreachable!("Expected ParseError"),
    }
}

/// Test that parsing fails when required field 'id' is missing
#[test]
fn test_parse_fails_without_required_id() {
    let content = r#"---
name: Test Rule
description: Missing ID
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
    assert!(result.is_err(), "Should fail without required 'id' field");

    match result.unwrap_err() {
        RuleError::ParseError(msg) => {
            assert!(msg.contains("id"), "Error should mention missing 'id'");
        }
        _ => unreachable!("Expected ParseError"),
    }
}

/// Test that parsing fails when required field 'name' is missing
#[test]
fn test_parse_fails_without_required_name() {
    let content = r#"---
id: test-rule
description: Missing name
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
    assert!(result.is_err(), "Should fail without required 'name' field");

    match result.unwrap_err() {
        RuleError::ParseError(msg) => {
            assert!(msg.contains("name"), "Error should mention missing 'name'");
        }
        _ => unreachable!("Expected ParseError"),
    }
}

/// Test that default values are applied correctly
#[test]
fn test_parse_applies_default_values() {
    let content = r#"---
id: defaults-rule
name: Defaults Rule
description: Testing default values
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
    assert!(result.is_ok(), "Should parse and apply defaults");

    let rule = result.expect("test: should succeed");
    // Default version should be "1.0.0"
    assert_eq!(rule.version, "1.0.0");
    // Default category should be "default"
    assert_eq!(rule.category, "default");
    // Default priority should be 0
    assert_eq!(rule.priority, 0);
}

/// Test frontmatter extraction helper
#[test]
fn test_frontmatter_extraction() {
    let content_with_frontmatter = r"---
id: test
name: Test
---
Body content here";

    let result = FrontmatterParser::extract_frontmatter(content_with_frontmatter);
    assert!(result.is_ok());

    let (frontmatter_opt, remaining) = result.expect("test: should succeed");
    assert!(frontmatter_opt.is_some());
    assert!(remaining.contains("Body content"));

    // Test content without frontmatter
    let content_without = "No frontmatter here";
    let result = FrontmatterParser::extract_frontmatter(content_without);
    assert!(result.is_ok());

    let (frontmatter_opt, remaining) = result.expect("test: should succeed");
    assert!(frontmatter_opt.is_none());
    assert_eq!(remaining, content_without);
}

/// Test parsing with single 'pattern' field instead of 'patterns' array
#[test]
fn test_parse_with_single_pattern_field() {
    let content = r#"---
id: single-pattern-rule
name: Single Pattern Rule
description: Uses single pattern field
pattern: "single_pattern"
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
    assert!(result.is_ok(), "Should parse with single 'pattern' field");

    let rule = result.expect("test: should succeed");
    assert_eq!(rule.patterns.len(), 1);
    assert_eq!(rule.patterns[0], "single_pattern");
}
