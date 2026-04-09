// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Types for the Context Learning Manager
//!
//! Configuration, domain types (episodes, sessions, observations), and reward
//! parameters extracted from `manager.rs` for clarity and module-size compliance.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::time::Duration;

use super::engine::{RLAction, RLState};

/// Context learning manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextLearningManagerConfig {
    /// Episode timeout in seconds
    pub episode_timeout: u64,

    /// Maximum episodes per session
    pub max_episodes_per_session: usize,

    /// Learning update interval
    pub learning_update_interval: Duration,

    /// Context observation interval
    pub context_observation_interval: Duration,

    /// Enable automatic episode detection
    pub auto_episode_detection: bool,

    /// Enable context state preprocessing
    pub enable_preprocessing: bool,

    /// Feature extraction method
    pub feature_extraction: FeatureExtractionMethod,

    /// State space dimensionality
    pub state_space_size: usize,

    /// Action space size
    pub action_space_size: usize,

    /// Reward calculation parameters
    pub reward_params: RewardParameters,
}

impl Default for ContextLearningManagerConfig {
    fn default() -> Self {
        Self {
            episode_timeout: 3600,
            max_episodes_per_session: 1000,
            learning_update_interval: Duration::from_secs(10),
            context_observation_interval: Duration::from_secs(1),
            auto_episode_detection: true,
            enable_preprocessing: true,
            feature_extraction: FeatureExtractionMethod::Statistical,
            state_space_size: 128,
            action_space_size: 32,
            reward_params: RewardParameters::default(),
        }
    }
}

/// Feature extraction method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeatureExtractionMethod {
    /// Simple statistical features
    Statistical,
    /// Rule-based features
    RuleBased,
    /// Context-aware features
    ContextAware,
    /// Custom feature extraction
    Custom(String),
}

/// Reward calculation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardParameters {
    /// Success reward
    pub success_reward: f64,

    /// Failure penalty
    pub failure_penalty: f64,

    /// Step penalty
    pub step_penalty: f64,

    /// Context improvement reward
    pub context_improvement_reward: f64,

    /// Rule efficiency reward
    pub rule_efficiency_reward: f64,

    /// Synchronization reward
    pub synchronization_reward: f64,

    /// Error penalty
    pub error_penalty: f64,
}

impl Default for RewardParameters {
    fn default() -> Self {
        Self {
            success_reward: 10.0,
            failure_penalty: -5.0,
            step_penalty: -0.1,
            context_improvement_reward: 5.0,
            rule_efficiency_reward: 2.0,
            synchronization_reward: 1.0,
            error_penalty: -10.0,
        }
    }
}

/// Learning episode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEpisode {
    /// Episode ID
    pub id: String,

    /// Episode start time
    pub start_time: DateTime<Utc>,

    /// Episode end time
    pub end_time: Option<DateTime<Utc>>,

    /// Context ID
    pub context_id: String,

    /// Initial state
    pub initial_state: RLState,

    /// Final state
    pub final_state: Option<RLState>,

    /// Actions taken
    pub actions: Vec<RLAction>,

    /// Rewards received
    pub rewards: Vec<f64>,

    /// Total reward
    pub total_reward: f64,

    /// Episode success
    pub success: bool,

    /// Episode metadata
    pub metadata: Option<Value>,

    /// Episode duration
    pub duration: Option<Duration>,
}

/// Context observation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextObservation {
    /// Observation ID
    pub id: String,

    /// Observation timestamp
    pub timestamp: DateTime<Utc>,

    /// Context ID
    pub context_id: String,

    /// Context state snapshot
    pub context_state: Value,

    /// Extracted features
    pub features: Vec<f64>,

    /// Rule evaluation results
    pub rule_results: Option<Value>,

    /// Performance metrics
    pub performance_metrics: Option<Value>,
}

/// Learning session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningSession {
    /// Session ID
    pub id: String,

    /// Session start time
    pub start_time: DateTime<Utc>,

    /// Session end time
    pub end_time: Option<DateTime<Utc>>,

    /// Episodes in this session
    pub episodes: Vec<String>,

    /// Total episodes
    pub total_episodes: usize,

    /// Successful episodes
    pub successful_episodes: usize,

    /// Average reward
    pub average_reward: f64,

    /// Session metadata
    pub metadata: Option<Value>,
}
