// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! RL domain types for the learning engine.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Learning engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEngineConfig {
    /// Algorithm type
    pub algorithm: LearningAlgorithm,

    /// Learning rate
    pub learning_rate: f64,

    /// Discount factor
    pub discount_factor: f64,

    /// Exploration rate
    pub exploration_rate: f64,

    /// Exploration decay rate
    pub exploration_decay: f64,

    /// Minimum exploration rate
    pub min_exploration_rate: f64,

    /// Target network update frequency
    pub target_update_frequency: u64,

    /// Experience replay buffer size
    pub buffer_size: usize,

    /// Batch size for training
    pub batch_size: usize,

    /// Enable double DQN
    pub double_dqn: bool,

    /// Enable dueling DQN
    pub dueling_dqn: bool,

    /// Enable prioritized experience replay
    pub prioritized_replay: bool,
}

impl Default for LearningEngineConfig {
    fn default() -> Self {
        Self {
            algorithm: LearningAlgorithm::DeepQLearning,
            learning_rate: 0.001,
            discount_factor: 0.95,
            exploration_rate: 1.0,
            exploration_decay: 0.995,
            min_exploration_rate: 0.01,
            target_update_frequency: 1000,
            buffer_size: 10000,
            batch_size: 32,
            double_dqn: true,
            dueling_dqn: true,
            prioritized_replay: true,
        }
    }
}

/// Learning algorithm type
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LearningAlgorithm {
    /// Q-Learning
    QLearning,
    /// Deep Q-Network
    DeepQLearning,
    /// Double DQN
    DoubleDQN,
    /// Dueling DQN
    DuelingDQN,
    /// Actor-Critic
    ActorCritic,
    /// Proximal Policy Optimization
    Ppo,
    /// Soft Actor-Critic
    Sac,
}

/// State representation for RL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLState {
    /// State ID
    pub id: String,

    /// State features
    pub features: Vec<f64>,

    /// Context ID
    pub context_id: String,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// State metadata
    pub metadata: Option<Value>,
}

/// Action representation for RL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLAction {
    /// Action ID
    pub id: String,

    /// Action type
    pub action_type: String,

    /// Action parameters
    pub parameters: Value,

    /// Action confidence
    pub confidence: f64,

    /// Expected reward
    pub expected_reward: f64,
}

/// Experience for reinforcement learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLExperience {
    /// Experience ID
    pub id: String,

    /// Current state
    pub state: RLState,

    /// Action taken
    pub action: RLAction,

    /// Reward received
    pub reward: f64,

    /// Next state
    pub next_state: Option<RLState>,

    /// Whether episode is done
    pub done: bool,

    /// Experience timestamp
    pub timestamp: DateTime<Utc>,

    /// Priority for prioritized replay
    pub priority: f64,
}

/// Q-value estimation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QValue {
    /// State-action pair
    pub state_action: String,

    /// Q-value
    pub value: f64,

    /// Confidence
    pub confidence: f64,

    /// Update count
    pub update_count: u64,

    /// Last update time
    pub last_update: DateTime<Utc>,
}
