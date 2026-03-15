// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use squirrel_mcp::resilience::retry::{RetryConfig, BackoffStrategy};

use crate::common::{ChatRequest, ChatResponse, ChatResponseStream, AIClient};
use crate::common::capability::AICapabilities;
use crate::Result;

pub mod openrouter;
pub mod llamacpp;
pub mod huggingface;
pub mod registry;

/// Configuration for a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider_type: String,
    pub name: String,
    pub enabled: bool,
    pub priority: u8,
    pub settings: HashMap<String, serde_json::Value>,
    pub rate_limits: Option<RateLimitConfig>,
    pub retry_config: Option<RetryConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub requests_per_hour: Option<u32>,
    pub requests_per_day: Option<u32>,
    pub burst_limit: Option<u32>,
}

/// Plugin trait for AI providers
#[async_trait]
pub trait ProviderPlugin: Send + Sync + 'static {
    /// Get the provider name
    fn name(&self) -> &str;
    
    /// Get the provider version
    fn version(&self) -> &str;
    
    /// Get the provider type (cloud, local, hybrid, etc.)
    fn provider_type(&self) -> ProviderType;
    
    /// Get supported models
    fn supported_models(&self) -> Vec<ModelInfo>;
    
    /// Initialize the provider with configuration
    async fn initialize(&mut self, config: ProviderConfig) -> Result<()>;
    
    /// Check if the provider is healthy and available
    async fn health_check(&self) -> Result<HealthStatus>;
    
    /// Get provider capabilities
    async fn get_capabilities(&self, model: &str) -> Result<AICapabilities>;
    
    /// Send a chat request
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;
    
    /// Send a streaming chat request
    async fn chat_stream(&self, request: ChatRequest) -> Result<ChatResponseStream>;
    
    /// List available models (dynamic discovery)
    async fn list_models(&self) -> Result<Vec<ModelInfo>>;
    
    /// Get cost estimate for a request
    async fn estimate_cost(&self, request: &ChatRequest) -> Result<CostEstimate>;
    
    /// Get provider statistics
    async fn get_stats(&self) -> Result<ProviderStats>;
    
    /// Shutdown the provider gracefully
    async fn shutdown(&mut self) -> Result<()>;
}

/// Provider type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProviderType {
    /// Cloud-based API provider
    Cloud,
    /// Local model provider
    Local,
    /// Hybrid (local with cloud fallback)
    Hybrid,
    /// Proxy/aggregator (like OpenRouter)
    Proxy,
    /// Remote squirrel node
    Remote,
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub context_window: Option<usize>,
    pub max_output_tokens: Option<usize>,
    pub input_cost_per_1k_tokens: Option<f64>,
    pub output_cost_per_1k_tokens: Option<f64>,
    pub capabilities: Vec<ModelCapability>,
    pub tags: Vec<String>,
}

/// Model capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelCapability {
    TextGeneration,
    TextCompletion,
    ChatCompletion,
    FunctionCalling,
    ToolUse,
    CodeGeneration,
    ImageGeneration,
    ImageUnderstanding,
    AudioTranscription,
    AudioGeneration,
    Embedding,
    Reasoning,
    Math,
    Coding,
}

/// Health status of a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: HealthLevel,
    pub message: Option<String>,
    pub last_checked: chrono::DateTime<chrono::Utc>,
    pub response_time_ms: Option<u64>,
    pub error_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthLevel {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Cost estimate for a request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimate {
    pub estimated_input_tokens: u32,
    pub estimated_output_tokens: u32,
    pub estimated_cost_usd: f64,
    pub currency: String,
    pub breakdown: Option<CostBreakdown>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    pub input_cost: f64,
    pub output_cost: f64,
    pub fixed_cost: Option<f64>,
    pub additional_fees: Option<f64>,
}

/// Provider statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: f64,
    pub total_tokens_processed: u64,
    pub total_cost_usd: f64,
    pub uptime_percentage: f64,
    pub last_error: Option<String>,
    pub last_error_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// Adapter to make ProviderPlugin compatible with AIClient
pub struct PluginAdapter {
    plugin: Box<dyn ProviderPlugin>,
    config: ProviderConfig,
}

impl PluginAdapter {
    pub fn new(plugin: Box<dyn ProviderPlugin>, config: ProviderConfig) -> Self {
        Self { plugin, config }
    }
    
    pub fn plugin(&self) -> &dyn ProviderPlugin {
        self.plugin.as_ref()
    }
    
    pub fn config(&self) -> &ProviderConfig {
        &self.config
    }
}

#[async_trait]
impl AIClient for PluginAdapter {
    fn provider_name(&self) -> &str {
        self.plugin.name()
    }
    
    async fn get_capabilities(&self, model: &str) -> Result<AICapabilities> {
        self.plugin.get_capabilities(model).await
    }
    
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        self.plugin.chat(request).await
    }
    
    async fn chat_stream(&self, request: ChatRequest) -> Result<ChatResponseStream> {
        self.plugin.chat_stream(request).await
    }
    
    async fn list_models(&self) -> Result<Vec<String>> {
        let models = self.plugin.list_models().await?;
        Ok(models.into_iter().map(|m| m.id).collect())
    }
    
    async fn is_available(&self) -> bool {
        match self.plugin.health_check().await {
            Ok(status) => status.status == HealthLevel::Healthy,
            Err(_) => false,
        }
    }
}

/// Factory for creating provider plugins
pub type ProviderFactory = fn(config: ProviderConfig) -> Result<Box<dyn ProviderPlugin>>;

/// Registry for provider plugins
pub struct ProviderPluginRegistry {
    factories: HashMap<String, ProviderFactory>,
    instances: HashMap<String, Arc<PluginAdapter>>,
}

impl ProviderPluginRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            factories: HashMap::new(),
            instances: HashMap::new(),
        };
        
        // Register built-in providers
        registry.register_factory("openrouter", openrouter::create_provider);
        registry.register_factory("llamacpp", llamacpp::create_provider);
        registry.register_factory("huggingface", huggingface::create_provider);
        
        registry
    }
    
    /// Register a provider factory
    pub fn register_factory(&mut self, provider_type: &str, factory: ProviderFactory) {
        self.factories.insert(provider_type.to_string(), factory);
    }
    
    /// Create and register a provider instance
    pub async fn create_provider(&mut self, config: ProviderConfig) -> Result<Arc<PluginAdapter>> {
        let factory = self.factories.get(&config.provider_type)
            .ok_or_else(|| crate::error::Error::Configuration(
                format!("Unknown provider type: {}", config.provider_type)
            ))?;
        
        let mut plugin = factory(config.clone())?;
        plugin.initialize(config.clone()).await?;
        
        let adapter = Arc::new(PluginAdapter::new(plugin, config.clone()));
        self.instances.insert(config.name.clone(), adapter.clone());
        
        Ok(adapter)
    }
    
    /// Get a provider instance
    pub fn get_provider(&self, name: &str) -> Option<Arc<PluginAdapter>> {
        self.instances.get(name).cloned()
    }
    
    /// List all registered provider types
    pub fn list_provider_types(&self) -> Vec<String> {
        self.factories.keys().cloned().collect()
    }
    
    /// List all active provider instances
    pub fn list_instances(&self) -> Vec<Arc<PluginAdapter>> {
        self.instances.values().cloned().collect()
    }
    
    /// Remove a provider instance
    pub async fn remove_provider(&mut self, name: &str) -> Result<()> {
        if let Some(adapter) = self.instances.remove(name) {
            // Try to shutdown gracefully
            if let Ok(mut plugin) = Arc::try_unwrap(adapter) {
                let _ = plugin.plugin.shutdown().await;
            }
        }
        Ok(())
    }
    
    /// Health check all providers
    pub async fn health_check_all(&self) -> HashMap<String, HealthStatus> {
        let mut results = HashMap::new();
        
        for (name, adapter) in &self.instances {
            match adapter.plugin.health_check().await {
                Ok(status) => {
                    results.insert(name.clone(), status);
                }
                Err(_) => {
                    results.insert(name.clone(), HealthStatus {
                        status: HealthLevel::Unhealthy,
                        message: Some("Health check failed".to_string()),
                        last_checked: chrono::Utc::now(),
                        response_time_ms: None,
                        error_rate: None,
                    });
                }
            }
        }
        
        results
    }
}

impl Default for ProviderPluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro for easier provider plugin registration
#[macro_export]
macro_rules! register_provider {
    ($registry:expr, $type:expr, $factory:expr) => {
        $registry.register_factory($type, $factory);
    };
}

/// Helper trait for provider configuration validation
pub trait ConfigValidator {
    fn validate(&self) -> Result<()>;
    fn required_fields(&self) -> Vec<&'static str>;
    fn optional_fields(&self) -> Vec<&'static str>;
}

impl ConfigValidator for ProviderConfig {
    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(crate::error::Error::Configuration(
                "Provider name cannot be empty".to_string()
            ));
        }
        
        if self.provider_type.is_empty() {
            return Err(crate::error::Error::Configuration(
                "Provider type cannot be empty".to_string()
            ));
        }
        
        if self.priority > 100 {
            return Err(crate::error::Error::Configuration(
                "Provider priority must be between 0 and 100".to_string()
            ));
        }
        
        Ok(())
    }
    
    fn required_fields(&self) -> Vec<&'static str> {
        vec!["name", "provider_type", "enabled"]
    }
    
    fn optional_fields(&self) -> Vec<&'static str> {
        vec!["priority", "settings", "rate_limits", "retry_config"]
    }
} 