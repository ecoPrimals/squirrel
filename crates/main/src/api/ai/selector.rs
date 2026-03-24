// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Provider selection logic
//!
//! Intelligent provider selection based on requirements, scoring by cost,
//! quality, latency, and reliability.

use super::types::ActionRequirements;
use tracing::{debug, info};

/// Provider information for selection
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub provider_id: String,
    pub provider_name: String,
    pub capabilities: Vec<String>,
    pub quality_tier: QualityTier,
    pub cost_per_unit: Option<f64>,
    pub avg_latency_ms: u64,
    pub reliability: f64, // 0.0 - 1.0
    pub is_local: bool,
    pub is_available: bool,
}

/// Quality tier classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityTier {
    Low,
    Medium,
    High,
    Premium,
}

impl QualityTier {
    const fn score(self) -> f64 {
        match self {
            Self::Low => 1.0,
            Self::Medium => 2.0,
            Self::High => 3.0,
            Self::Premium => 4.0,
        }
    }

    fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "low" => Self::Low,
            "high" => Self::High,
            "premium" => Self::Premium,
            _ => Self::Medium, // Default
        }
    }
}

/// Provider selector for intelligent provider choice
pub struct ProviderSelector {
    /// Fallback to any available provider if preferred fails
    enable_fallback: bool,
}

impl ProviderSelector {
    /// Create a new provider selector
    pub const fn new() -> Self {
        Self {
            enable_fallback: true,
        }
    }

    /// Select the best provider based on requirements
    pub fn select_best(
        &self,
        providers: &[ProviderInfo],
        requirements: Option<&ActionRequirements>,
    ) -> Result<ProviderInfo, SelectionError> {
        if providers.is_empty() {
            return Err(SelectionError::NoProvidersAvailable);
        }

        // Filter to available providers only
        let available: Vec<_> = providers.iter().filter(|p| p.is_available).collect();

        if available.is_empty() {
            return Err(SelectionError::AllProvidersUnavailable);
        }

        // Check for preferred provider override
        if let Some(reqs) = requirements
            && let Some(ref preferred) = reqs.preferred_provider
        {
            if let Some(provider) = available.iter().find(|p| &p.provider_id == preferred) {
                info!("✅ Using preferred provider: {}", preferred);
                return Ok((*provider).clone());
            } else if !self.enable_fallback {
                return Err(SelectionError::PreferredProviderNotAvailable(
                    preferred.clone(),
                ));
            }
            // Fall through to scoring if fallback enabled
        }

        // Score all available providers
        let mut scored: Vec<_> = available
            .iter()
            .map(|provider| {
                let score = self.score_provider(provider, requirements);
                debug!(
                    "Provider {} scored: {:.2} (quality: {:?}, cost: {:?}, latency: {}ms, reliability: {:.2})",
                    provider.provider_name,
                    score,
                    provider.quality_tier,
                    provider.cost_per_unit,
                    provider.avg_latency_ms,
                    provider.reliability
                );
                (provider, score)
            })
            .collect();

        // Sort by score (descending)
        scored.sort_by(|a, b| b.1.total_cmp(&a.1));

        let best = scored.first().ok_or(SelectionError::NoSuitableProvider)?;

        info!(
            "🎯 Selected best provider: {} (score: {:.2})",
            best.0.provider_name, best.1
        );

        Ok((*best.0).clone())
    }

    /// Score a provider based on requirements
    fn score_provider(
        &self,
        provider: &ProviderInfo,
        requirements: Option<&ActionRequirements>,
    ) -> f64 {
        let reqs = requirements.cloned().unwrap_or_default();
        let mut score = 0.0;

        // Quality scoring (weight: 30%)
        if let Some(ref quality) = reqs.quality {
            let desired_quality = QualityTier::from_string(quality);
            let quality_match = provider.quality_tier.score() / desired_quality.score();
            score += quality_match.clamp(0.0, 2.0) * 30.0;
        } else {
            // No preference, score by absolute quality
            score += provider.quality_tier.score() * 7.5;
        }

        // Cost scoring (weight: 25%, but 40% when optimizing)
        if let Some(ref cost_pref) = reqs.cost_preference {
            match cost_pref.as_str() {
                "optimize" => {
                    // Prefer low cost - higher weight for cost optimization
                    let cost_score = if let Some(cost) = provider.cost_per_unit {
                        if cost < 0.001 {
                            40.0 // Strongly prefer very cheap
                        } else if cost < 0.01 {
                            25.0
                        } else if cost < 0.05 {
                            15.0
                        } else {
                            5.0
                        }
                    } else {
                        35.0 // No cost = free/local (very good for optimization)
                    };
                    score += cost_score;
                }
                "premium" => {
                    // Don't penalize cost
                    score += 15.0;
                }
                _ => {
                    // Balanced
                    let cost_score = if let Some(cost) = provider.cost_per_unit {
                        if cost < 0.01 {
                            20.0
                        } else if cost < 0.05 {
                            15.0
                        } else {
                            10.0
                        }
                    } else {
                        20.0
                    };
                    score += cost_score;
                }
            }
        } else {
            // Default: prefer reasonable cost
            score += 15.0;
        }

        // Latency scoring (weight: 20%)
        if let Some(max_latency) = reqs.max_latency_ms {
            if provider.avg_latency_ms <= max_latency {
                score += 20.0;
            } else {
                // Penalize for exceeding max latency
                let penalty =
                    ((provider.avg_latency_ms - max_latency) as f64 / max_latency as f64).min(1.0);
                score += 20.0 * (1.0 - penalty);
            }
        } else {
            // Prefer lower latency
            let latency_score = if provider.avg_latency_ms < 1000 {
                20.0
            } else if provider.avg_latency_ms < 5000 {
                15.0
            } else if provider.avg_latency_ms < 15000 {
                10.0
            } else {
                5.0
            };
            score += latency_score;
        }

        // Reliability scoring (weight: 20%)
        score += provider.reliability * 20.0;

        // Privacy scoring (weight: 5%)
        if let Some(ref privacy) = reqs.privacy_level {
            match privacy.as_str() {
                "local" | "private" => {
                    if provider.is_local {
                        score += 5.0;
                    }
                }
                _ => {
                    score += 2.5;
                }
            }
        } else {
            score += 2.5;
        }

        score
    }
}

impl Default for ProviderSelector {
    fn default() -> Self {
        Self::new()
    }
}

/// Provider selection errors
#[derive(Debug, thiserror::Error)]
pub enum SelectionError {
    #[error("No providers available for this action")]
    NoProvidersAvailable,

    #[error("All providers are currently unavailable")]
    AllProvidersUnavailable,

    #[error("Preferred provider '{0}' is not available")]
    PreferredProviderNotAvailable(String),

    #[error("No suitable provider found matching requirements")]
    NoSuitableProvider,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_provider(
        id: &str,
        quality: QualityTier,
        cost: Option<f64>,
        latency: u64,
    ) -> ProviderInfo {
        ProviderInfo {
            provider_id: id.to_string(),
            provider_name: id.to_string(),
            capabilities: vec!["image.generation".to_string()],
            quality_tier: quality,
            cost_per_unit: cost,
            avg_latency_ms: latency,
            reliability: 0.95,
            is_local: cost.is_none(),
            is_available: true,
        }
    }

    #[test]
    fn test_quality_preference() {
        let selector = ProviderSelector::new();
        let providers = vec![
            mock_provider("low", QualityTier::Low, Some(0.0001), 1000),
            mock_provider("high", QualityTier::High, Some(0.02), 2000),
        ];

        let reqs = ActionRequirements {
            quality: Some("high".to_string()),
            ..Default::default()
        };

        let best = selector
            .select_best(&providers, Some(&reqs))
            .expect("should succeed");
        assert_eq!(best.provider_id, "high");
    }

    #[test]
    fn test_cost_optimization() {
        let selector = ProviderSelector::new();
        let providers = vec![
            mock_provider("cheap", QualityTier::Medium, Some(0.001), 2000),
            mock_provider("expensive", QualityTier::High, Some(0.05), 1000),
        ];

        let reqs = ActionRequirements {
            cost_preference: Some("optimize".to_string()),
            ..Default::default()
        };

        let best = selector
            .select_best(&providers, Some(&reqs))
            .expect("should succeed");
        assert_eq!(best.provider_id, "cheap");
    }

    #[test]
    fn test_latency_preference() {
        let selector = ProviderSelector::new();
        let providers = vec![
            mock_provider("slow", QualityTier::High, Some(0.01), 10000),
            mock_provider("fast", QualityTier::Medium, Some(0.01), 500),
        ];

        let reqs = ActionRequirements {
            max_latency_ms: Some(2000),
            ..Default::default()
        };

        let best = selector
            .select_best(&providers, Some(&reqs))
            .expect("should succeed");
        assert_eq!(best.provider_id, "fast");
    }
}
