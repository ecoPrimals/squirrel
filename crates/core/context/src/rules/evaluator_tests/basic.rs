// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Basic evaluator tests - creation, Exists, Match, Compare conditions

use crate::rules::evaluator::RuleEvaluator;
use crate::rules::models::RuleCondition;
use serde_json::json;

#[test]
fn test_evaluator_new() {
    let _evaluator = RuleEvaluator::new();
    assert!(true, "Should create evaluator successfully");
}

#[test]
fn test_evaluator_default() {
    let _evaluator = RuleEvaluator::default();
    assert!(true, "Should create evaluator with default");
}

#[tokio::test]
async fn test_evaluate_condition_exists_true() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "user": {
            "name": "Alice",
            "age": 30
        }
    });

    let condition = RuleCondition::Exists {
        path: "user.name".to_string(),
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(result.is_ok());
    assert!(
        result.expect("test: operation should succeed"),
        "Should find existing path"
    );
}

#[tokio::test]
async fn test_evaluate_condition_exists_false() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "user": {
            "name": "Alice"
        }
    });

    let condition = RuleCondition::Exists {
        path: "user.email".to_string(),
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(result.is_ok());
    assert!(
        !result.expect("test: operation should succeed"),
        "Should not find non-existing path"
    );
}

#[tokio::test]
async fn test_evaluate_condition_match_string() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "user": {
            "email": "alice@example.com"
        }
    });

    let condition = RuleCondition::Match {
        path: "user.email".to_string(),
        pattern: ".*@example\\.com$".to_string(),
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(result.is_ok());
    assert!(
        result.expect("test: operation should succeed"),
        "Should match email pattern"
    );
}

#[tokio::test]
async fn test_evaluate_condition_match_no_match() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "user": {
            "email": "alice@example.com"
        }
    });

    let condition = RuleCondition::Match {
        path: "user.email".to_string(),
        pattern: "^bob@".to_string(),
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(result.is_ok());
    assert!(
        !result.expect("test: operation should succeed"),
        "Should not match different pattern"
    );
}

#[tokio::test]
async fn test_evaluate_condition_match_number() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "user": {
            "age": 42
        }
    });

    let condition = RuleCondition::Match {
        path: "user.age".to_string(),
        pattern: "^4[0-9]$".to_string(),
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(result.is_ok());
    assert!(
        result.expect("test: operation should succeed"),
        "Should match number converted to string"
    );
}

#[tokio::test]
async fn test_evaluate_condition_match_missing_path() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "user": {}
    });

    let condition = RuleCondition::Match {
        path: "user.email".to_string(),
        pattern: ".*".to_string(),
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(result.is_ok());
    assert!(
        !result.expect("test: operation should succeed"),
        "Should return false for missing path"
    );
}

#[tokio::test]
async fn test_evaluate_condition_compare_eq() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "user": {
            "status": "active",
            "expected": "active"
        }
    });

    let condition = RuleCondition::Compare {
        path1: "user.status".to_string(),
        path2: "user.expected".to_string(),
        operator: "eq".to_string(),
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(result.is_ok());
    assert!(
        result.expect("test: operation should succeed"),
        "Should return true for equal values"
    );
}

#[tokio::test]
async fn test_evaluate_condition_compare_ne() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "user": {
            "status": "active",
            "other": "inactive"
        }
    });

    let condition = RuleCondition::Compare {
        path1: "user.status".to_string(),
        path2: "user.other".to_string(),
        operator: "ne".to_string(),
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(result.is_ok());
    assert!(
        result.expect("test: operation should succeed"),
        "Should return true for non-equal values"
    );
}

#[tokio::test]
async fn test_evaluate_condition_compare_gt() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "metrics": {
            "current": 100,
            "threshold": 50
        }
    });

    let condition = RuleCondition::Compare {
        path1: "metrics.current".to_string(),
        path2: "metrics.threshold".to_string(),
        operator: "gt".to_string(),
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(result.is_ok());
    assert!(
        result.expect("test: operation should succeed"),
        "Should return true for greater than"
    );
}

#[tokio::test]
async fn test_evaluate_condition_compare_lt() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "metrics": {
            "current": 25,
            "threshold": 50
        }
    });

    let condition = RuleCondition::Compare {
        path1: "metrics.current".to_string(),
        path2: "metrics.threshold".to_string(),
        operator: "lt".to_string(),
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(result.is_ok());
    assert!(
        result.expect("test: operation should succeed"),
        "Should return true for less than"
    );
}

#[tokio::test]
async fn test_evaluate_condition_compare_ge() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "metrics": {
            "current": 50,
            "threshold": 50
        }
    });

    let condition = RuleCondition::Compare {
        path1: "metrics.current".to_string(),
        path2: "metrics.threshold".to_string(),
        operator: "ge".to_string(),
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(result.is_ok());
    assert!(
        result.expect("test: operation should succeed"),
        "Should return true for greater than or equal"
    );
}

#[tokio::test]
async fn test_evaluate_condition_compare_le() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "metrics": {
            "current": 50,
            "threshold": 100
        }
    });

    let condition = RuleCondition::Compare {
        path1: "metrics.current".to_string(),
        path2: "metrics.threshold".to_string(),
        operator: "le".to_string(),
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(result.is_ok());
    assert!(
        result.expect("test: operation should succeed"),
        "Should return true for less than or equal"
    );
}

#[tokio::test]
async fn test_evaluate_condition_compare_missing_path() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "metrics": {
            "current": 50
        }
    });

    let condition = RuleCondition::Compare {
        path1: "metrics.current".to_string(),
        path2: "metrics.missing".to_string(),
        operator: "eq".to_string(),
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(result.is_ok());
    assert!(
        !result.expect("test: operation should succeed"),
        "Should return false for missing path"
    );
}

#[tokio::test]
async fn test_evaluate_condition_javascript_not_implemented() {
    let evaluator = RuleEvaluator::new();
    let context = json!({});

    let condition = RuleCondition::JavaScript {
        expression: "true".to_string(),
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(result.is_ok());
    assert!(
        !result.expect("test: operation should succeed"),
        "JavaScript condition not implemented"
    );
}

#[tokio::test]
async fn test_evaluate_condition_custom_not_implemented() {
    let evaluator = RuleEvaluator::new();
    let context = json!({});

    let condition = RuleCondition::Custom {
        id: "custom-check".to_string(),
        config: json!({}),
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(result.is_ok());
    assert!(
        !result.expect("test: operation should succeed"),
        "Custom condition not implemented"
    );
}
