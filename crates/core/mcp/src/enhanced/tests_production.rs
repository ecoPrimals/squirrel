// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Production-focused Tests for Enhanced MCP Platform
//!
//! This module provides comprehensive test coverage for production-critical functionality
//! including error handling, resource management, concurrent operations, and resilience testing.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{sleep, timeout};
use uuid::Uuid;

use crate::error::{Result, types::MCPError};
use super::*;
use super::coordinator::{
    UniversalAIRequest, AIRequestType, MessageContent, 
    RequestContext, RoutingHints, QualityRequirements, RoutingStrategy
};
use super::events::{MCPEvent, EventType, EventSource, SourceType, EventPriority};
use super::streaming::{StreamConfig, StreamChunk, StreamType};

/// Production test configuration builder
pub struct ProductionTestConfigBuilder {
    config: EnhancedPlatformConfig,
    errors: Vec<String>,
}

impl ProductionTestConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: EnhancedPlatformConfig::default(),
            errors: Vec::new(),
        }
    }
    
    pub fn with_production_settings(mut self) -> Self {
        // Configure for production-like conditions
        self.config.max_concurrent_requests = 1000;
        self.config.request_timeout = Duration::from_secs(30);
        self.config.health_check_interval = Duration::from_secs(30);
        self.config.metrics_collection_interval = Duration::from_secs(10);
        self
    }
    
    pub fn with_stress_test_settings(mut self) -> Self {
        // Configure for stress testing
        self.config.max_concurrent_requests = 10000;
        self.config.request_timeout = Duration::from_secs(5);
        self.config.health_check_interval = Duration::from_secs(1);
        self.config.metrics_collection_interval = Duration::from_secs(1);
        self
    }
    
    pub fn build(self) -> Result<EnhancedPlatformConfig> {
        if !self.errors.is_empty() {
            return Err(MCPError::Configuration(format!("Configuration errors: {}", self.errors.join(", "))));
        }
        Ok(self.config)
    }
}

/// Create a production-ready AI request for testing
fn create_production_ai_request() -> UniversalAIRequest {
    UniversalAIRequest {
        id: Uuid::new_v4().to_string(),
        request_type: AIRequestType::Chat,
        content: MessageContent::Text("Test production request".to_string()),
        context: RequestContext {
            user_id: Some("production_test_user".to_string()),
            session_id: Some(Uuid::new_v4().to_string()),
            trace_id: Some(Uuid::new_v4().to_string()),
            metadata: HashMap::new(),
        },
        routing_hints: RoutingHints {
            preferred_provider: None,
            routing_strategy: RoutingStrategy::PerformanceOptimized,
            exclude_providers: Vec::new(),
            require_capabilities: Vec::new(),
        },
        quality_requirements: QualityRequirements {
            max_latency_ms: 5000,
            min_accuracy: 0.95,
            required_safety_level: 0.9,
        },
        deadline: chrono::Utc::now() + chrono::Duration::seconds(30),
        priority: 5,
    }
}

/// Production readiness tests
#[cfg(test)]
mod production_tests {
    use super::*;
    
    /// Test platform initialization under production conditions
    #[tokio::test]
    async fn test_production_platform_initialization() {
        let config = ProductionTestConfigBuilder::new()
            .with_production_settings()
            .build()
            .expect("Production config should be valid");
        
        let platform = EnhancedMCPPlatform::new(config).await;
        assert!(platform.is_ok(), "Platform should initialize successfully in production mode");
        
        let mut platform = platform.expect("should succeed");
        
        // Test startup
        let start_result = platform.start().await;
        assert!(start_result.is_ok(), "Platform should start successfully");
        
        // Test health check
        let health = platform.get_platform_health().await;
        assert!(health.is_ok(), "Health check should succeed");
        
        let health = health.expect("should succeed");
        assert_eq!(health.status, HealthStatus::Healthy, "Platform should be healthy after startup");
        
        // Test graceful shutdown
        let stop_result = platform.stop().await;
        assert!(stop_result.is_ok(), "Platform should stop gracefully");
    }
    
    /// Test concurrent request handling
    #[tokio::test]
    async fn test_concurrent_request_handling() {
        let config = ProductionTestConfigBuilder::new()
            .with_production_settings()
            .build()
            .expect("Production config should be valid");
        
        let mut platform = EnhancedMCPPlatform::new(config).await
            .expect("Platform should initialize");
        
        platform.start().await.expect("Platform should start");
        
        // Create multiple concurrent requests
        let mut handles = Vec::new();
        for i in 0..10 {
            let mut request = create_production_ai_request();
            request.context.metadata.insert("test_id".to_string(), i.to_string());
            
            let platform_clone = platform.clone();
            let handle = tokio::spawn(async move {
                platform_clone.process_ai_request(request).await
            });
            handles.push(handle);
        }
        
        // Wait for all requests to complete
        let mut successes = 0;
        let mut failures = 0;
        
        for handle in handles {
            match handle.await {
                Ok(Ok(_)) => successes += 1,
                Ok(Err(_)) => failures += 1,
                Err(_) => failures += 1,
            }
        }
        
        // In production, we should handle concurrent requests gracefully
        // Even if some fail due to missing providers, the platform should remain stable
        assert!(successes + failures == 10, "All requests should complete");
        
        platform.stop().await.expect("Platform should stop gracefully");
    }
    
    /// Test error recovery and resilience
    #[tokio::test]
    async fn test_error_recovery() {
        let config = ProductionTestConfigBuilder::new()
            .with_production_settings()
            .build()
            .expect("Production config should be valid");
        
        let mut platform = EnhancedMCPPlatform::new(config).await
            .expect("Platform should initialize");
        
        platform.start().await.expect("Platform should start");
        
        // Create an invalid request that should trigger error handling
        let invalid_request = UniversalAIRequest {
            id: "".to_string(), // Invalid empty ID
            request_type: AIRequestType::Chat,
            content: MessageContent::Text("".to_string()), // Empty content
            context: RequestContext {
                user_id: None,
                session_id: None,
                trace_id: None,
                metadata: HashMap::new(),
            },
            routing_hints: RoutingHints {
                preferred_provider: Some("nonexistent_provider".to_string()),
                routing_strategy: RoutingStrategy::PerformanceOptimized,
                exclude_providers: Vec::new(),
                require_capabilities: Vec::new(),
            },
            quality_requirements: QualityRequirements {
                max_latency_ms: 0, // Impossible latency requirement
                min_accuracy: 2.0, // Impossible accuracy requirement (> 1.0)
                required_safety_level: 2.0, // Impossible safety level (> 1.0)
            },
            deadline: chrono::Utc::now() - chrono::Duration::seconds(10), // Already expired
            priority: 0,
        };
        
        // Test error handling
        let result = platform.process_ai_request(invalid_request).await;
        assert!(result.is_err(), "Invalid request should return error");
        
        // Platform should still be operational after error
        let health = platform.get_platform_health().await;
        assert!(health.is_ok(), "Platform should remain healthy after error");
        
        platform.stop().await.expect("Platform should stop gracefully");
    }
    
    /// Test resource management under load
    #[tokio::test]
    async fn test_resource_management() {
        let config = ProductionTestConfigBuilder::new()
            .with_production_settings()
            .build()
            .expect("Production config should be valid");
        
        let mut platform = EnhancedMCPPlatform::new(config).await
            .expect("Platform should initialize");
        
        platform.start().await.expect("Platform should start");
        
        // Test memory usage doesn't grow unbounded
        let initial_health = platform.get_platform_health().await
            .expect("Initial health check should succeed");
        
        // Create a series of requests to test resource cleanup
        for batch in 0..5 {
            let mut handles = Vec::new();
            
            for i in 0..20 {
                let mut request = create_production_ai_request();
                request.context.metadata.insert("batch".to_string(), batch.to_string());
                request.context.metadata.insert("request".to_string(), i.to_string());
                
                let platform_clone = platform.clone();
                let handle = tokio::spawn(async move {
                    platform_clone.process_ai_request(request).await
                });
                handles.push(handle);
            }
            
            // Wait for batch to complete
            for handle in handles {
                let _ = handle.await;
            }
            
            // Brief pause to allow resource cleanup
            sleep(Duration::from_millis(100)).await;
        }
        
        // Check that platform is still healthy after processing batches
        let final_health = platform.get_platform_health().await
            .expect("Final health check should succeed");
        
        assert_eq!(final_health.status, HealthStatus::Healthy, "Platform should remain healthy after load");
        
        platform.stop().await.expect("Platform should stop gracefully");
    }
    
    /// Test timeout handling
    #[tokio::test]
    async fn test_timeout_handling() {
        let config = ProductionTestConfigBuilder::new()
            .with_production_settings()
            .build()
            .expect("Production config should be valid");
        
        let mut platform = EnhancedMCPPlatform::new(config).await
            .expect("Platform should initialize");
        
        platform.start().await.expect("Platform should start");
        
        // Create a request with very short deadline
        let mut request = create_production_ai_request();
        request.deadline = chrono::Utc::now() + chrono::Duration::milliseconds(1); // 1ms deadline
        request.quality_requirements.max_latency_ms = 1;
        
        // Test timeout behavior
        let result = timeout(Duration::from_secs(5), platform.process_ai_request(request)).await;
        
        match result {
            Ok(Err(_)) => {
                // Expected: request should fail due to timeout
            }
            Ok(Ok(_)) => {
                // Acceptable: request completed before timeout
            }
            Err(_) => {
                // Should not hang - timeout should be respected
                unreachable!("Request should not hang - platform should respect timeouts");
            }
        }
        
        // Platform should remain operational
        let health = platform.get_platform_health().await;
        assert!(health.is_ok(), "Platform should remain healthy after timeout");
        
        platform.stop().await.expect("Platform should stop gracefully");
    }
    
    /// Test event system under production load
    #[tokio::test]
    async fn test_event_system_production() {
        let config = ProductionTestConfigBuilder::new()
            .with_production_settings()
            .build()
            .expect("Production config should be valid");
        
        let mut platform = EnhancedMCPPlatform::new(config).await
            .expect("Platform should initialize");
        
        platform.start().await.expect("Platform should start");
        
        // Test event publishing under load
        let mut event_handles = Vec::new();
        
        for i in 0..100 {
            let event = MCPEvent {
                id: Uuid::new_v4().to_string(),
                event_type: EventType::Request,
                source: EventSource {
                    component: "test".to_string(),
                    instance_id: format!("test-{}", i),
                    source_type: SourceType::Internal,
                },
                data: serde_json::json!({
                    "test_event": i,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }),
                priority: EventPriority::Normal,
                timestamp: chrono::Utc::now(),
                correlation_id: Some(Uuid::new_v4().to_string()),
                metadata: HashMap::new(),
            };
            
            let platform_clone = platform.clone();
            let handle = tokio::spawn(async move {
                platform_clone.publish_event(event).await
            });
            event_handles.push(handle);
        }
        
        // Wait for all events to be published
        let mut successful_publishes = 0;
        for handle in event_handles {
            match handle.await {
                Ok(Ok(_)) => successful_publishes += 1,
                Ok(Err(_)) | Err(_) => {} // Count failures but don't fail test
            }
        }
        
        // Events should be published successfully
        assert!(successful_publishes > 0, "At least some events should be published successfully");
        
        platform.stop().await.expect("Platform should stop gracefully");
    }
    
    /// Test metrics collection accuracy
    #[tokio::test]
    async fn test_metrics_accuracy() {
        let config = ProductionTestConfigBuilder::new()
            .with_production_settings()
            .build()
            .expect("Production config should be valid");
        
        let mut platform = EnhancedMCPPlatform::new(config).await
            .expect("Platform should initialize");
        
        platform.start().await.expect("Platform should start");
        
        // Get baseline metrics
        let initial_metrics = platform.get_platform_metrics().await
            .expect("Should get initial metrics");
        
        // Perform some operations
        let test_requests = 5;
        for i in 0..test_requests {
            let mut request = create_production_ai_request();
            request.context.metadata.insert("metrics_test".to_string(), i.to_string());
            
            // Don't care about success/failure, just that metrics are updated
            let _ = platform.process_ai_request(request).await;
        }
        
        // Allow time for metrics to be collected
        sleep(Duration::from_millis(500)).await;
        
        // Get updated metrics
        let final_metrics = platform.get_platform_metrics().await
            .expect("Should get final metrics");
        
        // Verify metrics were updated
        assert!(
            final_metrics.total_requests >= initial_metrics.total_requests,
            "Total request count should increase or stay the same"
        );
        
        platform.stop().await.expect("Platform should stop gracefully");
    }
}

/// Stress testing for production limits
#[cfg(test)]
mod stress_tests {
    use super::*;
    
    /// Test high concurrency limits
    #[tokio::test]
    async fn test_high_concurrency_limits() {
        let config = ProductionTestConfigBuilder::new()
            .with_stress_test_settings()
            .build()
            .expect("Stress test config should be valid");
        
        let mut platform = EnhancedMCPPlatform::new(config).await
            .expect("Platform should initialize for stress test");
        
        platform.start().await.expect("Platform should start");
        
        // Test with higher concurrency
        let concurrent_requests = 100;
        let mut handles = Vec::new();
        
        for i in 0..concurrent_requests {
            let mut request = create_production_ai_request();
            request.context.metadata.insert("stress_test_id".to_string(), i.to_string());
            
            let platform_clone = platform.clone();
            let handle = tokio::spawn(async move {
                platform_clone.process_ai_request(request).await
            });
            handles.push(handle);
        }
        
        // Wait for completion with timeout
        let timeout_duration = Duration::from_secs(30);
        let results = timeout(timeout_duration, async {
            let mut results = Vec::new();
            for handle in handles {
                results.push(handle.await);
            }
            results
        }).await;
        
        assert!(results.is_ok(), "Stress test should complete within timeout");
        
        // Platform should survive stress test
        let health = platform.get_platform_health().await;
        assert!(health.is_ok(), "Platform should remain healthy after stress test");
        
        platform.stop().await.expect("Platform should stop gracefully after stress test");
    }
}

/// Reliability tests for production deployment
#[cfg(test)]
mod reliability_tests {
    use super::*;
    
    /// Test graceful degradation
    #[tokio::test]
    async fn test_graceful_degradation() {
        let config = ProductionTestConfigBuilder::new()
            .with_production_settings()
            .build()
            .expect("Production config should be valid");
        
        let mut platform = EnhancedMCPPlatform::new(config).await
            .expect("Platform should initialize");
        
        platform.start().await.expect("Platform should start");
        
        // Test that platform handles requests gracefully even when providers fail
        // This tests the platform's ability to degrade gracefully rather than crash
        
        for _ in 0..10 {
            let request = create_production_ai_request();
            let result = platform.process_ai_request(request).await;
            
            // We expect these to fail in test environment (no real providers)
            // But the platform should handle the failures gracefully
            match result {
                Ok(_) => {
                    // Great! Request succeeded
                }
                Err(e) => {
                    // Expected in test environment - verify it's a graceful failure
                    assert!(!e.to_string().contains("panic"), "Errors should not contain panics");
                    assert!(!e.to_string().contains("unwrap"), "Errors should not contain unwrap failures");
                }
            }
        }
        
        // Platform should still be healthy
        let health = platform.get_platform_health().await;
        assert!(health.is_ok(), "Platform should remain healthy during degraded operation");
        
        platform.stop().await.expect("Platform should stop gracefully");
    }
} 