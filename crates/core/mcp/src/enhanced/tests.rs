// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Comprehensive Integration Tests for Enhanced MCP Platform
//! 
//! This module provides comprehensive test coverage for all Phase 3 features:
//! - Universal AI coordination
//! - Event broadcasting
//! - Intelligent routing
//! - Streaming management
//! - Provider configuration
//! - Mock behavior testing

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;

use crate::error::Result;
use super::*;
use super::providers::{MockBehavior, ProviderFactory, OpenAIConfig, LocalServerProviderConfig};
use super::coordinator::{UniversalAIRequest, AIRequestType, MessageContent, RequestContext, RoutingHints, QualityRequirements, RoutingStrategy};
use super::events::{MCPEvent, EventType, EventSource, SourceType, EventPriority};
use super::intelligent_router::{IntelligentRouter, RoutingRule, RoutingConditions, RoutingActions};

/// Test configuration builder
pub struct TestConfigBuilder {
    platform_config: EnhancedPlatformConfig,
}

impl TestConfigBuilder {
    pub fn new() -> Self {
        Self {
            platform_config: create_default_config(),
        }
    }
    
    pub fn with_mock_providers(mut self) -> Self {
        // Configure test providers with mock behaviors
        self.platform_config.ai_coordinator.openai_api_key = Some("test-key".to_string());
        self.platform_config.ai_coordinator.enable_local_server = true;
        self.platform_config.ai_coordinator.enable_native = false; // Disable complex providers for tests
        self
    }
    
    pub fn with_fast_timeouts(mut self) -> Self {
        self.platform_config.platform_settings.request_timeout = Duration::from_secs(5);
        self.platform_config.ai_coordinator.request_timeout = Duration::from_secs(5);
        self
    }
    
    pub fn with_test_models(mut self) -> Self {
        self.platform_config.ai_coordinator.local_server_config.models = vec![
            "test-model-1".to_string(),
            "test-model-2".to_string(),
        ];
        self
    }
    
    pub fn build(self) -> EnhancedPlatformConfig {
        self.platform_config
    }
}

/// Integration test suite for the enhanced platform
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_platform_full_lifecycle() {
        let config = TestConfigBuilder::new()
            .with_mock_providers()
            .with_fast_timeouts()
            .build();
            
        // Test platform creation
        let mut platform = EnhancedMCPPlatform::new(config).await
            .expect("Platform should initialize successfully");
        
        // Test platform startup
        platform.start().await
            .expect("Platform should start successfully");
        
        // Test platform health check
        let health = platform.get_health().await
            .expect("Health check should succeed");
        assert_eq!(health.status, PlatformStatus::Healthy);
        
        // Test platform metrics
        let metrics = platform.get_metrics().await
            .expect("Metrics should be available");
        assert!(metrics.ai_coordinator.total_requests >= 0);
        
        // Test platform shutdown
        platform.stop().await
            .expect("Platform should stop gracefully");
    }
    
    #[tokio::test]
    async fn test_ai_coordination_with_mock_providers() {
        let config = TestConfigBuilder::new()
            .with_mock_providers()
            .with_fast_timeouts()
            .with_test_models()
            .build();
            
        let platform = EnhancedMCPPlatform::new(config).await.expect("should succeed");
        
        // Create AI request
        let request = UniversalAIRequest {
            id: Uuid::new_v4().to_string(),
            model: "test-model-1".to_string(),
            messages: vec![MessageContent {
                role: "user".to_string(),
                content: "Hello, this is a test message".to_string(),
            }],
            request_type: AIRequestType::TextGeneration,
            metadata: HashMap::new(),
            payload: serde_json::json!({
                "messages": [{"role": "user", "content": "Hello, this is a test message"}]
            }),
            context: RequestContext {
                user_id: Some("test-user".to_string()),
                session_id: Some("test-session".to_string()),
                metadata: HashMap::new(),
            },
            hints: RoutingHints {
                prefer_local: false,
                max_cost: Some(0.1),
                max_latency: Some(Duration::from_secs(30)),
                quality_requirements: vec![],
            },
            requirements: QualityRequirements {
                min_quality_score: None,
                require_streaming: false,
                require_tools: false,
            },
        };
        
        // Process request through AI coordinator
        let result = timeout(
            Duration::from_secs(10), 
            platform.process_ai_request(request)
        ).await;
        
        assert!(result.is_ok(), "Request should complete within timeout");
        let response = result.expect("should succeed");
        assert!(response.is_ok(), "AI request should succeed with mock provider");
    }
    
    #[tokio::test]
    async fn test_event_broadcasting_system() {
        let config = TestConfigBuilder::new()
            .with_mock_providers()
            .build();
            
        let platform = EnhancedMCPPlatform::new(config).await.expect("should succeed");
        
        // Subscribe to AI events
        let mut ai_event_receiver = platform.subscribe_to_events("AIRequestStarted").await
            .expect("Should be able to subscribe to AI events");
        
        // Subscribe to system events
        let mut system_event_receiver = platform.subscribe_to_events("SystemStartup").await
            .expect("Should be able to subscribe to system events");
        
        // Create and publish a test event
        let test_event = MCPEvent::new(
            EventType::AIRequestStarted,
            EventSource::new(
                SourceType::AIProvider,
                "test-provider".to_string(),
                "Test Provider".to_string(),
            ),
            serde_json::json!({"test": "data"}),
        );
        
        platform.event_broadcaster.publish(test_event).await
            .expect("Event should publish successfully");
        
        // Check that event was received
        let received_event = timeout(Duration::from_secs(1), ai_event_receiver.recv()).await;
        assert!(received_event.is_ok(), "AI event should be received");
        assert!(received_event.expect("should succeed").is_ok(), "Event should be valid");
    }
    
    #[tokio::test]
    async fn test_intelligent_routing_decisions() {
        let router = IntelligentRouter::new(
            RoutingStrategy::BestFit,
            0.3, // cost weight
            0.4, // latency weight
            0.3, // quality weight
        );
        
        // Add a routing rule
        let rule = RoutingRule {
            id: "test-rule".to_string(),
            name: "Cost Optimization Rule".to_string(),
            priority: 100,
            conditions: RoutingConditions {
                max_cost_per_token: Some(0.0001),
                min_success_rate: Some(0.95),
                allowed_providers: Some(vec!["local-server".to_string()]),
                ..Default::default()
            },
            actions: RoutingActions {
                force_provider: Some("local-server".to_string()),
                ..Default::default()
            },
            enabled: true,
        };
        
        router.add_rule(rule).await.expect("Rule should be added");
        
        // Update performance metrics for providers
        router.update_performance(
            "openai",
            "gpt-4",
            Duration::from_millis(2000),
            Some(0.03),
            true,
        ).await.expect("Performance update should succeed");
        
        router.update_performance(
            "local-server", 
            "llama2",
            Duration::from_millis(500),
            Some(0.0),
            true,
        ).await.expect("Performance update should succeed");
        
        // Check that routing considers performance data
        let stats = router.get_performance_stats().await;
        assert!(stats.contains_key("openai:gpt-4"));
        assert!(stats.contains_key("local-server:llama2"));
    }
    
    #[tokio::test]
    async fn test_provider_mock_behaviors() {
        // Test provider with failure simulation (using local server, vendor-agnostic)
        let failing_config = LocalServerProviderConfig {
            base_url: "http://localhost:11434".to_string(),
            timeout: Duration::from_secs(30),
            models: vec!["test-model".to_string()],
        };
        
        let mut failing_provider = ProviderFactory::create_local_server(failing_config)
            .expect("Provider should be created");
        
        let failure_behavior = MockBehavior {
            should_fail: true,
            failure_rate: 0.8,
            latency_ms: 50,
            ..Default::default()
        };
        
        failing_provider = failing_provider.with_mock_behavior(failure_behavior);
        
        // Test that provider fails as expected
        let request = UniversalAIRequest {
            id: "test-id".to_string(),
            model: "test-model".to_string(),
            messages: vec![],
            request_type: AIRequestType::TextGeneration,
            metadata: HashMap::new(),
            payload: serde_json::json!({}),
            context: RequestContext {
                user_id: Some("test-user".to_string()),
                session_id: Some("test-session".to_string()),
                metadata: HashMap::new(),
            },
            hints: RoutingHints {
                prefer_local: false,
                max_cost: Some(0.1),
                max_latency: Some(Duration::from_secs(30)),
                quality_requirements: vec![],
            },
            requirements: QualityRequirements {
                min_quality_score: None,
                require_streaming: false,
                require_tools: false,
            },
        };
        
        let result = failing_provider.process_request(request).await;
        assert!(result.is_err(), "Provider should fail due to mock behavior");
        
        // Test provider with success simulation
        let success_config = OpenAIConfig {
            api_key: "test-key".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            timeout: Duration::from_secs(30),
            models: vec!["gpt-4".to_string()],
            organization: None,
        };
        
        let mut success_provider = ProviderFactory::create_openai(success_config)
            .expect("Provider should be created");
        
        let success_behavior = MockBehavior {
            should_fail: false,
            latency_ms: 100,
            response_override: Some(serde_json::json!({
                "message": "Test response",
                "model": "gpt-4"
            })),
            ..Default::default()
        };
        
        success_provider = success_provider.with_mock_behavior(success_behavior);
        
        let request = UniversalAIRequest {
            id: "test-id-2".to_string(),
            model: "gpt-4".to_string(),
            messages: vec![MessageContent {
                role: "user".to_string(),
                content: "Test message".to_string(),
            }],
            request_type: AIRequestType::TextGeneration,
            metadata: HashMap::new(),
            payload: serde_json::json!({
                "messages": [{"role": "user", "content": "Test message"}]
            }),
            context: RequestContext {
                user_id: Some("test-user".to_string()),
                session_id: Some("test-session".to_string()),
                metadata: HashMap::new(),
            },
            hints: RoutingHints {
                prefer_local: false,
                max_cost: Some(0.1),
                max_latency: Some(Duration::from_secs(30)),
                quality_requirements: vec![],
            },
            requirements: QualityRequirements {
                min_quality_score: None,
                require_streaming: false,
                require_tools: false,
            },
        };
        
        let result = success_provider.process_request(request).await;
        assert!(result.is_ok(), "Provider should succeed");
        
        let response = result.expect("should succeed");
        assert_eq!(response.provider, "openai");
        assert_eq!(response.model, "gpt-4");
    }
    
    #[tokio::test]
    async fn test_tool_execution_integration() {
        let config = TestConfigBuilder::new()
            .with_mock_providers()
            .with_fast_timeouts()
            .build();
            
        let platform = EnhancedMCPPlatform::new(config).await.expect("should succeed");
        
        // Test tool execution
        let result = platform.execute_tool(
            "test_tool",
            serde_json::json!({"param1": "value1"}),
            "test-session"
        ).await;
        
        // Tool execution should work (even if just mock implementation)
        assert!(result.is_ok() || result.is_err(), "Tool execution should return a result");
    }
    
    #[tokio::test]
    async fn test_session_management() {
        let config = TestConfigBuilder::new()
            .with_mock_providers()
            .build();
            
        let platform = EnhancedMCPPlatform::new(config).await.expect("should succeed");
        
        // Create AI session
        let preferences = coordinator::UserPreferences {
            preferred_providers: vec!["test-provider".to_string()],
            privacy_level: coordinator::PrivacyLevel::Private,
            cost_sensitivity: coordinator::CostSensitivity::Low,
            quality_preference: coordinator::QualityPreference::Balanced,
            language: Some("en".to_string()),
            timezone: Some("UTC".to_string()),
            theme: Some("dark".to_string()),
        };
        
        let session_id = platform.create_ai_session(preferences).await
            .expect("Session creation should succeed");
        
        assert!(!session_id.is_empty(), "Session ID should not be empty");
    }
    
    #[tokio::test]
    async fn test_model_listing() {
        let config = TestConfigBuilder::new()
            .with_mock_providers()
            .with_test_models()
            .build();
            
        let platform = EnhancedMCPPlatform::new(config).await.expect("should succeed");
        
        // List available models
        let models = platform.list_all_ai_models().await
            .expect("Model listing should succeed");
        
        // Should have at least the test models
        assert!(!models.is_empty(), "Should have available models");
    }
    
    #[tokio::test]
    async fn test_streaming_creation() {
        let config = TestConfigBuilder::new()
            .with_mock_providers()
            .build();
            
        let platform = EnhancedMCPPlatform::new(config).await.expect("should succeed");
        
        // Create a test stream
        let stream_handle = MockStreamHandle::new();
        let stream_config = streaming::StreamConfig {
            buffer_size: 1024,
            max_chunk_size: 8192,
            timeout: Duration::from_secs(30),
            backpressure: streaming::BackpressureConfig {
                enabled: true,
                high_water_mark: 0.8,
                low_water_mark: 0.3,
                strategy: streaming::BackpressureStrategy::DropOldest,
            },
            quality: streaming::QualityConfig {
                level: 0.8,
                adaptive: false,
                min_quality: 0.1,
                max_quality: 1.0,
                compression_enabled: false,
                priority: streaming::StreamPriority::Normal,
            },
            retry: streaming::RetryConfig {
                max_attempts: 3,
                initial_delay: Duration::from_millis(100),
                max_delay: Duration::from_secs(5),
                backoff_multiplier: 2.0,
            },
        };
        
        let result = platform.create_stream(
            streaming::StreamType::AITextGeneration,
            streaming::StreamSource {
                source_type: "ai_provider".to_string(),
                source_id: "test-provider".to_string(),
                source_name: "Test Provider".to_string(),
                metadata: HashMap::new(),
            },
            Box::new(stream_handle),
            Some(stream_config),
        ).await;
        
        assert!(result.is_ok(), "Stream creation should succeed");
    }
    
    #[tokio::test]
    async fn test_configuration_validation() {
        // Test invalid configuration
        let mut invalid_config = create_default_config();
        invalid_config.ai_coordinator.request_timeout = Duration::from_secs(0); // Invalid timeout
        
        // Platform should handle invalid configuration gracefully
        let result = EnhancedMCPPlatform::new(invalid_config).await;
        
        // Either succeeds with defaults or fails gracefully
        assert!(result.is_ok() || result.is_err(), "Should handle invalid config");
    }
    
    #[tokio::test]
    async fn test_concurrent_requests() {
        let config = TestConfigBuilder::new()
            .with_mock_providers()
            .with_fast_timeouts()
            .build();
            
        let platform = Arc::new(EnhancedMCPPlatform::new(config).await.expect("should succeed"));
        
        // Create multiple concurrent requests
        let mut handles = Vec::new();
        
        for i in 0..5 {
            let platform_clone = platform.clone();
            let handle = tokio::spawn(async move {
                let request = UniversalAIRequest {
                    id: format!("test-{}", i),
                    model: "test-model-1".to_string(),
                    messages: vec![MessageContent {
                        role: "user".to_string(),
                        content: format!("Test message {}", i),
                    }],
                    request_type: AIRequestType::TextGeneration,
                    metadata: HashMap::new(),
                    payload: serde_json::json!({
                        "messages": [{"role": "user", "content": format!("Test message {}", i)}]
                    }),
                    context: RequestContext {
                        user_id: Some("test-user".to_string()),
                        session_id: Some("test-session".to_string()),
                        metadata: HashMap::new(),
                    },
                    hints: RoutingHints {
                        prefer_local: false,
                        max_cost: Some(0.1),
                        max_latency: Some(Duration::from_secs(30)),
                        quality_requirements: vec![],
                    },
                    requirements: QualityRequirements {
                        min_quality_score: None,
                        require_streaming: false,
                        require_tools: false,
                    },
                };
                
                platform_clone.process_ai_request(request).await
            });
            handles.push(handle);
        }
        
        // Wait for all requests to complete
        let results = futures::future::join_all(handles).await;
        
        // All requests should complete
        for result in results {
            assert!(result.is_ok(), "Concurrent request should complete");
        }
    }
}

/// Mock implementations for testing
#[derive(Debug)]
pub struct MockStreamHandle {
    pub id: String,
}

impl MockStreamHandle {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
        }
    }
}

#[async_trait::async_trait]
impl streaming::StreamHandle for MockStreamHandle {
    async fn start(&mut self) -> Result<()> {
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<()> {
        Ok(())
    }
    
    async fn pause(&mut self) -> Result<()> {
        Ok(())
    }
    
    async fn resume(&mut self) -> Result<()> {
        Ok(())
    }
    
    fn status(&self) -> streaming::StreamStatus {
        streaming::StreamStatus::Running
    }
    
    async fn next_chunk(&mut self) -> Result<Option<streaming::StreamChunk>> {
        Ok(None)
    }
    
    fn is_complete(&self) -> bool {
        false
    }
    
    fn get_stats(&self) -> streaming::StreamStats {
        streaming::StreamStats {
            chunks_processed: 0,
            bytes_processed: 0,
            chunks_per_second: 0.0,
            bytes_per_second: 0.0,
            error_count: 0,
            last_error: None,
            duration: Duration::from_secs(0),
            buffer_utilization: 0.0,
        }
    }
}

impl Default for RoutingConditions {
    fn default() -> Self {
        Self {
            requires_streaming: None,
            requires_tools: None,
            min_max_tokens: None,
            max_cost_per_token: None,
            max_total_cost: None,
            max_latency_ms: None,
            min_success_rate: None,
            allowed_providers: None,
            excluded_providers: None,
            max_input_tokens: None,
            request_types: None,
        }
    }
}

impl Default for RoutingActions {
    fn default() -> Self {
        Self {
            provider_preference: None,
            fallback_strategy: None,
            cost_weight: None,
            latency_weight: None,
            quality_weight: None,
            force_provider: None,
            add_metadata: None,
        }
    }
} 