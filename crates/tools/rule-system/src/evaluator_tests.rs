// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Comprehensive tests for rule evaluator

#[cfg(test)]
mod tests {
    use crate::evaluator::RuleEvaluator;
    use crate::models::{Rule, RuleCondition};
    use serde_json::{Value, json};

    /// Helper to create test context data
    fn test_context() -> Value {
        json!({
            "user": {
                "name": "Test User",
                "age": 30,
                "active": true,
                "roles": ["admin", "user"]
            },
            "session": {
                "id": "session123",
                "duration": 3600
            },
            "metrics": {
                "cpu": 45.5,
                "memory": 78.2
            }
        })
    }

    #[tokio::test]
    async fn test_evaluator_creation() {
        let evaluator = RuleEvaluator::new();
        evaluator.initialize();
        // If we get here without panic, initialization succeeded
    }

    #[tokio::test]
    async fn test_evaluate_equals_condition_match() {
        let evaluator = RuleEvaluator::new();
        evaluator.initialize();

        let rule = Rule::new("test-equals")
            .with_name("Equals Test")
            .with_condition(RuleCondition::Equals {
                path: "user.name".to_string(),
                value: json!("Test User"),
            });

        let context = test_context();
        let result = evaluator
            .evaluate_rule(&rule, "ctx1", &context)
            .await
            .expect("should succeed");

        assert!(result.matches);
        assert_eq!(result.rule_id, "test-equals");
        assert_eq!(result.context_id, "ctx1");
    }

    #[tokio::test]
    async fn test_evaluate_equals_condition_no_match() {
        let evaluator = RuleEvaluator::new();
        evaluator.initialize();

        let rule = Rule::new("test-equals-nomatch")
            .with_name("Equals No Match")
            .with_condition(RuleCondition::Equals {
                path: "user.name".to_string(),
                value: json!("Wrong Name"),
            });

        let context = test_context();
        let result = evaluator
            .evaluate_rule(&rule, "ctx1", &context)
            .await
            .expect("should succeed");

        assert!(!result.matches);
    }

    #[tokio::test]
    async fn test_evaluate_exists_condition() {
        let evaluator = RuleEvaluator::new();
        evaluator.initialize();

        let rule = Rule::new("test-exists")
            .with_name("Exists Test")
            .with_condition(RuleCondition::Exists {
                path: "user.name".to_string(),
            });

        let context = test_context();
        let result = evaluator
            .evaluate_rule(&rule, "ctx1", &context)
            .await
            .expect("should succeed");

        assert!(result.matches);
    }

    #[tokio::test]
    async fn test_evaluate_not_exists_condition() {
        let evaluator = RuleEvaluator::new();
        evaluator.initialize();

        let rule = Rule::new("test-not-exists")
            .with_name("Not Exists Test")
            .with_condition(RuleCondition::NotExists {
                path: "nonexistent.field".to_string(),
            });

        let context = test_context();
        let result = evaluator
            .evaluate_rule(&rule, "ctx1", &context)
            .await
            .expect("should succeed");

        assert!(result.matches);
    }

    #[tokio::test]
    async fn test_evaluate_greater_than_condition() {
        let evaluator = RuleEvaluator::new();
        evaluator.initialize();

        let rule = Rule::new("test-gt")
            .with_name("Greater Than Test")
            .with_condition(RuleCondition::GreaterThan {
                path: "user.age".to_string(),
                value: json!(25),
            });

        let context = test_context();
        let result = evaluator
            .evaluate_rule(&rule, "ctx1", &context)
            .await
            .expect("should succeed");

        assert!(result.matches); // 30 > 25
    }

    #[tokio::test]
    async fn test_evaluate_less_than_condition() {
        let evaluator = RuleEvaluator::new();
        evaluator.initialize();

        let rule = Rule::new("test-lt")
            .with_name("Less Than Test")
            .with_condition(RuleCondition::LessThan {
                path: "metrics.cpu".to_string(),
                value: json!(50.0),
            });

        let context = test_context();
        let result = evaluator
            .evaluate_rule(&rule, "ctx1", &context)
            .await
            .expect("should succeed");

        assert!(result.matches); // 45.5 < 50.0
    }

    #[tokio::test]
    async fn test_evaluate_matches_condition() {
        let evaluator = RuleEvaluator::new();
        evaluator.initialize();

        let rule = Rule::new("test-matches")
            .with_name("Matches Test")
            .with_condition(RuleCondition::Matches {
                path: "user.name".to_string(),
                pattern: "Test*".to_string(),
            });

        let context = test_context();
        let result = evaluator
            .evaluate_rule(&rule, "ctx1", &context)
            .await
            .expect("should succeed");

        assert!(result.matches);
    }

    #[tokio::test]
    async fn test_evaluate_all_conditions_success() {
        let evaluator = RuleEvaluator::new();
        evaluator.initialize();

        let rule = Rule::new("test-all")
            .with_name("All Conditions Test")
            .with_condition(RuleCondition::All {
                conditions: vec![
                    RuleCondition::Equals {
                        path: "user.name".to_string(),
                        value: json!("Test User"),
                    },
                    RuleCondition::GreaterThan {
                        path: "user.age".to_string(),
                        value: json!(18),
                    },
                    RuleCondition::Exists {
                        path: "session.id".to_string(),
                    },
                ],
            });

        let context = test_context();
        let result = evaluator
            .evaluate_rule(&rule, "ctx1", &context)
            .await
            .expect("should succeed");

        assert!(result.matches);
    }

    #[tokio::test]
    async fn test_evaluate_all_conditions_failure() {
        let evaluator = RuleEvaluator::new();
        evaluator.initialize();

        let rule = Rule::new("test-all-fail")
            .with_name("All Conditions Fail")
            .with_condition(RuleCondition::All {
                conditions: vec![
                    RuleCondition::Equals {
                        path: "user.name".to_string(),
                        value: json!("Test User"),
                    },
                    RuleCondition::GreaterThan {
                        path: "user.age".to_string(),
                        value: json!(100), // This will fail
                    },
                ],
            });

        let context = test_context();
        let result = evaluator
            .evaluate_rule(&rule, "ctx1", &context)
            .await
            .expect("should succeed");

        assert!(!result.matches); // Should fail because age is not > 100
    }

    #[tokio::test]
    async fn test_evaluate_any_conditions_success() {
        let evaluator = RuleEvaluator::new();
        evaluator.initialize();

        let rule = Rule::new("test-any")
            .with_name("Any Conditions Test")
            .with_condition(RuleCondition::Any {
                conditions: vec![
                    RuleCondition::Equals {
                        path: "user.name".to_string(),
                        value: json!("Wrong Name"), // Fails
                    },
                    RuleCondition::GreaterThan {
                        path: "user.age".to_string(),
                        value: json!(18), // Succeeds
                    },
                ],
            });

        let context = test_context();
        let result = evaluator
            .evaluate_rule(&rule, "ctx1", &context)
            .await
            .expect("should succeed");

        assert!(result.matches); // Should succeed because at least one passes
    }

    #[tokio::test]
    async fn test_evaluate_any_conditions_all_fail() {
        let evaluator = RuleEvaluator::new();
        evaluator.initialize();

        let rule = Rule::new("test-any-fail")
            .with_name("Any Conditions All Fail")
            .with_condition(RuleCondition::Any {
                conditions: vec![
                    RuleCondition::Equals {
                        path: "user.name".to_string(),
                        value: json!("Wrong Name"),
                    },
                    RuleCondition::GreaterThan {
                        path: "user.age".to_string(),
                        value: json!(100),
                    },
                ],
            });

        let context = test_context();
        let result = evaluator
            .evaluate_rule(&rule, "ctx1", &context)
            .await
            .expect("should succeed");

        assert!(!result.matches);
    }

    #[tokio::test]
    async fn test_evaluate_not_condition() {
        let evaluator = RuleEvaluator::new();
        evaluator.initialize();

        let rule = Rule::new("test-not")
            .with_name("Not Condition Test")
            .with_condition(RuleCondition::Not {
                condition: Box::new(RuleCondition::Equals {
                    path: "user.active".to_string(),
                    value: json!(false), // active is true, so NOT false = true
                }),
            });

        let context = test_context();
        let result = evaluator
            .evaluate_rule(&rule, "ctx1", &context)
            .await
            .expect("should succeed");

        assert!(result.matches);
    }

    #[tokio::test]
    async fn test_evaluate_empty_conditions() {
        let evaluator = RuleEvaluator::new();
        evaluator.initialize();

        let rule = Rule::new("test-empty");

        let context = test_context();
        let result = evaluator
            .evaluate_rule(&rule, "ctx1", &context)
            .await
            .expect("should succeed");

        assert!(result.matches); // Empty conditions should match
    }

    #[tokio::test]
    async fn test_evaluation_caching() {
        let evaluator = RuleEvaluator::new();
        evaluator.initialize();

        let rule = Rule::new("test-cache")
            .with_name("Cache Test")
            .with_condition(RuleCondition::Equals {
                path: "user.name".to_string(),
                value: json!("Test User"),
            });

        let context = test_context();

        // First evaluation (not cached)
        let result1 = evaluator
            .evaluate_rule(&rule, "ctx1", &context)
            .await
            .expect("should succeed");
        assert!(result1.matches);

        // Second evaluation (should be cached)
        let result2 = evaluator
            .evaluate_rule(&rule, "ctx1", &context)
            .await
            .expect("should succeed");
        assert!(result2.matches);
        assert_eq!(result1.rule_id, result2.rule_id);
    }

    #[tokio::test]
    async fn test_nested_conditions() {
        let evaluator = RuleEvaluator::new();
        evaluator.initialize();

        let rule = Rule::new("test-nested")
            .with_name("Nested Conditions")
            .with_condition(RuleCondition::All {
                conditions: vec![
                    RuleCondition::Exists {
                        path: "user".to_string(),
                    },
                    RuleCondition::Any {
                        conditions: vec![
                            RuleCondition::Equals {
                                path: "user.name".to_string(),
                                value: json!("Test User"),
                            },
                            RuleCondition::Equals {
                                path: "user.name".to_string(),
                                value: json!("Admin User"),
                            },
                        ],
                    },
                ],
            });

        let context = test_context();
        let result = evaluator
            .evaluate_rule(&rule, "ctx1", &context)
            .await
            .expect("should succeed");

        assert!(result.matches);
    }

    #[tokio::test]
    async fn test_numeric_comparison_edge_cases() {
        let evaluator = RuleEvaluator::new();
        evaluator.initialize();

        // Test with exact equality edge case for greater than
        let rule = Rule::new("test-gt-edge")
            .with_name("GT Edge Case")
            .with_condition(RuleCondition::GreaterThan {
                path: "user.age".to_string(),
                value: json!(30), // Exactly equal, not greater
            });

        let context = test_context();
        let result = evaluator
            .evaluate_rule(&rule, "ctx1", &context)
            .await
            .expect("should succeed");

        assert!(!result.matches); // 30 is NOT > 30
    }

    #[tokio::test]
    async fn test_missing_path_conditions() {
        let evaluator = RuleEvaluator::new();
        evaluator.initialize();

        let rule = Rule::new("test-missing")
            .with_name("Missing Path")
            .with_condition(RuleCondition::Equals {
                path: "nonexistent.deeply.nested.path".to_string(),
                value: json!("value"),
            });

        let context = test_context();
        let result = evaluator
            .evaluate_rule(&rule, "ctx1", &context)
            .await
            .expect("should succeed");

        assert!(!result.matches); // Missing path should not match
    }

    #[tokio::test]
    async fn test_statistics_collection() {
        let evaluator = RuleEvaluator::new();
        evaluator.initialize();

        let rule = Rule::new("test-stats").with_condition(RuleCondition::Exists {
            path: "user".to_string(),
        });

        let context = test_context();

        // Run multiple evaluations
        for i in 0..5 {
            let _ = evaluator
                .evaluate_rule(&rule, &format!("ctx{i}"), &context)
                .await;
        }

        // Get statistics (just verify it doesn't error)
        let _stats = evaluator.get_statistics().await;
        // Note: fields are private, so we just verify the method works
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let evaluator = RuleEvaluator::new();
        evaluator.initialize();

        let rule = Rule::new("test-cache-exp").with_condition(RuleCondition::Exists {
            path: "user".to_string(),
        });

        let context = test_context();

        // First evaluation
        let _ = evaluator.evaluate_rule(&rule, "ctx1", &context).await;

        // Cache is valid for 5 minutes (300 seconds)
        // In a real test, we'd mock time or wait
        // For now, just verify it doesn't error

        // Second evaluation (within cache time)
        let result = evaluator.evaluate_rule(&rule, "ctx1", &context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_clear_cache() {
        let evaluator = RuleEvaluator::new();
        evaluator.initialize();

        let rule = Rule::new("test-clear").with_condition(RuleCondition::Exists {
            path: "user".to_string(),
        });

        let context = test_context();

        // Evaluate to populate cache
        let _ = evaluator.evaluate_rule(&rule, "ctx1", &context).await;

        // Clear cache
        let _ = evaluator.clear_cache().await;

        // Evaluate again (should not use cache)
        let result = evaluator.evaluate_rule(&rule, "ctx1", &context).await;
        assert!(result.is_ok());
    }
}
