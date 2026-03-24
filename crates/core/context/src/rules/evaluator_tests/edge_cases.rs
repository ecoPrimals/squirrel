// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Edge case tests - path navigation, invalid inputs, type conversions

use crate::rules::evaluator::RuleEvaluator;
use crate::rules::models::RuleCondition;
use serde_json::json;

#[tokio::test]
async fn test_path_navigation_nested() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "level1": {
            "level2": {
                "level3": {
                    "value": "deep"
                }
            }
        }
    });

    let condition = RuleCondition::Exists {
        path: "level1.level2.level3.value".to_string(),
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(result.is_ok());
    assert!(
        result.expect("test: operation should succeed"),
        "Should navigate deeply nested path"
    );
}

#[tokio::test]
async fn test_path_navigation_array() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "users": [
            {"name": "Alice"},
            {"name": "Bob"},
            {"name": "Charlie"}
        ]
    });

    let condition = RuleCondition::Match {
        path: "users.1.name".to_string(),
        pattern: "Bob".to_string(),
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(result.is_ok());
    assert!(
        result.expect("test: operation should succeed"),
        "Should navigate array with index"
    );
}

#[tokio::test]
async fn test_path_navigation_array_out_of_bounds() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "users": [
            {"name": "Alice"}
        ]
    });

    let condition = RuleCondition::Exists {
        path: "users.5.name".to_string(),
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(result.is_ok());
    assert!(
        !result.expect("test: operation should succeed"),
        "Should return false for out of bounds index"
    );
}

#[tokio::test]
async fn test_invalid_regex_pattern() {
    let evaluator = RuleEvaluator::new();
    let context = json!({"value": "test"});

    let condition = RuleCondition::Match {
        path: "value".to_string(),
        pattern: "[invalid(".to_string(), // Invalid regex
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(result.is_err(), "Should return error for invalid regex");
}

#[tokio::test]
async fn test_unknown_comparison_operator() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "a": 5,
        "b": 10
    });

    let condition = RuleCondition::Compare {
        path1: "a".to_string(),
        path2: "b".to_string(),
        operator: "unknown".to_string(),
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(result.is_err(), "Should return error for unknown operator");
}

#[tokio::test]
async fn test_numeric_comparison_invalid_string() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "a": "not_a_number",
        "b": 10
    });

    let condition = RuleCondition::Compare {
        path1: "a".to_string(),
        path2: "b".to_string(),
        operator: "gt".to_string(),
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(
        result.is_err(),
        "Should return error for invalid numeric conversion"
    );
}

#[tokio::test]
async fn test_numeric_comparison_boolean() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "a": true,
        "b": 1
    });

    let condition = RuleCondition::Compare {
        path1: "a".to_string(),
        path2: "b".to_string(),
        operator: "gt".to_string(), // Use gt since eq compares values directly, not numbers
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(result.is_ok());
    assert!(
        !result.expect("test: operation should succeed"),
        "Boolean true (1.0) is not greater than 1"
    );
}

#[tokio::test]
async fn test_numeric_comparison_boolean_false() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "a": false,
        "b": 1
    });

    let condition = RuleCondition::Compare {
        path1: "a".to_string(),
        path2: "b".to_string(),
        operator: "lt".to_string(), // Use lt to test boolean false (0.0) < 1
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(result.is_ok());
    assert!(
        result.expect("test: operation should succeed"),
        "Boolean false (0.0) should be less than 1"
    );
}

#[tokio::test]
async fn test_numeric_comparison_valid_string() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "a": "42.5",
        "b": 50
    });

    let condition = RuleCondition::Compare {
        path1: "a".to_string(),
        path2: "b".to_string(),
        operator: "lt".to_string(),
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(result.is_ok());
    assert!(
        result.expect("test: operation should succeed"),
        "String '42.5' should be less than 50"
    );
}

#[tokio::test]
async fn test_numeric_comparison_array() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "a": [1, 2, 3],
        "b": 5
    });

    let condition = RuleCondition::Compare {
        path1: "a".to_string(),
        path2: "b".to_string(),
        operator: "gt".to_string(),
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(
        result.is_err(),
        "Should return error for array to number conversion"
    );
}

#[tokio::test]
async fn test_numeric_comparison_object() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "a": {"value": 5},
        "b": 5
    });

    let condition = RuleCondition::Compare {
        path1: "a".to_string(),
        path2: "b".to_string(),
        operator: "gt".to_string(), // Use gt to force numeric conversion which should fail
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(
        result.is_err(),
        "Should return error for object to number conversion in numeric comparison"
    );
}

#[tokio::test]
async fn test_path_navigation_array_non_numeric() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "items": [1, 2, 3]
    });

    let condition = RuleCondition::Exists {
        path: "items.invalid.value".to_string(),
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(result.is_ok());
    assert!(
        !result.expect("test: operation should succeed"),
        "Should return false for non-numeric array index"
    );
}

#[tokio::test]
async fn test_path_navigation_through_primitive() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "value": "string"
    });

    let condition = RuleCondition::Exists {
        path: "value.nested".to_string(),
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(result.is_ok());
    assert!(
        !result.expect("test: operation should succeed"),
        "Should return false when trying to navigate through primitive"
    );
}

#[tokio::test]
async fn test_empty_path() {
    let evaluator = RuleEvaluator::new();
    let context = json!({"value": 42});

    let condition = RuleCondition::Exists {
        path: String::new(),
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(result.is_ok());
    // Empty path splits to [""], tries to look up "" key which doesn't exist
    assert!(!result.expect("test: operation should succeed"));
}

#[tokio::test]
async fn test_complex_nested_navigation() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "data": {
            "users": [
                {
                    "profile": {
                        "contacts": [
                            {"type": "email", "value": "alice@example.com"},
                            {"type": "phone", "value": "+1234567890"}
                        ]
                    }
                },
                {
                    "profile": {
                        "contacts": [
                            {"type": "email", "value": "bob@example.com"}
                        ]
                    }
                }
            ]
        }
    });

    let condition = RuleCondition::Match {
        path: "data.users.1.profile.contacts.0.value".to_string(),
        pattern: "bob@.*".to_string(),
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(result.is_ok());
    assert!(
        result.expect("test: operation should succeed"),
        "Should navigate complex nested structure"
    );
}

#[tokio::test]
async fn test_compare_ge_equal() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "a": 100,
        "b": 100
    });

    let condition = RuleCondition::Compare {
        path1: "a".to_string(),
        path2: "b".to_string(),
        operator: "ge".to_string(),
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(result.is_ok());
    assert!(
        result.expect("test: operation should succeed"),
        "Greater than or equal should be true for equal values"
    );
}

#[tokio::test]
async fn test_compare_le_equal() {
    let evaluator = RuleEvaluator::new();
    let context = json!({
        "a": 50,
        "b": 50
    });

    let condition = RuleCondition::Compare {
        path1: "a".to_string(),
        path2: "b".to_string(),
        operator: "le".to_string(),
    };

    let result = evaluator.evaluate_condition(&condition, &context).await;
    assert!(result.is_ok());
    assert!(
        result.expect("test: operation should succeed"),
        "Less than or equal should be true for equal values"
    );
}
