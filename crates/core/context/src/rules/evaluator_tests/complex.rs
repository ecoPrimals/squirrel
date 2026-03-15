// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Complex rule tests - find_matching_rules, multiple conditions, regex caching

use super::create_test_rule;
use crate::rules::Rule;
use crate::rules::evaluator::RuleEvaluator;
use crate::rules::models::RuleCondition;
use serde_json::json;
use std::sync::Arc;

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
