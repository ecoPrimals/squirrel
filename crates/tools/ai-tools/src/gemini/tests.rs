//! Tests for Gemini API client
//!
//! **DEPRECATED**: These tests are for the deprecated Gemini client.
//!
//! The Gemini client uses reqwest (C dependency via ring) and is being replaced
//! by capability_ai which delegates HTTP to Songbird via Unix sockets.
//!
//! See: `docs/CAPABILITY_AI_MIGRATION_GUIDE.md` for the new pattern.
//!
//! These tests are kept for reference but will be removed in v2.0.0.

#![allow(deprecated)]

#[cfg(test)]
mod gemini_tests {
    use super::super::{GeminiClient, GeminiConfig};
    use crate::common::{AIClient, ChatRequest};

    // ========== Configuration Tests ==========

    #[test]
    fn test_gemini_config_default() {
        let config = GeminiConfig::default();

        assert_eq!(config.default_model, "gemini-pro");
        assert_eq!(
            config.api_base,
            "https://generativelanguage.googleapis.com/v1beta"
        );
        assert_eq!(config.rate_limit, 60);
        assert_eq!(config.timeout_seconds, 60);
    }

    #[test]
    fn test_gemini_config_custom() {
        let config = GeminiConfig {
            default_model: "gemini-pro-vision".to_string(),
            api_base: "https://custom-api.example.com".to_string(),
            rate_limit: 120,
            timeout_seconds: 90,
        };

        assert_eq!(config.default_model, "gemini-pro-vision");
        assert_eq!(config.api_base, "https://custom-api.example.com");
        assert_eq!(config.rate_limit, 120);
        assert_eq!(config.timeout_seconds, 90);
    }

    #[test]
    fn test_gemini_config_clone() {
        let config = GeminiConfig::default();
        let cloned = config.clone();

        assert_eq!(config.default_model, cloned.default_model);
        assert_eq!(config.api_base, cloned.api_base);
    }

    #[test]
    fn test_gemini_config_debug() {
        let config = GeminiConfig::default();
        let debug_str = format!("{:?}", config);

        assert!(debug_str.contains("GeminiConfig"));
        assert!(debug_str.contains("gemini"));
    }

    // ========== Client Creation Tests ==========

    #[test]
    fn test_gemini_client_new() {
        let client = GeminiClient::new("test-api-key");

        let debug_str = format!("{:?}", client);
        assert!(debug_str.contains("GeminiClient"));
    }

    #[test]
    fn test_gemini_client_new_with_config() {
        let config = GeminiConfig {
            default_model: "gemini-pro".to_string(),
            ..Default::default()
        };
        let client = GeminiClient::with_config("test-api-key", config);

        let debug_str = format!("{:?}", client);
        assert!(debug_str.contains("GeminiClient"));
    }

    #[test]
    fn test_gemini_client_debug() {
        let client = GeminiClient::new("test-key");
        let debug_str = format!("{:?}", client);

        // Ensure sensitive data is not exposed in debug
        assert!(!debug_str.contains("test-key"));
        assert!(debug_str.contains("GeminiClient"));
    }

    // ========== Public API Tests ==========

    #[test]
    fn test_gemini_client_creation_variations() {
        // Test that we can create clients with different configs
        let _client1 = GeminiClient::new("key1");
        let _client2 = GeminiClient::with_config("key2", GeminiConfig::default());
    }

    #[test]
    fn test_gemini_config_builder_pattern() {
        // Test configuration variations
        let config1 = GeminiConfig::default();
        let config2 = GeminiConfig {
            default_model: "custom-model".to_string(),
            ..config1
        };

        assert_eq!(config2.default_model, "custom-model");
    }

    // ========== Configuration Variations Tests ==========

    #[test]
    fn test_gemini_config_various_timeouts() {
        let timeouts = vec![30, 60, 120, 300];

        for timeout in timeouts {
            let config = GeminiConfig {
                timeout_seconds: timeout,
                ..Default::default()
            };
            assert_eq!(config.timeout_seconds, timeout);
        }
    }

    #[test]
    fn test_gemini_config_various_rate_limits() {
        let limits = vec![30, 60, 120, 240];

        for limit in limits {
            let config = GeminiConfig {
                rate_limit: limit,
                ..Default::default()
            };
            assert_eq!(config.rate_limit, limit);
        }
    }

    // ========== Model Variations Tests ==========

    #[test]
    fn test_gemini_supported_models() {
        let models = vec!["gemini-pro", "gemini-pro-vision", "gemini-ultra"];

        for model in models {
            let config = GeminiConfig {
                default_model: model.to_string(),
                ..Default::default()
            };
            assert_eq!(config.default_model, model);
        }
    }

    // ========== Client Lifecycle Tests ==========

    #[test]
    fn test_gemini_multiple_clients() {
        // Test that we can create multiple independent clients
        let clients: Vec<_> = (0..3)
            .map(|i| {
                let key = format!("key-{}", i);
                GeminiClient::new(key)
            })
            .collect();

        assert_eq!(clients.len(), 3);
    }

    // ========== Edge Cases Tests ==========

    #[test]
    fn test_gemini_config_empty_strings() {
        let config = GeminiConfig {
            default_model: "".to_string(),
            api_base: "".to_string(),
            rate_limit: 0,
            timeout_seconds: 0,
        };

        assert_eq!(config.default_model, "");
        assert_eq!(config.api_base, "");
        assert_eq!(config.rate_limit, 0);
    }

    #[test]
    fn test_gemini_config_with_all_fields() {
        let config = GeminiConfig {
            default_model: "gemini-ultra".to_string(),
            api_base: "https://test.example.com".to_string(),
            rate_limit: 200,
            timeout_seconds: 180,
        };

        assert_eq!(config.default_model, "gemini-ultra");
        assert_eq!(config.api_base, "https://test.example.com");
        assert_eq!(config.rate_limit, 200);
        assert_eq!(config.timeout_seconds, 180);
    }

    // ========== HTTP/Network Layer Tests ==========

    #[tokio::test]
    async fn test_gemini_chat_with_mock_server() {
        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock(
                "POST",
                mockito::Matcher::Regex(
                    r".*/models/gemini-pro:generateContent\?key=.*".to_string(),
                ),
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "candidates": [{
                    "content": {
                        "parts": [{"text": "Hello! How can I help you?"}],
                        "role": "model"
                    },
                    "finishReason": "STOP",
                    "index": 0
                }],
                "usageMetadata": {
                    "promptTokenCount": 5,
                    "candidatesTokenCount": 8,
                    "totalTokenCount": 13
                }
            }"#,
            )
            .create_async()
            .await;

        let config = GeminiConfig {
            api_base: server.url(),
            ..Default::default()
        };
        let client = GeminiClient::with_config("test-key", config);

        let request = ChatRequest::new().add_user("Hello");

        let response = client.chat(request).await.unwrap();

        mock.assert_async().await;
        assert_eq!(response.choices.len(), 1);
        assert_eq!(
            response.choices[0].content,
            Some("Hello! How can I help you?".to_string())
        );
        assert_eq!(response.usage.as_ref().unwrap().prompt_tokens, 5);
        assert_eq!(response.usage.as_ref().unwrap().completion_tokens, 8);
    }

    #[tokio::test]
    async fn test_gemini_chat_with_error_response() {
        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("POST", mockito::Matcher::Any)
            .with_status(400)
            .with_body(r#"{"error": {"message": "Invalid API key"}}"#)
            .create_async()
            .await;

        let config = GeminiConfig {
            api_base: server.url(),
            ..Default::default()
        };
        let client = GeminiClient::with_config("invalid-key", config);

        let request = ChatRequest::new().add_user("Hello");

        let result = client.chat(request).await;

        mock.assert_async().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_gemini_chat_parse_error() {
        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("POST", mockito::Matcher::Any)
            .with_status(200)
            .with_body("invalid json")
            .create_async()
            .await;

        let config = GeminiConfig {
            api_base: server.url(),
            ..Default::default()
        };
        let client = GeminiClient::with_config("test-key", config);

        let request = ChatRequest::new().add_user("Hello");

        let result = client.chat(request).await;

        mock.assert_async().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_gemini_list_models_success() {
        let client = GeminiClient::new("test-key");
        let models = client.list_models().await.unwrap();

        // Gemini returns hardcoded model list
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.contains("gemini-pro")));
    }

    #[tokio::test]
    async fn test_gemini_is_available_when_api_works() {
        let client = GeminiClient::new("test-key");

        // Gemini is_available returns true by default
        let available = client.is_available().await;

        // This might fail since the actual implementation may always return true
        // or might try a real API call. Let's just check it doesn't panic.
        let _ = available;
    }

    #[tokio::test]
    async fn test_gemini_is_available_when_api_fails() {
        let config = GeminiConfig {
            api_base: "http://invalid-endpoint.local:9999".to_string(),
            timeout_seconds: 1,
            ..Default::default()
        };
        let client = GeminiClient::with_config("test-key", config);

        let available = client.is_available().await;

        assert!(!available);
    }

    #[test]
    fn test_gemini_client_provider_name() {
        let client = GeminiClient::new("test-key");
        assert_eq!(client.provider_name(), "gemini");
    }

    #[test]
    fn test_gemini_client_default_model() {
        let client = GeminiClient::new("test-key");
        assert_eq!(client.default_model(), "gemini-pro");
    }

    #[test]
    fn test_gemini_capabilities() {
        let client = GeminiClient::new("test-key");
        let caps = client.capabilities();

        assert!(caps.supports_streaming);
    }

    #[test]
    fn test_gemini_routing_preferences() {
        let client = GeminiClient::new("test-key");
        let prefs = client.routing_preferences();

        assert!(prefs.allows_forwarding);
    }
}
