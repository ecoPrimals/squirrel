// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Rule evaluator for evaluating rules against context data
//!
//! Evaluator methods use `&self` for consistency and future state access.

#![allow(clippy::unused_self)]

use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::{RuleEvaluatorError, RuleSystemError, RuleSystemResult};
use crate::models::{EvaluationResult, Rule, RuleCondition};
use crate::utils;

/// Rule evaluator for evaluating rules against context data
#[derive(Debug)]
pub struct RuleEvaluator {
    /// Evaluation cache for performance
    evaluation_cache: Arc<RwLock<HashMap<String, CachedEvaluation>>>,
    /// Plugin registry for custom evaluators
    plugin_registry: Arc<RwLock<HashMap<String, Box<dyn ConditionEvaluator>>>>,
    /// Evaluation statistics
    stats: Arc<RwLock<EvaluationStatistics>>,
}

/// Cached evaluation result
#[derive(Debug, Clone)]
struct CachedEvaluation {
    /// Evaluation result
    result: bool,
    /// When the evaluation was cached
    timestamp: DateTime<Utc>,
    /// Time taken for evaluation
    duration: chrono::Duration,
}

/// Evaluation statistics
#[derive(Debug, Clone)]
/// Rule evaluation statistics
pub struct EvaluationStatistics {
    /// Total number of evaluations
    total_evaluations: u64,
    /// Number of cached evaluations
    cached_evaluations: u64,
    /// Average evaluation time
    average_duration: chrono::Duration,
    /// Number of successful evaluations
    successful_evaluations: u64,
    /// Number of failed evaluations
    #[allow(dead_code)] // Metric for future evaluation error reporting
    failed_evaluations: u64,
}

impl Default for EvaluationStatistics {
    fn default() -> Self {
        Self {
            total_evaluations: 0,
            cached_evaluations: 0,
            average_duration: chrono::Duration::zero(),
            successful_evaluations: 0,
            failed_evaluations: 0,
        }
    }
}

/// Trait for custom condition evaluators
#[async_trait::async_trait]
pub trait ConditionEvaluator: Send + Sync + std::fmt::Debug {
    /// Evaluate a condition against context data
    async fn evaluate(
        &self,
        condition: &RuleCondition,
        context_data: &Value,
    ) -> RuleSystemResult<bool>;

    /// Get the name of the evaluator
    fn name(&self) -> &str;
}

impl RuleEvaluator {
    /// Create a new rule evaluator
    #[must_use]
    pub fn new() -> Self {
        Self {
            evaluation_cache: Arc::new(RwLock::new(HashMap::new())),
            plugin_registry: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(EvaluationStatistics::default())),
        }
    }

    /// Initialize the evaluator
    pub fn initialize(&self) {
        // Register built-in evaluators
        self.register_builtin_evaluators();
    }

    /// Register built-in condition evaluators
    fn register_builtin_evaluators(&self) {
        // Built-in evaluators are handled directly in evaluate_condition
        // This is a placeholder for future plugin architecture
    }

    /// Evaluate a rule against context data
    pub async fn evaluate_rule(
        &self,
        rule: &Rule,
        context_id: &str,
        context_data: &Value,
    ) -> RuleSystemResult<EvaluationResult> {
        let start_time = Utc::now();

        // Check cache first
        let cache_key = format!("{}:{}", rule.id, context_id);
        if let Some(cached) = self.get_cached_evaluation(&cache_key).await {
            // Return cached result if not expired (cache for 5 minutes)
            if start_time
                .signed_duration_since(cached.timestamp)
                .num_seconds()
                < 300
            {
                self.update_stats(true, cached.duration).await;
                return Ok(EvaluationResult {
                    rule_id: rule.id.clone(),
                    context_id: context_id.to_string(),
                    matches: cached.result,
                    timestamp: start_time,
                });
            }
        }

        // Evaluate conditions
        let matches = self
            .evaluate_conditions(&rule.conditions, context_data)
            .await?;

        let end_time = Utc::now();
        let duration = end_time.signed_duration_since(start_time);

        // Cache the result
        self.cache_evaluation(&cache_key, matches, start_time, duration)
            .await;

        // Update statistics
        self.update_stats(false, duration).await;

        Ok(EvaluationResult {
            rule_id: rule.id.clone(),
            context_id: context_id.to_string(),
            matches,
            timestamp: start_time,
        })
    }

    /// Evaluate a list of conditions (AND logic)
    async fn evaluate_conditions(
        &self,
        conditions: &[RuleCondition],
        context_data: &Value,
    ) -> RuleSystemResult<bool> {
        // If no conditions, return true
        if conditions.is_empty() {
            return Ok(true);
        }

        // Evaluate each condition
        for condition in conditions {
            let result = Box::pin(self.evaluate_condition(condition, context_data)).await?;
            if !result {
                return Ok(false); // Short-circuit on first false
            }
        }

        Ok(true)
    }

    /// Evaluate a single condition
    async fn evaluate_condition(
        &self,
        condition: &RuleCondition,
        context_data: &Value,
    ) -> RuleSystemResult<bool> {
        match condition {
            RuleCondition::Equals { path, value } => {
                let context_value = utils::extract_value_by_path(context_data, path);
                Ok(context_value.as_ref() == Some(value))
            }

            RuleCondition::Matches { path, pattern } => {
                let context_value = utils::extract_value_by_path(context_data, path);
                if let Some(Value::String(text)) = context_value {
                    Ok(utils::match_pattern(pattern, &text))
                } else {
                    Ok(false)
                }
            }

            RuleCondition::GreaterThan { path, value } => {
                let context_value = utils::extract_value_by_path(context_data, path);
                match (context_value, value) {
                    (Some(Value::Number(ctx_num)), Value::Number(val_num)) => {
                        Ok(ctx_num.as_f64().unwrap_or(0.0) > val_num.as_f64().unwrap_or(0.0))
                    }
                    _ => Ok(false),
                }
            }

            RuleCondition::LessThan { path, value } => {
                let context_value = utils::extract_value_by_path(context_data, path);
                match (context_value, value) {
                    (Some(Value::Number(ctx_num)), Value::Number(val_num)) => {
                        Ok(ctx_num.as_f64().unwrap_or(0.0) < val_num.as_f64().unwrap_or(0.0))
                    }
                    _ => Ok(false),
                }
            }

            RuleCondition::Exists { path } => {
                let context_value = utils::extract_value_by_path(context_data, path);
                Ok(context_value.is_some())
            }

            RuleCondition::NotExists { path } => {
                let context_value = utils::extract_value_by_path(context_data, path);
                Ok(context_value.is_none())
            }

            RuleCondition::All { conditions } => {
                Box::pin(self.evaluate_conditions(conditions, context_data)).await
            }

            RuleCondition::Any { conditions } => {
                // If no conditions, return false
                if conditions.is_empty() {
                    return Ok(false);
                }

                // Evaluate each condition
                for condition in conditions {
                    let result = Box::pin(self.evaluate_condition(condition, context_data)).await?;
                    if result {
                        return Ok(true); // Short-circuit on first true
                    }
                }

                Ok(false)
            }

            RuleCondition::Not { condition } => {
                let result = Box::pin(self.evaluate_condition(condition, context_data)).await?;
                Ok(!result)
            }

            RuleCondition::Plugin { plugin_id, config } => {
                // Check if plugin is registered
                let plugin_registry = self.plugin_registry.read().await;
                if let Some(evaluator) = plugin_registry.get(plugin_id) {
                    // Create a plugin condition with the config
                    let plugin_condition = RuleCondition::Plugin {
                        plugin_id: plugin_id.clone(),
                        config: config.clone(),
                    };
                    evaluator.evaluate(&plugin_condition, context_data).await
                } else {
                    Err(RuleSystemError::EvaluatorError(RuleEvaluatorError::Other(
                        format!("Plugin evaluator not found: {plugin_id}"),
                    )))
                }
            }

            RuleCondition::Script { script, language } => {
                // For now, we'll return an error for scripts
                // In a real implementation, this would use a scripting engine
                Err(RuleSystemError::EvaluatorError(RuleEvaluatorError::Other(
                    format!("Script evaluation not implemented: {script} ({language})"),
                )))
            }
        }
    }

    /// Get cached evaluation if available
    async fn get_cached_evaluation(&self, key: &str) -> Option<CachedEvaluation> {
        self.evaluation_cache.read().await.get(key).cloned()
    }

    /// Cache an evaluation result
    async fn cache_evaluation(
        &self,
        key: &str,
        result: bool,
        timestamp: DateTime<Utc>,
        duration: chrono::Duration,
    ) {
        let cached = CachedEvaluation {
            result,
            timestamp,
            duration,
        };

        self.evaluation_cache
            .write()
            .await
            .insert(key.to_string(), cached);
    }

    /// Update evaluation statistics
    async fn update_stats(&self, was_cached: bool, duration: chrono::Duration) {
        let mut stats = self.stats.write().await;

        stats.total_evaluations += 1;

        if was_cached {
            stats.cached_evaluations += 1;
        }

        // Update average duration using saturating arithmetic to avoid truncation
        #[allow(clippy::cast_possible_truncation)]
        let count = stats.total_evaluations.min(i32::MAX as u64) as i32;
        let total_duration = stats.average_duration * (count.saturating_sub(1)) + duration;
        stats.average_duration = if count > 0 {
            total_duration / count
        } else {
            chrono::Duration::zero()
        };

        stats.successful_evaluations += 1;
    }

    /// Register a custom condition evaluator
    pub async fn register_evaluator(
        &self,
        evaluator: Box<dyn ConditionEvaluator>,
    ) -> RuleSystemResult<()> {
        let name = evaluator.name().to_string();
        self.plugin_registry.write().await.insert(name, evaluator);
        Ok(())
    }

    /// Unregister a custom condition evaluator
    pub async fn unregister_evaluator(&self, name: &str) -> RuleSystemResult<()> {
        self.plugin_registry.write().await.remove(name);
        Ok(())
    }

    /// Get registered evaluators
    pub async fn get_registered_evaluators(&self) -> Vec<String> {
        self.plugin_registry.read().await.keys().cloned().collect()
    }

    /// Clear evaluation cache
    pub async fn clear_cache(&self) -> RuleSystemResult<()> {
        self.evaluation_cache.write().await.clear();
        Ok(())
    }

    /// Get evaluation statistics
    pub async fn get_statistics(&self) -> EvaluationStatistics {
        self.stats.read().await.clone()
    }

    /// Reset evaluation statistics
    pub async fn reset_statistics(&self) -> RuleSystemResult<()> {
        *self.stats.write().await = EvaluationStatistics::default();
        Ok(())
    }
}

impl Default for RuleEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Example custom condition evaluator
#[derive(Debug)]
pub struct RegexEvaluator;

#[async_trait::async_trait]
impl ConditionEvaluator for RegexEvaluator {
    async fn evaluate(
        &self,
        condition: &RuleCondition,
        context_data: &Value,
    ) -> RuleSystemResult<bool> {
        if let RuleCondition::Plugin { config, .. } = condition {
            // Extract pattern and path from config
            let pattern = config
                .get("pattern")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    RuleEvaluatorError::Other(
                        "Missing pattern in regex evaluator config".to_string(),
                    )
                })?;

            let path = config.get("path").and_then(|v| v.as_str()).ok_or_else(|| {
                RuleEvaluatorError::Other("Missing path in regex evaluator config".to_string())
            })?;

            // Extract value from context
            let context_value = utils::extract_value_by_path(context_data, path);

            if let Some(Value::String(text)) = context_value {
                // Use regex to match
                let regex = regex::Regex::new(pattern).map_err(|e| {
                    RuleEvaluatorError::Other(format!("Invalid regex pattern: {e}"))
                })?;

                Ok(regex.is_match(&text))
            } else {
                Ok(false)
            }
        } else {
            Err(RuleSystemError::EvaluatorError(RuleEvaluatorError::Other(
                "Invalid condition type for regex evaluator".to_string(),
            )))
        }
    }

    fn name(&self) -> &'static str {
        "regex"
    }
}

/// Create a new rule evaluator with default configuration
pub fn create_rule_evaluator() -> RuleSystemResult<RuleEvaluator> {
    Ok(RuleEvaluator::new())
}

/// Create a rule evaluator with custom configuration
pub fn create_rule_evaluator_with_config() -> RuleSystemResult<RuleEvaluator> {
    let evaluator = RuleEvaluator::new();
    // Add any custom configuration here
    Ok(evaluator)
}
