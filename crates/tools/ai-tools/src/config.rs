//! Configuration for AI tools
//!
//! This module provides configuration structures for AI providers and tools.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

/// Configuration for AI tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIToolsConfig {
    /// Default provider to use
    pub default_provider: String,

    /// Provider configurations
    pub providers: HashMap<String, ProviderConfig>,

    /// Request timeout in seconds
    pub request_timeout: u64,

    /// Maximum retries for failed requests
    pub max_retries: u32,

    /// Enable request logging
    pub enable_logging: bool,

    /// Routing strategy for provider selection
    pub routing_strategy: String,
}

/// Configuration for a specific AI provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider type (openai, anthropic, ollama, etc.)
    pub provider_type: String,

    /// API key for the provider
    pub api_key: Option<String>,

    /// Base URL for the provider API
    pub base_url: Option<String>,

    /// Default model to use
    pub default_model: Option<String>,

    /// Provider-specific settings
    pub settings: HashMap<String, serde_json::Value>,
}

impl Default for AIToolsConfig {
    fn default() -> Self {
        Self {
            default_provider: "openai".to_string(),
            providers: Self::default_providers(),
            request_timeout: 30,
            max_retries: 3,
            enable_logging: true,
            routing_strategy: "round_robin".to_string(),
        }
    }
}

impl AIToolsConfig {
    /// Create a new configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Override default provider if specified
        if let Ok(provider) = env::var("AI_DEFAULT_PROVIDER") {
            config.default_provider = provider;
        }

        // Override timeout if specified
        if let Ok(timeout) = env::var("AI_REQUEST_TIMEOUT") {
            if let Ok(timeout_val) = timeout.parse::<u64>() {
                config.request_timeout = timeout_val;
            }
        }

        // Override max retries if specified
        if let Ok(retries) = env::var("AI_MAX_RETRIES") {
            if let Ok(retries_val) = retries.parse::<u32>() {
                config.max_retries = retries_val;
            }
        }

        // Override logging if specified
        if let Ok(logging) = env::var("AI_ENABLE_LOGGING") {
            config.enable_logging = logging.to_lowercase() == "true";
        }

        // Override routing strategy if specified
        if let Ok(strategy) = env::var("AI_ROUTING_STRATEGY") {
            config.routing_strategy = strategy;
        }

        // Update provider configurations from environment
        config.providers = Self::providers_from_env();

        config
    }

    /// Get default provider configurations
    fn default_providers() -> HashMap<String, ProviderConfig> {
        let mut providers = HashMap::new();

        // OpenAI configuration
        providers.insert(
            "openai".to_string(),
            ProviderConfig {
                provider_type: "openai".to_string(),
                api_key: None, // Will be loaded from environment
                base_url: Some("https://api.openai.com/v1".to_string()),
                default_model: Some("gpt-3.5-turbo".to_string()),
                settings: HashMap::new(),
            },
        );

        // Anthropic configuration
        providers.insert(
            "anthropic".to_string(),
            ProviderConfig {
                provider_type: "anthropic".to_string(),
                api_key: None, // Will be loaded from environment
                base_url: Some("https://api.anthropic.com/v1".to_string()),
                default_model: Some("claude-3-sonnet-20240229".to_string()),
                settings: HashMap::new(),
            },
        );

        // Ollama configuration
        providers.insert(
            "ollama".to_string(),
            ProviderConfig {
                provider_type: "ollama".to_string(),
                api_key: None, // Ollama doesn't require API key
                base_url: Some("http://localhost:11434".to_string()),
                default_model: Some("llama2".to_string()),
                settings: HashMap::new(),
            },
        );

        providers
    }

    /// Load provider configurations from environment variables
    fn providers_from_env() -> HashMap<String, ProviderConfig> {
        let mut providers = Self::default_providers();

        // OpenAI configuration from environment
        if let Some(openai_config) = providers.get_mut("openai") {
            if let Ok(api_key) = env::var("OPENAI_API_KEY") {
                openai_config.api_key = Some(api_key);
            }
            if let Ok(base_url) = env::var("OPENAI_BASE_URL") {
                openai_config.base_url = Some(base_url);
            }
            if let Ok(model) = env::var("OPENAI_DEFAULT_MODEL") {
                openai_config.default_model = Some(model);
            }
        }

        // Anthropic configuration from environment
        if let Some(anthropic_config) = providers.get_mut("anthropic") {
            if let Ok(api_key) = env::var("ANTHROPIC_API_KEY") {
                anthropic_config.api_key = Some(api_key);
            }
            if let Ok(base_url) = env::var("ANTHROPIC_BASE_URL") {
                anthropic_config.base_url = Some(base_url);
            }
            if let Ok(model) = env::var("ANTHROPIC_DEFAULT_MODEL") {
                anthropic_config.default_model = Some(model);
            }
        }

        // Ollama configuration from environment
        if let Some(ollama_config) = providers.get_mut("ollama") {
            if let Ok(endpoint) = env::var("OLLAMA_ENDPOINT") {
                ollama_config.base_url = Some(endpoint);
            }
            if let Ok(model) = env::var("OLLAMA_DEFAULT_MODEL") {
                ollama_config.default_model = Some(model);
            }
        }

        providers
    }

    /// Get provider configuration by name
    pub fn get_provider(&self, name: &str) -> Option<&ProviderConfig> {
        self.providers.get(name)
    }

    /// Get the default provider configuration
    pub fn get_default_provider(&self) -> Option<&ProviderConfig> {
        self.providers.get(&self.default_provider)
    }

    /// Add or update a provider configuration
    pub fn add_provider(&mut self, name: String, config: ProviderConfig) {
        self.providers.insert(name, config);
    }

    /// Remove a provider configuration
    pub fn remove_provider(&mut self, name: &str) -> Option<ProviderConfig> {
        self.providers.remove(name)
    }

    /// List all configured providers
    pub fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        // Check if default provider exists
        if !self.providers.contains_key(&self.default_provider) {
            return Err(format!(
                "Default provider '{}' not found in configuration",
                self.default_provider
            ));
        }

        // Validate each provider configuration
        for (name, config) in &self.providers {
            if config.provider_type.is_empty() {
                return Err(format!("Provider '{name}' has empty provider_type"));
            }

            // Check if API key is required but missing
            if matches!(config.provider_type.as_str(), "openai" | "anthropic")
                && config.api_key.is_none()
            {
                return Err(format!("Provider '{name}' requires an API key"));
            }
        }

        Ok(())
    }
}

impl ProviderConfig {
    /// Create a new provider configuration
    pub fn new(provider_type: String) -> Self {
        Self {
            provider_type,
            api_key: None,
            base_url: None,
            default_model: None,
            settings: HashMap::new(),
        }
    }

    /// Set the API key for this provider
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// Set the base URL for this provider
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = Some(base_url);
        self
    }

    /// Set the default model for this provider
    pub fn with_default_model(mut self, model: String) -> Self {
        self.default_model = Some(model);
        self
    }

    /// Add a setting to this provider
    pub fn with_setting(mut self, key: String, value: serde_json::Value) -> Self {
        self.settings.insert(key, value);
        self
    }

    /// Get the API key for this provider
    pub fn get_api_key(&self) -> Option<&String> {
        self.api_key.as_ref()
    }

    /// Get the base URL for this provider
    pub fn get_base_url(&self) -> Option<&String> {
        self.base_url.as_ref()
    }

    /// Get the default model for this provider
    pub fn get_default_model(&self) -> Option<&String> {
        self.default_model.as_ref()
    }

    /// Get a setting value
    pub fn get_setting(&self, key: &str) -> Option<&serde_json::Value> {
        self.settings.get(key)
    }
}
