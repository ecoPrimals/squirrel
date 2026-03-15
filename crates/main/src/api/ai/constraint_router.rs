// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Constraint-based provider selection
#![allow(dead_code)] // Public API surface awaiting consumer activation
//!
//! Applies routing constraints to filter and score providers.

use super::adapters::{AiProviderAdapter, QualityTier};
use super::constraints::RoutingConstraint;
use std::sync::Arc;
use tracing::debug;

/// Score for a provider given constraints
#[derive(Clone)]
pub struct ProviderScore {
    pub provider: Arc<dyn AiProviderAdapter>,
    pub score: f64,
    pub meets_requirements: bool,
}

/// Filter and score providers based on constraints
pub fn select_provider_with_constraints(
    providers: &[Arc<dyn AiProviderAdapter>],
    constraints: &[RoutingConstraint],
    task_type: &str,
) -> Option<Arc<dyn AiProviderAdapter>> {
    if providers.is_empty() {
        return None;
    }

    // If no constraints, use basic scoring
    if constraints.is_empty() {
        return select_default_provider(providers, task_type);
    }

    debug!(
        "Applying {} constraint(s) to {} provider(s)",
        constraints.len(),
        providers.len()
    );

    // Filter providers that meet hard constraints
    let mut candidates: Vec<_> = providers
        .iter()
        .filter(|p| meets_required_constraints(p, constraints))
        .cloned()
        .collect();

    if candidates.is_empty() {
        debug!("⚠️  No providers meet required constraints, falling back to all providers");
        candidates = providers.to_vec();
    }

    // Score providers based on preferences
    let mut scored: Vec<_> = candidates
        .iter()
        .map(|provider| {
            let score = calculate_provider_score(provider, constraints);
            ProviderScore {
                provider: provider.clone(),
                score,
                meets_requirements: true,
            }
        })
        .collect();

    // Sort by score descending
    // Safe comparison: NaN scores are treated as worst (equal to negative infinity)
    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Less)
    });

    // Return best provider
    scored.first().map(|s| s.provider.clone())
}

/// Check if provider meets all required constraints
fn meets_required_constraints(
    provider: &Arc<dyn AiProviderAdapter>,
    constraints: &[RoutingConstraint],
) -> bool {
    for constraint in constraints {
        match constraint {
            RoutingConstraint::RequireLocal => {
                if !provider.is_local() {
                    debug!("❌ Provider {} rejected: not local", provider.provider_id());
                    return false;
                }
            }
            RoutingConstraint::RequireProvider(name) => {
                if provider.provider_id() != name {
                    return false;
                }
            }
            RoutingConstraint::MaxCost(max_cost) => {
                if let Some(cost) = provider.cost_per_unit() {
                    if cost > *max_cost {
                        debug!(
                            "❌ Provider {} rejected: cost ${:.4} > max ${:.4}",
                            provider.provider_id(),
                            cost,
                            max_cost
                        );
                        return false;
                    }
                }
            }
            RoutingConstraint::MaxLatency(max_latency) => {
                if provider.avg_latency_ms() > *max_latency {
                    debug!(
                        "❌ Provider {} rejected: latency {}ms > max {}ms",
                        provider.provider_id(),
                        provider.avg_latency_ms(),
                        max_latency
                    );
                    return false;
                }
            }
            RoutingConstraint::MinQuality(min_quality) => {
                if provider.quality_tier() < *min_quality {
                    debug!(
                        "❌ Provider {} rejected: quality {:?} < min {:?}",
                        provider.provider_id(),
                        provider.quality_tier(),
                        min_quality
                    );
                    return false;
                }
            }
            _ => {} // Other constraints are preferences, not requirements
        }
    }
    true
}

/// Calculate score for provider based on preference constraints
fn calculate_provider_score(
    provider: &Arc<dyn AiProviderAdapter>,
    constraints: &[RoutingConstraint],
) -> f64 {
    let mut score = 50.0; // Base score

    for constraint in constraints {
        match constraint {
            RoutingConstraint::OptimizeCost => {
                // Heavily favor low-cost providers
                if provider.is_local() {
                    score += 100.0; // Local is free
                } else if let Some(cost) = provider.cost_per_unit() {
                    score += (1.0 - (cost * 100.0).min(1.0)) * 50.0;
                }
                debug!(
                    "   Cost optimization: {} → score {:.2}",
                    provider.provider_id(),
                    score
                );
            }
            RoutingConstraint::OptimizeSpeed => {
                // Favor low latency
                let latency = provider.avg_latency_ms() as f64;
                score += (5000.0 - latency.min(5000.0)) / 100.0;
                debug!(
                    "   Speed optimization: {} → score {:.2}",
                    provider.provider_id(),
                    score
                );
            }
            RoutingConstraint::OptimizeQuality => {
                // Favor high quality tiers
                score += match provider.quality_tier() {
                    QualityTier::Basic => 0.0,
                    QualityTier::Fast => 10.0, // Fast models sacrifice quality for speed
                    QualityTier::Standard => 25.0,
                    QualityTier::High => 50.0,
                    QualityTier::Premium => 75.0,
                };
                debug!(
                    "   Quality optimization: {} → score {:.2}",
                    provider.provider_id(),
                    score
                );
            }
            RoutingConstraint::PreferLocal => {
                if provider.is_local() {
                    score += 30.0;
                    debug!(
                        "   Local preference: {} → score {:.2}",
                        provider.provider_id(),
                        score
                    );
                }
            }
            RoutingConstraint::PreferQuality(tier) => {
                if provider.quality_tier() >= *tier {
                    score += 20.0;
                }
            }
            _ => {} // Requirements already handled
        }
    }

    score
}

/// Select provider without constraints (default behavior)
fn select_default_provider(
    providers: &[Arc<dyn AiProviderAdapter>],
    task_type: &str,
) -> Option<Arc<dyn AiProviderAdapter>> {
    // Simple heuristic: prefer capable providers
    for provider in providers {
        match task_type {
            "text" if provider.supports_text_generation() => {
                return Some(provider.clone());
            }
            "image" if provider.supports_image_generation() => {
                return Some(provider.clone());
            }
            _ => {}
        }
    }

    // Fallback to first provider
    providers.first().cloned()
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_constraint_filtering() {
        // Tests would go here
    }
}
