// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Rule evaluator for evaluating rules against context
use chrono::{DateTime, Utc};
use log::warn;
use parking_lot;
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;

use super::error::{Result, RuleError};
use super::models::{Rule, RuleCondition};

/// Result of a rule evaluation
#[derive(Debug, Clone)]
#[allow(dead_code)] // Kept for future implementation
pub struct EvaluationResult {
    /// Rule ID
    pub rule_id: String,
    /// Context ID
    pub context_id: String,
    /// Whether the rule matched
    pub matches: bool,
    /// Timestamp of the evaluation
    pub timestamp: DateTime<Utc>,
}

/// Cache for evaluation results
#[allow(dead_code)] // Kept for future implementation
type EvaluationCache = HashMap<String, EvaluationResult>;

/// Rule evaluator for matching rules against context and evaluating conditions
#[derive(Debug)]
pub struct RuleEvaluator {
    /// Regular expression cache
    regex_cache: parking_lot::Mutex<std::collections::HashMap<String, Regex>>,
}

impl Default for RuleEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleEvaluator {
    /// Create a new rule evaluator
    pub fn new() -> Self {
        Self {
            regex_cache: parking_lot::Mutex::new(std::collections::HashMap::new()),
        }
    }

    /// Find matching rules for a given context
    pub async fn find_matching_rules(
        &self,
        rules: &[Arc<Rule>],
        context: &Value,
    ) -> Result<Vec<Arc<Rule>>> {
        let mut matching_rules = Vec::new();

        for rule in rules {
            let mut matches = true;

            for condition in &rule.conditions {
                if !self.evaluate_condition(condition, context).await? {
                    matches = false;
                    break;
                }
            }

            if matches {
                matching_rules.push(Arc::clone(rule));
            }
        }

        // Sort by priority (lower is higher priority)
        matching_rules.sort_by_key(|r| r.priority);

        Ok(matching_rules)
    }

    /// Evaluate a rule condition
    pub async fn evaluate_condition(
        &self,
        condition: &RuleCondition,
        context: &Value,
    ) -> Result<bool> {
        match condition {
            RuleCondition::Exists { path } => self.path_exists(path, context),

            RuleCondition::Match { path, pattern } => {
                if let Some(value) = self.get_value_at_path(path, context) {
                    if let Some(value_str) = value.as_str() {
                        return self.match_pattern(value_str, pattern);
                    } else {
                        return self.match_pattern(&value.to_string(), pattern);
                    }
                }

                Ok(false)
            }

            RuleCondition::Compare {
                path1,
                path2,
                operator,
            } => {
                let value1 = self.get_value_at_path(path1, context);
                let value2 = self.get_value_at_path(path2, context);

                match (value1, value2) {
                    (Some(v1), Some(v2)) => self.compare_values(v1, v2, operator),
                    _ => Ok(false),
                }
            }

            RuleCondition::JavaScript {
                expression: _expression,
            } => {
                // JavaScript execution not implemented yet
                warn!("JavaScript condition not implemented");
                Ok(false)
            }

            RuleCondition::Custom {
                id: _id,
                config: _config,
            } => {
                // Custom condition not implemented yet
                warn!("Custom condition not implemented");
                Ok(false)
            }
        }
    }

    /// Check if a path exists in the context
    fn path_exists(&self, path: &str, context: &Value) -> Result<bool> {
        let exists = self.get_value_at_path(path, context).is_some();
        Ok(exists)
    }

    /// Get a value at a path in the context
    fn get_value_at_path<'a>(&self, path: &str, context: &'a Value) -> Option<&'a Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = context;

        for part in parts {
            if let Some(obj) = current.as_object() {
                if let Some(value) = obj.get(part) {
                    current = value;
                } else {
                    return None;
                }
            } else if let Some(array) = current.as_array() {
                if let Ok(index) = part.parse::<usize>() {
                    if index < array.len() {
                        current = &array[index];
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }

        Some(current)
    }

    /// Match a string value against a pattern
    fn match_pattern(&self, value: &str, pattern: &str) -> Result<bool> {
        let regex = {
            let mut cache = self.regex_cache.lock();

            if !cache.contains_key(pattern) {
                let regex = Regex::new(pattern).map_err(|e| {
                    RuleError::EvaluationError(format!("Invalid regex pattern: {e}"))
                })?;
                cache.insert(pattern.to_string(), regex);
            }

            cache
                .get(pattern)
                .ok_or_else(|| {
                    RuleError::EvaluationError("Failed to retrieve regex from cache".to_string())
                })?
                .clone()
        };

        Ok(regex.is_match(value))
    }

    /// Compare two values
    fn compare_values(&self, value1: &Value, value2: &Value, operator: &str) -> Result<bool> {
        match operator {
            "eq" => Ok(value1 == value2),
            "ne" => Ok(value1 != value2),
            "gt" => self.compare_numeric(value1, value2, |a, b| a > b),
            "ge" => self.compare_numeric(value1, value2, |a, b| a >= b),
            "lt" => self.compare_numeric(value1, value2, |a, b| a < b),
            "le" => self.compare_numeric(value1, value2, |a, b| a <= b),
            _ => Err(RuleError::EvaluationError(format!(
                "Unknown operator: {operator}"
            ))),
        }
    }

    /// Compare numeric values
    fn compare_numeric<F>(&self, value1: &Value, value2: &Value, compare_fn: F) -> Result<bool>
    where
        F: Fn(f64, f64) -> bool,
    {
        let num1 = self.to_number(value1)?;
        let num2 = self.to_number(value2)?;

        Ok(compare_fn(num1, num2))
    }

    /// Convert a value to a number
    fn to_number(&self, value: &Value) -> Result<f64> {
        match value {
            Value::Number(n) => {
                if let Some(num) = n.as_f64() {
                    Ok(num)
                } else {
                    Err(RuleError::EvaluationError(
                        "Failed to convert number to f64".to_string(),
                    ))
                }
            }
            Value::String(s) => s.parse::<f64>().map_err(|_| {
                RuleError::EvaluationError(format!("Failed to parse string as number: {s}"))
            }),
            Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
            _ => Err(RuleError::EvaluationError(
                "Cannot convert value to number".to_string(),
            )),
        }
    }
}

/// Evaluate if a path exists in the context
#[allow(dead_code)] // Kept for future implementation
fn evaluate_path_exists(path: &str, context: &Value) -> Result<bool> {
    // Navigate the context using the path
    let value = get_value_at_path(context, path)?;

    // The condition is met if the value exists (is not null)
    Ok(!value.is_null())
}

/// Evaluate if a path equals a value in the context
#[allow(dead_code)] // Kept for future implementation
fn evaluate_path_equals(path: &str, expected: &Value, context: &Value) -> Result<bool> {
    // Navigate the context using the path
    let value = get_value_at_path(context, path)?;

    // The condition is met if the value equals the expected value
    Ok(*value == *expected)
}

/// Evaluate if a path contains a value in the context
#[allow(dead_code)] // Kept for future implementation
fn evaluate_path_contains(path: &str, expected: &Value, context: &Value) -> Result<bool> {
    // Navigate the context using the path
    let value = get_value_at_path(context, path)?;

    // Handle different value types
    match value {
        Value::String(s) => {
            // If expected is a string, check if it's contained in s
            if let Value::String(expected_str) = expected {
                Ok(s.contains(expected_str))
            } else {
                Ok(false)
            }
        }
        Value::Array(arr) => {
            // If we're checking an array, check if it contains the expected value
            Ok(arr.contains(expected))
        }
        Value::Object(obj) => {
            // If we're checking an object and expected is a string, check if any key matches
            if let Value::String(key) = expected {
                Ok(obj.contains_key(key))
            } else {
                Ok(false)
            }
        }
        _ => Ok(false),
    }
}

/// Evaluate if a path starts with a value in the context
#[allow(dead_code)] // Kept for future implementation
fn evaluate_path_starts_with(path: &str, expected: &Value, context: &Value) -> Result<bool> {
    // Navigate the context using the path
    let value = get_value_at_path(context, path)?;

    // Only applicable to strings
    if let Value::String(s) = value {
        if let Value::String(expected_str) = expected {
            Ok(s.starts_with(expected_str))
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

/// Evaluate if a path ends with a value in the context
#[allow(dead_code)] // Kept for future implementation
fn evaluate_path_ends_with(path: &str, expected: &Value, context: &Value) -> Result<bool> {
    // Navigate the context using the path
    let value = get_value_at_path(context, path)?;

    // Only applicable to strings
    if let Value::String(s) = value {
        if let Value::String(expected_str) = expected {
            Ok(s.ends_with(expected_str))
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

/// Evaluate if a path is of a specific type in the context
#[allow(dead_code)] // Kept for future implementation
fn evaluate_path_is_type(path: &str, expected_type: &str, context: &Value) -> Result<bool> {
    // Navigate the context using the path
    let value = get_value_at_path(context, path)?;

    // Check the type
    match expected_type {
        "string" => Ok(value.is_string()),
        "number" => Ok(value.is_number()),
        "boolean" => Ok(value.is_boolean()),
        "array" => Ok(value.is_array()),
        "object" => Ok(value.is_object()),
        "null" => Ok(value.is_null()),
        _ => Err(RuleError::InvalidType(expected_type.to_string())),
    }
}

/// Evaluate an AND condition in the context
#[allow(dead_code)] // Kept for future implementation
async fn evaluate_and_condition(
    evaluator: &RuleEvaluator,
    conditions: &[RuleCondition],
    context: &Value,
) -> Result<bool> {
    for condition in conditions {
        if !evaluator.evaluate_condition(condition, context).await? {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Evaluate an OR condition in the context
#[allow(dead_code)] // Kept for future implementation
async fn evaluate_or_condition(
    evaluator: &RuleEvaluator,
    conditions: &[RuleCondition],
    context: &Value,
) -> Result<bool> {
    for condition in conditions {
        if evaluator.evaluate_condition(condition, context).await? {
            return Ok(true);
        }
    }

    // If there are no conditions, the OR is false
    if conditions.is_empty() {
        return Ok(false);
    }

    Ok(false)
}

/// Evaluate a NOT condition in the context
#[allow(dead_code)] // Kept for future implementation
async fn evaluate_not_condition(
    evaluator: &RuleEvaluator,
    condition: &RuleCondition,
    context: &Value,
) -> Result<bool> {
    let result = evaluator.evaluate_condition(condition, context).await?;
    Ok(!result)
}

/// Get a value at a specific path in the context
#[allow(dead_code)] // Kept for future implementation
fn get_value_at_path<'a>(value: &'a Value, path: &str) -> Result<&'a Value> {
    let parts: Vec<&str> = path.split('.').collect();

    let mut current = value;
    for part in parts {
        match current {
            Value::Object(obj) => {
                current = obj.get(part).unwrap_or(&Value::Null);
            }
            Value::Array(arr) => {
                // Handle array indexing
                if let Ok(index) = part.parse::<usize>() {
                    current = arr.get(index).unwrap_or(&Value::Null);
                } else {
                    return Err(RuleError::InvalidPath(format!(
                        "Invalid array index: {part}"
                    )));
                }
            }
            _ => {
                return Err(RuleError::InvalidPath(format!(
                    "Cannot navigate through non-object value at path segment: {part}"
                )));
            }
        }
    }

    Ok(current)
}
