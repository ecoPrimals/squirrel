// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for learning engine types and configs

use crate::learning::engine::{
    LearningAlgorithm, LearningEngineConfig, RLAction, RLExperience, RLState,
};

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
        _ => unreachable!("Deserialization failed"),
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
