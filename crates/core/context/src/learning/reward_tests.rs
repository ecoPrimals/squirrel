// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Tests for reward system

use super::engine::{RLAction, RLState};
use super::reward::*;
use super::test_helpers;
use super::*;
use chrono::Utc;

#[tokio::test]
async fn test_reward_system_new() {
    let config = test_helpers::create_test_learning_config();
    let system = RewardSystem::new(Arc::new(config))
        .await
        .expect("Should create reward system");

    let metrics = system.get_metrics().await;
    assert_eq!(metrics.total_rewards, 0);
}

#[tokio::test]
async fn test_reward_system_initialize() {
    let config = test_helpers::create_test_learning_config();
    let system = RewardSystem::new(Arc::new(config))
        .await
        .expect("Should create system");

    system.initialize().await.expect("Should initialize");

    let baselines = system.get_baselines().await;
    assert!(baselines.contains_key("success"));
}

#[tokio::test]
async fn test_calculate_reward_success() {
    let config = test_helpers::create_test_learning_config();
    let system = RewardSystem::new(Arc::new(config))
        .await
        .expect("Should create system");

    let context = create_success_reward_context();
    let reward = system
        .calculate_reward(context)
        .await
        .expect("Should calculate reward");

    // Successful context should have positive reward
    assert!(reward > 0.0);
}

#[tokio::test]
async fn test_calculate_reward_failure() {
    let config = test_helpers::create_test_learning_config();
    let system = RewardSystem::new(Arc::new(config))
        .await
        .expect("Should create system");

    let context = create_failure_reward_context();
    let reward = system
        .calculate_reward(context)
        .await
        .expect("Should calculate reward");

    // Failed context should have negative reward
    assert!(reward < 0.0);
}

#[tokio::test]
async fn test_reward_metrics_update() {
    let config = test_helpers::create_test_learning_config();
    let system = RewardSystem::new(Arc::new(config))
        .await
        .expect("Should create system");

    let context = create_success_reward_context();
    let _ = system.calculate_reward(context).await;

    let metrics = system.get_metrics().await;
    assert_eq!(metrics.total_rewards, 1);
    assert!(metrics.average_reward != 0.0);
}

#[tokio::test]
async fn test_reward_history() {
    let config = test_helpers::create_test_learning_config();
    let system = RewardSystem::new(Arc::new(config))
        .await
        .expect("Should create system");

    let context = create_success_reward_context();
    let _ = system.calculate_reward(context).await;

    let history = system.get_reward_history().await;
    assert_eq!(history.len(), 1);
}

#[tokio::test]
async fn test_get_context_rewards() {
    let config = test_helpers::create_test_learning_config();
    let system = RewardSystem::new(Arc::new(config))
        .await
        .expect("Should create system");

    let context = create_success_reward_context();
    let context_id = context.current_state.context_id.clone();
    let _ = system.calculate_reward(context).await;

    let context_rewards = system.get_context_rewards(&context_id).await;
    assert_eq!(context_rewards.len(), 1);
}

#[tokio::test]
async fn test_clear_history() {
    let config = test_helpers::create_test_learning_config();
    let system = RewardSystem::new(Arc::new(config))
        .await
        .expect("Should create system");

    let context = create_success_reward_context();
    let _ = system.calculate_reward(context).await;

    let history_before = system.get_reward_history().await;
    assert_eq!(history_before.len(), 1);

    system.clear_history().await.expect("Should clear history");

    let history_after = system.get_reward_history().await;
    assert_eq!(history_after.len(), 0);

    let metrics = system.get_metrics().await;
    assert_eq!(metrics.total_rewards, 0);
}

#[tokio::test]
async fn test_update_baselines() {
    let config = test_helpers::create_test_learning_config();
    let system = RewardSystem::new(Arc::new(config))
        .await
        .expect("Should create system");

    // Add 100+ rewards to enable baseline calculation
    for _ in 0..105 {
        let context = create_success_reward_context();
        let _ = system.calculate_reward(context).await;
    }

    system
        .update_baselines()
        .await
        .expect("Should update baselines");

    let baselines = system.get_baselines().await;
    assert!(baselines.contains_key("overall"));
}

#[tokio::test]
async fn test_add_custom_calculator() {
    let config = test_helpers::create_test_learning_config();
    let system = RewardSystem::new(Arc::new(config))
        .await
        .expect("Should create system");

    let calculator = Box::new(SuccessRewardCalculator::new(5.0, -2.0));
    system
        .add_calculator("custom".to_string(), calculator)
        .await
        .expect("Should add calculator");
}

#[tokio::test]
async fn test_remove_calculator() {
    let config = test_helpers::create_test_learning_config();
    let system = RewardSystem::new(Arc::new(config))
        .await
        .expect("Should create system");

    let calculator = Box::new(SuccessRewardCalculator::new(5.0, -2.0));
    system
        .add_calculator("custom".to_string(), calculator)
        .await
        .expect("Should add calculator");

    system
        .remove_calculator("custom")
        .await
        .expect("Should remove calculator");
}

#[test]
fn test_success_reward_calculator() {
    let calculator = SuccessRewardCalculator::new(10.0, -5.0);

    let success_context = create_success_reward_context();
    let success_reward = calculator
        .calculate_reward(&success_context)
        .expect("Should calculate");
    assert_eq!(success_reward, 10.0);

    let failure_context = create_failure_reward_context();
    let failure_reward = calculator
        .calculate_reward(&failure_context)
        .expect("Should calculate");
    assert_eq!(failure_reward, -5.0);
}

#[test]
fn test_performance_reward_calculator() {
    let thresholds = PerformanceThresholds::default();
    let calculator = PerformanceRewardCalculator::new(thresholds);

    let context = create_success_reward_context();
    let reward = calculator
        .calculate_reward(&context)
        .expect("Should calculate");

    // Should be based on performance metrics
    assert!(reward.is_finite());
}

#[test]
fn test_rule_efficiency_reward_calculator() {
    let calculator = RuleEfficiencyRewardCalculator::new(2.0);

    let context = create_success_reward_context();
    let reward = calculator
        .calculate_reward(&context)
        .expect("Should calculate");

    // Should be based on rule efficiency
    assert!(reward >= 0.0);
}

#[test]
fn test_synchronization_reward_calculator() {
    let calculator = SynchronizationRewardCalculator::new(1.0);

    let context = create_success_reward_context();
    let reward = calculator
        .calculate_reward(&context)
        .expect("Should calculate");

    // Synced context should get bonus
    assert_eq!(reward, 1.0);
}

#[test]
fn test_error_severity_variants() {
    let severities = vec![
        ErrorSeverity::Low,
        ErrorSeverity::Medium,
        ErrorSeverity::High,
        ErrorSeverity::Critical,
    ];

    for severity in severities {
        let serialized = serde_json::to_string(&severity).expect("Should serialize");
        assert!(!serialized.is_empty());
    }
}

#[test]
fn test_reward_breakdown_serialization() {
    let breakdown = RewardBreakdown {
        base_reward: 10.0,
        performance_bonus: 2.0,
        efficiency_bonus: 1.0,
        sync_bonus: 1.0,
        error_penalty: 0.0,
        time_penalty: 0.5,
        final_reward: 13.5,
    };

    let serialized = serde_json::to_string(&breakdown).expect("Should serialize");
    let deserialized: RewardBreakdown =
        serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(deserialized.base_reward, 10.0);
    assert_eq!(deserialized.final_reward, 13.5);
}

#[test]
fn test_reward_metrics_default() {
    let metrics = RewardMetrics::default();
    assert_eq!(metrics.total_rewards, 0);
    assert_eq!(metrics.average_reward, 0.0);
    assert_eq!(metrics.max_reward, f64::NEG_INFINITY);
    assert_eq!(metrics.min_reward, f64::INFINITY);
}

#[test]
fn test_performance_thresholds_default() {
    let thresholds = PerformanceThresholds::default();
    assert_eq!(thresholds.min_success_rate, 0.8);
    assert_eq!(thresholds.max_processing_time, 1.0);
    assert_eq!(thresholds.min_throughput, 10.0);
    assert_eq!(thresholds.max_error_rate, 0.1);
}

#[tokio::test]
async fn test_reward_metrics_min_max() {
    let config = test_helpers::create_test_learning_config();
    let system = RewardSystem::new(Arc::new(config))
        .await
        .expect("Should create system");

    // Add rewards with different values
    for i in 0..5 {
        let mut context = create_success_reward_context();
        context.performance_metrics.success_rate = 0.5 + (i as f64) * 0.1;
        let _ = system.calculate_reward(context).await;
    }

    let metrics = system.get_metrics().await;
    assert_ne!(metrics.max_reward, f64::NEG_INFINITY);
    assert_ne!(metrics.min_reward, f64::INFINITY);
    assert!(metrics.max_reward >= metrics.min_reward);
}

#[tokio::test]
async fn test_reward_positive_negative_counts() {
    let config = test_helpers::create_test_learning_config();
    let system = RewardSystem::new(Arc::new(config))
        .await
        .expect("Should create system");

    // Add some positive rewards
    for _ in 0..3 {
        let context = create_success_reward_context();
        let _ = system.calculate_reward(context).await;
    }

    // Add some negative rewards
    for _ in 0..2 {
        let context = create_failure_reward_context();
        let _ = system.calculate_reward(context).await;
    }

    let metrics = system.get_metrics().await;
    assert!(metrics.positive_rewards > 0);
    assert!(metrics.negative_rewards > 0);
}

// Helper functions
fn create_success_reward_context() -> RewardContext {
    RewardContext {
        action: RLAction {
            id: "action_1".to_string(),
            action_type: "test_action".to_string(),
            parameters: serde_json::Value::Null,
            confidence: 0.8,
            expected_reward: 1.0,
        },
        previous_state: RLState {
            id: "state_1".to_string(),
            context_id: "context_1".to_string(),
            features: vec![1.0, 2.0, 3.0],
            metadata: None,
            timestamp: Utc::now(),
        },
        current_state: RLState {
            id: "state_2".to_string(),
            context_id: "context_1".to_string(),
            features: vec![1.1, 2.1, 3.1],
            metadata: None,
            timestamp: Utc::now(),
        },
        performance_metrics: PerformanceMetrics {
            sync_status: true,
            version: 1,
            active_contexts: 1,
            memory_usage: 100.0,
            processing_time: 0.5,
            success_rate: 0.9,
            error_rate: 0.05,
            throughput: 15.0,
        },
        rule_results: Some(RuleResults {
            rules_applied: 5,
            successful_applications: 5,
            efficiency_score: 0.95,
            failed_rules: vec![],
            execution_time: 0.3,
        }),
        error_info: None,
        timestamp: Utc::now(),
    }
}

fn create_failure_reward_context() -> RewardContext {
    let mut context = create_success_reward_context();
    context.error_info = Some(ErrorInfo {
        error_type: "TestError".to_string(),
        severity: ErrorSeverity::Medium,
        message: "Test error message".to_string(),
        recoverable: true,
        impact: 0.5,
    });
    context.performance_metrics.success_rate = 0.3;
    context.performance_metrics.error_rate = 0.4;
    context
}
