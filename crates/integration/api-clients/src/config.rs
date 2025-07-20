//! Configuration module for API clients
//!
//! This module provides configurable settings for all API clients to replace
//! hardcoded values and improve maintainability.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Default timeout values in seconds
pub const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 30;
pub const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 10;
pub const DEFAULT_READ_TIMEOUT_SECS: u64 = 60;

/// Default pagination settings
pub const DEFAULT_PER_PAGE: u32 = 30;
pub const DEFAULT_MAX_PER_PAGE: u32 = 100;

/// Default API base URLs
pub const OPENAI_BASE_URL: &str = "https://api.openai.com";
pub const ANTHROPIC_BASE_URL: &str = "https://api.anthropic.com";

/// Global API client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiClientConfig {
    /// HTTP request timeout
    pub request_timeout: Duration,
    /// HTTP connection timeout  
    pub connect_timeout: Duration,
    /// HTTP read timeout
    pub read_timeout: Duration,
    /// Maximum number of retries
    pub max_retries: u32,
    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
    /// Default pagination size
    pub default_per_page: u32,
    /// Maximum allowed pagination size
    pub max_per_page: u32,
}

impl Default for ApiClientConfig {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECS),
            connect_timeout: Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS),
            read_timeout: Duration::from_secs(DEFAULT_READ_TIMEOUT_SECS),
            max_retries: 3,
            retry_delay_ms: 1000,
            default_per_page: DEFAULT_PER_PAGE,
            max_per_page: DEFAULT_MAX_PER_PAGE,
        }
    }
}

/// OpenAI specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIConfig {
    /// Base URL for OpenAI API
    pub base_url: String,
    /// Global configuration
    pub client: ApiClientConfig,
    /// Model-specific timeout overrides
    pub model_timeouts: std::collections::HashMap<String, Duration>,
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        let mut model_timeouts = std::collections::HashMap::new();
        // GPT-4 tends to be slower
        model_timeouts.insert("gpt-4".to_string(), Duration::from_secs(60));
        model_timeouts.insert("gpt-4-turbo".to_string(), Duration::from_secs(45));

        Self {
            base_url: OPENAI_BASE_URL.to_string(),
            client: ApiClientConfig::default(),
            model_timeouts,
        }
    }
}

/// Anthropic specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicConfig {
    /// Base URL for Anthropic API
    pub base_url: String,
    /// Global configuration
    pub client: ApiClientConfig,
    /// Claude model timeout overrides
    pub model_timeouts: std::collections::HashMap<String, Duration>,
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        let mut model_timeouts = std::collections::HashMap::new();
        // Claude models timeout configuration
        model_timeouts.insert("claude-3-opus".to_string(), Duration::from_secs(90));
        model_timeouts.insert("claude-3-sonnet".to_string(), Duration::from_secs(60));
        model_timeouts.insert("claude-3-haiku".to_string(), Duration::from_secs(30));

        Self {
            base_url: ANTHROPIC_BASE_URL.to_string(),
            client: ApiClientConfig::default(),
            model_timeouts,
        }
    }
}

impl ApiClientConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(timeout) = std::env::var("API_REQUEST_TIMEOUT_SECS") {
            if let Ok(secs) = timeout.parse::<u64>() {
                config.request_timeout = Duration::from_secs(secs);
            }
        }

        if let Ok(timeout) = std::env::var("API_CONNECT_TIMEOUT_SECS") {
            if let Ok(secs) = timeout.parse::<u64>() {
                config.connect_timeout = Duration::from_secs(secs);
            }
        }

        if let Ok(timeout) = std::env::var("API_READ_TIMEOUT_SECS") {
            if let Ok(secs) = timeout.parse::<u64>() {
                config.read_timeout = Duration::from_secs(secs);
            }
        }

        if let Ok(retries) = std::env::var("API_MAX_RETRIES") {
            if let Ok(max_retries) = retries.parse::<u32>() {
                config.max_retries = max_retries;
            }
        }

        if let Ok(delay) = std::env::var("API_RETRY_DELAY_MS") {
            if let Ok(delay_ms) = delay.parse::<u64>() {
                config.retry_delay_ms = delay_ms;
            }
        }

        config
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<(), String> {
        if self.request_timeout.as_secs() == 0 {
            return Err("Request timeout cannot be zero".to_string());
        }

        if self.connect_timeout.as_secs() == 0 {
            return Err("Connect timeout cannot be zero".to_string());
        }

        if self.max_retries > 10 {
            return Err("Max retries cannot exceed 10".to_string());
        }

        if self.default_per_page == 0 || self.default_per_page > self.max_per_page {
            return Err("Invalid pagination configuration".to_string());
        }

        Ok(())
    }
}

impl OpenAIConfig {
    /// Create OpenAI configuration from environment variables
    pub fn from_env() -> Self {
        let base_url =
            std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| OPENAI_BASE_URL.to_string());

        Self {
            base_url,
            client: ApiClientConfig::from_env(),
            ..Self::default()
        }
    }

    /// Get timeout for a specific model, falling back to default
    pub fn timeout_for_model(&self, model: &str) -> Duration {
        self.model_timeouts
            .get(model)
            .cloned()
            .unwrap_or(self.client.request_timeout)
    }
}

impl AnthropicConfig {
    /// Create Anthropic configuration from environment variables
    pub fn from_env() -> Self {
        let base_url =
            std::env::var("ANTHROPIC_BASE_URL").unwrap_or_else(|_| ANTHROPIC_BASE_URL.to_string());

        Self {
            base_url,
            client: ApiClientConfig::from_env(),
            ..Self::default()
        }
    }

    /// Get timeout for a specific Claude model, falling back to default
    pub fn timeout_for_model(&self, model: &str) -> Duration {
        self.model_timeouts
            .get(model)
            .cloned()
            .unwrap_or(self.client.request_timeout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ApiClientConfig::default();
        assert_eq!(config.request_timeout, Duration::from_secs(30));
        assert_eq!(config.max_retries, 3);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_openai_config() {
        let config = OpenAIConfig::default();
        assert_eq!(config.base_url, OPENAI_BASE_URL);
        assert!(config.timeout_for_model("gpt-4") >= Duration::from_secs(60));
    }

    #[test]
    fn test_anthropic_config() {
        let config = AnthropicConfig::default();
        assert_eq!(config.base_url, ANTHROPIC_BASE_URL);
        assert!(config.timeout_for_model("claude-3-opus") >= Duration::from_secs(90));
    }

    #[test]
    fn test_config_validation() {
        let mut config = ApiClientConfig::default();
        assert!(config.validate().is_ok());

        config.request_timeout = Duration::from_secs(0);
        assert!(config.validate().is_err());

        config.request_timeout = Duration::from_secs(30);
        config.max_retries = 15;
        assert!(config.validate().is_err());
    }
}
