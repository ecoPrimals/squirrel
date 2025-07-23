//! Core configuration for AI tools
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
            providers: HashMap::new(),
            request_timeout: 30,
            max_retries: 3,
            enable_logging: true,
            routing_strategy: "round_robin".to_string(),
        }
    }
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            provider_type: "openai".to_string(),
            api_key: None,
            base_url: None,
            default_model: None,
            settings: HashMap::new(),
        }
    }
}

impl AIToolsConfig {
    /// Load configuration from environment variables and files
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let mut config = Self::default();

        // Try to load from SQUIRREL_AI_CONFIG environment variable
        if let Ok(config_path) = env::var("SQUIRREL_AI_CONFIG") {
            let config_str = std::fs::read_to_string(config_path)?;
            config = toml::from_str(&config_str)?;
        }

        // Override with environment variables
        if let Ok(default_provider) = env::var("SQUIRREL_DEFAULT_AI_PROVIDER") {
            config.default_provider = default_provider;
        }

        if let Ok(timeout) = env::var("SQUIRREL_AI_REQUEST_TIMEOUT") {
            config.request_timeout = timeout.parse()?;
        }

        if let Ok(retries) = env::var("SQUIRREL_AI_MAX_RETRIES") {
            config.max_retries = retries.parse()?;
        }

        if let Ok(logging) = env::var("SQUIRREL_AI_ENABLE_LOGGING") {
            config.enable_logging = logging.parse()?;
        }

        // Load provider-specific configurations
        config.load_provider_configs()?;

        Ok(config)
    }

    /// Load provider configurations from environment variables
    fn load_provider_configs(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // OpenAI configuration
        if let Ok(api_key) = env::var("OPENAI_API_KEY") {
            let provider_config = ProviderConfig {
                provider_type: "openai".to_string(),
                api_key: Some(api_key),
                base_url: env::var("OPENAI_BASE_URL").ok(),
                default_model: env::var("OPENAI_DEFAULT_MODEL").ok(),
                settings: HashMap::new(),
            };
            self.providers.insert("openai".to_string(), provider_config);
        }

        // Anthropic configuration
        if let Ok(api_key) = env::var("ANTHROPIC_API_KEY") {
            let provider_config = ProviderConfig {
                provider_type: "anthropic".to_string(),
                api_key: Some(api_key),
                base_url: env::var("ANTHROPIC_BASE_URL").ok(),
                default_model: env::var("ANTHROPIC_DEFAULT_MODEL").ok(),
                settings: HashMap::new(),
            };
            self.providers
                .insert("anthropic".to_string(), provider_config);
        }

        // Google/Gemini configuration
        if let Ok(api_key) = env::var("GEMINI_API_KEY") {
            let provider_config = ProviderConfig {
                provider_type: "gemini".to_string(),
                api_key: Some(api_key),
                base_url: env::var("GEMINI_BASE_URL").ok(),
                default_model: env::var("GEMINI_DEFAULT_MODEL").ok(),
                settings: HashMap::new(),
            };
            self.providers.insert("gemini".to_string(), provider_config);
        }

        // Ollama configuration
        if env::var("OLLAMA_HOST").is_ok() || env::var("OLLAMA_URL").is_ok() {
            let mut settings = HashMap::new();
            if let Ok(host) = env::var("OLLAMA_HOST") {
                settings.insert("host".to_string(), serde_json::Value::String(host));
            }

            let provider_config = ProviderConfig {
                provider_type: "ollama".to_string(),
                api_key: None,
                base_url: env::var("OLLAMA_URL").ok(),
                default_model: env::var("OLLAMA_DEFAULT_MODEL").ok(),
                settings,
            };
            self.providers.insert("ollama".to_string(), provider_config);
        }

        Ok(())
    }

    /// Get provider configuration by name
    pub fn get_provider(&self, name: &str) -> Option<&ProviderConfig> {
        self.providers.get(name)
    }

    /// Add or update a provider configuration
    pub fn add_provider(&mut self, name: String, config: ProviderConfig) {
        self.providers.insert(name, config);
    }

    /// Remove a provider configuration
    pub fn remove_provider(&mut self, name: &str) -> Option<ProviderConfig> {
        self.providers.remove(name)
    }

    /// Get all available provider names
    pub fn provider_names(&self) -> Vec<&String> {
        self.providers.keys().collect()
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.providers.is_empty() {
            return Err("No providers configured".to_string());
        }

        if !self.providers.contains_key(&self.default_provider) {
            return Err(format!(
                "Default provider '{}' not found in configured providers",
                self.default_provider
            ));
        }

        if self.request_timeout == 0 {
            return Err("Request timeout must be greater than 0".to_string());
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

    /// Set the API key
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// Set the base URL
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = Some(base_url);
        self
    }

    /// Set the default model
    pub fn with_default_model(mut self, model: String) -> Self {
        self.default_model = Some(model);
        self
    }

    /// Add a setting
    pub fn with_setting(mut self, key: String, value: serde_json::Value) -> Self {
        self.settings.insert(key, value);
        self
    }

    /// Get a setting value
    pub fn get_setting(&self, key: &str) -> Option<&serde_json::Value> {
        self.settings.get(key)
    }

    /// Check if this configuration is valid
    pub fn is_valid(&self) -> bool {
        match self.provider_type.as_str() {
            "openai" | "anthropic" | "gemini" => self.api_key.is_some(),
            "ollama" | "llamacpp" => true, // Local providers don't need API keys
            _ => false,                    // Unknown provider type
        }
    }
}
