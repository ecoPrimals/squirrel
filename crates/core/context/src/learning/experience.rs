//! Experience Replay System
//!
//! This module implements the Experience Replay system for the Context Learning System.
//! It provides efficient storage, sampling, and management of learning experiences
//! for reinforcement learning algorithms.

use chrono::{DateTime, Utc};
use rand::prelude::*;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};


use super::engine::RLExperience;
use crate::error::Result;

/// Experience replay buffer
#[derive(Debug)]
pub struct ExperienceReplay {
    /// Buffer for storing experiences
    buffer: Arc<RwLock<ExperienceBuffer>>,

    /// Maximum buffer size
    max_size: usize,

    /// Sampling strategy
    sampling_strategy: SamplingStrategy,

    /// Buffer statistics
    stats: Arc<RwLock<ExperienceStats>>,
}

/// Experience buffer implementation
#[derive(Debug, Clone)]
pub struct ExperienceBuffer {
    /// Circular buffer for experiences
    experiences: VecDeque<RLExperience>,

    /// Current position in buffer
    position: usize,

    /// Number of experiences stored
    size: usize,

    /// Maximum capacity
    capacity: usize,
}

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

/// Experience priority for prioritized replay
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

impl ExperienceBuffer {
    /// Create a new experience buffer
    pub fn new(capacity: usize) -> Self {
        Self {
            experiences: VecDeque::with_capacity(capacity),
            position: 0,
            size: 0,
            capacity,
        }
    }

    /// Add an experience to the buffer
    pub fn add(&mut self, experience: RLExperience) {
        if self.size < self.capacity {
            self.experiences.push_back(experience);
            self.size += 1;
        } else {
            // Replace oldest experience
            self.experiences[self.position] = experience;
            self.position = (self.position + 1) % self.capacity;
        }
    }

    /// Get experience by index
    pub fn get(&self, index: usize) -> Option<&RLExperience> {
        self.experiences.get(index)
    }

    /// Get all experiences
    pub fn get_all(&self) -> Vec<RLExperience> {
        self.experiences.iter().cloned().collect()
    }

    /// Get buffer size
    pub fn size(&self) -> usize {
        self.size
    }

    /// Get buffer capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Check if buffer is full
    pub fn is_full(&self) -> bool {
        self.size >= self.capacity
    }

    /// Sample random experiences
    pub fn sample_uniform(&self, batch_size: usize) -> Vec<RLExperience> {
        let mut rng = thread_rng();
        let experiences: Vec<_> = self.experiences.iter().cloned().collect();

        experiences
            .choose_multiple(&mut rng, batch_size.min(self.size))
            .cloned()
            .collect()
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.experiences.clear();
        self.position = 0;
        self.size = 0;
    }
}

impl ExperienceReplay {
    /// Create a new experience replay system
    pub fn new(max_size: usize) -> Self {
        Self {
            buffer: Arc::new(RwLock::new(ExperienceBuffer::new(max_size))),
            max_size,
            sampling_strategy: SamplingStrategy::Uniform,
            stats: Arc::new(RwLock::new(ExperienceStats::default())),
        }
    }

    /// Create with custom sampling strategy
    pub fn with_sampling_strategy(max_size: usize, strategy: SamplingStrategy) -> Self {
        Self {
            buffer: Arc::new(RwLock::new(ExperienceBuffer::new(max_size))),
            max_size,
            sampling_strategy: strategy,
            stats: Arc::new(RwLock::new(ExperienceStats::default())),
        }
    }

    /// Add an experience to the replay buffer
    pub async fn add_experience(&self, experience: RLExperience) -> Result<()> {
        let mut buffer = self.buffer.write().await;
        buffer.add(experience.clone());

        // Update statistics
        self.update_stats().await?;

        debug!("Added experience to replay buffer: {}", experience.id);
        Ok(())
    }

    /// Add multiple experiences
    pub async fn add_experiences(&self, experiences: Vec<RLExperience>) -> Result<()> {
        let mut buffer = self.buffer.write().await;

        for experience in experiences {
            buffer.add(experience.clone());
        }

        // Update statistics
        self.update_stats().await?;

        debug!("Added {} experiences to replay buffer", buffer.size());
        Ok(())
    }

    /// Sample a batch of experiences
    pub async fn sample_batch(&self, batch_size: usize) -> Result<ExperienceBatch> {
        let buffer = self.buffer.read().await;

        if buffer.is_empty() {
            return Ok(ExperienceBatch {
                experiences: Vec::new(),
                weights: Vec::new(),
                indices: Vec::new(),
                metadata: None,
            });
        }

        let effective_batch_size = batch_size.min(buffer.size());

        match &self.sampling_strategy {
            SamplingStrategy::Uniform => {
                let experiences = buffer.sample_uniform(effective_batch_size);
                Ok(ExperienceBatch {
                    experiences,
                    weights: vec![1.0; effective_batch_size],
                    indices: (0..effective_batch_size).collect(),
                    metadata: Some(serde_json::json!({
                        "strategy": "uniform",
                        "batch_size": effective_batch_size
                    })),
                })
            }
            SamplingStrategy::Prioritized(config) => {
                self.sample_prioritized(&buffer, effective_batch_size, config)
                    .await
            }
            SamplingStrategy::Temporal(config) => {
                self.sample_temporal(&buffer, effective_batch_size, config)
                    .await
            }
            SamplingStrategy::Balanced(config) => {
                self.sample_balanced(&buffer, effective_batch_size, config)
                    .await
            }
        }
    }

    /// Sample using prioritized experience replay
    async fn sample_prioritized(
        &self,
        buffer: &ExperienceBuffer,
        batch_size: usize,
        config: &PrioritizedConfig,
    ) -> Result<ExperienceBatch> {
        let experiences = buffer.get_all();

        // Calculate priorities
        let priorities: Vec<f64> = experiences
            .iter()
            .map(|exp| (exp.priority + config.epsilon).powf(config.alpha))
            .collect();

        let total_priority: f64 = priorities.iter().sum();

        // Sample experiences based on priorities
        let mut sampled_experiences = Vec::new();
        let mut sampled_indices = Vec::new();
        let mut weights = Vec::new();
        let mut rng = thread_rng();

        for _ in 0..batch_size {
            let mut cumulative_prob = 0.0;
            let random_prob = rng.gen::<f64>() * total_priority;

            for (i, &priority) in priorities.iter().enumerate() {
                cumulative_prob += priority;
                if cumulative_prob >= random_prob {
                    sampled_experiences.push(experiences[i].clone());
                    sampled_indices.push(i);

                    // Calculate importance sampling weight
                    let prob = priority / total_priority;
                    let weight = (buffer.size() as f64 * prob).powf(-config.beta);
                    weights.push(weight);
                    break;
                }
            }
        }

        // Normalize weights
        let max_weight = weights.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        if max_weight > 0.0 {
            for weight in &mut weights {
                *weight /= max_weight;
            }
        }

        Ok(ExperienceBatch {
            experiences: sampled_experiences,
            weights,
            indices: sampled_indices,
            metadata: Some(serde_json::json!({
                "strategy": "prioritized",
                "alpha": config.alpha,
                "beta": config.beta,
                "batch_size": batch_size
            })),
        })
    }

    /// Sample using temporal strategy
    async fn sample_temporal(
        &self,
        buffer: &ExperienceBuffer,
        batch_size: usize,
        config: &TemporalConfig,
    ) -> Result<ExperienceBatch> {
        let experiences = buffer.get_all();
        let now = Utc::now();

        // Calculate temporal weights
        let weights: Vec<f64> = experiences
            .iter()
            .map(|exp| {
                let age = (now - exp.timestamp).num_seconds() as f64;
                let weight = config.decay_factor.powf(age / 3600.0); // Decay per hour
                weight.max(config.min_probability)
            })
            .collect();

        // Sample based on temporal weights
        let mut sampled_experiences = Vec::new();
        let mut sampled_indices = Vec::new();
        let mut rng = thread_rng();

        for _ in 0..batch_size {
            let weighted_choices: Vec<_> =
                weights.iter().enumerate().map(|(i, &w)| (i, w)).collect();

            if let Some((idx, _)) = weighted_choices
                .choose_weighted(&mut rng, |item| item.1)
                .ok()
            {
                sampled_experiences.push(experiences[*idx].clone());
                sampled_indices.push(*idx);
            }
        }

        let batch_size = sampled_experiences.len();
        Ok(ExperienceBatch {
            experiences: sampled_experiences,
            weights: vec![1.0; batch_size],
            indices: sampled_indices,
            metadata: Some(serde_json::json!({
                "strategy": "temporal",
                "decay_factor": config.decay_factor,
                "batch_size": batch_size
            })),
        })
    }

    /// Sample using balanced strategy
    async fn sample_balanced(
        &self,
        buffer: &ExperienceBuffer,
        batch_size: usize,
        config: &BalancedConfig,
    ) -> Result<ExperienceBatch> {
        let experiences = buffer.get_all();
        let recent_count = (batch_size as f64 * config.recent_ratio) as usize;
        let old_count = batch_size - recent_count;

        // Split experiences into recent and old
        let threshold_idx = (experiences.len() as f64 * config.recent_threshold) as usize;
        let recent_experiences = &experiences[threshold_idx..];
        let old_experiences = &experiences[..threshold_idx];

        let mut sampled_experiences = Vec::new();
        let mut sampled_indices = Vec::new();
        let mut rng = thread_rng();

        // Sample recent experiences
        let recent_sample = recent_experiences.choose_multiple(&mut rng, recent_count);
        for exp in recent_sample {
            sampled_experiences.push(exp.clone());
            // Find index in original array
            if let Some(idx) = experiences.iter().position(|e| e.id == exp.id) {
                sampled_indices.push(idx);
            }
        }

        // Sample old experiences
        let old_sample = old_experiences.choose_multiple(&mut rng, old_count);
        for exp in old_sample {
            sampled_experiences.push(exp.clone());
            // Find index in original array
            if let Some(idx) = experiences.iter().position(|e| e.id == exp.id) {
                sampled_indices.push(idx);
            }
        }

        let batch_size = sampled_experiences.len();
        Ok(ExperienceBatch {
            experiences: sampled_experiences,
            weights: vec![1.0; batch_size],
            indices: sampled_indices,
            metadata: Some(serde_json::json!({
                "strategy": "balanced",
                "recent_ratio": config.recent_ratio,
                "batch_size": batch_size
            })),
        })
    }

    /// Update experience priorities (for prioritized replay)
    pub async fn update_priorities(&self, indices: Vec<usize>, priorities: Vec<f64>) -> Result<()> {
        let mut buffer = self.buffer.write().await;

        for (idx, priority) in indices.into_iter().zip(priorities) {
            if let Some(experience) = buffer.experiences.get_mut(idx) {
                experience.priority = priority;
            }
        }

        Ok(())
    }

    /// Update statistics
    async fn update_stats(&self) -> Result<()> {
        let buffer = self.buffer.read().await;
        let mut stats = self.stats.write().await;

        stats.current_size = buffer.size();
        stats.utilization = buffer.size() as f64 / buffer.capacity() as f64;

        if !buffer.is_empty() {
            let experiences = buffer.get_all();

            // Calculate average reward
            stats.average_reward =
                experiences.iter().map(|exp| exp.reward).sum::<f64>() / experiences.len() as f64;

            // Calculate success rate
            stats.success_rate = experiences.iter().filter(|exp| exp.reward > 0.0).count() as f64
                / experiences.len() as f64;

            // Calculate average priority
            stats.average_priority =
                experiences.iter().map(|exp| exp.priority).sum::<f64>() / experiences.len() as f64;

            // Calculate oldest experience age
            let now = Utc::now();
            if let Some(oldest) = experiences.iter().min_by_key(|exp| exp.timestamp) {
                stats.oldest_experience_age = (now - oldest.timestamp).num_seconds() as f64;
            }
        }

        stats.last_update = Utc::now();
        Ok(())
    }

    /// Get buffer statistics
    pub async fn get_stats(&self) -> ExperienceStats {
        self.stats.read().await.clone()
    }

    /// Get buffer size
    pub async fn size(&self) -> usize {
        self.buffer.read().await.size()
    }

    /// Get buffer capacity
    pub fn capacity(&self) -> usize {
        self.max_size
    }

    /// Check if buffer is empty
    pub async fn is_empty(&self) -> bool {
        self.buffer.read().await.is_empty()
    }

    /// Check if buffer is full
    pub async fn is_full(&self) -> bool {
        self.buffer.read().await.is_full()
    }

    /// Clear the buffer
    pub async fn clear(&self) -> Result<()> {
        let mut buffer = self.buffer.write().await;
        buffer.clear();

        // Reset statistics
        let mut stats = self.stats.write().await;
        *stats = ExperienceStats::default();

        info!("Experience replay buffer cleared");
        Ok(())
    }

    /// Get all experiences (for analysis)
    pub async fn get_all_experiences(&self) -> Vec<RLExperience> {
        self.buffer.read().await.get_all()
    }

    /// Set sampling strategy
    pub fn set_sampling_strategy(&mut self, strategy: SamplingStrategy) {
        self.sampling_strategy = strategy;
    }

    /// Get sampling strategy
    pub fn get_sampling_strategy(&self) -> &SamplingStrategy {
        &self.sampling_strategy
    }
}

/// Experience trait for type-safe experience handling
pub trait Experience: Send + Sync {
    fn id(&self) -> &str;
    fn timestamp(&self) -> DateTime<Utc>;
    fn priority(&self) -> f64;
    fn set_priority(&mut self, priority: f64);
}

impl Experience for RLExperience {
    fn id(&self) -> &str {
        &self.id
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn priority(&self) -> f64 {
        self.priority
    }

    fn set_priority(&mut self, priority: f64) {
        self.priority = priority;
    }
}
