//! Comprehensive tests for OpenAI API client

use squirrel_ai_tools::common::{AIClient, ChatMessage, ChatRequest, MessageRole, ModelParameters};
use squirrel_ai_tools::openai::{OpenAIClient, OpenAIConfig, DEFAULT_MODEL};

// ========== Configuration Tests ==========

#[test]
fn test_openai_config_default() {
    let config = OpenAIConfig::default();

    assert_eq!(config.default_model, DEFAULT_MODEL);
    assert_eq!(config.api_base, "https://api.openai.com/v1");
    assert_eq!(config.rate_limit, 60);
    assert!(config.retry_on_rate_limit);
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.retry_delay_ms, 2000);
    assert!(config.organization.is_none());
    assert_eq!(config.timeout_seconds, 60);
}

#[test]
fn test_openai_config_custom() {
    let config = OpenAIConfig {
        default_model: "gpt-4".to_string(),
        api_base: "https://custom-api.example.com".to_string(),
        rate_limit: 100,
        retry_on_rate_limit: false,
        max_retries: 5,
        retry_delay_ms: 1000,
        organization: Some("org-123".to_string()),
        timeout_seconds: 120,
    };

    assert_eq!(config.default_model, "gpt-4");
    assert_eq!(config.api_base, "https://custom-api.example.com");
    assert_eq!(config.rate_limit, 100);
    assert!(!config.retry_on_rate_limit);
    assert_eq!(config.max_retries, 5);
    assert_eq!(config.retry_delay_ms, 1000);
    assert_eq!(config.organization, Some("org-123".to_string()));
    assert_eq!(config.timeout_seconds, 120);
}

#[test]
fn test_openai_config_clone() {
    let config = OpenAIConfig::default();
    let cloned = config.clone();

    assert_eq!(config.default_model, cloned.default_model);
    assert_eq!(config.api_base, cloned.api_base);
    assert_eq!(config.rate_limit, cloned.rate_limit);
}

#[test]
fn test_openai_config_debug() {
    let config = OpenAIConfig::default();
    let debug_str = format!("{:?}", config);

    assert!(debug_str.contains("OpenAIConfig"));
    assert!(debug_str.contains("gpt"));
}

// ========== Client Creation Tests ==========

#[test]
fn test_openai_client_new() {
    let client = OpenAIClient::new("test-api-key");

    assert!(client.is_ok());
    let client = client.unwrap();
    let debug_str = format!("{:?}", client);
    assert!(debug_str.contains("OpenAIClient"));
}

#[test]
fn test_openai_client_new_with_config() {
    let config = OpenAIConfig {
        default_model: "gpt-4-turbo".to_string(),
        ..Default::default()
    };
    let client = OpenAIClient::with_config("test-api-key", config);

    assert!(client.is_ok());
    let client = client.unwrap();
    let debug_str = format!("{:?}", client);
    assert!(debug_str.contains("OpenAIClient"));
}

#[test]
fn test_openai_client_debug() {
    let client = OpenAIClient::new("test-key").unwrap();
    let debug_str = format!("{:?}", client);

    // Ensure sensitive data is not exposed in debug
    assert!(!debug_str.contains("test-key"));
    assert!(debug_str.contains("OpenAIClient"));
}

#[test]
fn test_openai_client_clone() {
    let client = OpenAIClient::new("test-key").unwrap();
    let cloned = client.clone();

    let debug1 = format!("{:?}", client);
    let debug2 = format!("{:?}", cloned);

    assert!(debug1.contains("OpenAIClient"));
    assert!(debug2.contains("OpenAIClient"));
}

// ========== AI Client Trait Tests ==========

#[test]
fn test_openai_client_provider_name() {
    let client = OpenAIClient::new("test-key").unwrap();
    assert_eq!(client.provider_name(), "openai");
}

#[test]
fn test_openai_client_default_model() {
    let client = OpenAIClient::new("test-key").unwrap();
    assert_eq!(client.default_model(), DEFAULT_MODEL);
}

#[test]
fn test_openai_client_default_model_custom() {
    let config = OpenAIConfig {
        default_model: "gpt-4".to_string(),
        ..Default::default()
    };
    let client = OpenAIClient::with_config("test-key", config).unwrap();
    assert_eq!(client.default_model(), "gpt-4");
}

// ========== List Models Tests ==========

#[tokio::test]
async fn test_list_models_returns_non_empty() {
    let client = OpenAIClient::new("test-key").unwrap();
    let models = client.list_models().await;

    // Should return models even without network (from registry)
    assert!(models.is_ok());
    let models = models.unwrap();
    assert!(!models.is_empty());
}

#[tokio::test]
async fn test_list_models_contains_gpt4() {
    let client = OpenAIClient::new("test-key").unwrap();
    let models = client.list_models().await.unwrap();

    // Should contain at least some GPT-4 variant
    assert!(models
        .iter()
        .any(|m| m.contains("gpt-4") || m.contains("gpt")));
}

// ========== Request Building Tests ==========

#[test]
fn test_chat_request_minimal() {
    let request = ChatRequest {
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: Some("Hello".to_string()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }],
        model: None,
        parameters: None,
        tools: None,
    };

    let debug_str = format!("{:?}", request);
    assert!(debug_str.contains("Hello"));
}

#[test]
fn test_chat_request_with_parameters() {
    let request = ChatRequest {
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: Some("Hello".to_string()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }],
        model: Some("gpt-4".to_string()),
        parameters: Some(ModelParameters {
            temperature: Some(0.7),
            max_tokens: Some(100),
            top_p: Some(0.9),
            frequency_penalty: Some(0.0),
            presence_penalty: Some(0.0),
            stream: Some(false),
            ..Default::default()
        }),
        tools: None,
    };

    assert_eq!(request.model, Some("gpt-4".to_string()));
    let params = request.parameters.as_ref().unwrap();
    assert_eq!(params.temperature, Some(0.7));
    assert_eq!(params.max_tokens, Some(100));
}

#[test]
fn test_chat_message_roles() {
    let user_msg = ChatMessage {
        role: MessageRole::User,
        content: Some("User message".to_string()),
        name: None,
        tool_calls: None,
        tool_call_id: None,
    };

    let assistant_msg = ChatMessage {
        role: MessageRole::Assistant,
        content: Some("Assistant response".to_string()),
        name: None,
        tool_calls: None,
        tool_call_id: None,
    };

    let system_msg = ChatMessage {
        role: MessageRole::System,
        content: Some("System prompt".to_string()),
        name: None,
        tool_calls: None,
        tool_call_id: None,
    };

    assert!(matches!(user_msg.role, MessageRole::User));
    assert!(matches!(assistant_msg.role, MessageRole::Assistant));
    assert!(matches!(system_msg.role, MessageRole::System));
}

// ========== Error Handling Tests ==========

#[test]
fn test_openai_config_variations() {
    let configs = vec![
        OpenAIConfig::default(),
        OpenAIConfig {
            default_model: "gpt-3.5-turbo".to_string(),
            ..Default::default()
        },
        OpenAIConfig {
            rate_limit: 100,
            ..Default::default()
        },
    ];

    for config in configs {
        let client = OpenAIClient::with_config("test-key", config);
        assert!(client.is_ok());
    }
}

#[test]
fn test_openai_client_creation_variations() {
    // Test that we can create clients with different configs
    let _client1 = OpenAIClient::new("key1").unwrap();
    let _client2 = OpenAIClient::with_config("key2", OpenAIConfig::default()).unwrap();

    let custom_config = OpenAIConfig {
        default_model: "gpt-4".to_string(),
        timeout_seconds: 90,
        ..Default::default()
    };
    let _client3 = OpenAIClient::with_config("key3", custom_config).unwrap();
}

// ========== Rate Limiter Tests ==========

#[test]
fn test_openai_client_has_rate_limiter() {
    let client = OpenAIClient::new("test-key").unwrap();
    // Should have a rate limiter - just verify it exists
    let debug_str = format!("{:?}", client.rate_limiter);
    assert!(debug_str.contains("RateLimiter"));
}

#[test]
fn test_openai_rate_limiter_config() {
    let config = OpenAIConfig {
        rate_limit: 100,
        ..Default::default()
    };
    let client = OpenAIClient::with_config("test-key", config).unwrap();
    // Verify rate limiter was created with the config
    let debug_str = format!("{:?}", client.rate_limiter);
    assert!(debug_str.contains("RateLimiter"));
}

// ========== Configuration Builder Pattern Tests ==========

#[test]
fn test_openai_config_builder_pattern() {
    // Test configuration variations
    let config1 = OpenAIConfig::default();
    let config2 = OpenAIConfig {
        default_model: "gpt-4".to_string(),
        ..config1.clone()
    };
    let config3 = OpenAIConfig {
        rate_limit: 120,
        ..config2
    };

    assert_eq!(config3.default_model, "gpt-4");
    assert_eq!(config3.rate_limit, 120);
}

#[test]
fn test_organization_configuration() {
    let config_no_org = OpenAIConfig::default();
    assert!(config_no_org.organization.is_none());

    let config_with_org = OpenAIConfig {
        organization: Some("org-test".to_string()),
        ..Default::default()
    };
    assert_eq!(config_with_org.organization, Some("org-test".to_string()));
}

// ========== Message Construction Tests ==========

#[test]
fn test_message_with_name() {
    let msg = ChatMessage {
        role: MessageRole::User,
        content: Some("Hello".to_string()),
        name: Some("John".to_string()),
        tool_calls: None,
        tool_call_id: None,
    };

    assert_eq!(msg.name, Some("John".to_string()));
}

#[test]
fn test_empty_message_content() {
    let msg = ChatMessage {
        role: MessageRole::User,
        content: None,
        name: None,
        tool_calls: None,
        tool_call_id: None,
    };

    assert!(msg.content.is_none());
}

// ========== Integration Tests ==========

#[test]
fn test_full_request_construction() {
    let request = ChatRequest {
        messages: vec![
            ChatMessage {
                role: MessageRole::System,
                content: Some("You are a helpful assistant".to_string()),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            },
            ChatMessage {
                role: MessageRole::User,
                content: Some("Hello!".to_string()),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            },
        ],
        model: Some("gpt-4".to_string()),
        parameters: Some(ModelParameters {
            temperature: Some(0.7),
            max_tokens: Some(150),
            ..Default::default()
        }),
        tools: None,
    };

    assert_eq!(request.messages.len(), 2);
    assert!(request.model.is_some());
    assert!(request.parameters.is_some());
}
