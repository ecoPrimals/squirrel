// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use crate::error::{RuleActionError, RuleSystemError};
use crate::models::{ActionResult, RuleAction};
use serde_json::json;

/// Mock plugin for testing `ExecutePlugin` action
#[derive(Debug)]
struct MockActionPlugin;

#[async_trait::async_trait]
impl ActionPlugin for MockActionPlugin {
    async fn execute(
        &self,
        _action: &RuleAction,
        context_id: &str,
        rule_id: &str,
    ) -> RuleSystemResult<ActionResult> {
        Ok(ActionResult {
            action_id: "mock_action_123".to_string(),
            rule_id: rule_id.to_string(),
            context_id: context_id.to_string(),
            success: true,
            message: "Mock plugin executed".to_string(),
            data: Some(json!({"mock": true})),
            timestamp: Utc::now(),
        })
    }

    fn name(&self) -> &'static str {
        "mock_plugin"
    }

    fn description(&self) -> &'static str {
        "Mock plugin for testing"
    }
}

#[tokio::test]
async fn test_action_executor_new() {
    let executor = ActionExecutor::new();
    assert!(executor.get_registered_plugins().await.is_empty());
}

#[tokio::test]
async fn test_action_executor_default() {
    let executor = ActionExecutor::default();
    assert!(executor.get_registered_plugins().await.is_empty());
}

#[tokio::test]
async fn test_action_executor_initialize() {
    let executor = ActionExecutor::new();
    let result = executor.initialize().await;
    assert!(result.is_ok());
    let plugins = executor.get_registered_plugins().await;
    assert!(plugins.contains(&"notification".to_string()));
    assert!(plugins.contains(&"context_modification".to_string()));
    assert!(plugins.contains(&"validation".to_string()));
}

#[tokio::test]
async fn test_execute_modify_context() {
    let executor = ActionExecutor::new();
    executor.initialize().await.expect("should succeed");

    let action = RuleAction::ModifyContext {
        path: "data.value".to_string(),
        value: json!(42),
    };
    let result = executor
        .execute_action(&action, "ctx-1", "rule-1")
        .await
        .expect("should succeed");

    assert!(result.success);
    assert_eq!(result.rule_id, "rule-1");
    assert_eq!(result.context_id, "ctx-1");
    assert!(result.action_id.starts_with("modify_context_"));
    assert!(result.message.contains("Modified context"));
    assert_eq!(
        result.data.as_ref().expect("should succeed")["path"],
        "data.value"
    );
    assert_eq!(result.data.as_ref().expect("should succeed")["value"], 42);
}

#[tokio::test]
async fn test_execute_create_recovery_point() {
    let executor = ActionExecutor::new();
    executor.initialize().await.expect("should succeed");

    let action = RuleAction::CreateRecoveryPoint {
        description: "Before migration".to_string(),
    };
    let result = executor
        .execute_action(&action, "ctx-2", "rule-2")
        .await
        .expect("should succeed");

    assert!(result.success);
    assert_eq!(result.rule_id, "rule-2");
    assert_eq!(result.context_id, "ctx-2");
    assert!(result.action_id.starts_with("recovery_point_"));
    assert!(result.message.contains("Before migration"));
    assert!(
        result
            .data
            .as_ref()
            .expect("should succeed")
            .get("recovery_id")
            .is_some()
    );
    assert_eq!(
        result.data.as_ref().expect("should succeed")["description"],
        "Before migration"
    );
}

#[tokio::test]
async fn test_execute_transformation() {
    let executor = ActionExecutor::new();
    executor.initialize().await.expect("should succeed");

    let action = RuleAction::ExecuteTransformation {
        transformation_id: "transform-1".to_string(),
        config: Some(json!({"param": "value"})),
    };
    let result = executor
        .execute_action(&action, "ctx-3", "rule-3")
        .await
        .expect("should succeed");

    assert!(result.success);
    assert_eq!(result.rule_id, "rule-3");
    assert!(result.action_id.starts_with("transformation_"));
    assert!(result.message.contains("transform-1"));
    assert_eq!(
        result.data.as_ref().expect("should succeed")["transformation_id"],
        "transform-1"
    );
}

#[tokio::test]
async fn test_execute_transformation_without_config() {
    let executor = ActionExecutor::new();
    executor.initialize().await.expect("should succeed");

    let action = RuleAction::ExecuteTransformation {
        transformation_id: "transform-2".to_string(),
        config: None,
    };
    let result = executor
        .execute_action(&action, "ctx-4", "rule-4")
        .await
        .expect("should succeed");

    assert!(result.success);
    assert_eq!(
        result.data.as_ref().expect("should succeed")["config"],
        serde_json::Value::Null
    );
}

#[tokio::test]
async fn test_execute_notify() {
    let executor = ActionExecutor::new();
    executor.initialize().await.expect("should succeed");

    let action = RuleAction::Notify {
        channel: "slack".to_string(),
        message: "Test alert".to_string(),
        data: Some(json!({"priority": "high"})),
    };
    let result = executor
        .execute_action(&action, "ctx-5", "rule-5")
        .await
        .expect("should succeed");

    assert!(result.success);
    assert_eq!(result.rule_id, "rule-5");
    assert!(result.action_id.starts_with("notify_"));
    assert!(result.message.contains("slack"));
    assert!(result.message.contains("Test alert"));
    assert_eq!(
        result.data.as_ref().expect("should succeed")["channel"],
        "slack"
    );
    assert_eq!(
        result.data.as_ref().expect("should succeed")["message"],
        "Test alert"
    );
}

#[tokio::test]
async fn test_execute_notify_without_data() {
    let executor = ActionExecutor::new();
    executor.initialize().await.expect("should succeed");

    let action = RuleAction::Notify {
        channel: "email".to_string(),
        message: "Simple message".to_string(),
        data: None,
    };
    let result = executor
        .execute_action(&action, "ctx-6", "rule-6")
        .await
        .expect("should succeed");

    assert!(result.success);
    assert_eq!(
        result.data.as_ref().expect("should succeed")["data"],
        serde_json::Value::Null
    );
}

#[tokio::test]
async fn test_execute_plugin_registered() {
    let executor = ActionExecutor::new();
    executor
        .register_plugin(Box::new(MockActionPlugin))
        .await
        .expect("should succeed");

    let action = RuleAction::ExecutePlugin {
        plugin_id: "mock_plugin".to_string(),
        config: json!({}),
    };
    let result = executor
        .execute_action(&action, "ctx-7", "rule-7")
        .await
        .expect("should succeed");

    assert!(result.success);
    assert_eq!(result.action_id, "mock_action_123");
    assert_eq!(result.message, "Mock plugin executed");
    assert_eq!(result.data.as_ref().expect("should succeed")["mock"], true);
}

#[tokio::test]
async fn test_execute_plugin_not_found() {
    let executor = ActionExecutor::new();
    executor.initialize().await.expect("should succeed");

    let action = RuleAction::ExecutePlugin {
        plugin_id: "nonexistent_plugin".to_string(),
        config: json!({}),
    };
    let result = executor
        .execute_action(&action, "ctx-8", "rule-8")
        .await
        .expect("should succeed");

    assert!(!result.success);
    assert!(result.action_id.starts_with("plugin_error_"));
    assert!(result.message.contains("nonexistent_plugin"));
}

#[tokio::test]
async fn test_execute_script() {
    let executor = ActionExecutor::new();
    executor.initialize().await.expect("should succeed");

    let action = RuleAction::ExecuteScript {
        script: "return 1 + 1".to_string(),
        language: "js".to_string(),
    };
    let result = executor
        .execute_action(&action, "ctx-9", "rule-9")
        .await
        .expect("should succeed");

    assert!(!result.success);
    assert!(result.message.contains("not implemented"));
    assert_eq!(
        result.data.as_ref().expect("should succeed")["script"],
        "return 1 + 1"
    );
    assert_eq!(
        result.data.as_ref().expect("should succeed")["language"],
        "js"
    );
}

#[tokio::test]
async fn test_execute_validate_context() {
    let executor = ActionExecutor::new();
    executor.initialize().await.expect("should succeed");

    let schema = json!({
        "type": "object",
        "properties": {"name": {"type": "string"}}
    });
    let action = RuleAction::ValidateContext {
        schema: schema.clone(),
    };
    let result = executor
        .execute_action(&action, "ctx-10", "rule-10")
        .await
        .expect("should succeed");

    assert!(result.success);
    assert!(result.action_id.starts_with("validate_"));
    assert_eq!(result.message, "Context validation completed");
    assert_eq!(
        result.data.as_ref().expect("should succeed")["schema"],
        schema
    );
}

#[tokio::test]
async fn test_register_and_unregister_plugin() {
    let executor = ActionExecutor::new();
    assert!(executor.get_registered_plugins().await.is_empty());

    executor
        .register_plugin(Box::new(MockActionPlugin))
        .await
        .expect("should succeed");
    let plugins = executor.get_registered_plugins().await;
    assert_eq!(plugins.len(), 1);
    assert!(plugins.contains(&"mock_plugin".to_string()));

    executor
        .unregister_plugin("mock_plugin")
        .await
        .expect("should succeed");
    assert!(executor.get_registered_plugins().await.is_empty());
}

#[tokio::test]
async fn test_unregister_nonexistent_plugin() {
    let executor = ActionExecutor::new();
    let result = executor.unregister_plugin("nonexistent").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_execution_history() {
    let executor = ActionExecutor::new();
    executor.initialize().await.expect("should succeed");

    let action = RuleAction::ModifyContext {
        path: "x".to_string(),
        value: json!(1),
    };
    executor
        .execute_action(&action, "ctx-h", "rule-h")
        .await
        .expect("should succeed");
    executor
        .execute_action(&action, "ctx-h", "rule-h")
        .await
        .expect("should succeed");

    let history = executor.get_execution_history().await;
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].rule_id(), "rule-h");
    assert_eq!(history[0].context_id(), "ctx-h");
    assert!(history[0].action_id().starts_with("modify_context_"));
    assert!(history[0].result().success);
}

#[tokio::test]
async fn test_statistics_tracking() {
    let executor = ActionExecutor::new();
    executor.initialize().await.expect("should succeed");

    let success_action = RuleAction::ModifyContext {
        path: "x".to_string(),
        value: json!(1),
    };
    let fail_action = RuleAction::ExecuteScript {
        script: "x".to_string(),
        language: "js".to_string(),
    };

    executor
        .execute_action(&success_action, "ctx", "rule")
        .await
        .expect("should succeed");
    executor
        .execute_action(&success_action, "ctx", "rule")
        .await
        .expect("should succeed");
    executor
        .execute_action(&fail_action, "ctx", "rule")
        .await
        .expect("should succeed");

    let stats = executor.get_statistics().await;
    assert_eq!(stats.total_actions(), 3);
    assert_eq!(stats.successful_actions(), 2);
    assert_eq!(stats.failed_actions(), 1);
    assert_eq!(stats.count_for_type("modify_context"), 2);
    assert_eq!(stats.count_for_type("execute_script"), 1);
}

#[tokio::test]
async fn test_clear_history() {
    let executor = ActionExecutor::new();
    executor.initialize().await.expect("should succeed");

    let action = RuleAction::ModifyContext {
        path: "x".to_string(),
        value: json!(1),
    };
    executor
        .execute_action(&action, "ctx", "rule")
        .await
        .expect("should succeed");
    assert_eq!(executor.get_execution_history().await.len(), 1);

    executor.clear_history().await.expect("should succeed");
    assert!(executor.get_execution_history().await.is_empty());
}

#[tokio::test]
async fn test_reset_statistics() {
    let executor = ActionExecutor::new();
    executor.initialize().await.expect("should succeed");

    let action = RuleAction::ModifyContext {
        path: "x".to_string(),
        value: json!(1),
    };
    executor
        .execute_action(&action, "ctx", "rule")
        .await
        .expect("should succeed");
    assert_eq!(executor.get_statistics().await.total_actions(), 1);

    executor.reset_statistics().await.expect("should succeed");
    let stats = executor.get_statistics().await;
    assert_eq!(stats.total_actions(), 0);
    assert_eq!(stats.successful_actions(), 0);
    assert_eq!(stats.failed_actions(), 0);
}

#[tokio::test]
async fn test_notification_action_plugin() {
    let plugin = NotificationAction;
    assert_eq!(plugin.name(), "notification");
    assert_eq!(
        plugin.description(),
        "Sends notifications to external systems"
    );

    let action = RuleAction::Notify {
        channel: "test".to_string(),
        message: "hello".to_string(),
        data: None,
    };
    let result = plugin
        .execute(&action, "ctx", "rule")
        .await
        .expect("should succeed");
    assert!(result.success);
    assert!(result.action_id.starts_with("notification_"));
    assert!(result.message.contains("test"));
    assert!(result.message.contains("hello"));
}

#[tokio::test]
async fn test_notification_action_plugin_wrong_type() {
    let plugin = NotificationAction;
    let action = RuleAction::ModifyContext {
        path: "x".to_string(),
        value: json!(1),
    };
    let result = plugin.execute(&action, "ctx", "rule").await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        RuleSystemError::ActionError(RuleActionError::Other(_))
    ));
}

#[tokio::test]
async fn test_context_modification_action_plugin() {
    let plugin = ContextModificationAction;
    assert_eq!(plugin.name(), "context_modification");
    assert_eq!(plugin.description(), "Modifies context data");

    let action = RuleAction::ModifyContext {
        path: "data.x".to_string(),
        value: json!("value"),
    };
    let result = plugin
        .execute(&action, "ctx", "rule")
        .await
        .expect("should succeed");
    assert!(result.success);
    assert!(result.action_id.starts_with("context_mod_"));
    assert!(result.message.contains("data.x"));
}

#[tokio::test]
async fn test_context_modification_action_plugin_wrong_type() {
    let plugin = ContextModificationAction;
    let action = RuleAction::Notify {
        channel: "x".to_string(),
        message: "y".to_string(),
        data: None,
    };
    let result = plugin.execute(&action, "ctx", "rule").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_validation_action_plugin() {
    let plugin = ValidationAction;
    assert_eq!(plugin.name(), "validation");
    assert_eq!(
        plugin.description(),
        "Validates context data against schemas"
    );

    let schema = json!({"type": "object"});
    let action = RuleAction::ValidateContext {
        schema: schema.clone(),
    };
    let result = plugin
        .execute(&action, "ctx", "rule")
        .await
        .expect("should succeed");
    assert!(result.success);
    assert!(result.action_id.starts_with("validation_"));
}

#[tokio::test]
async fn test_validation_action_plugin_wrong_type() {
    let plugin = ValidationAction;
    let action = RuleAction::Notify {
        channel: "x".to_string(),
        message: "y".to_string(),
        data: None,
    };
    let result = plugin.execute(&action, "ctx", "rule").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_action_statistics_default() {
    let stats = ActionStatistics::default();
    assert_eq!(stats.total_actions(), 0);
    assert_eq!(stats.successful_actions(), 0);
    assert_eq!(stats.failed_actions(), 0);
    assert_eq!(stats.count_for_type("modify_context"), 0);
}

#[tokio::test]
async fn test_create_action_executor() {
    let result = create_action_executor();
    assert!(result.is_ok());
    let executor = result.expect("should succeed");
    assert!(executor.get_registered_plugins().await.is_empty());
}

#[tokio::test]
async fn test_create_action_executor_async() {
    let result = create_action_executor();
    assert!(result.is_ok());
    let executor = result.expect("should succeed");
    executor.initialize().await.expect("should succeed");
    assert!(!executor.get_registered_plugins().await.is_empty());
}

#[tokio::test]
async fn test_create_action_executor_with_config() {
    let result = create_action_executor_with_config();
    assert!(result.is_ok());
    let executor = result.expect("should succeed");
    assert!(executor.get_registered_plugins().await.is_empty());
}

#[tokio::test]
async fn test_concurrent_executions() {
    let executor = std::sync::Arc::new(ActionExecutor::new());
    executor.initialize().await.expect("should succeed");

    let mut handles = vec![];
    for i in 0..5 {
        let exec = std::sync::Arc::clone(&executor);
        let action = RuleAction::ModifyContext {
            path: format!("data.{i}"),
            value: json!(i),
        };
        handles.push(tokio::spawn(async move {
            exec.execute_action(&action, "ctx", "rule").await
        }));
    }

    for handle in handles {
        let result = handle.await.expect("should succeed");
        assert!(result.is_ok());
    }

    let stats = executor.get_statistics().await;
    assert_eq!(stats.total_actions(), 5);
    assert_eq!(executor.get_execution_history().await.len(), 5);
}

#[tokio::test]
async fn test_action_result_serialization_roundtrip() {
    let original = ActionResult {
        action_id: "act-1".to_string(),
        rule_id: "rule-1".to_string(),
        context_id: "ctx-1".to_string(),
        success: true,
        message: "Done".to_string(),
        data: Some(json!({"key": "value"})),
        timestamp: Utc::now(),
    };
    let json = serde_json::to_string(&original).expect("should succeed");
    let deserialized: ActionResult = serde_json::from_str(&json).expect("should succeed");
    assert_eq!(original.action_id, deserialized.action_id);
    assert_eq!(original.rule_id, deserialized.rule_id);
    assert_eq!(original.context_id, deserialized.context_id);
    assert_eq!(original.success, deserialized.success);
    assert_eq!(original.message, deserialized.message);
}

#[tokio::test]
async fn test_rule_action_modify_context_serialization() {
    let action = RuleAction::ModifyContext {
        path: "data.x".to_string(),
        value: json!(42),
    };
    let json = serde_json::to_string(&action).expect("should succeed");
    let deserialized: RuleAction = serde_json::from_str(&json).expect("should succeed");
    if let (
        RuleAction::ModifyContext {
            path: p1,
            value: v1,
        },
        RuleAction::ModifyContext {
            path: p2,
            value: v2,
        },
    ) = (&action, &deserialized)
    {
        assert_eq!(p1, p2);
        assert_eq!(v1, v2);
    } else {
        unreachable!("Expected ModifyContext variant");
    }
}

#[tokio::test]
async fn test_rule_action_notify_serialization() {
    let action = RuleAction::Notify {
        channel: "slack".to_string(),
        message: "hello".to_string(),
        data: Some(json!({"a": 1})),
    };
    let json = serde_json::to_string(&action).expect("should succeed");
    let deserialized: RuleAction = serde_json::from_str(&json).expect("should succeed");
    if let (
        RuleAction::Notify {
            channel: c1,
            message: m1,
            ..
        },
        RuleAction::Notify {
            channel: c2,
            message: m2,
            ..
        },
    ) = (&action, &deserialized)
    {
        assert_eq!(c1, c2);
        assert_eq!(m1, m2);
    } else {
        unreachable!("Expected Notify variant");
    }
}
