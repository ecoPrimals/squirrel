// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Types for the experience replay system: sampling strategies, statistics,
//! batch structures, and priority metadata.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::engine::RLExperience;

/// Sampling strategy for experience replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SamplingStrategy {
    /// Uniform random sampling
    Uniform,
    /// Prioritized experience replay
    Prioritized(PrioritizedConfig),
    /// Temporal sampling (recent experiences favored)
    Temporal(TemporalConfig),
    /// Balanced sampling (mix of old and new)
    Balanced(BalancedConfig),
}

/// Prioritized experience replay configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritizedConfig {
    /// Priority exponent alpha
    pub alpha: f64,
    /// Importance sampling exponent beta
    pub beta: f64,
    /// Beta annealing rate
    pub beta_annealing_rate: f64,
    /// Maximum beta value
    pub max_beta: f64,
    /// Small constant to avoid zero probabilities
    pub epsilon: f64,
}

impl Default for PrioritizedConfig {
    fn default() -> Self {
        Self {
            alpha: 0.6,
            beta: 0.4,
            beta_annealing_rate: 0.001,
            max_beta: 1.0,
            epsilon: 1e-6,
        }
    }
}

/// Temporal sampling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalConfig {
    /// Decay factor for older experiences
    pub decay_factor: f64,
    /// Minimum sampling probability
    pub min_probability: f64,
}

impl Default for TemporalConfig {
    fn default() -> Self {
        Self {
            decay_factor: 0.95,
            min_probability: 0.01,
        }
    }
}

/// Balanced sampling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalancedConfig {
    /// Ratio of recent to old experiences
    pub recent_ratio: f64,
    /// Threshold for considering experience as recent
    pub recent_threshold: f64,
}

impl Default for BalancedConfig {
    fn default() -> Self {
        Self {
            recent_ratio: 0.7,
            recent_threshold: 0.2,
        }
    }
}

/// Experience statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceStats {
    /// Total experiences stored
    pub total_experiences: usize,
    /// Current buffer size
    pub current_size: usize,
    /// Buffer utilization
    pub utilization: f64,
    /// Average experience priority
    pub average_priority: f64,
    /// Number of samples drawn
    pub samples_drawn: usize,
    /// Average reward in buffer
    pub average_reward: f64,
    /// Success rate in buffer
    pub success_rate: f64,
    /// Oldest experience age
    pub oldest_experience_age: f64,
    /// Last update time
    pub last_update: DateTime<Utc>,
}

impl Default for ExperienceStats {
    fn default() -> Self {
        Self {
            total_experiences: 0,
            current_size: 0,
            utilization: 0.0,
            average_priority: 1.0,
            samples_drawn: 0,
            average_reward: 0.0,
            success_rate: 0.0,
            oldest_experience_age: 0.0,
            last_update: Utc::now(),
        }
    }
}

/// Experience batch for training
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceBatch {
    /// Batch of experiences
    pub experiences: Vec<RLExperience>,
    /// Importance sampling weights (for prioritized replay)
    pub weights: Vec<f64>,
    /// Indices of sampled experiences
    pub indices: Vec<usize>,
    /// Batch metadata
    pub metadata: Option<Value>,
}

/// Experience priority for prioritized replay (reserved for future prioritized experience replay)
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "Phase 2 / reserved for prioritized experience replay"
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperiencePriority {
    /// Experience ID
    pub experience_id: String,
    /// Priority value
    pub priority: f64,
    /// TD error (for priority calculation)
    pub td_error: f64,
    /// Last update time
    pub last_update: DateTime<Utc>,
}
