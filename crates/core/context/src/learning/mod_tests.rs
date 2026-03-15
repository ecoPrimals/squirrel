// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for learning system types and enums

use super::test_helpers;
use super::*;
use serde_json;

#[test]
fn test_learning_state_variants() {
    let states = vec![
        LearningState::Initializing,
        LearningState::Learning,
        LearningState::Evaluating,
        LearningState::Adapting,
        LearningState::Paused,
        LearningState::Stopped,
    ];

    for state in states {
        let cloned = state.clone();
        let debug_str = format!("{:?}", cloned);
        assert!(!debug_str.is_empty());
    }
}

#[test]
fn test_learning_state_serialization() {
    let state = LearningState::Learning;
    let serialized = serde_json::to_string(&state).expect("Failed to serialize");
    let deserialized: LearningState =
        serde_json::from_str(&serialized).expect("Failed to deserialize");

    match deserialized {
        LearningState::Learning => {}
        _ => panic!("Deserialization failed"),
    }
}

#[test]
fn test_learning_action_type_variants() {
    let actions = vec![
        LearningActionType::ModifyContext,
        LearningActionType::ApplyRule,
        LearningActionType::UpdatePolicy,
        LearningActionType::AdaptRule,
        LearningActionType::CreateSnapshot,
        LearningActionType::TriggerSync,
        LearningActionType::Custom("test_action".to_string()),
    ];

    for action in actions {
        let cloned = action.clone();
        let debug_str = format!("{:?}", cloned);
        assert!(!debug_str.is_empty());
    }
}

#[test]
fn test_learning_action_type_custom() {
    let action = LearningActionType::Custom("my_custom_action".to_string());
    let serialized = serde_json::to_string(&action).expect("Failed to serialize");
    assert!(serialized.contains("my_custom_action"));
}

#[test]
fn test_learning_event_system_initialized() {
    let config = LearningSystemConfig::default();
    let event = LearningEvent::SystemInitialized {
        timestamp: chrono::Utc::now(),
        config,
    };

    let debug_str = format!("{:?}", event);
    assert!(debug_str.contains("SystemInitialized"));
}

#[test]
fn test_learning_event_training_started() {
    let event = LearningEvent::TrainingStarted {
        episode: 42,
        timestamp: chrono::Utc::now(),
    };

    let cloned = event.clone();
    let debug_str = format!("{:?}", cloned);
    assert!(debug_str.contains("TrainingStarted"));
    assert!(debug_str.contains("42"));
}

#[test]
fn test_learning_event_reward_received() {
    let event = LearningEvent::RewardReceived {
        reward: 1.5,
        context: "test_context".to_string(),
        timestamp: chrono::Utc::now(),
    };

    let serialized = serde_json::to_string(&event).expect("Failed to serialize");
    assert!(serialized.contains("1.5"));
    assert!(serialized.contains("test_context"));
}

#[test]
fn test_learning_event_policy_updated() {
    let event = LearningEvent::PolicyUpdated {
        loss: 0.25,
        accuracy: 0.85,
        timestamp: chrono::Utc::now(),
    };

    let debug_str = format!("{:?}", event);
    assert!(debug_str.contains("PolicyUpdated"));
}

#[test]
fn test_learning_event_adaptation_triggered() {
    let event = LearningEvent::AdaptationTriggered {
        rule_count: 10,
        timestamp: chrono::Utc::now(),
    };

    let serialized = serde_json::to_string(&event).expect("Failed to serialize");
    assert!(serialized.contains("10"));
}

#[test]
fn test_learning_event_metrics_updated() {
    let mut metrics = HashMap::new();
    metrics.insert("accuracy".to_string(), 0.95);
    metrics.insert("loss".to_string(), 0.05);

    let event = LearningEvent::MetricsUpdated {
        metrics,
        timestamp: chrono::Utc::now(),
    };

    let debug_str = format!("{:?}", event);
    assert!(debug_str.contains("MetricsUpdated"));
}

#[test]
fn test_learning_system_stats_default() {
    let stats = LearningSystemStats::default();
    assert_eq!(stats.total_episodes, 0);
    assert_eq!(stats.total_actions, 0);
    assert_eq!(stats.total_rewards, 0.0);
    assert_eq!(stats.average_reward_per_episode, 0.0);
}

#[test]
fn test_learning_system_stats_clone() {
    let stats = LearningSystemStats {
        total_episodes: 100,
        total_rewards: 500.0,
        ..Default::default()
    };

    let cloned = stats.clone();
    assert_eq!(cloned.total_episodes, 100);
    assert_eq!(cloned.total_rewards, 500.0);
}

#[test]
fn test_learning_system_stats_serialization() {
    let stats = LearningSystemStats::default();
    let serialized = serde_json::to_string(&stats).expect("Failed to serialize");
    let deserialized: LearningSystemStats =
        serde_json::from_str(&serialized).expect("Failed to deserialize");

    assert_eq!(deserialized.total_episodes, stats.total_episodes);
}

#[test]
fn test_learning_system_config_default() {
    let config = LearningSystemConfig::default();
    assert!(config.learning_rate > 0.0);
    assert!(config.discount_factor > 0.0 && config.discount_factor <= 1.0);
}

#[test]
fn test_learning_system_config_clone() {
    let config = LearningSystemConfig::default();
    let cloned = config.clone();
    assert_eq!(cloned.learning_rate, config.learning_rate);
}

#[test]
fn test_learning_system_config_serialization() {
    let config = LearningSystemConfig::default();
    let serialized = serde_json::to_string(&config).expect("Failed to serialize");
    let deserialized: LearningSystemConfig =
        serde_json::from_str(&serialized).expect("Failed to deserialize");

    assert_eq!(deserialized.learning_rate, config.learning_rate);
}

#[test]
fn test_learning_action_type_serialization() {
    let action = LearningActionType::ModifyContext;
    let serialized = serde_json::to_string(&action).expect("Failed to serialize");
    let deserialized: LearningActionType =
        serde_json::from_str(&serialized).expect("Failed to deserialize");

    match deserialized {
        LearningActionType::ModifyContext => {}
        _ => panic!("Deserialization failed"),
    }
}

#[test]
fn test_multiple_learning_states() {
    let states = vec![
        (LearningState::Initializing, "Initializing"),
        (LearningState::Learning, "Learning"),
        (LearningState::Evaluating, "Evaluating"),
        (LearningState::Adapting, "Adapting"),
        (LearningState::Paused, "Paused"),
        (LearningState::Stopped, "Stopped"),
    ];

    for (state, name) in states {
        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains(name));
    }
}

#[test]
fn test_learning_event_large_metrics() {
    let mut metrics = HashMap::new();
    for i in 0..100 {
        metrics.insert(format!("metric_{}", i), i as f64);
    }

    let event = LearningEvent::MetricsUpdated {
        metrics,
        timestamp: chrono::Utc::now(),
    };

    let serialized = serde_json::to_string(&event).expect("Failed to serialize");
    assert!(!serialized.is_empty());
}

#[test]
fn test_learning_system_stats_debug() {
    let stats = LearningSystemStats::default();
    let debug_str = format!("{:?}", stats);
    assert!(debug_str.contains("LearningSystemStats"));
}

#[test]
fn test_learning_event_cloning() {
    let event = LearningEvent::TrainingStarted {
        episode: 1,
        timestamp: chrono::Utc::now(),
    };

    let cloned = event.clone();
    let debug1 = format!("{:?}", event);
    let debug2 = format!("{:?}", cloned);

    assert_eq!(debug1, debug2);
}

#[test]
fn test_learning_action_type_eq() {
    let action1 = LearningActionType::ModifyContext;
    let action2 = LearningActionType::ModifyContext;

    let s1 = serde_json::to_string(&action1).expect("test: should succeed");
    let s2 = serde_json::to_string(&action2).expect("test: should succeed");

    assert_eq!(s1, s2);
}

#[test]
fn test_learning_state_all_variants_coverage() {
    let variants = [
        LearningState::Initializing,
        LearningState::Learning,
        LearningState::Evaluating,
        LearningState::Adapting,
        LearningState::Paused,
        LearningState::Stopped,
    ];

    for variant in &variants {
        let _serialized = serde_json::to_string(variant).expect("Serialization failed");
    }
}

#[test]
fn test_learning_event_reward_negative() {
    let event = LearningEvent::RewardReceived {
        reward: -0.5,
        context: "penalty_context".to_string(),
        timestamp: chrono::Utc::now(),
    };

    let serialized = serde_json::to_string(&event).expect("Failed to serialize");
    assert!(serialized.contains("-0.5"));
}

#[test]
fn test_learning_event_empty_context() {
    let event = LearningEvent::RewardReceived {
        reward: 1.0,
        context: String::new(),
        timestamp: chrono::Utc::now(),
    };

    let _serialized = serde_json::to_string(&event).expect("Failed to serialize");
}

#[test]
fn test_learning_stats_with_values() {
    let stats = LearningSystemStats {
        total_episodes: 1000,
        total_actions: 50000,
        total_rewards: 12345.67,
        ..Default::default()
    };

    let serialized = serde_json::to_string(&stats).expect("Failed to serialize");
    assert!(serialized.contains("1000"));
    assert!(serialized.contains("50000"));
}

// ============================================================================
// PHASE 2: Configuration Tests (Sprint 1)
// ============================================================================

#[test]
fn test_learning_system_config_default_values() {
    let config = LearningSystemConfig::default();
    test_helpers::assert_valid_config(&config);

    assert!(config.enable_reinforcement_learning);
    assert!(config.enable_experience_replay);
    assert!(config.enable_adaptive_rules);
    assert!(config.enable_learning_metrics);
    assert_eq!(config.learning_rate, 0.001);
    assert_eq!(config.discount_factor, 0.95);
    assert_eq!(config.exploration_rate, 0.1);
    assert_eq!(config.experience_buffer_size, 10000);
    assert_eq!(config.batch_size, 32);
}

#[test]
fn test_learning_system_config_custom_settings() {
    let config = LearningSystemConfig {
        learning_rate: 0.01,
        exploration_rate: 0.5,
        enable_experience_replay: false,
        ..Default::default()
    };

    test_helpers::assert_valid_config(&config);
    assert_eq!(config.learning_rate, 0.01);
    assert_eq!(config.exploration_rate, 0.5);
    assert!(!config.enable_experience_replay);
}

#[test]
fn test_learning_system_config_serialization_complete() {
    let config = LearningSystemConfig::default();
    let serialized = serde_json::to_string(&config).expect("Failed to serialize");
    let deserialized: LearningSystemConfig =
        serde_json::from_str(&serialized).expect("Failed to deserialize");

    assert_eq!(deserialized.learning_rate, config.learning_rate);
    assert_eq!(deserialized.discount_factor, config.discount_factor);
    assert_eq!(
        deserialized.enable_reinforcement_learning,
        config.enable_reinforcement_learning
    );
}

#[test]
fn test_policy_network_config_default() {
    let config = PolicyNetworkConfig::default();
    test_helpers::assert_valid_policy_config(&config);

    assert_eq!(config.input_size, 128);
    assert_eq!(config.output_size, 32);
    assert_eq!(config.hidden_layers, vec![256, 128, 64]);
    assert_eq!(config.activation_function, "relu");
    assert_eq!(config.optimizer, "adam");
}

#[test]
fn test_policy_network_config_custom() {
    let config = PolicyNetworkConfig {
        input_size: 64,
        hidden_layers: vec![128, 64],
        output_size: 16,
        activation_function: "tanh".to_string(),
        optimizer: "sgd".to_string(),
        dropout_rate: 0.3,
    };

    test_helpers::assert_valid_policy_config(&config);
    assert_eq!(config.input_size, 64);
    assert_eq!(config.hidden_layers.len(), 2);
    assert_eq!(config.activation_function, "tanh");
}

#[test]
fn test_policy_network_config_serialization() {
    let config = PolicyNetworkConfig::default();
    let serialized = serde_json::to_string(&config).expect("Failed to serialize");
    let deserialized: PolicyNetworkConfig =
        serde_json::from_str(&serialized).expect("Failed to deserialize");

    assert_eq!(deserialized.input_size, config.input_size);
    assert_eq!(deserialized.output_size, config.output_size);
    assert_eq!(deserialized.hidden_layers, config.hidden_layers);
}

#[test]
fn test_reward_calculation_type_simple() {
    let reward_type = RewardCalculationType::Simple;
    let serialized = serde_json::to_string(&reward_type).expect("Failed to serialize");
    assert!(serialized.contains("Simple"));
}

#[test]
fn test_reward_calculation_type_composite() {
    let reward_type = RewardCalculationType::Composite;
    let serialized = serde_json::to_string(&reward_type).expect("Failed to serialize");
    assert!(serialized.contains("Composite"));
}

#[test]
fn test_reward_calculation_type_custom() {
    let reward_type = RewardCalculationType::Custom("my_reward_fn".to_string());
    let serialized = serde_json::to_string(&reward_type).expect("Failed to serialize");
    assert!(serialized.contains("my_reward_fn"));

    let deserialized: RewardCalculationType =
        serde_json::from_str(&serialized).expect("Failed to deserialize");
    match deserialized {
        RewardCalculationType::Custom(name) => assert_eq!(name, "my_reward_fn"),
        _ => panic!("Expected Custom variant"),
    }
}

#[test]
fn test_learning_system_stats_default_state() {
    let stats = LearningSystemStats::default();
    test_helpers::assert_valid_stats(&stats);

    assert_eq!(stats.total_episodes, 0);
    assert_eq!(stats.total_actions, 0);
    assert_eq!(stats.total_rewards, 0.0);
    assert_eq!(stats.average_reward_per_episode, 0.0);
    assert_eq!(stats.success_rate, 0.0);
    assert_eq!(stats.learning_accuracy, 0.0);
    assert!(stats.last_performance.is_none());
}

#[test]
fn test_learning_system_stats_updates() {
    let stats = LearningSystemStats {
        total_episodes: 10,
        total_actions: 100,
        total_rewards: 50.0,
        average_reward_per_episode: 5.0,
        success_rate: 0.7,
        learning_accuracy: 0.85,
        ..Default::default()
    };

    test_helpers::assert_valid_stats(&stats);
    assert_eq!(stats.total_episodes, 10);
    assert_eq!(stats.average_reward_per_episode, 5.0);
}

#[test]
fn test_learning_system_stats_serialization_with_values() {
    let stats = test_helpers::create_test_stats();
    let serialized = serde_json::to_string(&stats).expect("Failed to serialize");
    let deserialized: LearningSystemStats =
        serde_json::from_str(&serialized).expect("Failed to deserialize");

    assert_eq!(deserialized.total_episodes, stats.total_episodes);
    assert_eq!(deserialized.total_actions, stats.total_actions);
    assert_eq!(deserialized.total_rewards, stats.total_rewards);
}

#[test]
fn test_learning_system_config_with_test_helpers() {
    let config = test_helpers::create_test_learning_config();
    test_helpers::assert_valid_config(&config);

    // Verify it's optimized for testing
    assert!(config.enable_reinforcement_learning);
    assert!(!config.enable_experience_replay); // Simplified
    assert!(!config.enable_adaptive_rules); // Simplified
    assert_eq!(config.experience_buffer_size, 100); // Smaller
}

#[test]
fn test_learning_system_config_full_features() {
    let config = test_helpers::create_full_test_config();
    test_helpers::assert_valid_config(&config);

    // Verify all features enabled
    assert!(config.enable_reinforcement_learning);
    assert!(config.enable_experience_replay);
    assert!(config.enable_adaptive_rules);
    assert!(config.enable_learning_metrics);
}

#[test]
fn test_config_validates_learning_rate_bounds() {
    let mut config = test_helpers::create_test_learning_config();

    // Test valid learning rates
    config.learning_rate = 0.001;
    test_helpers::assert_valid_config(&config);

    config.learning_rate = 0.5;
    test_helpers::assert_valid_config(&config);

    config.learning_rate = 0.999;
    test_helpers::assert_valid_config(&config);
}

#[test]
fn test_config_validates_discount_factor_bounds() {
    let mut config = test_helpers::create_test_learning_config();

    // Test valid discount factors
    config.discount_factor = 0.0;
    test_helpers::assert_valid_config(&config);

    config.discount_factor = 0.95;
    test_helpers::assert_valid_config(&config);

    config.discount_factor = 1.0;
    test_helpers::assert_valid_config(&config);
}
