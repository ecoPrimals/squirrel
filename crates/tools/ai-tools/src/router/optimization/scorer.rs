// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Numeric scoring engine for provider fitness evaluation.

use crate::AiClientImpl;
use crate::common::AIClient;
use crate::float_helpers;
use crate::router::types::RequestContext;
use std::sync::Arc;
use tracing::debug;

/// Provider scoring engine for evaluating how well a provider matches a task
pub struct ProviderScorer;

impl ProviderScorer {
    /// Create a new provider scorer
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Score a provider based on how well it matches the task
    pub fn score_provider(&self, provider: &Arc<AiClientImpl>, context: &RequestContext) -> u32 {
        let mut score = 0;
        let capabilities = provider.capabilities();
        let preferences = provider.routing_preferences();

        score += u32::from(preferences.priority);

        if context.task.requires_streaming && capabilities.supports_streaming {
            score += 10;
        }

        if context.task.requires_function_calling && capabilities.supports_function_calling {
            score += 10;
        }

        if context.task.requires_tool_use && capabilities.supports_tool_use {
            score += 10;
        }

        if let Some(required_size) = context.task.min_context_size
            && capabilities.max_context_size >= required_size
        {
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

        if let Some(hint) = &context.routing_hint
            && let Some(ref max_cost_tier) = hint.max_cost_tier
        {
            if preferences.cost_tier > *max_cost_tier {
                score = score.saturating_sub(50);
            } else if preferences.cost_tier < *max_cost_tier {
                score += 10;
            }
        }

        if context.task.security_requirements.contains_sensitive_data
            && preferences.handles_sensitive_data
        {
            score += 20;
        }

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
    #[must_use]
    pub fn calculate_compatibility_score(
        &self,
        provider: &Arc<AiClientImpl>,
        context: &RequestContext,
    ) -> f64 {
        let capabilities = provider.capabilities();
        let mut compatibility_factors = Vec::new();

        if capabilities.supports_task(&context.task.task_type) {
            compatibility_factors.push(1.0);
        } else {
            compatibility_factors.push(0.0);
        }

        if let Some(ref model_type) = context.task.required_model_type {
            if capabilities.supports_model_type(model_type) {
                compatibility_factors.push(1.0);
            } else {
                compatibility_factors.push(0.0);
            }
        } else {
            compatibility_factors.push(1.0);
        }

        if let Some(required_size) = context.task.min_context_size {
            let req = required_size.max(1);
            let ratio = float_helpers::usize_to_f64_lossy(capabilities.max_context_size)
                / float_helpers::usize_to_f64_lossy(req);
            compatibility_factors.push(ratio.min(1.0));
        } else {
            compatibility_factors.push(1.0);
        }

        if context.task.requires_streaming {
            if capabilities.supports_streaming {
                compatibility_factors.push(1.0);
            } else {
                compatibility_factors.push(0.0);
            }
        } else {
            compatibility_factors.push(1.0);
        }

        if context.task.requires_function_calling {
            if capabilities.supports_function_calling {
                compatibility_factors.push(1.0);
            } else {
                compatibility_factors.push(0.0);
            }
        } else {
            compatibility_factors.push(1.0);
        }

        if context.task.requires_tool_use {
            if capabilities.supports_tool_use {
                compatibility_factors.push(1.0);
            } else {
                compatibility_factors.push(0.0);
            }
        } else {
            compatibility_factors.push(1.0);
        }

        let product: f64 = compatibility_factors.iter().product();
        let n = float_helpers::usize_to_f64_lossy(compatibility_factors.len().max(1));
        product.powf(1.0 / n)
    }

    /// Calculate performance score based on provider metrics
    #[must_use]
    pub fn calculate_performance_score(&self, provider: &Arc<AiClientImpl>) -> f64 {
        let capabilities = provider.capabilities();
        let mut performance_score = 0.0;

        if let Some(latency) = capabilities.performance_metrics.avg_latency_ms {
            performance_score += (100.0 / float_helpers::u64_to_f64_lossy(latency)).min(1.0);
        } else {
            performance_score += 0.5;
        }

        if let Some(throughput) = capabilities.performance_metrics.avg_tokens_per_second {
            performance_score += (throughput / 1000.0).min(1.0);
        } else {
            performance_score += 0.5;
        }

        if let Some(uptime) = capabilities.performance_metrics.success_rate {
            performance_score += uptime / 100.0;
        } else {
            performance_score += 0.9;
        }

        performance_score / 3.0
    }
}

impl Default for ProviderScorer {
    fn default() -> Self {
        Self::new()
    }
}
