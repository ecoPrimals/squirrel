// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! LearningIntegration-specific tests — lifecycle, config, state, reward feedback.

use super::*;
use crate::learning::test_helpers;
use std::sync::Arc;

#[tokio::test]
async fn test_learning_integration_new_with_full_config() {
    let config = Arc::new(test_helpers::create_full_test_config());
    let integration = LearningIntegration::new(config).await;

    assert!(
        integration.is_ok(),
        "LearningIntegration creation should succeed with full config"
    );
}

#[tokio::test]
async fn test_learning_integration_get_state() {
    let config = Arc::new(test_helpers::create_full_test_config());
    let integration = LearningIntegration::new(config)
        .await
        .expect("Failed to create integration");

    let state = integration.get_state().await;
    assert!(!state.active_integrations.is_empty() || state.errors.is_empty());
}

#[tokio::test]
async fn test_learning_integration_get_stats() {
    let config = Arc::new(test_helpers::create_full_test_config());
    let integration = LearningIntegration::new(config)
        .await
        .expect("Failed to create integration");

    let stats = integration.get_stats().await;
    assert_eq!(stats.total_operations, 0);
    assert_eq!(stats.successful_operations, 0);
    assert_eq!(stats.failed_operations, 0);
}

#[tokio::test]
async fn test_learning_integration_initialize() {
    let config = Arc::new(test_helpers::create_full_test_config());
    let integration = LearningIntegration::new(config)
        .await
        .expect("Failed to create integration");

    let result = integration.initialize().await;
    assert!(result.is_ok(), "Integration initialize should succeed");
}

#[tokio::test]
async fn test_learning_integration_trigger_learning_episode() {
    let config = Arc::new(test_helpers::create_full_test_config());
    let integration = LearningIntegration::new(config)
        .await
        .expect("Failed to create integration");

    let result = integration.trigger_learning_episode("test_context").await;

    assert!(
        result.is_ok(),
        "Trigger learning episode should succeed: {:?}",
        result.err()
    );
    let episode_id = result.expect("Should have episode id");
    assert!(!episode_id.is_empty());
}

#[tokio::test]
async fn test_learning_integration_calculate_reward() {
    let config = Arc::new(test_helpers::create_full_test_config());
    let integration = LearningIntegration::new(config)
        .await
        .expect("Failed to create integration");

    let result = integration
        .calculate_reward("test_context", serde_json::json!({"action": "test"}))
        .await;

    assert!(
        result.is_ok(),
        "Calculate reward should succeed: {:?}",
        result.err()
    );
    let reward = result.expect("Should have reward");
    assert!(reward.is_finite());
}

#[tokio::test]
async fn test_learning_integration_get_learning_status() {
    let config = Arc::new(test_helpers::create_full_test_config());
    let integration = LearningIntegration::new(config)
        .await
        .expect("Failed to create integration");

    let result = integration.get_learning_status().await;
    assert!(result.is_ok(), "Get learning status should succeed");

    let status = result.expect("Should have status");
    assert!(status.get("integration_state").is_some());
    assert!(status.get("integration_stats").is_some());
    assert!(status.get("components").is_some());
}

#[tokio::test]
async fn test_learning_integration_start_stop() {
    let config = Arc::new(test_helpers::create_full_test_config());
    let integration = LearningIntegration::new(config)
        .await
        .expect("Failed to create integration");

    integration
        .initialize()
        .await
        .expect("Failed to initialize");

    integration.start().await.expect("Failed to start");
    let state = integration.get_state().await;
    assert!(matches!(
        state.status,
        super::integration::IntegrationStatus::Active
    ));

    integration.stop().await.expect("Failed to stop");
    let state = integration.get_state().await;
    assert!(matches!(
        state.status,
        super::integration::IntegrationStatus::Stopped
    ));
}

#[tokio::test]
async fn test_learning_integration_config_default() {
    let config = super::integration::LearningIntegrationConfig::default();

    assert!(config.enable_context_manager);
    assert!(config.enable_rule_manager);
    assert!(config.enable_visualization);
    assert!(config.enable_auto_triggers);
    assert_eq!(config.update_interval, std::time::Duration::from_secs(30));
}

#[tokio::test]
async fn test_trigger_thresholds_default() {
    let thresholds = super::integration::TriggerThresholds::default();

    assert_eq!(thresholds.min_context_changes, 10);
    assert_eq!(thresholds.min_rule_applications, 5);
    assert!((thresholds.error_rate_threshold - 0.2).abs() < 1e-9);
    assert!((thresholds.performance_threshold - 0.7).abs() < 1e-9);
}

#[tokio::test]
async fn test_integration_stats_after_operations() {
    let config = Arc::new(test_helpers::create_full_test_config());
    let integration = LearningIntegration::new(config)
        .await
        .expect("Failed to create integration");

    let _ = integration.trigger_learning_episode("ctx1").await;
    let _ = integration
        .calculate_reward("ctx1", serde_json::json!({}))
        .await;

    let stats = integration.get_stats().await;
    assert!(stats.total_operations >= 1);
    assert!(stats.learning_episodes >= 1);
}

#[tokio::test]
async fn test_learning_integration_minimal_config() {
    let mut config = test_helpers::create_test_learning_config();
    config.enable_reinforcement_learning = true;
    config.enable_adaptive_rules = false;
    config.enable_learning_metrics = false;

    let config = Arc::new(config);
    let integration = LearningIntegration::new(config).await;

    assert!(
        integration.is_ok(),
        "Integration should work with minimal config"
    );
}

#[tokio::test]
async fn test_integration_state_serialization() {
    use super::integration::{IntegrationError, IntegrationState, IntegrationStatus};

    let state = IntegrationState {
        status: IntegrationStatus::Active,
        last_update: chrono::Utc::now(),
        active_integrations: vec!["context".to_string(), "rules".to_string()],
        errors: vec![IntegrationError {
            id: "err1".to_string(),
            error_type: "test".to_string(),
            message: "Test error".to_string(),
            timestamp: chrono::Utc::now(),
            component: "test_component".to_string(),
        }],
    };

    let serialized = serde_json::to_string(&state).expect("Should serialize");
    assert!(serialized.contains("Active"));
    assert!(serialized.contains("err1"));
}

#[tokio::test]
async fn test_integration_status_variants() {
    use super::integration::IntegrationStatus;

    let statuses = [
        IntegrationStatus::Initializing,
        IntegrationStatus::Active,
        IntegrationStatus::Paused,
        IntegrationStatus::Stopped,
        IntegrationStatus::Error,
    ];

    for status in statuses {
        let debug_str = format!("{:?}", status);
        assert!(!debug_str.is_empty());
    }
}

#[tokio::test]
async fn test_learning_integration_rl_disabled_episode_and_reward_unavailable() {
    let mut config = test_helpers::create_test_learning_config();
    config.enable_reinforcement_learning = false;
    let config = Arc::new(config);
    let integration = LearningIntegration::new(config)
        .await
        .expect("integration with RL off should still construct");
    assert!(integration.trigger_learning_episode("ctx").await.is_err());
    assert!(
        integration
            .calculate_reward("ctx", serde_json::json!({}))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn test_learning_integration_setter_hooks_keep_running() {
    let config = Arc::new(test_helpers::create_full_test_config());
    let mut integration = LearningIntegration::new(config)
        .await
        .expect("create integration");
    integration.set_context_manager(Arc::new(crate::manager::ContextManager::new()));
    integration.set_rule_manager(Arc::new(crate::rules::RuleManager::new("./rules")));
    let _ = integration.get_state().await;
}

#[tokio::test]
async fn test_learning_integration_initial_state_when_rl_disabled() {
    let mut config = test_helpers::create_test_learning_config();
    config.enable_reinforcement_learning = false;
    let config = Arc::new(config);
    let integration = LearningIntegration::new(config)
        .await
        .expect("constructs without RL components");
    let state = integration.get_state().await;
    assert!(matches!(
        state.status,
        super::integration::IntegrationStatus::Initializing
    ));
}

#[tokio::test]
async fn test_learning_integration_feedback_trigger_episode_then_reward_updates_stats() {
    let config = Arc::new(test_helpers::create_full_test_config());
    let integration = LearningIntegration::new(config).await.expect("integration");
    let before = integration.get_stats().await;
    let eid = integration
        .trigger_learning_episode("ctx-feedback")
        .await
        .expect("episode");
    assert!(!eid.is_empty());
    let reward = integration
        .calculate_reward("ctx-feedback", serde_json::json!({"op": "sync"}))
        .await
        .expect("reward");
    assert!(reward.is_finite());
    let after = integration.get_stats().await;
    assert!(after.learning_episodes > before.learning_episodes);
    assert!(after.total_operations > before.total_operations);
}

#[tokio::test]
async fn test_learning_integration_config_serde_roundtrip() {
    let cfg = super::integration::LearningIntegrationConfig::default();
    let json = serde_json::to_string(&cfg).expect("ser");
    let back: super::integration::LearningIntegrationConfig =
        serde_json::from_str(&json).expect("de");
    assert_eq!(back.update_interval, std::time::Duration::from_secs(30));
    assert_eq!(
        back.trigger_thresholds.min_context_changes,
        cfg.trigger_thresholds.min_context_changes
    );
}

#[tokio::test]
async fn test_learning_integration_start_runs_background_sync_tick() {
    let mut cfg = test_helpers::create_full_test_config();
    cfg.learning_update_interval = std::time::Duration::from_millis(30);
    let config = Arc::new(cfg);
    let integration = LearningIntegration::new(config).await.expect("integration");
    integration.initialize().await.expect("init");
    integration.start().await.expect("start");
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    integration.stop().await.expect("stop");
    let state = integration.get_state().await;
    assert!(matches!(
        state.status,
        super::integration::IntegrationStatus::Stopped
    ));
}
