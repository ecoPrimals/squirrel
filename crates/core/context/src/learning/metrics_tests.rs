// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Tests for learning metrics system

use super::metrics::*;
use super::test_helpers;
use super::*;
use std::collections::HashMap;
use std::f64::consts::PI;

#[tokio::test]
async fn test_learning_metrics_new() {
    let config = test_helpers::create_test_learning_config();
    let metrics = LearningMetrics::new(Arc::new(config))
        .await
        .expect("Should create learning metrics");

    let stats = metrics.get_stats().await;
    assert_eq!(stats.total_episodes, 0);
}

#[tokio::test]
async fn test_learning_metrics_initialize() {
    let config = test_helpers::create_test_learning_config();
    let metrics = LearningMetrics::new(Arc::new(config.clone()))
        .await
        .expect("Should create metrics");

    metrics.initialize().await.expect("Should initialize");

    let performance = metrics.get_performance().await;
    assert_eq!(performance.learning_rate, config.learning_rate);
    assert_eq!(performance.exploration_rate, config.exploration_rate);
}

#[tokio::test]
async fn test_learning_metrics_start() {
    let config = test_helpers::create_test_learning_config();
    let metrics = LearningMetrics::new(Arc::new(config))
        .await
        .expect("Should create metrics");

    metrics.initialize().await.expect("Should initialize");
    metrics.start().await.expect("Should start");

    let history = metrics.get_history().await;
    assert_eq!(history.len(), 1); // Initial snapshot
}

#[tokio::test]
async fn test_update_performance() {
    let config = test_helpers::create_test_learning_config();
    let metrics = LearningMetrics::new(Arc::new(config))
        .await
        .expect("Should create metrics");

    let mut updates = HashMap::new();
    updates.insert("success_rate".to_string(), 0.85);
    updates.insert("average_reward".to_string(), 10.5);

    metrics
        .update_performance(updates)
        .await
        .expect("Should update performance");

    let performance = metrics.get_performance().await;
    assert_eq!(performance.success_rate, 0.85);
    assert_eq!(performance.average_reward, 10.5);
}

#[tokio::test]
async fn test_update_stats() {
    let config = test_helpers::create_test_learning_config();
    let metrics = LearningMetrics::new(Arc::new(config))
        .await
        .expect("Should create metrics");

    let mut updates = HashMap::new();
    updates.insert("total_episodes".to_string(), 100.0);
    updates.insert("successful_episodes".to_string(), 85.0);

    metrics
        .update_stats(updates)
        .await
        .expect("Should update stats");

    let stats = metrics.get_stats().await;
    assert_eq!(stats.total_episodes, 100);
    assert_eq!(stats.successful_episodes, 85);
}

#[tokio::test]
async fn test_record_episode() {
    let config = test_helpers::create_test_learning_config();
    let metrics = LearningMetrics::new(Arc::new(config))
        .await
        .expect("Should create metrics");

    metrics
        .record_episode(true, 15.5, 10, 1.0)
        .await
        .expect("Should record episode");

    let stats = metrics.get_stats().await;
    assert_eq!(stats.total_episodes, 1);
    assert_eq!(stats.successful_episodes, 1);
    assert_eq!(stats.total_actions, 10);
}

#[tokio::test]
async fn test_record_episode_failure() {
    let config = test_helpers::create_test_learning_config();
    let metrics = LearningMetrics::new(Arc::new(config))
        .await
        .expect("Should create metrics");

    metrics
        .record_episode(false, -2.0, 5, 1.0)
        .await
        .expect("Should record episode");

    let stats = metrics.get_stats().await;
    assert_eq!(stats.total_episodes, 1);
    assert_eq!(stats.successful_episodes, 0);
}

#[tokio::test]
async fn test_record_policy_update() {
    let config = test_helpers::create_test_learning_config();
    let metrics = LearningMetrics::new(Arc::new(config))
        .await
        .expect("Should create metrics");

    metrics
        .record_policy_update(0.85, 0.15)
        .await
        .expect("Should record policy update");

    let stats = metrics.get_stats().await;
    assert_eq!(stats.policy_updates, 1);
}

#[tokio::test]
async fn test_record_rule_adaptation() {
    let config = test_helpers::create_test_learning_config();
    let metrics = LearningMetrics::new(Arc::new(config))
        .await
        .expect("Should create metrics");

    metrics
        .record_rule_adaptation("test_rule", 0.1)
        .await
        .expect("Should record rule adaptation");

    let stats = metrics.get_stats().await;
    assert_eq!(stats.rule_adaptations, 1);
}

#[tokio::test]
async fn test_take_snapshot() {
    let config = test_helpers::create_test_learning_config();
    let metrics = LearningMetrics::new(Arc::new(config))
        .await
        .expect("Should create metrics");

    metrics.take_snapshot().await.expect("Should take snapshot");

    let history = metrics.get_history().await;
    assert_eq!(history.len(), 1);
}

#[tokio::test]
async fn test_add_custom_metric() {
    let config = test_helpers::create_test_learning_config();
    let metrics = LearningMetrics::new(Arc::new(config))
        .await
        .expect("Should create metrics");

    metrics
        .set_custom_metric("custom_metric".to_string(), 42.0)
        .await
        .expect("Should set custom metric");

    let custom_metrics = metrics.get_custom_metrics().await;
    assert_eq!(custom_metrics.get("custom_metric"), Some(&42.0));
}

#[tokio::test]
async fn test_get_custom_metric() {
    let config = test_helpers::create_test_learning_config();
    let metrics = LearningMetrics::new(Arc::new(config))
        .await
        .expect("Should create metrics");

    metrics
        .set_custom_metric("test_metric".to_string(), PI)
        .await
        .expect("Should set metric");

    let value = metrics.get_custom_metric("test_metric").await;
    assert_eq!(value, Some(PI));
}

#[tokio::test]
async fn test_clear_history() {
    let config = test_helpers::create_test_learning_config();
    let metrics = LearningMetrics::new(Arc::new(config))
        .await
        .expect("Should create metrics");

    metrics.take_snapshot().await.expect("Should take snapshot");
    metrics.take_snapshot().await.expect("Should take snapshot");

    let history_before = metrics.get_history().await;
    assert_eq!(history_before.len(), 2);

    metrics.clear_history().await.expect("Should clear history");

    let history_after = metrics.get_history().await;
    assert_eq!(history_after.len(), 0);
}

#[tokio::test]
async fn test_export_metrics() {
    let config = test_helpers::create_test_learning_config();
    let metrics = LearningMetrics::new(Arc::new(config))
        .await
        .expect("Should create metrics");

    let export = metrics
        .export_metrics()
        .await
        .expect("Should export metrics");

    // Check for the actual keys that export_metrics() returns
    assert!(export.get("current_performance").is_some());
    assert!(export.get("current_stats").is_some());
    assert!(export.get("custom_metrics").is_some());
    assert!(export.get("export_timestamp").is_some());
}

#[test]
fn test_learning_performance_default() {
    let performance = LearningPerformance::default();
    assert_eq!(performance.learning_rate, 0.0);
    assert_eq!(performance.success_rate, 0.0);
    assert_eq!(performance.exploration_rate, 1.0);
}

#[test]
fn test_learning_stats_default() {
    let stats = LearningStats::default();
    assert_eq!(stats.total_episodes, 0);
    assert_eq!(stats.successful_episodes, 0);
    assert_eq!(stats.total_actions, 0);
    assert_eq!(stats.total_rewards, 0.0);
}

#[test]
fn test_learning_performance_serialization() {
    let performance = LearningPerformance {
        learning_rate: 0.01,
        success_rate: 0.85,
        average_reward: 10.5,
        episode_completion_rate: 0.9,
        training_accuracy: 0.88,
        convergence_rate: 0.75,
        exploration_rate: 0.1,
        policy_stability: 0.95,
        last_update: Utc::now(),
    };

    let serialized = serde_json::to_string(&performance).expect("Should serialize");
    let deserialized: LearningPerformance =
        serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(deserialized.learning_rate, 0.01);
    assert_eq!(deserialized.success_rate, 0.85);
    assert_eq!(deserialized.average_reward, 10.5);
}

#[test]
fn test_learning_stats_serialization() {
    let stats = LearningStats {
        total_episodes: 1000,
        successful_episodes: 850,
        total_actions: 50000,
        total_rewards: 12500.0,
        total_learning_time: 3600.0,
        average_episode_length: 50.0,
        average_reward_per_episode: 12.5,
        policy_updates: 100,
        rule_adaptations: 25,
        contexts_learned: 200,
        last_update: Utc::now(),
    };

    let serialized = serde_json::to_string(&stats).expect("Should serialize");
    let deserialized: LearningStats =
        serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(deserialized.total_episodes, 1000);
    assert_eq!(deserialized.successful_episodes, 850);
    assert_eq!(deserialized.total_actions, 50000);
}

#[test]
fn test_metrics_snapshot_serialization() {
    let snapshot = MetricsSnapshot {
        id: "snapshot_1".to_string(),
        timestamp: Utc::now(),
        performance: LearningPerformance::default(),
        stats: LearningStats::default(),
        custom_metrics: HashMap::new(),
    };

    let serialized = serde_json::to_string(&snapshot).expect("Should serialize");
    let deserialized: MetricsSnapshot =
        serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(deserialized.id, "snapshot_1");
}

#[tokio::test]
async fn test_multiple_episode_records() {
    let config = test_helpers::create_test_learning_config();
    let metrics = LearningMetrics::new(Arc::new(config))
        .await
        .expect("Should create metrics");

    // Record multiple episodes
    for i in 0..10 {
        let success = i % 2 == 0;
        let reward = if success { 10.0 } else { -5.0 };
        metrics
            .record_episode(success, reward, 5, 1.0)
            .await
            .expect("Should record episode");
    }

    let stats = metrics.get_stats().await;
    assert_eq!(stats.total_episodes, 10);
    assert_eq!(stats.successful_episodes, 5);
    assert_eq!(stats.total_actions, 50); // 10 episodes * 5 actions
}

#[tokio::test]
async fn test_average_reward_calculation() {
    let config = test_helpers::create_test_learning_config();
    let metrics = LearningMetrics::new(Arc::new(config))
        .await
        .expect("Should create metrics");

    // Record episodes with known rewards
    metrics
        .record_episode(true, 10.0, 5, 1.0)
        .await
        .expect("Should record");
    metrics
        .record_episode(true, 20.0, 5, 1.0)
        .await
        .expect("Should record");
    metrics
        .record_episode(true, 30.0, 5, 1.0)
        .await
        .expect("Should record");

    let stats = metrics.get_stats().await;
    assert_eq!(stats.average_reward_per_episode, 20.0); // (10 + 20 + 30) / 3
}

#[tokio::test]
async fn test_history_ordering() {
    let config = test_helpers::create_test_learning_config();
    let metrics = LearningMetrics::new(Arc::new(config))
        .await
        .expect("Should create metrics");

    // Take multiple snapshots
    for _ in 0..5 {
        metrics.take_snapshot().await.expect("Should take snapshot");
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    let history = metrics.get_history().await;
    assert_eq!(history.len(), 5);

    // Verify chronological ordering
    for i in 1..history.len() {
        assert!(history[i].timestamp >= history[i - 1].timestamp);
    }
}

#[tokio::test]
async fn test_custom_metrics_update() {
    let config = test_helpers::create_test_learning_config();
    let metrics = LearningMetrics::new(Arc::new(config))
        .await
        .expect("Should create metrics");

    metrics
        .set_custom_metric("metric_1".to_string(), 1.0)
        .await
        .expect("Should set metric");

    // Update the same metric
    metrics
        .set_custom_metric("metric_1".to_string(), 2.0)
        .await
        .expect("Should update metric");

    let value = metrics.get_custom_metric("metric_1").await;
    assert_eq!(value, Some(2.0));
}

#[tokio::test]
async fn test_performance_stability() {
    let config = test_helpers::create_test_learning_config();
    let metrics = LearningMetrics::new(Arc::new(config))
        .await
        .expect("Should create metrics");

    let mut updates = HashMap::new();
    updates.insert("policy_stability".to_string(), 0.95);

    metrics
        .update_performance(updates)
        .await
        .expect("Should update performance");

    let performance = metrics.get_performance().await;
    assert_eq!(performance.policy_stability, 0.95);
}
