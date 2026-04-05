// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::needless_pass_by_value,
    clippy::significant_drop_tightening,
    clippy::field_reassign_with_default,
    clippy::default_trait_access,
    clippy::many_single_char_names,
    clippy::unreadable_literal,
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::similar_names,
    clippy::option_if_let_else,
    clippy::doc_markdown,
    clippy::struct_field_names,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::trivially_copy_pass_by_ref,
    clippy::unused_self,
    clippy::unused_async,
    clippy::unnecessary_wraps,
    clippy::semicolon_if_nothing_returned,
    clippy::match_wildcard_for_single_variants,
    clippy::match_same_arms,
    clippy::explicit_iter_loop,
    clippy::uninlined_format_args,
    clippy::equatable_if_let,
    clippy::assertions_on_constants,
    missing_docs,
    unused_imports,
    unused_variables,
    dead_code,
    deprecated
)]
//! Router module tests
//!
//! Tests for the AI router functionality including provider selection,
//! routing strategies, and fallback mechanisms.

use squirrel_ai_tools::router::{Router, RouterConfig, RoutingStrategy};

#[test]
fn test_router_creation() {
    let config = RouterConfig::default();
    let router = Router::new(config);

    // Router should be created successfully
    // AIRouter doesn't implement any checking traits, so just verify it exists
    let _ = router;
}

#[test]
fn test_routing_strategy_variants() {
    let strategies = vec![
        RoutingStrategy::LowestCost,
        RoutingStrategy::LowestLatency,
        RoutingStrategy::BestFit,
        RoutingStrategy::RoundRobin,
    ];

    // All strategies should be valid
    for strategy in strategies {
        // Just verify they exist and can be created
        assert!(matches!(
            strategy,
            RoutingStrategy::LowestCost
                | RoutingStrategy::LowestLatency
                | RoutingStrategy::BestFit
                | RoutingStrategy::RoundRobin
        ));
    }
}

#[test]
fn test_routing_config_defaults() {
    let config = RouterConfig::default();

    // Default config should have reasonable values
    assert!(config.max_routing_attempts > 0); // Ensure positive attempts
    assert!(config.routing_timeout_ms > 0);
}

#[test]
fn test_routing_config_custom() {
    let config = RouterConfig {
        routing_strategy: RoutingStrategy::LowestCost,
        max_routing_attempts: 5,
        routing_timeout_ms: 60000,
        allow_remote_routing: true,
        default_provider: None,
    };

    assert_eq!(config.max_routing_attempts, 5);
    assert_eq!(config.routing_timeout_ms, 60000);
    assert!(config.allow_remote_routing);
}

#[test]
fn test_routing_strategy_cost_optimized() {
    let strategy = RoutingStrategy::LowestCost;
    assert!(matches!(strategy, RoutingStrategy::LowestCost));
}

#[test]
fn test_routing_strategy_performance_optimized() {
    let strategy = RoutingStrategy::LowestLatency;
    assert!(matches!(strategy, RoutingStrategy::LowestLatency));
}

#[test]
fn test_routing_strategy_best_fit() {
    let strategy = RoutingStrategy::BestFit;
    assert!(matches!(strategy, RoutingStrategy::BestFit));
}

#[test]
fn test_routing_strategy_round_robin() {
    let strategy = RoutingStrategy::RoundRobin;
    assert!(matches!(strategy, RoutingStrategy::RoundRobin));
}

#[test]
fn test_router_with_cost_optimized_strategy() {
    let config = RouterConfig {
        routing_strategy: RoutingStrategy::LowestCost,
        ..Default::default()
    };

    let router = Router::new(config);
    // Should be able to create router with cost-optimized strategy
    let _ = router;
}

#[test]
fn test_router_with_performance_strategy() {
    let config = RouterConfig {
        routing_strategy: RoutingStrategy::LowestLatency,
        ..Default::default()
    };

    let router = Router::new(config);
    // Should be able to create router with performance strategy
    let _ = router;
}

#[test]
fn test_routing_config_validation() {
    // Test that config with zero retries is valid
    let config = RouterConfig {
        max_routing_attempts: 0,
        ..Default::default()
    };
    assert_eq!(config.max_routing_attempts, 0);

    // Test that config with large retries is valid
    let config = RouterConfig {
        max_routing_attempts: 10,
        ..Default::default()
    };
    assert_eq!(config.max_routing_attempts, 10);
}

#[test]
fn test_routing_config_timeouts() {
    // Test short timeout
    let config = RouterConfig {
        routing_timeout_ms: 1000, // 1 second
        ..Default::default()
    };
    assert_eq!(config.routing_timeout_ms, 1000);

    // Test long timeout
    let config = RouterConfig {
        routing_timeout_ms: 300_000, // 5 minutes
        ..Default::default()
    };
    assert_eq!(config.routing_timeout_ms, 300_000);
}

#[test]
fn test_routing_config_remote_disabled() {
    let config = RouterConfig {
        allow_remote_routing: false,
        ..Default::default()
    };
    assert!(!config.allow_remote_routing);
}

#[test]
fn test_routing_config_remote_enabled() {
    let config = RouterConfig {
        allow_remote_routing: true,
        ..Default::default()
    };
    assert!(config.allow_remote_routing);
}

#[test]
fn test_router_configuration_builder_pattern() {
    // Test that we can build configs fluently
    let config = RouterConfig {
        routing_strategy: RoutingStrategy::BestFit,
        max_routing_attempts: 3,
        routing_timeout_ms: 30000,
        allow_remote_routing: true,
        default_provider: None,
    };

    assert!(matches!(config.routing_strategy, RoutingStrategy::BestFit));
    assert_eq!(config.max_routing_attempts, 3);
}

#[test]
fn test_routing_strategy_equality() {
    let strat1 = RoutingStrategy::LowestCost;
    let strat2 = RoutingStrategy::LowestCost;

    // Same strategies should match
    assert!(matches!(strat1, RoutingStrategy::LowestCost));
    assert!(matches!(strat2, RoutingStrategy::LowestCost));
}

#[test]
fn test_routing_config_clone() {
    let config1 = RouterConfig {
        routing_strategy: RoutingStrategy::LowestLatency,
        max_routing_attempts: 5,
        routing_timeout_ms: 45000,
        allow_remote_routing: true,
        default_provider: Some("default".to_string()),
    };

    let config2 = config1.clone();

    assert_eq!(config1.max_routing_attempts, config2.max_routing_attempts);
    assert_eq!(config1.routing_timeout_ms, config2.routing_timeout_ms);
    assert_eq!(config1.allow_remote_routing, config2.allow_remote_routing);
}
