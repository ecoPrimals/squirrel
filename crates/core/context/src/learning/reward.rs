// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Reward System
//!
//! This module implements the reward system for the Context Learning System.
//! It calculates rewards based on context management outcomes, rule effectiveness,
//! and overall system performance to guide reinforcement learning.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info};
use uuid::Uuid;

use super::{
    LearningSystemConfig,
    engine::{RLAction, RLState},
};
use crate::error::Result;

/// Reward system for calculating learning rewards
#[derive(Debug)]
pub struct RewardSystem {
    /// System configuration
    #[expect(dead_code, reason = "planned feature not yet wired")]
    config: Arc<LearningSystemConfig>,

    /// Reward calculators
    calculators: Arc<RwLock<HashMap<String, RewardBackend>>>,

    /// Reward history
    reward_history: Arc<RwLock<Vec<RewardEntry>>>,

    /// Reward metrics
    metrics: Arc<Mutex<RewardMetrics>>,

    /// Baseline rewards for normalization
    baselines: Arc<RwLock<HashMap<String, f64>>>,
}

/// Reward calculator trait
pub trait RewardCalculator: Send + Sync + std::fmt::Debug {
    /// Calculate reward for an action
    fn calculate_reward(&self, context: &RewardContext) -> Result<f64>;

    /// Get calculator name
    fn name(&self) -> &str;

    /// Get calculator weight
    fn weight(&self) -> f64;
}

/// Reward context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardContext {
    /// Action that was taken
    pub action: RLAction,

    /// Previous context state
    pub previous_state: RLState,

    /// Current context state
    pub current_state: RLState,

    /// Context performance metrics
    pub performance_metrics: PerformanceMetrics,

    /// Rule evaluation results
    pub rule_results: Option<RuleResults>,

    /// Error information
    pub error_info: Option<ErrorInfo>,

    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Performance metrics for reward calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Context synchronization status
    pub sync_status: bool,

    /// Context state version
    pub version: u64,

    /// Number of active contexts
    pub active_contexts: usize,

    /// Memory usage
    pub memory_usage: f64,

    /// Processing time
    pub processing_time: f64,

    /// Success rate
    pub success_rate: f64,

    /// Error rate
    pub error_rate: f64,

    /// Throughput
    pub throughput: f64,
}

/// Rule evaluation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleResults {
    /// Number of rules applied
    pub rules_applied: usize,

    /// Number of successful rule applications
    pub successful_applications: usize,

    /// Rule efficiency score
    pub efficiency_score: f64,

    /// Rules that failed
    pub failed_rules: Vec<String>,

    /// Rule execution time
    pub execution_time: f64,
}

/// Error information for penalty calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    /// Error type
    pub error_type: String,

    /// Error severity
    pub severity: ErrorSeverity,

    /// Error message
    pub message: String,

    /// Recovery possible
    pub recoverable: bool,

    /// Impact on system
    pub impact: f64,
}

/// Error severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

/// Reward entry for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardEntry {
    /// Entry ID
    pub id: String,

    /// Context ID
    pub context_id: String,

    /// Action ID
    pub action_id: String,

    /// Calculated reward
    pub reward: f64,

    /// Reward breakdown
    pub breakdown: RewardBreakdown,

    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Reward breakdown by component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardBreakdown {
    /// Base reward
    pub base_reward: f64,

    /// Performance bonus
    pub performance_bonus: f64,

    /// Efficiency bonus
    pub efficiency_bonus: f64,

    /// Synchronization bonus
    pub sync_bonus: f64,

    /// Error penalty
    pub error_penalty: f64,

    /// Time penalty
    pub time_penalty: f64,

    /// Final reward
    pub final_reward: f64,
}

/// Reward metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardMetrics {
    /// Total rewards calculated
    pub total_rewards: usize,

    /// Average reward
    pub average_reward: f64,

    /// Maximum reward
    pub max_reward: f64,

    /// Minimum reward
    pub min_reward: f64,

    /// Positive rewards
    pub positive_rewards: usize,

    /// Negative rewards
    pub negative_rewards: usize,

    /// Reward variance
    pub reward_variance: f64,

    /// Last calculation time
    pub last_calculation: DateTime<Utc>,
}

impl Default for RewardMetrics {
    fn default() -> Self {
        Self {
            total_rewards: 0,
            average_reward: 0.0,
            max_reward: f64::NEG_INFINITY,
            min_reward: f64::INFINITY,
            positive_rewards: 0,
            negative_rewards: 0,
            reward_variance: 0.0,
            last_calculation: Utc::now(),
        }
    }
}

/// Success reward calculator
#[derive(Debug)]
pub struct SuccessRewardCalculator {
    /// Calculator name
    name: String,

    /// Calculator weight
    weight: f64,

    /// Success reward value
    success_reward: f64,

    /// Failure penalty
    failure_penalty: f64,
}

impl SuccessRewardCalculator {
    pub fn new(success_reward: f64, failure_penalty: f64) -> Self {
        Self {
            name: "success".to_string(),
            weight: 1.0,
            success_reward,
            failure_penalty,
        }
    }
}

impl RewardCalculator for SuccessRewardCalculator {
    fn calculate_reward(&self, context: &RewardContext) -> Result<f64> {
        let reward = if context.error_info.is_some() {
            self.failure_penalty
        } else {
            self.success_reward
        };

        Ok(reward)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn weight(&self) -> f64 {
        self.weight
    }
}

/// Performance reward calculator
#[derive(Debug)]
pub struct PerformanceRewardCalculator {
    /// Calculator name
    name: String,

    /// Calculator weight
    weight: f64,

    /// Performance thresholds
    thresholds: PerformanceThresholds,
}

/// Performance thresholds for reward calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    /// Minimum success rate for bonus
    pub min_success_rate: f64,

    /// Maximum processing time for bonus
    pub max_processing_time: f64,

    /// Minimum throughput for bonus
    pub min_throughput: f64,

    /// Maximum error rate for bonus
    pub max_error_rate: f64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            min_success_rate: 0.8,
            max_processing_time: 1.0,
            min_throughput: 10.0,
            max_error_rate: 0.1,
        }
    }
}

impl PerformanceRewardCalculator {
    pub fn new(thresholds: PerformanceThresholds) -> Self {
        Self {
            name: "performance".to_string(),
            weight: 0.5,
            thresholds,
        }
    }
}

impl RewardCalculator for PerformanceRewardCalculator {
    fn calculate_reward(&self, context: &RewardContext) -> Result<f64> {
        let metrics = &context.performance_metrics;
        let mut reward = 0.0;

        // Success rate bonus
        if metrics.success_rate >= self.thresholds.min_success_rate {
            reward += (metrics.success_rate - self.thresholds.min_success_rate) * 5.0;
        }

        // Processing time penalty
        if metrics.processing_time > self.thresholds.max_processing_time {
            reward -= (metrics.processing_time - self.thresholds.max_processing_time) * 2.0;
        }

        // Throughput bonus
        if metrics.throughput >= self.thresholds.min_throughput {
            reward += (metrics.throughput / self.thresholds.min_throughput - 1.0) * 3.0;
        }

        // Error rate penalty
        if metrics.error_rate > self.thresholds.max_error_rate {
            reward -= (metrics.error_rate - self.thresholds.max_error_rate) * 10.0;
        }

        Ok(reward)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn weight(&self) -> f64 {
        self.weight
    }
}

/// Rule efficiency reward calculator
#[derive(Debug)]
pub struct RuleEfficiencyRewardCalculator {
    /// Calculator name
    name: String,

    /// Calculator weight
    weight: f64,

    /// Efficiency bonus multiplier
    efficiency_bonus: f64,
}

impl RuleEfficiencyRewardCalculator {
    pub fn new(efficiency_bonus: f64) -> Self {
        Self {
            name: "rule_efficiency".to_string(),
            weight: 0.3,
            efficiency_bonus,
        }
    }
}

impl RewardCalculator for RuleEfficiencyRewardCalculator {
    fn calculate_reward(&self, context: &RewardContext) -> Result<f64> {
        let reward = if let Some(rule_results) = &context.rule_results {
            let efficiency_score = rule_results.efficiency_score;
            let success_rate = if rule_results.rules_applied > 0 {
                rule_results.successful_applications as f64 / rule_results.rules_applied as f64
            } else {
                0.0
            };

            efficiency_score * success_rate * self.efficiency_bonus
        } else {
            0.0
        };

        Ok(reward)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn weight(&self) -> f64 {
        self.weight
    }
}

/// Synchronization reward calculator
#[derive(Debug)]
pub struct SynchronizationRewardCalculator {
    /// Calculator name
    name: String,

    /// Calculator weight
    weight: f64,

    /// Synchronization bonus
    sync_bonus: f64,
}

impl SynchronizationRewardCalculator {
    pub fn new(sync_bonus: f64) -> Self {
        Self {
            name: "synchronization".to_string(),
            weight: 0.2,
            sync_bonus,
        }
    }
}

impl RewardCalculator for SynchronizationRewardCalculator {
    fn calculate_reward(&self, context: &RewardContext) -> Result<f64> {
        let reward = if context.performance_metrics.sync_status {
            self.sync_bonus
        } else {
            0.0
        };

        Ok(reward)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn weight(&self) -> f64 {
        self.weight
    }
}

/// Reward calculation backend (enum dispatch instead of `Box<dyn RewardCalculator>`).
pub enum RewardBackend {
    /// Success / failure based reward.
    Success(SuccessRewardCalculator),
    /// Performance-threshold based reward.
    Performance(PerformanceRewardCalculator),
    /// Rule efficiency reward.
    RuleEfficiency(RuleEfficiencyRewardCalculator),
    /// Synchronization reward.
    Synchronization(SynchronizationRewardCalculator),
}

impl std::fmt::Debug for RewardBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success(c) => f.debug_tuple("Success").field(c).finish(),
            Self::Performance(c) => f.debug_tuple("Performance").field(c).finish(),
            Self::RuleEfficiency(c) => f.debug_tuple("RuleEfficiency").field(c).finish(),
            Self::Synchronization(c) => f.debug_tuple("Synchronization").field(c).finish(),
        }
    }
}

impl RewardCalculator for RewardBackend {
    fn calculate_reward(&self, context: &RewardContext) -> Result<f64> {
        match self {
            Self::Success(c) => c.calculate_reward(context),
            Self::Performance(c) => c.calculate_reward(context),
            Self::RuleEfficiency(c) => c.calculate_reward(context),
            Self::Synchronization(c) => c.calculate_reward(context),
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::Success(c) => c.name(),
            Self::Performance(c) => c.name(),
            Self::RuleEfficiency(c) => c.name(),
            Self::Synchronization(c) => c.name(),
        }
    }

    fn weight(&self) -> f64 {
        match self {
            Self::Success(c) => c.weight(),
            Self::Performance(c) => c.weight(),
            Self::RuleEfficiency(c) => c.weight(),
            Self::Synchronization(c) => c.weight(),
        }
    }
}

impl RewardSystem {
    /// Create a new reward system
    pub async fn new(config: Arc<LearningSystemConfig>) -> Result<Self> {
        let system = Self {
            config,
            calculators: Arc::new(RwLock::new(HashMap::new())),
            reward_history: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(Mutex::new(RewardMetrics::default())),
            baselines: Arc::new(RwLock::new(HashMap::new())),
        };

        // Initialize default calculators
        system.initialize_default_calculators().await?;

        Ok(system)
    }

    /// Initialize default reward calculators
    async fn initialize_default_calculators(&self) -> Result<()> {
        let mut calculators = self.calculators.write().await;

        // Success/failure calculator
        let success_calculator = SuccessRewardCalculator::new(10.0, -5.0);
        calculators.insert(
            "success".to_string(),
            RewardBackend::Success(success_calculator),
        );

        // Performance calculator
        let performance_calculator =
            PerformanceRewardCalculator::new(PerformanceThresholds::default());
        calculators.insert(
            "performance".to_string(),
            RewardBackend::Performance(performance_calculator),
        );

        // Rule efficiency calculator
        let rule_calculator = RuleEfficiencyRewardCalculator::new(2.0);
        calculators.insert(
            "rule_efficiency".to_string(),
            RewardBackend::RuleEfficiency(rule_calculator),
        );

        // Synchronization calculator
        let sync_calculator = SynchronizationRewardCalculator::new(1.0);
        calculators.insert(
            "synchronization".to_string(),
            RewardBackend::Synchronization(sync_calculator),
        );

        info!("Initialized {} reward calculators", calculators.len());
        Ok(())
    }

    /// Initialize the reward system
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing reward system");

        // Initialize baselines
        self.initialize_baselines().await?;

        info!("Reward system initialized successfully");
        Ok(())
    }

    /// Initialize reward baselines
    async fn initialize_baselines(&self) -> Result<()> {
        let mut baselines = self.baselines.write().await;

        // Set default baselines
        baselines.insert("success".to_string(), 0.0);
        baselines.insert("performance".to_string(), 0.0);
        baselines.insert("rule_efficiency".to_string(), 0.0);
        baselines.insert("synchronization".to_string(), 0.0);

        Ok(())
    }

    /// Calculate reward for a context
    pub async fn calculate_reward(&self, context: RewardContext) -> Result<f64> {
        let calculators = self.calculators.read().await;
        let mut total_reward = 0.0;
        let mut breakdown = RewardBreakdown {
            base_reward: 0.0,
            performance_bonus: 0.0,
            efficiency_bonus: 0.0,
            sync_bonus: 0.0,
            error_penalty: 0.0,
            time_penalty: 0.0,
            final_reward: 0.0,
        };

        // Calculate rewards from all calculators
        for (name, calculator) in calculators.iter() {
            let calculator_reward = calculator.calculate_reward(&context)?;
            let weighted_reward = calculator_reward * calculator.weight();

            total_reward += weighted_reward;

            // Update breakdown
            match name.as_str() {
                "success" => breakdown.base_reward = weighted_reward,
                "performance" => breakdown.performance_bonus = weighted_reward,
                "rule_efficiency" => breakdown.efficiency_bonus = weighted_reward,
                "synchronization" => breakdown.sync_bonus = weighted_reward,
                _ => {}
            }
        }

        // Apply error penalties
        if let Some(error_info) = &context.error_info {
            let error_penalty = self.calculate_error_penalty(error_info).await?;
            total_reward -= error_penalty;
            breakdown.error_penalty = error_penalty;
        }

        // Apply time penalties
        let time_penalty = self.calculate_time_penalty(&context).await?;
        total_reward -= time_penalty;
        breakdown.time_penalty = time_penalty;

        breakdown.final_reward = total_reward;

        // Record reward
        self.record_reward(&context, total_reward, breakdown)
            .await?;

        // Update metrics
        self.update_metrics(total_reward).await?;

        debug!(
            "Calculated reward: {:.2} for action: {}",
            total_reward, context.action.action_type
        );
        Ok(total_reward)
    }

    /// Calculate error penalty
    async fn calculate_error_penalty(&self, error_info: &ErrorInfo) -> Result<f64> {
        let base_penalty = match error_info.severity {
            ErrorSeverity::Low => 1.0,
            ErrorSeverity::Medium => 3.0,
            ErrorSeverity::High => 7.0,
            ErrorSeverity::Critical => 15.0,
        };

        let impact_multiplier = (error_info.impact + 1.0).ln();
        let recovery_factor = if error_info.recoverable { 0.5 } else { 1.0 };

        Ok(base_penalty * impact_multiplier * recovery_factor)
    }

    /// Calculate time penalty
    async fn calculate_time_penalty(&self, context: &RewardContext) -> Result<f64> {
        let processing_time = context.performance_metrics.processing_time;
        let penalty = if processing_time > 1.0 {
            (processing_time - 1.0) * 0.1
        } else {
            0.0
        };

        Ok(penalty)
    }

    /// Record reward entry
    async fn record_reward(
        &self,
        context: &RewardContext,
        reward: f64,
        breakdown: RewardBreakdown,
    ) -> Result<()> {
        let entry = RewardEntry {
            id: Uuid::new_v4().to_string(),
            context_id: context.current_state.context_id.clone(),
            action_id: context.action.id.clone(),
            reward,
            breakdown,
            timestamp: Utc::now(),
        };

        let mut history = self.reward_history.write().await;
        history.push(entry);

        // Keep history size manageable
        if history.len() > 10000 {
            history.remove(0);
        }

        Ok(())
    }

    /// Update reward metrics
    async fn update_metrics(&self, reward: f64) -> Result<()> {
        let mut metrics = self.metrics.lock().await;

        metrics.total_rewards += 1;

        // Update average
        let old_avg = metrics.average_reward;
        metrics.average_reward =
            (old_avg * (metrics.total_rewards - 1) as f64 + reward) / metrics.total_rewards as f64;

        // Update min/max
        if reward > metrics.max_reward {
            metrics.max_reward = reward;
        }
        if reward < metrics.min_reward {
            metrics.min_reward = reward;
        }

        // Update positive/negative counts
        if reward > 0.0 {
            metrics.positive_rewards += 1;
        } else if reward < 0.0 {
            metrics.negative_rewards += 1;
        }

        // Update variance (simplified)
        let diff = reward - metrics.average_reward;
        metrics.reward_variance = (metrics.reward_variance * (metrics.total_rewards - 1) as f64
            + diff * diff)
            / metrics.total_rewards as f64;

        metrics.last_calculation = Utc::now();

        Ok(())
    }

    /// Add custom reward calculator
    pub async fn add_calculator(&self, name: String, calculator: RewardBackend) -> Result<()> {
        let mut calculators = self.calculators.write().await;
        calculators.insert(name.clone(), calculator);

        info!("Added custom reward calculator: {}", name);
        Ok(())
    }

    /// Remove reward calculator
    pub async fn remove_calculator(&self, name: &str) -> Result<()> {
        let mut calculators = self.calculators.write().await;
        calculators.remove(name);

        info!("Removed reward calculator: {}", name);
        Ok(())
    }

    /// Get reward metrics
    pub async fn get_metrics(&self) -> RewardMetrics {
        self.metrics.lock().await.clone()
    }

    /// Get reward history
    pub async fn get_reward_history(&self) -> Vec<RewardEntry> {
        self.reward_history.read().await.clone()
    }

    /// Get reward history for context
    pub async fn get_context_rewards(&self, context_id: &str) -> Vec<RewardEntry> {
        let history = self.reward_history.read().await;
        history
            .iter()
            .filter(|entry| entry.context_id == context_id)
            .cloned()
            .collect()
    }

    /// Clear reward history
    pub async fn clear_history(&self) -> Result<()> {
        let mut history = self.reward_history.write().await;
        history.clear();

        // Reset metrics
        let mut metrics = self.metrics.lock().await;
        *metrics = RewardMetrics::default();

        info!("Cleared reward history and metrics");
        Ok(())
    }

    /// Update baselines from recent performance
    pub async fn update_baselines(&self) -> Result<()> {
        let history = self.reward_history.read().await;
        let mut baselines = self.baselines.write().await;

        if history.len() >= 100 {
            // Calculate baseline as average of recent rewards
            let recent_rewards: Vec<f64> = history
                .iter()
                .rev()
                .take(100)
                .map(|entry| entry.reward)
                .collect();

            let baseline = recent_rewards.iter().sum::<f64>() / recent_rewards.len() as f64;
            baselines.insert("overall".to_string(), baseline);

            info!("Updated reward baseline to: {:.2}", baseline);
        }

        Ok(())
    }

    /// Get current baselines
    pub async fn get_baselines(&self) -> HashMap<String, f64> {
        self.baselines.read().await.clone()
    }
}
