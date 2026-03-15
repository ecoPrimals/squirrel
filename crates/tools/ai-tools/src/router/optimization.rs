// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Provider selection and optimization algorithms for AI routing.
//!
//! This module implements various strategies for selecting the best AI provider
//! for a given task, including scoring algorithms and routing optimizations.

use super::types::{RequestContext, RoutingStrategy};
use crate::common::AIClient;
use crate::error::Error;
use crate::Result;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tracing::debug;

/// Provider selection engine that implements various routing strategies
pub struct ProviderSelector {
    /// Round-robin state for providers
    round_robin_index: AtomicUsize,
}

impl ProviderSelector {
    /// Create a new provider selector
    pub fn new() -> Self {
        Self {
            round_robin_index: AtomicUsize::new(0),
        }
    }

    /// Select a provider based on the routing strategy
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
            return Ok(providers[0].clone());
        }

        match strategy {
            RoutingStrategy::FirstMatch => self.select_first_match(providers),
            RoutingStrategy::HighestPriority => self.select_highest_priority(providers),
            RoutingStrategy::LowestLatency => self.select_lowest_latency(providers),
            RoutingStrategy::LowestCost => self.select_lowest_cost(providers),
            RoutingStrategy::BestFit => self.select_best_fit(providers, context),
            RoutingStrategy::RoundRobin => self.select_round_robin(providers),
            RoutingStrategy::Random => self.select_random(providers),
        }
    }

    /// Select the first provider in the list
    fn select_first_match(
        &self,
        providers: Vec<(String, Arc<dyn AIClient>)>,
    ) -> Result<(String, Arc<dyn AIClient>)> {
        debug!("Using FirstMatch strategy");
        Ok(providers[0].clone())
    }

    /// Select the provider with the highest priority
    fn select_highest_priority(
        &self,
        providers: Vec<(String, Arc<dyn AIClient>)>,
    ) -> Result<(String, Arc<dyn AIClient>)> {
        debug!("Using HighestPriority strategy");
        let mut best_provider = providers[0].clone();
        let mut best_priority = best_provider.1.routing_preferences().priority;

        for (id, provider) in providers.iter().skip(1) {
            let priority = provider.routing_preferences().priority;
            if priority > best_priority {
                best_provider = (id.clone(), provider.clone());
                best_priority = priority;
            }
        }

        Ok(best_provider)
    }

    /// Select the provider with the lowest latency
    fn select_lowest_latency(
        &self,
        providers: Vec<(String, Arc<dyn AIClient>)>,
    ) -> Result<(String, Arc<dyn AIClient>)> {
        debug!("Using LowestLatency strategy");
        let mut best_provider = providers[0].clone();
        let mut best_latency = best_provider
            .1
            .capabilities()
            .performance_metrics
            .avg_latency_ms
            .unwrap_or(u64::MAX);

        for (id, provider) in providers.iter().skip(1) {
            if let Some(latency) = provider.capabilities().performance_metrics.avg_latency_ms {
                if latency < best_latency {
                    best_provider = (id.clone(), provider.clone());
                    best_latency = latency;
                }
            }
        }

        Ok(best_provider)
    }

    /// Select the provider with the lowest cost tier
    fn select_lowest_cost(
        &self,
        providers: Vec<(String, Arc<dyn AIClient>)>,
    ) -> Result<(String, Arc<dyn AIClient>)> {
        debug!("Using LowestCost strategy");
        let mut best_provider = providers[0].clone();
        let mut best_cost = best_provider.1.routing_preferences().cost_tier;

        for (id, provider) in providers.iter().skip(1) {
            let cost = provider.routing_preferences().cost_tier;
            // Lower cost tier is better (Free < Low < Medium < High)
            if cost < best_cost {
                best_provider = (id.clone(), provider.clone());
                best_cost = cost;
            }
        }

        Ok(best_provider)
    }

    /// Select the provider that best matches the task requirements
    fn select_best_fit(
        &self,
        providers: Vec<(String, Arc<dyn AIClient>)>,
        context: &RequestContext,
    ) -> Result<(String, Arc<dyn AIClient>)> {
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

        // Return the highest scoring provider
        Ok((scored_providers[0].0.clone(), scored_providers[0].1.clone()))
    }

    /// Select a provider using round-robin
    fn select_round_robin(
        &self,
        providers: Vec<(String, Arc<dyn AIClient>)>,
    ) -> Result<(String, Arc<dyn AIClient>)> {
        debug!("Using RoundRobin strategy");
        let index = self.round_robin_index.fetch_add(1, Ordering::SeqCst) % providers.len();
        Ok(providers[index].clone())
    }

    /// Select a provider randomly
    fn select_random(
        &self,
        providers: Vec<(String, Arc<dyn AIClient>)>,
    ) -> Result<(String, Arc<dyn AIClient>)> {
        debug!("Using Random strategy");
        let index = rand::random::<usize>() % providers.len();
        Ok(providers[index].clone())
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
    pub fn new() -> Self {
        Self
    }

    /// Score a provider based on how well it matches the task
    pub fn score_provider(&self, provider: &Arc<dyn AIClient>, context: &RequestContext) -> u32 {
        let mut score = 0;
        let capabilities = provider.capabilities();
        let preferences = provider.routing_preferences();

        // Base score from priority
        score += preferences.priority as u32;

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
        if let Some(required_size) = context.task.min_context_size {
            if capabilities.max_context_size >= required_size {
                // Higher bonus for models with just enough context (to avoid over-provisioning)
                let size_ratio = (capabilities.max_context_size as f32) / (required_size as f32);
                if size_ratio <= 1.5 {
                    score += 15;
                } else if size_ratio <= 2.0 {
                    score += 10;
                } else {
                    score += 5;
                }
            }
        }

        // Handle cost preferences if specified in routing hint
        if let Some(hint) = &context.routing_hint {
            if let Some(ref max_cost_tier) = hint.max_cost_tier {
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
        }

        // Bonus for handling sensitive data if required
        if context.task.security_requirements.contains_sensitive_data
            && preferences.handles_sensitive_data
        {
            score += 20;
        }

        // Apply routing hint preferences if present
        if let Some(hint) = &context.routing_hint {
            if let Some(preferred_model) = &hint.preferred_model {
                if provider.default_model() == preferred_model {
                    score += 25;
                }
            }

            if let Some(max_latency) = hint.max_latency_ms {
                if let Some(latency) = capabilities.performance_metrics.avg_latency_ms {
                    if latency <= max_latency {
                        score += 15;
                    } else {
                        score = score.saturating_sub(30);
                    }
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
            let ratio = (capabilities.max_context_size as f64) / (required_size as f64);
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
        product.powf(1.0 / compatibility_factors.len() as f64)
    }

    /// Calculate performance score based on provider metrics
    pub fn calculate_performance_score(&self, provider: &Arc<dyn AIClient>) -> f64 {
        let capabilities = provider.capabilities();
        let mut performance_score = 0.0;

        // Latency score (lower is better)
        if let Some(latency) = capabilities.performance_metrics.avg_latency_ms {
            // Convert to score where 100ms = 1.0, 1000ms = 0.1, etc.
            performance_score += (100.0 / latency as f64).min(1.0);
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
    pub fn filter_by_routing_hint(
        providers: Vec<(String, Arc<dyn AIClient>)>,
        context: &RequestContext,
    ) -> Vec<(String, Arc<dyn AIClient>)> {
        if let Some(hint) = &context.routing_hint {
            if let Some(preferred_provider) = &hint.preferred_provider {
                return providers
                    .into_iter()
                    .filter(|(id, _)| id == preferred_provider)
                    .collect();
            }
        }
        providers
    }

    /// Sort providers by priority
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
    use crate::common::capability::{AITask, SecurityRequirements, TaskType};
    use crate::router::types::RoutingHint;

    #[test]
    fn test_provider_selector_creation() {
        let selector = ProviderSelector::new();
        assert_eq!(selector.round_robin_index.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_provider_scorer_creation() {
        let scorer = ProviderScorer::new();
        // Just ensure it can be created
        assert!(std::ptr::eq(&scorer, &scorer));
    }

    #[test]
    fn test_empty_providers_error() {
        let selector = ProviderSelector::new();
        let context = RequestContext::new(AITask {
            task_type: TaskType::TextGeneration,
            required_model_type: None,
            min_context_size: None,
            requires_streaming: false,
            requires_function_calling: false,
            requires_tool_use: false,
            security_requirements: SecurityRequirements::default(),
            complexity_score: None,
            priority: 50,
        });

        let result = selector.select_provider(vec![], &context, RoutingStrategy::BestFit);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No suitable providers"));
    }

    #[test]
    fn test_filter_by_routing_hint() {
        let providers: Vec<(String, Arc<dyn AIClient>)> = vec![
            (
                "provider1".to_string(),
                Arc::new(MockProvider::new("provider1")) as Arc<dyn AIClient>,
            ),
            (
                "provider2".to_string(),
                Arc::new(MockProvider::new("provider2")) as Arc<dyn AIClient>,
            ),
        ];

        let context = RequestContext::new(AITask {
            task_type: TaskType::TextGeneration,
            required_model_type: None,
            min_context_size: None,
            requires_streaming: false,
            requires_function_calling: false,
            requires_tool_use: false,
            security_requirements: SecurityRequirements::default(),
            complexity_score: None,
            priority: 50,
        })
        .with_routing_hint(RoutingHint {
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

    // Mock provider for testing
    #[derive(Debug)]
    struct MockProvider {
        name: String,
    }

    impl MockProvider {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    #[async_trait::async_trait]
    impl crate::common::AIClient for MockProvider {
        async fn get_capabilities(
            &self,
            _model: &str,
        ) -> crate::error::Result<crate::common::capability::AICapabilities> {
            Ok(crate::common::capability::AICapabilities {
                supported_task_types: std::collections::HashSet::new(),
                supported_model_types: std::collections::HashSet::new(),
                max_context_size: 4096,
                supports_streaming: false,
                supports_function_calling: false,
                supports_tool_use: false,
                supports_images: false,
                performance_metrics: Default::default(),
                cost_metrics: Default::default(),
                resource_requirements: Default::default(),
                routing_preferences: Default::default(),
                security_requirements: Default::default(),
            })
        }

        fn provider_name(&self) -> &str {
            &self.name
        }

        async fn is_available(&self) -> bool {
            true
        }

        fn default_model(&self) -> &str {
            "mock-model"
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
                model: format!("{}-model", self.name),
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
}
