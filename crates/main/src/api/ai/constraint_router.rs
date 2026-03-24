// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Constraint-based provider selection
#![expect(dead_code, reason = "API surface awaiting consumer activation")]
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
                if let Some(cost) = provider.cost_per_unit()
                    && cost > *max_cost
                {
                    debug!(
                        "❌ Provider {} rejected: cost ${:.4} > max ${:.4}",
                        provider.provider_id(),
                        cost,
                        max_cost
                    );
                    return false;
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
    use super::*;
    use crate::api::ai::adapters::{AiProviderAdapter, QualityTier};
    use crate::api::ai::constraints::RoutingConstraint;
    use crate::api::ai::types::{
        ImageGenerationRequest, ImageGenerationResponse, TextGenerationRequest,
        TextGenerationResponse,
    };
    use crate::error::PrimalError;
    use async_trait::async_trait;
    use std::sync::Arc;

    #[derive(Clone)]
    struct MockAdapter {
        id: &'static str,
        is_local: bool,
        cost: Option<f64>,
        latency: u64,
        quality: QualityTier,
        text: bool,
        image: bool,
    }

    #[async_trait]
    impl AiProviderAdapter for MockAdapter {
        fn provider_id(&self) -> &str {
            self.id
        }

        fn provider_name(&self) -> &str {
            self.id
        }

        fn is_local(&self) -> bool {
            self.is_local
        }

        fn cost_per_unit(&self) -> Option<f64> {
            self.cost
        }

        fn avg_latency_ms(&self) -> u64 {
            self.latency
        }

        fn quality_tier(&self) -> QualityTier {
            self.quality
        }

        fn supports_text_generation(&self) -> bool {
            self.text
        }

        fn supports_image_generation(&self) -> bool {
            self.image
        }

        async fn generate_text(
            &self,
            _request: TextGenerationRequest,
        ) -> Result<TextGenerationResponse, PrimalError> {
            unreachable!("tests do not call generate")
        }

        async fn generate_image(
            &self,
            _request: ImageGenerationRequest,
        ) -> Result<ImageGenerationResponse, PrimalError> {
            unreachable!("tests do not call generate")
        }
    }

    fn arc(m: MockAdapter) -> Arc<dyn AiProviderAdapter> {
        Arc::new(m)
    }

    #[test]
    fn empty_providers_returns_none() {
        let providers: Vec<Arc<dyn AiProviderAdapter>> = vec![];
        assert!(select_provider_with_constraints(&providers, &[], "text").is_none());
    }

    #[test]
    fn no_constraints_prefers_text_capable_then_fallback_first() {
        let a = arc(MockAdapter {
            id: "a",
            is_local: false,
            cost: None,
            latency: 100,
            quality: QualityTier::Basic,
            text: false,
            image: false,
        });
        let b = arc(MockAdapter {
            id: "b",
            is_local: false,
            cost: None,
            latency: 10,
            quality: QualityTier::Standard,
            text: true,
            image: false,
        });
        let chosen = select_provider_with_constraints(&[a.clone(), b.clone()], &[], "text")
            .map(|p| p.provider_id().to_string());
        assert_eq!(chosen.as_deref(), Some("b"));

        let img = arc(MockAdapter {
            id: "img",
            is_local: false,
            cost: None,
            latency: 1,
            quality: QualityTier::Basic,
            text: false,
            image: true,
        });
        let chosen_img = select_provider_with_constraints(&[a.clone(), img.clone()], &[], "image")
            .map(|p| p.provider_id().to_string());
        assert_eq!(chosen_img.as_deref(), Some("img"));

        let fallback = select_provider_with_constraints(&[a], &[], "other")
            .map(|p| p.provider_id().to_string());
        assert_eq!(fallback.as_deref(), Some("a"));
    }

    #[test]
    fn require_local_filters_and_fallback_when_none_match() {
        let remote = arc(MockAdapter {
            id: "r",
            is_local: false,
            cost: Some(0.01),
            latency: 5,
            quality: QualityTier::Premium,
            text: true,
            image: false,
        });
        let local = arc(MockAdapter {
            id: "l",
            is_local: true,
            cost: None,
            latency: 50,
            quality: QualityTier::Basic,
            text: true,
            image: false,
        });
        let c = [RoutingConstraint::RequireLocal];
        let picked = select_provider_with_constraints(&[remote.clone(), local.clone()], &c, "text")
            .map(|p| p.provider_id().to_string());
        assert_eq!(picked.as_deref(), Some("l"));

        let picked2 = select_provider_with_constraints(std::slice::from_ref(&remote), &c, "text")
            .map(|p| p.provider_id().to_string());
        assert_eq!(picked2.as_deref(), Some("r"));
    }

    #[test]
    fn hard_limits_max_cost_latency_min_quality() {
        let cheap = arc(MockAdapter {
            id: "cheap",
            is_local: false,
            cost: Some(0.001),
            latency: 200,
            quality: QualityTier::Basic,
            text: true,
            image: false,
        });
        let pricey = arc(MockAdapter {
            id: "pricey",
            is_local: false,
            cost: Some(1.0),
            latency: 10,
            quality: QualityTier::Premium,
            text: true,
            image: false,
        });
        let cost_ok = select_provider_with_constraints(
            &[cheap.clone(), pricey.clone()],
            &[RoutingConstraint::MaxCost(0.01)],
            "text",
        )
        .map(|p| p.provider_id().to_string());
        assert_eq!(cost_ok.as_deref(), Some("cheap"));

        let fast = arc(MockAdapter {
            id: "fast",
            is_local: false,
            cost: None,
            latency: 5,
            quality: QualityTier::Basic,
            text: true,
            image: false,
        });
        let slow = arc(MockAdapter {
            id: "slow",
            is_local: false,
            cost: None,
            latency: 500,
            quality: QualityTier::Basic,
            text: true,
            image: false,
        });
        let lat = select_provider_with_constraints(
            &[slow.clone(), fast.clone()],
            &[RoutingConstraint::MaxLatency(100)],
            "text",
        )
        .map(|p| p.provider_id().to_string());
        assert_eq!(lat.as_deref(), Some("fast"));

        let low_q = arc(MockAdapter {
            id: "low",
            is_local: false,
            cost: None,
            latency: 1,
            quality: QualityTier::Basic,
            text: true,
            image: false,
        });
        let high_q = arc(MockAdapter {
            id: "high",
            is_local: false,
            cost: None,
            latency: 2,
            quality: QualityTier::High,
            text: true,
            image: false,
        });
        let q = select_provider_with_constraints(
            &[low_q, high_q.clone()],
            &[RoutingConstraint::MinQuality(QualityTier::Standard)],
            "text",
        )
        .map(|p| p.provider_id().to_string());
        assert_eq!(q.as_deref(), Some("high"));
    }

    #[test]
    fn require_provider_name() {
        let a = arc(MockAdapter {
            id: "a",
            is_local: true,
            cost: None,
            latency: 1,
            quality: QualityTier::Premium,
            text: true,
            image: false,
        });
        let b = arc(MockAdapter {
            id: "b",
            is_local: true,
            cost: None,
            latency: 2,
            quality: QualityTier::Basic,
            text: true,
            image: false,
        });
        let p = select_provider_with_constraints(
            &[a, b.clone()],
            &[RoutingConstraint::RequireProvider("b".to_string())],
            "text",
        )
        .map(|x| x.provider_id().to_string());
        assert_eq!(p.as_deref(), Some("b"));
    }

    #[test]
    fn preference_constraints_change_ordering() {
        let local = arc(MockAdapter {
            id: "loc",
            is_local: true,
            cost: None,
            latency: 100,
            quality: QualityTier::Standard,
            text: true,
            image: false,
        });
        let remote = arc(MockAdapter {
            id: "rem",
            is_local: false,
            cost: Some(0.5),
            latency: 10,
            quality: QualityTier::Standard,
            text: true,
            image: false,
        });
        let prefer_local = select_provider_with_constraints(
            &[remote.clone(), local.clone()],
            &[RoutingConstraint::PreferLocal],
            "text",
        )
        .map(|p| p.provider_id().to_string());
        assert_eq!(prefer_local.as_deref(), Some("loc"));

        let opt_cost = select_provider_with_constraints(
            &[remote.clone(), local.clone()],
            &[RoutingConstraint::OptimizeCost],
            "text",
        )
        .map(|p| p.provider_id().to_string());
        assert_eq!(opt_cost.as_deref(), Some("loc"));

        let opt_speed = select_provider_with_constraints(
            &[local.clone(), remote.clone()],
            &[RoutingConstraint::OptimizeSpeed],
            "text",
        )
        .map(|p| p.provider_id().to_string());
        assert_eq!(opt_speed.as_deref(), Some("rem"));
    }

    #[test]
    fn routing_constraint_serde_roundtrip() {
        let c = RoutingConstraint::RequireProvider("p".to_string());
        let json = serde_json::to_string(&c).expect("should succeed");
        let back: RoutingConstraint = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(back, c);
    }
}
