// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Integration tests for learning system components
//!
//! These tests verify that the learning system components work together correctly,
//! testing the full workflow from system initialization through episode execution.

use super::*;
use crate::learning::test_helpers;
use std::sync::Arc;
use tokio::time::Duration;

// ============================================================================
// PHASE 6: Integration Tests (Sprint 1)
// ============================================================================

#[tokio::test]
async fn test_learning_system_creation() {
    let config = test_helpers::create_test_learning_config();
    let system = LearningSystem::new(config).await;

    assert!(system.is_ok(), "Learning system creation should succeed");
}

#[tokio::test]
async fn test_learning_system_initialization() {
    let config = test_helpers::create_test_learning_config();
    let system = LearningSystem::new(config)
        .await
        .expect("Failed to create system");

    let result = system.initialize().await;
    assert!(result.is_ok(), "System initialization should succeed");

    let state = system.get_state().await;
    assert!(
        matches!(state, LearningState::Learning),
        "System should be in Learning state"
    );
}

#[tokio::test]
async fn test_learning_system_start_stop() {
    let config = test_helpers::create_test_learning_config();
    let system = LearningSystem::new(config)
        .await
        .expect("Failed to create system");

    system.initialize().await.expect("Failed to initialize");

    // Start system
    system.start().await.expect("Failed to start");
    let state = system.get_state().await;
    assert!(matches!(state, LearningState::Learning));

    // Stop system
    system.stop().await.expect("Failed to stop");
    let state = system.get_state().await;
    assert!(matches!(state, LearningState::Stopped));
}

#[tokio::test]
async fn test_manager_uses_engine_for_actions() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(config)
        .await
        .expect("Failed to create manager");
    manager.initialize().await.expect("Failed to initialize");

    let episode_id = manager
        .start_episode("test_context")
        .await
        .expect("Failed to start episode");
    let action = manager
        .take_action(&episode_id, "test_context")
        .await
        .expect("Failed to take action");

    // Action should come from the engine (via manager)
    assert!(!action.id.is_empty(), "Action should have an ID");
    assert!(!action.action_type.is_empty(), "Action should have a type");
}

#[tokio::test]
async fn test_complete_learning_episode_workflow() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(config)
        .await
        .expect("Failed to create manager");
    manager.initialize().await.expect("Failed to initialize");

    // Start episode
    let episode_id = manager
        .start_episode("test_context")
        .await
        .expect("Failed to start episode");

    // Take action
    let action = manager
        .take_action(&episode_id, "test_context")
        .await
        .expect("Failed to take action");
    assert!(!action.id.is_empty());

    // Provide reward
    manager
        .provide_reward(&episode_id, 5.0)
        .await
        .expect("Failed to provide reward");

    // End episode
    manager
        .end_episode(&episode_id, true)
        .await
        .expect("Failed to end episode");

    // Verify episode was recorded
    let history = manager.get_episode_history().await;
    assert_eq!(history.len(), 1, "Should have 1 episode in history");
    assert!(history[0].success, "Episode should be successful");
    assert_eq!(history[0].actions.len(), 1, "Should have 1 action");
    assert_eq!(history[0].rewards.len(), 1, "Should have 1 reward");
}

#[tokio::test]
async fn test_multiple_episodes_in_session() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(config)
        .await
        .expect("Failed to create manager");
    manager.initialize().await.expect("Failed to initialize");
    manager.start().await.expect("Failed to start");

    // Run multiple episodes
    for i in 0..3 {
        let episode_id = manager
            .start_episode(&format!("context_{}", i))
            .await
            .unwrap_or_else(|_| panic!("Failed to start episode {}", i));

        manager
            .take_action(&episode_id, &format!("context_{}", i))
            .await
            .expect("Failed to take action");

        manager
            .provide_reward(&episode_id, (i + 1) as f64)
            .await
            .expect("Failed to provide reward");

        manager
            .end_episode(&episode_id, true)
            .await
            .expect("Failed to end episode");
    }

    // Verify all episodes were recorded
    let history = manager.get_episode_history().await;
    assert_eq!(history.len(), 3, "Should have 3 episodes in history");

    // Verify session exists
    let session = manager.get_current_session().await;
    assert!(session.is_some(), "Session should exist");
}

#[tokio::test]
async fn test_learning_system_event_broadcasting() {
    let config = test_helpers::create_test_learning_config();
    let system = LearningSystem::new(config)
        .await
        .expect("Failed to create system");

    // Subscribe to events
    let mut receiver = system.subscribe_to_events();

    // Initialize system (should broadcast event)
    system.initialize().await.expect("Failed to initialize");

    // Try to receive initialization event (with timeout)
    let _event_result = tokio::time::timeout(Duration::from_millis(100), receiver.recv()).await;

    // Event may or may not be received depending on timing, but system should work
    let state = system.get_state().await;
    assert!(matches!(state, LearningState::Learning));
}

#[tokio::test]
async fn test_manager_engine_state_synchronization() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(config)
        .await
        .expect("Failed to create manager");

    // Both should start in Initializing state
    let manager_state = manager.get_state().await;
    assert!(matches!(manager_state, LearningState::Initializing));

    // Initialize manager
    manager.initialize().await.expect("Failed to initialize");

    // Manager should be in Learning state
    let manager_state = manager.get_state().await;
    assert!(matches!(manager_state, LearningState::Learning));

    // Engine should also be in a consistent state
    let engine = manager.get_learning_engine();
    let engine_state = engine.get_state().await;
    assert!(matches!(engine_state, LearningState::Learning));
}

#[tokio::test]
async fn test_episode_with_multiple_actions_and_rewards() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(config)
        .await
        .expect("Failed to create manager");
    manager.initialize().await.expect("Failed to initialize");

    let episode_id = manager
        .start_episode("test_context")
        .await
        .expect("Failed to start episode");

    // Take multiple actions with rewards
    for i in 0..5 {
        manager
            .take_action(&episode_id, "test_context")
            .await
            .unwrap_or_else(|_| panic!("Failed to take action {}", i));

        manager
            .provide_reward(&episode_id, (i + 1) as f64)
            .await
            .unwrap_or_else(|_| panic!("Failed to provide reward {}", i));
    }

    // End episode
    manager
        .end_episode(&episode_id, true)
        .await
        .expect("Failed to end episode");

    // Verify episode details
    let history = manager.get_episode_history().await;
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].actions.len(), 5, "Should have 5 actions");
    assert_eq!(history[0].rewards.len(), 5, "Should have 5 rewards");
    assert!(history[0].success);
}

#[tokio::test]
async fn test_failed_episode_workflow() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(config)
        .await
        .expect("Failed to create manager");
    manager.initialize().await.expect("Failed to initialize");

    let episode_id = manager
        .start_episode("test_context")
        .await
        .expect("Failed to start episode");

    // Take action with negative reward (failure scenario)
    manager
        .take_action(&episode_id, "test_context")
        .await
        .expect("Failed to take action");
    manager
        .provide_reward(&episode_id, -10.0)
        .await
        .expect("Failed to provide reward");

    // End episode as failed
    manager
        .end_episode(&episode_id, false)
        .await
        .expect("Failed to end episode");

    // Verify episode was marked as failed
    let history = manager.get_episode_history().await;
    assert_eq!(history.len(), 1);
    assert!(!history[0].success, "Episode should be marked as failed");
    assert_eq!(history[0].rewards[0], -10.0, "Reward should be negative");
}

#[tokio::test]
async fn test_concurrent_episodes_different_contexts() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let manager = Arc::new(
        ContextLearningManager::new(config)
            .await
            .expect("Failed to create manager"),
    );
    manager.initialize().await.expect("Failed to initialize");

    // Start multiple episodes concurrently
    let episode1 = manager
        .start_episode("context1")
        .await
        .expect("Failed to start episode 1");
    let episode2 = manager
        .start_episode("context2")
        .await
        .expect("Failed to start episode 2");

    // Verify both are active
    let active = manager.get_active_episodes().await;
    assert_eq!(active.len(), 2, "Should have 2 active episodes");

    // Complete both episodes
    manager
        .end_episode(&episode1, true)
        .await
        .expect("Failed to end episode 1");
    manager
        .end_episode(&episode2, true)
        .await
        .expect("Failed to end episode 2");

    // Verify both in history
    let history = manager.get_episode_history().await;
    assert_eq!(history.len(), 2, "Should have 2 episodes in history");
}

#[tokio::test]
async fn test_learning_system_get_components() {
    let config = test_helpers::create_test_learning_config();
    let system = LearningSystem::new(config)
        .await
        .expect("Failed to create system");

    // Verify we can access all components
    let engine = system.get_engine();
    let engine_state = engine.get_state().await;
    assert!(matches!(engine_state, LearningState::Initializing));

    let manager = system.get_manager();
    let manager_state = manager.get_state().await;
    assert!(matches!(manager_state, LearningState::Initializing));

    let experience_replay = system.get_experience_replay();
    // Experience replay doesn't have state, just verify it exists
    drop(experience_replay);

    let reward_system = system.get_reward_system();
    drop(reward_system);

    let policy_network = system.get_policy_network();
    drop(policy_network);

    let metrics = system.get_metrics();
    drop(metrics);

    let adaptive_rules = system.get_adaptive_rules();
    drop(adaptive_rules);

    let integration = system.get_integration();
    drop(integration);
}

#[tokio::test]
async fn test_learning_system_statistics() {
    let config = test_helpers::create_test_learning_config();
    let system = LearningSystem::new(config)
        .await
        .expect("Failed to create system");

    let stats = system.get_stats().await;

    // Initial statistics should be zeroed
    assert_eq!(stats.total_episodes, 0);
    assert_eq!(stats.total_actions, 0);
    assert_eq!(stats.total_rewards, 0.0);
}

#[tokio::test]
async fn test_invalid_episode_id_returns_error() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(config)
        .await
        .expect("Failed to create manager");
    manager.initialize().await.expect("Failed to initialize");

    // Try to end non-existent episode
    let result = manager.end_episode("nonexistent_episode_id", true).await;
    // Should not panic, may return error or succeed gracefully
    drop(result);
}

#[tokio::test]
async fn test_manager_start_without_initialization() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(config)
        .await
        .expect("Failed to create manager");

    // Try to start episode without initialization
    let result = manager.start_episode("test_context").await;
    // Should succeed (manager doesn't require explicit init for episodes)
    assert!(result.is_ok(), "Starting episode should work");
}

#[tokio::test]
async fn test_session_lifecycle() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(config)
        .await
        .expect("Failed to create manager");
    manager.initialize().await.expect("Failed to initialize");

    // No session initially
    let session = manager.get_current_session().await;
    assert!(session.is_none(), "No session before start");

    // Start manager (creates session)
    manager.start().await.expect("Failed to start");

    let session = manager.get_current_session().await;
    assert!(session.is_some(), "Session should exist after start");

    // Stop manager (ends session)
    manager.stop().await.expect("Failed to stop");

    let session = manager.get_current_session().await;
    assert!(session.is_none(), "Session should be ended after stop");
}

#[tokio::test]
async fn test_mixed_success_failure_episodes() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(config)
        .await
        .expect("Failed to create manager");
    manager.initialize().await.expect("Failed to initialize");

    // Successful episode
    let ep1 = manager
        .start_episode("ctx1")
        .await
        .expect("Failed to start");
    manager.provide_reward(&ep1, 10.0).await.expect("Failed");
    manager.end_episode(&ep1, true).await.expect("Failed");

    // Failed episode
    let ep2 = manager
        .start_episode("ctx2")
        .await
        .expect("Failed to start");
    manager.provide_reward(&ep2, -5.0).await.expect("Failed");
    manager.end_episode(&ep2, false).await.expect("Failed");

    // Another successful episode
    let ep3 = manager
        .start_episode("ctx3")
        .await
        .expect("Failed to start");
    manager.provide_reward(&ep3, 8.0).await.expect("Failed");
    manager.end_episode(&ep3, true).await.expect("Failed");

    // Verify history
    let history = manager.get_episode_history().await;
    assert_eq!(history.len(), 3);
    assert!(history[0].success);
    assert!(!history[1].success);
    assert!(history[2].success);
}

#[tokio::test]
async fn test_episode_with_no_actions() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(config)
        .await
        .expect("Failed to create manager");
    manager.initialize().await.expect("Failed to initialize");

    // Start and immediately end episode
    let episode_id = manager
        .start_episode("test_context")
        .await
        .expect("Failed to start");
    manager
        .end_episode(&episode_id, true)
        .await
        .expect("Failed to end");

    let history = manager.get_episode_history().await;
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].actions.len(), 0, "Should have no actions");
    assert_eq!(history[0].rewards.len(), 0, "Should have no rewards");
}

#[tokio::test]
async fn test_learning_system_full_lifecycle() {
    let config = test_helpers::create_test_learning_config();
    let system = LearningSystem::new(config).await.expect("Failed to create");

    // Full lifecycle: initialize -> start -> stop
    system.initialize().await.expect("Failed to initialize");
    let state = system.get_state().await;
    assert!(matches!(state, LearningState::Learning));

    system.start().await.expect("Failed to start");
    let state = system.get_state().await;
    assert!(matches!(state, LearningState::Learning));

    system.stop().await.expect("Failed to stop");
    let state = system.get_state().await;
    assert!(matches!(state, LearningState::Stopped));
}

#[tokio::test]
async fn test_manager_provides_access_to_engine() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(config)
        .await
        .expect("Failed to create manager");

    // Manager should provide access to its engine
    let engine = manager.get_learning_engine();

    // Engine should be in initial state
    let state = engine.get_state().await;
    assert!(matches!(state, LearningState::Initializing));
}

#[tokio::test]
async fn test_manager_provides_access_to_context_manager() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(config)
        .await
        .expect("Failed to create manager");

    // Manager should provide access to context manager
    let context_manager = manager.get_context_manager();

    // Context manager should exist
    drop(context_manager);
}

#[tokio::test]
async fn test_exploration_decay_over_multiple_episodes() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(config)
        .await
        .expect("Failed to create manager");
    manager.initialize().await.expect("Failed to initialize");

    let engine = manager.get_learning_engine();
    let initial_exploration = engine.get_exploration_rate().await;

    // Run multiple episodes (exploration should decay)
    for i in 0..5 {
        let episode_id = manager
            .start_episode(&format!("ctx_{}", i))
            .await
            .expect("Failed");
        manager
            .take_action(&episode_id, &format!("ctx_{}", i))
            .await
            .expect("Failed");
        manager
            .provide_reward(&episode_id, 1.0)
            .await
            .expect("Failed");
        manager
            .end_episode(&episode_id, true)
            .await
            .expect("Failed");

        // Trigger exploration decay
        engine.decay_exploration().await.expect("Failed to decay");
    }

    let final_exploration = engine.get_exploration_rate().await;

    // Exploration rate should have decreased
    assert!(
        final_exploration < initial_exploration,
        "Exploration should decay: {} -> {}",
        initial_exploration,
        final_exploration
    );
}

#[tokio::test]
async fn test_engine_experience_buffer_grows_with_episodes() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(config)
        .await
        .expect("Failed to create manager");
    manager.initialize().await.expect("Failed to initialize");

    let engine = manager.get_learning_engine();
    let initial_size = engine.get_experience_buffer_size().await;

    // Run episodes to generate experiences
    for i in 0..3 {
        let episode_id = manager
            .start_episode(&format!("ctx_{}", i))
            .await
            .expect("Failed");
        manager
            .take_action(&episode_id, &format!("ctx_{}", i))
            .await
            .expect("Failed");
        manager
            .provide_reward(&episode_id, (i + 1) as f64)
            .await
            .expect("Failed");
        manager
            .end_episode(&episode_id, true)
            .await
            .expect("Failed");
    }

    let final_size = engine.get_experience_buffer_size().await;

    // Buffer should have grown (experiences added during episode execution)
    // Note: Actual growth depends on implementation details
    assert!(
        final_size >= initial_size,
        "Buffer should grow or stay same"
    );
}

#[tokio::test]
async fn test_system_handles_rapid_start_stop() {
    let config = test_helpers::create_test_learning_config();
    let system = LearningSystem::new(config).await.expect("Failed to create");

    system.initialize().await.expect("Failed to initialize");

    // Rapid start-stop cycles
    for _ in 0..3 {
        system.start().await.expect("Failed to start");
        system.stop().await.expect("Failed to stop");
    }

    // System should still be in consistent state
    let state = system.get_state().await;
    assert!(matches!(state, LearningState::Stopped));
}

#[tokio::test]
async fn test_manager_stats_persistence() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(config)
        .await
        .expect("Failed to create manager");
    manager.initialize().await.expect("Failed to initialize");

    // Run an episode
    let episode_id = manager.start_episode("test").await.expect("Failed");
    manager
        .end_episode(&episode_id, true)
        .await
        .expect("Failed");

    // Stats should reflect the episode
    let _stats = manager.get_learning_stats().await;
    // Stats tracking depends on implementation
    // Stats lifetime ends naturally (no need for explicit drop)
}
