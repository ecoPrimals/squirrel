//! AI Service Composition Engine
//!
//! This module provides comprehensive AI service composition capabilities including
//! service discovery, dependency management, orchestration, and integration for
//! building complex AI workflows with multiple services.
//!
//! This module has been refactored to use a modular structure. The main implementation
//! is now in the `service_composition` module, with types organized separately.

// Re-export everything from the service_composition module for backward compatibility
pub use super::service_composition::*;

// Additional backward compatibility exports
pub use super::service_composition::types::*;
pub use super::service_composition::ServiceCompositionEngine;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;
    
    #[tokio::test]
    async fn test_service_composition_engine_creation() {
        let config = ServiceCompositionConfig::default();
        let event_broadcaster = Arc::new(crate::events::EventBroadcaster::new());
        let ai_coordinator = Arc::new(crate::coordinator::AICoordinator::new());
        
        let _engine = ServiceCompositionEngine::new(
            config,
            event_broadcaster,
            ai_coordinator,
        );
    }
    
    #[tokio::test]
    async fn test_service_config_creation() {
        let config = ServiceConfig {
            service_type: ServiceType::Inference,
            endpoint: "http://localhost:8080".to_string(),
            auth: None,
            timeout: Duration::from_secs(30),
            retry: RetryConfig {
                max_attempts: 3,
                delay: Duration::from_secs(1),
                backoff_strategy: BackoffStrategy::Exponential,
                conditions: vec![],
            },
            resources: ResourceLimits::default(),
            scaling: ScalingConfig {
                auto_scaling: false,
                min_instances: 1,
                max_instances: 5,
                metrics: vec![],
            },
        };
        
        assert_eq!(config.service_type, ServiceType::Inference);
        assert_eq!(config.endpoint, "http://localhost:8080");
        assert_eq!(config.timeout, Duration::from_secs(30));
    }
    
    #[tokio::test]
    async fn test_service_capability_creation() {
        let capability = ServiceCapability {
            name: "text-generation".to_string(),
            description: "Generate text based on prompts".to_string(),
            parameters: serde_json::json!({
                "max_tokens": 1000,
                "temperature": 0.7
            }),
            constraints: vec![
                CapabilityConstraint {
                    constraint_type: ConstraintType::MaxInputSize,
                    value: serde_json::json!(10000),
                    description: "Maximum input size in characters".to_string(),
                }
            ],
            performance: Some(CapabilityPerformance {
                avg_latency: Duration::from_millis(500),
                throughput: 10.0,
                success_rate: 0.95,
                quality_score: 0.9,
                cost_per_request: 0.01,
            }),
        };
        
        assert_eq!(capability.name, "text-generation");
        assert_eq!(capability.description, "Generate text based on prompts");
        assert!(capability.performance.is_some());
        
        let perf = capability.performance.unwrap();
        assert_eq!(perf.avg_latency, Duration::from_millis(500));
        assert_eq!(perf.throughput, 10.0);
        assert_eq!(perf.success_rate, 0.95);
    }
    
    #[tokio::test]
    async fn test_composition_workflow_creation() {
        let workflow = CompositionWorkflow {
            steps: vec![
                CompositionStep {
                    id: "step1".to_string(),
                    name: "Preprocessing".to_string(),
                    service: "preprocessor".to_string(),
                    config: serde_json::json!({
                        "format": "json"
                    }),
                    dependencies: vec![],
                    conditions: vec![],
                },
                CompositionStep {
                    id: "step2".to_string(),
                    name: "Inference".to_string(),
                    service: "llm".to_string(),
                    config: serde_json::json!({
                        "model": "gpt-4"
                    }),
                    dependencies: vec!["step1".to_string()],
                    conditions: vec![],
                },
            ],
            dependencies: vec![],
            config: WorkflowConfig {
                execution_strategy: ExecutionStrategy::Sequential,
                timeout: Duration::from_secs(300),
                retry: RetryConfig {
                    max_attempts: 3,
                    delay: Duration::from_secs(1),
                    backoff_strategy: BackoffStrategy::Exponential,
                    conditions: vec![],
                },
                error_handling: ErrorHandlingConfig {
                    strategy: ErrorHandlingStrategy::Retry,
                    recovery_actions: vec![],
                    notifications: vec![],
                },
            },
        };
        
        assert_eq!(workflow.steps.len(), 2);
        assert_eq!(workflow.steps[0].name, "Preprocessing");
        assert_eq!(workflow.steps[1].name, "Inference");
        assert_eq!(workflow.steps[1].dependencies.len(), 1);
        assert_eq!(workflow.steps[1].dependencies[0], "step1");
    }
    
    #[tokio::test]
    async fn test_service_health_status() {
        let health = ServiceHealth {
            status: HealthStatus::Healthy,
            score: 0.95,
            last_check: chrono::Utc::now(),
            metrics: HealthMetrics {
                cpu_usage: 0.5,
                memory_usage: 0.3,
                disk_usage: 0.2,
                network_latency: Duration::from_millis(20),
                request_count: 1000,
                error_count: 5,
                success_rate: 0.995,
            },
            issues: vec![],
        };
        
        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.score, 0.95);
        assert_eq!(health.metrics.cpu_usage, 0.5);
        assert_eq!(health.metrics.success_rate, 0.995);
        assert_eq!(health.issues.len(), 0);
    }
    
    #[tokio::test]
    async fn test_execution_result() {
        let result = ExecutionResult {
            id: "exec-123".to_string(),
            status: ExecutionStatus::Success,
            data: serde_json::json!({
                "result": "Generated text response"
            }),
            metadata: HashMap::new(),
            execution_time: Duration::from_millis(800),
            error: None,
        };
        
        assert_eq!(result.id, "exec-123");
        assert_eq!(result.status, ExecutionStatus::Success);
        assert_eq!(result.execution_time, Duration::from_millis(800));
        assert!(result.error.is_none());
    }
} 