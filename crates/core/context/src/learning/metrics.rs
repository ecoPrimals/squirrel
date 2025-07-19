//! Learning Metrics
//!
//! This module provides comprehensive metrics tracking for the Context Learning System.
//! It monitors learning performance, tracks progress, and provides analytics.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info, warn};
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
