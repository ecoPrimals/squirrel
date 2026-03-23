// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Learning Metrics
//!
//! This module provides comprehensive metrics tracking for the Context Learning System.
//! It monitors learning performance, tracks progress, and provides analytics.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

use super::LearningSystemConfig;
use crate::error::Result;

/// Learning metrics system
#[derive(Debug)]
pub struct LearningMetrics {
    /// System configuration
    config: Arc<LearningSystemConfig>,

    /// Performance metrics
    performance: Arc<RwLock<LearningPerformance>>,

    /// Learning statistics
    stats: Arc<RwLock<LearningStats>>,

    /// Metrics history
    history: Arc<RwLock<Vec<MetricsSnapshot>>>,

    /// Custom metrics
    custom_metrics: Arc<RwLock<HashMap<String, f64>>>,
}

/// Learning performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningPerformance {
    /// Learning rate
    pub learning_rate: f64,

    /// Success rate
    pub success_rate: f64,

    /// Average reward
    pub average_reward: f64,

    /// Episode completion rate
    pub episode_completion_rate: f64,

    /// Training accuracy
    pub training_accuracy: f64,

    /// Convergence rate
    pub convergence_rate: f64,

    /// Exploration rate
    pub exploration_rate: f64,

    /// Policy stability
    pub policy_stability: f64,

    /// Last performance update
    pub last_update: DateTime<Utc>,
}

impl Default for LearningPerformance {
    fn default() -> Self {
        Self {
            learning_rate: 0.0,
            success_rate: 0.0,
            average_reward: 0.0,
            episode_completion_rate: 0.0,
            training_accuracy: 0.0,
            convergence_rate: 0.0,
            exploration_rate: 1.0,
            policy_stability: 0.0,
            last_update: Utc::now(),
        }
    }
}

/// Learning statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningStats {
    /// Total episodes
    pub total_episodes: usize,

    /// Successful episodes
    pub successful_episodes: usize,

    /// Total actions taken
    pub total_actions: usize,

    /// Total rewards collected
    pub total_rewards: f64,

    /// Total learning time
    pub total_learning_time: f64,

    /// Average episode length
    pub average_episode_length: f64,

    /// Average reward per episode
    pub average_reward_per_episode: f64,

    /// Policy updates
    pub policy_updates: usize,

    /// Rule adaptations
    pub rule_adaptations: usize,

    /// Contexts learned
    pub contexts_learned: usize,

    /// Last statistics update
    pub last_update: DateTime<Utc>,
}

impl Default for LearningStats {
    fn default() -> Self {
        Self {
            total_episodes: 0,
            successful_episodes: 0,
            total_actions: 0,
            total_rewards: 0.0,
            total_learning_time: 0.0,
            average_episode_length: 0.0,
            average_reward_per_episode: 0.0,
            policy_updates: 0,
            rule_adaptations: 0,
            contexts_learned: 0,
            last_update: Utc::now(),
        }
    }
}

/// Metrics snapshot for historical tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// Snapshot ID
    pub id: String,

    /// Snapshot timestamp
    pub timestamp: DateTime<Utc>,

    /// Performance at time of snapshot
    pub performance: LearningPerformance,

    /// Statistics at time of snapshot
    pub stats: LearningStats,

    /// Custom metrics
    pub custom_metrics: HashMap<String, f64>,
}

impl LearningMetrics {
    /// Create a new learning metrics system
    pub async fn new(config: Arc<LearningSystemConfig>) -> Result<Self> {
        Ok(Self {
            config,
            performance: Arc::new(RwLock::new(LearningPerformance::default())),
            stats: Arc::new(RwLock::new(LearningStats::default())),
            history: Arc::new(RwLock::new(Vec::new())),
            custom_metrics: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Initialize the metrics system
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing learning metrics system");

        // Initialize with default values
        let mut performance = self.performance.write().await;
        performance.learning_rate = self.config.learning_rate;
        performance.exploration_rate = self.config.exploration_rate;
        performance.last_update = Utc::now();

        let mut stats = self.stats.write().await;
        stats.last_update = Utc::now();

        info!("Learning metrics system initialized successfully");
        Ok(())
    }

    /// Start the metrics system
    pub async fn start(&self) -> Result<()> {
        info!("Starting learning metrics system");

        // Take initial snapshot
        self.take_snapshot().await?;

        Ok(())
    }

    /// Update performance metrics
    pub async fn update_performance(&self, updates: HashMap<String, f64>) -> Result<()> {
        let mut performance = self.performance.write().await;

        for (key, value) in updates {
            match key.as_str() {
                "learning_rate" => performance.learning_rate = value,
                "success_rate" => performance.success_rate = value,
                "average_reward" => performance.average_reward = value,
                "episode_completion_rate" => performance.episode_completion_rate = value,
                "training_accuracy" => performance.training_accuracy = value,
                "convergence_rate" => performance.convergence_rate = value,
                "exploration_rate" => performance.exploration_rate = value,
                "policy_stability" => performance.policy_stability = value,
                _ => {
                    // Store as custom metric
                    let mut custom_metrics = self.custom_metrics.write().await;
                    custom_metrics.insert(key, value);
                }
            }
        }

        performance.last_update = Utc::now();

        debug!("Updated performance metrics");
        Ok(())
    }

    /// Update statistics
    pub async fn update_stats(&self, updates: HashMap<String, f64>) -> Result<()> {
        let mut stats = self.stats.write().await;

        for (key, value) in updates {
            match key.as_str() {
                "total_episodes" => stats.total_episodes = value as usize,
                "successful_episodes" => stats.successful_episodes = value as usize,
                "total_actions" => stats.total_actions = value as usize,
                "total_rewards" => stats.total_rewards = value,
                "total_learning_time" => stats.total_learning_time = value,
                "average_episode_length" => stats.average_episode_length = value,
                "average_reward_per_episode" => stats.average_reward_per_episode = value,
                "policy_updates" => stats.policy_updates = value as usize,
                "rule_adaptations" => stats.rule_adaptations = value as usize,
                "contexts_learned" => stats.contexts_learned = value as usize,
                _ => {
                    // Store as custom metric
                    let mut custom_metrics = self.custom_metrics.write().await;
                    custom_metrics.insert(key, value);
                }
            }
        }

        stats.last_update = Utc::now();

        debug!("Updated learning statistics");
        Ok(())
    }

    /// Record episode completion
    pub async fn record_episode(
        &self,
        success: bool,
        reward: f64,
        actions: usize,
        duration: f64,
    ) -> Result<()> {
        let mut stats = self.stats.write().await;
        let mut performance = self.performance.write().await;

        // Update episode statistics
        stats.total_episodes += 1;
        if success {
            stats.successful_episodes += 1;
        }

        stats.total_actions += actions;
        stats.total_rewards += reward;
        stats.total_learning_time += duration;

        // Update averages
        stats.average_episode_length = stats.total_actions as f64 / stats.total_episodes as f64;
        stats.average_reward_per_episode = stats.total_rewards / stats.total_episodes as f64;

        // Update performance metrics
        performance.success_rate = stats.successful_episodes as f64 / stats.total_episodes as f64;
        performance.average_reward = stats.average_reward_per_episode;
        performance.episode_completion_rate = 1.0; // All episodes are completing

        performance.last_update = Utc::now();
        stats.last_update = Utc::now();

        debug!(
            "Recorded episode: success={}, reward={:.2}, actions={}, duration={:.2}s",
            success, reward, actions, duration
        );

        Ok(())
    }

    /// Record policy update
    pub async fn record_policy_update(&self, accuracy: f64, loss: f64) -> Result<()> {
        let mut stats = self.stats.write().await;
        let mut performance = self.performance.write().await;

        stats.policy_updates += 1;
        performance.training_accuracy = accuracy;

        // Update policy stability (simplified)
        let stability_factor = 1.0 - loss.min(1.0);
        performance.policy_stability =
            (performance.policy_stability * 0.9) + (stability_factor * 0.1);

        performance.last_update = Utc::now();
        stats.last_update = Utc::now();

        debug!(
            "Recorded policy update: accuracy={:.4}, loss={:.4}",
            accuracy, loss
        );
        Ok(())
    }

    /// Record rule adaptation
    pub async fn record_rule_adaptation(&self, rule_id: &str, improvement: f64) -> Result<()> {
        let mut stats = self.stats.write().await;
        stats.rule_adaptations += 1;
        stats.last_update = Utc::now();

        debug!(
            "Recorded rule adaptation: rule={}, improvement={:.2}",
            rule_id, improvement
        );
        Ok(())
    }

    /// Take a metrics snapshot
    pub async fn take_snapshot(&self) -> Result<String> {
        let performance = self.performance.read().await;
        let stats = self.stats.read().await;
        let custom_metrics = self.custom_metrics.read().await;

        let snapshot = MetricsSnapshot {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            performance: performance.clone(),
            stats: stats.clone(),
            custom_metrics: custom_metrics.clone(),
        };

        let snapshot_id = snapshot.id.clone();

        // Store snapshot
        let mut history = self.history.write().await;
        history.push(snapshot);

        // Keep history size manageable
        if history.len() > 1000 {
            history.remove(0);
        }

        debug!("Took metrics snapshot: {}", snapshot_id);
        Ok(snapshot_id)
    }

    /// Get current performance metrics
    pub async fn get_performance(&self) -> LearningPerformance {
        self.performance.read().await.clone()
    }

    /// Get current statistics
    pub async fn get_stats(&self) -> LearningStats {
        self.stats.read().await.clone()
    }

    /// Get custom metrics
    pub async fn get_custom_metrics(&self) -> HashMap<String, f64> {
        self.custom_metrics.read().await.clone()
    }

    /// Get metrics history
    pub async fn get_history(&self) -> Vec<MetricsSnapshot> {
        self.history.read().await.clone()
    }

    /// Get snapshot by ID
    pub async fn get_snapshot(&self, id: &str) -> Option<MetricsSnapshot> {
        let history = self.history.read().await;
        history.iter().find(|s| s.id == id).cloned()
    }

    /// Set custom metric
    pub async fn set_custom_metric(&self, key: String, value: f64) -> Result<()> {
        let mut custom_metrics = self.custom_metrics.write().await;
        custom_metrics.insert(key.clone(), value);

        debug!("Set custom metric: {}={:.4}", key, value);
        Ok(())
    }

    /// Get custom metric
    pub async fn get_custom_metric(&self, key: &str) -> Option<f64> {
        let custom_metrics = self.custom_metrics.read().await;
        custom_metrics.get(key).copied()
    }

    /// Calculate performance trends
    pub async fn calculate_trends(&self, window_size: usize) -> Result<HashMap<String, f64>> {
        let history = self.history.read().await;
        let mut trends = HashMap::new();

        if history.len() >= window_size {
            let recent_snapshots = &history[history.len() - window_size..];
            let older_snapshots =
                &history[history.len() - window_size * 2..history.len() - window_size];

            if !older_snapshots.is_empty() {
                // Calculate trends for key metrics
                let recent_success_rate = recent_snapshots
                    .iter()
                    .map(|s| s.performance.success_rate)
                    .sum::<f64>()
                    / recent_snapshots.len() as f64;

                let older_success_rate = older_snapshots
                    .iter()
                    .map(|s| s.performance.success_rate)
                    .sum::<f64>()
                    / older_snapshots.len() as f64;

                trends.insert(
                    "success_rate_trend".to_string(),
                    recent_success_rate - older_success_rate,
                );

                let recent_avg_reward = recent_snapshots
                    .iter()
                    .map(|s| s.performance.average_reward)
                    .sum::<f64>()
                    / recent_snapshots.len() as f64;

                let older_avg_reward = older_snapshots
                    .iter()
                    .map(|s| s.performance.average_reward)
                    .sum::<f64>()
                    / older_snapshots.len() as f64;

                trends.insert(
                    "average_reward_trend".to_string(),
                    recent_avg_reward - older_avg_reward,
                );
            }
        }

        Ok(trends)
    }

    /// Generate metrics report
    pub async fn generate_report(&self) -> Result<Value> {
        let performance = self.get_performance().await;
        let stats = self.get_stats().await;
        let custom_metrics = self.get_custom_metrics().await;
        let trends = self.calculate_trends(10).await?;

        let report = serde_json::json!({
            "timestamp": Utc::now(),
            "performance": performance,
            "statistics": stats,
            "custom_metrics": custom_metrics,
            "trends": trends,
            "summary": {
                "total_episodes": stats.total_episodes,
                "success_rate": performance.success_rate,
                "average_reward": performance.average_reward,
                "training_accuracy": performance.training_accuracy,
                "policy_updates": stats.policy_updates,
                "rule_adaptations": stats.rule_adaptations
            }
        });

        Ok(report)
    }

    /// Clear metrics history
    pub async fn clear_history(&self) -> Result<()> {
        let mut history = self.history.write().await;
        history.clear();

        info!("Cleared metrics history");
        Ok(())
    }

    /// Export metrics to JSON
    pub async fn export_metrics(&self) -> Result<Value> {
        let performance = self.get_performance().await;
        let stats = self.get_stats().await;
        let history = self.get_history().await;
        let custom_metrics = self.get_custom_metrics().await;

        let export = serde_json::json!({
            "export_timestamp": Utc::now(),
            "current_performance": performance,
            "current_stats": stats,
            "custom_metrics": custom_metrics,
            "history": history
        });

        Ok(export)
    }

    /// Stop the metrics system
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping learning metrics system");

        // Take final snapshot
        self.take_snapshot().await?;

        info!("Learning metrics system stopped");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::learning::LearningSystemConfig;
    use serde_json::json;
    use std::collections::HashMap;

    fn test_config() -> Arc<LearningSystemConfig> {
        Arc::new(LearningSystemConfig::default())
    }

    #[tokio::test]
    async fn learning_performance_and_stats_serde_roundtrip() {
        let p = LearningPerformance::default();
        let json = serde_json::to_string(&p).unwrap();
        let back: LearningPerformance = serde_json::from_str(&json).unwrap();
        assert!((back.exploration_rate - p.exploration_rate).abs() < f64::EPSILON);

        let s = LearningStats::default();
        let json = serde_json::to_string(&s).unwrap();
        let _: LearningStats = serde_json::from_str(&json).unwrap();
    }

    #[tokio::test]
    async fn lifecycle_initialize_start_stop() {
        let m = LearningMetrics::new(test_config()).await.unwrap();
        m.initialize().await.unwrap();
        m.start().await.unwrap();
        assert_eq!(m.get_history().await.len(), 1);
        m.stop().await.unwrap();
        assert!(m.get_history().await.len() >= 2);
    }

    #[tokio::test]
    async fn update_performance_known_and_custom_keys() {
        let m = LearningMetrics::new(test_config()).await.unwrap();
        m.initialize().await.unwrap();

        let mut u = HashMap::new();
        u.insert("learning_rate".to_string(), 0.5);
        u.insert("success_rate".to_string(), 0.9);
        u.insert("custom_xyz".to_string(), 3.0);
        m.update_performance(u).await.unwrap();

        let p = m.get_performance().await;
        assert!((p.learning_rate - 0.5).abs() < 1e-9);
        assert!((p.success_rate - 0.9).abs() < 1e-9);
        assert!((m.get_custom_metric("custom_xyz").await.unwrap() - 3.0).abs() < 1e-9);
    }

    #[tokio::test]
    async fn update_stats_branches_and_custom() {
        let m = LearningMetrics::new(test_config()).await.unwrap();
        let mut u = HashMap::new();
        u.insert("total_episodes".to_string(), 5.0);
        u.insert("policy_updates".to_string(), 2.0);
        u.insert("extra_stat".to_string(), 1.25);
        m.update_stats(u).await.unwrap();
        let s = m.get_stats().await;
        assert_eq!(s.total_episodes, 5);
        assert_eq!(s.policy_updates, 2);
        assert!((m.get_custom_metric("extra_stat").await.unwrap() - 1.25).abs() < 1e-9);
    }

    #[tokio::test]
    async fn record_episode_and_policy_and_rule() {
        let m = LearningMetrics::new(test_config()).await.unwrap();
        m.record_episode(true, 1.0, 10, 0.5).await.unwrap();
        m.record_episode(false, 0.0, 3, 0.1).await.unwrap();
        let s = m.get_stats().await;
        assert_eq!(s.total_episodes, 2);
        assert_eq!(s.successful_episodes, 1);

        m.record_policy_update(0.88, 0.5).await.unwrap();
        m.record_rule_adaptation("r1", 0.2).await.unwrap();
        assert_eq!(m.get_stats().await.policy_updates, 1);
        assert_eq!(m.get_stats().await.rule_adaptations, 1);
    }

    #[tokio::test]
    async fn snapshots_history_clear_and_get_by_id() {
        let m = LearningMetrics::new(test_config()).await.unwrap();
        let id = m.take_snapshot().await.unwrap();
        assert!(m.get_snapshot(&id).await.is_some());
        assert!(m.get_snapshot("missing").await.is_none());
        m.clear_history().await.unwrap();
        assert!(m.get_history().await.is_empty());
    }

    #[tokio::test]
    async fn calculate_trends_when_insufficient_history() {
        let m = LearningMetrics::new(test_config()).await.unwrap();
        let t = m.calculate_trends(10).await.unwrap();
        assert!(t.is_empty());
    }

    #[tokio::test]
    async fn calculate_trends_with_two_windows() {
        let m = LearningMetrics::new(test_config()).await.unwrap();
        for i in 0..25 {
            let mut u = HashMap::new();
            u.insert("success_rate".to_string(), if i < 12 { 0.2 } else { 0.8 });
            u.insert("average_reward".to_string(), if i < 12 { 0.1 } else { 0.9 });
            m.update_performance(u).await.unwrap();
            m.take_snapshot().await.unwrap();
        }
        let trends = m.calculate_trends(10).await.unwrap();
        assert!(trends.contains_key("success_rate_trend"));
        assert!(trends.contains_key("average_reward_trend"));
    }

    #[tokio::test]
    async fn generate_report_and_export() {
        let m = LearningMetrics::new(test_config()).await.unwrap();
        m.initialize().await.unwrap();
        m.set_custom_metric("k".to_string(), 2.0).await.unwrap();
        let report = m.generate_report().await.unwrap();
        assert!(report.get("summary").is_some());
        let exp = m.export_metrics().await.unwrap();
        assert!(exp.get("history").is_some());
    }

    #[tokio::test]
    async fn metrics_snapshot_serde_roundtrip() {
        let snap = MetricsSnapshot {
            id: "id1".to_string(),
            timestamp: chrono::Utc::now(),
            performance: LearningPerformance::default(),
            stats: LearningStats::default(),
            custom_metrics: HashMap::from([("a".to_string(), 1.0)]),
        };
        let v = json!(snap);
        let back: MetricsSnapshot = serde_json::from_value(v).unwrap();
        assert_eq!(back.id, "id1");
    }

    #[tokio::test]
    async fn update_performance_covers_all_named_fields() {
        let m = LearningMetrics::new(test_config()).await.unwrap();
        let mut u = HashMap::new();
        u.insert("episode_completion_rate".to_string(), 0.25);
        u.insert("training_accuracy".to_string(), 0.9);
        u.insert("convergence_rate".to_string(), 0.12);
        u.insert("policy_stability".to_string(), 0.88);
        u.insert("exploration_rate".to_string(), 0.3);
        u.insert("average_reward".to_string(), 1.1);
        m.update_performance(u).await.unwrap();
        let p = m.get_performance().await;
        assert!((p.episode_completion_rate - 0.25).abs() < 1e-9);
        assert!((p.training_accuracy - 0.9).abs() < 1e-9);
        assert!((p.convergence_rate - 0.12).abs() < 1e-9);
        assert!((p.policy_stability - 0.88).abs() < 1e-9);
        assert!((p.exploration_rate - 0.3).abs() < 1e-9);
        assert!((p.average_reward - 1.1).abs() < 1e-9);
    }

    #[tokio::test]
    async fn update_stats_covers_remaining_counters() {
        let m = LearningMetrics::new(test_config()).await.unwrap();
        let mut u = HashMap::new();
        u.insert("successful_episodes".to_string(), 3.0);
        u.insert("total_actions".to_string(), 30.0);
        u.insert("total_rewards".to_string(), 9.0);
        u.insert("total_learning_time".to_string(), 12.5);
        u.insert("average_episode_length".to_string(), 4.0);
        u.insert("average_reward_per_episode".to_string(), 3.0);
        u.insert("rule_adaptations".to_string(), 2.0);
        u.insert("contexts_learned".to_string(), 7.0);
        m.update_stats(u).await.unwrap();
        let s = m.get_stats().await;
        assert_eq!(s.successful_episodes, 3);
        assert_eq!(s.total_actions, 30);
        assert!((s.total_rewards - 9.0).abs() < 1e-9);
        assert!((s.total_learning_time - 12.5).abs() < 1e-9);
        assert!((s.average_episode_length - 4.0).abs() < 1e-9);
        assert!((s.average_reward_per_episode - 3.0).abs() < 1e-9);
        assert_eq!(s.rule_adaptations, 2);
        assert_eq!(s.contexts_learned, 7);
    }

    #[tokio::test]
    async fn record_policy_update_clamps_large_loss_for_stability() {
        let m = LearningMetrics::new(test_config()).await.unwrap();
        m.record_policy_update(0.5, 5.0).await.unwrap();
        let p = m.get_performance().await;
        // Loss > 1 is clamped to 1.0, so stability factor is 0; EMA still yields 0 on first update.
        assert!((p.policy_stability - 0.0).abs() < 1e-9);
        assert!((p.training_accuracy - 0.5).abs() < 1e-9);
        m.record_policy_update(0.6, 0.25).await.unwrap();
        assert!(m.get_performance().await.policy_stability > 0.0);
    }

    #[tokio::test]
    async fn snapshot_history_trims_past_1000() {
        let m = LearningMetrics::new(test_config()).await.unwrap();
        for _ in 0..1002 {
            m.take_snapshot().await.unwrap();
        }
        assert_eq!(m.get_history().await.len(), 1000);
    }
}
