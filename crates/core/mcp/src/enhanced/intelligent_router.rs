// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Intelligent AI Request Router
//!
//! This module provides intelligent routing of AI requests based on multiple factors:
//! - Model capabilities and performance
//! - Cost optimization
//! - Latency requirements
//! - Provider health and availability
//! - Historical performance data

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, debug, warn, error, instrument};
use chrono::{DateTime, Utc};

use crate::error::Result;
use super::coordinator::{
    UniversalAIRequest, RoutingStrategy, ProviderHealth, CostEstimate
};
use super::providers::UniversalAIProvider;

/// Performance metrics for routing decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPerformance {
    /// Provider name
    pub provider: String,
    
    /// Model name
    pub model: String,
    
    /// Average response time
    pub avg_response_time: Duration,
    
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    
    /// Average cost per request
    pub avg_cost: f64,
    
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
    
    /// Total requests processed
    pub total_requests: u64,
    
    /// Recent error rate
    pub error_rate: f64,
}

/// Routing rule for custom routing logic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    /// Rule ID
    pub id: String,
    
    /// Rule name
    pub name: String,
    
    /// Rule priority (higher = more important)
    pub priority: i32,
    
    /// Conditions for applying this rule
    pub conditions: RoutingConditions,
    
    /// Actions to take when rule matches
    pub actions: RoutingActions,
    
    /// Whether this rule is enabled
    pub enabled: bool,
}

/// Conditions for routing rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConditions {
    /// Required model capabilities
    pub requires_streaming: Option<bool>,
    pub requires_tools: Option<bool>,
    pub min_max_tokens: Option<usize>,
    
    /// Cost constraints
    pub max_cost_per_token: Option<f64>,
    pub max_total_cost: Option<f64>,
    
    /// Performance requirements
    pub max_latency_ms: Option<u64>,
    pub min_success_rate: Option<f64>,
    
    /// Provider constraints
    pub allowed_providers: Option<Vec<String>>,
    pub excluded_providers: Option<Vec<String>>,
    
    /// Request characteristics
    pub max_input_tokens: Option<usize>,
    pub request_types: Option<Vec<String>>,
}

/// Actions to take when routing rule matches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingActions {
    /// Preferred provider order
    pub provider_preference: Option<Vec<String>>,
    
    /// Fallback strategy
    pub fallback_strategy: Option<RoutingStrategy>,
    
    /// Weight adjustments
    pub cost_weight: Option<f64>,
    pub latency_weight: Option<f64>,
    pub quality_weight: Option<f64>,
    
    /// Force specific provider
    pub force_provider: Option<String>,
    
    /// Add request metadata
    pub add_metadata: Option<HashMap<String, String>>,
}

/// Intelligent router implementation
#[derive(Debug)]
pub struct IntelligentRouter {
    /// Default routing strategy
    default_strategy: RoutingStrategy,
    
    /// Performance history
    performance_history: Arc<RwLock<HashMap<String, ProviderPerformance>>>,
    
    /// Routing rules
    rules: Arc<RwLock<Vec<RoutingRule>>>,
    
    /// Fallback chain
    fallback_chain: Arc<RwLock<Vec<String>>>,
    
    /// Configuration weights
    cost_weight: f64,
    latency_weight: f64,
    quality_weight: f64,
    
    /// Circuit breaker states
    circuit_breakers: Arc<RwLock<HashMap<String, CircuitBreakerState>>>,
}

/// Circuit breaker state for providers
#[derive(Debug, Clone)]
pub struct CircuitBreakerState {
    /// Is circuit breaker open?
    pub is_open: bool,
    
    /// Number of consecutive failures
    pub failure_count: u32,
    
    /// Failure threshold
    pub failure_threshold: u32,
    
    /// Last failure time
    pub last_failure: Option<DateTime<Utc>>,
    
    /// Recovery timeout
    pub recovery_timeout: Duration,
}

impl IntelligentRouter {
    pub fn new(
        default_strategy: RoutingStrategy,
        cost_weight: f64,
        latency_weight: f64,
        quality_weight: f64,
    ) -> Self {
        Self {
            default_strategy,
            performance_history: Arc::new(RwLock::new(HashMap::new())),
            rules: Arc::new(RwLock::new(Vec::new())),
            fallback_chain: Arc::new(RwLock::new(Vec::new())),
            cost_weight,
            latency_weight,
            quality_weight,
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Route request to best available provider
    #[instrument(skip(self, providers))]
    pub async fn route_request(
        &self,
        request: &UniversalAIRequest,
        providers: &Arc<RwLock<HashMap<String, Arc<dyn UniversalAIProvider>>>>,
    ) -> Result<String> {
        debug!("Routing request {} for model {}", request.id, request.model);
        
        // Check circuit breakers first
        let available_providers = self.get_available_providers(providers).await?;
        if available_providers.is_empty() {
            return Err(crate::error::types::MCPError::ResourceExhausted(
                "No available providers".to_string()
            ));
        }
        
        // Apply routing rules
        if let Some(provider) = self.apply_routing_rules(request, &available_providers).await? {
            debug!("Rule-based routing selected provider: {}", provider);
            return Ok(provider);
        }
        
        // Use strategy-based routing
        let selected = match &self.default_strategy {
            RoutingStrategy::BestFit => self.route_best_fit(request, &available_providers).await?,
            RoutingStrategy::CostOptimized => self.route_cost_optimized(request, &available_providers).await?,
            RoutingStrategy::LatencyOptimized => self.route_latency_optimized(request, &available_providers).await?,
            RoutingStrategy::RoundRobin => self.route_round_robin(&available_providers).await?,
            RoutingStrategy::WeightedRandom => self.route_weighted_random(request, &available_providers).await?,
            RoutingStrategy::LowestLatency => self.route_latency_optimized(request, &available_providers).await?,
            RoutingStrategy::HighestQuality => self.route_best_fit(request, &available_providers).await?,
            RoutingStrategy::LocalFirst => self.route_best_fit(request, &available_providers).await?,
            RoutingStrategy::CloudFirst => self.route_best_fit(request, &available_providers).await?,
            RoutingStrategy::LowestCost => self.route_cost_optimized(request, &available_providers).await?,
            RoutingStrategy::Custom(strategy_name) => {
                warn!("Custom routing strategy '{}' not implemented, using BestFit", strategy_name);
                self.route_best_fit(request, &available_providers).await?
            }
        };
        
        debug!("Strategy-based routing selected provider: {}", selected);
        Ok(selected)
    }
    
    /// Get providers that are not circuit-broken
    async fn get_available_providers(
        &self,
        providers: &Arc<RwLock<HashMap<String, Arc<dyn UniversalAIProvider>>>>,
    ) -> Result<Vec<String>> {
        let providers_guard = providers.read().await;
        let circuit_breakers = self.circuit_breakers.read().await;
        
        let available: Vec<String> = providers_guard
            .keys()
            .filter(|&provider| {
                if let Some(breaker) = circuit_breakers.get(provider) {
                    !breaker.is_open || self.should_attempt_recovery(breaker)
                } else {
                    true
                }
            })
            .cloned()
            .collect();
            
        Ok(available)
    }
    
    /// Check if circuit breaker should attempt recovery
    fn should_attempt_recovery(&self, breaker: &CircuitBreakerState) -> bool {
        if let Some(last_failure) = breaker.last_failure {
            Utc::now() - last_failure > chrono::Duration::from_std(breaker.recovery_timeout).unwrap_or_default()
        } else {
            false
        }
    }
    
    /// Apply routing rules to determine provider
    async fn apply_routing_rules(
        &self,
        request: &UniversalAIRequest,
        available_providers: &[String],
    ) -> Result<Option<String>> {
        let rules = self.rules.read().await;
        let mut sorted_rules: Vec<_> = rules.iter().filter(|r| r.enabled).collect();
        sorted_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        for rule in sorted_rules {
            if self.rule_matches(rule, request, available_providers).await {
                if let Some(ref provider) = rule.actions.force_provider {
                    if available_providers.contains(provider) {
                        return Ok(Some(provider.clone()));
                    }
                }
                
                if let Some(ref preferences) = rule.actions.provider_preference {
                    for provider in preferences {
                        if available_providers.contains(provider) {
                            return Ok(Some(provider.clone()));
                        }
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    /// Check if routing rule matches request
    async fn rule_matches(
        &self,
        rule: &RoutingRule,
        request: &UniversalAIRequest,
        available_providers: &[String],
    ) -> bool {
        let conditions = &rule.conditions;
        
        // Check provider constraints
        if let Some(ref allowed) = conditions.allowed_providers {
            if !available_providers.iter().any(|p| allowed.contains(p)) {
                return false;
            }
        }
        
        if let Some(ref excluded) = conditions.excluded_providers {
            if available_providers.iter().any(|p| excluded.contains(p)) {
                return false;
            }
        }
        
        // Check input token constraints
        if let Some(max_tokens) = conditions.max_input_tokens {
            let estimated_tokens: usize = request.messages.iter()
                .map(|msg| msg.content.len() / 4)
                .sum();
            if estimated_tokens > max_tokens {
                return false;
            }
        }
        
        // Additional rule matching logic would go here...
        
        true
    }
    
    /// Route using best fit strategy
    async fn route_best_fit(
        &self,
        request: &UniversalAIRequest,
        available_providers: &[String],
    ) -> Result<String> {
        let mut best_score = f64::NEG_INFINITY;
        let mut best_provider = available_providers[0].clone();
        
        for provider in available_providers {
            let score = self.calculate_provider_score(provider, request).await?;
            if score > best_score {
                best_score = score;
                best_provider = provider.clone();
            }
        }
        
        Ok(best_provider)
    }
    
    /// Calculate provider score for best fit routing
    async fn calculate_provider_score(
        &self,
        provider: &str,
        request: &UniversalAIRequest,
    ) -> Result<f64> {
        let performance_history = self.performance_history.read().await;
        let key = format!("{}:{}", provider, request.model);
        
        let performance = performance_history.get(&key);
        
        let cost_score = if let Some(perf) = performance {
            1.0 / (1.0 + perf.avg_cost)
        } else {
            0.5 // Default score for unknown providers
        };
        
        let latency_score = if let Some(perf) = performance {
            1.0 / (1.0 + perf.avg_response_time.as_millis() as f64)
        } else {
            0.5
        };
        
        let quality_score = if let Some(perf) = performance {
            perf.success_rate
        } else {
            0.5
        };
        
        Ok(cost_score * self.cost_weight + 
           latency_score * self.latency_weight + 
           quality_score * self.quality_weight)
    }
    
    /// Route using cost optimization
    async fn route_cost_optimized(
        &self,
        _request: &UniversalAIRequest,
        available_providers: &[String],
    ) -> Result<String> {
        let performance_history = self.performance_history.read().await;
        
        let mut best_cost = f64::INFINITY;
        let mut best_provider = available_providers[0].clone();
        
        for provider in available_providers {
            // Find the cheapest provider with acceptable quality
            if let Some(perf) = performance_history.values()
                .find(|p| p.provider == *provider) {
                if perf.success_rate > 0.95 && perf.avg_cost < best_cost {
                    best_cost = perf.avg_cost;
                    best_provider = provider.clone();
                }
            }
        }
        
        Ok(best_provider)
    }
    
    /// Route using latency optimization
    async fn route_latency_optimized(
        &self,
        _request: &UniversalAIRequest,
        available_providers: &[String],
    ) -> Result<String> {
        let performance_history = self.performance_history.read().await;
        
        let mut best_latency = Duration::from_secs(3600);
        let mut best_provider = available_providers[0].clone();
        
        for provider in available_providers {
            if let Some(perf) = performance_history.values()
                .find(|p| p.provider == *provider) {
                if perf.success_rate > 0.95 && perf.avg_response_time < best_latency {
                    best_latency = perf.avg_response_time;
                    best_provider = provider.clone();
                }
            }
        }
        
        Ok(best_provider)
    }
    
    /// Route using round robin
    async fn route_round_robin(&self, available_providers: &[String]) -> Result<String> {
        // Simple round robin based on timestamp
        let index = (Utc::now().timestamp() as usize) % available_providers.len();
        Ok(available_providers[index].clone())
    }
    
    /// Route using weighted random selection
    async fn route_weighted_random(
        &self,
        request: &UniversalAIRequest,
        available_providers: &[String],
    ) -> Result<String> {
        let mut weights = Vec::new();
        let mut total_weight = 0.0;
        
        for provider in available_providers {
            let score = self.calculate_provider_score(provider, request).await?;
            weights.push(score);
            total_weight += score;
        }
        
        let random_value = rand::random::<f64>() * total_weight;
        let mut current_weight = 0.0;
        
        for (i, weight) in weights.iter().enumerate() {
            current_weight += weight;
            if random_value <= current_weight {
                return Ok(available_providers[i].clone());
            }
        }
        
        // Fallback to first provider
        Ok(available_providers[0].clone())
    }
    
    /// Update performance metrics after request completion
    pub async fn update_performance(
        &self,
        provider: &str,
        model: &str,
        duration: Duration,
        cost: Option<f64>,
        success: bool,
    ) -> Result<()> {
        let mut performance_history = self.performance_history.write().await;
        let key = format!("{}:{}", provider, model);
        
        let perf = performance_history.entry(key).or_insert_with(|| ProviderPerformance {
            provider: provider.to_string(),
            model: model.to_string(),
            avg_response_time: Duration::from_millis(0),
            success_rate: 1.0,
            avg_cost: 0.0,
            last_updated: Utc::now(),
            total_requests: 0,
            error_rate: 0.0,
        });
        
        // Update metrics using exponential moving average
        let alpha = 0.1; // Smoothing factor
        
        perf.avg_response_time = Duration::from_millis(
            ((1.0 - alpha) * perf.avg_response_time.as_millis() as f64 + 
             alpha * duration.as_millis() as f64) as u64
        );
        
        if let Some(c) = cost {
            perf.avg_cost = (1.0 - alpha) * perf.avg_cost + alpha * c;
        }
        
        perf.total_requests += 1;
        
        // Update success rate
        let success_value = if success { 1.0 } else { 0.0 };
        perf.success_rate = (1.0 - alpha) * perf.success_rate + alpha * success_value;
        
        // Update error rate
        let error_value = if success { 0.0 } else { 1.0 };
        perf.error_rate = (1.0 - alpha) * perf.error_rate + alpha * error_value;
        
        perf.last_updated = Utc::now();
        
        // Update circuit breaker
        self.update_circuit_breaker(provider, success).await;
        
        Ok(())
    }
    
    /// Update circuit breaker state
    async fn update_circuit_breaker(&self, provider: &str, success: bool) {
        let mut circuit_breakers = self.circuit_breakers.write().await;
        let breaker = circuit_breakers.entry(provider.to_string()).or_insert_with(|| {
            CircuitBreakerState {
                is_open: false,
                failure_count: 0,
                failure_threshold: 5,
                last_failure: None,
                recovery_timeout: Duration::from_secs(60),
            }
        });
        
        if success {
            breaker.failure_count = 0;
            if breaker.is_open {
                breaker.is_open = false;
                info!("Circuit breaker closed for provider: {}", provider);
            }
        } else {
            breaker.failure_count += 1;
            breaker.last_failure = Some(Utc::now());
            
            if breaker.failure_count >= breaker.failure_threshold && !breaker.is_open {
                breaker.is_open = true;
                warn!("Circuit breaker opened for provider: {}", provider);
            }
        }
    }
    
    /// Add routing rule
    pub async fn add_rule(&self, rule: RoutingRule) -> Result<()> {
        let mut rules = self.rules.write().await;
        rules.push(rule);
        rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        Ok(())
    }
    
    /// Remove routing rule
    pub async fn remove_rule(&self, rule_id: &str) -> Result<()> {
        let mut rules = self.rules.write().await;
        rules.retain(|r| r.id != rule_id);
        Ok(())
    }
    
    /// Get current performance statistics
    pub async fn get_performance_stats(&self) -> HashMap<String, ProviderPerformance> {
        self.performance_history.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enhanced::coordinator::AIRequestType;
    
    #[tokio::test]
    async fn test_intelligent_routing() {
        let router = IntelligentRouter::new(
            RoutingStrategy::BestFit,
            0.3, // cost weight
            0.4, // latency weight  
            0.3, // quality weight
        );
        
        let available_providers = vec!["openai".to_string(), "anthropic".to_string()];
        
        let request = crate::enhanced::coordinator::UniversalAIRequest {
            id: "test-id".to_string(),
            model: "gpt-4".to_string(),
            messages: vec![],
            request_type: crate::enhanced::coordinator::AIRequestType::TextGeneration,
            metadata: HashMap::new(),
            payload: serde_json::json!({}),
            context: crate::enhanced::coordinator::RequestContext {
                user_id: Some("test-user".to_string()),
                session_id: Some("test-session".to_string()),
                metadata: HashMap::new(),
            },
            hints: crate::enhanced::coordinator::RoutingHints {
                prefer_local: false,
                max_cost: Some(0.1),
                max_latency: Some(Duration::from_secs(30)),
                quality_requirements: vec![],
            },
            requirements: crate::enhanced::coordinator::QualityRequirements {
                min_quality_score: None,
                require_streaming: false,
                require_tools: false,
            },
        };
        
        // Should select first provider as default
        let result = router.route_best_fit(&request, &available_providers).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_performance_tracking() {
        let router = IntelligentRouter::new(
            RoutingStrategy::BestFit,
            0.3, 0.4, 0.3,
        );
        
        router.update_performance(
            "openai",
            "gpt-4",
            Duration::from_millis(1000),
            Some(0.01),
            true,
        ).await.expect("should succeed");
        
        let stats = router.get_performance_stats().await;
        assert!(stats.contains_key("openai:gpt-4"));
    }
} 