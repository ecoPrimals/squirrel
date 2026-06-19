// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Behavioral tests for Context Learning Manager — episodes, stats, inference, and sessions.

use super::manager::ContextLearningManager;
use crate::learning::test_helpers;
use serde_json::json;
use std::sync::Arc;

// ============================================================================
// Model management, training, inference tests
// ============================================================================

#[tokio::test]
async fn test_manager_learning_stats_after_multiple_episodes() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(system_config)
        .await
        .expect("Failed to create manager");

    manager.initialize().await.expect("Failed to initialize");

    for i in 0..5 {
        let episode_id = manager
            .start_episode(&format!("context_{}", i))
            .await
            .expect("Failed to start episode");
        manager
            .take_action(&episode_id, &format!("context_{}", i))
            .await
            .expect("Failed to take action");
        manager
            .provide_reward(&episode_id, (i + 1) as f64)
            .await
            .expect("Failed to provide reward");
        manager
            .end_episode(&episode_id, i % 2 == 0)
            .await
            .expect("Failed to end episode");
    }

    let stats = manager.get_learning_stats().await;
    assert_eq!(stats.total_episodes, 5);
    assert!(stats.successful_episodes >= 2);
    assert!(stats.success_rate > 0.0);
    assert!(stats.average_reward_per_episode > 0.0);
}

#[tokio::test]
async fn test_manager_episode_total_reward_calculation() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(system_config)
        .await
        .expect("Failed to create manager");

    manager.initialize().await.expect("Failed to initialize");

    let episode_id = manager
        .start_episode("test_context")
        .await
        .expect("Failed to start episode");

    manager
        .provide_reward(&episode_id, 1.0)
        .await
        .expect("Failed");
    manager
        .provide_reward(&episode_id, 2.0)
        .await
        .expect("Failed");
    manager
        .provide_reward(&episode_id, 3.0)
        .await
        .expect("Failed");

    manager
        .end_episode(&episode_id, true)
        .await
        .expect("Failed");

    let history = manager.get_episode_history().await;
    assert_eq!(history.len(), 1);
    assert!((history[0].total_reward - 6.0).abs() < 0.001);
}

#[tokio::test]
async fn test_manager_episode_duration_recorded() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(system_config)
        .await
        .expect("Failed to create manager");

    manager.initialize().await.expect("Failed to initialize");

    let episode_id = manager.start_episode("test_context").await.expect("Failed");
    manager
        .end_episode(&episode_id, true)
        .await
        .expect("Failed");

    let history = manager.get_episode_history().await;
    assert_eq!(history.len(), 1);
    assert!(history[0].duration.is_some());
}

#[tokio::test]
async fn test_manager_set_rule_manager() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(system_config)
        .await
        .expect("Failed to create manager");

    manager.initialize().await.expect("Failed to initialize");

    let rule_manager = Arc::new(crate::rules::RuleManager::new("./rules"));
    manager.set_rule_manager(rule_manager).await;
}

#[tokio::test]
async fn test_manager_set_rule_manager_with_session_updates_rules_applied() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(system_config)
        .await
        .expect("Failed to create manager");

    manager.initialize().await.expect("Failed to initialize");
    manager.start().await.expect("Failed to start");

    let before = manager.get_learning_stats().await.rules_applied;
    let rule_manager = Arc::new(crate::rules::RuleManager::new("./rules"));
    manager.set_rule_manager(rule_manager).await;
    let after = manager.get_learning_stats().await.rules_applied;
    assert_eq!(after, before + 1);
}

#[tokio::test]
async fn test_manager_end_episode_unknown_id_is_ok() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(system_config)
        .await
        .expect("Failed to create manager");

    manager.initialize().await.expect("Failed to initialize");
    manager
        .end_episode("nonexistent-episode-id", false)
        .await
        .expect("end unknown episode should not error");
}

#[tokio::test]
async fn test_extract_features_dense_object_and_empty() {
    let rich = json!({
        "version": 2u64,
        "data": { "a": 1, "b": "s", "c": true },
        "synchronized": true,
        "extra": "x",
        "n": 3.5
    });
    let v = ContextLearningManager::extract_features(&rich)
        .await
        .expect("features");
    assert_eq!(v.len(), 6);
    assert!((v[0] - 2.0).abs() < f64::EPSILON);
    assert!((v[1] - 3.0).abs() < f64::EPSILON);
    assert!((v[2] - 1.0).abs() < f64::EPSILON);

    let empty = json!({});
    let v0 = ContextLearningManager::extract_features(&empty)
        .await
        .expect("empty");
    assert!(v0.iter().all(|x| x.abs() < f64::EPSILON));
}

#[tokio::test]
async fn test_manager_action_inference_from_engine() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(system_config)
        .await
        .expect("Failed to create manager");

    manager.initialize().await.expect("Failed to initialize");

    let episode_id = manager
        .start_episode("inference_context")
        .await
        .expect("Failed");

    let action = manager
        .take_action(&episode_id, "inference_context")
        .await
        .expect("Failed to take action");

    assert!(!action.id.is_empty());
    assert!(!action.action_type.is_empty());
    assert!(action.confidence >= 0.0 && action.confidence <= 1.0);
}

#[tokio::test]
async fn test_manager_session_tracks_multiple_episodes() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(system_config)
        .await
        .expect("Failed to create manager");

    manager.initialize().await.expect("Failed to initialize");
    manager.start().await.expect("Failed to start");

    let ep1 = manager.start_episode("ctx1").await.expect("Failed");
    let ep2 = manager.start_episode("ctx2").await.expect("Failed");

    manager.end_episode(&ep1, true).await.expect("Failed");
    manager.end_episode(&ep2, false).await.expect("Failed");

    let session = manager.get_current_session().await;
    assert!(session.is_some());
    let session = session.expect("should succeed");
    assert!(session.episodes.len() >= 2);
}

#[tokio::test]
async fn test_manager_learning_stats_success_rate() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(system_config)
        .await
        .expect("Failed to create manager");

    manager.initialize().await.expect("Failed to initialize");

    for i in 0..5 {
        let ep = manager
            .start_episode(&format!("ctx_{}", i))
            .await
            .expect("Failed");
        manager.provide_reward(&ep, 1.0).await.expect("Failed");
        manager.end_episode(&ep, i < 3).await.expect("Failed");
    }

    let stats = manager.get_learning_stats().await;
    assert_eq!(stats.total_episodes, 5);
    assert_eq!(stats.successful_episodes, 3);
    assert!((stats.success_rate - 0.6).abs() < 0.01);
}

#[tokio::test]
async fn test_manager_context_manager_access() {
    use squirrel_interfaces::context::ContextManager as ContextManagerTrait;

    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(system_config)
        .await
        .expect("Failed to create manager");

    let context_manager = manager.get_context_manager();
    manager.initialize().await.expect("Failed to initialize");

    let init_result = context_manager.initialize().await;
    assert!(init_result.is_ok());
}

#[tokio::test]
async fn test_manager_learning_engine_access() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(system_config)
        .await
        .expect("Failed to create manager");

    let engine = manager.get_learning_engine();
    let state = engine.get_state().await;
    assert!(matches!(
        state,
        crate::learning::LearningState::Initializing
    ));
}

#[tokio::test]
async fn test_manager_provide_reward_updates_learning_engine() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(system_config)
        .await
        .expect("Failed to create manager");

    manager.initialize().await.expect("Failed to initialize");

    let episode_id = manager.start_episode("test_context").await.expect("Failed");

    manager
        .take_action(&episode_id, "test_context")
        .await
        .expect("Failed to take action");

    let result = manager.provide_reward(&episode_id, 5.0).await;
    assert!(result.is_ok());

    let active = manager.get_active_episodes().await;
    assert_eq!(active[0].rewards.len(), 1);
    assert!((active[0].rewards[0] - 5.0).abs() < 0.001);
}

#[tokio::test]
async fn test_manager_episode_final_state_recorded() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(system_config)
        .await
        .expect("Failed to create manager");

    manager.initialize().await.expect("Failed to initialize");

    let episode_id = manager.start_episode("test_context").await.expect("Failed");

    manager
        .end_episode(&episode_id, true)
        .await
        .expect("Failed");

    let history = manager.get_episode_history().await;
    assert_eq!(history.len(), 1);
    assert!(history[0].final_state.is_some());
}

#[test]
fn test_manager_reward_parameters_default() {
    let params = super::manager::RewardParameters::default();
    assert!((params.success_reward - 10.0).abs() < f64::EPSILON);
    assert!((params.failure_penalty - (-5.0)).abs() < f64::EPSILON);
    assert!((params.step_penalty - (-0.1)).abs() < f64::EPSILON);
    assert!((params.error_penalty - (-10.0)).abs() < f64::EPSILON);
}
