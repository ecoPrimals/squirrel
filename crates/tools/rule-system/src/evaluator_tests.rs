//! Comprehensive tests for rule evaluator

use super::evaluator::*;
use super::models::*;
use serde_json::json;

// Helper function to create a test rule
fn create_test_rule(id: &str, conditions: Vec<RuleCondition>) -> Rule {
    let mut rule = Rule::new(id);
    rule.name = "Test Rule".to_string();
    rule.priority = 1;
    rule.conditions = conditions;
    rule
}

#[tokio::test]
async fn test_evaluator_creation() {
    let evaluator = RuleEvaluator::new();
    let stats = evaluator.get_statistics().await;
    assert_eq!(stats.total_evaluations, 0);
    assert_eq!(stats.cached_evaluations, 0);
    assert_eq!(stats.successful_evaluations, 0);
}

#[tokio::test]
async fn test_evaluator_default() {
    let evaluator = RuleEvaluator::default();
    let stats = evaluator.get_statistics().await;
    assert_eq!(stats.total_evaluations, 0);
}

#[tokio::test]
async fn test_evaluator_initialize() {
    let evaluator = RuleEvaluator::new();
    let result = evaluator.initialize().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_equals_condition_match() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "user": {
            "name": "John"
        }
    });

    let rule = create_test_rule(
        "test_rule",
        vec![RuleCondition::Equals {
            path: "user.name".to_string(),
            value: json!("John"),
        }],
    );

    let result = evaluator
        .evaluate_rule(&rule, "context1", &context)
        .await
        .unwrap();
    assert!(result.matches);
}

#[tokio::test]
async fn test_equals_condition_no_match() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "user": {
            "name": "Jane"
        }
    });

    let rule = create_test_rule(
        "test_rule",
        vec![RuleCondition::Equals {
            path: "user.name".to_string(),
            value: json!("John"),
        }],
    );

    let result = evaluator
        .evaluate_rule(&rule, "context1", &context)
        .await
        .unwrap();
    assert!(!result.matches);
}

#[tokio::test]
async fn test_greater_than_condition() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "user": {
            "age": 30
        }
    });

    let rule = create_test_rule(
        "test_rule",
        vec![RuleCondition::GreaterThan {
            path: "user.age".to_string(),
            value: json!(25),
        }],
    );

    let result = evaluator
        .evaluate_rule(&rule, "context1", &context)
        .await
        .unwrap();
    assert!(result.matches);
}

#[tokio::test]
async fn test_less_than_condition() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "user": {
            "age": 20
        }
    });

    let rule = create_test_rule(
        "test_rule",
        vec![RuleCondition::LessThan {
            path: "user.age".to_string(),
            value: json!(25),
        }],
    );

    let result = evaluator
        .evaluate_rule(&rule, "context1", &context)
        .await
        .unwrap();
    assert!(result.matches);
}

#[tokio::test]
async fn test_exists_condition() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "user": {
            "email": "test@example.com"
        }
    });

    let rule = create_test_rule(
        "test_rule",
        vec![RuleCondition::Exists {
            path: "user.email".to_string(),
        }],
    );

    let result = evaluator
        .evaluate_rule(&rule, "context1", &context)
        .await
        .unwrap();
    assert!(result.matches);
}

#[tokio::test]
async fn test_not_exists_condition() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "user": {
            "name": "John"
        }
    });

    let rule = create_test_rule(
        "test_rule",
        vec![RuleCondition::NotExists {
            path: "user.email".to_string(),
        }],
    );

    let result = evaluator
        .evaluate_rule(&rule, "context1", &context)
        .await
        .unwrap();
    assert!(result.matches);
}

#[tokio::test]
async fn test_all_conditions_match() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "user": {
            "name": "John",
            "age": 30
        }
    });

    let rule = create_test_rule(
        "test_rule",
        vec![RuleCondition::All {
            conditions: vec![
                RuleCondition::Equals {
                    path: "user.name".to_string(),
                    value: json!("John"),
                },
                RuleCondition::GreaterThan {
                    path: "user.age".to_string(),
                    value: json!(25),
                },
            ],
        }],
    );

    let result = evaluator
        .evaluate_rule(&rule, "context1", &context)
        .await
        .unwrap();
    assert!(result.matches);
}

#[tokio::test]
async fn test_all_conditions_no_match() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "user": {
            "name": "John",
            "age": 20
        }
    });

    let rule = create_test_rule(
        "test_rule",
        vec![RuleCondition::All {
            conditions: vec![
                RuleCondition::Equals {
                    path: "user.name".to_string(),
                    value: json!("John"),
                },
                RuleCondition::GreaterThan {
                    path: "user.age".to_string(),
                    value: json!(25),
                },
            ],
        }],
    );

    let result = evaluator
        .evaluate_rule(&rule, "context1", &context)
        .await
        .unwrap();
    assert!(!result.matches);
}

#[tokio::test]
async fn test_any_conditions_match() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "user": {
            "name": "Jane",
            "age": 30
        }
    });

    let rule = create_test_rule(
        "test_rule",
        vec![RuleCondition::Any {
            conditions: vec![
                RuleCondition::Equals {
                    path: "user.name".to_string(),
                    value: json!("John"),
                },
                RuleCondition::GreaterThan {
                    path: "user.age".to_string(),
                    value: json!(25),
                },
            ],
        }],
    );

    let result = evaluator
        .evaluate_rule(&rule, "context1", &context)
        .await
        .unwrap();
    assert!(result.matches);
}

#[tokio::test]
async fn test_not_condition() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "user": {
            "name": "Jane"
        }
    });

    let rule = create_test_rule(
        "test_rule",
        vec![RuleCondition::Not {
            condition: Box::new(RuleCondition::Equals {
                path: "user.name".to_string(),
                value: json!("John"),
            }),
        }],
    );

    let result = evaluator
        .evaluate_rule(&rule, "context1", &context)
        .await
        .unwrap();
    assert!(result.matches);
}

#[tokio::test]
async fn test_empty_conditions() {
    let evaluator = RuleEvaluator::new();
    let context = json!({});

    let rule = create_test_rule("test_rule", vec![]);

    let result = evaluator
        .evaluate_rule(&rule, "context1", &context)
        .await
        .unwrap();
    assert!(result.matches);
}

#[tokio::test]
async fn test_caching_behavior() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "user": {
            "name": "John"
        }
    });

    let rule = create_test_rule(
        "test_rule",
        vec![RuleCondition::Equals {
            path: "user.name".to_string(),
            value: json!("John"),
        }],
    );

    // First evaluation
    evaluator
        .evaluate_rule(&rule, "context1", &context)
        .await
        .unwrap();

    // Second evaluation (should be cached)
    evaluator
        .evaluate_rule(&rule, "context1", &context)
        .await
        .unwrap();

    let stats = evaluator.get_statistics().await;
    assert_eq!(stats.total_evaluations, 2);
    assert_eq!(stats.cached_evaluations, 1);
}

#[tokio::test]
async fn test_clear_cache() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "user": {
            "name": "John"
        }
    });

    let rule = create_test_rule(
        "test_rule",
        vec![RuleCondition::Equals {
            path: "user.name".to_string(),
            value: json!("John"),
        }],
    );

    // First evaluation
    evaluator
        .evaluate_rule(&rule, "context1", &context)
        .await
        .unwrap();

    // Clear cache
    evaluator.clear_cache().await.unwrap();

    // Second evaluation (should not be cached)
    evaluator
        .evaluate_rule(&rule, "context1", &context)
        .await
        .unwrap();

    let stats = evaluator.get_statistics().await;
    assert_eq!(stats.total_evaluations, 2);
    assert_eq!(stats.cached_evaluations, 0);
}

#[tokio::test]
async fn test_reset_statistics() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "value": 10
    });

    let rule = create_test_rule(
        "test_rule",
        vec![RuleCondition::GreaterThan {
            path: "value".to_string(),
            value: json!(5),
        }],
    );

    // Evaluate once
    evaluator
        .evaluate_rule(&rule, "context1", &context)
        .await
        .unwrap();

    // Reset stats
    evaluator.reset_statistics().await.unwrap();

    let stats = evaluator.get_statistics().await;
    assert_eq!(stats.total_evaluations, 0);
    assert_eq!(stats.successful_evaluations, 0);
    assert_eq!(stats.cached_evaluations, 0);
}

#[tokio::test]
async fn test_get_registered_evaluators() {
    let evaluator = RuleEvaluator::new();
    let evaluators = evaluator.get_registered_evaluators().await;
    // By default, should be empty
    assert_eq!(evaluators.len(), 0);
}

#[tokio::test]
async fn test_create_rule_evaluator() {
    let result = create_rule_evaluator();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_create_rule_evaluator_with_config() {
    let result = create_rule_evaluator_with_config();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_nested_conditions() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "user": {
            "name": "John",
            "age": 30
        }
    });

    let rule = create_test_rule(
        "test_rule",
        vec![RuleCondition::All {
            conditions: vec![
                RuleCondition::Any {
                    conditions: vec![
                        RuleCondition::Equals {
                            path: "user.name".to_string(),
                            value: json!("John"),
                        },
                        RuleCondition::Equals {
                            path: "user.name".to_string(),
                            value: json!("Jane"),
                        },
                    ],
                },
                RuleCondition::GreaterThan {
                    path: "user.age".to_string(),
                    value: json!(25),
                },
            ],
        }],
    );

    let result = evaluator
        .evaluate_rule(&rule, "context1", &context)
        .await
        .unwrap();
    assert!(result.matches);
}
