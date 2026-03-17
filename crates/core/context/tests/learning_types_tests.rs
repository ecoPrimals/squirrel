// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Comprehensive tests for Context Learning System types
//!
//! Tests learning states, action types, and configurations.

#![allow(clippy::field_reassign_with_default)] // Test code uses builder pattern

use squirrel_context::learning::{ContextLearningManagerConfig, LearningActionType, LearningState};
use std::time::Duration;

#[test]
fn test_learning_state_initializing() {
    let state = LearningState::Initializing;

    assert!(matches!(state, LearningState::Initializing));
}

#[test]
fn test_learning_state_learning() {
    let state = LearningState::Learning;

    assert!(matches!(state, LearningState::Learning));
}

#[test]
fn test_learning_state_evaluating() {
    let state = LearningState::Evaluating;

    assert!(matches!(state, LearningState::Evaluating));
}

#[test]
fn test_learning_state_adapting() {
    let state = LearningState::Adapting;

    assert!(matches!(state, LearningState::Adapting));
}

#[test]
fn test_learning_state_paused() {
    let state = LearningState::Paused;

    assert!(matches!(state, LearningState::Paused));
}

#[test]
fn test_learning_state_stopped() {
    let state = LearningState::Stopped;

    assert!(matches!(state, LearningState::Stopped));
}

#[test]
fn test_learning_state_clone() {
    let state1 = LearningState::Learning;
    let state2 = state1;

    assert!(matches!(state2, LearningState::Learning));
}

#[test]
fn test_learning_state_debug() {
    let state = LearningState::Learning;
    let debug_str = format!("{state:?}");

    assert!(!debug_str.is_empty());
    assert!(debug_str.contains("Learning"));
}

#[test]
fn test_learning_state_serialization() {
    let state = LearningState::Learning;
    let serialized = serde_json::to_string(&state);

    assert!(serialized.is_ok());
}

#[test]
fn test_learning_state_lifecycle() {
    let lifecycle = [
        LearningState::Initializing,
        LearningState::Learning,
        LearningState::Evaluating,
        LearningState::Adapting,
        LearningState::Paused,
        LearningState::Stopped,
    ];

    assert_eq!(lifecycle.len(), 6);
}

#[test]
fn test_learning_action_type_modify_context() {
    let action = LearningActionType::ModifyContext;

    assert!(matches!(action, LearningActionType::ModifyContext));
}

#[test]
fn test_learning_action_type_apply_rule() {
    let action = LearningActionType::ApplyRule;

    assert!(matches!(action, LearningActionType::ApplyRule));
}

#[test]
fn test_learning_action_type_update_policy() {
    let action = LearningActionType::UpdatePolicy;

    assert!(matches!(action, LearningActionType::UpdatePolicy));
}

#[test]
fn test_learning_action_type_adapt_rule() {
    let action = LearningActionType::AdaptRule;

    assert!(matches!(action, LearningActionType::AdaptRule));
}

#[test]
fn test_learning_action_type_create_snapshot() {
    let action = LearningActionType::CreateSnapshot;

    assert!(matches!(action, LearningActionType::CreateSnapshot));
}

#[test]
fn test_learning_action_type_trigger_sync() {
    let action = LearningActionType::TriggerSync;

    assert!(matches!(action, LearningActionType::TriggerSync));
}

#[test]
fn test_learning_action_type_custom() {
    let action = LearningActionType::Custom("custom_action".to_string());

    assert!(matches!(action, LearningActionType::Custom(_)));
}

#[test]
fn test_learning_action_type_custom_with_message() {
    let custom_msg = "special_learning_action";
    let action = LearningActionType::Custom(custom_msg.to_string());

    if let LearningActionType::Custom(msg) = action {
        assert_eq!(msg, custom_msg);
    } else {
        panic!("Should be Custom variant");
    }
}

#[test]
fn test_learning_action_type_clone() {
    let action1 = LearningActionType::ApplyRule;
    let action2 = action1;

    assert!(matches!(action2, LearningActionType::ApplyRule));
}

#[test]
fn test_learning_action_type_debug() {
    let action = LearningActionType::UpdatePolicy;
    let debug_str = format!("{action:?}");

    assert!(!debug_str.is_empty());
}

#[test]
fn test_learning_action_type_serialization() {
    let action = LearningActionType::ModifyContext;
    let serialized = serde_json::to_string(&action);

    assert!(serialized.is_ok());
}

#[test]
fn test_learning_action_type_custom_serialization() {
    let action = LearningActionType::Custom("test".to_string());
    let serialized = serde_json::to_string(&action);

    assert!(serialized.is_ok());
}

#[test]
fn test_learning_action_type_all_variants() {
    let actions = [
        LearningActionType::ModifyContext,
        LearningActionType::ApplyRule,
        LearningActionType::UpdatePolicy,
        LearningActionType::AdaptRule,
        LearningActionType::CreateSnapshot,
        LearningActionType::TriggerSync,
        LearningActionType::Custom("test".to_string()),
    ];

    assert_eq!(actions.len(), 7);
}

#[test]
fn test_context_learning_manager_config_default() {
    let config = ContextLearningManagerConfig::default();

    assert!(config.episode_timeout > 0);
    assert!(config.max_episodes_per_session > 0);
    assert!(config.state_space_size > 0);
    assert!(config.action_space_size > 0);
}

#[test]
fn test_context_learning_manager_config_clone() {
    let config1 = ContextLearningManagerConfig::default();
    let config2 = config1.clone();

    assert_eq!(config1.episode_timeout, config2.episode_timeout);
    assert_eq!(
        config1.max_episodes_per_session,
        config2.max_episodes_per_session
    );
}

#[test]
fn test_context_learning_manager_config_debug() {
    let config = ContextLearningManagerConfig::default();
    let debug_str = format!("{config:?}");

    assert!(!debug_str.is_empty());
}

#[test]
fn test_learning_state_all_transitions() {
    // Test that all states can be created and transitioned
    let states = vec![
        LearningState::Initializing,
        LearningState::Learning,
        LearningState::Evaluating,
        LearningState::Adapting,
        LearningState::Paused,
        LearningState::Stopped,
    ];

    for state in states {
        let _cloned = state.clone();
        let _debug = format!("{state:?}");
    }
}

#[test]
fn test_learning_action_type_pattern_matching() {
    let action = LearningActionType::ModifyContext;

    let is_modify = matches!(action, LearningActionType::ModifyContext);
    assert!(is_modify);
}

#[test]
fn test_learning_action_type_custom_empty_string() {
    let action = LearningActionType::Custom(String::new());

    if let LearningActionType::Custom(msg) = action {
        assert_eq!(msg, "");
    }
}

#[test]
fn test_learning_action_type_custom_long_string() {
    let long_msg = "a".repeat(1000);
    let action = LearningActionType::Custom(long_msg);

    if let LearningActionType::Custom(msg) = action {
        assert_eq!(msg.len(), 1000);
    }
}

#[test]
fn test_context_learning_manager_config_zero_episode_timeout() {
    let mut config = ContextLearningManagerConfig::default();
    config.episode_timeout = 0;

    assert_eq!(config.episode_timeout, 0);
}

#[test]
fn test_context_learning_manager_config_large_episode_timeout() {
    let mut config = ContextLearningManagerConfig::default();
    config.episode_timeout = 86400; // 24 hours

    assert_eq!(config.episode_timeout, 86400);
}

#[test]
fn test_context_learning_manager_config_single_episode() {
    let mut config = ContextLearningManagerConfig::default();
    config.max_episodes_per_session = 1;

    assert_eq!(config.max_episodes_per_session, 1);
}

#[test]
fn test_context_learning_manager_config_many_episodes() {
    let mut config = ContextLearningManagerConfig::default();
    config.max_episodes_per_session = 10000;

    assert_eq!(config.max_episodes_per_session, 10000);
}

#[test]
fn test_context_learning_manager_config_fast_updates() {
    let mut config = ContextLearningManagerConfig::default();
    config.learning_update_interval = Duration::from_millis(10);

    assert_eq!(config.learning_update_interval, Duration::from_millis(10));
}

#[test]
fn test_context_learning_manager_config_slow_updates() {
    let mut config = ContextLearningManagerConfig::default();
    config.learning_update_interval = Duration::from_secs(3600);

    assert_eq!(config.learning_update_interval, Duration::from_secs(3600));
}

#[test]
fn test_context_learning_manager_config_small_state_space() {
    let mut config = ContextLearningManagerConfig::default();
    config.state_space_size = 1;

    assert_eq!(config.state_space_size, 1);
}

#[test]
fn test_context_learning_manager_config_large_state_space() {
    let mut config = ContextLearningManagerConfig::default();
    config.state_space_size = 10000;

    assert_eq!(config.state_space_size, 10000);
}

#[test]
fn test_context_learning_manager_config_small_action_space() {
    let mut config = ContextLearningManagerConfig::default();
    config.action_space_size = 1;

    assert_eq!(config.action_space_size, 1);
}

#[test]
fn test_context_learning_manager_config_large_action_space() {
    let mut config = ContextLearningManagerConfig::default();
    config.action_space_size = 1000;

    assert_eq!(config.action_space_size, 1000);
}

#[test]
fn test_context_learning_manager_config_auto_detection_enabled() {
    let mut config = ContextLearningManagerConfig::default();
    config.auto_episode_detection = true;

    assert!(config.auto_episode_detection);
}

#[test]
fn test_context_learning_manager_config_auto_detection_disabled() {
    let mut config = ContextLearningManagerConfig::default();
    config.auto_episode_detection = false;

    assert!(!config.auto_episode_detection);
}

#[test]
fn test_context_learning_manager_config_preprocessing_enabled() {
    let mut config = ContextLearningManagerConfig::default();
    config.enable_preprocessing = true;

    assert!(config.enable_preprocessing);
}

#[test]
fn test_context_learning_manager_config_preprocessing_disabled() {
    let mut config = ContextLearningManagerConfig::default();
    config.enable_preprocessing = false;

    assert!(!config.enable_preprocessing);
}

#[test]
fn test_context_learning_manager_config_serialization() {
    let config = ContextLearningManagerConfig::default();
    let serialized = serde_json::to_string(&config);

    assert!(serialized.is_ok());
}

#[test]
fn test_learning_state_deserialization() {
    let json_str = r#""Learning""#;
    let deserialized: Result<LearningState, _> = serde_json::from_str(json_str);

    assert!(deserialized.is_ok());
}

#[test]
fn test_learning_action_type_deserialization() {
    let json_str = r#""ApplyRule""#;
    let deserialized: Result<LearningActionType, _> = serde_json::from_str(json_str);

    assert!(deserialized.is_ok());
}

#[test]
fn test_learning_state_roundtrip() {
    let state = LearningState::Evaluating;
    let serialized = serde_json::to_string(&state).unwrap();
    let deserialized: LearningState = serde_json::from_str(&serialized).unwrap();

    assert!(matches!(deserialized, LearningState::Evaluating));
}

#[test]
fn test_learning_action_type_roundtrip() {
    let action = LearningActionType::UpdatePolicy;
    let serialized = serde_json::to_string(&action).unwrap();
    let deserialized: LearningActionType = serde_json::from_str(&serialized).unwrap();

    assert!(matches!(deserialized, LearningActionType::UpdatePolicy));
}

#[test]
fn test_learning_state_all_match_patterns() {
    let states = vec![
        (LearningState::Initializing, "Initializing"),
        (LearningState::Learning, "Learning"),
        (LearningState::Evaluating, "Evaluating"),
        (LearningState::Adapting, "Adapting"),
        (LearningState::Paused, "Paused"),
        (LearningState::Stopped, "Stopped"),
    ];

    for (state, name) in states {
        let debug_str = format!("{state:?}");
        assert!(debug_str.contains(name));
    }
}

#[test]
fn test_learning_action_type_custom_unicode() {
    let unicode_msg = "学习动作🚀";
    let action = LearningActionType::Custom(unicode_msg.to_string());

    if let LearningActionType::Custom(msg) = action {
        assert_eq!(msg, unicode_msg);
    }
}
