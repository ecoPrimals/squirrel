// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for learning engine types and configs

use crate::learning::engine::{LearningEngine, RLAction, RLExperience, RLState};
use crate::learning::test_helpers;
use std::sync::Arc;

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

#[tokio::test]
async fn test_dqn_train_network_runs_after_batch_size_reached() {
    let mut config = test_helpers::create_test_learning_config();
    config.batch_size = 8;
    // Avoid target network update while `training_steps` write lock is held (would deadlock).
    config.target_update_frequency = 10_000;
    let system_config = Arc::new(config);
    let engine = LearningEngine::new(system_config).await.expect("engine");
    engine.initialize().await.expect("init");
    let initial_steps = engine.get_metrics().await.total_steps;
    for i in 0..12 {
        let experience = RLExperience {
            id: format!("exp_{i}"),
            state: RLState {
                id: format!("s{i}"),
                features: vec![i as f64],
                context_id: "ctx".to_string(),
                timestamp: chrono::Utc::now(),
                metadata: None,
            },
            action: RLAction {
                id: format!("a{i}"),
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
        engine.update_q_values(&experience).await.expect("update");
    }
    let after = engine.get_metrics().await;
    assert!(
        after.total_steps > initial_steps,
        "training should increment metrics.total_steps"
    );
    assert!(after.training_time > 0.0);
}

#[tokio::test]
async fn test_engine_reward_processing_accumulates_in_buffer() {
    let system_config = Arc::new(test_helpers::create_test_learning_config());
    let engine = LearningEngine::new(system_config).await.expect("engine");
    engine.initialize().await.expect("init");
    let exp = RLExperience {
        id: "r1".to_string(),
        state: RLState {
            id: "s1".to_string(),
            features: vec![0.5],
            context_id: "ctx".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: None,
        },
        action: RLAction {
            id: "a1".to_string(),
            action_type: "step".to_string(),
            parameters: serde_json::json!({}),
            confidence: 0.5,
            expected_reward: 0.0,
        },
        reward: -1.5,
        next_state: Some(RLState {
            id: "s2".to_string(),
            features: vec![0.6],
            context_id: "ctx".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: None,
        }),
        done: false,
        timestamp: chrono::Utc::now(),
        priority: 1.0,
    };
    let before = engine.get_experience_buffer_size().await;
    engine.update_q_values(&exp).await.expect("update");
    assert_eq!(engine.get_experience_buffer_size().await, before + 1);
}
