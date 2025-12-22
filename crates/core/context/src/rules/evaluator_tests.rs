//! Tests for the Rules Evaluator
//!
//! These tests cover condition evaluation, pattern matching, value comparison,
//! and rule matching logic.

#[cfg(test)]
mod tests {
    use crate::rules::evaluator::RuleEvaluator;
    use crate::rules::models::{Rule, RuleAction, RuleCondition, RuleMetadata};
    use serde_json::json;
    use std::sync::Arc;

    /// Helper to create a test rule
    fn create_test_rule(id: &str, priority: i32, conditions: Vec<RuleCondition>) -> Arc<Rule> {
        Arc::new(Rule {
            id: id.to_string(),
            name: format!("Test Rule {}", id),
            description: "Test rule".to_string(),
            version: "1.0.0".to_string(),
            category: "test".to_string(),
            priority,
            patterns: vec!["test".to_string()],
            conditions,
            actions: vec![RuleAction::LogMessage {
                level: "info".to_string(),
                message: "test".to_string(),
            }],
            metadata: RuleMetadata::new(),
        })
    }

    /// Test evaluator creation
    #[test]
    fn test_evaluator_new() {
        let _evaluator = RuleEvaluator::new();
        assert!(true, "Should create evaluator successfully");
    }

    /// Test evaluator default
    #[test]
    fn test_evaluator_default() {
        let _evaluator = RuleEvaluator::default();
        assert!(true, "Should create evaluator with default");
    }

    /// Test Exists condition with existing path
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

    /// Test Exists condition with non-existing path
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

    /// Test Match condition with string value
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

    /// Test Match condition with non-matching pattern
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

    /// Test Match condition with non-string value (converts to string)
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

    /// Test Match condition with non-existing path
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

    /// Test Compare condition with eq operator
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

    /// Test Compare condition with ne operator
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

    /// Test Compare condition with gt operator
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

    /// Test Compare condition with lt operator
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

    /// Test Compare condition with ge operator
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

    /// Test Compare condition with le operator
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

    /// Test Compare condition with missing path (should return false)
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

    /// Test JavaScript condition (not implemented, should return false)
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

    /// Test Custom condition (not implemented, should return false)
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

    /// Test path navigation with nested objects
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

    /// Test path navigation with arrays
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

    /// Test path navigation with invalid array index
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

    /// Test finding matching rules
    #[tokio::test]
    async fn test_find_matching_rules_single() {
        let evaluator = RuleEvaluator::new();
        let context = json!({
            "user": {
                "role": "admin"
            }
        });

        let rule = create_test_rule(
            "rule-1",
            100,
            vec![RuleCondition::Match {
                path: "user.role".to_string(),
                pattern: "admin".to_string(),
            }],
        );

        let rules = vec![rule.clone()];
        let result = evaluator.find_matching_rules(&rules, &context).await;

        assert!(result.is_ok());
        let matching = result.expect("test: operation should succeed");
        assert_eq!(matching.len(), 1);
        assert_eq!(matching[0].id, "rule-1");
    }

    /// Test finding no matching rules
    #[tokio::test]
    async fn test_find_matching_rules_none() {
        let evaluator = RuleEvaluator::new();
        let context = json!({
            "user": {
                "role": "user"
            }
        });

        let rule = create_test_rule(
            "rule-1",
            100,
            vec![RuleCondition::Match {
                path: "user.role".to_string(),
                pattern: "admin".to_string(),
            }],
        );

        let rules = vec![rule];
        let result = evaluator.find_matching_rules(&rules, &context).await;

        assert!(result.is_ok());
        let matching = result.expect("test: operation should succeed");
        assert_eq!(matching.len(), 0, "Should find no matching rules");
    }

    /// Test finding multiple matching rules sorted by priority
    #[tokio::test]
    async fn test_find_matching_rules_sorted_by_priority() {
        let evaluator = RuleEvaluator::new();
        let context = json!({
            "user": {
                "active": true
            }
        });

        let rule1 = create_test_rule(
            "rule-low",
            500,
            vec![RuleCondition::Exists {
                path: "user.active".to_string(),
            }],
        );

        let rule2 = create_test_rule(
            "rule-high",
            100,
            vec![RuleCondition::Exists {
                path: "user.active".to_string(),
            }],
        );

        let rule3 = create_test_rule(
            "rule-medium",
            300,
            vec![RuleCondition::Exists {
                path: "user.active".to_string(),
            }],
        );

        let rules = vec![rule1, rule2, rule3];
        let result = evaluator.find_matching_rules(&rules, &context).await;

        assert!(result.is_ok());
        let matching = result.expect("test: operation should succeed");
        assert_eq!(matching.len(), 3, "Should find all matching rules");

        // Should be sorted by priority (lower is higher priority)
        assert_eq!(matching[0].id, "rule-high");
        assert_eq!(matching[1].id, "rule-medium");
        assert_eq!(matching[2].id, "rule-low");
    }

    /// Test rule with multiple conditions (all must match)
    #[tokio::test]
    async fn test_find_matching_rules_multiple_conditions() {
        let evaluator = RuleEvaluator::new();
        let context = json!({
            "user": {
                "role": "admin",
                "active": true
            }
        });

        let rule = create_test_rule(
            "rule-multi",
            100,
            vec![
                RuleCondition::Match {
                    path: "user.role".to_string(),
                    pattern: "admin".to_string(),
                },
                RuleCondition::Exists {
                    path: "user.active".to_string(),
                },
            ],
        );

        let rules = vec![rule];
        let result = evaluator.find_matching_rules(&rules, &context).await;

        assert!(result.is_ok());
        let matching = result.expect("test: operation should succeed");
        assert_eq!(
            matching.len(),
            1,
            "Should match when all conditions are met"
        );
    }

    /// Test rule with multiple conditions (one fails)
    #[tokio::test]
    async fn test_find_matching_rules_condition_fails() {
        let evaluator = RuleEvaluator::new();
        let context = json!({
            "user": {
                "role": "user"
            }
        });

        let rule = create_test_rule(
            "rule-multi",
            100,
            vec![
                RuleCondition::Match {
                    path: "user.role".to_string(),
                    pattern: "admin".to_string(),
                },
                RuleCondition::Exists {
                    path: "user.active".to_string(),
                },
            ],
        );

        let rules = vec![rule];
        let result = evaluator.find_matching_rules(&rules, &context).await;

        assert!(result.is_ok());
        let matching = result.expect("test: operation should succeed");
        assert_eq!(
            matching.len(),
            0,
            "Should not match when any condition fails"
        );
    }

    /// Test regex caching (multiple matches with same pattern)
    #[tokio::test]
    async fn test_regex_caching() {
        let evaluator = RuleEvaluator::new();
        let context1 = json!({"email": "alice@example.com"});
        let context2 = json!({"email": "bob@example.com"});

        let pattern = ".*@example\\.com$".to_string();

        let condition1 = RuleCondition::Match {
            path: "email".to_string(),
            pattern: pattern.clone(),
        };

        let condition2 = RuleCondition::Match {
            path: "email".to_string(),
            pattern: pattern.clone(),
        };

        // First evaluation (should compile and cache regex)
        let result1 = evaluator.evaluate_condition(&condition1, &context1).await;
        assert!(result1.is_ok());
        assert!(result1.expect("test: operation should succeed"));

        // Second evaluation (should use cached regex)
        let result2 = evaluator.evaluate_condition(&condition2, &context2).await;
        assert!(result2.is_ok());
        assert!(result2.expect("test: operation should succeed"));
    }

    /// Test invalid regex pattern (should return error)
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

    /// Test unknown comparison operator (should return error)
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

    /// Test numeric comparison with non-numeric string (should return error)
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

    /// Test numeric comparison with boolean (should convert to 0 or 1)
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

    /// Test numeric comparison with boolean false  
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

    /// Test numeric comparison with valid numeric string
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

    /// Test numeric comparison with array (should return error)
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

    /// Test numeric comparison with object (eq works, numeric comparison fails)
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

    /// Test path navigation with non-numeric array index
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

    /// Test path navigation through non-object, non-array
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

    /// Test empty path
    #[tokio::test]
    async fn test_empty_path() {
        let evaluator = RuleEvaluator::new();
        let context = json!({"value": 42});

        let condition = RuleCondition::Exists {
            path: "".to_string(),
        };

        let result = evaluator.evaluate_condition(&condition, &context).await;
        assert!(result.is_ok());
        // Empty path splits to [""], tries to look up "" key which doesn't exist
        assert!(!result.expect("test: operation should succeed"));
    }

    /// Test complex nested array and object navigation
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

    /// Test ge operator with equal values
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

    /// Test le operator with equal values
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

    /// Test finding rules with empty list
    #[tokio::test]
    async fn test_find_matching_rules_empty_list() {
        let evaluator = RuleEvaluator::new();
        let context = json!({"data": "value"});

        let rules: Vec<Arc<Rule>> = vec![];
        let result = evaluator.find_matching_rules(&rules, &context).await;

        assert!(result.is_ok());
        let matching = result.expect("test: operation should succeed");
        assert_eq!(
            matching.len(),
            0,
            "Should return empty list for empty rules"
        );
    }

    /// Test rule with zero conditions (should always match)
    #[tokio::test]
    async fn test_rule_with_zero_conditions() {
        let evaluator = RuleEvaluator::new();
        let context = json!({"data": "value"});

        let rule = create_test_rule("rule-unconditional", 100, vec![]);

        let rules = vec![rule];
        let result = evaluator.find_matching_rules(&rules, &context).await;

        assert!(result.is_ok());
        let matching = result.expect("test: operation should succeed");
        assert_eq!(
            matching.len(),
            1,
            "Rule with no conditions should always match"
        );
    }
}
