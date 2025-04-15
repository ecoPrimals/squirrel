//! AI Agent adapter configuration
//!
//! This module provides configuration structures for the AI Agent adapter.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Circuit breaker configuration for resilience
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening the circuit
    pub failure_threshold: u32,
    
    /// Time in milliseconds before attempting to close the circuit
    pub reset_timeout: u64,
    
    /// Number of calls to allow in half-open state
    pub half_open_max_calls: u32,
    
    /// Number of successful calls needed to close the circuit from half-open state
    pub half_open_success_threshold: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            reset_timeout: 30000, // 30 seconds
            half_open_max_calls: 3,
            half_open_success_threshold: 2,
        }
    }
}

/// Resource limits for AI operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum number of tokens to use per request
    pub max_tokens: u32,
    
    /// Maximum number of requests per minute
    pub max_requests_per_minute: u32,
    
    /// Maximum number of concurrent requests
    pub max_concurrent_requests: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_tokens: 4000,
            max_requests_per_minute: 60,
            max_concurrent_requests: 10,
        }
    }
}

/// Configuration for the AI Agent adapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAgentConfig {
    /// AI service provider (openai, anthropic, gemini)
    pub provider: String,
    
    /// API key for the AI service
    pub api_key: String,
    
    /// Model to use for AI operations
    pub model: String,
    
    /// Timeout for API requests in milliseconds
    pub timeout_ms: u64,
    
    /// Maximum number of retry attempts
    pub max_retries: u32,
    
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,
    
    /// Resource usage limits
    pub resource_limits: ResourceLimits,
    
    /// Cache size for responses
    pub cache_size: Option<usize>,
}

impl AIAgentConfig {
    /// Create a new configuration with the given provider and API key
    pub fn new(provider: impl Into<String>, api_key: impl Into<String>) -> Self {
        let provider_str = provider.into();
        Self {
            provider: provider_str.clone(),
            api_key: api_key.into(),
            model: default_model_for_provider(&provider_str),
            timeout_ms: 10000, // 10 seconds
            max_retries: 3,
            circuit_breaker: CircuitBreakerConfig::default(),
            resource_limits: ResourceLimits::default(),
            cache_size: Some(100),
        }
    }
    
    /// Set the model to use
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }
    
    /// Set the timeout for API requests
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
    
    /// Set the maximum number of retry attempts
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }
    
    /// Set the circuit breaker configuration
    pub fn with_circuit_breaker(mut self, circuit_breaker: CircuitBreakerConfig) -> Self {
        self.circuit_breaker = circuit_breaker;
        self
    }
    
    /// Set the resource limits
    pub fn with_resource_limits(mut self, resource_limits: ResourceLimits) -> Self {
        self.resource_limits = resource_limits;
        self
    }
    
    /// Set the cache size
    pub fn with_cache_size(mut self, cache_size: usize) -> Self {
        self.cache_size = Some(cache_size);
        self
    }
    
    /// Get the timeout as a Duration
    pub fn timeout_duration(&self) -> Duration {
        Duration::from_millis(self.timeout_ms)
    }
}

impl Default for AIAgentConfig {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            api_key: "".to_string(),
            model: "gpt-4o".to_string(),
            timeout_ms: 10000, // 10 seconds
            max_retries: 3,
            circuit_breaker: CircuitBreakerConfig::default(),
            resource_limits: ResourceLimits::default(),
            cache_size: Some(100),
        }
    }
}

/// Returns the default model for a given provider
fn default_model_for_provider(provider: &str) -> String {
    match provider.to_lowercase().as_str() {
        "openai" => "gpt-4o".to_string(),
        "anthropic" => "claude-3-opus-20240229".to_string(),
        "gemini" => "gemini-1.5-pro".to_string(),
        _ => "gpt-4o".to_string(),
    }
} 