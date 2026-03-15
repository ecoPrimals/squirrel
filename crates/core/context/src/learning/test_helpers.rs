// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Test helpers for learning system
//!
//! This module provides utilities and helpers for testing the learning system components.
//! It includes mock configurations, test data builders, and common assertions.

use super::*;

/// Create a minimal test configuration for the learning system
///
/// This configuration is optimized for testing with smaller buffers,
/// faster update intervals, and disabled optional features.
pub fn create_test_learning_config() -> LearningSystemConfig {
    LearningSystemConfig {
        enable_reinforcement_learning: true,
        enable_experience_replay: false, // Simplify for basic tests
        enable_adaptive_rules: false,    // Simplify for basic tests
        enable_learning_metrics: false,  // Simplify for basic tests
        learning_rate: 0.1,
        discount_factor: 0.9,
        exploration_rate: 0.1,
        experience_buffer_size: 100, // Smaller for tests
        batch_size: 10,              // Smaller for tests
        target_update_frequency: 10,
        min_experiences_for_learning: 10, // Lower threshold for tests
        max_episode_length: 100,          // Shorter episodes for tests
        reward_calculation: RewardCalculationType::Simple,
        policy_network_config: create_test_policy_config(),
        learning_update_interval: Duration::from_secs(1), // Faster for tests
        enable_debug_logging: false,
    }
}

/// Create a minimal policy network configuration for testing
///
/// Uses small layer sizes for fast test execution.
pub fn create_test_policy_config() -> PolicyNetworkConfig {
    PolicyNetworkConfig {
        input_size: 10,              // Small for tests
        hidden_layers: vec![20, 10], // Small hidden layers
        output_size: 5,              // Small output
        activation_function: "relu".to_string(),
        optimizer: "adam".to_string(),
        dropout_rate: 0.0, // Disable dropout for deterministic tests
    }
}

/// Create a configuration with all features enabled for integration tests
pub fn create_full_test_config() -> LearningSystemConfig {
    LearningSystemConfig {
        enable_reinforcement_learning: true,
        enable_experience_replay: true,
        enable_adaptive_rules: true,
        enable_learning_metrics: true,
        learning_rate: 0.01,
        discount_factor: 0.95,
        exploration_rate: 0.2,
        experience_buffer_size: 500,
        batch_size: 16,
        target_update_frequency: 50,
        min_experiences_for_learning: 50,
        max_episode_length: 200,
        reward_calculation: RewardCalculationType::Composite,
        policy_network_config: create_large_policy_config(),
        learning_update_interval: Duration::from_secs(5),
        enable_debug_logging: true,
    }
}

/// Create a larger policy configuration for integration tests
pub fn create_large_policy_config() -> PolicyNetworkConfig {
    PolicyNetworkConfig {
        input_size: 64,
        hidden_layers: vec![128, 64, 32],
        output_size: 16,
        activation_function: "relu".to_string(),
        optimizer: "adam".to_string(),
        dropout_rate: 0.1,
    }
}

/// Create a test learning system statistics object
pub fn create_test_stats() -> LearningSystemStats {
    LearningSystemStats {
        total_episodes: 10,
        total_actions: 100,
        total_rewards: 50.0,
        average_reward_per_episode: 5.0,
        success_rate: 0.7,
        learning_accuracy: 0.8,
        policy_updates: 20,
        rule_adaptations: 5,
        uptime: Duration::from_secs(600),
        last_performance: Some(chrono::Utc::now()),
    }
}

/// Create a test rule for testing adaptive system
pub fn create_test_rule() -> crate::rules::Rule {
    use crate::rules::{Rule, RuleAction, RuleCondition};
    use uuid::Uuid;

    Rule::builder()
        .with_id(Uuid::new_v4().to_string())
        .with_name("test_rule".to_string())
        .with_description("Test rule for adaptive system".to_string())
        .with_condition(RuleCondition::Exists {
            path: "test.path".to_string(),
        })
        .with_action(RuleAction::ModifyContext {
            path: "test.result".to_string(),
            value: serde_json::json!("test_value"),
        })
        .with_priority(1)
        .build()
        .expect("Should build test rule")
}

/// Assert that a learning config has reasonable values
pub fn assert_valid_config(config: &LearningSystemConfig) {
    assert!(
        config.learning_rate > 0.0 && config.learning_rate < 1.0,
        "Learning rate should be between 0 and 1"
    );
    assert!(
        config.discount_factor >= 0.0 && config.discount_factor <= 1.0,
        "Discount factor should be between 0 and 1"
    );
    assert!(
        config.exploration_rate >= 0.0 && config.exploration_rate <= 1.0,
        "Exploration rate should be between 0 and 1"
    );
    assert!(
        config.experience_buffer_size > 0,
        "Experience buffer size must be positive"
    );
    assert!(config.batch_size > 0, "Batch size must be positive");
    assert!(
        config.batch_size <= config.experience_buffer_size,
        "Batch size should not exceed buffer size"
    );
    assert!(
        config.min_experiences_for_learning <= config.experience_buffer_size,
        "Min experiences should not exceed buffer size"
    );
}

/// Assert that policy network config has valid dimensions
pub fn assert_valid_policy_config(config: &PolicyNetworkConfig) {
    assert!(config.input_size > 0, "Input size must be positive");
    assert!(config.output_size > 0, "Output size must be positive");
    assert!(
        !config.hidden_layers.is_empty(),
        "Should have at least one hidden layer"
    );
    for (i, &size) in config.hidden_layers.iter().enumerate() {
        assert!(size > 0, "Hidden layer {} size must be positive", i);
    }
    assert!(
        config.dropout_rate >= 0.0 && config.dropout_rate < 1.0,
        "Dropout rate should be between 0 and 1"
    );
}

/// Assert that learning stats are within reasonable bounds
pub fn assert_valid_stats(stats: &LearningSystemStats) {
    // Note: total_episodes and total_actions are u64, always non-negative
    // Just verify they exist and are accessible
    let _ = stats.total_episodes;
    let _ = stats.total_actions;
    assert!(
        stats.success_rate >= 0.0 && stats.success_rate <= 1.0,
        "Success rate should be between 0 and 1"
    );
    assert!(
        stats.learning_accuracy >= 0.0 && stats.learning_accuracy <= 1.0,
        "Learning accuracy should be between 0 and 1"
    );

    if stats.total_episodes > 0 {
        let expected_avg = stats.total_rewards / stats.total_episodes as f64;
        assert!(
            (stats.average_reward_per_episode - expected_avg).abs() < 0.01
                || stats.average_reward_per_episode == 0.0, // Allow for default case
            "Average reward calculation seems incorrect"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_learning_config() {
        let config = create_test_learning_config();
        assert_valid_config(&config);
        assert!(config.enable_reinforcement_learning);
        assert!(!config.enable_experience_replay); // Simplified for tests
        assert_eq!(config.learning_rate, 0.1);
    }

    #[test]
    fn test_create_full_test_config() {
        let config = create_full_test_config();
        assert_valid_config(&config);
        assert!(config.enable_reinforcement_learning);
        assert!(config.enable_experience_replay); // Enabled for full tests
        assert!(config.enable_adaptive_rules);
    }

    #[test]
    fn test_create_test_policy_config() {
        let config = create_test_policy_config();
        assert_valid_policy_config(&config);
        assert_eq!(config.input_size, 10);
        assert_eq!(config.output_size, 5);
        assert_eq!(config.dropout_rate, 0.0); // Disabled for deterministic tests
    }

    #[test]
    fn test_create_large_policy_config() {
        let config = create_large_policy_config();
        assert_valid_policy_config(&config);
        assert!(config.input_size > 10);
        assert!(config.hidden_layers.len() > 1);
    }

    #[test]
    fn test_create_test_stats() {
        let stats = create_test_stats();
        assert_valid_stats(&stats);
        assert_eq!(stats.total_episodes, 10);
        assert_eq!(stats.total_actions, 100);
        assert!(stats.last_performance.is_some());
    }

    #[test]
    fn test_assert_valid_config_catches_invalid_learning_rate() {
        let mut config = create_test_learning_config();
        config.learning_rate = 1.5; // Invalid

        let result = std::panic::catch_unwind(|| {
            assert_valid_config(&config);
        });
        assert!(result.is_err(), "Should panic on invalid learning rate");
    }

    #[test]
    fn test_assert_valid_config_catches_invalid_batch_size() {
        let mut config = create_test_learning_config();
        config.batch_size = config.experience_buffer_size + 1; // Invalid

        let result = std::panic::catch_unwind(|| {
            assert_valid_config(&config);
        });
        assert!(
            result.is_err(),
            "Should panic when batch size exceeds buffer"
        );
    }

    #[test]
    fn test_assert_valid_policy_config_catches_zero_input() {
        let mut config = create_test_policy_config();
        config.input_size = 0; // Invalid

        let result = std::panic::catch_unwind(|| {
            assert_valid_policy_config(&config);
        });
        assert!(result.is_err(), "Should panic on zero input size");
    }

    #[test]
    fn test_assert_valid_stats_catches_invalid_success_rate() {
        let mut stats = create_test_stats();
        stats.success_rate = 1.5; // Invalid

        let result = std::panic::catch_unwind(|| {
            assert_valid_stats(&stats);
        });
        assert!(result.is_err(), "Should panic on invalid success rate");
    }
}
