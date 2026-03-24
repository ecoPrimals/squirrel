// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(clippy::unwrap_used, clippy::expect_used)] // Test code: explicit unwrap/expect and local lint noise
//! Integration tests for AI routing functionality

use squirrel_ai_tools::common::AICapabilities;
use squirrel_ai_tools::common::capability::{AITask, SecurityRequirements, TaskType};
use squirrel_ai_tools::router::{
    AIRouter, ProviderSelector, RequestContext, RouterConfig, RoutingStrategy,
};

// ========== Router Configuration Tests ==========

#[test]
fn test_router_config_default() {
    let config = RouterConfig::default();
    assert!(matches!(config.routing_strategy, RoutingStrategy::BestFit));
    assert!(config.allow_remote_routing);
}

#[test]
fn test_router_config_custom() {
    let config = RouterConfig {
        routing_strategy: RoutingStrategy::LowestCost,
        allow_remote_routing: false,
        routing_timeout_ms: 30000,
        max_routing_attempts: 5,
        ..Default::default()
    };

    assert!(matches!(
        config.routing_strategy,
        RoutingStrategy::LowestCost
    ));
    assert!(!config.allow_remote_routing);
    assert_eq!(config.routing_timeout_ms, 30000);
    assert_eq!(config.max_routing_attempts, 5);
}

// ========== Router Creation Tests ==========

#[test]
fn test_router_new() {
    let config = RouterConfig::default();
    let router = AIRouter::new(config);

    assert_eq!(router.get_provider_count(), 0);
    assert!(router.is_remote_routing_enabled());
}

#[test]
fn test_router_with_strategy() {
    let config = RouterConfig {
        routing_strategy: RoutingStrategy::HighestPriority,
        ..Default::default()
    };
    let router = AIRouter::new(config);

    assert_eq!(
        router.get_routing_strategy(),
        RoutingStrategy::HighestPriority
    );
}

// ========== Provider Management Tests ==========

#[test]
fn test_router_list_providers_empty() {
    let config = RouterConfig::default();
    let router = AIRouter::new(config);

    let providers = router.list_providers();
    assert_eq!(providers.len(), 0);
}

#[test]
fn test_router_has_provider() {
    let config = RouterConfig::default();
    let router = AIRouter::new(config);

    assert!(!router.has_provider("gpt-4"));
}

// ========== Request Context Tests ==========

#[test]
fn test_request_context_new() {
    let task = AITask {
        task_type: TaskType::TextGeneration,
        required_model_type: None,
        min_context_size: None,
        requires_streaming: false,
        requires_function_calling: false,
        requires_tool_use: false,
        security_requirements: SecurityRequirements::default(),
        complexity_score: None,
        priority: 50,
    };

    let context = RequestContext::new(task.clone());

    assert_eq!(context.task, task);
    assert!(context.user_id.is_none());
    assert!(context.session_id.is_none());
}

#[test]
fn test_request_context_builder() {
    let task = AITask {
        task_type: TaskType::TextGeneration,
        required_model_type: None,
        min_context_size: None,
        requires_streaming: false,
        requires_function_calling: false,
        requires_tool_use: false,
        security_requirements: SecurityRequirements::default(),
        complexity_score: None,
        priority: 50,
    };

    let session_id = uuid::Uuid::new_v4();
    let context = RequestContext::new(task)
        .with_user_id("user123")
        .with_session_id(session_id);

    assert_eq!(context.user_id, Some("user123".to_string()));
    assert_eq!(context.session_id, Some(session_id));
}

// ========== Routing Strategy Tests ==========

#[test]
fn test_routing_strategy_variants() {
    // Test that all routing strategies can be created
    let strategies = vec![
        RoutingStrategy::FirstMatch,
        RoutingStrategy::HighestPriority,
        RoutingStrategy::LowestLatency,
        RoutingStrategy::LowestCost,
        RoutingStrategy::BestFit,
        RoutingStrategy::LowestLatency,
        RoutingStrategy::RoundRobin,
        RoutingStrategy::Random,
    ];

    for strategy in strategies {
        let config = RouterConfig {
            routing_strategy: strategy,
            ..Default::default()
        };
        let router = AIRouter::new(config);
        assert_eq!(router.get_routing_strategy(), strategy);
    }
}

// ========== Capability Matching Tests ==========

#[test]
fn test_task_matches_text_generation() {
    let task = AITask {
        task_type: TaskType::TextGeneration,
        required_model_type: None,
        min_context_size: None,
        requires_streaming: false,
        requires_function_calling: false,
        requires_tool_use: false,
        security_requirements: SecurityRequirements::default(),
        complexity_score: None,
        priority: 50,
    };

    let mut caps = AICapabilities::default();
    caps.add_task_type(TaskType::TextGeneration);

    assert!(squirrel_ai_tools::router::task_matches_capabilities(
        &task, &caps
    ));
}

#[test]
fn test_task_does_not_match_missing_capability() {
    let task = AITask {
        task_type: TaskType::ImageGeneration,
        required_model_type: None,
        min_context_size: None,
        requires_streaming: false,
        requires_function_calling: false,
        requires_tool_use: false,
        security_requirements: SecurityRequirements::default(),
        complexity_score: None,
        priority: 50,
    };

    let mut caps = AICapabilities::default();
    caps.add_task_type(TaskType::TextGeneration);

    assert!(!squirrel_ai_tools::router::task_matches_capabilities(
        &task, &caps
    ));
}

// ========== Capability Registry Tests ==========

#[test]
fn test_router_registry_access() {
    let config = RouterConfig::default();
    let router = AIRouter::new(config);

    // Get access to the capability registry
    let registry = router.registry();

    // Verify we can access it
    assert!(registry.list_providers().is_empty());
}

// ========== Error Handling Tests ==========

#[test]
fn test_unregister_nonexistent_provider() {
    let config = RouterConfig::default();
    let router = AIRouter::new(config);

    let result = router.unregister_provider("nonexistent");
    // unregister_provider may succeed even if provider doesn't exist (idempotent)
    // Just verify it doesn't panic
    let _ = result;
}

// ========== Router Stats Tests ==========

#[tokio::test]
async fn test_router_stats() {
    let config = RouterConfig::default();
    let router = AIRouter::new(config);

    let stats = router.get_stats();

    assert_eq!(stats.total_requests, 0);
    assert_eq!(stats.successful_requests, 0);
    assert_eq!(stats.failed_requests, 0);
    assert_eq!(stats.provider_usage.len(), 0);
}

// ========== Optimization Utils Tests ==========

#[test]
fn test_provider_selector_exists() {
    // Test that provider selector is accessible
    let _selector = ProviderSelector::new();
}

// ========== Task Creation Tests ==========

#[test]
fn test_create_text_generation_task() {
    let config = RouterConfig::default();
    let router = AIRouter::new(config);

    let task = router.create_text_generation_task();
    assert_eq!(task.task_type, TaskType::TextGeneration);
    assert!(task.min_context_size.is_none());
    assert!(!task.requires_streaming);
}

// ========== Complex Integration Tests ==========

#[test]
fn test_router_configuration_update() {
    let config = RouterConfig::default();
    let mut router = AIRouter::new(config);

    assert_eq!(router.get_routing_strategy(), RoutingStrategy::BestFit);

    router.set_routing_strategy(RoutingStrategy::LowestCost);
    assert_eq!(router.get_routing_strategy(), RoutingStrategy::LowestCost);
}

#[test]
fn test_router_with_different_strategies() {
    let strategies = vec![
        RoutingStrategy::FirstMatch,
        RoutingStrategy::BestFit,
        RoutingStrategy::LowestCost,
        RoutingStrategy::HighestPriority,
        RoutingStrategy::LowestLatency,
        RoutingStrategy::RoundRobin,
        RoutingStrategy::Random,
    ];

    for strategy in strategies {
        let config = RouterConfig {
            routing_strategy: strategy,
            ..Default::default()
        };
        let router = AIRouter::new(config);

        assert_eq!(router.get_routing_strategy(), strategy);
    }
}

// ========== Remote Routing Tests ==========

#[test]
fn test_remote_routing_configuration() {
    let config_enabled = RouterConfig {
        allow_remote_routing: true,
        ..Default::default()
    };
    let router_enabled = AIRouter::new(config_enabled);
    assert!(router_enabled.is_remote_routing_enabled());

    let config_disabled = RouterConfig {
        allow_remote_routing: false,
        ..Default::default()
    };
    let router_disabled = AIRouter::new(config_disabled);
    assert!(!router_disabled.is_remote_routing_enabled());
}
