// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Tests for the rules plugin module

use super::error::Result;
use super::plugin::{ActionExecutor, ConditionEvaluator, RulePluginManager};
use crate::rules::DummyPluginManager;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

// Test condition evaluator
#[derive(Debug)]
struct TestConditionEvaluator {
    result: bool,
}

#[async_trait]
impl ConditionEvaluator for TestConditionEvaluator {
    async fn evaluate(&self, _params: &Value, _context: &Value) -> Result<bool> {
        Ok(self.result)
    }
}

// Test action executor
#[derive(Debug)]
struct TestActionExecutor {
    output: Value,
}

#[async_trait]
impl ActionExecutor for TestActionExecutor {
    async fn execute(&self, _params: &Value, _context: &Value) -> Result<Value> {
        Ok(self.output.clone())
    }
}

#[tokio::test]
async fn test_rule_plugin_manager_new() {
    let core_plugin: Arc<dyn crate::rules::ContextPlugin> = Arc::new(DummyPluginManager::default());
    let manager = RulePluginManager::new(core_plugin.clone());

    // Just verify the manager was created successfully
    assert!(Arc::ptr_eq(manager.core_plugin_manager(), &core_plugin));
}

#[tokio::test]
async fn test_register_and_get_condition_evaluator() {
    let core_plugin = Arc::new(DummyPluginManager::default());
    let manager = RulePluginManager::new(core_plugin);

    let evaluator = TestConditionEvaluator { result: true };
    manager
        .register_condition_evaluator("test_condition", evaluator)
        .await;

    let retrieved = manager
        .get_condition_evaluator("test_condition")
        .await
        .expect("Failed to get evaluator");

    let result = retrieved
        .evaluate(&json!({}), &json!({}))
        .await
        .expect("Evaluation failed");
    assert!(result);
}

#[tokio::test]
async fn test_get_condition_evaluator_not_found() {
    let core_plugin = Arc::new(DummyPluginManager::default());
    let manager = RulePluginManager::new(core_plugin);

    let result = manager.get_condition_evaluator("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_register_and_get_action_executor() {
    let core_plugin = Arc::new(DummyPluginManager::default());
    let manager = RulePluginManager::new(core_plugin);

    let executor = TestActionExecutor {
        output: json!({"status": "success"}),
    };
    manager
        .register_action_executor("test_action", executor)
        .await;

    let retrieved = manager
        .get_action_executor("test_action")
        .await
        .expect("Failed to get executor");

    let result = retrieved
        .execute(&json!({}), &json!({}))
        .await
        .expect("Execution failed");
    assert_eq!(result["status"], "success");
}

#[tokio::test]
async fn test_get_action_executor_not_found() {
    let core_plugin = Arc::new(DummyPluginManager::default());
    let manager = RulePluginManager::new(core_plugin);

    let result = manager.get_action_executor("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_transformation_from_core() {
    let core_plugin = Arc::new(DummyPluginManager::default());
    let manager = RulePluginManager::new(core_plugin);

    let result = manager.get_transformation("test_transform").await;
    assert!(result.is_err()); // DummyPluginManager returns error for all transformations
}

#[tokio::test]
async fn test_get_adapter_from_core() {
    let core_plugin = Arc::new(DummyPluginManager::default());
    let manager = RulePluginManager::new(core_plugin);

    let result = manager.get_adapter("test_adapter").await;
    assert!(result.is_err()); // DummyPluginManager returns error for all adapters
}

#[tokio::test]
async fn test_multiple_condition_evaluators() {
    let core_plugin = Arc::new(DummyPluginManager::default());
    let manager = RulePluginManager::new(core_plugin);

    let eval1 = TestConditionEvaluator { result: true };
    let eval2 = TestConditionEvaluator { result: false };

    manager
        .register_condition_evaluator("true_eval", eval1)
        .await;
    manager
        .register_condition_evaluator("false_eval", eval2)
        .await;

    let retrieved1 = manager
        .get_condition_evaluator("true_eval")
        .await
        .expect("Failed to get evaluator 1");
    let result1 = retrieved1
        .evaluate(&json!({}), &json!({}))
        .await
        .expect("test: should succeed");
    assert!(result1);

    let retrieved2 = manager
        .get_condition_evaluator("false_eval")
        .await
        .expect("Failed to get evaluator 2");
    let result2 = retrieved2
        .evaluate(&json!({}), &json!({}))
        .await
        .expect("test: should succeed");
    assert!(!result2);
}

#[tokio::test]
async fn test_multiple_action_executors() {
    let core_plugin = Arc::new(DummyPluginManager::default());
    let manager = RulePluginManager::new(core_plugin);

    let exec1 = TestActionExecutor {
        output: json!({"action": "first"}),
    };
    let exec2 = TestActionExecutor {
        output: json!({"action": "second"}),
    };

    manager.register_action_executor("action1", exec1).await;
    manager.register_action_executor("action2", exec2).await;

    let retrieved1 = manager
        .get_action_executor("action1")
        .await
        .expect("Failed to get executor 1");
    let result1 = retrieved1
        .execute(&json!({}), &json!({}))
        .await
        .expect("test: should succeed");
    assert_eq!(result1["action"], "first");

    let retrieved2 = manager
        .get_action_executor("action2")
        .await
        .expect("Failed to get executor 2");
    let result2 = retrieved2
        .execute(&json!({}), &json!({}))
        .await
        .expect("test: should succeed");
    assert_eq!(result2["action"], "second");
}

#[tokio::test]
async fn test_condition_evaluator_with_params() {
    #[derive(Debug)]
    struct ParamConditionEvaluator;

    #[async_trait]
    impl ConditionEvaluator for ParamConditionEvaluator {
        async fn evaluate(&self, params: &Value, _context: &Value) -> Result<bool> {
            Ok(params["enabled"].as_bool().unwrap_or(false))
        }
    }

    let core_plugin = Arc::new(DummyPluginManager::default());
    let manager = RulePluginManager::new(core_plugin);

    manager
        .register_condition_evaluator("param_eval", ParamConditionEvaluator)
        .await;

    let evaluator = manager
        .get_condition_evaluator("param_eval")
        .await
        .expect("Failed to get evaluator");

    let result1 = evaluator
        .evaluate(&json!({"enabled": true}), &json!({}))
        .await
        .expect("test: should succeed");
    assert!(result1);

    let result2 = evaluator
        .evaluate(&json!({"enabled": false}), &json!({}))
        .await
        .expect("test: should succeed");
    assert!(!result2);
}

#[tokio::test]
async fn test_action_executor_with_context() {
    #[derive(Debug)]
    struct ContextActionExecutor;

    #[async_trait]
    impl ActionExecutor for ContextActionExecutor {
        async fn execute(&self, _params: &Value, context: &Value) -> Result<Value> {
            Ok(json!({
                "user": context["user"].clone(),
                "timestamp": "2024-01-01"
            }))
        }
    }

    let core_plugin = Arc::new(DummyPluginManager::default());
    let manager = RulePluginManager::new(core_plugin);

    manager
        .register_action_executor("context_action", ContextActionExecutor)
        .await;

    let executor = manager
        .get_action_executor("context_action")
        .await
        .expect("Failed to get executor");

    let result = executor
        .execute(&json!({}), &json!({"user": "test_user"}))
        .await
        .expect("test: should succeed");
    assert_eq!(result["user"], "test_user");
    assert_eq!(result["timestamp"], "2024-01-01");
}

#[tokio::test]
async fn test_overwrite_condition_evaluator() {
    let core_plugin = Arc::new(DummyPluginManager::default());
    let manager = RulePluginManager::new(core_plugin);

    let eval1 = TestConditionEvaluator { result: true };
    manager.register_condition_evaluator("test", eval1).await;

    let eval2 = TestConditionEvaluator { result: false };
    manager.register_condition_evaluator("test", eval2).await;

    let evaluator = manager
        .get_condition_evaluator("test")
        .await
        .expect("Failed to get evaluator");
    let result = evaluator
        .evaluate(&json!({}), &json!({}))
        .await
        .expect("test: should succeed");
    assert!(!result); // Should be the second evaluator
}

#[tokio::test]
async fn test_overwrite_action_executor() {
    let core_plugin = Arc::new(DummyPluginManager::default());
    let manager = RulePluginManager::new(core_plugin);

    let exec1 = TestActionExecutor {
        output: json!({"version": 1}),
    };
    manager.register_action_executor("test", exec1).await;

    let exec2 = TestActionExecutor {
        output: json!({"version": 2}),
    };
    manager.register_action_executor("test", exec2).await;

    let executor = manager
        .get_action_executor("test")
        .await
        .expect("Failed to get executor");
    let result = executor
        .execute(&json!({}), &json!({}))
        .await
        .expect("test: should succeed");
    assert_eq!(result["version"], 2); // Should be the second executor
}

#[tokio::test]
async fn test_rule_plugin_manager_debug() {
    let core_plugin = Arc::new(DummyPluginManager::default());
    let manager = RulePluginManager::new(core_plugin);

    let debug_str = format!("{:?}", manager);
    assert!(debug_str.contains("RulePluginManager"));
}

#[tokio::test]
async fn test_concurrent_registrations() {
    let core_plugin = Arc::new(DummyPluginManager::default());
    let manager = Arc::new(RulePluginManager::new(core_plugin));

    let mut handles = vec![];
    for i in 0..10 {
        let manager_clone = manager.clone();
        let handle = tokio::spawn(async move {
            let eval = TestConditionEvaluator { result: i % 2 == 0 };
            manager_clone
                .register_condition_evaluator(format!("eval_{}", i), eval)
                .await;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.expect("Task failed");
    }

    // Verify all were registered
    for i in 0..10 {
        let result = manager
            .get_condition_evaluator(&format!("eval_{}", i))
            .await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_concurrent_retrievals() {
    let core_plugin = Arc::new(DummyPluginManager::default());
    let manager = Arc::new(RulePluginManager::new(core_plugin));

    let eval = TestConditionEvaluator { result: true };
    manager.register_condition_evaluator("shared", eval).await;

    let mut handles = vec![];
    for _ in 0..10 {
        let manager_clone = manager.clone();
        let handle = tokio::spawn(async move {
            manager_clone
                .get_condition_evaluator("shared")
                .await
                .expect("Failed to get evaluator")
        });
        handles.push(handle);
    }

    for handle in handles {
        let evaluator = handle.await.expect("Task failed");
        let result = evaluator
            .evaluate(&json!({}), &json!({}))
            .await
            .expect("test: should succeed");
        assert!(result);
    }
}
