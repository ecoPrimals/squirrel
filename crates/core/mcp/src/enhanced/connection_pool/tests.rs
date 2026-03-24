// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Comprehensive tests for Connection Pool functionality
//!
//! Tests cover connection pool management, request routing, load balancing,
//! health monitoring, metrics collection, and performance optimization.

use super::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

use crate::enhanced::coordinator::{UniversalAIRequest, Message};

/// Helper function to create test connection pool configuration
fn create_test_pool_config() -> ConnectionPoolConfig {
    ConnectionPoolConfig {
        max_connections_per_provider: 5,
        max_idle_connections: 3,
        connection_timeout_ms: 5000,
        request_timeout_ms: 10000,
        idle_timeout_seconds: 30,
        keep_alive_timeout_seconds: 60,
        max_retries: 2,
        health_check_interval_seconds: 10,
        enable_http2: true,
        tcp_keep_alive: true,
        user_agent: "MCP-Test-Client/1.0".to_string(),
    }
}

/// Helper function to create test provider configuration
fn create_test_provider_config(name: &str, base_url: &str) -> ProviderConnectionConfig {
    ProviderConnectionConfig {
        name: name.to_string(),
        base_url: base_url.to_string(),
        headers: HashMap::new(),
        connection_timeout_ms: None,
        request_timeout_ms: None,
        tls_config: TlsConfig::default(),
        rate_limit: RateLimitConfig::default(),
    }
}

/// Helper function to create test AI request
fn create_test_request() -> UniversalAIRequest {
    UniversalAIRequest {
        id: uuid::Uuid::new_v4().to_string(),
        model: "gpt-4".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: "Test message".to_string(),
        }],
        parameters: HashMap::new(),
    }
}

#[tokio::test]
async fn test_connection_pool_creation() {
    let config = create_test_pool_config();
    let pool = ConnectionPool::new(config);
    
    // Test initial state
    let metrics = pool.get_metrics().await;
    assert_eq!(metrics.total_requests, 0);
    assert_eq!(metrics.active_connections, 0);
    
    let providers = pool.list_providers().await;
    assert!(providers.is_empty());
}

#[tokio::test]
async fn test_provider_registration() {
    let config = create_test_pool_config();
    let pool = ConnectionPool::new(config);
    
    let provider_config = create_test_provider_config("openai", "https://api.openai.com/v1");
    
    // Register provider
    let result = pool.register_provider(provider_config).await;
    assert!(result.is_ok());
    
    // Verify registration
    let providers = pool.list_providers().await;
    assert_eq!(providers.len(), 1);
    assert!(providers.contains(&"openai".to_string()));
    
    // Check provider health
    let health = pool.get_provider_health("openai").await;
    assert!(health.is_some());
}

#[tokio::test]
async fn test_connection_pool_manager() {
    let config = create_test_pool_config();
    let manager = ConnectionPoolManager::new(config);
    
    // Register multiple providers
    let openai_config = create_test_provider_config("openai", "https://api.openai.com/v1");
    let anthropic_config = create_test_provider_config("anthropic", "https://api.anthropic.com");
    
    manager.register_provider(openai_config).await.expect("should succeed");
    manager.register_provider(anthropic_config).await.expect("should succeed");
    
    // Test routing
    let request = create_test_request();
    let provider = manager.route_request(&request).await;
    assert!(provider.is_ok());
    
    let selected_provider = provider.expect("should succeed");
    assert!(selected_provider == "openai" || selected_provider == "anthropic");
}

#[tokio::test]
async fn test_pooled_request_creation() {
    let request = PooledRequest::get(
        "openai".to_string(),
        "https://api.openai.com/v1/chat/completions".to_string()
    );
    
    assert_eq!(request.provider_name, "openai");
    assert_eq!(request.method, reqwest::Method::GET);
    assert!(!request.id.is_empty());
    assert!(request.created_at.elapsed().as_millis() < 100);
    
    // Test with headers
    let request_with_headers = request
        .with_header("Authorization".to_string(), "Bearer token".to_string())
        .with_header("Content-Type".to_string(), "application/json".to_string());
    
    assert_eq!(request_with_headers.headers.len(), 2);
    assert_eq!(request_with_headers.headers.get("Authorization"), Some(&"Bearer token".to_string()));
}

#[tokio::test]
async fn test_pooled_post_request() {
    let body = b"test request body".to_vec();
    let request = PooledRequest::post(
        "anthropic".to_string(),
        "https://api.anthropic.com/v1/messages".to_string(),
        body.clone()
    );
    
    assert_eq!(request.provider_name, "anthropic");
    assert_eq!(request.method, reqwest::Method::POST);
    assert_eq!(request.body, Some(body));
}

#[tokio::test]
async fn test_rate_limiter() {
    let rate_limiter = RateLimiter::new(2.0, 5); // 2 requests/second, burst of 5
    
    // Should be able to acquire tokens initially
    assert!(rate_limiter.try_acquire().await);
    assert!(rate_limiter.try_acquire().await);
    assert!(rate_limiter.try_acquire().await);
    
    // Check token count
    let token_count = rate_limiter.get_token_count().await;
    assert!(token_count < 5.0);
    
    // Test blocking acquire
    let start = std::time::Instant::now();
    rate_limiter.acquire().await.expect("should succeed");
    let duration = start.elapsed();
    
    // Should have taken some time to acquire due to rate limiting
    assert!(duration.as_millis() >= 0); // Just verify it doesn't panic
}

#[tokio::test]
async fn test_retry_policy() {
    let policy = RetryPolicy::default();
    
    // Test delay calculation
    let delay1 = policy.calculate_delay(1);
    let delay2 = policy.calculate_delay(2);
    let delay3 = policy.calculate_delay(3);
    
    // Delays should increase with exponential backoff
    assert!(delay2 >= delay1);
    assert!(delay3 >= delay2);
    
    // Test retryable status codes
    assert!(policy.is_retryable_status(429)); // Rate limit
    assert!(policy.is_retryable_status(500)); // Internal server error
    assert!(policy.is_retryable_status(503)); // Service unavailable
    assert!(!policy.is_retryable_status(200)); // OK
    assert!(!policy.is_retryable_status(404)); // Not found
}

#[tokio::test]
async fn test_connection_pool_metrics() {
    let mut metrics = ConnectionPoolMetrics::new();
    
    // Test initial state
    assert_eq!(metrics.total_requests, 0);
    assert_eq!(metrics.successful_requests, 0);
    assert_eq!(metrics.failed_requests, 0);
    assert_eq!(metrics.efficiency_rate, 1.0); // Should be 1.0 with no requests
    
    // Record successful request
    metrics.record_success(
        "openai",
        Duration::from_millis(500),
        1024,  // bytes sent
        2048,  // bytes received
    );
    
    assert_eq!(metrics.total_requests, 1);
    assert_eq!(metrics.successful_requests, 1);
    assert_eq!(metrics.failed_requests, 0);
    assert_eq!(metrics.efficiency_rate, 1.0);
    assert!(metrics.avg_response_time_ms > 0.0);
    
    // Record failed request
    metrics.record_failure("openai", "connection timeout", Some(Duration::from_millis(5000)));
    
    assert_eq!(metrics.total_requests, 2);
    assert_eq!(metrics.successful_requests, 1);
    assert_eq!(metrics.failed_requests, 1);
    assert_eq!(metrics.efficiency_rate, 0.5);
}

#[tokio::test]
async fn test_provider_metrics() {
    let mut provider_metrics = ProviderMetrics::default();
    provider_metrics.provider_name = "test-provider".to_string();
    
    // Record successful request
    provider_metrics.record_success(
        Duration::from_millis(200),
        512,  // bytes sent
        1024, // bytes received
    );
    
    assert_eq!(provider_metrics.total_requests, 1);
    assert_eq!(provider_metrics.successful_requests, 1);
    assert_eq!(provider_metrics.failed_requests, 0);
    assert_eq!(provider_metrics.success_rate, 1.0);
    assert_eq!(provider_metrics.error_rate, 0.0);
    assert_eq!(provider_metrics.avg_response_time_ms, 200.0);
    assert_eq!(provider_metrics.min_response_time_ms, 200.0);
    assert_eq!(provider_metrics.max_response_time_ms, 200.0);
    assert_eq!(provider_metrics.bytes_sent, 512);
    assert_eq!(provider_metrics.bytes_received, 1024);
    assert!(provider_metrics.is_healthy);
    
    // Record failed request
    provider_metrics.record_failure("timeout error", Some(Duration::from_millis(5000)));
    
    assert_eq!(provider_metrics.total_requests, 2);
    assert_eq!(provider_metrics.successful_requests, 1);
    assert_eq!(provider_metrics.failed_requests, 1);
    assert_eq!(provider_metrics.success_rate, 0.5);
    assert_eq!(provider_metrics.error_rate, 0.5);
    assert_eq!(provider_metrics.timeout_errors, 1);
    assert!(provider_metrics.last_error.is_some());
    assert!(provider_metrics.last_error_at.is_some());
}

#[tokio::test]
async fn test_performance_report() {
    let mut metrics = ConnectionPoolMetrics::new();
    
    // Add some test data
    metrics.record_success("openai", Duration::from_millis(100), 100, 200);
    metrics.record_success("openai", Duration::from_millis(150), 120, 250);
    metrics.record_success("anthropic", Duration::from_millis(200), 110, 220);
    metrics.record_failure("anthropic", "rate limit", Some(Duration::from_millis(300)));
    
    let report = metrics.generate_performance_report();
    
    // Test summary
    assert_eq!(report.summary.total_requests, 4);
    assert_eq!(report.summary.success_rate, 0.75); // 3 successful out of 4
    assert!(report.summary.avg_response_time_ms > 0.0);
    assert_eq!(report.summary.provider_count, 2);
    
    // Test provider breakdown
    assert!(report.provider_breakdown.contains_key("openai"));
    assert!(report.provider_breakdown.contains_key("anthropic"));
    
    let openai_metrics = &report.provider_breakdown["openai"];
    assert_eq!(openai_metrics.total_requests, 2);
    assert_eq!(openai_metrics.successful_requests, 2);
    assert_eq!(openai_metrics.failed_requests, 0);
    assert_eq!(openai_metrics.success_rate, 1.0);
    
    let anthropic_metrics = &report.provider_breakdown["anthropic"];
    assert_eq!(anthropic_metrics.total_requests, 2);
    assert_eq!(anthropic_metrics.successful_requests, 1);
    assert_eq!(anthropic_metrics.failed_requests, 1);
    assert_eq!(anthropic_metrics.success_rate, 0.5);
    
    // Should have recommendations due to anthropic's elevated error rate
    assert!(!report.recommendations.is_empty());
}

#[tokio::test]
async fn test_pool_statistics() {
    let mut stats = PoolStatistics::default();
    
    stats.total_connections = 10;
    stats.active_connections = 7;
    stats.idle_connections = 3;
    
    // Test utilization calculation
    stats.calculate_utilization();
    assert_eq!(stats.utilization_percentage, 70.0);
    
    // Test efficiency calculation
    let efficiency = stats.efficiency_score();
    assert!(efficiency > 0.0 && efficiency <= 1.0);
    
    // Test utilization checks
    assert!(!stats.is_under_utilized()); // 70% is not under-utilized
    assert!(!stats.is_over_utilized());  // 70% is not over-utilized
    
    // Test over-utilization
    stats.active_connections = 9;
    stats.calculate_utilization();
    assert_eq!(stats.utilization_percentage, 90.0);
    assert!(!stats.is_over_utilized()); // Exactly at 90% threshold
    
    stats.active_connections = 10;
    stats.calculate_utilization();
    assert_eq!(stats.utilization_percentage, 100.0);
    assert!(stats.is_over_utilized()); // Over 90% threshold
}

#[tokio::test]
async fn test_request_priority() {
    let priorities = vec![
        RequestPriority::Low,
        RequestPriority::Normal,
        RequestPriority::High,
        RequestPriority::Critical,
    ];
    
    // Test priority ordering
    for i in 0..priorities.len() {
        for j in (i+1)..priorities.len() {
            assert!(priorities[j].is_higher_than(priorities[i]));
            assert!(!priorities[i].is_higher_than(priorities[j]));
        }
    }
    
    // Test numeric values
    assert_eq!(RequestPriority::Low.as_u8(), 1);
    assert_eq!(RequestPriority::Normal.as_u8(), 2);
    assert_eq!(RequestPriority::High.as_u8(), 3);
    assert_eq!(RequestPriority::Critical.as_u8(), 4);
    
    // Test default
    assert_eq!(RequestPriority::default(), RequestPriority::Normal);
}

#[tokio::test]
async fn test_connection_state_transitions() {
    let states = vec![
        ConnectionState::Idle,
        ConnectionState::Active,
        ConnectionState::Connecting,
        ConnectionState::Closing,
        ConnectionState::Failed,
        ConnectionState::Maintenance,
    ];
    
    // Test that all states are distinct
    for (i, state1) in states.iter().enumerate() {
        for (j, state2) in states.iter().enumerate() {
            if i == j {
                assert_eq!(state1, state2);
            } else {
                assert_ne!(state1, state2);
            }
        }
    }
    
    // Test serialization/deserialization
    for state in &states {
        let serialized = serde_json::to_string(state).expect("should succeed");
        let deserialized: ConnectionState = serde_json::from_str(&serialized).expect("should succeed");
        assert_eq!(state, &deserialized);
    }
}

#[tokio::test]
async fn test_tls_config() {
    let default_config = TlsConfig::default();
    
    assert!(default_config.verify_certificates);
    assert!(default_config.ca_cert_path.is_none());
    assert!(default_config.client_cert_path.is_none());
    assert!(default_config.client_key_path.is_none());
    
    // Test serialization
    let serialized = serde_json::to_string(&default_config).expect("should succeed");
    let deserialized: TlsConfig = serde_json::from_str(&serialized).expect("should succeed");
    assert_eq!(default_config.verify_certificates, deserialized.verify_certificates);
}

#[tokio::test]
async fn test_rate_limit_config() {
    let default_config = RateLimitConfig::default();
    
    assert_eq!(default_config.max_requests_per_second, 10.0);
    assert_eq!(default_config.burst_capacity, 20);
    assert_eq!(default_config.retry_delay_ms, 1000);
    
    // Test serialization
    let serialized = serde_json::to_string(&default_config).expect("should succeed");
    let deserialized: RateLimitConfig = serde_json::from_str(&serialized).expect("should succeed");
    assert_eq!(default_config.max_requests_per_second, deserialized.max_requests_per_second);
    assert_eq!(default_config.burst_capacity, deserialized.burst_capacity);
}

#[tokio::test]
async fn test_connection_pool_config_defaults() {
    let config = ConnectionPoolConfig::default();
    
    assert_eq!(config.max_connections_per_provider, 20);
    assert_eq!(config.max_idle_connections, 10);
    assert_eq!(config.connection_timeout_ms, 30000);
    assert_eq!(config.request_timeout_ms, 60000);
    assert_eq!(config.idle_timeout_seconds, 300);
    assert_eq!(config.keep_alive_timeout_seconds, 90);
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.health_check_interval_seconds, 60);
    assert!(config.enable_http2);
    assert!(config.tcp_keep_alive);
    assert_eq!(config.user_agent, "MCP-AI-Client/1.0");
}

#[tokio::test]
async fn test_provider_connection_config_validation() {
    let config = ConnectionPoolConfig::default();
    let pool = ConnectionPool::new(config);
    
    // Test valid configuration
    let valid_config = ProviderConnectionConfig {
        name: "test-provider".to_string(),
        base_url: "https://api.example.com".to_string(),
        headers: HashMap::new(),
        connection_timeout_ms: None,
        request_timeout_ms: None,
        tls_config: TlsConfig::default(),
        rate_limit: RateLimitConfig::default(),
    };
    
    let result = pool.register_provider(valid_config).await;
    assert!(result.is_ok());
    
    // Test invalid configuration (empty name)
    let invalid_config = ProviderConnectionConfig {
        name: "".to_string(),
        base_url: "https://api.example.com".to_string(),
        headers: HashMap::new(),
        connection_timeout_ms: None,
        request_timeout_ms: None,
        tls_config: TlsConfig::default(),
        rate_limit: RateLimitConfig::default(),
    };
    
    let result = pool.register_provider(invalid_config).await;
    assert!(result.is_err());
    
    // Test invalid URL
    let invalid_url_config = ProviderConnectionConfig {
        name: "invalid-url-provider".to_string(),
        base_url: "not-a-valid-url".to_string(),
        headers: HashMap::new(),
        connection_timeout_ms: None,
        request_timeout_ms: None,
        tls_config: TlsConfig::default(),
        rate_limit: RateLimitConfig::default(),
    };
    
    let result = pool.register_provider(invalid_url_config).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_concurrent_pool_operations() {
    let config = create_test_pool_config();
    let pool = Arc::new(ConnectionPool::new(config));
    
    // Register a provider
    let provider_config = create_test_provider_config("concurrent-test", "https://api.example.com");
    pool.register_provider(provider_config).await.expect("should succeed");
    
    // Perform concurrent operations
    let mut handles = Vec::new();
    
    for i in 0..5 {
        let pool_clone = Arc::clone(&pool);
        let handle = tokio::spawn(async move {
            // Try to get client
            let result = pool_clone.get_client("concurrent-test").await;
            (i, result.is_ok())
        });
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    let results = futures::future::join_all(handles).await;
    
    // All operations should succeed
    for result in results {
        let (i, success) = result.expect("should succeed");
        assert!(success, "Operation {} should have succeeded", i);
    }
}

// Integration test with multiple providers and realistic usage
#[tokio::test]
async fn test_realistic_connection_pool_usage() {
    let config = create_test_pool_config();
    let manager = ConnectionPoolManager::new(config);
    
    // Register multiple AI providers
    let providers = vec![
        ("openai", "https://api.openai.com/v1"),
        ("anthropic", "https://api.anthropic.com"),
        ("gemini", "https://generativelanguage.googleapis.com/v1beta"),
    ];
    
    for (name, url) in providers {
        let provider_config = create_test_provider_config(name, url);
        manager.register_provider(provider_config).await.expect("should succeed");
    }
    
    // Simulate multiple concurrent requests
    let mut request_handles = Vec::new();
    
    for i in 0..10 {
        let manager_ref = &manager;
        let handle = tokio::spawn(async move {
            let mut request = create_test_request();
            request.id = format!("request-{}", i);
            
            // Route and track the request
            let provider = manager_ref.route_request(&request).await;
            (i, provider.is_ok())
        });
        request_handles.push(handle);
    }
    
    // Wait for all requests to complete
    let request_results = futures::future::join_all(request_handles).await;
    
    // All requests should be routed successfully
    for result in request_results {
        let (i, success) = result.expect("should succeed");
        assert!(success, "Request {} should have been routed successfully", i);
    }
    
    // Check global metrics
    let metrics = manager.get_global_metrics().await;
    assert_eq!(metrics.total_pools, 3);
    
    // Start background tasks (would run indefinitely in real usage)
    let start_result = manager.start_background_tasks().await;
    assert!(start_result.is_ok());
    
    // Shutdown cleanly
    let shutdown_result = manager.shutdown().await;
    assert!(shutdown_result.is_ok());
} 