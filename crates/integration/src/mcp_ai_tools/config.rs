//! MCP-AI Tools integration configuration
//!
//! Configuration structures for the MCP-AI Tools integration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Configuration for the MCP-AI Tools adapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpAiToolsConfig {
    /// Default AI provider to use
    pub default_provider: String,
    
    /// Settings for different providers
    pub providers: HashMap<String, ProviderSettings>,
    
    /// Default timeout for requests in milliseconds
    pub default_timeout_ms: u64,
    
    /// Maximum size of message history cache
    pub max_history_size: usize,
    
    /// Default conversation context size
    pub default_context_size: usize,
    
    /// Enable streaming responses
    pub enable_streaming: bool,
    
    /// Cache size for conversation history
    pub history_cache_size: usize,
}

impl Default for McpAiToolsConfig {
    fn default() -> Self {
        let mut providers = HashMap::new();
        providers.insert("openai".to_string(), ProviderSettings::default_openai());
        providers.insert("anthropic".to_string(), ProviderSettings::default_anthropic());
        providers.insert("gemini".to_string(), ProviderSettings::default_gemini());
        
        Self {
            default_provider: "openai".to_string(),
            providers,
            default_timeout_ms: 30000,
            max_history_size: 100,
            default_context_size: 10,
            enable_streaming: true,
            history_cache_size: 1000,
        }
    }
}

impl McpAiToolsConfig {
    /// Create a new configuration with the given default provider
    pub fn new(default_provider: impl Into<String>) -> Self {
        let mut config = Self::default();
        config.default_provider = default_provider.into();
        config
    }
    
    /// Add a provider with the given settings
    pub fn with_provider(mut self, name: impl Into<String>, settings: ProviderSettings) -> Self {
        self.providers.insert(name.into(), settings);
        self
    }
    
    /// Set the default timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.default_timeout_ms = timeout_ms;
        self
    }
    
    /// Set streaming mode
    pub fn with_streaming(mut self, enable: bool) -> Self {
        self.enable_streaming = enable;
        self
    }
    
    /// Get the default timeout as a Duration
    pub fn default_timeout(&self) -> Duration {
        Duration::from_millis(self.default_timeout_ms)
    }
}

/// Settings for a specific AI provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSettings {
    /// API key for the provider
    pub api_key: String,
    
    /// Default model to use
    pub default_model: String,
    
    /// Available models
    pub available_models: Vec<String>,
    
    /// Default parameters for the provider
    pub default_parameters: HashMap<String, serde_json::Value>,
    
    /// Provider-specific timeout in milliseconds
    pub timeout_ms: Option<u64>,
}

impl ProviderSettings {
    /// Create new provider settings
    pub fn new(api_key: impl Into<String>, default_model: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            default_model: default_model.into(),
            available_models: vec![],
            default_parameters: HashMap::new(),
            timeout_ms: None,
        }
    }
    
    /// Add available models
    pub fn with_models(mut self, models: Vec<impl Into<String>>) -> Self {
        self.available_models = models.into_iter().map(|m| m.into()).collect();
        self
    }
    
    /// Add a default parameter
    pub fn with_parameter(mut self, name: impl Into<String>, value: serde_json::Value) -> Self {
        self.default_parameters.insert(name.into(), value);
        self
    }
    
    /// Set provider timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }
    
    /// Default settings for OpenAI
    pub fn default_openai() -> Self {
        Self {
            api_key: "".to_string(),
            default_model: "gpt-4o".to_string(),
            available_models: vec![
                "gpt-4o".to_string(),
                "gpt-4-turbo".to_string(),
                "gpt-3.5-turbo".to_string(),
            ],
            default_parameters: {
                let mut params = HashMap::new();
                params.insert("temperature".to_string(), serde_json::json!(0.7));
                params.insert("top_p".to_string(), serde_json::json!(1.0));
                params
            },
            timeout_ms: Some(30000),
        }
    }
    
    /// Default settings for Anthropic
    pub fn default_anthropic() -> Self {
        Self {
            api_key: "".to_string(),
            default_model: "claude-3-opus-20240229".to_string(),
            available_models: vec![
                "claude-3-opus-20240229".to_string(),
                "claude-3-sonnet-20240229".to_string(),
                "claude-3-haiku-20240307".to_string(),
            ],
            default_parameters: {
                let mut params = HashMap::new();
                params.insert("temperature".to_string(), serde_json::json!(0.7));
                params.insert("top_p".to_string(), serde_json::json!(1.0));
                params
            },
            timeout_ms: Some(60000),
        }
    }
    
    /// Default settings for Gemini
    pub fn default_gemini() -> Self {
        Self {
            api_key: "".to_string(),
            default_model: "gemini-1.5-pro".to_string(),
            available_models: vec![
                "gemini-1.5-pro".to_string(),
                "gemini-1.5-flash".to_string(),
                "gemini-1.0-pro".to_string(),
            ],
            default_parameters: {
                let mut params = HashMap::new();
                params.insert("temperature".to_string(), serde_json::json!(0.7));
                params.insert("top_p".to_string(), serde_json::json!(1.0));
                params
            },
            timeout_ms: Some(30000),
        }
    }
} 