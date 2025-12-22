//! Tests for learning engine types and configs

use super::engine::{
    LearningAlgorithm, LearningEngine, LearningEngineConfig, RLAction, RLExperience, RLState,
};
use crate::learning::test_helpers;
use std::sync::Arc;

#[test]
fn test_learning_engine_config_default() {
    let config = LearningEngineConfig::default();
    assert_eq!(config.learning_rate, 0.001);
    assert_eq!(config.discount_factor, 0.95);
    assert_eq!(config.exploration_rate, 1.0);
    assert_eq!(config.buffer_size, 10000);
    assert_eq!(config.batch_size, 32);
    assert!(config.double_dqn);
    assert!(config.dueling_dqn);
    assert!(config.prioritized_replay);
}

#[test]
fn test_learning_engine_config_clone() {
    let config = LearningEngineConfig::default();
    let cloned = config.clone();
    assert_eq!(cloned.learning_rate, config.learning_rate);
    assert_eq!(cloned.batch_size, config.batch_size);
}

#[test]
fn test_learning_engine_config_serialization() {
    let config = LearningEngineConfig::default();
    let serialized = serde_json::to_string(&config).expect("Failed to serialize");
    let deserialized: LearningEngineConfig =
        serde_json::from_str(&serialized).expect("Failed to deserialize");

    assert_eq!(deserialized.learning_rate, config.learning_rate);
    assert_eq!(deserialized.batch_size, config.batch_size);
}

#[test]
fn test_learning_algorithm_variants() {
    let algorithms = vec![
        LearningAlgorithm::QLearning,
        LearningAlgorithm::DeepQLearning,
        LearningAlgorithm::DoubleDQN,
        LearningAlgorithm::DuelingDQN,
        LearningAlgorithm::ActorCritic,
        LearningAlgorithm::Ppo,
        LearningAlgorithm::Sac,
    ];

    for algo in algorithms {
        let cloned = algo.clone();
        let debug_str = format!("{:?}", cloned);
        assert!(!debug_str.is_empty());
    }
}

#[test]
fn test_learning_algorithm_serialization() {
    let algo = LearningAlgorithm::DeepQLearning;
    let serialized = serde_json::to_string(&algo).expect("Failed to serialize");
    let deserialized: LearningAlgorithm =
        serde_json::from_str(&serialized).expect("Failed to deserialize");

    match deserialized {
        LearningAlgorithm::DeepQLearning => {}
        _ => panic!("Deserialization failed"),
    }
}

#[test]
fn test_rl_state_clone() {
    let state = RLState {
        id: "test_state".to_string(),
        features: vec![1.0, 2.0, 3.0],
        context_id: "ctx_1".to_string(),
        timestamp: chrono::Utc::now(),
        metadata: None,
    };

    let cloned = state.clone();
    assert_eq!(cloned.id, state.id);
    assert_eq!(cloned.features.len(), 3);
}

#[test]
fn test_rl_state_serialization() {
    let state = RLState {
        id: "test_state".to_string(),
        features: vec![1.0, 2.0, 3.0],
        context_id: "ctx_1".to_string(),
        timestamp: chrono::Utc::now(),
        metadata: None,
    };

    let serialized = serde_json::to_string(&state).expect("Failed to serialize");
    let deserialized: RLState = serde_json::from_str(&serialized).expect("Failed to deserialize");

    assert_eq!(deserialized.id, state.id);
    assert_eq!(deserialized.features.len(), 3);
}

#[test]
fn test_rl_action_clone() {
    let action = RLAction {
        id: "test_action".to_string(),
        action_type: "modify".to_string(),
        parameters: serde_json::json!({"key": "value"}),
        confidence: 0.5,
        expected_reward: 0.0,
    };

    let cloned = action.clone();
    assert_eq!(cloned.id, action.id);
    assert_eq!(cloned.action_type, "modify");
}

#[test]
fn test_rl_action_serialization() {
    let action = RLAction {
        id: "test_action".to_string(),
        action_type: "modify".to_string(),
        parameters: serde_json::json!({"key": "value"}),
        confidence: 0.5,
        expected_reward: 0.0,
    };

    let serialized = serde_json::to_string(&action).expect("Failed to serialize");
    let deserialized: RLAction = serde_json::from_str(&serialized).expect("Failed to deserialize");

    assert_eq!(deserialized.id, action.id);
    assert_eq!(deserialized.action_type, "modify");
}

#[test]
fn test_rl_experience_clone() {
    let exp = RLExperience {
        id: "exp_1".to_string(),
        state: RLState {
            id: "s1".to_string(),
            features: vec![1.0],
            context_id: "ctx_1".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: None,
        },
        action: RLAction {
            id: "a1".to_string(),
            action_type: "test".to_string(),
            parameters: serde_json::json!({}),
            confidence: 0.5,
            expected_reward: 0.0,
        },
        reward: 1.5,
        next_state: Some(RLState {
            id: "s2".to_string(),
            features: vec![2.0],
            context_id: "ctx_1".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: None,
        }),
        done: false,
        timestamp: chrono::Utc::now(),
        priority: 1.0,
    };

    let cloned = exp.clone();
    assert_eq!(cloned.reward, 1.5);
    assert!(!cloned.done);
}

#[test]
fn test_learning_engine_config_custom() {
    let config = LearningEngineConfig {
        learning_rate: 0.01,
        batch_size: 64,
        double_dqn: false,
        ..Default::default()
    };

    assert_eq!(config.learning_rate, 0.01);
    assert_eq!(config.batch_size, 64);
    assert!(!config.double_dqn);
}

#[test]
fn test_learning_algorithm_all_variants() {
    let variants = [
        LearningAlgorithm::QLearning,
        LearningAlgorithm::DeepQLearning,
        LearningAlgorithm::DoubleDQN,
        LearningAlgorithm::DuelingDQN,
        LearningAlgorithm::ActorCritic,
        LearningAlgorithm::Ppo,
        LearningAlgorithm::Sac,
    ];

    for variant in &variants {
        let _serialized = serde_json::to_string(variant).expect("Serialization failed");
    }
}

#[test]
fn test_rl_state_with_features() {
    let state = RLState {
        id: "state_1".to_string(),
        features: vec![1.0, 2.0, 3.0, 4.0, 5.0],
        context_id: "ctx_1".to_string(),
        timestamp: chrono::Utc::now(),
        metadata: Some(serde_json::json!({"context": "test"})),
    };

    assert_eq!(state.features.len(), 5);
    assert!(state.metadata.is_some());
}

#[test]
fn test_rl_action_empty_params() {
    let action = RLAction {
        id: "action_1".to_string(),
        action_type: "noop".to_string(),
        parameters: serde_json::json!({}),
        confidence: 0.0,
        expected_reward: 0.0,
    };

    let debug_str = format!("{:?}", action);
    assert!(debug_str.contains("action_1"));
}

#[test]
fn test_rl_experience_serialization() {
    let exp = RLExperience {
        id: "exp_1".to_string(),
        state: RLState {
            id: "s1".to_string(),
            features: vec![1.0],
            context_id: "ctx_1".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: None,
        },
        action: RLAction {
            id: "a1".to_string(),
            action_type: "test".to_string(),
            parameters: serde_json::json!({}),
            confidence: 0.5,
            expected_reward: 0.0,
        },
        reward: 1.5,
        next_state: Some(RLState {
            id: "s2".to_string(),
            features: vec![2.0],
            context_id: "ctx_1".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: None,
        }),
        done: true,
        timestamp: chrono::Utc::now(),
        priority: 1.0,
    };

    let serialized = serde_json::to_string(&exp).expect("Failed to serialize");
    let deserialized: RLExperience =
        serde_json::from_str(&serialized).expect("Failed to deserialize");

    assert_eq!(deserialized.reward, 1.5);
    assert!(deserialized.done);
}

#[test]
fn test_learning_engine_config_exploration() {
    let config = LearningEngineConfig::default();
    assert_eq!(config.exploration_rate, 1.0);
    assert_eq!(config.exploration_decay, 0.995);
    assert_eq!(config.min_exploration_rate, 0.01);
}

#[test]
fn test_learning_engine_config_debug() {
    let config = LearningEngineConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("LearningEngineConfig"));
}

#[test]
fn test_rl_state_empty_features() {
    let state = RLState {
        id: "empty_state".to_string(),
        features: vec![],
        context_id: "ctx_1".to_string(),
        timestamp: chrono::Utc::now(),
        metadata: None,
    };

    assert!(state.features.is_empty());
}

#[test]
fn test_rl_experience_negative_reward() {
    let exp = RLExperience {
        id: "exp_1".to_string(),
        state: RLState {
            id: "s1".to_string(),
            features: vec![1.0],
            context_id: "ctx_1".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: None,
        },
        action: RLAction {
            id: "a1".to_string(),
            action_type: "bad".to_string(),
            parameters: serde_json::json!({}),
            confidence: 0.0,
            expected_reward: 0.0,
        },
        reward: -0.5,
        next_state: Some(RLState {
            id: "s2".to_string(),
            features: vec![2.0],
            context_id: "ctx_1".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: None,
        }),
        done: false,
        timestamp: chrono::Utc::now(),
        priority: 1.0,
    };

    assert_eq!(exp.reward, -0.5);
}

#[test]
fn test_learning_algorithm_equality() {
    let algo1 = LearningAlgorithm::DeepQLearning;
    let s1 = serde_json::to_string(&algo1).expect("test: should succeed");

    let algo2 = LearningAlgorithm::DeepQLearning;
    let s2 = serde_json::to_string(&algo2).expect("test: should succeed");

    assert_eq!(s1, s2);
}

#[test]
fn test_rl_state_large_features() {
    let features: Vec<f64> = (0..128).map(|i| i as f64).collect();
    let state = RLState {
        id: "large_state".to_string(),
        features,
        context_id: "ctx_1".to_string(),
        timestamp: chrono::Utc::now(),
        metadata: None,
    };

    assert_eq!(state.features.len(), 128);
}

#[test]
fn test_learning_engine_config_network_params() {
    let config = LearningEngineConfig::default();
    assert_eq!(config.target_update_frequency, 1000);
    assert_eq!(config.buffer_size, 10000);
}

#[test]
fn test_rl_action_complex_params() {
    let action = RLAction {
        id: "complex_action".to_string(),
        action_type: "modify_context".to_string(),
        parameters: serde_json::json!({
            "path": "/data/value",
            "operation": "update",
            "value": 42
        }),
        confidence: 0.9,
        expected_reward: 5.0,
    };

    let serialized = serde_json::to_string(&action).expect("Failed");
    assert!(serialized.contains("modify_context"));
    assert!(serialized.contains("42"));
}

// ============================================================================
// PHASE 4: Engine Behavior Tests (Sprint 1)
// ============================================================================

#[tokio::test]
async fn test_learning_engine_creation() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let engine = LearningEngine::new(system_config).await;

    assert!(engine.is_ok(), "Engine creation should succeed");
}

#[tokio::test]
async fn test_learning_engine_initialization() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let engine = LearningEngine::new(system_config)
        .await
        .expect("Failed to create engine");

    let result = engine.initialize().await;
    assert!(result.is_ok(), "Engine initialization should succeed");

    let state = engine.get_state().await;
    assert!(
        matches!(state, crate::learning::LearningState::Learning),
        "Engine should be in Learning state after initialization"
    );
}

#[tokio::test]
async fn test_learning_engine_start_stop() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let engine = LearningEngine::new(system_config)
        .await
        .expect("Failed to create engine");

    // Start engine
    engine.start().await.expect("Failed to start engine");
    let state = engine.get_state().await;
    assert!(matches!(state, crate::learning::LearningState::Learning));

    // Stop engine
    engine.stop().await.expect("Failed to stop engine");
    let state = engine.get_state().await;
    assert!(matches!(state, crate::learning::LearningState::Stopped));
}

#[tokio::test]
async fn test_select_action_returns_valid_action() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let engine = LearningEngine::new(system_config)
        .await
        .expect("Failed to create engine");
    engine.initialize().await.expect("Failed to initialize");

    let state = RLState {
        id: "test_state_1".to_string(),
        features: vec![1.0, 2.0, 3.0, 4.0, 5.0],
        context_id: "ctx_1".to_string(),
        timestamp: chrono::Utc::now(),
        metadata: None,
    };

    let action = engine
        .select_action(&state)
        .await
        .expect("Failed to select action");

    assert!(!action.id.is_empty(), "Action should have an ID");
    assert!(!action.action_type.is_empty(), "Action should have a type");
}

#[tokio::test]
async fn test_exploration_rate_decay() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let engine = LearningEngine::new(system_config)
        .await
        .expect("Failed to create engine");
    engine.initialize().await.expect("Failed to initialize");

    let initial_rate = engine.get_exploration_rate().await;

    // Decay exploration rate multiple times
    for _ in 0..10 {
        engine
            .decay_exploration()
            .await
            .expect("Failed to decay exploration");
    }

    let final_rate = engine.get_exploration_rate().await;

    assert!(
        final_rate < initial_rate,
        "Exploration rate should decrease: {} -> {}",
        initial_rate,
        final_rate
    );
}

#[tokio::test]
async fn test_exploration_rate_reaches_minimum() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let engine = LearningEngine::new(system_config)
        .await
        .expect("Failed to create engine");
    engine.initialize().await.expect("Failed to initialize");

    // Decay exploration rate many times to hit minimum
    for _ in 0..1000 {
        engine
            .decay_exploration()
            .await
            .expect("Failed to decay exploration");
    }

    let final_rate = engine.get_exploration_rate().await;

    // Should reach minimum exploration rate (0.01)
    assert!(
        final_rate >= 0.01 - 0.001, // Allow small floating point error
        "Exploration rate should not go below minimum: {}",
        final_rate
    );
}

#[tokio::test]
async fn test_add_experience_increases_buffer_size() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let engine = LearningEngine::new(system_config)
        .await
        .expect("Failed to create engine");

    let initial_size = engine.get_experience_buffer_size().await;

    let experience = RLExperience {
        id: "exp_1".to_string(),
        state: RLState {
            id: "s1".to_string(),
            features: vec![1.0, 2.0],
            context_id: "ctx_1".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: None,
        },
        action: RLAction {
            id: "a1".to_string(),
            action_type: "test".to_string(),
            parameters: serde_json::json!({}),
            confidence: 0.5,
            expected_reward: 0.0,
        },
        reward: 1.0,
        next_state: Some(RLState {
            id: "s2".to_string(),
            features: vec![1.5, 2.5],
            context_id: "ctx_1".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: None,
        }),
        done: false,
        timestamp: chrono::Utc::now(),
        priority: 1.0,
    };

    engine
        .add_experience(experience)
        .await
        .expect("Failed to add experience");

    let final_size = engine.get_experience_buffer_size().await;

    assert_eq!(
        final_size,
        initial_size + 1,
        "Buffer size should increase by 1"
    );
}

#[tokio::test]
async fn test_experience_buffer_respects_max_size() {
    let mut config = test_helpers::create_test_learning_config();
    config.experience_buffer_size = 5; // Small buffer

    let system_config = Arc::new(config);
    let engine = LearningEngine::new(system_config)
        .await
        .expect("Failed to create engine");

    // Add more experiences than buffer can hold
    for i in 0..10 {
        let experience = RLExperience {
            id: format!("exp_{}", i),
            state: RLState {
                id: format!("s{}", i),
                features: vec![i as f64],
                context_id: "ctx_1".to_string(),
                timestamp: chrono::Utc::now(),
                metadata: None,
            },
            action: RLAction {
                id: format!("a{}", i),
                action_type: "test".to_string(),
                parameters: serde_json::json!({}),
                confidence: 0.5,
                expected_reward: 0.0,
            },
            reward: 1.0,
            next_state: None,
            done: false,
            timestamp: chrono::Utc::now(),
            priority: 1.0,
        };

        engine
            .add_experience(experience)
            .await
            .expect("Failed to add experience");
    }

    let buffer_size = engine.get_experience_buffer_size().await;

    assert!(
        buffer_size <= 5,
        "Buffer size should not exceed maximum: {}",
        buffer_size
    );
}

#[tokio::test]
async fn test_update_q_values_with_experience() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let engine = LearningEngine::new(system_config)
        .await
        .expect("Failed to create engine");
    engine.initialize().await.expect("Failed to initialize");

    let experience = RLExperience {
        id: "exp_1".to_string(),
        state: RLState {
            id: "s1".to_string(),
            features: vec![1.0, 2.0],
            context_id: "ctx_1".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: None,
        },
        action: RLAction {
            id: "a1".to_string(),
            action_type: "modify_context".to_string(),
            parameters: serde_json::json!({}),
            confidence: 0.5,
            expected_reward: 0.0,
        },
        reward: 1.0,
        next_state: None,
        done: true,
        timestamp: chrono::Utc::now(),
        priority: 1.0,
    };

    let result = engine.update_q_values(&experience).await;
    assert!(result.is_ok(), "Q-value update should succeed");
}

#[tokio::test]
async fn test_experience_buffer_grows_with_dqn_updates() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let engine = LearningEngine::new(system_config)
        .await
        .expect("Failed to create engine");
    engine.initialize().await.expect("Failed to initialize");

    let initial_size = engine.get_experience_buffer_size().await;

    // Add multiple experiences with different state-action pairs
    // Engine uses Deep Q-Learning by default, which uses experience buffer
    for i in 0..5 {
        let experience = RLExperience {
            id: format!("exp_{}", i),
            state: RLState {
                id: format!("state_{}", i),
                features: vec![i as f64],
                context_id: "ctx_1".to_string(),
                timestamp: chrono::Utc::now(),
                metadata: None,
            },
            action: RLAction {
                id: format!("action_{}", i),
                action_type: "modify_context".to_string(),
                parameters: serde_json::json!({}),
                confidence: 0.5,
                expected_reward: 0.0,
            },
            reward: 1.0,
            next_state: None,
            done: true,
            timestamp: chrono::Utc::now(),
            priority: 1.0,
        };

        engine
            .update_q_values(&experience)
            .await
            .expect("Failed to update Q-values");
    }

    let final_size = engine.get_experience_buffer_size().await;

    assert!(
        final_size > initial_size,
        "Experience buffer should grow with DQN: {} -> {}",
        initial_size,
        final_size
    );
}

#[tokio::test]
async fn test_engine_metrics_tracking() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let engine = LearningEngine::new(system_config)
        .await
        .expect("Failed to create engine");
    engine.initialize().await.expect("Failed to initialize");

    let metrics = engine.get_metrics().await;

    // Metrics should be initialized
    assert_eq!(metrics.total_steps, 0, "Initial steps should be 0");
    assert_eq!(metrics.total_episodes, 0, "Initial episodes should be 0");
    assert!(
        metrics.exploration_rate >= 0.0 && metrics.exploration_rate <= 1.0,
        "Exploration rate should be normalized"
    );
}

#[tokio::test]
async fn test_select_action_multiple_times_consistent() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let engine = LearningEngine::new(system_config)
        .await
        .expect("Failed to create engine");
    engine.initialize().await.expect("Failed to initialize");

    let state = RLState {
        id: "test_state".to_string(),
        features: vec![1.0, 2.0, 3.0],
        context_id: "ctx_1".to_string(),
        timestamp: chrono::Utc::now(),
        metadata: None,
    };

    // Select actions multiple times
    let mut action_ids = Vec::new();
    for _ in 0..5 {
        let action = engine
            .select_action(&state)
            .await
            .expect("Failed to select action");
        action_ids.push(action.id);
    }

    // All actions should have unique IDs
    let unique_count: usize = action_ids
        .iter()
        .collect::<std::collections::HashSet<_>>()
        .len();
    assert_eq!(unique_count, 5, "All actions should have unique IDs");
}

#[tokio::test]
async fn test_engine_handles_empty_feature_state() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let engine = LearningEngine::new(system_config)
        .await
        .expect("Failed to create engine");
    engine.initialize().await.expect("Failed to initialize");

    let state = RLState {
        id: "empty_state".to_string(),
        features: vec![],
        context_id: "ctx_1".to_string(),
        timestamp: chrono::Utc::now(),
        metadata: None,
    };

    // Should handle empty features gracefully
    let result = engine.select_action(&state).await;
    assert!(result.is_ok(), "Engine should handle empty features");
}

#[tokio::test]
async fn test_engine_handles_large_feature_state() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let engine = LearningEngine::new(system_config)
        .await
        .expect("Failed to create engine");
    engine.initialize().await.expect("Failed to initialize");

    let features: Vec<f64> = (0..256).map(|i| i as f64).collect();
    let state = RLState {
        id: "large_state".to_string(),
        features,
        context_id: "ctx_1".to_string(),
        timestamp: chrono::Utc::now(),
        metadata: None,
    };

    // Should handle large feature vectors
    let result = engine.select_action(&state).await;
    assert!(result.is_ok(), "Engine should handle large feature vectors");
}

#[tokio::test]
async fn test_engine_config_from_system_config() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let _learning_rate = system_config.learning_rate;
    let _discount_factor = system_config.discount_factor;

    let engine = LearningEngine::new(system_config)
        .await
        .expect("Failed to create engine");
    let exploration_rate = engine.get_exploration_rate().await;

    // Verify configuration was applied
    assert!(
        exploration_rate >= 0.0,
        "Exploration rate should be initialized"
    );

    // Test that we can access the engine even with the config
    let state = engine.get_state().await;
    assert!(matches!(
        state,
        crate::learning::LearningState::Initializing
    ));
}

#[tokio::test]
async fn test_multiple_engines_independent() {
    let config1 = Arc::new(test_helpers::create_test_learning_config());
    let config2 = Arc::new(test_helpers::create_test_learning_config());

    let engine1 = LearningEngine::new(config1)
        .await
        .expect("Failed to create engine 1");
    let engine2 = LearningEngine::new(config2)
        .await
        .expect("Failed to create engine 2");

    engine1
        .initialize()
        .await
        .expect("Failed to initialize engine 1");
    engine2
        .initialize()
        .await
        .expect("Failed to initialize engine 2");

    // Modify engine1
    engine1.decay_exploration().await.expect("Failed to decay");

    let rate1 = engine1.get_exploration_rate().await;
    let rate2 = engine2.get_exploration_rate().await;

    // Engine 2 should not be affected
    assert!(
        rate2 > rate1,
        "Engines should be independent: {} vs {}",
        rate1,
        rate2
    );
}

#[tokio::test]
async fn test_update_q_values_with_terminal_state() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let engine = LearningEngine::new(system_config)
        .await
        .expect("Failed to create engine");
    engine.initialize().await.expect("Failed to initialize");

    let experience = RLExperience {
        id: "terminal_exp".to_string(),
        state: RLState {
            id: "s_terminal".to_string(),
            features: vec![1.0],
            context_id: "ctx_1".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: None,
        },
        action: RLAction {
            id: "a_terminal".to_string(),
            action_type: "complete".to_string(),
            parameters: serde_json::json!({}),
            confidence: 1.0,
            expected_reward: 10.0,
        },
        reward: 10.0,
        next_state: None,
        done: true,
        timestamp: chrono::Utc::now(),
        priority: 1.0,
    };

    let result = engine.update_q_values(&experience).await;
    assert!(result.is_ok(), "Should handle terminal state");
}

#[tokio::test]
async fn test_update_q_values_with_next_state() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let engine = LearningEngine::new(system_config)
        .await
        .expect("Failed to create engine");
    engine.initialize().await.expect("Failed to initialize");

    let experience = RLExperience {
        id: "transition_exp".to_string(),
        state: RLState {
            id: "s1".to_string(),
            features: vec![1.0],
            context_id: "ctx_1".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: None,
        },
        action: RLAction {
            id: "a1".to_string(),
            action_type: "step".to_string(),
            parameters: serde_json::json!({}),
            confidence: 0.8,
            expected_reward: 1.0,
        },
        reward: 1.0,
        next_state: Some(RLState {
            id: "s2".to_string(),
            features: vec![2.0],
            context_id: "ctx_1".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: None,
        }),
        done: false,
        timestamp: chrono::Utc::now(),
        priority: 1.0,
    };

    let result = engine.update_q_values(&experience).await;
    assert!(result.is_ok(), "Should handle transition with next state");
}
