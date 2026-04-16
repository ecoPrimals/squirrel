// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Data types for the learning reward subsystem.
//!
//! This module holds serializable context, metrics, and history structures used when
//! computing and recording rewards. Calculator implementations and [`super::reward::RewardSystem`]
//! live in [`super::reward`].

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::engine::{RLAction, RLState};

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
