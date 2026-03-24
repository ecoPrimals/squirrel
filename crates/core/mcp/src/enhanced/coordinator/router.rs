// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Universal AI Router
//!
//! This module contains the AIRouter implementation for intelligent routing
//! to any AI system based on capabilities, cost, latency, and quality requirements.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::RwLock;

use crate::error::types::Result;
use crate::enhanced::providers::UniversalAIProvider;
use super::types::{
    RoutingStrategy, RoutingRule, RoutingConfig, PerformanceMetrics, 
    UniversalAIRequest, RuleCondition, RuleAction
};

/// Universal AI Router - intelligently routes to ANY AI system
#[derive(Debug)]
pub struct AIRouter {
    /// Routing strategy
    strategy: RoutingStrategy,
    
    /// Model performance history
    performance_history: Arc<RwLock<HashMap<String, PerformanceMetrics>>>,
    
    /// Routing rules
    rules: Arc<RwLock<Vec<RoutingRule>>>,
    
    /// Fallback chain
    fallback_chain: Arc<RwLock<Vec<String>>>,
    
    /// Round robin counter
    round_robin_counter: Arc<AtomicUsize>,
}

impl AIRouter {
    /// Create a new AI router
    pub fn new(config: RoutingConfig) -> Self {
        Self {
            strategy: config.default_strategy,
            performance_history: Arc::new(RwLock::new(HashMap::new())),
            rules: Arc::new(RwLock::new(Vec::new())),
            fallback_chain: Arc::new(RwLock::new(Vec::new())),
            round_robin_counter: Arc::new(AtomicUsize::new(0)),
        }
    }
    
    /// Route a request to the best available provider
    pub async fn route_request(
        &self, 
        request: &UniversalAIRequest, 
        providers: &Arc<RwLock<HashMap<String, Arc<dyn UniversalAIProvider>>>>
    ) -> Result<String> {
        // Apply routing rules first
        if let Some(provider) = self.apply_routing_rules(request, providers).await? {
            return Ok(provider);
        }
        
        // Fall back to strategy-based routing
        self.apply_routing_strategy(request, providers).await
    }
    
    /// Apply routing rules to determine provider
    async fn apply_routing_rules(
        &self,
        request: &UniversalAIRequest,
        providers: &Arc<RwLock<HashMap<String, Arc<dyn UniversalAIProvider>>>>
    ) -> Result<Option<String>> {
        let rules = self.rules.read().await;
        let mut sorted_rules = rules.clone();
        
        // Sort rules by priority (highest first)
        sorted_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        for rule in sorted_rules {
            if self.matches_condition(&rule.condition, request).await? {
                if let Some(provider) = self.apply_rule_action(&rule.action, providers).await? {
                    return Ok(Some(provider));
                }
            }
        }
        
        Ok(None)
    }
    
    /// Check if a request matches a rule condition
    async fn matches_condition(
        &self,
        condition: &RuleCondition,
        request: &UniversalAIRequest
    ) -> Result<bool> {
        match condition {
            RuleCondition::SensitiveData => {
                // Check if request contains sensitive data
                Ok(request.hints.prefer_local || 
                   request.context.metadata.contains_key("sensitive"))
            }
            RuleCondition::HighQuality => {
                // Check if request requires high quality
                Ok(request.requirements.min_quality_score.unwrap_or(0.0) > 0.8)
            }
            RuleCondition::LowLatency => {
                // Check if request requires low latency
                Ok(request.hints.max_latency.is_some())
            }
            RuleCondition::LowCost => {
                // Check if request requires low cost
                Ok(request.hints.max_cost.is_some())
            }
            RuleCondition::ModelType(model_type) => {
                // Check if request requires specific model type
                Ok(request.model.contains(model_type))
            }
            RuleCondition::TaskType(task_type) => {
                // Check if request is for specific task type
                Ok(request.request_type.to_string().contains(task_type))
            }
            RuleCondition::Custom(_) => {
                // Custom condition logic would go here
                Ok(false)
            }
        }
    }
    
    /// Apply a rule action to determine provider
    async fn apply_rule_action(
        &self,
        action: &RuleAction,
        providers: &Arc<RwLock<HashMap<String, Arc<dyn UniversalAIProvider>>>>
    ) -> Result<Option<String>> {
        match action {
            RuleAction::PreferProvider(provider_name) => {
                let providers_guard = providers.read().await;
                if providers_guard.contains_key(provider_name) {
                    Ok(Some(provider_name.clone()))
                } else {
                    Ok(None)
                }
            }
            RuleAction::RequireLocal => {
                // Find first local provider
                let providers_guard = providers.read().await;
                for (name, provider) in providers_guard.iter() {
                    let provider_type = provider.provider_type();
                    if matches!(provider_type, crate::enhanced::providers::ProviderType::LocalServer | 
                                              crate::enhanced::providers::ProviderType::LocalNative) {
                        return Ok(Some(name.clone()));
                    }
                }
                Ok(None)
            }
            RuleAction::AllowCloud => {
                // This doesn't select a specific provider, just allows cloud
                Ok(None)
            }
            RuleAction::MaxCost(_) => {
                // Cost filtering would be applied in strategy-based routing
                Ok(None)
            }
            RuleAction::MaxLatency(_) => {
                // Latency filtering would be applied in strategy-based routing
                Ok(None)
            }
            RuleAction::Custom(_) => {
                // Custom action logic would go here
                Ok(None)
            }
        }
    }
    
    /// Apply routing strategy to select provider
    async fn apply_routing_strategy(
        &self,
        request: &UniversalAIRequest,
        providers: &Arc<RwLock<HashMap<String, Arc<dyn UniversalAIProvider>>>>
    ) -> Result<String> {
        let providers_guard = providers.read().await;
        
        if providers_guard.is_empty() {
            return Err(crate::error::types::MCPError::Configuration(
                "No providers available".to_string()
            ));
        }
        
        match &self.strategy {
            RoutingStrategy::BestFit => {
                self.find_best_fit_provider(request, &providers_guard).await
            }
            RoutingStrategy::LowestCost | RoutingStrategy::CostOptimized => {
                self.find_lowest_cost_provider(request, &providers_guard).await
            }
            RoutingStrategy::LowestLatency | RoutingStrategy::LatencyOptimized => {
                self.find_lowest_latency_provider(request, &providers_guard).await
            }
            RoutingStrategy::HighestQuality => {
                self.find_highest_quality_provider(request, &providers_guard).await
            }
            RoutingStrategy::LocalFirst => {
                self.find_local_first_provider(request, &providers_guard).await
            }
            RoutingStrategy::CloudFirst => {
                self.find_cloud_first_provider(request, &providers_guard).await
            }
            RoutingStrategy::RoundRobin => {
                self.find_round_robin_provider(&providers_guard).await
            }
            RoutingStrategy::WeightedRandom => {
                self.find_weighted_random_provider(&providers_guard).await
            }
            RoutingStrategy::Custom(strategy) => {
                self.apply_custom_strategy(strategy, request, &providers_guard).await
            }
        }
    }
    
    /// Find best fit provider based on capabilities
    ///
    /// Scores providers based on:
    /// - Model capability match
    /// - Performance history
    /// - Current load
    async fn find_best_fit_provider(
        &self,
        request: &UniversalAIRequest,
        providers: &HashMap<String, Arc<dyn UniversalAIProvider>>
    ) -> Result<String> {
        if providers.is_empty() {
            return Err(crate::error::types::MCPError::Configuration(
                "No providers available".to_string()
            ));
        }
        
        let performance = self.performance_history.read().await;
        let mut best_provider = None;
        let mut best_score = f64::MIN;
        
        for (provider_id, _provider) in providers {
            let mut score = 0.0;
            
            // Score based on performance history
            if let Some(metrics) = performance.get(provider_id) {
                // Higher success rate is better
                score += metrics.success_rate * 50.0;
                
                // Lower latency is better (inverse score)
                if metrics.avg_latency_ms > 0.0 {
                    score += 1000.0 / metrics.avg_latency_ms;
                }
                
                // Lower cost is better (inverse score)
                if metrics.avg_cost > 0.0 {
                    score += 10.0 / metrics.avg_cost;
                }
                
                // Penalize high load
                score -= metrics.current_load as f64 * 0.5;
            } else {
                // New provider gets a neutral score
                score = 25.0;
            }
            
            // Model-specific scoring
            if provider_id.contains(&request.model) {
                score += 20.0; // Bonus for exact model match
            }
            
            if score > best_score {
                best_score = score;
                best_provider = Some(provider_id.clone());
            }
        }
        
        best_provider.ok_or_else(|| crate::error::types::MCPError::Configuration(
            "No suitable provider found".to_string()
        ))
    }
    
    /// Find lowest cost provider
    ///
    /// Selects provider with lowest average cost per request
    async fn find_lowest_cost_provider(
        &self,
        _request: &UniversalAIRequest,
        providers: &HashMap<String, Arc<dyn UniversalAIProvider>>
    ) -> Result<String> {
        if providers.is_empty() {
            return Err(crate::error::types::MCPError::Configuration(
                "No providers available".to_string()
            ));
        }
        
        let performance = self.performance_history.read().await;
        let mut lowest_cost_provider = None;
        let mut lowest_cost = f64::MAX;
        
        for (provider_id, _provider) in providers {
            let cost = if let Some(metrics) = performance.get(provider_id) {
                metrics.avg_cost
            } else {
                // Unknown cost - assume moderate
                0.01
            };
            
            if cost < lowest_cost {
                lowest_cost = cost;
                lowest_cost_provider = Some(provider_id.clone());
            }
        }
        
        lowest_cost_provider.ok_or_else(|| crate::error::types::MCPError::Configuration(
            "No providers available".to_string()
        ))
    }
    
    /// Find lowest latency provider
    async fn find_lowest_latency_provider(
        &self,
        _request: &UniversalAIRequest,
        providers: &HashMap<String, Arc<dyn UniversalAIProvider>>
    ) -> Result<String> {
        let performance = self.performance_history.read().await;
        
        // Find provider with lowest average latency
        let mut best_provider = None;
        let mut best_latency = std::time::Duration::MAX;
        
        for (name, _) in providers.iter() {
            if let Some(metrics) = performance.get(name) {
                if metrics.avg_latency < best_latency {
                    best_latency = metrics.avg_latency;
                    best_provider = Some(name.clone());
                }
            }
        }
        
        // If no performance data, return first provider
        best_provider.or_else(|| providers.keys().next().map(|s| s.clone()))
            .ok_or_else(|| crate::error::types::MCPError::Configuration(
                "No providers available".to_string()
            ))
    }
    
    /// Find highest quality provider
    async fn find_highest_quality_provider(
        &self,
        _request: &UniversalAIRequest,
        providers: &HashMap<String, Arc<dyn UniversalAIProvider>>
    ) -> Result<String> {
        let performance = self.performance_history.read().await;
        
        // Find provider with highest quality score
        let mut best_provider = None;
        let mut best_quality = 0.0;
        
        for (name, _) in providers.iter() {
            if let Some(metrics) = performance.get(name) {
                if let Some(quality) = metrics.quality_score {
                    if quality > best_quality {
                        best_quality = quality;
                        best_provider = Some(name.clone());
                    }
                }
            }
        }
        
        // If no performance data, return first provider
        best_provider.or_else(|| providers.keys().next().map(|s| s.clone()))
            .ok_or_else(|| crate::error::types::MCPError::Configuration(
                "No providers available".to_string()
            ))
    }
    
    /// Find local provider first
    async fn find_local_first_provider(
        &self,
        _request: &UniversalAIRequest,
        providers: &HashMap<String, Arc<dyn UniversalAIProvider>>
    ) -> Result<String> {
        // First try to find local providers
        for (name, provider) in providers.iter() {
            let provider_type = provider.provider_type();
            if matches!(provider_type, crate::enhanced::providers::ProviderType::LocalServer | 
                                      crate::enhanced::providers::ProviderType::LocalNative) {
                return Ok(name.clone());
            }
        }
        
        // If no local providers, use any available
        providers.keys().next()
            .ok_or_else(|| crate::error::types::MCPError::Configuration(
                "No providers available".to_string()
            ))
            .map(|s| s.clone())
    }
    
    /// Find cloud provider first
    async fn find_cloud_first_provider(
        &self,
        _request: &UniversalAIRequest,
        providers: &HashMap<String, Arc<dyn UniversalAIProvider>>
    ) -> Result<String> {
        // First try to find cloud providers
        for (name, provider) in providers.iter() {
            let provider_type = provider.provider_type();
            if matches!(provider_type, crate::enhanced::providers::ProviderType::CloudAPI) {
                return Ok(name.clone());
            }
        }
        
        // If no cloud providers, use any available
        providers.keys().next()
            .ok_or_else(|| crate::error::types::MCPError::Configuration(
                "No providers available".to_string()
            ))
            .map(|s| s.clone())
    }
    
    /// Find provider using round robin
    ///
    /// Distributes requests evenly across all providers
    async fn find_round_robin_provider(
        &self,
        providers: &HashMap<String, Arc<dyn UniversalAIProvider>>
    ) -> Result<String> {
        if providers.is_empty() {
            return Err(crate::error::types::MCPError::Configuration(
                "No providers available".to_string()
            ));
        }
        
        // Get provider list (sorted for consistent ordering)
        let mut provider_ids: Vec<String> = providers.keys().cloned().collect();
        provider_ids.sort();
        
        // Get next provider using atomic counter
        let index = self.round_robin_counter.fetch_add(1, Ordering::Relaxed);
        let selected_index = index % provider_ids.len();
        
        Ok(provider_ids[selected_index].clone())
    }
    
    /// Find provider using weighted random
    ///
    /// Weights providers based on success rate and performance
    async fn find_weighted_random_provider(
        &self,
        providers: &HashMap<String, Arc<dyn UniversalAIProvider>>
    ) -> Result<String> {
        if providers.is_empty() {
            return Err(crate::error::types::MCPError::Configuration(
                "No providers available".to_string()
            ));
        }
        
        let performance = self.performance_history.read().await;
        let mut weights: Vec<(String, f64)> = Vec::new();
        let mut total_weight = 0.0;
        
        for (provider_id, _provider) in providers {
            let weight = if let Some(metrics) = performance.get(provider_id) {
                // Weight based on success rate and inverse latency
                let success_weight = metrics.success_rate * 100.0;
                let latency_weight = if metrics.avg_latency_ms > 0.0 {
                    100.0 / metrics.avg_latency_ms
                } else {
                    1.0
                };
                success_weight + latency_weight
            } else {
                // Default weight for unknown providers
                50.0
            };
            
            total_weight += weight;
            weights.push((provider_id.clone(), weight));
        }
        
        // Generate random value between 0 and total_weight
        use std::collections::hash_map::RandomState;
        use std::hash::{BuildHasher, Hash, Hasher};
        
        let random_state = RandomState::new();
        let mut hasher = random_state.build_hasher();
        std::time::SystemTime::now().hash(&mut hasher);
        let random_value = (hasher.finish() as f64 / u64::MAX as f64) * total_weight;
        
        // Select provider based on weighted random
        let mut cumulative = 0.0;
        for (provider_id, weight) in weights {
            cumulative += weight;
            if random_value <= cumulative {
                return Ok(provider_id);
            }
        }
        
        // Fallback to first provider (shouldn't reach here)
        Ok(weights[0].0.clone())
    }
    
    /// Apply custom routing strategy
    ///
    /// Supports custom strategies defined by users
    /// Format: "custom:strategy_name"
    async fn apply_custom_strategy(
        &self,
        strategy: &str,
        request: &UniversalAIRequest,
        providers: &HashMap<String, Arc<dyn UniversalAIProvider>>
    ) -> Result<String> {
        if providers.is_empty() {
            return Err(crate::error::types::MCPError::Configuration(
                "No providers available".to_string()
            ));
        }
        
        // Parse custom strategy
        match strategy {
            "fastest" => {
                // Select provider with lowest average latency
                let performance = self.performance_history.read().await;
                let mut fastest_provider = None;
                let mut lowest_latency = f64::MAX;
                
                for (provider_id, _provider) in providers {
                    let latency = if let Some(metrics) = performance.get(provider_id) {
                        metrics.avg_latency_ms
                    } else {
                        1000.0 // Unknown - assume slow
                    };
                    
                    if latency < lowest_latency {
                        lowest_latency = latency;
                        fastest_provider = Some(provider_id.clone());
                    }
                }
                
                fastest_provider.ok_or_else(|| crate::error::types::MCPError::Configuration(
                    "No providers available".to_string()
                ))
            }
            "most_reliable" => {
                // Select provider with highest success rate
                let performance = self.performance_history.read().await;
                let mut most_reliable = None;
                let mut highest_success = 0.0;
                
                for (provider_id, _provider) in providers {
                    let success_rate = if let Some(metrics) = performance.get(provider_id) {
                        metrics.success_rate
                    } else {
                        0.5 // Unknown - assume moderate
                    };
                    
                    if success_rate > highest_success {
                        highest_success = success_rate;
                        most_reliable = Some(provider_id.clone());
                    }
                }
                
                most_reliable.ok_or_else(|| crate::error::types::MCPError::Configuration(
                    "No providers available".to_string()
                ))
            }
            "prefer_local" => {
                // Prefer providers with "local" in their ID
                for (provider_id, _provider) in providers {
                    if provider_id.contains("local") || provider_id.contains("native") {
                        return Ok(provider_id.clone());
                    }
                }
                // Fallback to best fit
                self.find_best_fit_provider(request, providers).await
            }
            "prefer_cloud" => {
                // Prefer providers with cloud service names
                for (provider_id, _provider) in providers {
                    if provider_id.contains("openai") || provider_id.contains("anthropic") 
                        || provider_id.contains("google") || provider_id.contains("azure") {
                        return Ok(provider_id.clone());
                    }
                }
                // Fallback to best fit
                self.find_best_fit_provider(request, providers).await
            }
            _ => {
                // Unknown custom strategy - fallback to best fit
                tracing::warn!("Unknown custom strategy '{}', falling back to best fit", strategy);
                self.find_best_fit_provider(request, providers).await
            }
        }
    }
    
    /// Add routing rule
    pub async fn add_rule(&self, rule: RoutingRule) -> Result<()> {
        let mut rules = self.rules.write().await;
        rules.push(rule);
        Ok(())
    }
    
    /// Remove routing rule
    pub async fn remove_rule(&self, rule_name: &str) -> Result<bool> {
        let mut rules = self.rules.write().await;
        let initial_len = rules.len();
        rules.retain(|r| r.name != rule_name);
        Ok(rules.len() < initial_len)
    }
    
    /// Update performance metrics for a provider
    pub async fn update_performance(&self, provider_name: &str, metrics: PerformanceMetrics) -> Result<()> {
        let mut performance = self.performance_history.write().await;
        performance.insert(provider_name.to_string(), metrics);
        Ok(())
    }
    
    /// Get performance metrics for a provider
    pub async fn get_performance(&self, provider_name: &str) -> Result<Option<PerformanceMetrics>> {
        let performance = self.performance_history.read().await;
        Ok(performance.get(provider_name).cloned())
    }
    
    /// Set fallback chain
    pub async fn set_fallback_chain(&self, chain: Vec<String>) -> Result<()> {
        let mut fallback = self.fallback_chain.write().await;
        *fallback = chain;
        Ok(())
    }
    
    /// Get fallback chain
    pub async fn get_fallback_chain(&self) -> Result<Vec<String>> {
        let fallback = self.fallback_chain.read().await;
        Ok(fallback.clone())
    }
}

// Helper trait for converting AIRequestType to string
trait AIRequestTypeExt {
    fn to_string(&self) -> String;
}

impl AIRequestTypeExt for super::types::AIRequestType {
    fn to_string(&self) -> String {
        match self {
            super::types::AIRequestType::TextGeneration => "text_generation".to_string(),
            super::types::AIRequestType::ImageGeneration => "image_generation".to_string(),
            super::types::AIRequestType::ImageAnalysis => "image_analysis".to_string(),
            super::types::AIRequestType::AudioGeneration => "audio_generation".to_string(),
            super::types::AIRequestType::AudioTranscription => "audio_transcription".to_string(),
            super::types::AIRequestType::VideoGeneration => "video_generation".to_string(),
            super::types::AIRequestType::VideoAnalysis => "video_analysis".to_string(),
            super::types::AIRequestType::Embeddings => "embeddings".to_string(),
            super::types::AIRequestType::FineTuning => "fine_tuning".to_string(),
            super::types::AIRequestType::Evaluation => "evaluation".to_string(),
            super::types::AIRequestType::Custom(s) => s.clone(),
            super::types::AIRequestType::Future(s) => s.clone(),
        }
    }
} 