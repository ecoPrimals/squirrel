// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Plugin manager for rule-specific plugins
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::error::{Result, RuleError};
use super::ContextAdapter;
use super::ContextPlugin;
use squirrel_interfaces::context::ContextTransformation;

/// Rule plugin manager for managing custom conditions and actions
#[derive(Debug)]
pub struct RulePluginManager {
    /// Core plugin manager
    core_plugin_manager: Arc<dyn ContextPlugin>,
    /// Custom condition evaluators
    condition_evaluators: RwLock<HashMap<String, Arc<dyn ConditionEvaluator>>>,
    /// Custom action executors
    action_executors: RwLock<HashMap<String, Arc<dyn ActionExecutor>>>,
}

impl RulePluginManager {
    /// Create a new rule plugin manager
    pub fn new(core_plugin_manager: Arc<dyn ContextPlugin>) -> Self {
        Self {
            core_plugin_manager,
            condition_evaluators: RwLock::new(HashMap::new()),
            action_executors: RwLock::new(HashMap::new()),
        }
    }

    /// Get the core plugin manager
    pub fn core_plugin_manager(&self) -> &Arc<dyn ContextPlugin> {
        &self.core_plugin_manager
    }

    /// Register a custom condition evaluator
    pub async fn register_condition_evaluator<E>(&self, id: impl Into<String>, evaluator: E)
    where
        E: ConditionEvaluator + 'static,
    {
        let mut evaluators = self.condition_evaluators.write().await;
        evaluators.insert(id.into(), Arc::new(evaluator));
    }

    /// Register a custom action executor
    pub async fn register_action_executor<E>(&self, id: impl Into<String>, executor: E)
    where
        E: ActionExecutor + 'static,
    {
        let mut executors = self.action_executors.write().await;
        executors.insert(id.into(), Arc::new(executor));
    }

    /// Get a custom condition evaluator
    pub async fn get_condition_evaluator(&self, id: &str) -> Result<Arc<dyn ConditionEvaluator>> {
        let evaluators = self.condition_evaluators.read().await;

        if let Some(evaluator) = evaluators.get(id) {
            Ok(Arc::clone(evaluator))
        } else {
            Err(RuleError::PluginNotFound(id.to_string()))
        }
    }

    /// Get a custom action executor
    pub async fn get_action_executor(&self, id: &str) -> Result<Arc<dyn ActionExecutor>> {
        let executors = self.action_executors.read().await;

        if let Some(executor) = executors.get(id) {
            Ok(Arc::clone(executor))
        } else {
            Err(RuleError::PluginNotFound(id.to_string()))
        }
    }

    /// Get a transformation by ID
    pub async fn get_transformation(&self, id: &str) -> Result<Arc<dyn ContextTransformation>> {
        self.core_plugin_manager
            .get_transformation(id)
            .await
            .map_err(|e| {
                RuleError::PluginError(format!("Failed to get transformation '{id}': {e}"))
            })
    }

    /// Get an adapter by ID
    pub async fn get_adapter(&self, id: &str) -> Result<Arc<dyn ContextAdapter>> {
        self.core_plugin_manager
            .get_adapter(id)
            .await
            .map_err(|e| RuleError::PluginError(format!("Failed to get adapter '{id}': {e}")))
    }
}

/// Custom condition evaluator trait
#[async_trait]
pub trait ConditionEvaluator: Send + Sync + Debug {
    /// Evaluate a custom condition
    async fn evaluate(&self, params: &Value, context: &Value) -> Result<bool>;
}

/// Custom action executor trait
#[async_trait]
pub trait ActionExecutor: Send + Sync + Debug {
    /// Execute a custom action
    async fn execute(&self, params: &Value, context: &Value) -> Result<Value>;
}
