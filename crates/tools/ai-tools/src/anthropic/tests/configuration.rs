//! Configuration and client creation tests for Anthropic API client
//!
//! **DEPRECATED**: These tests are for the deprecated Anthropic client.
//!
//! The Anthropic client uses reqwest (C dependency via ring) and is being replaced
//! by capability_ai which delegates HTTP to Songbird via Unix sockets.
//!
//! See: `docs/CAPABILITY_AI_MIGRATION_GUIDE.md` for the new pattern.
//!
//! These tests are kept for reference but will be removed in v2.0.0.

#![allow(deprecated)]

use super::super::{AnthropicClient, AnthropicConfig};
use crate::common::AIClient;

// ========== Configuration Tests ==========

#[test]
fn test_anthropic_config_default() {
    let config = AnthropicConfig::default();

    assert_eq!(config.default_model, "claude-3-opus-20240229");
    assert_eq!(config.api_base, "https://api.anthropic.com/v1");
    assert_eq!(config.rate_limit, 40);
    assert!(config.organization.is_none());
    assert_eq!(config.timeout_seconds, 60);
}

#[test]
fn test_anthropic_config_custom() {
    let config = AnthropicConfig {
        default_model: "claude-3-sonnet-20240229".to_string(),
        api_base: "https://custom-api.example.com".to_string(),
        rate_limit: 100,
        organization: Some("org-123".to_string()),
        timeout_seconds: 120,
    };

    assert_eq!(config.default_model, "claude-3-sonnet-20240229");
    assert_eq!(config.api_base, "https://custom-api.example.com");
    assert_eq!(config.rate_limit, 100);
    assert_eq!(config.organization, Some("org-123".to_string()));
    assert_eq!(config.timeout_seconds, 120);
}

#[test]
fn test_anthropic_config_clone() {
    let config = AnthropicConfig::default();
    let cloned = config.clone();

    assert_eq!(config.default_model, cloned.default_model);
    assert_eq!(config.api_base, cloned.api_base);
}

#[test]
fn test_anthropic_config_debug() {
    let config = AnthropicConfig::default();
    let debug_str = format!("{:?}", config);

    assert!(debug_str.contains("AnthropicConfig"));
    assert!(debug_str.contains("claude"));
}

// ========== Client Creation Tests ==========

#[test]
fn test_anthropic_client_new() {
    let client = AnthropicClient::new("test-api-key");

    // Client should be created successfully
    let debug_str = format!("{:?}", client);
    assert!(debug_str.contains("AnthropicClient"));
}

#[test]
fn test_anthropic_client_new_with_config() {
    let config = AnthropicConfig {
        default_model: "claude-3-haiku-20240307".to_string(),
        ..Default::default()
    };
    let client = AnthropicClient::with_config("test-api-key", config);

    let debug_str = format!("{:?}", client);
    assert!(debug_str.contains("AnthropicClient"));
}

#[test]
fn test_anthropic_client_debug() {
    let client = AnthropicClient::new("test-key");
    let debug_str = format!("{:?}", client);

    // Ensure sensitive data is not exposed in debug
    assert!(!debug_str.contains("test-key"));
    assert!(debug_str.contains("AnthropicClient"));
}

// ========== Public API Tests ==========

#[test]
fn test_anthropic_client_creation_variations() {
    // Test that we can create clients with different configs
    let _client1 = AnthropicClient::new("key1");
    let _client2 = AnthropicClient::with_config("key2", AnthropicConfig::default());
}

#[test]
fn test_anthropic_config_builder_pattern() {
    // Test configuration variations
    let config1 = AnthropicConfig::default();
    let config2 = AnthropicConfig {
        default_model: "custom-model".to_string(),
        ..config1
    };

    assert_eq!(config2.default_model, "custom-model");
}

// ========== Configuration Variations Tests ==========

#[test]
fn test_anthropic_config_with_organization() {
    let config = AnthropicConfig {
        organization: Some("test-org".to_string()),
        ..Default::default()
    };

    assert!(config.organization.is_some());
    assert_eq!(config.organization.unwrap(), "test-org");
}

#[test]
fn test_anthropic_config_without_organization() {
    let config = AnthropicConfig::default();

    assert!(config.organization.is_none());
}

#[test]
fn test_anthropic_config_various_timeouts() {
    let timeouts = vec![30, 60, 120, 300];

    for timeout in timeouts {
        let config = AnthropicConfig {
            timeout_seconds: timeout,
            ..Default::default()
        };
        assert_eq!(config.timeout_seconds, timeout);
    }
}

#[test]
fn test_anthropic_config_various_rate_limits() {
    let limits = vec![10, 40, 100, 1000];

    for limit in limits {
        let config = AnthropicConfig {
            rate_limit: limit,
            ..Default::default()
        };
        assert_eq!(config.rate_limit, limit);
    }
}

// ========== Model Variations Tests ==========

#[test]
fn test_anthropic_supported_models() {
    let models = vec![
        "claude-3-opus-20240229",
        "claude-3-sonnet-20240229",
        "claude-3-haiku-20240307",
    ];

    for model in models {
        let config = AnthropicConfig {
            default_model: model.to_string(),
            ..Default::default()
        };
        assert_eq!(config.default_model, model);
    }
}

// ========== Edge Cases Tests ==========

#[test]
fn test_anthropic_config_empty_strings() {
    let config = AnthropicConfig {
        default_model: "".to_string(),
        api_base: "".to_string(),
        rate_limit: 0,
        organization: Some("".to_string()),
        timeout_seconds: 0,
    };

    assert_eq!(config.default_model, "");
    assert_eq!(config.api_base, "");
    assert_eq!(config.rate_limit, 0);
}

#[test]
fn test_anthropic_multiple_clients() {
    // Test that we can create multiple independent clients
    let clients: Vec<_> = (0..3)
        .map(|i| {
            let key = format!("key-{}", i);
            AnthropicClient::new(key)
        })
        .collect();

    assert_eq!(clients.len(), 3);
}

// ========== Client Interface Tests ==========

#[test]
fn test_anthropic_client_provider_name() {
    let client = AnthropicClient::new("test-key");
    assert_eq!(client.provider_name(), "anthropic");
}

#[test]
fn test_anthropic_client_default_model() {
    let config = AnthropicConfig {
        default_model: "claude-3-haiku-20240307".to_string(),
        ..Default::default()
    };
    let client = AnthropicClient::with_config("test-key", config);
    assert_eq!(client.default_model(), "claude-3-haiku-20240307");
}

#[tokio::test]
async fn test_anthropic_list_models() {
    let client = AnthropicClient::new("test-key");
    let models = client.list_models().await.unwrap();

    // Should return at least the core Claude 3 models
    assert!(!models.is_empty());
    assert!(models.iter().any(|m| m.contains("claude-3")));
}

#[test]
fn test_anthropic_client_capabilities() {
    let client = AnthropicClient::new("test-key");
    let caps = client.capabilities();

    assert!(caps.supports_streaming);
}

// ========== Additional Config Edge Cases ==========

#[test]
fn test_anthropic_config_with_endpoint() {
    let config = AnthropicConfig {
        api_base: "https://custom.example.com/v1".to_string(),
        ..Default::default()
    };

    assert_eq!(config.api_base, "https://custom.example.com/v1");
}

#[test]
fn test_anthropic_config_very_high_rate_limit() {
    let config = AnthropicConfig {
        rate_limit: 10000,
        ..Default::default()
    };

    assert_eq!(config.rate_limit, 10000);
}

#[test]
fn test_anthropic_config_very_long_timeout() {
    let config = AnthropicConfig {
        timeout_seconds: 600,
        ..Default::default()
    };

    assert_eq!(config.timeout_seconds, 600);
}

#[test]
fn test_anthropic_config_custom_model_name() {
    let config = AnthropicConfig {
        default_model: "claude-4-future-model".to_string(),
        ..Default::default()
    };

    assert_eq!(config.default_model, "claude-4-future-model");
}
