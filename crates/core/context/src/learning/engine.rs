// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Learning Engine
//!
//! This module implements the core reinforcement learning engine for the Context Management System.
//! It provides Q-learning, Deep Q-Network (DQN), and other RL algorithms for learning optimal
//! context management policies.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info};
use uuid::Uuid;

use super::{LearningState, LearningSystemConfig};
use crate::error::Result;

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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Learning engine implementation
#[derive(Debug)]
pub struct LearningEngine {
    /// Configuration
    config: Arc<LearningEngineConfig>,

    /// Q-table for tabular Q-learning
    q_table: Arc<RwLock<HashMap<String, QValue>>>,

    /// Neural network for deep learning (placeholder)
    neural_network: Arc<Mutex<Option<NeuralNetwork>>>,

    /// Experience buffer
    experience_buffer: Arc<RwLock<Vec<RLExperience>>>,

    /// Current exploration rate
    exploration_rate: Arc<RwLock<f64>>,

    /// Training step count
    training_steps: Arc<RwLock<u64>>,

    /// Last target update
    last_target_update: Arc<RwLock<u64>>,

    /// Engine state
    state: Arc<RwLock<LearningState>>,

    /// Performance metrics
    metrics: Arc<Mutex<EngineMetrics>>,
}

/// Neural network placeholder (reserved for future ML integration)
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NeuralNetwork {
    /// Network architecture
    pub layers: Vec<usize>,

    /// Weights (simplified representation)
    pub weights: Vec<Vec<f64>>,

    /// Biases
    pub biases: Vec<Vec<f64>>,

    /// Activation function
    pub activation: String,
}

/// Engine performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineMetrics {
    /// Total training steps
    pub total_steps: u64,

    /// Total episodes
    pub total_episodes: u64,

    /// Average Q-value
    pub average_q_value: f64,

    /// Loss value
    pub loss: f64,

    /// Exploration rate
    pub exploration_rate: f64,

    /// Success rate
    pub success_rate: f64,

    /// Training time
    pub training_time: f64,

    /// Last update
    pub last_update: DateTime<Utc>,
}

impl Default for EngineMetrics {
    fn default() -> Self {
        Self {
            total_steps: 0,
            total_episodes: 0,
            average_q_value: 0.0,
            loss: 0.0,
            exploration_rate: 1.0,
            success_rate: 0.0,
            training_time: 0.0,
            last_update: Utc::now(),
        }
    }
}

impl LearningEngine {
    /// Create a new learning engine
    pub async fn new(system_config: Arc<LearningSystemConfig>) -> Result<Self> {
        let config = Arc::new(LearningEngineConfig {
            algorithm: LearningAlgorithm::DeepQLearning,
            learning_rate: system_config.learning_rate,
            discount_factor: system_config.discount_factor,
            exploration_rate: system_config.exploration_rate,
            exploration_decay: 0.995,
            min_exploration_rate: 0.01,
            target_update_frequency: system_config.target_update_frequency,
            buffer_size: system_config.experience_buffer_size,
            batch_size: system_config.batch_size,
            double_dqn: true,
            dueling_dqn: true,
            prioritized_replay: true,
        });

        Ok(Self {
            config,
            q_table: Arc::new(RwLock::new(HashMap::new())),
            neural_network: Arc::new(Mutex::new(None)),
            experience_buffer: Arc::new(RwLock::new(Vec::new())),
            exploration_rate: Arc::new(RwLock::new(system_config.exploration_rate)),
            training_steps: Arc::new(RwLock::new(0)),
            last_target_update: Arc::new(RwLock::new(0)),
            state: Arc::new(RwLock::new(LearningState::Initializing)),
            metrics: Arc::new(Mutex::new(EngineMetrics::default())),
        })
    }

    /// Initialize the learning engine
    pub async fn initialize(&self) -> Result<()> {
        info!(
            "Initializing learning engine with algorithm: {:?}",
            self.config.algorithm
        );

        // Initialize neural network if using deep learning
        if matches!(
            self.config.algorithm,
            LearningAlgorithm::DeepQLearning
                | LearningAlgorithm::DoubleDQN
                | LearningAlgorithm::DuelingDQN
        ) {
            let network = NeuralNetwork {
                layers: vec![128, 256, 128, 64, 32],
                weights: vec![],
                biases: vec![],
                activation: "relu".to_string(),
            };

            *self.neural_network.lock().await = Some(network);
        }

        // Set initial exploration rate
        *self.exploration_rate.write().await = self.config.exploration_rate;

        // Update state
        *self.state.write().await = LearningState::Learning;

        info!("Learning engine initialized successfully");
        Ok(())
    }

    /// Start the learning engine
    pub async fn start(&self) -> Result<()> {
        info!("Starting learning engine");
        *self.state.write().await = LearningState::Learning;
        Ok(())
    }

    /// Stop the learning engine
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping learning engine");
        *self.state.write().await = LearningState::Stopped;
        Ok(())
    }

    /// Select action using current policy
    pub async fn select_action(&self, state: &RLState) -> Result<RLAction> {
        let exploration_rate = *self.exploration_rate.read().await;

        // Epsilon-greedy action selection
        if rand::random::<f64>() < exploration_rate {
            // Explore: random action
            self.select_random_action(state).await
        } else {
            // Exploit: best action according to current policy
            self.select_best_action(state).await
        }
    }

    /// Select random action for exploration
    async fn select_random_action(&self, state: &RLState) -> Result<RLAction> {
        let action_types = [
            "modify_context",
            "apply_rule",
            "create_snapshot",
            "trigger_sync",
            "adapt_rule",
            "update_policy",
        ];

        let action_type = action_types[rand::random::<usize>() % action_types.len()];

        Ok(RLAction {
            id: Uuid::new_v4().to_string(),
            action_type: action_type.to_string(),
            parameters: serde_json::json!({
                "context_id": state.context_id,
                "random": true,
                "exploration": true
            }),
            confidence: 0.0,
            expected_reward: 0.0,
        })
    }

    /// Select best action according to current policy
    async fn select_best_action(&self, state: &RLState) -> Result<RLAction> {
        match self.config.algorithm {
            LearningAlgorithm::QLearning => self.select_best_action_q_learning(state).await,
            LearningAlgorithm::DeepQLearning
            | LearningAlgorithm::DoubleDQN
            | LearningAlgorithm::DuelingDQN => self.select_best_action_dqn(state).await,
            _ => {
                // Default to Q-learning
                self.select_best_action_q_learning(state).await
            }
        }
    }

    /// Select best action using Q-learning
    async fn select_best_action_q_learning(&self, state: &RLState) -> Result<RLAction> {
        let q_table = self.q_table.read().await;
        let mut best_action = None;
        let mut best_q_value = f64::NEG_INFINITY;

        // Find action with highest Q-value for this state
        for (state_action, q_value) in q_table.iter() {
            if state_action.starts_with(&state.id) && q_value.value > best_q_value {
                best_q_value = q_value.value;
                best_action = Some(state_action.clone());
            }
        }

        // If no action found, return random action
        if best_action.is_none() {
            return self.select_random_action(state).await;
        }

        // Extract action from state_action string
        let best_action_str = best_action.unwrap();
        let action_parts: Vec<&str> = best_action_str.split('_').collect();
        let action_type = if action_parts.len() > 1 {
            action_parts[1..].join("_")
        } else {
            "modify_context".to_string()
        };

        Ok(RLAction {
            id: Uuid::new_v4().to_string(),
            action_type,
            parameters: serde_json::json!({
                "context_id": state.context_id,
                "q_value": best_q_value,
                "greedy": true
            }),
            confidence: (best_q_value + 1.0) / 2.0, // Normalize to 0-1
            expected_reward: best_q_value,
        })
    }

    /// Select best action using Deep Q-Network
    async fn select_best_action_dqn(&self, state: &RLState) -> Result<RLAction> {
        let network = self.neural_network.lock().await;

        if network.is_none() {
            return self.select_random_action(state).await;
        }

        // Simplified neural network forward pass
        let q_values = self.forward_pass(&state.features).await?;

        // Find action with highest Q-value
        let mut best_action_index = 0;
        let mut best_q_value = f64::NEG_INFINITY;

        for (i, &q_value) in q_values.iter().enumerate() {
            if q_value > best_q_value {
                best_q_value = q_value;
                best_action_index = i;
            }
        }

        let action_types = [
            "modify_context",
            "apply_rule",
            "create_snapshot",
            "trigger_sync",
            "adapt_rule",
            "update_policy",
        ];

        let action_type = action_types[best_action_index % action_types.len()];

        Ok(RLAction {
            id: Uuid::new_v4().to_string(),
            action_type: action_type.to_string(),
            parameters: serde_json::json!({
                "context_id": state.context_id,
                "q_value": best_q_value,
                "network": true
            }),
            confidence: (best_q_value + 1.0) / 2.0,
            expected_reward: best_q_value,
        })
    }

    /// Simple forward pass through neural network
    async fn forward_pass(&self, features: &[f64]) -> Result<Vec<f64>> {
        // Simplified neural network computation
        // In a real implementation, this would use a proper ML framework
        let mut activations = features.to_vec();

        // Simple linear transformations
        for _ in 0..3 {
            for activation in &mut activations {
                *activation = (*activation * 0.5 + 0.1).tanh();
            }
        }

        // Return Q-values for each action
        Ok(activations.into_iter().take(6).collect())
    }

    /// Update Q-values based on experience
    pub async fn update_q_values(&self, experience: &RLExperience) -> Result<()> {
        match self.config.algorithm {
            LearningAlgorithm::QLearning => self.update_q_learning(experience).await,
            LearningAlgorithm::DeepQLearning
            | LearningAlgorithm::DoubleDQN
            | LearningAlgorithm::DuelingDQN => self.update_dqn(experience).await,
            _ => self.update_q_learning(experience).await,
        }
    }

    /// Update Q-values using Q-learning algorithm
    async fn update_q_learning(&self, experience: &RLExperience) -> Result<()> {
        let state_action = format!("{}_{}", experience.state.id, experience.action.action_type);

        let mut q_table = self.q_table.write().await;

        // Get current Q-value
        let current_q = q_table.get(&state_action).map(|q| q.value).unwrap_or(0.0);

        // Calculate target Q-value
        let target_q = if experience.done {
            experience.reward
        } else if let Some(next_state) = &experience.next_state {
            // Find max Q-value for next state
            let max_next_q = q_table
                .iter()
                .filter(|(sa, _)| sa.starts_with(&next_state.id))
                .map(|(_, q)| q.value)
                .fold(f64::NEG_INFINITY, f64::max);

            experience.reward + self.config.discount_factor * max_next_q
        } else {
            experience.reward
        };

        // Update Q-value using Q-learning update rule
        let new_q = current_q + self.config.learning_rate * (target_q - current_q);

        // Get current update count before inserting
        let state_action_key = format!("{}_{}", experience.state.id, experience.action.action_type);
        let update_count = q_table
            .get(&state_action_key)
            .map(|q| q.update_count + 1)
            .unwrap_or(1);

        // Store updated Q-value
        q_table.insert(
            state_action,
            QValue {
                state_action: state_action_key,
                value: new_q,
                confidence: 1.0,
                update_count,
                last_update: Utc::now(),
            },
        );

        debug!(
            "Updated Q-value for {}: {} -> {}",
            format!("{}_{}", experience.state.id, experience.action.action_type),
            current_q,
            new_q
        );

        Ok(())
    }

    /// Update neural network using Deep Q-Network algorithm
    async fn update_dqn(&self, experience: &RLExperience) -> Result<()> {
        // Add experience to buffer
        {
            let mut buffer = self.experience_buffer.write().await;
            buffer.push(experience.clone());

            // Keep buffer size under limit
            if buffer.len() > self.config.buffer_size {
                buffer.remove(0);
            }
        }

        // Update training step count
        let mut steps = self.training_steps.write().await;
        *steps += 1;

        // Train if we have enough experiences
        if *steps >= self.config.batch_size as u64 {
            self.train_network().await?;
        }

        // Update target network if needed
        if *steps % self.config.target_update_frequency == 0 {
            self.update_target_network().await?;
        }

        Ok(())
    }

    /// Train the neural network
    async fn train_network(&self) -> Result<()> {
        let buffer = self.experience_buffer.read().await;

        if buffer.len() < self.config.batch_size {
            return Ok(());
        }

        // Sample random batch from experience buffer
        let batch: Vec<_> = (0..self.config.batch_size)
            .map(|_| {
                let idx = rand::random::<usize>() % buffer.len();
                buffer[idx].clone()
            })
            .collect();

        // Simplified training step
        // In a real implementation, this would use backpropagation
        debug!("Training network with batch size: {}", batch.len());

        // Update metrics
        let mut metrics = self.metrics.lock().await;
        metrics.total_steps += 1;
        metrics.training_time += 0.1; // Simplified
        metrics.last_update = Utc::now();

        Ok(())
    }

    /// Update target network
    async fn update_target_network(&self) -> Result<()> {
        let mut last_update = self.last_target_update.write().await;
        *last_update = *self.training_steps.read().await;

        debug!("Updated target network at step: {}", *last_update);
        Ok(())
    }

    /// Decay exploration rate
    pub async fn decay_exploration(&self) -> Result<()> {
        let mut exploration_rate = self.exploration_rate.write().await;
        *exploration_rate *= self.config.exploration_decay;

        if *exploration_rate < self.config.min_exploration_rate {
            *exploration_rate = self.config.min_exploration_rate;
        }

        // Update metrics
        let mut metrics = self.metrics.lock().await;
        metrics.exploration_rate = *exploration_rate;

        Ok(())
    }

    /// Add experience to the buffer
    pub async fn add_experience(&self, experience: RLExperience) -> Result<()> {
        let mut buffer = self.experience_buffer.write().await;
        buffer.push(experience);

        // Keep buffer size under limit
        if buffer.len() > self.config.buffer_size {
            buffer.remove(0);
        }

        Ok(())
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> EngineMetrics {
        self.metrics.lock().await.clone()
    }

    /// Get Q-table size
    pub async fn get_q_table_size(&self) -> usize {
        self.q_table.read().await.len()
    }

    /// Get experience buffer size
    pub async fn get_experience_buffer_size(&self) -> usize {
        self.experience_buffer.read().await.len()
    }

    /// Get current exploration rate
    pub async fn get_exploration_rate(&self) -> f64 {
        *self.exploration_rate.read().await
    }

    /// Get current state
    pub async fn get_state(&self) -> LearningState {
        self.state.read().await.clone()
    }
}
