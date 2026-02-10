// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Tests for the rules actions module

use super::actions::{ActionExecutor, AppliedRule, RuleApplicationResult, SingleActionResult};
use super::models::{Rule, RuleAction};
use super::plugin::RulePluginManager;
use crate::rules::DummyPluginManager;
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;

#[test]
fn test_single_action_result_creation() {
    let result = SingleActionResult {
        action_type: "modify".to_string(),
        success: true,
        error: None,
        result: Some(json!({"status": "ok"})),
        timestamp: Utc::now(),
    };

    assert_eq!(result.action_type, "modify");
    assert!(result.success);
    assert!(result.error.is_none());
    assert!(result.result.is_some());
}

#[test]
fn test_single_action_result_with_error() {
    let result = SingleActionResult {
        action_type: "transform".to_string(),
        success: false,
        error: Some("Transformation failed".to_string()),
        result: None,
        timestamp: Utc::now(),
    };

    assert_eq!(result.action_type, "transform");
    assert!(!result.success);
    assert_eq!(result.error, Some("Transformation failed".to_string()));
    assert!(result.result.is_none());
}

#[test]
fn test_applied_rule_creation() {
    let rule = AppliedRule {
        id: "rule-001".to_string(),
        name: "Test Rule".to_string(),
        version: "1.0.0".to_string(),
        category: "test".to_string(),
        applied_at: Utc::now(),
    };

    assert_eq!(rule.id, "rule-001");
    assert_eq!(rule.name, "Test Rule");
    assert_eq!(rule.version, "1.0.0");
    assert_eq!(rule.category, "test");
}

#[test]
fn test_rule_application_result_empty() {
    let result = RuleApplicationResult {
        rules_applied: vec![],
    };

    assert!(result.rules_applied.is_empty());
}

#[test]
fn test_rule_application_result_with_rules() {
    let rule = AppliedRule {
        id: "rule-001".to_string(),
        name: "Test Rule".to_string(),
        version: "1.0.0".to_string(),
        category: "test".to_string(),
        applied_at: Utc::now(),
    };

    let result = RuleApplicationResult {
        rules_applied: vec![rule.clone()],
    };

    assert_eq!(result.rules_applied.len(), 1);
    assert_eq!(result.rules_applied[0].id, "rule-001");
}

#[test]
fn test_action_executor_new() {
    let plugin_manager = Arc::new(RulePluginManager::new(Arc::new(
        DummyPluginManager::default(),
    )));
    let executor = ActionExecutor::new(plugin_manager.clone());

    assert!(executor.default_context().is_none());
    assert!(Arc::ptr_eq(executor.plugin_manager(), &plugin_manager));
}

#[test]
fn test_action_executor_with_default_context() {
    let plugin_manager = Arc::new(RulePluginManager::new(Arc::new(
        DummyPluginManager::default(),
    )));
    let context = json!({"user": "test", "role": "admin"});
    let executor = ActionExecutor::with_default_context(plugin_manager, context.clone());

    assert!(executor.default_context().is_some());
    assert_eq!(
        executor.default_context().expect("test: should succeed"),
        &context
    );
}

#[test]
fn test_action_executor_set_default_context() {
    let plugin_manager = Arc::new(RulePluginManager::new(Arc::new(
        DummyPluginManager::default(),
    )));
    let mut executor = ActionExecutor::new(plugin_manager);

    assert!(executor.default_context().is_none());

    let context = json!({"data": "value"});
    executor.set_default_context(context.clone());

    assert_eq!(
        executor.default_context().expect("test: should succeed"),
        &context
    );
}

#[tokio::test]
async fn test_execute_action_modify_context() {
    let plugin_manager = Arc::new(RulePluginManager::new(Arc::new(
        DummyPluginManager::default(),
    )));
    let executor = ActionExecutor::new(plugin_manager);

    let mut context = json!({"name": "old"});
    let action = RuleAction::ModifyContext {
        path: "name".to_string(),
        value: json!("new"),
    };

    let result = executor.execute_action(&action, &mut context).await;
    assert!(result.is_ok());
    assert_eq!(context["name"], "new");
}

#[tokio::test]
async fn test_execute_action_modify_nested_context() {
    let plugin_manager = Arc::new(RulePluginManager::new(Arc::new(
        DummyPluginManager::default(),
    )));
    let executor = ActionExecutor::new(plugin_manager);

    let mut context = json!({"user": {"name": "old", "age": 30}});
    let action = RuleAction::ModifyContext {
        path: "user.name".to_string(),
        value: json!("new"),
    };

    let result = executor.execute_action(&action, &mut context).await;
    assert!(result.is_ok());
    assert_eq!(context["user"]["name"], "new");
    assert_eq!(context["user"]["age"], 30);
}

#[tokio::test]
async fn test_execute_action_create_recovery_point() {
    let plugin_manager = Arc::new(RulePluginManager::new(Arc::new(
        DummyPluginManager::default(),
    )));
    let executor = ActionExecutor::new(plugin_manager);

    let mut context = json!({});
    let action = RuleAction::CreateRecoveryPoint {
        name: "checkpoint-1".to_string(),
        description: Some("Test checkpoint".to_string()),
    };

    let result = executor.execute_action(&action, &mut context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_execute_action_execute_command() {
    let plugin_manager = Arc::new(RulePluginManager::new(Arc::new(
        DummyPluginManager::default(),
    )));
    let executor = ActionExecutor::new(plugin_manager);

    let mut context = json!({});
    let action = RuleAction::ExecuteCommand {
        command: "echo".to_string(),
        args: Some(vec!["hello".to_string()]),
        working_dir: None,
    };

    let result = executor.execute_action(&action, &mut context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_execute_action_call_api() {
    let plugin_manager = Arc::new(RulePluginManager::new(Arc::new(
        DummyPluginManager::default(),
    )));
    let executor = ActionExecutor::new(plugin_manager);

    let mut context = json!({});
    let action = RuleAction::CallApi {
        url: "https://api.example.com/data".to_string(),
        method: "GET".to_string(),
        headers: None,
        body: None,
        response_path: Some("api_response".to_string()),
    };

    let result = executor.execute_action(&action, &mut context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_execute_rule_actions_with_provided_context() {
    let plugin_manager = Arc::new(RulePluginManager::new(Arc::new(
        DummyPluginManager::default(),
    )));
    let executor = ActionExecutor::new(plugin_manager);

    let mut context = json!({"count": 1});
    let rule = Rule::new("test-rule", "Test Rule").with_action(RuleAction::ModifyContext {
        path: "count".to_string(),
        value: json!(2),
    });

    let result = executor
        .execute_rule_actions(&rule, Some(&mut context))
        .await;

    assert!(result.is_ok());
    let result_value = result.expect("test: should succeed");
    assert_eq!(result_value["count"], 2);
}

#[tokio::test]
async fn test_execute_rule_actions_with_default_context() {
    let plugin_manager = Arc::new(RulePluginManager::new(Arc::new(
        DummyPluginManager::default(),
    )));
    let default_context = json!({"count": 5});
    let executor = ActionExecutor::with_default_context(plugin_manager, default_context);

    let rule = Rule::new("test-rule", "Test Rule").with_action(RuleAction::ModifyContext {
        path: "count".to_string(),
        value: json!(10),
    });

    let result = executor.execute_rule_actions(&rule, None).await;

    assert!(result.is_ok());
    let result_value = result.expect("test: should succeed");
    assert_eq!(result_value["count"], 10);
}

#[tokio::test]
async fn test_execute_rule_actions_multiple_actions() {
    let plugin_manager = Arc::new(RulePluginManager::new(Arc::new(
        DummyPluginManager::default(),
    )));
    let executor = ActionExecutor::new(plugin_manager);

    let mut context = json!({"a": 1, "b": 2});
    let rule = Rule::new("test-rule", "Test Rule")
        .with_action(RuleAction::ModifyContext {
            path: "a".to_string(),
            value: json!(10),
        })
        .with_action(RuleAction::ModifyContext {
            path: "b".to_string(),
            value: json!(20),
        })
        .with_action(RuleAction::ModifyContext {
            path: "c".to_string(),
            value: json!(30),
        });

    let result = executor
        .execute_rule_actions(&rule, Some(&mut context))
        .await;

    assert!(result.is_ok());
    let result_value = result.expect("test: should succeed");
    assert_eq!(result_value["a"], 10);
    assert_eq!(result_value["b"], 20);
    assert_eq!(result_value["c"], 30);
}

#[tokio::test]
async fn test_execute_rule_actions_no_actions() {
    let plugin_manager = Arc::new(RulePluginManager::new(Arc::new(
        DummyPluginManager::default(),
    )));
    let executor = ActionExecutor::new(plugin_manager);

    let mut context = json!({"data": "unchanged"});
    let rule = Rule::new("test-rule", "Test Rule");

    let result = executor
        .execute_rule_actions(&rule, Some(&mut context))
        .await;

    assert!(result.is_ok());
    let result_value = result.expect("test: should succeed");
    assert_eq!(result_value["data"], "unchanged");
}

#[test]
fn test_single_action_result_clone() {
    let result = SingleActionResult {
        action_type: "modify".to_string(),
        success: true,
        error: None,
        result: Some(json!({"status": "ok"})),
        timestamp: Utc::now(),
    };

    let cloned = result.clone();
    assert_eq!(result.action_type, cloned.action_type);
    assert_eq!(result.success, cloned.success);
}

#[test]
fn test_applied_rule_clone() {
    let rule = AppliedRule {
        id: "rule-001".to_string(),
        name: "Test Rule".to_string(),
        version: "1.0.0".to_string(),
        category: "test".to_string(),
        applied_at: Utc::now(),
    };

    let cloned = rule.clone();
    assert_eq!(rule.id, cloned.id);
    assert_eq!(rule.name, cloned.name);
    assert_eq!(rule.version, cloned.version);
    assert_eq!(rule.category, cloned.category);
}

#[test]
fn test_rule_application_result_clone() {
    let rule = AppliedRule {
        id: "rule-001".to_string(),
        name: "Test Rule".to_string(),
        version: "1.0.0".to_string(),
        category: "test".to_string(),
        applied_at: Utc::now(),
    };

    let result = RuleApplicationResult {
        rules_applied: vec![rule],
    };

    let cloned = result.clone();
    assert_eq!(result.rules_applied.len(), cloned.rules_applied.len());
    assert_eq!(result.rules_applied[0].id, cloned.rules_applied[0].id);
}

#[test]
fn test_action_executor_debug() {
    let plugin_manager = Arc::new(RulePluginManager::new(Arc::new(
        DummyPluginManager::default(),
    )));
    let executor = ActionExecutor::new(plugin_manager);

    let debug = format!("{:?}", executor);
    assert!(debug.contains("ActionExecutor"));
}

#[tokio::test]
async fn test_execute_action_modify_context_create_new_field() {
    let plugin_manager = Arc::new(RulePluginManager::new(Arc::new(
        DummyPluginManager::default(),
    )));
    let executor = ActionExecutor::new(plugin_manager);

    let mut context = json!({"existing": "value"});
    let action = RuleAction::ModifyContext {
        path: "new_field".to_string(),
        value: json!("new_value"),
    };

    let result = executor.execute_action(&action, &mut context).await;
    assert!(result.is_ok());
    assert_eq!(context["existing"], "value");
    assert_eq!(context["new_field"], "new_value");
}

#[tokio::test]
async fn test_execute_action_modify_context_deep_nested() {
    let plugin_manager = Arc::new(RulePluginManager::new(Arc::new(
        DummyPluginManager::default(),
    )));
    let executor = ActionExecutor::new(plugin_manager);

    let mut context = json!({
        "level1": {
            "level2": {
                "level3": {
                    "value": "old"
                }
            }
        }
    });

    let action = RuleAction::ModifyContext {
        path: "level1.level2.level3.value".to_string(),
        value: json!("new"),
    };

    let result = executor.execute_action(&action, &mut context).await;
    assert!(result.is_ok());
    assert_eq!(context["level1"]["level2"]["level3"]["value"], "new");
}

#[tokio::test]
async fn test_execute_action_create_recovery_point_without_description() {
    let plugin_manager = Arc::new(RulePluginManager::new(Arc::new(
        DummyPluginManager::default(),
    )));
    let executor = ActionExecutor::new(plugin_manager);

    let mut context = json!({});
    let action = RuleAction::CreateRecoveryPoint {
        name: "checkpoint-1".to_string(),
        description: None,
    };

    let result = executor.execute_action(&action, &mut context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_execute_action_execute_command_with_working_dir() {
    let plugin_manager = Arc::new(RulePluginManager::new(Arc::new(
        DummyPluginManager::default(),
    )));
    let executor = ActionExecutor::new(plugin_manager);

    let mut context = json!({});
    let action = RuleAction::ExecuteCommand {
        command: "ls".to_string(),
        args: Some(vec!["-la".to_string()]),
        working_dir: Some("/tmp".to_string()),
    };

    let result = executor.execute_action(&action, &mut context).await;
    assert!(result.is_ok());
}
