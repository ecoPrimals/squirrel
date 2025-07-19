//! AI configuration types for Squirrel MCP
//!
//! This module defines AI provider configuration, rate limiting,
//! and AI service management structures.

use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;
use url::Url;

/// AI configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AIConfig {
    pub providers: Vec<AIProvider>,
    pub default_provider: String,
    pub max_retries: u32,
    pub timeout: Duration,
    pub fallback_enabled: bool,
    pub health_check_interval: Duration,
}

/// AI provider configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AIProvider {
    pub name: String,
    pub provider_type: AIProviderType,
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub priority: u32,
    pub enabled: bool,
    pub rate_limit: RateLimit,
}

/// AI provider types
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AIProviderType {
    #[serde(rename = "openai")]
    OpenAI,
    #[serde(rename = "anthropic")]
    Anthropic,
    #[serde(rename = "ollama")]
    Ollama,
    #[serde(rename = "azure")]
    Azure,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub tokens_per_minute: u32,
    pub burst_limit: u32,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            providers: vec![
                AIProvider {
                    name: "openai".to_string(),
                    provider_type: AIProviderType::OpenAI,
                    endpoint: Self::parse_endpoint_safely("https://api.openai.com/v1", "OpenAI"),
                    api_key: env::var("OPENAI_API_KEY").unwrap_or_default(),
                    model: "gpt-4".to_string(),
                    max_tokens: 4096,
                    temperature: 0.7,
                    priority: 1,
                    enabled: true,
                    rate_limit: RateLimit {
                        requests_per_minute: 60,
                        tokens_per_minute: 200000,
                        burst_limit: 10,
                    },
                },
                AIProvider {
                    name: "anthropic".to_string(),
                    provider_type: AIProviderType::Anthropic,
                    endpoint: Self::parse_endpoint_safely(
                        "https://api.anthropic.com/v1",
                        "Anthropic",
                    ),
                    api_key: env::var("ANTHROPIC_API_KEY").unwrap_or_default(),
                    model: "claude-3-sonnet-20240229".to_string(),
                    max_tokens: 4096,
                    temperature: 0.7,
                    priority: 2,
                    enabled: true,
                    rate_limit: RateLimit {
                        requests_per_minute: 60,
                        tokens_per_minute: 200000,
                        burst_limit: 10,
                    },
                },
                AIProvider {
                    name: "ollama".to_string(),
                    provider_type: AIProviderType::Ollama,
                    endpoint: Self::parse_endpoint_safely("http://localhost:11434", "Ollama"),
                    api_key: String::new(),
                    model: "llama3.2".to_string(),
                    max_tokens: 4096,
                    temperature: 0.7,
                    priority: 3,
                    enabled: true,
                    rate_limit: RateLimit {
                        requests_per_minute: 120,
                        tokens_per_minute: 400000,
                        burst_limit: 20,
                    },
                },
            ],
            default_provider: "openai".to_string(),
            max_retries: 3,
            timeout: Duration::from_secs(60),
            fallback_enabled: true,
            health_check_interval: Duration::from_secs(60),
        }
    }
}

impl AIConfig {
    /// Safely parse endpoint URLs with proper error handling - guaranteed to never panic
    fn parse_endpoint_safely(url: &str, provider_name: &str) -> String {
        // Try the requested URL first
        if let Ok(parsed) = url.parse::<Url>() {
            return parsed.to_string();
        }

        tracing::warn!(
            "Failed to parse {} endpoint URL '{}'. Trying fallbacks.",
            provider_name,
            url
        );

        // Try known-good fallback URLs
        let fallbacks = [
            "http://localhost:8080",
            "http://127.0.0.1:8080",
            "http://localhost",
            "http://127.0.0.1",
            "http://disabled",
            "http://error",
        ];

        for fallback in &fallbacks {
            if let Ok(parsed) = fallback.parse::<Url>() {
                tracing::warn!("Using fallback URL '{}' for {}", fallback, provider_name);
                return parsed.to_string();
            }
        }

        // PRODUCTION SAFE: If all known fallbacks fail, create a URL that represents a disabled provider
        // This prevents crashes while maintaining system stability
        tracing::error!(
            "CRITICAL: All URL parsing failed for {}. Provider will be disabled.",
            provider_name
        );

        // Create a disabled URL that will cause the provider to be skipped
        // This approach is safe and prevents application crashes
        Self::create_disabled_url(provider_name)
    }

    /// Creates a URL that represents a disabled provider
    /// This is a helper function that uses safe URL creation patterns
    fn create_disabled_url(provider_name: &str) -> String {
        // Try multiple strategies to create a URL that represents a disabled provider
        // This URL will cause the provider to be skipped during initialization

        // Strategy 1: Create a provider-specific disabled URL
        let clean_name = provider_name
            .chars()
            .filter(|c| c.is_alphanumeric())
            .take(10)
            .collect::<String>();

        if !clean_name.is_empty() {
            let disabled_url = format!("http://disabled-{clean_name}");
            if let Ok(url) = disabled_url.parse::<Url>() {
                return url.to_string();
            }
        }

        // Strategy 2: Use a generic disabled URL
        if let Ok(url) = "http://disabled".parse::<Url>() {
            return url.to_string();
        }

        // Strategy 3: Use localhost as a safe fallback
        if let Ok(url) = "http://localhost".parse::<Url>() {
            return url.to_string();
        }

        // Strategy 4: Use IP address as fallback
        if let Ok(url) = "http://127.0.0.1".parse::<Url>() {
            return url.to_string();
        }

        // Strategy 5: Use the most basic HTTP URL
        if let Ok(url) = "http://error".parse::<Url>() {
            return url.to_string();
        }

        // FINAL FALLBACK: If even basic URLs fail, this indicates a fundamental system issue
        // Log the critical error and exit the process safely rather than panic
        tracing::error!(
            "SYSTEM CRITICAL: Cannot create any URL for {}. URL crate appears broken.",
            provider_name
        );
        tracing::error!(
            "This indicates a fundamental system failure. Exiting to prevent undefined behavior."
        );

        // Exit the process cleanly rather than panic or use dangerous patterns
        std::process::exit(1);
    }
}
