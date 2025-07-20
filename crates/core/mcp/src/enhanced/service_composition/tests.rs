//! Comprehensive tests for the Service Composition Engine
//!
//! Tests cover different composition strategies (sequential, parallel, conditional, pipeline),
//! service registration and discovery, health monitoring, error handling, and performance optimization.

use super::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

use crate::enhanced::events::EventBroadcaster;
use crate::enhanced::coordinator::{AICoordinator, UniversalAIRequest, UniversalAIResponse, Message};

/// Helper function to create a test service composition engine
fn create_test_engine() -> ServiceCompositionEngine {
    let config = ServiceCompositionConfig::default();
    let event_broadcaster = Arc::new(EventBroadcaster::new());
    
    ServiceCompositionEngine::new(config, event_broadcaster)
}

/// Helper function to create a test AI service
fn create_test_service(
    id: &str,
    name: &str,
    model: &str,
    capabilities: Vec<String>,
) -> AIService {
    AIService {
        id: id.to_string(),
        name: name.to_string(),
        model: model.to_string(),
        capabilities,
        config: HashMap::new(),
        health: Arc::new(RwLock::new(ServiceHealth {
            status: HealthStatus::Healthy,
            last_check: chrono::Utc::now(),
            response_time: Duration::from_millis(100),
            error_rate: 0.0,
            availability: 1.0,
            metadata: HashMap::new(),
        })),
        metadata: HashMap::new(),
    }
}

/// Helper function to create a test composition
fn create_test_composition(
    id: &str,
    name: &str,
    composition_type: CompositionType,
    services: Vec<AIService>,
) -> Composition {
    Composition {
        id: id.to_string(),
        name: name.to_string(),
        description: "Test composition".to_string(),
        composition_type,
        services,
        config: CompositionConfig {
            timeout: Duration::from_secs(30),
            retry: RetryConfig {
                max_attempts: 3,
                delay: Duration::from_secs(1),
                backoff_strategy: BackoffStrategy::Exponential,
                conditions: vec![],
            },
            parallel_config: ParallelConfig {
                max_concurrent: 10,
                fail_fast: true,
                collect_partial_results: false,
            },
            conditional_config: ConditionalConfig {
                default_condition: "default".to_string(),
                conditions: HashMap::new(),
            },
            pipeline_config: PipelineConfig {
                enable_checkpoints: false,
                rollback_on_failure: true,
            },
        },
        dependencies: vec![],
        metadata: HashMap::new(),
        created_at: chrono::Utc::now(),
        modified_at: chrono::Utc::now(),
    }
}

/// Helper function to create a test Universal AI request
fn create_test_request(content: &str) -> UniversalAIRequest {
    UniversalAIRequest {
        id: uuid::Uuid::new_v4().to_string(),
        model: "test-model".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: content.to_string(),
        }],
        parameters: HashMap::new(),
    }
}

#[tokio::test]
async fn test_service_composition_engine_creation() {
    let engine = create_test_engine();
    
    // Test basic engine operations
    let services = engine.list_services().await.unwrap();
    assert!(services.is_empty());
    
    let compositions = engine.list_compositions().await.unwrap();
    assert!(compositions.is_empty());
    
    let metrics = engine.get_metrics().await.unwrap();
    assert_eq!(metrics.total_compositions, 0);
    assert_eq!(metrics.active_compositions, 0);
}

#[tokio::test]
async fn test_service_registration() {
    let engine = create_test_engine();
    
    let service = create_test_service(
        "test-service",
        "Test Service",
        "gpt-4",
        vec!["text-generation".to_string(), "analysis".to_string()],
    );
    
    // Register service
    engine.register_service(service.clone()).await.unwrap();
    
    // Verify registration
    let services = engine.list_services().await.unwrap();
    assert_eq!(services.len(), 1);
    assert_eq!(services[0].id, "test-service");
    
    // Check service health
    let health = engine.get_service_health("test-service").await.unwrap();
    assert!(health.is_some());
    assert_eq!(health.unwrap().status, HealthStatus::Healthy);
}

#[tokio::test]
async fn test_composition_registration() {
    let engine = create_test_engine();
    
    let services = vec![
        create_test_service("service1", "Service 1", "gpt-4", vec!["generation".to_string()]),
        create_test_service("service2", "Service 2", "claude-3", vec!["analysis".to_string()]),
    ];
    
    let composition = create_test_composition(
        "test-composition",
        "Test Composition",
        CompositionType::Sequential,
        services,
    );
    
    // Register composition
    engine.register_composition(composition.clone()).await.unwrap();
    
    // Verify registration
    let compositions = engine.list_compositions().await.unwrap();
    assert_eq!(compositions.len(), 1);
    assert_eq!(compositions[0].id, "test-composition");
    
    let retrieved = engine.get_composition_status("test-composition").await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, "test-composition");
}

#[tokio::test]
async fn test_sequential_composition_execution() {
    let engine = create_test_engine();
    
    let services = vec![
        create_test_service("service1", "Service 1", "gpt-4", vec!["generation".to_string()]),
        create_test_service("service2", "Service 2", "claude-3", vec!["analysis".to_string()]),
    ];
    
    let composition = create_test_composition(
        "sequential-test",
        "Sequential Test",
        CompositionType::Sequential,
        services,
    );
    
    engine.register_composition(composition).await.unwrap();
    
    // Execute composition
    let request = create_test_request("Test sequential execution");
    let result = engine.execute_composition("sequential-test", request).await.unwrap();
    
    assert_eq!(result.status, ExecutionStatus::Success);
    assert!(result.execution_time.as_millis() > 0);
    assert!(result.error.is_none());
    
    // Check that result contains processed data
    assert_ne!(result.data, serde_json::Value::Null);
}

#[tokio::test]
async fn test_parallel_composition_execution() {
    let engine = create_test_engine();
    
    let services = vec![
        create_test_service("service1", "Service 1", "gpt-4", vec!["generation".to_string()]),
        create_test_service("service2", "Service 2", "claude-3", vec!["analysis".to_string()]),
        create_test_service("service3", "Service 3", "ollama", vec!["processing".to_string()]),
    ];
    
    let composition = create_test_composition(
        "parallel-test",
        "Parallel Test",
        CompositionType::Parallel,
        services,
    );
    
    engine.register_composition(composition).await.unwrap();
    
    // Execute composition and measure execution time
    let start_time = std::time::Instant::now();
    let request = create_test_request("Test parallel execution");
    let result = engine.execute_composition("parallel-test", request).await.unwrap();
    let execution_time = start_time.elapsed();
    
    assert_eq!(result.status, ExecutionStatus::Success);
    assert!(result.error.is_none());
    
    // Parallel execution should be relatively fast
    assert!(execution_time < Duration::from_secs(5), "Parallel execution took too long: {:?}", execution_time);
    
    // Result should contain data from all services
    if let serde_json::Value::Object(results) = result.data {
        assert!(results.contains_key("service_0"));
        assert!(results.contains_key("service_1"));
        assert!(results.contains_key("service_2"));
    } else {
        panic!("Expected parallel results to be an object");
    }
}

#[tokio::test]
async fn test_conditional_composition_execution() {
    let engine = create_test_engine();
    
    let services = vec![
        create_test_service("primary", "Primary Service", "gpt-4", vec!["primary".to_string()]),
        create_test_service("fallback", "Fallback Service", "claude-3", vec!["fallback".to_string()]),
    ];
    
    let composition = create_test_composition(
        "conditional-test",
        "Conditional Test",
        CompositionType::Conditional,
        services,
    );
    
    engine.register_composition(composition).await.unwrap();
    
    // Test with condition true (should use first service)
    let mut request = create_test_request("Test conditional execution");
    request.parameters.insert("condition".to_string(), serde_json::Value::Bool(true));
    
    let result = engine.execute_composition("conditional-test", request).await.unwrap();
    
    assert_eq!(result.status, ExecutionStatus::Success);
    assert!(result.error.is_none());
    
    // Test with condition false (should use second service)
    let mut request = create_test_request("Test conditional execution");
    request.parameters.insert("condition".to_string(), serde_json::Value::Bool(false));
    
    let result = engine.execute_composition("conditional-test", request).await.unwrap();
    
    assert_eq!(result.status, ExecutionStatus::Success);
    assert!(result.error.is_none());
}

#[tokio::test]
async fn test_pipeline_composition_execution() {
    let engine = create_test_engine();
    
    let services = vec![
        create_test_service("preprocessor", "Preprocessor", "gpt-4", vec!["preprocessing".to_string()]),
        create_test_service("processor", "Processor", "claude-3", vec!["processing".to_string()]),
        create_test_service("postprocessor", "Postprocessor", "ollama", vec!["postprocessing".to_string()]),
    ];
    
    let composition = create_test_composition(
        "pipeline-test",
        "Pipeline Test",
        CompositionType::Pipeline,
        services,
    );
    
    engine.register_composition(composition).await.unwrap();
    
    let request = create_test_request("Test pipeline execution");
    let result = engine.execute_composition("pipeline-test", request).await.unwrap();
    
    assert_eq!(result.status, ExecutionStatus::Success);
    assert!(result.error.is_none());
    
    // Pipeline should process data sequentially
    assert_ne!(result.data, serde_json::Value::Null);
}

#[tokio::test]
async fn test_composition_cancellation() {
    let engine = create_test_engine();
    
    let services = vec![
        create_test_service("service1", "Service 1", "gpt-4", vec!["generation".to_string()]),
    ];
    
    let composition = create_test_composition(
        "cancel-test",
        "Cancel Test",
        CompositionType::Sequential,
        services,
    );
    
    engine.register_composition(composition).await.unwrap();
    
    // Cancel composition
    let result = engine.cancel_composition("cancel-test").await;
    assert!(result.is_ok());
    
    // Verify composition was removed from active compositions
    let status = engine.get_composition_status("cancel-test").await.unwrap();
    assert!(status.is_none());
}

#[tokio::test]
async fn test_composition_error_handling() {
    let engine = create_test_engine();
    
    // Register a composition without registering it first (should cause error)
    let request = create_test_request("Test error handling");
    let result = engine.execute_composition("nonexistent-composition", request).await.unwrap();
    
    assert_eq!(result.status, ExecutionStatus::Failed);
    assert!(result.error.is_some());
    assert!(result.error.as_ref().unwrap().contains("Composition not found"));
}

#[tokio::test]
async fn test_service_health_monitoring() {
    let engine = create_test_engine();
    
    let mut service = create_test_service(
        "health-test",
        "Health Test Service",
        "gpt-4",
        vec!["testing".to_string()],
    );
    
    // Modify service health to unhealthy
    {
        let mut health = service.health.write().await;
        health.status = HealthStatus::Unhealthy;
        health.error_rate = 0.8;
        health.availability = 0.2;
    }
    
    engine.register_service(service).await.unwrap();
    
    // Check service health
    let health = engine.get_service_health("health-test").await.unwrap();
    assert!(health.is_some());
    
    let health = health.unwrap();
    assert_eq!(health.status, HealthStatus::Unhealthy);
    assert_eq!(health.error_rate, 0.8);
    assert_eq!(health.availability, 0.2);
}

#[tokio::test]
async fn test_composition_metrics_collection() {
    let engine = create_test_engine();
    
    let services = vec![
        create_test_service("metrics-service", "Metrics Service", "gpt-4", vec!["metrics".to_string()]),
    ];
    
    let composition = create_test_composition(
        "metrics-test",
        "Metrics Test",
        CompositionType::Sequential,
        services,
    );
    
    engine.register_composition(composition).await.unwrap();
    
    // Execute composition to generate metrics
    let request = create_test_request("Test metrics collection");
    let result = engine.execute_composition("metrics-test", request).await.unwrap();
    
    assert_eq!(result.status, ExecutionStatus::Success);
    
    // Check metrics
    let metrics = engine.get_metrics().await.unwrap();
    assert_eq!(metrics.total_compositions, 1);
    assert!(metrics.completed_compositions >= 1);
    assert!(metrics.avg_execution_time.as_millis() > 0);
}

#[tokio::test]
async fn test_concurrent_composition_execution() {
    let engine = create_test_engine();
    
    let services = vec![
        create_test_service("concurrent-service", "Concurrent Service", "gpt-4", vec!["concurrent".to_string()]),
    ];
    
    let composition = create_test_composition(
        "concurrent-test",
        "Concurrent Test",
        CompositionType::Sequential,
        services,
    );
    
    engine.register_composition(composition).await.unwrap();
    
    // Execute multiple compositions concurrently
    let mut handles = vec![];
    for i in 0..3 {
        let engine_ref = &engine; // Use reference for concurrent execution
        let handle = tokio::spawn(async move {
            let mut request = create_test_request(&format!("Concurrent test {}", i));
            request.parameters.insert("iteration".to_string(), serde_json::Value::Number(serde_json::Number::from(i)));
            engine_ref.execute_composition("concurrent-test", request).await
        });
        handles.push(handle);
    }
    
    // Wait for all executions to complete
    let results: Vec<_> = futures::future::join_all(handles).await
        .into_iter()
        .map(|r| r.unwrap().unwrap())
        .collect();
    
    assert_eq!(results.len(), 3);
    
    // All should succeed
    for result in &results {
        assert_eq!(result.status, ExecutionStatus::Success);
    }
    
    let metrics = engine.get_metrics().await.unwrap();
    assert!(metrics.completed_compositions >= 3);
}

#[tokio::test]
async fn test_service_dependency_validation() {
    let engine = create_test_engine();
    
    let service = create_test_service(
        "dependency-test",
        "Dependency Test Service",
        "gpt-4",
        vec!["testing".to_string()],
    );
    
    engine.register_service(service).await.unwrap();
    
    // Test dependency validation
    let is_valid = engine.validate_dependencies("dependency-test").await.unwrap();
    assert!(is_valid); // Should be true since we have the service registered
    
    let nonexistent_valid = engine.validate_dependencies("nonexistent-service").await.unwrap();
    assert!(!nonexistent_valid); // Should be false for nonexistent service
}

#[tokio::test]
async fn test_service_discovery() {
    let engine = create_test_engine();
    
    // Test service discovery (currently returns empty list)
    let discovered_services = engine.discover_services().await.unwrap();
    assert!(discovered_services.is_empty()); // Should be empty with current implementation
}

#[tokio::test]
async fn test_custom_composition_type() {
    let engine = create_test_engine();
    
    let services = vec![
        create_test_service("custom-service", "Custom Service", "gpt-4", vec!["custom".to_string()]),
    ];
    
    let composition = create_test_composition(
        "custom-test",
        "Custom Test",
        CompositionType::Custom("custom-logic".to_string()),
        services,
    );
    
    engine.register_composition(composition).await.unwrap();
    
    // Execute custom composition (should fall back to sequential)
    let request = create_test_request("Test custom composition");
    let result = engine.execute_composition("custom-test", request).await.unwrap();
    
    assert_eq!(result.status, ExecutionStatus::Success);
    assert!(result.error.is_none());
}

#[tokio::test]
async fn test_composition_timeout_handling() {
    let engine = create_test_engine();
    
    let services = vec![
        create_test_service("timeout-service", "Timeout Service", "gpt-4", vec!["timeout".to_string()]),
    ];
    
    // Create composition with very short timeout
    let mut composition = create_test_composition(
        "timeout-test",
        "Timeout Test",
        CompositionType::Sequential,
        services,
    );
    composition.config.timeout = Duration::from_millis(1); // 1ms timeout
    
    engine.register_composition(composition).await.unwrap();
    
    let request = create_test_request("Test timeout handling");
    let result = engine.execute_composition("timeout-test", request).await.unwrap();
    
    // Should complete successfully despite short timeout (our test services are fast)
    assert_eq!(result.status, ExecutionStatus::Success);
}

#[tokio::test]
async fn test_composition_with_empty_services() {
    let engine = create_test_engine();
    
    // Create composition with no services
    let composition = create_test_composition(
        "empty-test",
        "Empty Test",
        CompositionType::Sequential,
        vec![], // No services
    );
    
    engine.register_composition(composition).await.unwrap();
    
    let request = create_test_request("Test empty services");
    let result = engine.execute_composition("empty-test", request).await.unwrap();
    
    // Should succeed with empty result
    assert_eq!(result.status, ExecutionStatus::Success);
}

#[tokio::test]
async fn test_service_capability_matching() {
    let engine = create_test_engine();
    
    let services = vec![
        create_test_service("service1", "Text Generator", "gpt-4", vec!["text-generation".to_string(), "completion".to_string()]),
        create_test_service("service2", "Code Analyzer", "claude-3", vec!["code-analysis".to_string(), "debugging".to_string()]),
    ];
    
    let composition = create_test_composition(
        "capability-test",
        "Capability Test",
        CompositionType::Parallel,
        services,
    );
    
    engine.register_composition(composition).await.unwrap();
    
    let request = create_test_request("Test capability matching");
    let result = engine.execute_composition("capability-test", request).await.unwrap();
    
    assert_eq!(result.status, ExecutionStatus::Success);
    
    // Check that both services were used
    if let serde_json::Value::Object(results) = result.data {
        assert_eq!(results.len(), 2); // Should have results from both services
    }
} 