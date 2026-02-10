// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Tests for adaptive rule system

use super::adaptive::*;
use super::test_helpers;
use super::*;

#[tokio::test]
async fn test_adaptive_rule_system_new() {
    let config = test_helpers::create_test_learning_config();
    let system = AdaptiveRuleSystem::new(Arc::new(config))
        .await
        .expect("Should create adaptive rule system");

    let stats = system.get_stats().await;
    assert_eq!(stats.total_adaptations, 0);
    assert_eq!(stats.successful_adaptations, 0);
}

#[tokio::test]
async fn test_adaptive_rule_system_initialize() {
    let config = test_helpers::create_test_learning_config();
    let system = AdaptiveRuleSystem::new(Arc::new(config))
        .await
        .expect("Should create system");

    system.initialize().await.expect("Should initialize");

    let rules = system.get_adaptive_rules().await;
    assert_eq!(rules.len(), 0);
}

#[tokio::test]
async fn test_add_rule() {
    let config = test_helpers::create_test_learning_config();
    let system = AdaptiveRuleSystem::new(Arc::new(config))
        .await
        .expect("Should create system");

    let rule = test_helpers::create_test_rule();
    system
        .add_rule(rule.clone())
        .await
        .expect("Should add rule");

    let rules = system.get_adaptive_rules().await;
    assert_eq!(rules.len(), 1);
    assert!(rules.contains_key(rule.id()));
}

#[tokio::test]
async fn test_update_rule_performance() {
    let config = test_helpers::create_test_learning_config();
    let system = AdaptiveRuleSystem::new(Arc::new(config))
        .await
        .expect("Should create system");

    let rule = test_helpers::create_test_rule();
    let rule_id = rule.id().to_string();
    system.add_rule(rule).await.expect("Should add rule");

    // Update performance with successful execution
    system
        .update_rule_performance(&rule_id, true, 0.5)
        .await
        .expect("Should update performance");

    let performance = system
        .get_rule_performance(&rule_id)
        .await
        .expect("Should get performance");

    assert_eq!(performance.application_count, 1);
    assert_eq!(performance.success_count, 1);
    assert_eq!(performance.success_rate, 1.0);
}

#[tokio::test]
async fn test_rule_performance_multiple_updates() {
    let config = test_helpers::create_test_learning_config();
    let system = AdaptiveRuleSystem::new(Arc::new(config))
        .await
        .expect("Should create system");

    let rule = test_helpers::create_test_rule();
    let rule_id = rule.id().to_string();
    system.add_rule(rule).await.expect("Should add rule");

    // Update performance multiple times
    for i in 0..10 {
        let success = i % 2 == 0; // Alternate success/failure
        system
            .update_rule_performance(&rule_id, success, 0.5)
            .await
            .expect("Should update performance");
    }

    let performance = system
        .get_rule_performance(&rule_id)
        .await
        .expect("Should get performance");

    assert_eq!(performance.application_count, 10);
    assert_eq!(performance.success_count, 5); // 50% success rate
    assert_eq!(performance.success_rate, 0.5);
}

#[tokio::test]
async fn test_adapt_rules() {
    let config = test_helpers::create_test_learning_config();
    let system = AdaptiveRuleSystem::new(Arc::new(config))
        .await
        .expect("Should create system");

    let rule = test_helpers::create_test_rule();
    let rule_id = rule.id().to_string();
    system.add_rule(rule).await.expect("Should add rule");

    // Update performance to trigger adaptation (low effectiveness)
    for _ in 0..20 {
        system
            .update_rule_performance(&rule_id, false, 2.0) // High time, low success
            .await
            .expect("Should update performance");
    }

    let adaptations = system.adapt_rules().await.expect("Should adapt rules");

    // Should have at least one adaptation
    assert!(!adaptations.is_empty());
}

#[tokio::test]
async fn test_adaptation_strategy_gradual() {
    let meta = AdaptationMetadata {
        rule_id: "test_rule".to_string(),
        adaptation_level: 0.0,
        learning_rate: 0.1,
        strategy: AdaptationStrategy::Gradual,
        last_adaptation: Utc::now(),
        adaptation_count: 0,
    };

    let serialized = serde_json::to_string(&meta).expect("Should serialize");
    assert!(serialized.contains("Gradual"));
}

#[tokio::test]
async fn test_adaptation_strategy_threshold() {
    let strategy = AdaptationStrategy::Threshold(0.75);
    let serialized = serde_json::to_string(&strategy).expect("Should serialize");
    assert!(serialized.contains("Threshold"));
    assert!(serialized.contains("0.75"));
}

#[tokio::test]
async fn test_rule_performance_default() {
    let performance = RulePerformance::default();
    assert_eq!(performance.success_rate, 0.0);
    assert_eq!(performance.avg_execution_time, 0.0);
    assert_eq!(performance.application_count, 0);
    assert_eq!(performance.success_count, 0);
}

#[tokio::test]
async fn test_adaptation_stats_default() {
    let stats = AdaptationStats::default();
    assert_eq!(stats.total_adaptations, 0);
    assert_eq!(stats.successful_adaptations, 0);
    assert_eq!(stats.failed_adaptations, 0);
    assert_eq!(stats.average_improvement, 0.0);
}

#[tokio::test]
async fn test_rule_change_serialization() {
    let change = RuleChange {
        change_type: "condition_threshold".to_string(),
        target: "condition_0".to_string(),
        previous_value: serde_json::json!(0.5),
        new_value: serde_json::json!(0.6),
        confidence: 0.8,
    };

    let serialized = serde_json::to_string(&change).expect("Should serialize");
    let deserialized: RuleChange = serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(deserialized.change_type, "condition_threshold");
    assert_eq!(deserialized.confidence, 0.8);
}

#[tokio::test]
async fn test_adaptation_type_variants() {
    let types = vec![
        AdaptationType::ConditionModification,
        AdaptationType::ActionModification,
        AdaptationType::PriorityAdjustment,
        AdaptationType::ParameterAdjustment,
        AdaptationType::EnablementChange,
    ];

    for adaptation_type in types {
        let serialized = serde_json::to_string(&adaptation_type).expect("Should serialize");
        assert!(!serialized.is_empty());
    }
}

#[tokio::test]
async fn test_remove_rule() {
    let config = test_helpers::create_test_learning_config();
    let system = AdaptiveRuleSystem::new(Arc::new(config))
        .await
        .expect("Should create system");

    let rule = test_helpers::create_test_rule();
    let rule_id = rule.id().to_string();
    system.add_rule(rule).await.expect("Should add rule");

    let rules_before = system.get_adaptive_rules().await;
    assert_eq!(rules_before.len(), 1);

    system
        .remove_rule(&rule_id)
        .await
        .expect("Should remove rule");

    let rules_after = system.get_adaptive_rules().await;
    assert_eq!(rules_after.len(), 0);
}

#[tokio::test]
async fn test_clear_history() {
    let config = test_helpers::create_test_learning_config();
    let system = AdaptiveRuleSystem::new(Arc::new(config))
        .await
        .expect("Should create system");

    let rule = test_helpers::create_test_rule();
    let rule_id = rule.id().to_string();
    system.add_rule(rule).await.expect("Should add rule");

    // Trigger some adaptations
    for _ in 0..20 {
        system
            .update_rule_performance(&rule_id, false, 2.0)
            .await
            .expect("Should update performance");
    }
    let _ = system.adapt_rules().await;

    system.clear_history().await.expect("Should clear history");

    let adaptations = system.get_adaptations().await;
    assert_eq!(adaptations.len(), 0);

    let stats = system.get_stats().await;
    assert_eq!(stats.total_adaptations, 0);
}

#[tokio::test]
async fn test_export_rules() {
    let config = test_helpers::create_test_learning_config();
    let system = AdaptiveRuleSystem::new(Arc::new(config))
        .await
        .expect("Should create system");

    let rule = test_helpers::create_test_rule();
    system.add_rule(rule).await.expect("Should add rule");

    let export = system.export_rules().await.expect("Should export rules");

    assert!(export.get("adaptive_rules").is_some());
    assert!(export.get("adaptations").is_some());
    assert!(export.get("statistics").is_some());
    assert!(export.get("export_timestamp").is_some());
}

#[tokio::test]
async fn test_adaptive_rule_serialization() {
    let rule = test_helpers::create_test_rule();
    let adaptive_rule = AdaptiveRule {
        base_rule: rule.clone(),
        adaptation_meta: AdaptationMetadata {
            rule_id: rule.id().to_string(),
            adaptation_level: 0.5,
            learning_rate: 0.1,
            strategy: AdaptationStrategy::Gradual,
            last_adaptation: Utc::now(),
            adaptation_count: 5,
        },
        performance: RulePerformance::default(),
        adaptation_history: vec!["adaptation_1".to_string(), "adaptation_2".to_string()],
    };

    let serialized = serde_json::to_string(&adaptive_rule).expect("Should serialize");
    let deserialized: AdaptiveRule = serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(deserialized.adaptation_meta.rule_id, rule.id());
    assert_eq!(deserialized.adaptation_meta.adaptation_level, 0.5);
}

#[tokio::test]
async fn test_rule_effectiveness_calculation() {
    let config = test_helpers::create_test_learning_config();
    let system = AdaptiveRuleSystem::new(Arc::new(config))
        .await
        .expect("Should create system");

    let rule = test_helpers::create_test_rule();
    let rule_id = rule.id().to_string();
    system.add_rule(rule).await.expect("Should add rule");

    // Update with known values to test effectiveness calculation
    system
        .update_rule_performance(&rule_id, true, 0.5)
        .await
        .expect("Should update performance");

    let performance = system
        .get_rule_performance(&rule_id)
        .await
        .expect("Should get performance");

    // Effectiveness should be success_rate * (1 / (1 + avg_execution_time))
    let expected_effectiveness = 1.0 * (1.0 / (1.0 + 0.5));
    assert!((performance.effectiveness - expected_effectiveness).abs() < 0.001);
}

#[tokio::test]
async fn test_adaptation_with_high_performance() {
    let config = test_helpers::create_test_learning_config();
    let system = AdaptiveRuleSystem::new(Arc::new(config))
        .await
        .expect("Should create system");

    let rule = test_helpers::create_test_rule();
    let rule_id = rule.id().to_string();
    system.add_rule(rule).await.expect("Should add rule");

    // Update with high performance
    for _ in 0..20 {
        system
            .update_rule_performance(&rule_id, true, 0.1) // Fast, successful
            .await
            .expect("Should update performance");
    }

    let adaptations = system.adapt_rules().await.expect("Should adapt rules");

    // High-performing rules should not trigger adaptation
    assert_eq!(adaptations.len(), 0);
}

#[tokio::test]
async fn test_get_adaptations_empty() {
    let config = test_helpers::create_test_learning_config();
    let system = AdaptiveRuleSystem::new(Arc::new(config))
        .await
        .expect("Should create system");

    let adaptations = system.get_adaptations().await;
    assert_eq!(adaptations.len(), 0);
}

#[tokio::test]
async fn test_rule_performance_impact() {
    let performance = RulePerformance {
        success_rate: 0.9,
        avg_execution_time: 0.2,
        impact: 0.85,
        ..Default::default()
    };

    let serialized = serde_json::to_string(&performance).expect("Should serialize");
    assert!(serialized.contains("0.9"));
    assert!(serialized.contains("0.85"));
}
