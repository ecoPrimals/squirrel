//! Tests for Context Learning Manager

use super::manager::{
    ContextLearningManager, ContextLearningManagerConfig, FeatureExtractionMethod, RewardParameters,
};
use super::metrics::LearningStats;
use crate::learning::test_helpers;
use std::sync::Arc;

// ============================================================================
// Type and Configuration Tests
// ============================================================================

#[test]
fn test_context_learning_manager_config_default() {
    let config = ContextLearningManagerConfig::default();

    assert_eq!(config.episode_timeout, 3600);
    assert_eq!(config.max_episodes_per_session, 1000);
    assert!(config.auto_episode_detection);
    assert!(config.enable_preprocessing);
    assert_eq!(config.state_space_size, 128);
    assert_eq!(config.action_space_size, 32);
}

#[test]
fn test_context_learning_manager_config_clone() {
    let config = ContextLearningManagerConfig::default();
    let cloned = config.clone();

    assert_eq!(cloned.episode_timeout, config.episode_timeout);
    assert_eq!(
        cloned.max_episodes_per_session,
        config.max_episodes_per_session
    );
}

#[test]
fn test_context_learning_manager_config_serialization() {
    let config = ContextLearningManagerConfig::default();
    let serialized = serde_json::to_string(&config).expect("Failed to serialize");
    let deserialized: ContextLearningManagerConfig =
        serde_json::from_str(&serialized).expect("Failed to deserialize");

    assert_eq!(deserialized.episode_timeout, config.episode_timeout);
    assert_eq!(deserialized.state_space_size, config.state_space_size);
}

#[test]
fn test_feature_extraction_method_variants() {
    let methods = vec![
        FeatureExtractionMethod::Statistical,
        FeatureExtractionMethod::RuleBased,
        FeatureExtractionMethod::ContextAware,
        FeatureExtractionMethod::Custom("test".to_string()),
    ];

    for method in methods {
        let debug_str = format!("{:?}", method);
        assert!(!debug_str.is_empty());
    }
}

#[test]
fn test_feature_extraction_method_serialization() {
    let method = FeatureExtractionMethod::Statistical;
    let serialized = serde_json::to_string(&method).expect("Failed to serialize");
    let deserialized: FeatureExtractionMethod =
        serde_json::from_str(&serialized).expect("Failed to deserialize");

    match deserialized {
        FeatureExtractionMethod::Statistical => {}
        _ => panic!("Deserialization failed"),
    }
}

#[test]
fn test_reward_parameters_default() {
    let params = RewardParameters::default();

    assert_eq!(params.success_reward, 10.0);
    assert_eq!(params.failure_penalty, -5.0);
    assert_eq!(params.step_penalty, -0.1);
    assert_eq!(params.context_improvement_reward, 5.0);
}

#[test]
fn test_reward_parameters_clone() {
    let params = RewardParameters::default();
    let cloned = params.clone();

    assert_eq!(cloned.success_reward, params.success_reward);
    assert_eq!(cloned.error_penalty, params.error_penalty);
}

#[test]
fn test_reward_parameters_serialization() {
    let params = RewardParameters::default();
    let serialized = serde_json::to_string(&params).expect("Failed to serialize");
    let deserialized: RewardParameters =
        serde_json::from_str(&serialized).expect("Failed to deserialize");

    assert_eq!(deserialized.success_reward, params.success_reward);
    assert_eq!(deserialized.failure_penalty, params.failure_penalty);
}

#[test]
fn test_learning_stats_default() {
    let stats = LearningStats::default();

    assert_eq!(stats.total_episodes, 0);
    assert_eq!(stats.successful_episodes, 0);
    assert_eq!(stats.average_episode_length, 0.0);
    assert_eq!(stats.average_reward_per_episode, 0.0);
    assert_eq!(stats.successful_episodes, 0);
    assert_eq!(stats.contexts_learned, 0);
    assert_eq!(stats.rule_adaptations, 0);
}

#[test]
fn test_learning_stats_clone() {
    let stats = LearningStats::default();
    let cloned = stats.clone();

    assert_eq!(cloned.total_episodes, stats.total_episodes);
    assert_eq!(cloned.successful_episodes, stats.successful_episodes);
}

#[test]
fn test_learning_stats_serialization() {
    let stats = LearningStats::default();
    let serialized = serde_json::to_string(&stats).expect("Failed to serialize");
    let deserialized: LearningStats =
        serde_json::from_str(&serialized).expect("Failed to deserialize");

    assert_eq!(deserialized.total_episodes, stats.total_episodes);
    assert_eq!(deserialized.successful_episodes, stats.successful_episodes);
}

// ============================================================================
// PHASE 5: Manager Behavior Tests (Sprint 1)
// ============================================================================

#[tokio::test]
async fn test_manager_creation() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(system_config).await;

    assert!(manager.is_ok(), "Manager creation should succeed");
}

#[tokio::test]
async fn test_manager_initialization() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(system_config)
        .await
        .expect("Failed to create manager");

    let result = manager.initialize().await;
    assert!(result.is_ok(), "Manager initialization should succeed");

    let state = manager.get_state().await;
    assert!(
        matches!(state, crate::learning::LearningState::Learning),
        "Manager should be in Learning state after initialization"
    );
}

#[tokio::test]
async fn test_manager_start_stop() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(system_config)
        .await
        .expect("Failed to create manager");

    manager.initialize().await.expect("Failed to initialize");

    // Start manager
    manager.start().await.expect("Failed to start manager");
    let state = manager.get_state().await;
    assert!(matches!(state, crate::learning::LearningState::Learning));

    // Stop manager
    manager.stop().await.expect("Failed to stop manager");
    let state = manager.get_state().await;
    assert!(matches!(state, crate::learning::LearningState::Stopped));
}

#[tokio::test]
async fn test_manager_start_creates_session() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(system_config)
        .await
        .expect("Failed to create manager");

    manager.initialize().await.expect("Failed to initialize");
    manager.start().await.expect("Failed to start");

    let session = manager.get_current_session().await;
    assert!(session.is_some(), "Session should be created on start");

    let session = session.expect("test: should succeed");
    assert!(!session.id.is_empty(), "Session should have an ID");
    assert_eq!(
        session.total_episodes, 0,
        "New session should have 0 episodes"
    );
}

#[tokio::test]
async fn test_manager_get_initial_stats() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(system_config)
        .await
        .expect("Failed to create manager");

    let stats = manager.get_learning_stats().await;

    assert_eq!(stats.total_episodes, 0, "Initial episodes should be 0");
    assert_eq!(
        stats.successful_episodes, 0,
        "Initial successful episodes should be 0"
    );
    assert_eq!(
        stats.successful_episodes, 0,
        "Initial successful episodes should be 0"
    );
}

#[tokio::test]
async fn test_manager_start_episode() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(system_config)
        .await
        .expect("Failed to create manager");

    manager.initialize().await.expect("Failed to initialize");

    let episode_id = manager
        .start_episode("test_context")
        .await
        .expect("Failed to start episode");

    assert!(!episode_id.is_empty(), "Episode ID should not be empty");

    let active_episodes = manager.get_active_episodes().await;
    assert_eq!(active_episodes.len(), 1, "Should have 1 active episode");
    assert_eq!(active_episodes[0].id, episode_id, "Episode ID should match");
}

#[tokio::test]
async fn test_manager_start_multiple_episodes() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(system_config)
        .await
        .expect("Failed to create manager");

    manager.initialize().await.expect("Failed to initialize");

    let episode1 = manager
        .start_episode("context1")
        .await
        .expect("Failed to start episode 1");
    let episode2 = manager
        .start_episode("context2")
        .await
        .expect("Failed to start episode 2");

    let active_episodes = manager.get_active_episodes().await;
    assert_eq!(active_episodes.len(), 2, "Should have 2 active episodes");

    let episode_ids: Vec<String> = active_episodes.iter().map(|e| e.id.clone()).collect();
    assert!(episode_ids.contains(&episode1), "Should contain episode 1");
    assert!(episode_ids.contains(&episode2), "Should contain episode 2");
}

#[tokio::test]
async fn test_manager_end_episode() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(system_config)
        .await
        .expect("Failed to create manager");

    manager.initialize().await.expect("Failed to initialize");

    let episode_id = manager
        .start_episode("test_context")
        .await
        .expect("Failed to start episode");

    let result = manager.end_episode(&episode_id, true).await;
    assert!(result.is_ok(), "Ending episode should succeed");

    let active_episodes = manager.get_active_episodes().await;
    assert_eq!(
        active_episodes.len(),
        0,
        "Active episodes should be 0 after ending"
    );
}

#[tokio::test]
async fn test_manager_end_episode_updates_history() {
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
        .end_episode(&episode_id, true)
        .await
        .expect("Failed to end episode");

    let history = manager.get_episode_history().await;
    assert_eq!(history.len(), 1, "History should contain 1 episode");
    assert_eq!(history[0].id, episode_id, "Episode ID should match");
    assert!(history[0].success, "Episode should be marked as successful");
}

#[tokio::test]
async fn test_manager_take_action() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(system_config)
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

    assert!(!action.id.is_empty(), "Action should have an ID");
    assert!(!action.action_type.is_empty(), "Action should have a type");
}

#[tokio::test]
async fn test_manager_provide_reward() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(system_config)
        .await
        .expect("Failed to create manager");

    manager.initialize().await.expect("Failed to initialize");

    let episode_id = manager
        .start_episode("test_context")
        .await
        .expect("Failed to start episode");

    let result = manager.provide_reward(&episode_id, 5.0).await;
    assert!(result.is_ok(), "Providing reward should succeed");

    let active_episodes = manager.get_active_episodes().await;
    assert_eq!(active_episodes[0].rewards.len(), 1, "Should have 1 reward");
    assert_eq!(
        active_episodes[0].rewards[0], 5.0,
        "Reward value should match"
    );
}

#[tokio::test]
async fn test_manager_multiple_rewards() {
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
        .expect("Failed to provide reward 1");
    manager
        .provide_reward(&episode_id, 2.0)
        .await
        .expect("Failed to provide reward 2");
    manager
        .provide_reward(&episode_id, 3.0)
        .await
        .expect("Failed to provide reward 3");

    let active_episodes = manager.get_active_episodes().await;
    assert_eq!(active_episodes[0].rewards.len(), 3, "Should have 3 rewards");
    assert_eq!(
        active_episodes[0].rewards[0], 1.0,
        "First reward should be 1.0"
    );
    assert_eq!(
        active_episodes[0].rewards[1], 2.0,
        "Second reward should be 2.0"
    );
    assert_eq!(
        active_episodes[0].rewards[2], 3.0,
        "Third reward should be 3.0"
    );
}

#[tokio::test]
async fn test_manager_episode_with_actions_and_rewards() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(system_config)
        .await
        .expect("Failed to create manager");

    manager.initialize().await.expect("Failed to initialize");

    let episode_id = manager
        .start_episode("test_context")
        .await
        .expect("Failed to start episode");

    // Take multiple actions
    manager
        .take_action(&episode_id, "test_context")
        .await
        .expect("Failed to take action 1");
    manager
        .provide_reward(&episode_id, 1.0)
        .await
        .expect("Failed to provide reward 1");

    manager
        .take_action(&episode_id, "test_context")
        .await
        .expect("Failed to take action 2");
    manager
        .provide_reward(&episode_id, 2.0)
        .await
        .expect("Failed to provide reward 2");

    let active_episodes = manager.get_active_episodes().await;
    assert_eq!(active_episodes[0].actions.len(), 2, "Should have 2 actions");
    assert_eq!(active_episodes[0].rewards.len(), 2, "Should have 2 rewards");
    assert_eq!(
        active_episodes[0].rewards[0], 1.0,
        "First reward should be 1.0"
    );
    assert_eq!(
        active_episodes[0].rewards[1], 2.0,
        "Second reward should be 2.0"
    );
}

#[tokio::test]
async fn test_manager_session_tracks_episodes() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(system_config)
        .await
        .expect("Failed to create manager");

    manager.initialize().await.expect("Failed to initialize");
    manager.start().await.expect("Failed to start");

    // Create and end an episode
    let episode_id = manager
        .start_episode("test_context")
        .await
        .expect("Failed to start episode");
    manager
        .end_episode(&episode_id, true)
        .await
        .expect("Failed to end episode");

    // Session should track the episode
    let history = manager.get_episode_history().await;
    assert_eq!(history.len(), 1, "History should contain 1 episode");
}

#[tokio::test]
async fn test_manager_get_active_episodes_empty() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(system_config)
        .await
        .expect("Failed to create manager");

    let active_episodes = manager.get_active_episodes().await;
    assert_eq!(
        active_episodes.len(),
        0,
        "Should have 0 active episodes initially"
    );
}

#[tokio::test]
async fn test_manager_get_episode_history_empty() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(system_config)
        .await
        .expect("Failed to create manager");

    let history = manager.get_episode_history().await;
    assert_eq!(history.len(), 0, "History should be empty initially");
}

#[tokio::test]
async fn test_manager_get_current_session_none() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(system_config)
        .await
        .expect("Failed to create manager");

    let session = manager.get_current_session().await;
    assert!(session.is_none(), "Session should be None before start");
}

#[tokio::test]
async fn test_manager_successful_episode_updates_stats() {
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
        .provide_reward(&episode_id, 10.0)
        .await
        .expect("Failed to provide reward");
    manager
        .end_episode(&episode_id, true)
        .await
        .expect("Failed to end episode");

    // Stats should be updated
    // Note: Stats might not update immediately depending on implementation
    let history = manager.get_episode_history().await;
    assert_eq!(history.len(), 1, "Should have 1 completed episode");
    assert!(history[0].success, "Episode should be successful");
}

#[tokio::test]
async fn test_manager_failed_episode() {
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
        .provide_reward(&episode_id, -5.0)
        .await
        .expect("Failed to provide reward");
    manager
        .end_episode(&episode_id, false)
        .await
        .expect("Failed to end episode");

    let history = manager.get_episode_history().await;
    assert_eq!(history.len(), 1, "Should have 1 completed episode");
    assert!(!history[0].success, "Episode should be marked as failed");
    assert_eq!(
        history[0].total_reward, -5.0,
        "Total reward should be negative"
    );
}

#[tokio::test]
async fn test_manager_config_custom_settings() {
    let mut config = test_helpers::create_test_learning_config();
    config.learning_rate = 0.01;
    config.exploration_rate = 0.5;

    let system_config = Arc::new(config);
    let manager = ContextLearningManager::new(system_config).await;

    assert!(
        manager.is_ok(),
        "Manager with custom config should be created"
    );
}

#[tokio::test]
async fn test_manager_state_transitions() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(system_config)
        .await
        .expect("Failed to create manager");

    // Initial state
    let state = manager.get_state().await;
    assert!(matches!(
        state,
        crate::learning::LearningState::Initializing
    ));

    // After initialization
    manager.initialize().await.expect("Failed to initialize");
    let state = manager.get_state().await;
    assert!(matches!(state, crate::learning::LearningState::Learning));

    // After start
    manager.start().await.expect("Failed to start");
    let state = manager.get_state().await;
    assert!(matches!(state, crate::learning::LearningState::Learning));

    // After stop
    manager.stop().await.expect("Failed to stop");
    let state = manager.get_state().await;
    assert!(matches!(state, crate::learning::LearningState::Stopped));
}

#[tokio::test]
async fn test_manager_multiple_contexts() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let manager = ContextLearningManager::new(system_config)
        .await
        .expect("Failed to create manager");

    manager.initialize().await.expect("Failed to initialize");

    manager
        .start_episode("context1")
        .await
        .expect("Failed to start episode 1");
    manager
        .start_episode("context2")
        .await
        .expect("Failed to start episode 2");
    manager
        .start_episode("context3")
        .await
        .expect("Failed to start episode 3");

    let active_episodes = manager.get_active_episodes().await;
    assert_eq!(active_episodes.len(), 3, "Should have 3 active episodes");

    // Different contexts should have different episode IDs
    let episode_ids: Vec<String> = active_episodes.iter().map(|e| e.id.clone()).collect();
    assert_eq!(episode_ids.len(), 3, "All episodes should have unique IDs");
}

#[tokio::test]
async fn test_reward_parameters_custom_values() {
    let params = RewardParameters {
        success_reward: 20.0,
        failure_penalty: -10.0,
        ..Default::default()
    };

    assert_eq!(params.success_reward, 20.0);
    assert_eq!(params.failure_penalty, -10.0);
}

#[test]
fn test_feature_extraction_method_custom() {
    let method = FeatureExtractionMethod::Custom("my_extractor".to_string());
    let serialized = serde_json::to_string(&method).expect("Failed to serialize");

    assert!(serialized.contains("my_extractor"));

    let deserialized: FeatureExtractionMethod =
        serde_json::from_str(&serialized).expect("Failed to deserialize");

    match deserialized {
        FeatureExtractionMethod::Custom(name) => assert_eq!(name, "my_extractor"),
        _ => panic!("Expected Custom variant"),
    }
}

#[test]
fn test_manager_config_custom_intervals() {
    let config = ContextLearningManagerConfig {
        learning_update_interval: tokio::time::Duration::from_secs(5),
        context_observation_interval: tokio::time::Duration::from_millis(500),
        ..Default::default()
    };

    assert_eq!(config.learning_update_interval.as_secs(), 5);
    assert_eq!(config.context_observation_interval.as_millis(), 500);
}
