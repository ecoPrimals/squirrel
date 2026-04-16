// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Targeted coverage for [`super::integration::LearningIntegration`] internals and branches.

use super::LearningIntegration;
use super::test_helpers;
use crate::error::ContextError;
use std::sync::Arc;

fn disabled_rl_config() -> Arc<super::LearningSystemConfig> {
    let mut c = test_helpers::create_test_learning_config();
    c.enable_reinforcement_learning = false;
    c.enable_learning_metrics = false;
    c.enable_adaptive_rules = false;
    Arc::new(c)
}

#[tokio::test]
async fn trigger_episode_errors_when_context_learning_disabled() {
    let integration = LearningIntegration::new(disabled_rl_config())
        .await
        .expect("create");

    let err = integration
        .trigger_learning_episode("ctx")
        .await
        .expect_err("expected NotInitialized");
    assert!(matches!(err, ContextError::NotInitialized));
}

#[tokio::test]
async fn calculate_reward_errors_when_reward_system_disabled() {
    let integration = LearningIntegration::new(disabled_rl_config())
        .await
        .expect("create");

    let err = integration
        .calculate_reward("ctx", serde_json::json!({}))
        .await
        .expect_err("expected NotInitialized");
    assert!(matches!(err, ContextError::NotInitialized));
}

#[tokio::test]
async fn monitor_context_tick_ok_with_empty_sessions() {
    let config = Arc::new(test_helpers::create_test_learning_config());
    let integration = LearningIntegration::new(config).await.expect("create");

    integration
        .test_monitor_context_changes()
        .await
        .expect("monitor");
}

#[tokio::test]
async fn monitor_rule_performance_ok_without_adaptive_rules() {
    let config = disabled_rl_config();
    let integration = LearningIntegration::new(config).await.expect("create");

    integration
        .test_monitor_rule_performance()
        .await
        .expect("rules");
}

#[tokio::test]
async fn synchronize_learning_ok_without_metrics() {
    let config = disabled_rl_config();
    let integration = LearningIntegration::new(config).await.expect("create");

    integration.test_synchronize_learning().await.expect("sync");
}

#[tokio::test]
async fn get_learning_status_includes_partial_components() {
    let mut c = test_helpers::create_test_learning_config();
    c.enable_reinforcement_learning = true;
    c.enable_learning_metrics = true;
    c.enable_adaptive_rules = false;

    let integration = LearningIntegration::new(Arc::new(c)).await.expect("create");

    let status = integration.get_learning_status().await.expect("status");
    assert!(status.get("components").is_some());
    assert!(status["components"].get("learning_engine").is_some());
    assert!(status["components"].get("learning_metrics").is_some());
}
