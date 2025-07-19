//! AI routing functionality for the enhanced coordinator
//!
//! This module provides intelligent routing of AI requests to the most
//! appropriate providers based on various strategies and rules.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

use crate::error::types::Result;
use crate::enhanced::ai_types::UniversalAIRequest;
use crate::enhanced::providers::UniversalAIProvider;

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

/// Routing strategies for universal AI coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingStrategy {
    /// Best fit based on capabilities
    BestFit,
    
    /// Lowest cost
    LowestCost,
    
    /// Cost optimized
    CostOptimized,
    
    /// Fastest response
    LowestLatency,
    
    /// Latency optimized
    LatencyOptimized,
    
    /// Highest quality
    HighestQuality,
    
    /// Local first (privacy)
    LocalFirst,
    
    /// Cloud first (performance)
    CloudFirst,
    
    /// Round robin
    RoundRobin,
    
    /// Weighted random
    WeightedRandom,
    
    /// Custom strategy
    Custom(String),
}

/// Routing rules for intelligent AI selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    /// Rule priority (higher = more important)
    pub priority: u32,
    
    /// Condition to match
    pub condition: RuleCondition,
    
    /// Action to take
    pub action: RuleAction,
    
    /// Rule name/description
    pub name: String,
}

/// Rule conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    /// Request contains sensitive data
    SensitiveData,
    
    /// Request requires high quality
    HighQuality,
    
    /// Request requires low latency
    LowLatency,
    
    /// Request requires low cost
    LowCost,
    
    /// Model type required
    ModelType(String),
    
    /// Task type required
    TaskType(String),
    
    /// Custom condition
    Custom(serde_json::Value),
}

/// Rule actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction {
    /// Prefer specific provider
    PreferProvider(String),
    
    /// Require local processing
    RequireLocal,
    
    /// Allow cloud processing
    AllowCloud,
    
    /// Set maximum cost
    MaxCost(f64),
    
    /// Set maximum latency
    MaxLatency(Duration),
    
    /// Custom action
    Custom(serde_json::Value),
}

/// Performance metrics for routing decisions
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub avg_latency: Duration,
    pub success_rate: f64,
    pub cost_per_request: Option<f64>,
    pub quality_score: Option<f64>,
}

/// Routing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    pub default_strategy: RoutingStrategy,
    pub fallback_enabled: bool,
    pub cost_optimization: bool,
    pub latency_optimization: bool,
    pub quality_optimization: bool,
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

    /// Route request to the best provider
    pub async fn route_request(
        &self, 
        _request: &UniversalAIRequest, 
        _providers: &Arc<RwLock<HashMap<String, Arc<dyn UniversalAIProvider>>>>
    ) -> Result<String> {
        // Implementation would evaluate routing rules and select provider
        // For now, return a default provider
        Ok("default".to_string())
    }

    /// Add a routing rule
    pub async fn add_rule(&self, rule: RoutingRule) -> Result<()> {
        let mut rules = self.rules.write().await;
        rules.push(rule);
        // Sort by priority (highest first)
        rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        Ok(())
    }

    /// Remove a routing rule by name
    pub async fn remove_rule(&self, name: &str) -> Result<bool> {
        let mut rules = self.rules.write().await;
        let initial_len = rules.len();
        rules.retain(|rule| rule.name != name);
        Ok(rules.len() != initial_len)
    }

    /// Get all routing rules
    pub async fn get_rules(&self) -> Vec<RoutingRule> {
        self.rules.read().await.clone()
    }

    /// Update performance metrics for a provider
    pub async fn update_performance(&self, provider: &str, metrics: PerformanceMetrics) {
        let mut history = self.performance_history.write().await;
        history.insert(provider.to_string(), metrics);
    }

    /// Get performance metrics for a provider
    pub async fn get_performance(&self, provider: &str) -> Option<PerformanceMetrics> {
        self.performance_history.read().await.get(provider).cloned()
    }

    /// Set fallback chain
    pub async fn set_fallback_chain(&self, providers: Vec<String>) {
        let mut fallback_chain = self.fallback_chain.write().await;
        *fallback_chain = providers;
    }

    /// Get fallback chain
    pub async fn get_fallback_chain(&self) -> Vec<String> {
        self.fallback_chain.read().await.clone()
    }

    /// Change routing strategy
    pub fn set_strategy(&mut self, strategy: RoutingStrategy) {
        self.strategy = strategy;
    }

    /// Get current routing strategy
    pub fn get_strategy(&self) -> &RoutingStrategy {
        &self.strategy
    }
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            default_strategy: RoutingStrategy::BestFit,
            fallback_enabled: true,
            cost_optimization: false,
            latency_optimization: false,
            quality_optimization: true,
        }
    }
}

impl RoutingStrategy {
    /// Get a description of the routing strategy
    pub fn description(&self) -> &'static str {
        match self {
            RoutingStrategy::BestFit => "Route to the best-fit provider based on capabilities",
            RoutingStrategy::LowestCost => "Route to the provider with the lowest cost",
            RoutingStrategy::CostOptimized => "Route with cost optimization considerations",
            RoutingStrategy::LowestLatency => "Route to the provider with the lowest latency",
            RoutingStrategy::LatencyOptimized => "Route with latency optimization considerations",
            RoutingStrategy::HighestQuality => "Route to the provider with the highest quality",
            RoutingStrategy::LocalFirst => "Prefer local providers for privacy",
            RoutingStrategy::CloudFirst => "Prefer cloud providers for performance",
            RoutingStrategy::RoundRobin => "Distribute requests evenly across providers",
            RoutingStrategy::WeightedRandom => "Randomly select providers based on weights",
            RoutingStrategy::Custom(_) => "Custom routing strategy",
        }
    }
} 