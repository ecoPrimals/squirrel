// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Experience Replay System
//!
//! This module implements the Experience Replay system for the Context Learning System.
//! It provides efficient storage, sampling, and management of learning experiences
//! for reinforcement learning algorithms.

use chrono::{DateTime, Utc};
use rand::prelude::*;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use super::engine::RLExperience;
pub use super::experience_types::*;
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
        let mut rng = rand::rng();
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
        let id = experience.id.clone();
        {
            let mut buffer = self.buffer.write().await;
            buffer.add(experience);
        }

        self.update_stats().await?;

        debug!("Added experience to replay buffer: {}", id);
        Ok(())
    }

    /// Add multiple experiences
    pub async fn add_experiences(&self, experiences: Vec<RLExperience>) -> Result<()> {
        let count = {
            let mut buffer = self.buffer.write().await;
            for experience in experiences {
                buffer.add(experience);
            }
            buffer.size()
        };

        self.update_stats().await?;

        debug!("Added experiences to replay buffer (size: {})", count);
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
        let mut rng = rand::rng();

        for _ in 0..batch_size {
            let mut cumulative_prob = 0.0;
            let random_prob = rng.random::<f64>() * total_priority;

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
        let mut rng = rand::rng();

        for _ in 0..batch_size {
            let weighted_choices: Vec<_> =
                weights.iter().enumerate().map(|(i, &w)| (i, w)).collect();

            if let Ok((idx, _)) = weighted_choices.choose_weighted(&mut rng, |item| item.1) {
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
        let mut rng = rand::rng();

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
    /// Get the experience ID
    fn id(&self) -> &str;
    /// Get the experience timestamp
    fn timestamp(&self) -> DateTime<Utc>;
    /// Get the experience priority
    fn priority(&self) -> f64;
    /// Set the experience priority
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

#[cfg(test)]
mod tests {
    use super::super::engine::{RLAction, RLExperience, RLState};
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn sample_exp(id: &str, reward: f64, priority: f64) -> RLExperience {
        let ts = Utc::now();
        RLExperience {
            id: id.to_string(),
            state: RLState {
                id: Uuid::new_v4().to_string(),
                context_id: "ctx".to_string(),
                features: vec![1.0, 2.0, 3.0],
                metadata: None,
                timestamp: ts,
            },
            action: RLAction {
                id: Uuid::new_v4().to_string(),
                action_type: "act".to_string(),
                parameters: serde_json::Value::Null,
                confidence: 0.8,
                expected_reward: 1.0,
            },
            reward,
            next_state: None,
            done: false,
            timestamp: ts,
            priority,
        }
    }

    #[test]
    fn sampling_strategy_serde_roundtrip_all_variants() {
        let strategies = vec![
            SamplingStrategy::Uniform,
            SamplingStrategy::Prioritized(PrioritizedConfig::default()),
            SamplingStrategy::Temporal(TemporalConfig::default()),
            SamplingStrategy::Balanced(BalancedConfig::default()),
        ];
        for s in strategies {
            let json = serde_json::to_string(&s).expect("serialize strategy");
            let back: SamplingStrategy = serde_json::from_str(&json).expect("deserialize");
            match (s, back) {
                (SamplingStrategy::Uniform, SamplingStrategy::Uniform) => {}
                (SamplingStrategy::Prioritized(a), SamplingStrategy::Prioritized(b)) => {
                    assert!((a.alpha - b.alpha).abs() < f64::EPSILON);
                }
                (SamplingStrategy::Temporal(a), SamplingStrategy::Temporal(b)) => {
                    assert!((a.decay_factor - b.decay_factor).abs() < f64::EPSILON);
                }
                (SamplingStrategy::Balanced(a), SamplingStrategy::Balanced(b)) => {
                    assert!((a.recent_ratio - b.recent_ratio).abs() < f64::EPSILON);
                }
                _ => unreachable!("variant mismatch after roundtrip"),
            }
        }
    }

    #[test]
    fn experience_stats_and_priority_serde_roundtrip() {
        let stats = ExperienceStats {
            total_experiences: 1,
            current_size: 2,
            utilization: 0.5,
            average_priority: 0.3,
            samples_drawn: 4,
            average_reward: 0.1,
            success_rate: 0.8,
            oldest_experience_age: 12.0,
            last_update: Utc::now(),
        };
        let json = serde_json::to_string(&stats).expect("stats serde");
        let back: ExperienceStats = serde_json::from_str(&json).expect("stats de");
        assert_eq!(back.current_size, stats.current_size);
        assert!((back.utilization - stats.utilization).abs() < 1e-9);

        let prio = ExperiencePriority {
            experience_id: "e1".to_string(),
            priority: 2.0,
            td_error: 0.25,
            last_update: Utc::now(),
        };
        let pj = serde_json::to_string(&prio).expect("prio ser");
        let prio2: ExperiencePriority = serde_json::from_str(&pj).expect("prio de");
        assert_eq!(prio2.experience_id, "e1");
        assert!((prio2.td_error - 0.25).abs() < 1e-9);
    }

    #[test]
    fn experience_batch_serde_metadata_none() {
        let batch = ExperienceBatch {
            experiences: vec![],
            weights: vec![],
            indices: vec![],
            metadata: None,
        };
        let json = serde_json::to_string(&batch).expect("batch");
        let back: ExperienceBatch = serde_json::from_str(&json).expect("batch de");
        assert!(back.metadata.is_none());
    }

    #[test]
    fn buffer_get_out_of_bounds() {
        let mut b = ExperienceBuffer::new(4);
        b.add(sample_exp("a", 1.0, 1.0));
        assert!(b.get(0).is_some());
        assert!(b.get(99).is_none());
    }

    #[test]
    fn buffer_sample_uniform_caps_at_size() {
        let mut b = ExperienceBuffer::new(100);
        b.add(sample_exp("only", 1.0, 1.0));
        let s = b.sample_uniform(50);
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn buffer_is_full_false_until_capacity() {
        let mut b = ExperienceBuffer::new(3);
        b.add(sample_exp("1", 0.0, 1.0));
        assert!(!b.is_full());
        b.add(sample_exp("2", 0.0, 1.0));
        b.add(sample_exp("3", 0.0, 1.0));
        assert!(b.is_full());
    }

    #[tokio::test]
    async fn replay_empty_buffer_leaves_default_averages() {
        let replay = ExperienceReplay::new(10);
        let stats = replay.get_stats().await;
        assert_eq!(stats.current_size, 0);
        assert_eq!(stats.utilization, 0.0);
    }

    #[tokio::test]
    async fn replay_update_priorities_ignores_oob_index() {
        let replay = ExperienceReplay::new(10);
        replay
            .add_experience(sample_exp("x", 1.0, 1.0))
            .await
            .expect("add");
        replay
            .update_priorities(vec![999], vec![99.0])
            .await
            .expect("update");
        let all = replay.get_all_experiences().await;
        assert!((all[0].priority - 1.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn replay_uniform_batch_larger_than_buffer() {
        let replay = ExperienceReplay::new(100);
        for i in 0..3 {
            replay
                .add_experience(sample_exp(&format!("e{i}"), 1.0, 1.0))
                .await
                .expect("add");
        }
        let batch = replay.sample_batch(100).await.expect("sample");
        assert_eq!(batch.experiences.len(), 3);
        assert_eq!(batch.weights.len(), 3);
    }

    #[tokio::test]
    async fn replay_prioritized_normalizes_weights_when_positive() {
        let cfg = PrioritizedConfig::default();
        let replay =
            ExperienceReplay::with_sampling_strategy(50, SamplingStrategy::Prioritized(cfg));
        for i in 0..20 {
            replay
                .add_experience(sample_exp(&format!("e{i}"), 1.0, (i as f64) * 0.01 + 0.1))
                .await
                .expect("add");
        }
        let batch = replay.sample_batch(8).await.expect("sample");
        assert_eq!(batch.experiences.len(), 8);
        if !batch.weights.is_empty() {
            let mx = batch
                .weights
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max);
            assert!((mx - 1.0).abs() < 1e-9 || mx <= 1.0 + 1e-9);
        }
    }

    #[tokio::test]
    async fn experience_trait_timestamp_and_id() {
        let ts = Utc::now();
        let mut e = sample_exp("tid", 0.5, 2.0);
        e.timestamp = ts;
        let exp: &dyn Experience = &e;
        assert_eq!(exp.id(), "tid");
        assert_eq!(exp.timestamp(), ts);
        assert!((exp.priority() - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn config_defaults_match_documented() {
        let p = PrioritizedConfig::default();
        assert!((p.beta_annealing_rate - 0.001).abs() < f64::EPSILON);
        assert!((p.epsilon - 1e-6).abs() < 1e-9);
        let t = TemporalConfig::default();
        assert!(t.min_probability > 0.0);
        let b = BalancedConfig::default();
        assert!(b.recent_threshold >= 0.0 && b.recent_threshold <= 1.0);
    }
}
