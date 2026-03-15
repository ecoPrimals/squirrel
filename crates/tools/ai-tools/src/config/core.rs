// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

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
    /// Provider type (openai, anthropic, local-server, etc.)
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

        // Local AI server configuration (agnostic: works with Ollama, llama.cpp, vLLM, etc.)
        // Checks agnostic env vars first, then falls back to legacy vendor-specific ones
        if env::var("LOCAL_AI_HOST").is_ok()
            || env::var("LOCAL_AI_URL").is_ok()
            || env::var("OLLAMA_HOST").is_ok()
            || env::var("OLLAMA_URL").is_ok()
        {
            let mut settings = HashMap::new();
            if let Ok(host) = env::var("LOCAL_AI_HOST").or_else(|_| env::var("OLLAMA_HOST")) {
                settings.insert("host".to_string(), serde_json::Value::String(host));
            }

            let provider_config = ProviderConfig {
                provider_type: "local-server".to_string(),
                api_key: None,
                base_url: env::var("LOCAL_AI_URL")
                    .or_else(|_| env::var("OLLAMA_URL"))
                    .ok(),
                default_model: env::var("LOCAL_AI_DEFAULT_MODEL")
                    .or_else(|_| env::var("OLLAMA_DEFAULT_MODEL"))
                    .ok(),
                settings,
            };
            self.providers.insert("local".to_string(), provider_config);
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
            "local-server" | "local" | "native" => true, // Local providers don't need API keys
            // Legacy names still valid for backward compat
            "ollama" | "llamacpp" => true,
            _ => false, // Unknown provider type
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_tools_config_default() {
        let config = AIToolsConfig::default();
        assert_eq!(config.default_provider, "openai");
        assert!(config.providers.is_empty());
        assert_eq!(config.request_timeout, 30);
        assert_eq!(config.max_retries, 3);
        assert!(config.enable_logging);
        assert_eq!(config.routing_strategy, "round_robin");
    }

    #[test]
    fn test_provider_config_default() {
        let config = ProviderConfig::default();
        assert_eq!(config.provider_type, "openai");
        assert!(config.api_key.is_none());
        assert!(config.base_url.is_none());
        assert!(config.default_model.is_none());
        assert!(config.settings.is_empty());
    }

    #[test]
    fn test_provider_config_new() {
        let config = ProviderConfig::new("anthropic".to_string());
        assert_eq!(config.provider_type, "anthropic");
        assert!(config.api_key.is_none());
    }

    #[test]
    fn test_provider_config_builder() {
        let config = ProviderConfig::new("openai".to_string())
            .with_api_key("sk-test".to_string())
            .with_base_url("https://api.openai.com".to_string())
            .with_default_model("gpt-4".to_string())
            .with_setting("timeout".to_string(), serde_json::json!(30));

        assert_eq!(config.provider_type, "openai");
        assert_eq!(config.api_key.as_deref(), Some("sk-test"));
        assert_eq!(config.base_url.as_deref(), Some("https://api.openai.com"));
        assert_eq!(config.default_model.as_deref(), Some("gpt-4"));
        assert_eq!(config.get_setting("timeout"), Some(&serde_json::json!(30)));
        assert!(config.get_setting("nonexistent").is_none());
    }

    #[test]
    fn test_provider_config_is_valid_cloud_providers() {
        // Cloud providers need API keys
        let without_key = ProviderConfig::new("openai".to_string());
        assert!(!without_key.is_valid());

        let with_key =
            ProviderConfig::new("openai".to_string()).with_api_key("sk-test".to_string());
        assert!(with_key.is_valid());

        let anthropic_no_key = ProviderConfig::new("anthropic".to_string());
        assert!(!anthropic_no_key.is_valid());

        let gemini_with_key =
            ProviderConfig::new("gemini".to_string()).with_api_key("key".to_string());
        assert!(gemini_with_key.is_valid());
    }

    #[test]
    fn test_provider_config_is_valid_local_providers() {
        // Local providers don't need API keys
        assert!(ProviderConfig::new("local-server".to_string()).is_valid());
        assert!(ProviderConfig::new("local".to_string()).is_valid());
        assert!(ProviderConfig::new("native".to_string()).is_valid());
        assert!(ProviderConfig::new("ollama".to_string()).is_valid());
        assert!(ProviderConfig::new("llamacpp".to_string()).is_valid());
    }

    #[test]
    fn test_provider_config_is_valid_unknown() {
        let unknown = ProviderConfig::new("unknown-provider".to_string());
        assert!(!unknown.is_valid());
    }

    #[test]
    fn test_ai_tools_config_add_and_get_provider() {
        let mut config = AIToolsConfig::default();
        let provider =
            ProviderConfig::new("openai".to_string()).with_api_key("sk-test".to_string());

        config.add_provider("openai".to_string(), provider);
        assert!(config.get_provider("openai").is_some());
        assert!(config.get_provider("nonexistent").is_none());
    }

    #[test]
    fn test_ai_tools_config_remove_provider() {
        let mut config = AIToolsConfig::default();
        config.add_provider(
            "openai".to_string(),
            ProviderConfig::new("openai".to_string()),
        );

        let removed = config.remove_provider("openai");
        assert!(removed.is_some());
        assert!(config.get_provider("openai").is_none());

        let not_found = config.remove_provider("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_ai_tools_config_provider_names() {
        let mut config = AIToolsConfig::default();
        config.add_provider(
            "openai".to_string(),
            ProviderConfig::new("openai".to_string()),
        );
        config.add_provider(
            "anthropic".to_string(),
            ProviderConfig::new("anthropic".to_string()),
        );

        let names = config.provider_names();
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn test_ai_tools_config_validate_no_providers() {
        let config = AIToolsConfig::default();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No providers configured"));
    }

    #[test]
    fn test_ai_tools_config_validate_missing_default() {
        let mut config = AIToolsConfig::default();
        config.add_provider(
            "anthropic".to_string(),
            ProviderConfig::new("anthropic".to_string()),
        );
        // default_provider is "openai" but only "anthropic" is configured
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Default provider"));
    }

    #[test]
    fn test_ai_tools_config_validate_zero_timeout() {
        let mut config = AIToolsConfig::default();
        config.add_provider(
            "openai".to_string(),
            ProviderConfig::new("openai".to_string()),
        );
        config.request_timeout = 0;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("timeout"));
    }

    #[test]
    fn test_ai_tools_config_validate_success() {
        let mut config = AIToolsConfig::default();
        config.add_provider(
            "openai".to_string(),
            ProviderConfig::new("openai".to_string()),
        );
        let result = config.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_ai_tools_config_serde() {
        let mut config = AIToolsConfig::default();
        config.add_provider(
            "openai".to_string(),
            ProviderConfig::new("openai".to_string()).with_api_key("sk-test".to_string()),
        );

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AIToolsConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.default_provider, config.default_provider);
        assert_eq!(deserialized.request_timeout, config.request_timeout);
        assert!(deserialized.get_provider("openai").is_some());
    }

    #[test]
    fn test_provider_config_serde() {
        let config = ProviderConfig::new("anthropic".to_string())
            .with_api_key("key-123".to_string())
            .with_base_url("https://api.anthropic.com".to_string())
            .with_default_model("claude-3-opus".to_string());

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ProviderConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.provider_type, "anthropic");
        assert_eq!(deserialized.api_key.as_deref(), Some("key-123"));
        assert_eq!(
            deserialized.base_url.as_deref(),
            Some("https://api.anthropic.com")
        );
        assert_eq!(deserialized.default_model.as_deref(), Some("claude-3-opus"));
    }

    #[test]
    fn test_ai_tools_config_from_env_defaults() {
        // Ensure env vars don't interfere - clean state
        unsafe { std::env::remove_var("SQUIRREL_AI_CONFIG") };
        unsafe { std::env::remove_var("SQUIRREL_DEFAULT_AI_PROVIDER") };
        unsafe { std::env::remove_var("SQUIRREL_AI_REQUEST_TIMEOUT") };
        unsafe { std::env::remove_var("SQUIRREL_AI_MAX_RETRIES") };
        unsafe { std::env::remove_var("SQUIRREL_AI_ENABLE_LOGGING") };
        unsafe { std::env::remove_var("OPENAI_API_KEY") };
        unsafe { std::env::remove_var("ANTHROPIC_API_KEY") };
        unsafe { std::env::remove_var("GEMINI_API_KEY") };
        unsafe { std::env::remove_var("LOCAL_AI_HOST") };
        unsafe { std::env::remove_var("LOCAL_AI_URL") };
        unsafe { std::env::remove_var("OLLAMA_HOST") };
        unsafe { std::env::remove_var("OLLAMA_URL") };

        let config = AIToolsConfig::from_env().unwrap();
        assert_eq!(config.default_provider, "openai");
        assert_eq!(config.request_timeout, 30);
        assert_eq!(config.max_retries, 3);
        assert!(config.enable_logging);
    }
}
