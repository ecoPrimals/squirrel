// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use crate::PrimalType;

// --- RetryConfig tests ---
#[test]
fn test_retry_config_default() {
    let config = RetryConfig::default();
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.initial_delay_ms, 1000);
    assert_eq!(config.max_delay_ms, 30000);
    assert!((config.backoff_multiplier - 2.0).abs() < f64::EPSILON);
}

#[test]
fn test_retry_config_custom() {
    let config = RetryConfig {
        max_retries: 5,
        initial_delay_ms: 500,
        max_delay_ms: 60000,
        backoff_multiplier: 3.0,
    };
    assert_eq!(config.max_retries, 5);
    assert_eq!(config.initial_delay_ms, 500);
    assert_eq!(config.max_delay_ms, 60000);
    assert!((config.backoff_multiplier - 3.0).abs() < f64::EPSILON);
}

#[test]
fn test_retry_config_clone() {
    let config = RetryConfig::default();
    let cloned = config.clone();
    assert_eq!(cloned.max_retries, config.max_retries);
    assert_eq!(cloned.initial_delay_ms, config.initial_delay_ms);
}

// --- ServiceQuery tests ---
#[test]
fn test_service_query_default() {
    let query = ServiceQuery::default();
    assert!(query.service_type.is_none());
    assert!(query.primal_type.is_none());
    assert!(query.capabilities.is_empty());
    assert!(query.health_status.is_none());
    assert!(query.metadata.is_empty());
}

#[test]
fn test_service_query_with_filters() {
    let query = ServiceQuery {
        service_type: Some("ai".to_string()),
        primal_type: Some(PrimalType::Squirrel),
        capabilities: vec!["inference".to_string()],
        health_status: None,
        metadata: std::collections::HashMap::new(),
    };
    assert_eq!(query.service_type.as_deref(), Some("ai"));
    assert!(matches!(query.primal_type, Some(PrimalType::Squirrel)));
    assert_eq!(query.capabilities.len(), 1);
}

// --- ServiceInfo tests ---
#[test]
fn test_service_info_serde() {
    let info = ServiceInfo {
        id: "svc-1".to_string(),
        name: "test-service".to_string(),
        service_type: "ai".to_string(),
        primal_type: PrimalType::Squirrel,
        endpoint: "http://localhost:8080".to_string(),
        capabilities: vec!["inference".to_string()],
        health_status: "healthy".to_string(),
        metadata: std::collections::HashMap::new(),
    };
    let json = serde_json::to_string(&info).expect("should succeed");
    let deserialized: ServiceInfo = serde_json::from_str(&json).expect("should succeed");
    assert_eq!(deserialized.id, "svc-1");
    assert_eq!(deserialized.name, "test-service");
    assert_eq!(deserialized.endpoint, "http://localhost:8080");
    assert_eq!(deserialized.capabilities.len(), 1);
}

#[test]
fn test_service_info_clone() {
    let info = ServiceInfo {
        id: "svc-2".to_string(),
        name: "cloned-service".to_string(),
        service_type: "compute".to_string(),
        primal_type: PrimalType::ToadStool,
        endpoint: "http://localhost:9090".to_string(),
        capabilities: vec![],
        health_status: "degraded".to_string(),
        metadata: std::collections::HashMap::new(),
    };
    let cloned = info.clone();
    assert_eq!(cloned.id, info.id);
    assert_eq!(cloned.health_status, "degraded");
}

// --- AICapability tests ---
#[test]
fn test_ai_capability_variants() {
    let caps = vec![
        AICapability::TextGeneration,
        AICapability::CodeGeneration,
        AICapability::ImageGeneration,
        AICapability::SpeechSynthesis,
        AICapability::LanguageTranslation,
        AICapability::QuestionAnswering,
        AICapability::Summarization,
        AICapability::Classification,
        AICapability::SentimentAnalysis,
        AICapability::MultiModal,
    ];
    assert_eq!(caps.len(), 10);
}

#[test]
fn test_ai_capability_eq() {
    assert_eq!(AICapability::TextGeneration, AICapability::TextGeneration);
    assert_ne!(AICapability::TextGeneration, AICapability::CodeGeneration);
}

#[test]
fn test_ai_capability_clone() {
    let cap = AICapability::MultiModal;
    let cloned = cap.clone();
    assert_eq!(cap, cloned);
}

// --- ProviderHealth tests ---
#[test]
fn test_provider_health_healthy() {
    let health = ProviderHealth {
        healthy: true,
        message: "OK".to_string(),
        response_time_ms: 50,
        error_rate: 0.0,
        load_percentage: 25.0,
    };
    assert!(health.is_healthy());
    assert_eq!(health.response_time_ms, 50);
}

#[test]
fn test_provider_health_unhealthy() {
    let health = ProviderHealth {
        healthy: false,
        message: "High error rate".to_string(),
        response_time_ms: 5000,
        error_rate: 50.0,
        load_percentage: 95.0,
    };
    assert!(!health.is_healthy());
    assert_eq!(health.message, "High error rate");
}

// --- AIError tests ---
#[test]
fn test_ai_error_display() {
    let err = AIError::ProviderUnavailable("test-provider".to_string());
    assert_eq!(err.to_string(), "Provider unavailable: test-provider");

    let err = AIError::RateLimitExceeded("too many requests".to_string());
    assert_eq!(err.to_string(), "Rate limit exceeded: too many requests");
}

#[test]
fn test_ai_error_variants() {
    let errors: Vec<AIError> = vec![
        AIError::ProviderUnavailable("a".to_string()),
        AIError::ProviderUnhealthy("b".to_string()),
        AIError::InvalidRequest("c".to_string()),
        AIError::RateLimitExceeded("d".to_string()),
        AIError::AuthenticationFailed("e".to_string()),
        AIError::NetworkError("f".to_string()),
        AIError::InternalError("g".to_string()),
    ];
    assert_eq!(errors.len(), 7);
    // All should display correctly
    for err in &errors {
        assert!(!err.to_string().is_empty());
    }
}

// --- TokenUsage tests ---
#[test]
fn test_token_usage() {
    let usage = TokenUsage {
        input_tokens: 100,
        output_tokens: 50,
        total_tokens: 150,
    };
    assert_eq!(usage.input_tokens, 100);
    assert_eq!(usage.output_tokens, 50);
    assert_eq!(usage.total_tokens, 150);
}

#[test]
fn test_token_usage_clone() {
    let usage = TokenUsage {
        input_tokens: 200,
        output_tokens: 100,
        total_tokens: 300,
    };
    let cloned = usage;
    assert_eq!(cloned.total_tokens, 300);
}

// --- InferenceRequest tests ---
#[test]
fn test_inference_request() {
    let req = InferenceRequest {
        id: "req-1".to_string(),
        input: "Hello world".to_string(),
        parameters: std::collections::HashMap::new(),
        context: Some("test context".to_string()),
        max_tokens: Some(100),
        temperature: Some(0.7),
        top_p: None,
    };
    assert_eq!(req.id, "req-1");
    assert_eq!(req.max_tokens, Some(100));
    assert!(req.context.is_some());
    assert!(req.top_p.is_none());
}

// --- InferenceResponse tests ---
#[test]
fn test_inference_response() {
    let resp = InferenceResponse {
        request_id: "req-1".to_string(),
        output: "Generated text".to_string(),
        metadata: std::collections::HashMap::new(),
        usage: TokenUsage {
            input_tokens: 10,
            output_tokens: 20,
            total_tokens: 30,
        },
        response_time_ms: 150,
    };
    assert_eq!(resp.request_id, "req-1");
    assert_eq!(resp.response_time_ms, 150);
    assert_eq!(resp.usage.total_tokens, 30);
}

// --- InferenceChunk tests ---
#[test]
fn test_inference_chunk() {
    let chunk = InferenceChunk {
        request_id: "req-1".to_string(),
        content: "partial output".to_string(),
        is_final: false,
        metadata: std::collections::HashMap::new(),
    };
    assert!(!chunk.is_final);
    assert_eq!(chunk.content, "partial output");
}

// --- AIRequest/AIResponse tests ---
#[test]
fn test_ai_request() {
    let req = AIRequest {
        id: "ai-req-1".to_string(),
        prompt: "What is Rust?".to_string(),
        capabilities: vec!["text_generation".to_string()],
        context: None,
        preferences: None,
    };
    assert_eq!(req.capabilities.len(), 1);
    assert!(req.context.is_none());
}

#[test]
fn test_ai_response() {
    let resp = AIResponse {
        request_id: "ai-req-1".to_string(),
        content: "Rust is a systems programming language".to_string(),
        provider: "test-provider".to_string(),
        metadata: std::collections::HashMap::new(),
        response_time_ms: 200,
    };
    assert_eq!(resp.provider, "test-provider");
    assert_eq!(resp.response_time_ms, 200);
}

// --- ServiceConfig tests ---
#[test]
fn test_service_config() {
    let config = ServiceConfig {
        name: "squirrel".to_string(),
        version: "1.0.0".to_string(),
        description: "AI Primal".to_string(),
        bind_address: "0.0.0.0".to_string(),
        port: 8080,
        log_level: "info".to_string(),
        instance_id: "inst-1".to_string(),
    };
    assert_eq!(config.name, "squirrel");
    assert_eq!(config.port, 8080);
}

// --- FeatureFlags tests ---
#[test]
fn test_feature_flags() {
    let flags = FeatureFlags {
        development_mode: true,
        debug_logging: false,
        metrics_enabled: true,
        tracing_enabled: true,
        experimental_features: vec!["feature_a".to_string()],
    };
    assert!(flags.development_mode);
    assert!(!flags.debug_logging);
    assert_eq!(flags.experimental_features.len(), 1);
}

// --- ResourceConfig tests ---
#[test]
fn test_resource_config() {
    let config = ResourceConfig {
        cpu_cores: Some(4.0),
        memory_mb: Some(8192),
        disk_mb: Some(100_000),
        network_bandwidth_mbps: Some(1000),
        gpu_count: Some(1),
    };
    assert_eq!(config.cpu_cores, Some(4.0));
    assert_eq!(config.gpu_count, Some(1));
}

#[test]
fn test_resource_config_empty() {
    let config = ResourceConfig {
        cpu_cores: None,
        memory_mb: None,
        disk_mb: None,
        network_bandwidth_mbps: None,
        gpu_count: None,
    };
    assert!(config.cpu_cores.is_none());
    assert!(config.gpu_count.is_none());
}

// --- NetworkConfig tests ---
#[test]
fn test_network_config() {
    let config = NetworkConfig {
        port: 443,
        max_connections: 1000,
        connection_timeout_secs: 30,
        read_timeout_secs: 60,
        write_timeout_secs: 60,
    };
    assert_eq!(config.port, 443);
    assert_eq!(config.max_connections, 1000);
}
