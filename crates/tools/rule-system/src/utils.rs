// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Utility functions for the rule system

use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::RuleSystemResult;
use crate::models::Rule;
use crate::parser;

/// Creates a template for a new rule
///
/// This function creates a template string for a new rule with the given ID,
/// which can be written to a file.
#[must_use]
pub fn create_rule_template(id: &str, name: &str, category: &str) -> String {
    format!(
        r#"---
id: "{id}"
name: "{name}"
description: "Description of the rule"
version: "1.0.0"
category: "{category}"
priority: 100
patterns:
  - "context.*"
dependencies: []
---

## Conditions

- type: Exists
  path: "data.someValue"

## Actions

- type: ModifyContext
  config:
    path: "data.processed"
    value: true

## Notes

This is a sample rule template. Add your notes here.
"#
    )
}

/// Extracts value from a JSON object using a path
///
/// Path format: `key1.key2.key3` or `key1[0].key2`
///
/// # Errors
///
/// Returns an error if the path is invalid or the value cannot be extracted.
#[must_use]
pub fn extract_value_by_path(data: &Value, path: &str) -> Option<Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = data;

    for part in parts {
        // Handle array indexing
        if let Some((key, index)) = parse_array_index(part) {
            // First get the array by key
            if let Some(array) = current.get(key)?.as_array() {
                // Then get the element by index
                current = array.get(index)?;
            } else {
                return None;
            }
        } else {
            // Regular object property
            current = current.get(part)?;
        }
    }

    Some(current.clone())
}

/// Parse array index from a path part (e.g., "items[0]")
fn parse_array_index(part: &str) -> Option<(&str, usize)> {
    let open_bracket = part.find('[')?;
    let close_bracket = part.find(']')?;

    if open_bracket < close_bracket {
        let key = &part[0..open_bracket];
        let index_str = &part[(open_bracket + 1)..close_bracket];
        let index = index_str.parse::<usize>().ok()?;

        Some((key, index))
    } else {
        None
    }
}

/// Sets a value in a JSON object using a path
///
/// Path format: `key1.key2.key3` or `key1[0].key2`
///
/// # Errors
///
/// Returns an error if the path is invalid or the value cannot be set.
pub fn set_value_by_path(data: &mut Value, path: &str, value: Value) -> bool {
    let parts: Vec<&str> = path.split('.').collect();
    set_value_recursive(data, &parts, 0, value)
}

/// Recursive helper for setting a value by path
fn set_value_recursive(data: &mut Value, parts: &[&str], index: usize, value: Value) -> bool {
    if index >= parts.len() {
        // We've reached the end of the path, so set the value
        *data = value;
        return true;
    }

    let part = parts[index];

    // Handle array indexing
    if let Some((key, array_index)) = parse_array_index(part) {
        if let Some(obj) = data.as_object_mut() {
            // Ensure the key exists
            if !obj.contains_key(key) {
                obj.insert(key.to_string(), json!([]));
            }

            // Get the array
            if let Some(array) = obj.get_mut(key).and_then(|v| v.as_array_mut()) {
                // Ensure the array is big enough
                while array.len() <= array_index {
                    array.push(json!({}));
                }

                // Set the value in the array
                return set_value_recursive(&mut array[array_index], parts, index + 1, value);
            }
        }
    } else {
        // Regular object property
        if let Some(obj) = data.as_object_mut() {
            // Ensure the key exists
            if !obj.contains_key(part) {
                if index == parts.len() - 1 {
                    // Last part, insert the value directly
                    obj.insert(part.to_string(), value);
                    return true;
                }
                // Intermediate part, insert an empty object
                obj.insert(part.to_string(), json!({}));
            }

            // Recurse to the next part
            if let Some(next) = obj.get_mut(part) {
                return set_value_recursive(next, parts, index + 1, value);
            }
        }
    }

    false
}

/// Checks if a rule matches a context
///
/// This function checks if a rule should be applied to a given context based on its patterns.
#[must_use]
pub fn rule_matches_context(rule: &Rule, context_id: &str) -> bool {
    rule.patterns.iter().any(|pattern| {
        // Simple glob-like pattern matching
        match_pattern(pattern, context_id)
    })
}

/// Format a timestamp as an RFC3339 string
#[must_use]
pub fn format_timestamp(timestamp: DateTime<Utc>) -> String {
    timestamp.to_rfc3339()
}

/// Parse an RFC3339 string as a timestamp
///
/// # Errors
///
/// Returns an error if the string cannot be parsed.
pub fn parse_timestamp(timestamp: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    DateTime::parse_from_rfc3339(timestamp).map(|dt| dt.with_timezone(&Utc))
}

/// Create a rule file with the given ID and content
///
/// # Errors
///
/// Returns an error if the file cannot be created.
pub async fn create_rule_file(
    directory: impl AsRef<Path>,
    id: &str,
    content: &str,
) -> RuleSystemResult<PathBuf> {
    let dir = directory.as_ref();
    let file_path = dir.join(format!("{id}.mdc"));

    tokio::fs::write(&file_path, content).await?;

    Ok(file_path)
}

/// Load a rule from a file
///
/// # Errors
///
/// Returns an error if the rule cannot be loaded.
pub async fn load_rule(path: impl AsRef<Path>) -> RuleSystemResult<Rule> {
    parser::parse_rule_file(path).await
}

/// Serialize a rule to YAML format
///
/// # Errors
///
/// Returns an error if the rule cannot be serialized.
pub fn serialize_rule_to_yaml(rule: &Rule) -> Result<String, serde_yml::Error> {
    serde_yml::to_string(rule)
}

/// Serialize a rule to JSON format
///
/// # Errors
///
/// Returns an error if the rule cannot be serialized.
pub fn serialize_rule_to_json(rule: &Rule) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(rule)
}

/// Create a map of rule IDs to rules
#[must_use]
pub fn create_rule_map(rules: &[Rule]) -> HashMap<String, Rule> {
    let mut map = HashMap::new();

    for rule in rules {
        map.insert(rule.id.clone(), rule.clone());
    }

    map
}

/// Sort rules by priority (lower values first)
pub fn sort_rules_by_priority(rules: &mut [Rule]) {
    rules.sort_by(|a, b| a.priority.cmp(&b.priority));
}

/// Filter rules by category
#[must_use]
pub fn filter_rules_by_category(rules: &[Rule], category: &str) -> Vec<Rule> {
    rules
        .iter()
        .filter(|rule| rule.category == category)
        .cloned()
        .collect()
}

/// Filter rules by pattern
#[must_use]
pub fn filter_rules_by_pattern(rules: &[Rule], pattern: &str) -> Vec<Rule> {
    rules
        .iter()
        .filter(|rule| rule_matches_context(rule, pattern))
        .cloned()
        .collect()
}

/// Match a pattern against a string
///
/// Supports simple glob patterns with * and ?.
#[must_use]
pub fn match_pattern(pattern: &str, text: &str) -> bool {
    // Convert glob pattern to regex
    let regex_pattern = pattern
        .replace('.', "\\.")
        .replace('*', ".*")
        .replace('?', ".");

    // Try to compile the regex pattern
    match regex::Regex::new(&format!("^{regex_pattern}$")) {
        Ok(regex) => regex.is_match(text),
        Err(_) => {
            // Fallback to exact matching if regex is invalid
            match regex::Regex::new(&format!("^{}$", regex::escape(pattern))) {
                Ok(regex) => regex.is_match(text),
                Err(_) => {
                    // Final fallback: simple string comparison
                    pattern == text
                }
            }
        }
    }
}
