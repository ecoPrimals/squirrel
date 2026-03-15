// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Tests for experience replay system

use super::engine::RLExperience;
use super::experience::*;
use chrono::Utc;
use uuid::Uuid;

#[test]
fn test_experience_buffer_new() {
    let buffer = ExperienceBuffer::new(100);
    assert_eq!(buffer.capacity(), 100);
    assert_eq!(buffer.size(), 0);
    assert!(buffer.is_empty());
    assert!(!buffer.is_full());
}

#[test]
fn test_experience_buffer_add() {
    let mut buffer = ExperienceBuffer::new(10);
    let experience = create_test_experience("exp_1");

    buffer.add(experience.clone());

    assert_eq!(buffer.size(), 1);
    assert!(!buffer.is_empty());
    assert!(!buffer.is_full());
}

#[test]
fn test_experience_buffer_circular() {
    let mut buffer = ExperienceBuffer::new(5);

    // Fill buffer beyond capacity
    for i in 0..10 {
        let experience = create_test_experience(&format!("exp_{}", i));
        buffer.add(experience);
    }

    // Should only keep last 5 experiences
    assert_eq!(buffer.size(), 5);
    assert!(buffer.is_full());
}

#[test]
fn test_experience_buffer_get() {
    let mut buffer = ExperienceBuffer::new(10);
    let experience = create_test_experience("exp_1");

    buffer.add(experience.clone());

    let retrieved = buffer.get(0).expect("Should get experience");
    assert_eq!(retrieved.id, "exp_1");
}

#[test]
fn test_experience_buffer_get_all() {
    let mut buffer = ExperienceBuffer::new(10);

    for i in 0..5 {
        let experience = create_test_experience(&format!("exp_{}", i));
        buffer.add(experience);
    }

    let all_experiences = buffer.get_all();
    assert_eq!(all_experiences.len(), 5);
}

#[test]
fn test_experience_buffer_sample_uniform() {
    let mut buffer = ExperienceBuffer::new(100);

    for i in 0..50 {
        let experience = create_test_experience(&format!("exp_{}", i));
        buffer.add(experience);
    }

    let sample = buffer.sample_uniform(10);
    assert_eq!(sample.len(), 10);
}

#[test]
fn test_experience_buffer_clear() {
    let mut buffer = ExperienceBuffer::new(10);

    for i in 0..5 {
        let experience = create_test_experience(&format!("exp_{}", i));
        buffer.add(experience);
    }

    assert!(!buffer.is_empty());

    buffer.clear();

    assert!(buffer.is_empty());
    assert_eq!(buffer.size(), 0);
}

#[tokio::test]
async fn test_experience_replay_new() {
    let replay = ExperienceReplay::new(1000);
    assert_eq!(replay.capacity(), 1000);
    assert!(replay.is_empty().await);
}

#[tokio::test]
async fn test_experience_replay_add_experience() {
    let replay = ExperienceReplay::new(1000);
    let experience = create_test_experience("exp_1");

    replay
        .add_experience(experience)
        .await
        .expect("Should add experience");

    assert_eq!(replay.size().await, 1);
    assert!(!replay.is_empty().await);
}

#[tokio::test]
async fn test_experience_replay_add_multiple() {
    let replay = ExperienceReplay::new(1000);

    let mut experiences = Vec::new();
    for i in 0..10 {
        experiences.push(create_test_experience(&format!("exp_{}", i)));
    }

    replay
        .add_experiences(experiences)
        .await
        .expect("Should add experiences");

    assert_eq!(replay.size().await, 10);
}

#[tokio::test]
async fn test_experience_replay_sample_batch_uniform() {
    let replay = ExperienceReplay::new(1000);

    for i in 0..50 {
        let experience = create_test_experience(&format!("exp_{}", i));
        replay
            .add_experience(experience)
            .await
            .expect("Should add experience");
    }

    let batch = replay.sample_batch(10).await.expect("Should sample batch");

    assert_eq!(batch.experiences.len(), 10);
    assert_eq!(batch.weights.len(), 10);
    assert_eq!(batch.indices.len(), 10);
}

#[tokio::test]
async fn test_experience_replay_sample_batch_empty() {
    let replay = ExperienceReplay::new(1000);

    let batch = replay.sample_batch(10).await.expect("Should sample batch");

    assert_eq!(batch.experiences.len(), 0);
}

#[tokio::test]
async fn test_experience_replay_with_prioritized_sampling() {
    let config = PrioritizedConfig::default();
    let replay =
        ExperienceReplay::with_sampling_strategy(1000, SamplingStrategy::Prioritized(config));

    for i in 0..50 {
        let mut experience = create_test_experience(&format!("exp_{}", i));
        experience.priority = (i as f64) / 50.0; // Varying priorities
        replay
            .add_experience(experience)
            .await
            .expect("Should add experience");
    }

    let batch = replay.sample_batch(10).await.expect("Should sample batch");

    assert_eq!(batch.experiences.len(), 10);
    // Higher priority experiences should be more likely to be sampled
}

#[tokio::test]
async fn test_experience_replay_with_temporal_sampling() {
    let config = TemporalConfig::default();
    let replay = ExperienceReplay::with_sampling_strategy(1000, SamplingStrategy::Temporal(config));

    for i in 0..50 {
        let experience = create_test_experience(&format!("exp_{}", i));
        replay
            .add_experience(experience)
            .await
            .expect("Should add experience");
    }

    let batch = replay.sample_batch(10).await.expect("Should sample batch");

    assert_eq!(batch.experiences.len(), 10);
}

#[tokio::test]
async fn test_experience_replay_with_balanced_sampling() {
    let config = BalancedConfig::default();
    let replay = ExperienceReplay::with_sampling_strategy(1000, SamplingStrategy::Balanced(config));

    for i in 0..50 {
        let experience = create_test_experience(&format!("exp_{}", i));
        replay
            .add_experience(experience)
            .await
            .expect("Should add experience");
    }

    let batch = replay.sample_batch(10).await.expect("Should sample batch");

    assert_eq!(batch.experiences.len(), 10);
}

#[tokio::test]
async fn test_experience_replay_update_priorities() {
    let replay = ExperienceReplay::new(1000);

    for i in 0..10 {
        let experience = create_test_experience(&format!("exp_{}", i));
        replay
            .add_experience(experience)
            .await
            .expect("Should add experience");
    }

    let indices = vec![0, 1, 2];
    let priorities = vec![1.5, 2.0, 2.5];

    replay
        .update_priorities(indices, priorities)
        .await
        .expect("Should update priorities");
}

#[tokio::test]
async fn test_experience_replay_get_stats() {
    let replay = ExperienceReplay::new(1000);

    for i in 0..10 {
        let experience = create_test_experience(&format!("exp_{}", i));
        replay
            .add_experience(experience)
            .await
            .expect("Should add experience");
    }

    let stats = replay.get_stats().await;

    assert_eq!(stats.current_size, 10);
    assert_eq!(stats.utilization, 10.0 / 1000.0);
}

#[tokio::test]
async fn test_experience_replay_clear() {
    let replay = ExperienceReplay::new(1000);

    for i in 0..10 {
        let experience = create_test_experience(&format!("exp_{}", i));
        replay
            .add_experience(experience)
            .await
            .expect("Should add experience");
    }

    assert!(!replay.is_empty().await);

    replay.clear().await.expect("Should clear");

    assert!(replay.is_empty().await);
    assert_eq!(replay.size().await, 0);
}

#[tokio::test]
async fn test_experience_replay_is_full() {
    let replay = ExperienceReplay::new(10);

    for i in 0..10 {
        let experience = create_test_experience(&format!("exp_{}", i));
        replay
            .add_experience(experience)
            .await
            .expect("Should add experience");
    }

    assert!(replay.is_full().await);
}

#[tokio::test]
async fn test_experience_replay_get_all_experiences() {
    let replay = ExperienceReplay::new(1000);

    for i in 0..5 {
        let experience = create_test_experience(&format!("exp_{}", i));
        replay
            .add_experience(experience)
            .await
            .expect("Should add experience");
    }

    let all_experiences = replay.get_all_experiences().await;
    assert_eq!(all_experiences.len(), 5);
}

#[tokio::test]
async fn test_experience_replay_set_sampling_strategy() {
    let mut replay = ExperienceReplay::new(1000);

    let config = PrioritizedConfig::default();
    replay.set_sampling_strategy(SamplingStrategy::Prioritized(config));

    // Verify the strategy is set correctly
    match replay.get_sampling_strategy() {
        SamplingStrategy::Prioritized(_) => {} // Success
        _ => panic!("Expected Prioritized sampling strategy"),
    }
}

#[test]
fn test_sampling_strategy_uniform() {
    let strategy = SamplingStrategy::Uniform;
    let serialized = serde_json::to_string(&strategy).expect("Should serialize");
    assert!(serialized.contains("Uniform"));
}

#[test]
fn test_prioritized_config_default() {
    let config = PrioritizedConfig::default();
    assert_eq!(config.alpha, 0.6);
    assert_eq!(config.beta, 0.4);
    assert_eq!(config.max_beta, 1.0);
}

#[test]
fn test_temporal_config_default() {
    let config = TemporalConfig::default();
    assert_eq!(config.decay_factor, 0.95);
    assert_eq!(config.min_probability, 0.01);
}

#[test]
fn test_balanced_config_default() {
    let config = BalancedConfig::default();
    assert_eq!(config.recent_ratio, 0.7);
    assert_eq!(config.recent_threshold, 0.2);
}

#[test]
fn test_experience_stats_default() {
    let stats = ExperienceStats::default();
    assert_eq!(stats.total_experiences, 0);
    assert_eq!(stats.current_size, 0);
    assert_eq!(stats.utilization, 0.0);
}

#[test]
fn test_experience_batch_serialization() {
    let batch = ExperienceBatch {
        experiences: vec![create_test_experience("exp_1")],
        weights: vec![1.0],
        indices: vec![0],
        metadata: Some(serde_json::json!({"strategy": "uniform"})),
    };

    let serialized = serde_json::to_string(&batch).expect("Should serialize");
    let deserialized: ExperienceBatch =
        serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(deserialized.experiences.len(), 1);
    assert_eq!(deserialized.weights.len(), 1);
}

#[test]
fn test_experience_trait() {
    let mut experience = create_test_experience("exp_1");

    assert_eq!(experience.id(), "exp_1");
    assert_eq!(experience.priority(), 1.0);

    experience.set_priority(2.5);
    assert_eq!(experience.priority(), 2.5);
}

#[tokio::test]
async fn test_experience_stats_average_reward() {
    let replay = ExperienceReplay::new(1000);

    for i in 0..10 {
        let mut experience = create_test_experience(&format!("exp_{}", i));
        experience.reward = i as f64;
        replay
            .add_experience(experience)
            .await
            .expect("Should add experience");
    }

    let stats = replay.get_stats().await;

    // Average of 0..9 is 4.5
    assert!((stats.average_reward - 4.5).abs() < 0.01);
}

#[tokio::test]
async fn test_experience_stats_success_rate() {
    let replay = ExperienceReplay::new(1000);

    for i in 0..10 {
        let mut experience = create_test_experience(&format!("exp_{}", i));
        experience.reward = if i % 2 == 0 { 1.0 } else { -1.0 };
        replay
            .add_experience(experience)
            .await
            .expect("Should add experience");
    }

    let stats = replay.get_stats().await;

    // 50% have positive rewards
    assert!((stats.success_rate - 0.5).abs() < 0.01);
}

// Helper function to create test experiences
fn create_test_experience(id: &str) -> RLExperience {
    use super::engine::{RLAction, RLState};

    RLExperience {
        id: id.to_string(),
        state: RLState {
            id: Uuid::new_v4().to_string(),
            context_id: "test_context".to_string(),
            features: vec![1.0, 2.0, 3.0],
            metadata: None,
            timestamp: Utc::now(),
        },
        action: RLAction {
            id: Uuid::new_v4().to_string(),
            action_type: "test_action".to_string(),
            parameters: serde_json::Value::Null,
            confidence: 0.8,
            expected_reward: 1.0,
        },
        reward: 1.0,
        next_state: Some(RLState {
            id: Uuid::new_v4().to_string(),
            context_id: "test_context".to_string(),
            features: vec![1.1, 2.1, 3.1],
            metadata: None,
            timestamp: Utc::now(),
        }),
        done: false,
        timestamp: Utc::now(),
        priority: 1.0,
    }
}
