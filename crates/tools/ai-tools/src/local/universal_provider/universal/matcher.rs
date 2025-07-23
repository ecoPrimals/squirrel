//! Universal AI Provider Capability Matcher
//!
//! Logic for finding the best capability provider for a given request.

use super::config::{CapabilityRequirement, CapabilityRequirements};
use super::types::{CapabilityProvider, CapabilityRegistry};
use crate::common::ChatRequest;

/// Capability matcher for finding the best capability for a request
#[derive(Debug)]
pub struct CapabilityMatcher {
    /// Performance weights for capability selection
    weights: CapabilityWeights,
}

#[derive(Debug, Clone)]
pub struct CapabilityWeights {
    pub latency_weight: f64,
    pub accuracy_weight: f64,
    pub cost_weight: f64,
    pub reliability_weight: f64,
    pub availability_weight: f64,
}

impl Default for CapabilityWeights {
    fn default() -> Self {
        Self {
            latency_weight: 0.3,
            accuracy_weight: 0.3,
            cost_weight: 0.2,
            reliability_weight: 0.15,
            availability_weight: 0.05,
        }
    }
}

impl CapabilityMatcher {
    pub fn new() -> Self {
        Self {
            weights: CapabilityWeights::default(),
        }
    }

    pub fn with_weights(weights: CapabilityWeights) -> Self {
        Self { weights }
    }

    /// Find the best capability provider for a request
    pub async fn find_best_capability(
        &self,
        request: &ChatRequest,
        registry: &CapabilityRegistry,
        requirements: &CapabilityRequirements,
    ) -> Option<CapabilityProvider> {
        let required_capability = self.determine_required_capability(request);
        let requirement = self.get_requirement_for_capability(&required_capability, requirements);

        let mut candidates: Vec<&CapabilityProvider> = registry
            .capabilities
            .get(&required_capability)
            .map(|providers| providers.iter().collect())
            .unwrap_or_default();

        // Filter by requirements
        candidates.retain(|provider| self.meets_requirements(provider, requirement));

        // Score remaining candidates
        let mut scored_candidates: Vec<(f64, &CapabilityProvider)> = candidates
            .into_iter()
            .map(|provider| (self.score_capability(provider, requirement), provider))
            .collect();

        // Sort by score (higher is better)
        scored_candidates
            .sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        scored_candidates
            .first()
            .map(|(_, provider)| (*provider).clone())
    }

    fn determine_required_capability(&self, request: &ChatRequest) -> String {
        // Analyze request to determine what capability is needed
        if let Some(model) = &request.model {
            if model.to_lowercase().contains("code") {
                return "code-generation".to_string();
            }
        }

        // Check message content
        for message in &request.messages {
            if let Some(content) = &message.content {
                let content_lower = content.to_lowercase();
                if content_lower.contains("code") || content_lower.contains("programming") {
                    return "code-generation".to_string();
                }
                if content_lower.contains("analyze") || content_lower.contains("analysis") {
                    return "analysis".to_string();
                }
            }
        }

        // Default to text generation
        "text-generation".to_string()
    }

    fn get_requirement_for_capability<'a>(
        &self,
        capability: &str,
        requirements: &'a CapabilityRequirements,
    ) -> &'a CapabilityRequirement {
        match capability {
            "code-generation" => &requirements.code_generation,
            "analysis" => &requirements.analysis,
            "question-answering" => &requirements.question_answering,
            _ => &requirements.text_generation,
        }
    }

    fn meets_requirements(
        &self,
        provider: &CapabilityProvider,
        requirement: &CapabilityRequirement,
    ) -> bool {
        provider.performance.average_latency_ms <= requirement.max_latency_ms as f64
            && provider.capability.quality_profile.accuracy >= requirement.min_accuracy
            && provider.performance.reliability >= requirement.min_reliability
            && provider.capability.cost_profile.cost_per_request <= requirement.max_cost_per_request
    }

    fn score_capability(
        &self,
        provider: &CapabilityProvider,
        requirement: &CapabilityRequirement,
    ) -> f64 {
        let mut score = 0.0;

        // Latency score (lower is better)
        let latency_score = 1.0
            - (provider.performance.average_latency_ms / requirement.max_latency_ms as f64)
                .min(1.0);
        score += latency_score * self.weights.latency_weight;

        // Accuracy score (higher is better)
        let accuracy_score = provider.capability.quality_profile.accuracy;
        score += accuracy_score * self.weights.accuracy_weight;

        // Cost score (lower cost is better)
        let cost_score = if provider.capability.cost_profile.is_free {
            1.0
        } else {
            1.0 - (provider.capability.cost_profile.cost_per_request
                / requirement.max_cost_per_request)
                .min(1.0)
        };
        score += cost_score * self.weights.cost_weight;

        // Reliability score
        score += provider.performance.reliability * self.weights.reliability_weight;

        // Availability score
        score += provider.performance.availability * self.weights.availability_weight;

        score
    }
}

impl Default for CapabilityMatcher {
    fn default() -> Self {
        Self::new()
    }
}
