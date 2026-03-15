// SPDX-License-Identifier: AGPL-3.0-only
// ORC-Notice: Learning system mechanics licensed under ORC
// Copyright (C) 2026 DataScienceBioLab

//! Comprehensive unit tests for the learning subsystem.

use super::adaptive::*;
use super::engine::{LearningAlgorithm, RLAction, RLExperience, RLState};
use super::experience::*;
use super::integration::*;
use super::manager::*;
use super::metrics::*;
use super::policy::*;
use super::reward::*;
use super::test_helpers;
use super::*;
use chrono::Utc;
use serde_json;
use std::sync::Arc;
use uuid::Uuid;

// --- mod.rs types ---

#[test]
fn test_learning_system_config_default() {
    let config = LearningSystemConfig::default();
    assert!(config.enable_reinforcement_learning);
    assert_eq!(config.learning_rate, 0.001);
    assert_eq!(config.discount_factor, 0.95);
    assert_eq!(config.exploration_rate, 0.1);
}

#[test]
fn test_learning_system_config_serialization() {
    let config = LearningSystemConfig::default();
    let json = serde_json::to_string(&config).expect("serialize");
    let restored: LearningSystemConfig = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(config.learning_rate, restored.learning_rate);
}

#[test]
fn test_reward_calculation_type_variants() {
    let _ = RewardCalculationType::Simple;
    let _ = RewardCalculationType::Composite;
    let _ = RewardCalculationType::Custom("custom".to_string());
}

#[test]
fn test_policy_network_config_default() {
    let config = PolicyNetworkConfig::default();
    assert_eq!(config.input_size, 128);
    assert_eq!(config.output_size, 32);
    assert!(!config.hidden_layers.is_empty());
}

#[test]
fn test_learning_state_variants() {
    let _ = LearningState::Initializing;
    let _ = LearningState::Learning;
    let _ = LearningState::Stopped;
}

#[test]
fn test_learning_system_stats_default() {
    let stats = LearningSystemStats::default();
    assert_eq!(stats.total_episodes, 0);
    assert_eq!(stats.total_actions, 0);
}

#[test]
fn test_learning_system_stats_serialization() {
    let stats = LearningSystemStats::default();
    let json = serde_json::to_string(&stats).expect("serialize");
    let _: LearningSystemStats = serde_json::from_str(&json).expect("deserialize");
}

// --- adaptive ---

#[tokio::test]
async fn test_adaptive_rule_system_new() {
    let config = test_helpers::create_test_learning_config();
    let system = AdaptiveRuleSystem::new(Arc::new(config))
        .await
        .expect("create");
    let stats = system.get_stats().await;
    assert_eq!(stats.total_adaptations, 0);
}

#[tokio::test]
async fn test_adaptive_rule_system_add_rule() {
    let config = test_helpers::create_test_learning_config();
    let system = AdaptiveRuleSystem::new(Arc::new(config))
        .await
        .expect("create");
    let rule = test_helpers::create_test_rule();
    system.add_rule(rule.clone()).await.expect("add");
    let rules = system.get_adaptive_rules().await;
    assert_eq!(rules.len(), 1);
    assert!(rules.contains_key(rule.id()));
}

#[tokio::test]
async fn test_adaptive_rule_system_update_performance() {
    let config = test_helpers::create_test_learning_config();
    let system = AdaptiveRuleSystem::new(Arc::new(config))
        .await
        .expect("create");
    let rule = test_helpers::create_test_rule();
    let id = rule.id().to_string();
    system.add_rule(rule).await.expect("add");
    system
        .update_rule_performance(&id, true, 0.5)
        .await
        .expect("update");
    let perf = system.get_rule_performance(&id).await.expect("get");
    assert_eq!(perf.application_count, 1);
    assert_eq!(perf.success_count, 1);
}

#[tokio::test]
async fn test_adaptive_rule_system_remove_rule() {
    let config = test_helpers::create_test_learning_config();
    let system = AdaptiveRuleSystem::new(Arc::new(config))
        .await
        .expect("create");
    let rule = test_helpers::create_test_rule();
    let id = rule.id().to_string();
    system.add_rule(rule).await.expect("add");
    system.remove_rule(&id).await.expect("remove");
    let rules = system.get_adaptive_rules().await;
    assert!(rules.is_empty());
}

#[test]
fn test_rule_performance_default() {
    let perf = RulePerformance::default();
    assert_eq!(perf.success_rate, 0.0);
    assert_eq!(perf.application_count, 0);
}

#[test]
fn test_adaptation_strategy_serialization() {
    let s = AdaptationStrategy::Gradual;
    let _ = serde_json::to_string(&s).expect("serialize");
}

#[test]
fn test_rule_change_construction() {
    let change = RuleChange {
        change_type: "test".to_string(),
        target: "t".to_string(),
        previous_value: serde_json::json!(1),
        new_value: serde_json::json!(2),
        confidence: 0.9,
    };
    assert_eq!(change.change_type, "test");
}

// --- engine ---

fn make_rl_state(id: &str) -> RLState {
    RLState {
        id: id.to_string(),
        features: vec![1.0, 2.0, 3.0],
        context_id: "ctx".to_string(),
        timestamp: Utc::now(),
        metadata: None,
    }
}

fn make_rl_experience(id: &str, reward: f64) -> RLExperience {
    RLExperience {
        id: id.to_string(),
        state: make_rl_state("s1"),
        action: RLAction {
            id: Uuid::new_v4().to_string(),
            action_type: "modify_context".to_string(),
            parameters: serde_json::Value::Null,
            confidence: 0.8,
            expected_reward: 0.0,
        },
        reward,
        next_state: Some(make_rl_state("s2")),
        done: false,
        timestamp: Utc::now(),
        priority: 1.0,
    }
}

#[tokio::test]
async fn test_learning_engine_new() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let engine = LearningEngine::new(config).await.expect("create");
    assert_eq!(engine.get_q_table_size().await, 0);
    assert_eq!(engine.get_experience_buffer_size().await, 0);
}

#[tokio::test]
async fn test_learning_engine_select_action() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let engine = LearningEngine::new(config).await.expect("create");
    engine.initialize().await.expect("init");
    let state = make_rl_state("s1");
    let action = engine.select_action(&state).await.expect("select");
    assert!(!action.id.is_empty());
    assert!(!action.action_type.is_empty());
}

#[tokio::test]
async fn test_learning_engine_add_experience() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let engine = LearningEngine::new(config).await.expect("create");
    let exp = make_rl_experience("e1", 1.0);
    engine.add_experience(exp).await.expect("add");
    assert_eq!(engine.get_experience_buffer_size().await, 1);
}

#[tokio::test]
async fn test_learning_engine_update_q_values() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let engine = LearningEngine::new(config).await.expect("create");
    let exp = make_rl_experience("e1", 1.0);
    engine.update_q_values(&exp).await.expect("update");
    // DQN adds to experience buffer; Q-learning updates Q-table
    assert!(engine.get_q_table_size().await >= 1 || engine.get_experience_buffer_size().await >= 1);
}

#[tokio::test]
async fn test_learning_engine_decay_exploration() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let engine = LearningEngine::new(config).await.expect("create");
    let before = engine.get_exploration_rate().await;
    engine.decay_exploration().await.expect("decay");
    let after = engine.get_exploration_rate().await;
    assert!(after <= before);
}

#[test]
fn test_learning_engine_config_default() {
    let config = LearningEngineConfig::default();
    assert!(matches!(config.algorithm, LearningAlgorithm::DeepQLearning));
}

#[test]
fn test_rl_state_serialization() {
    let state = make_rl_state("s1");
    let json = serde_json::to_string(&state).expect("serialize");
    let _: RLState = serde_json::from_str(&json).expect("deserialize");
}

// --- experience ---

#[test]
fn test_experience_buffer_new() {
    let buf = ExperienceBuffer::new(100);
    assert_eq!(buf.capacity(), 100);
    assert!(buf.is_empty());
}

#[test]
fn test_experience_buffer_add_and_get() {
    let mut buf = ExperienceBuffer::new(10);
    let exp = make_rl_experience("e1", 1.0);
    buf.add(exp);
    assert_eq!(buf.size(), 1);
    let got = buf.get(0).expect("get");
    assert_eq!(got.id, "e1");
}

#[test]
fn test_experience_buffer_sample_uniform() {
    let mut buf = ExperienceBuffer::new(100);
    for i in 0..20 {
        buf.add(make_rl_experience(&format!("e{}", i), 1.0));
    }
    let sample = buf.sample_uniform(5);
    assert_eq!(sample.len(), 5);
}

#[tokio::test]
async fn test_experience_replay_new() {
    let replay = ExperienceReplay::new(100);
    assert_eq!(replay.capacity(), 100);
    assert!(replay.is_empty().await);
}

#[tokio::test]
async fn test_experience_replay_add_and_sample() {
    let replay = ExperienceReplay::new(100);
    replay
        .add_experience(make_rl_experience("e1", 1.0))
        .await
        .expect("add");
    let batch = replay.sample_batch(1).await.expect("sample");
    assert_eq!(batch.experiences.len(), 1);
}

#[tokio::test]
async fn test_experience_replay_prioritized_sampling() {
    let config = PrioritizedConfig::default();
    let replay =
        ExperienceReplay::with_sampling_strategy(50, SamplingStrategy::Prioritized(config));
    for i in 0..20 {
        let mut exp = make_rl_experience(&format!("e{}", i), 1.0);
        exp.priority = (i as f64) / 20.0;
        replay.add_experience(exp).await.expect("add");
    }
    let batch = replay.sample_batch(5).await.expect("sample");
    assert!(!batch.experiences.is_empty());
}

#[test]
fn test_sampling_strategy_defaults() {
    let _ = PrioritizedConfig::default();
    let _ = TemporalConfig::default();
    let _ = BalancedConfig::default();
}

#[test]
fn test_experience_stats_default() {
    let stats = ExperienceStats::default();
    assert_eq!(stats.current_size, 0);
}

// --- metrics ---

#[tokio::test]
async fn test_learning_metrics_new() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let metrics = LearningMetrics::new(config).await.expect("create");
    let perf = metrics.get_performance().await;
    assert_eq!(perf.learning_rate, 0.0);
}

#[tokio::test]
async fn test_learning_metrics_update_performance() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let metrics = LearningMetrics::new(config).await.expect("create");
    metrics.initialize().await.expect("init");
    let mut updates = std::collections::HashMap::new();
    updates.insert("success_rate".to_string(), 0.9);
    metrics.update_performance(updates).await.expect("update");
    let perf = metrics.get_performance().await;
    assert_eq!(perf.success_rate, 0.9);
}

#[tokio::test]
async fn test_learning_metrics_record_episode() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let metrics = LearningMetrics::new(config).await.expect("create");
    metrics
        .record_episode(true, 10.0, 5, 1.0)
        .await
        .expect("record");
    let stats = metrics.get_stats().await;
    assert_eq!(stats.total_episodes, 1);
    assert_eq!(stats.successful_episodes, 1);
}

#[tokio::test]
async fn test_learning_metrics_take_snapshot() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let metrics = LearningMetrics::new(config).await.expect("create");
    let id = metrics.take_snapshot().await.expect("snapshot");
    assert!(!id.is_empty());
    let snap = metrics.get_snapshot(&id).await;
    assert!(snap.is_some());
}

#[tokio::test]
async fn test_learning_metrics_custom_metric() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let metrics = LearningMetrics::new(config).await.expect("create");
    metrics
        .set_custom_metric("custom".to_string(), 42.0)
        .await
        .expect("set");
    let val = metrics.get_custom_metric("custom").await;
    assert_eq!(val, Some(42.0));
}

#[test]
fn test_learning_performance_default() {
    let p = LearningPerformance::default();
    assert_eq!(p.success_rate, 0.0);
}

#[test]
fn test_learning_stats_default() {
    let s = super::metrics::LearningStats::default();
    assert_eq!(s.total_episodes, 0);
}

// --- policy ---

#[tokio::test]
async fn test_policy_network_new() {
    let config = test_helpers::create_test_policy_config();
    let net = PolicyNetwork::new(config).await.expect("create");
    let cfg = net.get_config();
    assert_eq!(cfg.input_size, 10);
}

#[tokio::test]
async fn test_policy_network_forward() {
    let config = test_helpers::create_test_policy_config();
    let net = PolicyNetwork::new(config).await.expect("create");
    let input: Vec<f64> = (0..10).map(|i| i as f64 / 10.0).collect();
    let action = net.forward(&input).await.expect("forward");
    assert_eq!(action.action_probabilities.len(), 5);
    assert!(action.selected_action < 5);
}

#[tokio::test]
async fn test_policy_network_forward_wrong_size() {
    let config = test_helpers::create_test_policy_config();
    let net = PolicyNetwork::new(config).await.expect("create");
    let input = vec![1.0, 2.0]; // wrong size
    let result = net.forward(&input).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_policy_network_predict() {
    let config = test_helpers::create_test_policy_config();
    let net = PolicyNetwork::new(config).await.expect("create");
    let state = make_rl_state("s1");
    let action = net.predict(&state).await.expect("predict");
    assert!(action.confidence >= 0.0 && action.confidence <= 1.0);
}

#[tokio::test]
async fn test_policy_network_train() {
    let config = test_helpers::create_test_policy_config();
    let net = PolicyNetwork::new(config).await.expect("create");
    let exps: Vec<RLExperience> = (0..5)
        .map(|i| make_rl_experience(&format!("e{}", i), 1.0))
        .collect();
    net.train(&exps).await.expect("train");
    let state = net.get_training_state().await;
    assert!(state.epoch >= 1);
}

#[test]
fn test_policy_action_construction() {
    let action = PolicyAction {
        action_probabilities: vec![0.5, 0.5],
        selected_action: 0,
        confidence: 0.5,
        value_estimate: 0.5,
    };
    assert_eq!(action.action_probabilities.len(), 2);
}

// --- reward ---

fn make_reward_context(success: bool) -> RewardContext {
    RewardContext {
        action: RLAction {
            id: Uuid::new_v4().to_string(),
            action_type: "test".to_string(),
            parameters: serde_json::Value::Null,
            confidence: 1.0,
            expected_reward: 0.0,
        },
        previous_state: make_rl_state("prev"),
        current_state: make_rl_state("curr"),
        performance_metrics: PerformanceMetrics {
            sync_status: true,
            version: 1,
            active_contexts: 1,
            memory_usage: 0.5,
            processing_time: 0.1,
            success_rate: 0.9,
            error_rate: 0.05,
            throughput: 20.0,
        },
        rule_results: None,
        error_info: if success {
            None
        } else {
            Some(ErrorInfo {
                error_type: "test".to_string(),
                severity: ErrorSeverity::Low,
                message: "err".to_string(),
                recoverable: true,
                impact: 0.5,
            })
        },
        timestamp: Utc::now(),
    }
}

#[tokio::test]
async fn test_reward_system_new() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let system = RewardSystem::new(config).await.expect("create");
    let metrics = system.get_metrics().await;
    assert_eq!(metrics.total_rewards, 0);
}

#[tokio::test]
async fn test_reward_system_calculate_success() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let system = RewardSystem::new(config).await.expect("create");
    let ctx = make_reward_context(true);
    let reward = system.calculate_reward(ctx).await.expect("calc");
    assert!(reward > 0.0);
}

#[tokio::test]
async fn test_reward_system_calculate_failure() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let system = RewardSystem::new(config).await.expect("create");
    let ctx = make_reward_context(false);
    let reward = system.calculate_reward(ctx).await.expect("calc");
    assert!(reward < 0.0);
}

#[tokio::test]
async fn test_reward_system_custom_calculator() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let system = RewardSystem::new(config).await.expect("create");
    let calc = SuccessRewardCalculator::new(5.0, -2.0);
    system
        .add_calculator("custom".to_string(), Box::new(calc))
        .await
        .expect("add");
    system.remove_calculator("custom").await.expect("remove");
}

#[test]
fn test_success_reward_calculator() {
    let calc = SuccessRewardCalculator::new(10.0, -5.0);
    let ctx = make_reward_context(true);
    let r = calc.calculate_reward(&ctx).expect("calc");
    assert_eq!(r, 10.0);
}

#[test]
fn test_performance_reward_calculator() {
    let calc = PerformanceRewardCalculator::new(PerformanceThresholds::default());
    let ctx = make_reward_context(true);
    let _ = calc.calculate_reward(&ctx).expect("calc");
}

#[test]
fn test_reward_breakdown_construction() {
    let b = RewardBreakdown {
        base_reward: 1.0,
        performance_bonus: 0.5,
        efficiency_bonus: 0.3,
        sync_bonus: 0.2,
        error_penalty: 0.0,
        time_penalty: 0.0,
        final_reward: 2.0,
    };
    assert_eq!(b.final_reward, 2.0);
}

// --- manager ---

#[tokio::test]
async fn test_context_learning_manager_new() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(config).await.expect("create");
    let state = manager.get_state().await;
    assert!(matches!(
        state,
        LearningState::Initializing | LearningState::Learning
    ));
}

#[tokio::test]
async fn test_context_learning_manager_start_episode() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(config).await.expect("create");
    manager.initialize().await.expect("init");
    let ep_id = manager.start_episode("ctx1").await.expect("start");
    assert!(!ep_id.is_empty());
    let episodes = manager.get_active_episodes().await;
    assert_eq!(episodes.len(), 1);
}

#[tokio::test]
async fn test_context_learning_manager_take_action() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(config).await.expect("create");
    manager.initialize().await.expect("init");
    let ep_id = manager.start_episode("ctx1").await.expect("start");
    let action = manager.take_action(&ep_id, "ctx1").await.expect("action");
    assert!(!action.action_type.is_empty());
}

#[tokio::test]
async fn test_context_learning_manager_provide_reward() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(config).await.expect("create");
    manager.initialize().await.expect("init");
    let ep_id = manager.start_episode("ctx1").await.expect("start");
    manager.provide_reward(&ep_id, 5.0).await.expect("reward");
}

#[tokio::test]
async fn test_context_learning_manager_end_episode() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(config).await.expect("create");
    manager.initialize().await.expect("init");
    let ep_id = manager.start_episode("ctx1").await.expect("start");
    manager.end_episode(&ep_id, true).await.expect("end");
    let episodes = manager.get_active_episodes().await;
    assert!(episodes.is_empty());
}

// --- integration ---

#[tokio::test]
async fn test_learning_integration_new() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let integration = LearningIntegration::new(config).await.expect("create");
    let state = integration.get_state().await;
    assert!(matches!(
        state.status,
        IntegrationStatus::Active | IntegrationStatus::Initializing
    ));
}

#[tokio::test]
async fn test_learning_integration_trigger_episode() {
    let mut config = test_helpers::create_test_learning_config();
    config.enable_reinforcement_learning = true;
    let config = Arc::new(config);
    let integration = LearningIntegration::new(config).await.expect("create");
    let ep_id = integration
        .trigger_learning_episode("ctx1")
        .await
        .expect("trigger");
    assert!(!ep_id.is_empty());
}

#[tokio::test]
async fn test_learning_integration_calculate_reward() {
    let mut config = test_helpers::create_test_learning_config();
    config.enable_reinforcement_learning = true;
    let config = Arc::new(config);
    let integration = LearningIntegration::new(config).await.expect("create");
    let reward = integration
        .calculate_reward("ctx1", serde_json::json!({}))
        .await
        .expect("calc");
    assert!(reward.is_finite());
}

#[test]
fn test_learning_integration_config_default() {
    let config = LearningIntegrationConfig::default();
    assert!(config.enable_context_manager);
}

#[test]
fn test_trigger_thresholds_default() {
    let t = TriggerThresholds::default();
    assert!(t.min_context_changes > 0);
}
