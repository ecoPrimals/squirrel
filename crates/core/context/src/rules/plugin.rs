// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Plugin manager for rule-specific plugins
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::ContextAdapterDyn;
use super::ContextPlugin;
use super::error::{Result, RuleError};
use squirrel_interfaces::context::DynContextTransformation;

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
    pub async fn get_transformation(&self, id: &str) -> Result<Arc<dyn DynContextTransformation>> {
        self.core_plugin_manager
            .get_transformation(id)
            .await
            .map_err(|e| {
                RuleError::PluginError(format!("Failed to get transformation '{id}': {e}"))
            })
    }

    /// Get an adapter by ID
    pub async fn get_adapter(&self, id: &str) -> Result<Arc<dyn ContextAdapterDyn>> {
        self.core_plugin_manager
            .get_adapter(id)
            .await
            .map_err(|e| RuleError::PluginError(format!("Failed to get adapter '{id}': {e}")))
    }
}

/// Custom condition evaluator trait
///
/// Uses explicit futures so `Arc<dyn ConditionEvaluator>` remains object-safe.
pub trait ConditionEvaluator: Send + Sync + Debug {
    /// Evaluate a custom condition
    fn evaluate(
        &self,
        params: &Value,
        context: &Value,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + '_>>;
}

/// Custom action executor trait
///
/// Uses explicit futures so `Arc<dyn ActionExecutor>` remains object-safe.
pub trait ActionExecutor: Send + Sync + Debug {
    /// Execute a custom action
    fn execute(
        &self,
        params: &Value,
        context: &Value,
    ) -> Pin<Box<dyn Future<Output = Result<Value>> + Send + '_>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::ContextPlugin;
    use crate::rules::NoOpPluginManager;
    use crate::rules::RuleError;
    use serde_json::json;
    use std::future::Future;
    use std::pin::Pin;

    #[derive(Debug)]
    struct TrueEvaluator;

    impl ConditionEvaluator for TrueEvaluator {
        fn evaluate(
            &self,
            _params: &serde_json::Value,
            _context: &serde_json::Value,
        ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + '_>> {
            Box::pin(async { Ok(true) })
        }
    }

    #[derive(Debug)]
    struct EchoActionExecutor;

    impl ActionExecutor for EchoActionExecutor {
        fn execute(
            &self,
            _params: &serde_json::Value,
            context: &serde_json::Value,
        ) -> Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send + '_>> {
            let context = context.clone();
            Box::pin(async move { Ok(context) })
        }
    }

    #[tokio::test]
    async fn register_and_get_condition_evaluator() {
        let mgr = RulePluginManager::new(Arc::new(NoOpPluginManager));
        mgr.register_condition_evaluator("ce1", TrueEvaluator).await;
        let ev = mgr
            .get_condition_evaluator("ce1")
            .await
            .expect("should succeed");
        assert!(
            ev.evaluate(&json!({}), &json!({}))
                .await
                .expect("should succeed")
        );
    }

    #[tokio::test]
    async fn get_condition_evaluator_missing_errors() {
        let mgr = RulePluginManager::new(Arc::new(NoOpPluginManager));
        let err = mgr.get_condition_evaluator("nope").await.unwrap_err();
        assert!(matches!(err, RuleError::PluginNotFound(_)));
    }

    #[tokio::test]
    async fn register_and_get_action_executor() {
        let mgr = RulePluginManager::new(Arc::new(NoOpPluginManager));
        mgr.register_action_executor("ae1", EchoActionExecutor)
            .await;
        let ex = mgr
            .get_action_executor("ae1")
            .await
            .expect("should succeed");
        let v = ex
            .execute(&json!({}), &json!({"k": 1}))
            .await
            .expect("should succeed");
        assert_eq!(v["k"], 1);
    }

    #[tokio::test]
    async fn get_action_executor_missing_errors() {
        let mgr = RulePluginManager::new(Arc::new(NoOpPluginManager));
        let err = mgr.get_action_executor("missing").await.unwrap_err();
        assert!(matches!(err, RuleError::PluginNotFound(_)));
    }

    #[tokio::test]
    async fn core_plugin_manager_accessor() {
        let dummy: Arc<dyn ContextPlugin> = Arc::new(NoOpPluginManager);
        let mgr = RulePluginManager::new(Arc::clone(&dummy));
        assert!(Arc::ptr_eq(mgr.core_plugin_manager(), &dummy));
    }

    #[tokio::test]
    async fn get_transformation_forwards_plugin_error() {
        let mgr = RulePluginManager::new(Arc::new(NoOpPluginManager));
        let err = mgr.get_transformation("any").await.unwrap_err();
        assert!(matches!(err, RuleError::PluginError(_)));
    }

    #[tokio::test]
    async fn get_adapter_forwards_plugin_error() {
        let mgr = RulePluginManager::new(Arc::new(NoOpPluginManager));
        let err = mgr.get_adapter("any").await.unwrap_err();
        assert!(matches!(err, RuleError::PluginError(_)));
    }
}
