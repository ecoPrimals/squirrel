// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![expect(
    clippy::expect_used,
    reason = "Router optimization tests use expect on test fixtures"
)]

use super::*;
use crate::AiClientImpl;
use crate::common::AIClient;
use crate::common::capability::{
    AICapabilities, AITask, CostTier, ModelType, PerformanceMetrics, RoutingPreferences,
    SecurityRequirements, TaskType,
};
use crate::router::harness::RouterHarnessClient;
use crate::router::types::RoutingHint;
use std::sync::Arc;
use std::sync::atomic::Ordering;

fn base_task() -> AITask {
    AITask {
        task_type: TaskType::TextGeneration,
        required_model_type: None,
        min_context_size: None,
        requires_streaming: false,
        requires_function_calling: false,
        requires_tool_use: false,
        security_requirements: SecurityRequirements::default(),
        complexity_score: None,
        priority: 50,
    }
}

#[test]
fn test_provider_selector_creation() {
    let selector = ProviderSelector::new();
    assert_eq!(selector.round_robin_index.load(Ordering::SeqCst), 0);
}

#[test]
fn test_provider_scorer_new_and_default() {
    let _ = ProviderScorer::new();
    let _ = ProviderScorer;
}

#[test]
fn test_empty_providers_error() {
    let selector = ProviderSelector::new();
    let context = crate::router::types::RequestContext::new(base_task());

    let result = selector.select_provider(
        vec![],
        &context,
        crate::router::types::RoutingStrategy::BestFit,
    );
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("No suitable providers")
    );
}

#[test]
fn test_single_provider_short_circuit() {
    let selector = ProviderSelector::new();
    let p = Arc::new(AiClientImpl::RouterHarness(RouterHarnessClient::new(
        "only",
    )));
    let ctx = crate::router::types::RequestContext::new(base_task());
    let out = selector
        .select_provider(
            vec![("only".to_string(), p)],
            &ctx,
            crate::router::types::RoutingStrategy::Random,
        )
        .expect("should succeed");
    assert_eq!(out.0, "only");
}

#[test]
fn test_first_match_and_highest_priority() {
    let selector = ProviderSelector::new();
    let ctx = crate::router::types::RequestContext::new(base_task());
    let low = Arc::new(AiClientImpl::RouterHarness(
        RouterHarnessClient::new("low").with_prefs(RoutingPreferences {
            priority: 10,
            ..RoutingPreferences::default()
        }),
    ));
    let high = Arc::new(AiClientImpl::RouterHarness(
        RouterHarnessClient::new("high").with_prefs(RoutingPreferences {
            priority: 99,
            ..RoutingPreferences::default()
        }),
    ));

    let first = selector
        .select_provider(
            vec![
                ("a".to_string(), low.clone()),
                ("b".to_string(), high.clone()),
            ],
            &ctx,
            crate::router::types::RoutingStrategy::FirstMatch,
        )
        .expect("should succeed");
    assert_eq!(first.0, "a");

    let best = selector
        .select_provider(
            vec![("a".to_string(), low), ("b".to_string(), high)],
            &ctx,
            crate::router::types::RoutingStrategy::HighestPriority,
        )
        .expect("should succeed");
    assert_eq!(best.0, "b");
}

#[test]
fn test_lowest_latency_and_lowest_cost() {
    let selector = ProviderSelector::new();
    let ctx = crate::router::types::RequestContext::new(base_task());

    let mut slow_caps = AICapabilities::new();
    slow_caps.add_task_type(TaskType::TextGeneration);
    slow_caps.max_context_size = 4096;
    slow_caps.performance_metrics.avg_latency_ms = Some(900);

    let mut fast_caps = AICapabilities::new();
    fast_caps.add_task_type(TaskType::TextGeneration);
    fast_caps.max_context_size = 4096;
    fast_caps.performance_metrics.avg_latency_ms = Some(50);

    let slow = Arc::new(AiClientImpl::RouterHarness(
        RouterHarnessClient::new("slow").with_caps(slow_caps),
    ));
    let fast = Arc::new(AiClientImpl::RouterHarness(
        RouterHarnessClient::new("fast").with_caps(fast_caps),
    ));

    let pick = selector
        .select_provider(
            vec![("slow".to_string(), slow), ("fast".to_string(), fast)],
            &ctx,
            crate::router::types::RoutingStrategy::LowestLatency,
        )
        .expect("should succeed");
    assert_eq!(pick.0, "fast");

    let cheap = Arc::new(AiClientImpl::RouterHarness(
        RouterHarnessClient::new("cheap").with_prefs(RoutingPreferences {
            cost_tier: CostTier::Free,
            ..RoutingPreferences::default()
        }),
    ));
    let pricey = Arc::new(AiClientImpl::RouterHarness(
        RouterHarnessClient::new("pricey").with_prefs(RoutingPreferences {
            cost_tier: CostTier::High,
            ..RoutingPreferences::default()
        }),
    ));

    let cost_pick = selector
        .select_provider(
            vec![("pricey".to_string(), pricey), ("cheap".to_string(), cheap)],
            &ctx,
            crate::router::types::RoutingStrategy::LowestCost,
        )
        .expect("should succeed");
    assert_eq!(cost_pick.0, "cheap");
}

#[test]
fn test_round_robin_advances() {
    let selector = ProviderSelector::new();
    let ctx = crate::router::types::RequestContext::new(base_task());
    let a = Arc::new(AiClientImpl::RouterHarness(RouterHarnessClient::new("a")));
    let b = Arc::new(AiClientImpl::RouterHarness(RouterHarnessClient::new("b")));
    let v = vec![("a".to_string(), a), ("b".to_string(), b)];
    let first = selector
        .select_provider(
            v.clone(),
            &ctx,
            crate::router::types::RoutingStrategy::RoundRobin,
        )
        .expect("should succeed")
        .0;
    let second = selector
        .select_provider(v, &ctx, crate::router::types::RoutingStrategy::RoundRobin)
        .expect("should succeed")
        .0;
    assert_ne!(first, second);
}

#[test]
fn test_best_fit_picks_higher_score() {
    let selector = ProviderSelector::new();
    let mut task = base_task();
    task.requires_streaming = true;
    let mut weak_caps = AICapabilities::new();
    weak_caps.add_task_type(TaskType::TextGeneration);
    weak_caps.max_context_size = 4096;
    weak_caps.supports_streaming = false;

    let mut strong_caps = AICapabilities::new();
    strong_caps.add_task_type(TaskType::TextGeneration);
    strong_caps.max_context_size = 16384;
    strong_caps.supports_streaming = true;
    strong_caps.performance_metrics.avg_latency_ms = Some(80);

    let weak = Arc::new(AiClientImpl::RouterHarness(
        RouterHarnessClient::new("weak").with_caps(weak_caps),
    ));
    let strong = Arc::new(AiClientImpl::RouterHarness(
        RouterHarnessClient::new("strong")
            .with_caps(strong_caps)
            .with_prefs(RoutingPreferences {
                priority: 95,
                ..RoutingPreferences::default()
            }),
    ));

    let ctx = crate::router::types::RequestContext::new(task);
    let best = selector
        .select_provider(
            vec![("weak".to_string(), weak), ("strong".to_string(), strong)],
            &ctx,
            crate::router::types::RoutingStrategy::BestFit,
        )
        .expect("should succeed");
    assert_eq!(best.0, "strong");
}

#[test]
fn test_filter_by_routing_hint() {
    let providers: Vec<(String, Arc<AiClientImpl>)> = vec![
        (
            "provider1".to_string(),
            Arc::new(AiClientImpl::RouterHarness(RouterHarnessClient::new(
                "provider1",
            ))),
        ),
        (
            "provider2".to_string(),
            Arc::new(AiClientImpl::RouterHarness(RouterHarnessClient::new(
                "provider2",
            ))),
        ),
    ];

    let context =
        crate::router::types::RequestContext::new(base_task()).with_routing_hint(RoutingHint {
            preferred_provider: Some("provider1".to_string()),
            preferred_model: None,
            allow_remote: None,
            max_latency_ms: None,
            max_cost_tier: None,
            priority: None,
        });

    let filtered = OptimizationUtils::filter_by_routing_hint(providers, &context);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].0, "provider1");
}

#[test]
fn test_filter_by_routing_hint_no_preference_returns_all() {
    let providers: Vec<(String, Arc<AiClientImpl>)> = vec![
        (
            "a".to_string(),
            Arc::new(AiClientImpl::RouterHarness(RouterHarnessClient::new("a"))),
        ),
        (
            "b".to_string(),
            Arc::new(AiClientImpl::RouterHarness(RouterHarnessClient::new("b"))),
        ),
    ];
    let ctx = crate::router::types::RequestContext::new(base_task());
    assert_eq!(
        OptimizationUtils::filter_by_routing_hint(providers, &ctx).len(),
        2
    );
}

#[test]
fn test_optimization_utils_sorts() {
    let a = Arc::new(AiClientImpl::RouterHarness(
        RouterHarnessClient::new("a").with_prefs(RoutingPreferences {
            priority: 10,
            cost_tier: CostTier::High,
            ..RoutingPreferences::default()
        }),
    ));
    let mut caps_b = AICapabilities::new();
    caps_b.add_task_type(TaskType::TextGeneration);
    caps_b.performance_metrics.avg_latency_ms = Some(500);
    let b = Arc::new(AiClientImpl::RouterHarness(
        RouterHarnessClient::new("b")
            .with_caps(caps_b)
            .with_prefs(RoutingPreferences {
                priority: 90,
                cost_tier: CostTier::Free,
                ..RoutingPreferences::default()
            }),
    ));

    let v = vec![
        ("x".to_string(), Arc::clone(&a)),
        ("y".to_string(), Arc::clone(&b)),
    ];
    let by_pri = OptimizationUtils::sort_by_priority(v.clone());
    assert_eq!(by_pri[0].0, "y");

    let by_cost = OptimizationUtils::sort_by_cost(v.clone());
    assert_eq!(by_cost[0].0, "y");

    let by_lat = OptimizationUtils::sort_by_latency(v);
    assert_eq!(by_lat[0].0, "x");
}

#[test]
fn test_provider_scorer_cost_tier_penalty_and_model_bonus() {
    let scorer = ProviderScorer::new();
    let p = Arc::new(AiClientImpl::RouterHarness(
        RouterHarnessClient::new("p")
            .with_default_model("special")
            .with_prefs(RoutingPreferences {
                priority: 50,
                cost_tier: CostTier::High,
                handles_sensitive_data: true,
                ..RoutingPreferences::default()
            }),
    ));

    let mut task = base_task();
    task.requires_streaming = true;
    task.security_requirements.contains_sensitive_data = true;
    let ctx_penalized =
        crate::router::types::RequestContext::new(task).with_routing_hint(RoutingHint {
            preferred_provider: None,
            preferred_model: Some("special".to_string()),
            allow_remote: None,
            max_latency_ms: Some(1000),
            max_cost_tier: Some(CostTier::Medium),
            priority: None,
        });

    let s_penalized = scorer.score_provider(&p, &ctx_penalized);
    assert!(s_penalized < 90);

    let ctx_latency_hit =
        crate::router::types::RequestContext::new(base_task()).with_routing_hint(RoutingHint {
            preferred_provider: None,
            preferred_model: None,
            allow_remote: None,
            max_latency_ms: Some(10),
            max_cost_tier: Some(CostTier::High),
            priority: None,
        });
    let s_latency = scorer.score_provider(&p, &ctx_latency_hit);
    assert!(
        s_latency
            < scorer.score_provider(&p, &crate::router::types::RequestContext::new(base_task()))
    );
}

#[test]
fn test_calculate_compatibility_and_performance_scores() {
    let scorer = ProviderScorer::new();
    let p = Arc::new(AiClientImpl::RouterHarness(RouterHarnessClient::new("p")));

    let mut task = base_task();
    task.required_model_type = Some(ModelType::LargeLanguageModel);
    task.min_context_size = Some(100);
    task.requires_streaming = true;
    let ctx = crate::router::types::RequestContext::new(task);

    let c = scorer.calculate_compatibility_score(&p, &ctx);
    assert!((0.0..=1.0).contains(&c));

    let mut caps = p.capabilities();
    caps.performance_metrics = PerformanceMetrics {
        avg_latency_ms: Some(100),
        avg_tokens_per_second: Some(500.0),
        success_rate: Some(0.95),
        ..Default::default()
    };
    let p2 = Arc::new(AiClientImpl::RouterHarness(
        RouterHarnessClient::new("p2").with_caps(caps),
    ));
    let perf = scorer.calculate_performance_score(&p2);
    assert!((0.0..=1.0).contains(&perf));
}
