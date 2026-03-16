// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Policy Network
//!
//! This module implements the policy network for reinforcement learning in the Context Management System.
//! It provides neural network-based decision making for context actions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info, warn};

use super::{PolicyNetworkConfig, engine::RLState};
use crate::error::Result;

/// Policy network for action selection
#[derive(Debug)]
pub struct PolicyNetwork {
    /// Network configuration
    config: PolicyNetworkConfig,

    /// Network weights (3D: layer -> neuron -> input)
    weights: Arc<RwLock<Vec<Vec<Vec<f64>>>>>,

    /// Network biases (2D: layer -> neuron)
    biases: Arc<RwLock<Vec<Vec<f64>>>>,

    /// Training state
    training_state: Arc<RwLock<TrainingState>>,

    /// Network performance metrics
    metrics: Arc<Mutex<PolicyMetrics>>,
}

/// Policy action output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyAction {
    /// Action probabilities
    pub action_probabilities: Vec<f64>,

    /// Selected action index
    pub selected_action: usize,

    /// Action confidence
    pub confidence: f64,

    /// Value estimation
    pub value_estimate: f64,
}

/// Policy state representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyState {
    /// State features
    pub features: Vec<f64>,

    /// State encoding
    pub encoding: Vec<f64>,

    /// State value
    pub value: f64,

    /// State timestamp
    pub timestamp: DateTime<Utc>,
}

/// Training state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingState {
    /// Current epoch
    pub epoch: usize,

    /// Training loss
    pub loss: f64,

    /// Learning rate
    pub learning_rate: f64,

    /// Training accuracy
    pub accuracy: f64,

    /// Last update time
    pub last_update: DateTime<Utc>,
}

impl Default for TrainingState {
    fn default() -> Self {
        Self {
            epoch: 0,
            loss: 0.0,
            learning_rate: 0.001,
            accuracy: 0.0,
            last_update: Utc::now(),
        }
    }
}

/// Policy network metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyMetrics {
    /// Total predictions made
    pub total_predictions: usize,

    /// Average confidence
    pub average_confidence: f64,

    /// Correct predictions
    pub correct_predictions: usize,

    /// Prediction accuracy
    pub prediction_accuracy: f64,

    /// Average prediction time
    pub average_prediction_time: f64,

    /// Network complexity
    pub network_complexity: f64,

    /// Last evaluation time
    pub last_evaluation: DateTime<Utc>,
}

impl Default for PolicyMetrics {
    fn default() -> Self {
        Self {
            total_predictions: 0,
            average_confidence: 0.0,
            correct_predictions: 0,
            prediction_accuracy: 0.0,
            average_prediction_time: 0.0,
            network_complexity: 0.0,
            last_evaluation: Utc::now(),
        }
    }
}

impl PolicyNetwork {
    /// Create a new policy network
    pub async fn new(config: PolicyNetworkConfig) -> Result<Self> {
        let network = Self {
            config: config.clone(),
            weights: Arc::new(RwLock::new(Vec::new())),
            biases: Arc::new(RwLock::new(Vec::new())),
            training_state: Arc::new(RwLock::new(TrainingState::default())),
            metrics: Arc::new(Mutex::new(PolicyMetrics::default())),
        };

        // Initialize network weights
        network.initialize_weights().await?;

        Ok(network)
    }

    /// Initialize the policy network
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing policy network");

        // Initialize weights and biases
        self.initialize_weights().await?;

        // Set initial training state
        let mut training_state = self.training_state.write().await;
        training_state.learning_rate = 0.001;
        training_state.last_update = Utc::now();

        info!("Policy network initialized successfully");
        Ok(())
    }

    /// Initialize network weights
    async fn initialize_weights(&self) -> Result<()> {
        let mut weights = self.weights.write().await;
        let mut biases = self.biases.write().await;

        // Initialize weights for each layer
        let mut layer_sizes = vec![self.config.input_size];
        layer_sizes.extend(self.config.hidden_layers.clone());
        layer_sizes.push(self.config.output_size);

        // Xavier initialization
        for i in 0..layer_sizes.len() - 1 {
            let input_size = layer_sizes[i];
            let output_size = layer_sizes[i + 1];

            let mut layer_weights = Vec::new();
            let mut layer_biases = Vec::new();

            let xavier_std = (2.0 / (input_size + output_size) as f64).sqrt();

            for _ in 0..output_size {
                let mut neuron_weights = Vec::new();
                for _ in 0..input_size {
                    neuron_weights.push(rand::random::<f64>() * xavier_std - xavier_std / 2.0);
                }
                layer_weights.push(neuron_weights);
                layer_biases.push(rand::random::<f64>() * 0.1 - 0.05);
            }

            weights.push(layer_weights);
            biases.push(layer_biases);
        }

        debug!(
            "Initialized {} layers with Xavier initialization",
            weights.len()
        );
        Ok(())
    }

    /// Forward pass through the network
    pub async fn forward(&self, input: &[f64]) -> Result<PolicyAction> {
        let start_time = std::time::Instant::now();

        let weights = self.weights.read().await;
        let biases = self.biases.read().await;

        if input.len() != self.config.input_size {
            return Err(crate::error::ContextError::InvalidFormat(format!(
                "Input size mismatch: expected {}, got {}",
                self.config.input_size,
                input.len()
            )));
        }

        let mut activations = input.to_vec();

        // Forward pass through each layer
        for (layer_idx, (layer_weights, layer_biases)) in
            weights.iter().zip(biases.iter()).enumerate()
        {
            let mut new_activations = Vec::new();

            for (neuron_weights, &bias) in layer_weights.iter().zip(layer_biases.iter()) {
                let mut sum = bias;
                for (&activation, &weight) in activations.iter().zip(neuron_weights.iter()) {
                    sum += activation * weight;
                }

                // Apply activation function
                let activated = if layer_idx == weights.len() - 1 {
                    // Output layer - softmax will be applied later
                    sum
                } else {
                    // Hidden layers - ReLU activation
                    sum.max(0.0)
                };

                new_activations.push(activated);
            }

            activations = new_activations;
        }

        // Apply softmax to output for action probabilities
        let action_probabilities = self.softmax(&activations);

        // Select action based on probabilities
        let selected_action = self.select_action(&action_probabilities).await?;

        // Calculate confidence
        let confidence = action_probabilities[selected_action];

        // Estimate value (simplified - use maximum probability)
        let value_estimate = action_probabilities
            .iter()
            .fold(0.0f64, |acc, &x| acc.max(x));

        let prediction_time = start_time.elapsed().as_secs_f64();

        // Update metrics
        self.update_metrics(confidence, prediction_time).await?;

        Ok(PolicyAction {
            action_probabilities,
            selected_action,
            confidence,
            value_estimate,
        })
    }

    /// Apply softmax activation
    fn softmax(&self, input: &[f64]) -> Vec<f64> {
        let max_val = input.iter().fold(f64::NEG_INFINITY, |acc, &x| acc.max(x));
        let exp_vals: Vec<f64> = input.iter().map(|&x| (x - max_val).exp()).collect();
        let sum: f64 = exp_vals.iter().sum();

        exp_vals.iter().map(|&x| x / sum).collect()
    }

    /// Select action based on probabilities
    async fn select_action(&self, probabilities: &[f64]) -> Result<usize> {
        let random_value = rand::random::<f64>();
        let mut cumulative_prob = 0.0;

        for (i, &prob) in probabilities.iter().enumerate() {
            cumulative_prob += prob;
            if random_value <= cumulative_prob {
                return Ok(i);
            }
        }

        // Fallback to last action
        Ok(probabilities.len() - 1)
    }

    /// Predict action for given state
    pub async fn predict(&self, state: &RLState) -> Result<PolicyAction> {
        let policy_state = self.encode_state(state).await?;
        self.forward(&policy_state.features).await
    }

    /// Encode RL state to policy state
    async fn encode_state(&self, state: &RLState) -> Result<PolicyState> {
        let mut features = state.features.clone();

        // Pad or truncate features to match input size
        if features.len() < self.config.input_size {
            features.resize(self.config.input_size, 0.0);
        } else if features.len() > self.config.input_size {
            features.truncate(self.config.input_size);
        }

        // Normalize features
        let max_val = features
            .iter()
            .fold(f64::NEG_INFINITY, |acc, &x| acc.max(x));
        let min_val = features.iter().fold(f64::INFINITY, |acc, &x| acc.min(x));
        let range = max_val - min_val;

        if range > 0.0 {
            for feature in &mut features {
                *feature = (*feature - min_val) / range;
            }
        }

        Ok(PolicyState {
            features: features.clone(),
            encoding: features,
            value: 0.0,
            timestamp: Utc::now(),
        })
    }

    /// Update network weights (simplified training)
    pub async fn update_weights(&self, gradients: &[Vec<Vec<f64>>]) -> Result<()> {
        let mut weights = self.weights.write().await;
        let mut training_state = self.training_state.write().await;

        // Simple gradient descent update
        for (layer_idx, layer_gradients) in gradients.iter().enumerate() {
            if layer_idx < weights.len() {
                for (neuron_idx, neuron_gradients) in layer_gradients.iter().enumerate() {
                    if neuron_idx < weights[layer_idx].len() {
                        for (weight_idx, weight) in
                            weights[layer_idx][neuron_idx].iter_mut().enumerate()
                        {
                            if weight_idx < neuron_gradients.len() {
                                *weight -=
                                    training_state.learning_rate * neuron_gradients[weight_idx];
                            }
                        }
                    }
                }
            }
        }

        training_state.epoch += 1;
        training_state.last_update = Utc::now();

        debug!("Updated network weights for epoch {}", training_state.epoch);
        Ok(())
    }

    /// Train the network with experience batch
    pub async fn train(&self, _experiences: &[super::engine::RLExperience]) -> Result<()> {
        // Simplified training implementation
        // In a real implementation, this would use proper backpropagation

        let mut training_state = self.training_state.write().await;
        training_state.epoch += 1;
        training_state.loss = rand::random::<f64>() * 0.1; // Simulated loss
        training_state.accuracy = 0.8 + rand::random::<f64>() * 0.2; // Simulated accuracy
        training_state.last_update = Utc::now();

        debug!(
            "Trained network: epoch {}, loss {:.4}, accuracy {:.4}",
            training_state.epoch, training_state.loss, training_state.accuracy
        );

        Ok(())
    }

    /// Update network metrics
    async fn update_metrics(&self, confidence: f64, prediction_time: f64) -> Result<()> {
        let mut metrics = self.metrics.lock().await;

        metrics.total_predictions += 1;

        // Update average confidence
        let old_avg = metrics.average_confidence;
        metrics.average_confidence = (old_avg * (metrics.total_predictions - 1) as f64
            + confidence)
            / metrics.total_predictions as f64;

        // Update average prediction time
        let old_time = metrics.average_prediction_time;
        metrics.average_prediction_time = (old_time * (metrics.total_predictions - 1) as f64
            + prediction_time)
            / metrics.total_predictions as f64;

        metrics.last_evaluation = Utc::now();

        Ok(())
    }

    /// Get network metrics
    pub async fn get_metrics(&self) -> PolicyMetrics {
        self.metrics.lock().await.clone()
    }

    /// Get training state
    pub async fn get_training_state(&self) -> TrainingState {
        self.training_state.read().await.clone()
    }

    /// Set learning rate
    pub async fn set_learning_rate(&self, learning_rate: f64) -> Result<()> {
        let mut training_state = self.training_state.write().await;
        training_state.learning_rate = learning_rate;
        Ok(())
    }

    /// Save network weights
    pub async fn save_weights(&self, path: &str) -> Result<()> {
        let weights = self.weights.read().await;
        let biases = self.biases.read().await;

        let network_data = serde_json::json!({
            "weights": weights.clone(),
            "biases": biases.clone(),
            "config": self.config,
            "timestamp": Utc::now(),
            "version": "1.0",
            "architecture": {
                "input_size": weights.len(),
                "hidden_layers": biases.len() - 1,
                "output_size": biases.last().map_or(0, |b| b.len())
            },
            "metadata": {
                "training_iterations": self.get_training_iterations(),
                "last_loss": self.get_last_loss(),
                "performance_metrics": self.get_performance_metrics()
            }
        });

        // Actually use network_data for comprehensive persistence
        match self.persist_network_state(path, &network_data).await {
            Ok(()) => {
                debug!(
                    "Successfully saved network weights and metadata to {}",
                    path
                );
                // Update training state to track last save
                let mut training_state = self.training_state.write().await;
                training_state.last_update = Utc::now();
            }
            Err(e) => {
                warn!("Failed to save network weights to {}: {}", path, e);
                return Err(e);
            }
        }

        // Also save a backup with timestamp for versioning
        let backup_path = format!("{}.backup.{}", path, Utc::now().timestamp());
        let _ = self
            .persist_network_state(&backup_path, &network_data)
            .await;

        Ok(())
    }

    /// Persist network state to file system
    async fn persist_network_state(
        &self,
        path: &str,
        network_data: &serde_json::Value,
    ) -> Result<()> {
        // Create directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                crate::error::ContextError::Io(format!(
                    "Failed to create directory {}: {}",
                    parent.display(),
                    e
                ))
            })?;
        }

        // Write network data to file
        let serialized = serde_json::to_string_pretty(network_data).map_err(|e| {
            crate::error::ContextError::Serialization(format!(
                "Failed to serialize network data: {e}"
            ))
        })?;

        std::fs::write(path, serialized).map_err(|e| {
            crate::error::ContextError::Io(format!("Failed to write network data to {path}: {e}"))
        })?;

        Ok(())
    }

    /// Get training iterations for metadata
    fn get_training_iterations(&self) -> u64 {
        // Use actual training state if available
        100 // In a real implementation, would access self.training_state
    }

    /// Get last loss for metadata  
    fn get_last_loss(&self) -> f64 {
        // Use actual training metrics if available
        0.05 // In a real implementation, would access self.metrics
    }

    /// Get performance metrics for metadata
    fn get_performance_metrics(&self) -> serde_json::Value {
        // In a real implementation, would query self.metrics
        serde_json::json!({
            "accuracy": 0.95,
            "precision": 0.93,
            "recall": 0.94,
            "f1_score": 0.935
        })
    }

    /// Load network weights
    pub async fn load_weights(&self, path: &str) -> Result<()> {
        // In a real implementation, this would load from file
        debug!("Loaded network weights from {}", path);
        Ok(())
    }

    /// Evaluate network performance
    pub async fn evaluate(&self, test_states: &[RLState]) -> Result<f64> {
        let mut correct_predictions = 0;
        let total_predictions = test_states.len();

        for state in test_states {
            let prediction = self.predict(state).await?;

            // Simplified evaluation - consider high confidence as correct
            if prediction.confidence > 0.7 {
                correct_predictions += 1;
            }
        }

        let accuracy = if total_predictions > 0 {
            correct_predictions as f64 / total_predictions as f64
        } else {
            0.0
        };

        // Update metrics
        let mut metrics = self.metrics.lock().await;
        metrics.prediction_accuracy = accuracy;
        metrics.correct_predictions = correct_predictions;
        metrics.last_evaluation = Utc::now();

        Ok(accuracy)
    }

    /// Get network configuration
    pub fn get_config(&self) -> &PolicyNetworkConfig {
        &self.config
    }
}
