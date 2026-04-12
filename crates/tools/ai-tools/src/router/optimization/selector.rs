// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Strategy-based provider selection engine.

use super::scorer::ProviderScorer;
use crate::AiClientImpl;
use crate::Result;
use crate::common::AIClient;
use crate::error::Error;
use crate::router::types::{RequestContext, RoutingStrategy};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::debug;

/// Provider selection engine that implements various routing strategies
pub struct ProviderSelector {
    pub(crate) round_robin_index: AtomicUsize,
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
        providers: Vec<(String, Arc<AiClientImpl>)>,
        context: &RequestContext,
        strategy: RoutingStrategy,
    ) -> Result<(String, Arc<AiClientImpl>)> {
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

    fn select_first_match(
        providers: &[(String, Arc<AiClientImpl>)],
    ) -> (String, Arc<AiClientImpl>) {
        debug!("Using FirstMatch strategy");
        let (id, client) = &providers[0];
        (id.clone(), Arc::clone(client))
    }

    fn select_highest_priority(
        providers: &[(String, Arc<AiClientImpl>)],
    ) -> (String, Arc<AiClientImpl>) {
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

    fn select_lowest_latency(
        providers: &[(String, Arc<AiClientImpl>)],
    ) -> (String, Arc<AiClientImpl>) {
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

    fn select_lowest_cost(
        providers: &[(String, Arc<AiClientImpl>)],
    ) -> (String, Arc<AiClientImpl>) {
        debug!("Using LowestCost strategy");
        let mut best_idx = 0;
        let mut best_cost = providers[0].1.routing_preferences().cost_tier;

        for (i, (_, provider)) in providers.iter().enumerate().skip(1) {
            let cost = provider.routing_preferences().cost_tier;
            if cost < best_cost {
                best_idx = i;
                best_cost = cost;
            }
        }

        let (id, client) = &providers[best_idx];
        (id.clone(), Arc::clone(client))
    }

    fn select_best_fit(
        providers: Vec<(String, Arc<AiClientImpl>)>,
        context: &RequestContext,
    ) -> (String, Arc<AiClientImpl>) {
        debug!("Using BestFit strategy");
        let scorer = ProviderScorer::new();

        let mut scored_providers: Vec<(String, Arc<AiClientImpl>, u32)> = providers
            .into_iter()
            .map(|(id, provider)| {
                let score = scorer.score_provider(&provider, context);
                (id, provider, score)
            })
            .collect();

        scored_providers.sort_by(|a, b| b.2.cmp(&a.2));

        debug!("Best fit provider score: {}", scored_providers[0].2);

        let best = scored_providers.swap_remove(0);
        (best.0, best.1)
    }

    fn select_round_robin(
        &self,
        providers: &[(String, Arc<AiClientImpl>)],
    ) -> (String, Arc<AiClientImpl>) {
        debug!("Using RoundRobin strategy");
        let index = self.round_robin_index.fetch_add(1, Ordering::SeqCst) % providers.len();
        let (id, client) = &providers[index];
        (id.clone(), Arc::clone(client))
    }

    fn select_random(providers: &[(String, Arc<AiClientImpl>)]) -> (String, Arc<AiClientImpl>) {
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
