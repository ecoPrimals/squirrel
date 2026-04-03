// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![expect(
    clippy::expect_used,
    reason = "Router optimization tests use expect on test fixtures"
)]

use super::*;
use crate::common::capability::{
    AICapabilities, AITask, CostTier, ModelType, PerformanceMetrics, RoutingPreferences,
    SecurityRequirements, TaskType,
};
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

/// Mock provider with configurable [`AICapabilities`] and [`RoutingPreferences`].
#[derive(Debug, Clone)]
struct MockClient {
    name: String,
    caps: AICapabilities,
    prefs: RoutingPreferences,
    default_model: String,
}

impl MockClient {
    fn new(name: &str) -> Self {
        let mut caps = AICapabilities::new();
        caps.add_task_type(TaskType::TextGeneration);
        caps.add_model_type(ModelType::LargeLanguageModel);
        caps.max_context_size = 8192;
        caps.supports_streaming = true;
        caps.supports_function_calling = true;
        caps.supports_tool_use = true;
        caps.performance_metrics.avg_latency_ms = Some(100);
        Self {
            name: name.to_string(),
            caps,
            prefs: RoutingPreferences::default(),
            default_model: "mock-model".to_string(),
        }
    }

    fn with_prefs(mut self, prefs: RoutingPreferences) -> Self {
        self.prefs = prefs;
        self
    }

    fn with_caps(mut self, caps: AICapabilities) -> Self {
        self.caps = caps;
        self
    }

    fn with_default_model(mut self, m: impl Into<String>) -> Self {
        self.default_model = m.into();
        self
    }
}

#[async_trait::async_trait]
impl crate::common::AIClient for MockClient {
    async fn get_capabilities(&self, _model: &str) -> crate::error::Result<AICapabilities> {
        Ok(self.caps.clone())
    }

    fn capabilities(&self) -> AICapabilities {
        self.caps.clone()
    }

    fn routing_preferences(&self) -> RoutingPreferences {
        self.prefs.clone()
    }

    fn provider_name(&self) -> &str {
        &self.name
    }

    async fn is_available(&self) -> bool {
        true
    }

    fn default_model(&self) -> &str {
        &self.default_model
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn list_models(&self) -> crate::error::Result<Vec<String>> {
        Ok(vec![format!("{}-model", self.name)])
    }

    async fn chat(
        &self,
        _request: crate::common::ChatRequest,
    ) -> crate::error::Result<crate::common::ChatResponse> {
        Ok(crate::common::ChatResponse {
            id: "mock-response".to_string(),
            model: self.default_model.clone(),
            choices: vec![crate::common::ChatChoice {
                index: 0,
                role: crate::common::MessageRole::Assistant,
                content: Some("Mock response".to_string()),
                finish_reason: Some("stop".to_string()),
                tool_calls: None,
            }],
            usage: None,
        })
    }

    async fn chat_stream(
        &self,
        _request: crate::common::ChatRequest,
    ) -> crate::error::Result<crate::common::ChatResponseStream> {
        Err(universal_error::tools::AIToolsError::Provider(
            "Streaming not supported in mock".to_string(),
        )
        .into())
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
    let p = Arc::new(MockClient::new("only")) as Arc<dyn crate::common::AIClient>;
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
    let low = Arc::new(MockClient::new("low").with_prefs(RoutingPreferences {
        priority: 10,
        ..RoutingPreferences::default()
    })) as Arc<dyn crate::common::AIClient>;
    let high = Arc::new(MockClient::new("high").with_prefs(RoutingPreferences {
        priority: 99,
        ..RoutingPreferences::default()
    })) as Arc<dyn crate::common::AIClient>;

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

    let slow =
        Arc::new(MockClient::new("slow").with_caps(slow_caps)) as Arc<dyn crate::common::AIClient>;
    let fast =
        Arc::new(MockClient::new("fast").with_caps(fast_caps)) as Arc<dyn crate::common::AIClient>;

    let pick = selector
        .select_provider(
            vec![("slow".to_string(), slow), ("fast".to_string(), fast)],
            &ctx,
            crate::router::types::RoutingStrategy::LowestLatency,
        )
        .expect("should succeed");
    assert_eq!(pick.0, "fast");

    let cheap = Arc::new(MockClient::new("cheap").with_prefs(RoutingPreferences {
        cost_tier: CostTier::Free,
        ..RoutingPreferences::default()
    })) as Arc<dyn crate::common::AIClient>;
    let pricey = Arc::new(MockClient::new("pricey").with_prefs(RoutingPreferences {
        cost_tier: CostTier::High,
        ..RoutingPreferences::default()
    })) as Arc<dyn crate::common::AIClient>;

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
    let a = Arc::new(MockClient::new("a")) as Arc<dyn crate::common::AIClient>;
    let b = Arc::new(MockClient::new("b")) as Arc<dyn crate::common::AIClient>;
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

    let weak =
        Arc::new(MockClient::new("weak").with_caps(weak_caps)) as Arc<dyn crate::common::AIClient>;
    let strong = Arc::new(MockClient::new("strong").with_caps(strong_caps).with_prefs(
        RoutingPreferences {
            priority: 95,
            ..RoutingPreferences::default()
        },
    )) as Arc<dyn crate::common::AIClient>;

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
    let providers: Vec<(String, Arc<dyn crate::common::AIClient>)> = vec![
        (
            "provider1".to_string(),
            Arc::new(MockClient::new("provider1")) as Arc<dyn crate::common::AIClient>,
        ),
        (
            "provider2".to_string(),
            Arc::new(MockClient::new("provider2")) as Arc<dyn crate::common::AIClient>,
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
    let providers: Vec<(String, Arc<dyn crate::common::AIClient>)> = vec![
        (
            "a".to_string(),
            Arc::new(MockClient::new("a")) as Arc<dyn crate::common::AIClient>,
        ),
        (
            "b".to_string(),
            Arc::new(MockClient::new("b")) as Arc<dyn crate::common::AIClient>,
        ),
    ];
    let ctx = crate::router::types::RequestContext::new(base_task());
    assert_eq!(
        OptimizationUtils::filter_by_routing_hint(providers.clone(), &ctx).len(),
        2
    );
}

#[test]
fn test_optimization_utils_sorts() {
    let a = Arc::new(MockClient::new("a").with_prefs(RoutingPreferences {
        priority: 10,
        cost_tier: CostTier::High,
        ..RoutingPreferences::default()
    })) as Arc<dyn crate::common::AIClient>;
    let mut caps_b = AICapabilities::new();
    caps_b.add_task_type(TaskType::TextGeneration);
    caps_b.performance_metrics.avg_latency_ms = Some(500);
    let b = Arc::new(
        MockClient::new("b")
            .with_caps(caps_b)
            .with_prefs(RoutingPreferences {
                priority: 90,
                cost_tier: CostTier::Free,
                ..RoutingPreferences::default()
            }),
    ) as Arc<dyn crate::common::AIClient>;

    let v = vec![("x".to_string(), a.clone()), ("y".to_string(), b.clone())];
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
    let p = Arc::new(
        MockClient::new("p")
            .with_default_model("special")
            .with_prefs(RoutingPreferences {
                priority: 50,
                cost_tier: CostTier::High,
                handles_sensitive_data: true,
                ..RoutingPreferences::default()
            }),
    ) as Arc<dyn crate::common::AIClient>;

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
    let p = Arc::new(MockClient::new("p")) as Arc<dyn crate::common::AIClient>;

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
    let p2 = Arc::new(MockClient::new("p2").with_caps(caps)) as Arc<dyn crate::common::AIClient>;
    let perf = scorer.calculate_performance_score(&p2);
    assert!((0.0..=1.0).contains(&perf));
}
