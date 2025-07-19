//! Universal AI Router
//!
//! This module contains the AIRouter implementation for intelligent routing
//! to any AI system based on capabilities, cost, latency, and quality requirements.

use std::collections::HashMap;
use std::sync::Arc;
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
}

impl AIRouter {
    /// Create a new AI router
    pub fn new(config: RoutingConfig) -> Self {
        Self {
            strategy: config.default_strategy,
            performance_history: Arc::new(RwLock::new(HashMap::new())),
            rules: Arc::new(RwLock::new(Vec::new())),
            fallback_chain: Arc::new(RwLock::new(Vec::new())),
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
    async fn find_best_fit_provider(
        &self,
        request: &UniversalAIRequest,
        providers: &HashMap<String, Arc<dyn UniversalAIProvider>>
    ) -> Result<String> {
        // For now, return first available provider
        // TODO: Implement actual capability matching
        providers.keys().next()
            .ok_or_else(|| crate::error::types::MCPError::Configuration(
                "No providers available".to_string()
            ))
            .map(|s| s.clone())
    }
    
    /// Find lowest cost provider
    async fn find_lowest_cost_provider(
        &self,
        _request: &UniversalAIRequest,
        providers: &HashMap<String, Arc<dyn UniversalAIProvider>>
    ) -> Result<String> {
        // For now, return first available provider
        // TODO: Implement actual cost estimation
        providers.keys().next()
            .ok_or_else(|| crate::error::types::MCPError::Configuration(
                "No providers available".to_string()
            ))
            .map(|s| s.clone())
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
    async fn find_round_robin_provider(
        &self,
        providers: &HashMap<String, Arc<dyn UniversalAIProvider>>
    ) -> Result<String> {
        // For now, return first available provider
        // TODO: Implement actual round robin logic
        providers.keys().next()
            .ok_or_else(|| crate::error::types::MCPError::Configuration(
                "No providers available".to_string()
            ))
            .map(|s| s.clone())
    }
    
    /// Find provider using weighted random
    async fn find_weighted_random_provider(
        &self,
        providers: &HashMap<String, Arc<dyn UniversalAIProvider>>
    ) -> Result<String> {
        // For now, return first available provider
        // TODO: Implement actual weighted random logic
        providers.keys().next()
            .ok_or_else(|| crate::error::types::MCPError::Configuration(
                "No providers available".to_string()
            ))
            .map(|s| s.clone())
    }
    
    /// Apply custom routing strategy
    async fn apply_custom_strategy(
        &self,
        _strategy: &str,
        _request: &UniversalAIRequest,
        providers: &HashMap<String, Arc<dyn UniversalAIProvider>>
    ) -> Result<String> {
        // For now, return first available provider
        // TODO: Implement custom strategy logic
        providers.keys().next()
            .ok_or_else(|| crate::error::types::MCPError::Configuration(
                "No providers available".to_string()
            ))
            .map(|s| s.clone())
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