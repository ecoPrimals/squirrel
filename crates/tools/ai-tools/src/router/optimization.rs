// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Provider selection and optimization algorithms for AI routing.
//!
//! This module implements various strategies for selecting the best AI provider
//! for a given task, including scoring algorithms and routing optimizations.

use super::types::{RequestContext, RoutingStrategy};
use crate::Result;
use crate::common::AIClient;
use crate::error::Error;
use crate::float_helpers;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::debug;

/// Provider selection engine that implements various routing strategies
pub struct ProviderSelector {
    /// Round-robin state for providers
    round_robin_index: AtomicUsize,
}

impl ProviderSelector {
    /// Create a new provider selector
    #[must_use]
    pub const fn new() -> Self {
        Self {
            round_robin_index: AtomicUsize::new(0),
        }
    }

    /// Select a provider based on the routing strategy
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`] when `providers` is empty.
    pub fn select_provider(
        &self,
        providers: Vec<(String, Arc<dyn AIClient>)>,
        context: &RequestContext,
        strategy: RoutingStrategy,
    ) -> Result<(String, Arc<dyn AIClient>)> {
        if providers.is_empty() {
            return Err(Error::Configuration(
                "No suitable providers available".to_string(),
            ));
        }

        if providers.len() == 1 {
            let mut providers = providers;
            return Ok(providers.swap_remove(0));
        }

        match strategy {
            RoutingStrategy::FirstMatch => Ok(Self::select_first_match(&providers)),
            RoutingStrategy::HighestPriority => Ok(Self::select_highest_priority(&providers)),
            RoutingStrategy::LowestLatency => Ok(Self::select_lowest_latency(&providers)),
            RoutingStrategy::LowestCost => Ok(Self::select_lowest_cost(&providers)),
            RoutingStrategy::BestFit => Ok(Self::select_best_fit(providers, context)),
            RoutingStrategy::RoundRobin => Ok(self.select_round_robin(&providers)),
            RoutingStrategy::Random => Ok(Self::select_random(&providers)),
        }
    }

    /// Select the first provider in the list
    fn select_first_match(
        providers: &[(String, Arc<dyn AIClient>)],
    ) -> (String, Arc<dyn AIClient>) {
        debug!("Using FirstMatch strategy");
        let (id, client) = &providers[0];
        (id.clone(), Arc::clone(client))
    }

    /// Select the provider with the highest priority
    fn select_highest_priority(
        providers: &[(String, Arc<dyn AIClient>)],
    ) -> (String, Arc<dyn AIClient>) {
        debug!("Using HighestPriority strategy");
        let mut best_idx = 0;
        let mut best_priority = providers[0].1.routing_preferences().priority;

        for (i, (_, provider)) in providers.iter().enumerate().skip(1) {
            let priority = provider.routing_preferences().priority;
            if priority > best_priority {
                best_idx = i;
                best_priority = priority;
            }
        }

        let (id, client) = &providers[best_idx];
        (id.clone(), Arc::clone(client))
    }

    /// Select the provider with the lowest latency
    fn select_lowest_latency(
        providers: &[(String, Arc<dyn AIClient>)],
    ) -> (String, Arc<dyn AIClient>) {
        debug!("Using LowestLatency strategy");
        let mut best_idx = 0;
        let mut best_latency = providers[0]
            .1
            .capabilities()
            .performance_metrics
            .avg_latency_ms
            .unwrap_or(u64::MAX);

        for (i, (_, provider)) in providers.iter().enumerate().skip(1) {
            if let Some(latency) = provider.capabilities().performance_metrics.avg_latency_ms
                && latency < best_latency
            {
                best_idx = i;
                best_latency = latency;
            }
        }

        let (id, client) = &providers[best_idx];
        (id.clone(), Arc::clone(client))
    }

    /// Select the provider with the lowest cost tier
    fn select_lowest_cost(
        providers: &[(String, Arc<dyn AIClient>)],
    ) -> (String, Arc<dyn AIClient>) {
        debug!("Using LowestCost strategy");
        let mut best_idx = 0;
        let mut best_cost = providers[0].1.routing_preferences().cost_tier;

        for (i, (_, provider)) in providers.iter().enumerate().skip(1) {
            let cost = provider.routing_preferences().cost_tier;
            // Lower cost tier is better (Free < Low < Medium < High)
            if cost < best_cost {
                best_idx = i;
                best_cost = cost;
            }
        }

        let (id, client) = &providers[best_idx];
        (id.clone(), Arc::clone(client))
    }

    /// Select the provider that best matches the task requirements
    fn select_best_fit(
        providers: Vec<(String, Arc<dyn AIClient>)>,
        context: &RequestContext,
    ) -> (String, Arc<dyn AIClient>) {
        debug!("Using BestFit strategy");
        let scorer = ProviderScorer::new();

        // Score each provider based on how well it matches the task
        let mut scored_providers: Vec<(String, Arc<dyn AIClient>, u32)> = providers
            .into_iter()
            .map(|(id, provider)| {
                let score = scorer.score_provider(&provider, context);
                (id, provider, score)
            })
            .collect();

        // Sort by score (highest first)
        scored_providers.sort_by(|a, b| b.2.cmp(&a.2));

        debug!("Best fit provider score: {}", scored_providers[0].2);

        // Return the highest scoring provider (move out — no tuple clone)
        let best = scored_providers.swap_remove(0);
        (best.0, best.1)
    }

    /// Select a provider using round-robin
    fn select_round_robin(
        &self,
        providers: &[(String, Arc<dyn AIClient>)],
    ) -> (String, Arc<dyn AIClient>) {
        debug!("Using RoundRobin strategy");
        let index = self.round_robin_index.fetch_add(1, Ordering::SeqCst) % providers.len();
        let (id, client) = &providers[index];
        (id.clone(), Arc::clone(client))
    }

    /// Select a provider randomly
    fn select_random(providers: &[(String, Arc<dyn AIClient>)]) -> (String, Arc<dyn AIClient>) {
        debug!("Using Random strategy");
        let index = rand::random::<usize>() % providers.len();
        let (id, client) = &providers[index];
        (id.clone(), Arc::clone(client))
    }
}

impl Default for ProviderSelector {
    fn default() -> Self {
        Self::new()
    }
}

/// Provider scoring engine for evaluating how well a provider matches a task
pub struct ProviderScorer;

impl ProviderScorer {
    /// Create a new provider scorer
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Score a provider based on how well it matches the task
    pub fn score_provider(&self, provider: &Arc<dyn AIClient>, context: &RequestContext) -> u32 {
        let mut score = 0;
        let capabilities = provider.capabilities();
        let preferences = provider.routing_preferences();

        // Base score from priority
        score += u32::from(preferences.priority);

        // Bonus for supporting streaming if required
        if context.task.requires_streaming && capabilities.supports_streaming {
            score += 10;
        }

        // Bonus for supporting function calling if required
        if context.task.requires_function_calling && capabilities.supports_function_calling {
            score += 10;
        }

        // Bonus for supporting tool use if required
        if context.task.requires_tool_use && capabilities.supports_tool_use {
            score += 10;
        }

        // Bonus for having a large context window if required
        if let Some(required_size) = context.task.min_context_size
            && capabilities.max_context_size >= required_size
        {
            // Higher bonus for models with just enough context (to avoid over-provisioning)
            let req = required_size.max(1);
            let size_ratio = float_helpers::usize_to_f64_lossy(capabilities.max_context_size)
                / float_helpers::usize_to_f64_lossy(req);
            if size_ratio <= 1.5 {
                score += 15;
            } else if size_ratio <= 2.0 {
                score += 10;
            } else {
                score += 5;
            }
        }

        // Handle cost preferences if specified in routing hint
        if let Some(hint) = &context.routing_hint
            && let Some(ref max_cost_tier) = hint.max_cost_tier
        {
            // Penalize providers that exceed the cost tier limit
            if preferences.cost_tier > *max_cost_tier {
                score = score.saturating_sub(50);
            } else {
                // Bonus for being well under budget
                if preferences.cost_tier < *max_cost_tier {
                    score += 10;
                }
            }
        }

        // Bonus for handling sensitive data if required
        if context.task.security_requirements.contains_sensitive_data
            && preferences.handles_sensitive_data
        {
            score += 20;
        }

        // Apply routing hint preferences if present
        if let Some(hint) = &context.routing_hint {
            if let Some(preferred_model) = &hint.preferred_model
                && provider.default_model() == preferred_model
            {
                score += 25;
            }

            if let Some(max_latency) = hint.max_latency_ms
                && let Some(latency) = capabilities.performance_metrics.avg_latency_ms
            {
                if latency <= max_latency {
                    score += 15;
                } else {
                    score = score.saturating_sub(30);
                }
            }
        }

        debug!("Provider score: {}", score);
        score
    }

    /// Calculate compatibility score between task and provider capabilities
    pub fn calculate_compatibility_score(
        &self,
        provider: &Arc<dyn AIClient>,
        context: &RequestContext,
    ) -> f64 {
        let capabilities = provider.capabilities();
        let mut compatibility_factors = Vec::new();

        // Task type compatibility
        if capabilities.supports_task(&context.task.task_type) {
            compatibility_factors.push(1.0);
        } else {
            compatibility_factors.push(0.0);
        }

        // Model type compatibility
        if let Some(ref model_type) = context.task.required_model_type {
            if capabilities.supports_model_type(model_type) {
                compatibility_factors.push(1.0);
            } else {
                compatibility_factors.push(0.0);
            }
        } else {
            compatibility_factors.push(1.0); // No requirement means compatible
        }

        // Context size compatibility
        if let Some(required_size) = context.task.min_context_size {
            let req = required_size.max(1);
            let ratio = float_helpers::usize_to_f64_lossy(capabilities.max_context_size)
                / float_helpers::usize_to_f64_lossy(req);
            compatibility_factors.push(ratio.min(1.0));
        } else {
            compatibility_factors.push(1.0);
        }

        // Streaming compatibility
        if context.task.requires_streaming {
            if capabilities.supports_streaming {
                compatibility_factors.push(1.0);
            } else {
                compatibility_factors.push(0.0);
            }
        } else {
            compatibility_factors.push(1.0);
        }

        // Function calling compatibility
        if context.task.requires_function_calling {
            if capabilities.supports_function_calling {
                compatibility_factors.push(1.0);
            } else {
                compatibility_factors.push(0.0);
            }
        } else {
            compatibility_factors.push(1.0);
        }

        // Tool use compatibility
        if context.task.requires_tool_use {
            if capabilities.supports_tool_use {
                compatibility_factors.push(1.0);
            } else {
                compatibility_factors.push(0.0);
            }
        } else {
            compatibility_factors.push(1.0);
        }

        // Calculate geometric mean for overall compatibility
        let product: f64 = compatibility_factors.iter().product();
        let n = float_helpers::usize_to_f64_lossy(compatibility_factors.len().max(1));
        product.powf(1.0 / n)
    }

    /// Calculate performance score based on provider metrics
    pub fn calculate_performance_score(&self, provider: &Arc<dyn AIClient>) -> f64 {
        let capabilities = provider.capabilities();
        let mut performance_score = 0.0;

        // Latency score (lower is better)
        if let Some(latency) = capabilities.performance_metrics.avg_latency_ms {
            // Convert to score where 100ms = 1.0, 1000ms = 0.1, etc.
            performance_score += (100.0 / float_helpers::u64_to_f64_lossy(latency)).min(1.0);
        } else {
            performance_score += 0.5; // Default score for unknown latency
        }

        // Throughput score (higher is better)
        if let Some(throughput) = capabilities.performance_metrics.avg_tokens_per_second {
            // Convert to score where 1000 tokens/sec = 1.0
            performance_score += (throughput / 1000.0).min(1.0);
        } else {
            performance_score += 0.5; // Default score for unknown throughput
        }

        // Uptime score
        if let Some(uptime) = capabilities.performance_metrics.success_rate {
            performance_score += uptime / 100.0;
        } else {
            performance_score += 0.9; // Default to 90% uptime
        }

        // Normalize to 0-1 range
        performance_score / 3.0
    }
}

impl Default for ProviderScorer {
    fn default() -> Self {
        Self::new()
    }
}

/// Optimization utilities for provider selection
pub struct OptimizationUtils;

impl OptimizationUtils {
    /// Filter providers based on routing hints
    #[must_use]
    pub fn filter_by_routing_hint(
        providers: Vec<(String, Arc<dyn AIClient>)>,
        context: &RequestContext,
    ) -> Vec<(String, Arc<dyn AIClient>)> {
        if let Some(hint) = &context.routing_hint
            && let Some(preferred_provider) = &hint.preferred_provider
        {
            return providers
                .into_iter()
                .filter(|(id, _)| id == preferred_provider)
                .collect();
        }
        providers
    }

    /// Sort providers by priority
    #[must_use]
    pub fn sort_by_priority(
        mut providers: Vec<(String, Arc<dyn AIClient>)>,
    ) -> Vec<(String, Arc<dyn AIClient>)> {
        providers.sort_by(|a, b| {
            b.1.routing_preferences()
                .priority
                .cmp(&a.1.routing_preferences().priority)
        });
        providers
    }

    /// Sort providers by cost tier (lowest first)
    #[must_use]
    pub fn sort_by_cost(
        mut providers: Vec<(String, Arc<dyn AIClient>)>,
    ) -> Vec<(String, Arc<dyn AIClient>)> {
        providers.sort_by(|a, b| {
            a.1.routing_preferences()
                .cost_tier
                .cmp(&b.1.routing_preferences().cost_tier)
        });
        providers
    }

    /// Sort providers by latency (lowest first)
    #[must_use]
    pub fn sort_by_latency(
        mut providers: Vec<(String, Arc<dyn AIClient>)>,
    ) -> Vec<(String, Arc<dyn AIClient>)> {
        providers.sort_by(|a, b| {
            let latency_a =
                a.1.capabilities()
                    .performance_metrics
                    .avg_latency_ms
                    .unwrap_or(u64::MAX);
            let latency_b =
                b.1.capabilities()
                    .performance_metrics
                    .avg_latency_ms
                    .unwrap_or(u64::MAX);
            latency_a.cmp(&latency_b)
        });
        providers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::capability::{
        AICapabilities, AITask, CostTier, ModelType, PerformanceMetrics, RoutingPreferences,
        SecurityRequirements, TaskType,
    };
    use crate::router::types::RoutingHint;

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
        let context = RequestContext::new(base_task());

        let result = selector.select_provider(vec![], &context, RoutingStrategy::BestFit);
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
        let p = Arc::new(MockClient::new("only")) as Arc<dyn AIClient>;
        let ctx = RequestContext::new(base_task());
        let out = selector
            .select_provider(vec![("only".to_string(), p)], &ctx, RoutingStrategy::Random)
            .expect("should succeed");
        assert_eq!(out.0, "only");
    }

    #[test]
    fn test_first_match_and_highest_priority() {
        let selector = ProviderSelector::new();
        let ctx = RequestContext::new(base_task());
        let low = Arc::new(MockClient::new("low").with_prefs(RoutingPreferences {
            priority: 10,
            ..RoutingPreferences::default()
        })) as Arc<dyn AIClient>;
        let high = Arc::new(MockClient::new("high").with_prefs(RoutingPreferences {
            priority: 99,
            ..RoutingPreferences::default()
        })) as Arc<dyn AIClient>;

        let first = selector
            .select_provider(
                vec![
                    ("a".to_string(), low.clone()),
                    ("b".to_string(), high.clone()),
                ],
                &ctx,
                RoutingStrategy::FirstMatch,
            )
            .expect("should succeed");
        assert_eq!(first.0, "a");

        let best = selector
            .select_provider(
                vec![("a".to_string(), low), ("b".to_string(), high)],
                &ctx,
                RoutingStrategy::HighestPriority,
            )
            .expect("should succeed");
        assert_eq!(best.0, "b");
    }

    #[test]
    fn test_lowest_latency_and_lowest_cost() {
        let selector = ProviderSelector::new();
        let ctx = RequestContext::new(base_task());

        let mut slow_caps = AICapabilities::new();
        slow_caps.add_task_type(TaskType::TextGeneration);
        slow_caps.max_context_size = 4096;
        slow_caps.performance_metrics.avg_latency_ms = Some(900);

        let mut fast_caps = AICapabilities::new();
        fast_caps.add_task_type(TaskType::TextGeneration);
        fast_caps.max_context_size = 4096;
        fast_caps.performance_metrics.avg_latency_ms = Some(50);

        let slow = Arc::new(MockClient::new("slow").with_caps(slow_caps)) as Arc<dyn AIClient>;
        let fast = Arc::new(MockClient::new("fast").with_caps(fast_caps)) as Arc<dyn AIClient>;

        let pick = selector
            .select_provider(
                vec![("slow".to_string(), slow), ("fast".to_string(), fast)],
                &ctx,
                RoutingStrategy::LowestLatency,
            )
            .expect("should succeed");
        assert_eq!(pick.0, "fast");

        let cheap = Arc::new(MockClient::new("cheap").with_prefs(RoutingPreferences {
            cost_tier: CostTier::Free,
            ..RoutingPreferences::default()
        })) as Arc<dyn AIClient>;
        let pricey = Arc::new(MockClient::new("pricey").with_prefs(RoutingPreferences {
            cost_tier: CostTier::High,
            ..RoutingPreferences::default()
        })) as Arc<dyn AIClient>;

        let cost_pick = selector
            .select_provider(
                vec![("pricey".to_string(), pricey), ("cheap".to_string(), cheap)],
                &ctx,
                RoutingStrategy::LowestCost,
            )
            .expect("should succeed");
        assert_eq!(cost_pick.0, "cheap");
    }

    #[test]
    fn test_round_robin_advances() {
        let selector = ProviderSelector::new();
        let ctx = RequestContext::new(base_task());
        let a = Arc::new(MockClient::new("a")) as Arc<dyn AIClient>;
        let b = Arc::new(MockClient::new("b")) as Arc<dyn AIClient>;
        let v = vec![("a".to_string(), a), ("b".to_string(), b)];
        let first = selector
            .select_provider(v.clone(), &ctx, RoutingStrategy::RoundRobin)
            .expect("should succeed")
            .0;
        let second = selector
            .select_provider(v, &ctx, RoutingStrategy::RoundRobin)
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

        let weak = Arc::new(MockClient::new("weak").with_caps(weak_caps)) as Arc<dyn AIClient>;
        let strong = Arc::new(MockClient::new("strong").with_caps(strong_caps).with_prefs(
            RoutingPreferences {
                priority: 95,
                ..RoutingPreferences::default()
            },
        )) as Arc<dyn AIClient>;

        let ctx = RequestContext::new(task);
        let best = selector
            .select_provider(
                vec![("weak".to_string(), weak), ("strong".to_string(), strong)],
                &ctx,
                RoutingStrategy::BestFit,
            )
            .expect("should succeed");
        assert_eq!(best.0, "strong");
    }

    #[test]
    fn test_filter_by_routing_hint() {
        let providers: Vec<(String, Arc<dyn AIClient>)> = vec![
            (
                "provider1".to_string(),
                Arc::new(MockClient::new("provider1")) as Arc<dyn AIClient>,
            ),
            (
                "provider2".to_string(),
                Arc::new(MockClient::new("provider2")) as Arc<dyn AIClient>,
            ),
        ];

        let context = RequestContext::new(base_task()).with_routing_hint(RoutingHint {
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
        let providers: Vec<(String, Arc<dyn AIClient>)> = vec![
            (
                "a".to_string(),
                Arc::new(MockClient::new("a")) as Arc<dyn AIClient>,
            ),
            (
                "b".to_string(),
                Arc::new(MockClient::new("b")) as Arc<dyn AIClient>,
            ),
        ];
        let ctx = RequestContext::new(base_task());
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
        })) as Arc<dyn AIClient>;
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
        ) as Arc<dyn AIClient>;

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
        ) as Arc<dyn AIClient>;

        let mut task = base_task();
        task.requires_streaming = true;
        task.security_requirements.contains_sensitive_data = true;
        let ctx_penalized = RequestContext::new(task).with_routing_hint(RoutingHint {
            preferred_provider: None,
            preferred_model: Some("special".to_string()),
            allow_remote: None,
            max_latency_ms: Some(1000),
            max_cost_tier: Some(CostTier::Medium),
            priority: None,
        });

        let s_penalized = scorer.score_provider(&p, &ctx_penalized);
        assert!(s_penalized < 90);

        let ctx_latency_hit = RequestContext::new(base_task()).with_routing_hint(RoutingHint {
            preferred_provider: None,
            preferred_model: None,
            allow_remote: None,
            max_latency_ms: Some(10),
            max_cost_tier: Some(CostTier::High),
            priority: None,
        });
        let s_latency = scorer.score_provider(&p, &ctx_latency_hit);
        assert!(s_latency < scorer.score_provider(&p, &RequestContext::new(base_task())));
    }

    #[test]
    fn test_calculate_compatibility_and_performance_scores() {
        let scorer = ProviderScorer::new();
        let p = Arc::new(MockClient::new("p")) as Arc<dyn AIClient>;

        let mut task = base_task();
        task.required_model_type = Some(ModelType::LargeLanguageModel);
        task.min_context_size = Some(100);
        task.requires_streaming = true;
        let ctx = RequestContext::new(task);

        let c = scorer.calculate_compatibility_score(&p, &ctx);
        assert!((0.0..=1.0).contains(&c));

        let mut caps = p.capabilities();
        caps.performance_metrics = PerformanceMetrics {
            avg_latency_ms: Some(100),
            avg_tokens_per_second: Some(500.0),
            success_rate: Some(0.95),
            ..Default::default()
        };
        let p2 = Arc::new(MockClient::new("p2").with_caps(caps)) as Arc<dyn AIClient>;
        let perf = scorer.calculate_performance_score(&p2);
        assert!((0.0..=1.0).contains(&perf));
    }
}
