// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Rule evaluator for evaluating rules against context
use regex::Regex;
use std::sync::{Arc, Mutex};
use tracing::warn;

use serde_json::Value;

use super::error::{Result, RuleError};
use super::models::{Rule, RuleCondition};

/// Rule evaluator for matching rules against context and evaluating conditions
#[derive(Debug)]
pub struct RuleEvaluator {
    /// Regular expression cache
    regex_cache: Mutex<std::collections::HashMap<String, Regex>>,
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
            regex_cache: Mutex::new(std::collections::HashMap::new()),
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
                    }
                    return self.match_pattern(&value.to_string(), pattern);
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
            let mut cache = self
                .regex_cache
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

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
