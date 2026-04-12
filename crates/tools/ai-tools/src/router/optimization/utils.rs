// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Pre/post-selection list transforms for provider filtering and sorting.

use crate::AiClientImpl;
use crate::common::AIClient;
use crate::router::types::RequestContext;
use std::sync::Arc;

/// Optimization utilities for provider selection
pub struct OptimizationUtils;

impl OptimizationUtils {
    /// Filter providers based on routing hints
    #[must_use]
    pub fn filter_by_routing_hint(
        providers: Vec<(String, Arc<AiClientImpl>)>,
        context: &RequestContext,
    ) -> Vec<(String, Arc<AiClientImpl>)> {
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
        mut providers: Vec<(String, Arc<AiClientImpl>)>,
    ) -> Vec<(String, Arc<AiClientImpl>)> {
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
        mut providers: Vec<(String, Arc<AiClientImpl>)>,
    ) -> Vec<(String, Arc<AiClientImpl>)> {
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
        mut providers: Vec<(String, Arc<AiClientImpl>)>,
    ) -> Vec<(String, Arc<AiClientImpl>)> {
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
