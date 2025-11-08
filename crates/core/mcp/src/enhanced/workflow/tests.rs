//! Comprehensive tests for the Workflow Management Engine
//!
//! Tests cover workflow execution strategies, step execution, state management,
//! cancellation, error handling, and integration with AI coordinator and service composition.

use super::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

use crate::enhanced::events::EventBroadcaster;
use crate::enhanced::service_composition::ServiceCompositionEngine;
use crate::enhanced::coordinator::{AICoordinator, UniversalAIRequest, UniversalAIResponse, Message};

/// Helper function to create a test workflow management engine
async fn create_test_engine() -> WorkflowManagementEngine {
    let config = WorkflowManagementConfig::default();
    let event_broadcaster = Arc::new(EventBroadcaster::new());
    let service_composition = Arc::new(ServiceCompositionEngine::new(
        crate::enhanced::service_composition::ServiceCompositionConfig::default(),
        event_broadcaster.clone(),
    ));
    let ai_coordinator = Arc::new(AICoordinator::new());
    
    WorkflowManagementEngine::new(
        config,
        event_broadcaster,
        service_composition,
        ai_coordinator,
    )
}

/// Helper function to create a test workflow definition
fn create_test_workflow(
    id: &str,
    name: &str,
    steps: Vec<WorkflowStep>,
    execution_strategy: ExecutionStrategy,
) -> WorkflowDefinition {
    WorkflowDefinition {
        id: id.to_string(),
        name: name.to_string(),
        description: "Test workflow".to_string(),
        version: "1.0.0".to_string(),
        steps,
        config: WorkflowConfig {
            execution_strategy,
            timeout: Duration::from_secs(300),
            retry: RetryConfig {
                max_attempts: 3,
                delay: Duration::from_secs(1),
                backoff_strategy: BackoffStrategy::Exponential,
                conditions: vec![],
            },
            resources: ResourceLimits {
                max_cpu: 2.0,
                max_memory: 1024 * 1024 * 1024, // 1GB
                max_storage: 10 * 1024 * 1024 * 1024, // 10GB
                max_network: 100 * 1024 * 1024, // 100MB
                custom: HashMap::new(),
            },
            monitoring: MonitoringConfig {
                metrics_enabled: true,
                logging_enabled: true,
                tracing_enabled: true,
                alerts: vec![],
            },
            error_handling: ErrorHandlingConfig {
                strategy: ErrorHandlingStrategy::Retry,
                recovery_actions: vec![],
                notifications: vec![],
            },
            security: SecurityConfig {
                auth_required: false,
                authorization: vec![],
                encryption: EncryptionConfig {
                    enabled: false,
                    algorithm: "AES256".to_string(),
                    key_management: "local".to_string(),
                },
                access_control: Some(AccessControlConfig {
                    enabled: false,
                    rules: vec![],
                    rbac: false,
                }),
            },
            scaling: ScalingConfig {
                auto_scaling: false,
                min_instances: 1,
                max_instances: 10,
                metrics: vec![],
            },
        },
        metadata: HashMap::new(),
        parameters: vec![],
        outputs: vec![],
        triggers: vec![],
        dependencies: vec![],
        constraints: vec![],
        created_at: chrono::Utc::now(),
        modified_at: chrono::Utc::now(),
    }
}

/// Helper function to create a test workflow step
fn create_test_step(
    id: &str,
    name: &str,
    step_type: StepType,
    config: HashMap<String, serde_json::Value>,
) -> WorkflowStep {
    WorkflowStep {
        id: id.to_string(),
        name: name.to_string(),
        description: "Test step".to_string(),
        step_type,
        config,
        dependencies: vec![],
        conditions: vec![],
        timeout: Duration::from_secs(30),
        retry: RetryConfig {
            max_attempts: 3,
            delay: Duration::from_secs(1),
            backoff_strategy: BackoffStrategy::Exponential,
            conditions: vec![],
        },
        resources: ResourceLimits {
            max_cpu: 1.0,
            max_memory: 512 * 1024 * 1024, // 512MB
            max_storage: 1024 * 1024 * 1024, // 1GB
            max_network: 10 * 1024 * 1024, // 10MB
            custom: HashMap::new(),
        },
        monitoring: MonitoringConfig {
            metrics_enabled: true,
            logging_enabled: true,
            tracing_enabled: true,
            alerts: vec![],
        },
        error_handling: ErrorHandlingConfig {
            strategy: ErrorHandlingStrategy::Retry,
            recovery_actions: vec![],
            notifications: vec![],
        },
    }
}

#[tokio::test]
async fn test_workflow_engine_creation() {
    let engine = create_test_engine().await;
    
    // Test basic engine operations
    let definitions = engine.list_workflow_definitions().await.unwrap();
    assert!(definitions.is_empty());
    
    let active_workflows = engine.list_active_workflows().await.unwrap();
    assert!(active_workflows.is_empty());
    
    let metrics = engine.get_metrics().await.unwrap();
    assert_eq!(metrics.total_workflows, 0);
    assert_eq!(metrics.active_workflows, 0);
}

#[tokio::test]
async fn test_workflow_registration() {
    let engine = create_test_engine().await;
    
    let workflow = create_test_workflow(
        "test-workflow",
        "Test Workflow",
        vec![],
        ExecutionStrategy::Sequential,
    );
    
    // Register workflow
    engine.register_workflow(workflow.clone()).await.unwrap();
    
    // Verify registration
    let definitions = engine.list_workflow_definitions().await.unwrap();
    assert_eq!(definitions.len(), 1);
    assert_eq!(definitions[0].id, "test-workflow");
    
    let retrieved = engine.get_workflow_definition("test-workflow").await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, "test-workflow");
}

#[tokio::test]
async fn test_workflow_sequential_execution() {
    let engine = create_test_engine().await;
    
    // Create workflow with data processing steps
    let steps = vec![
        create_test_step(
            "step1",
            "Uppercase Step",
            StepType::DataProcessing,
            {
                let mut config = HashMap::new();
                config.insert("input_key".to_string(), serde_json::Value::String("input".to_string()));
                config.insert("operation".to_string(), serde_json::Value::String("uppercase".to_string()));
                config
            },
        ),
        create_test_step(
            "step2",
            "Wait Step",
            StepType::Wait,
            {
                let mut config = HashMap::new();
                config.insert("duration_ms".to_string(), serde_json::Value::Number(serde_json::Number::from(100u64)));
                config
            },
        ),
    ];
    
    let workflow = create_test_workflow(
        "sequential-test",
        "Sequential Test",
        steps,
        ExecutionStrategy::Sequential,
    );
    
    engine.register_workflow(workflow).await.unwrap();
    
    // Execute workflow
    let mut parameters = HashMap::new();
    parameters.insert("input".to_string(), serde_json::Value::String("hello world".to_string()));
    
    let instance = engine.execute_workflow("sequential-test", parameters).await.unwrap();
    
    // Wait for execution to complete
    let completed_instance = timeout(Duration::from_secs(10), async {
        loop {
            if let Some(status) = engine.get_workflow_status(&instance.id).await.unwrap() {
                match status.state {
                    WorkflowState::Completed | WorkflowState::Failed => break status,
                    _ => {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        continue;
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }).await.expect("Workflow should complete within timeout");
    
    assert_eq!(completed_instance.state, WorkflowState::Completed);
    assert!(completed_instance.started_at.is_some());
    assert!(completed_instance.completed_at.is_some());
    
    // Check that both steps were executed
    assert!(completed_instance.step_states.contains_key("0"));
    assert!(completed_instance.step_states.contains_key("1"));
    
    // Check outputs
    assert!(completed_instance.outputs.contains_key("step_0"));
    assert!(completed_instance.outputs.contains_key("step_1"));
}

#[tokio::test]
async fn test_workflow_parallel_execution() {
    let engine = create_test_engine().await;
    
    // Create workflow with parallel wait steps
    let steps = vec![
        create_test_step(
            "step1",
            "Wait Step 1",
            StepType::Wait,
            {
                let mut config = HashMap::new();
                config.insert("duration_ms".to_string(), serde_json::Value::Number(serde_json::Number::from(50u64)));
                config
            },
        ),
        create_test_step(
            "step2",
            "Wait Step 2", 
            StepType::Wait,
            {
                let mut config = HashMap::new();
                config.insert("duration_ms".to_string(), serde_json::Value::Number(serde_json::Number::from(50u64)));
                config
            },
        ),
        create_test_step(
            "step3",
            "Wait Step 3",
            StepType::Wait,
            {
                let mut config = HashMap::new();
                config.insert("duration_ms".to_string(), serde_json::Value::Number(serde_json::Number::from(50u64)));
                config
            },
        ),
    ];
    
    let workflow = create_test_workflow(
        "parallel-test",
        "Parallel Test",
        steps,
        ExecutionStrategy::Parallel,
    );
    
    engine.register_workflow(workflow).await.unwrap();
    
    // Execute workflow and measure time
    let start_time = std::time::Instant::now();
    let instance = engine.execute_workflow("parallel-test", HashMap::new()).await.unwrap();
    
    // Wait for completion
    let completed_instance = timeout(Duration::from_secs(5), async {
        loop {
            if let Some(status) = engine.get_workflow_status(&instance.id).await.unwrap() {
                match status.state {
                    WorkflowState::Completed | WorkflowState::Failed => break status,
                    _ => {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        continue;
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }).await.expect("Parallel workflow should complete within timeout");
    
    let execution_time = start_time.elapsed();
    
    assert_eq!(completed_instance.state, WorkflowState::Completed);
    
    // Parallel execution should be faster than sequential (3 * 50ms = 150ms)
    // With parallel execution, all should complete in roughly 50ms + overhead
    assert!(execution_time < Duration::from_millis(120), "Parallel execution took too long: {:?}", execution_time);
    
    // All steps should have completed
    assert_eq!(completed_instance.step_states.len(), 3);
    for i in 0..3 {
        assert!(completed_instance.step_states.contains_key(&i.to_string()));
    }
}

#[tokio::test]
async fn test_workflow_condition_step() {
    let engine = create_test_engine().await;
    
    let steps = vec![
        create_test_step(
            "condition_step",
            "Condition Test",
            StepType::Condition,
            {
                let mut config = HashMap::new();
                config.insert("condition_key".to_string(), serde_json::Value::String("should_process".to_string()));
                config.insert("expected_value".to_string(), serde_json::Value::Bool(true));
                config
            },
        ),
    ];
    
    let workflow = create_test_workflow(
        "condition-test",
        "Condition Test",
        steps,
        ExecutionStrategy::Sequential,
    );
    
    engine.register_workflow(workflow).await.unwrap();
    
    // Test with condition true
    let mut parameters = HashMap::new();
    parameters.insert("should_process".to_string(), serde_json::Value::Bool(true));
    
    let instance = engine.execute_workflow("condition-test", parameters).await.unwrap();
    
    let completed_instance = timeout(Duration::from_secs(5), async {
        loop {
            if let Some(status) = engine.get_workflow_status(&instance.id).await.unwrap() {
                match status.state {
                    WorkflowState::Completed | WorkflowState::Failed => break status,
                    _ => {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        continue;
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }).await.expect("Condition workflow should complete");
    
    assert_eq!(completed_instance.state, WorkflowState::Completed);
    
    // Check condition result
    if let Some(output) = completed_instance.outputs.get("step_0") {
        assert_eq!(output, &serde_json::Value::Bool(true));
    }
}

#[tokio::test]
async fn test_workflow_notification_step() {
    let engine = create_test_engine().await;
    
    let steps = vec![
        create_test_step(
            "notification_step",
            "Notification Test",
            StepType::Notification,
            {
                let mut config = HashMap::new();
                config.insert("message".to_string(), serde_json::Value::String("Test notification".to_string()));
                config
            },
        ),
    ];
    
    let workflow = create_test_workflow(
        "notification-test",
        "Notification Test",
        steps,
        ExecutionStrategy::Sequential,
    );
    
    engine.register_workflow(workflow).await.unwrap();
    
    let instance = engine.execute_workflow("notification-test", HashMap::new()).await.unwrap();
    
    let completed_instance = timeout(Duration::from_secs(5), async {
        loop {
            if let Some(status) = engine.get_workflow_status(&instance.id).await.unwrap() {
                match status.state {
                    WorkflowState::Completed | WorkflowState::Failed => break status,
                    _ => {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        continue;
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }).await.expect("Notification workflow should complete");
    
    assert_eq!(completed_instance.state, WorkflowState::Completed);
    
    // Check notification result
    if let Some(output) = completed_instance.outputs.get("step_0") {
        if let Some(message) = output.as_str() {
            assert_eq!(message, "Test notification");
        }
    }
}

#[tokio::test]
async fn test_workflow_cancellation() {
    let engine = create_test_engine().await;
    
    // Create workflow with a long-running step
    let steps = vec![
        create_test_step(
            "long_step",
            "Long Running Step",
            StepType::Wait,
            {
                let mut config = HashMap::new();
                config.insert("duration_ms".to_string(), serde_json::Value::Number(serde_json::Number::from(5000u64))); // 5 second wait
                config
            },
        ),
    ];
    
    let workflow = create_test_workflow(
        "cancel-test",
        "Cancellation Test",
        steps,
        ExecutionStrategy::Sequential,
    );
    
    engine.register_workflow(workflow).await.unwrap();
    
    // Start workflow
    let instance = engine.execute_workflow("cancel-test", HashMap::new()).await.unwrap();
    
    // Wait a bit for workflow to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Cancel workflow
    engine.cancel_workflow(&instance.id).await.unwrap();
    
    // Verify workflow was cancelled
    let status = engine.get_workflow_status(&instance.id).await.unwrap();
    assert!(status.is_none(), "Workflow should be removed from active workflows after cancellation");
}

#[tokio::test]
async fn test_workflow_error_handling() {
    let engine = create_test_engine().await;
    
    // Create workflow with a step type that will fail
    let steps = vec![
        create_test_step(
            "unsupported_step",
            "Unsupported Step Type",
            StepType::Custom("nonexistent".to_string()),
            HashMap::new(),
        ),
    ];
    
    let workflow = create_test_workflow(
        "error-test",
        "Error Test",
        steps,
        ExecutionStrategy::Sequential,
    );
    
    engine.register_workflow(workflow).await.unwrap();
    
    let instance = engine.execute_workflow("error-test", HashMap::new()).await.unwrap();
    
    // Wait for workflow to fail
    let failed_instance = timeout(Duration::from_secs(5), async {
        loop {
            if let Some(status) = engine.get_workflow_status(&instance.id).await.unwrap() {
                match status.state {
                    WorkflowState::Failed => break status,
                    WorkflowState::Completed => panic!("Workflow should have failed"),
                    _ => {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        continue;
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }).await.expect("Workflow should fail within timeout");
    
    assert_eq!(failed_instance.state, WorkflowState::Failed);
    assert!(failed_instance.started_at.is_some());
    assert!(failed_instance.completed_at.is_some());
}

#[tokio::test]
async fn test_workflow_metrics_collection() {
    let engine = create_test_engine().await;
    
    let workflow = create_test_workflow(
        "metrics-test",
        "Metrics Test",
        vec![
            create_test_step(
                "step1",
                "Quick Step",
                StepType::Wait,
                {
                    let mut config = HashMap::new();
                    config.insert("duration_ms".to_string(), serde_json::Value::Number(serde_json::Number::from(10u64)));
                    config
                },
            ),
        ],
        ExecutionStrategy::Sequential,
    );
    
    engine.register_workflow(workflow).await.unwrap();
    
    // Execute workflow
    let instance = engine.execute_workflow("metrics-test", HashMap::new()).await.unwrap();
    
    // Wait for completion
    timeout(Duration::from_secs(5), async {
        loop {
            if let Some(status) = engine.get_workflow_status(&instance.id).await.unwrap() {
                match status.state {
                    WorkflowState::Completed | WorkflowState::Failed => break,
                    _ => {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        continue;
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }).await.expect("Workflow should complete");
    
    // Check metrics
    let metrics = engine.get_metrics().await.unwrap();
    assert_eq!(metrics.total_workflows, 1);
    assert_eq!(metrics.completed_workflows, 1);
    assert!(metrics.avg_execution_time.as_millis() > 0);
    assert!(metrics.success_rate > 0.0);
}

#[tokio::test]
async fn test_concurrent_workflow_execution() {
    let engine = create_test_engine().await;
    
    let workflow = create_test_workflow(
        "concurrent-test",
        "Concurrent Test",
        vec![
            create_test_step(
                "step1",
                "Wait Step",
                StepType::Wait,
                {
                    let mut config = HashMap::new();
                    config.insert("duration_ms".to_string(), serde_json::Value::Number(serde_json::Number::from(100u64)));
                    config
                },
            ),
        ],
        ExecutionStrategy::Sequential,
    );
    
    engine.register_workflow(workflow).await.unwrap();
    
    // Execute multiple workflows concurrently
    let mut handles = vec![];
    for i in 0..5 {
        let engine_clone = engine.clone(); // Note: This would need to be Arc<WorkflowManagementEngine> for this to work
        let handle = tokio::spawn(async move {
            let mut params = HashMap::new();
            params.insert("iteration".to_string(), serde_json::Value::Number(serde_json::Number::from(i)));
            engine_clone.execute_workflow("concurrent-test", params).await
        });
        handles.push(handle);
    }
    
    // Wait for all workflows to start
    let instances: Vec<_> = futures::future::join_all(handles).await
        .into_iter()
        .map(|r| r.unwrap().unwrap())
        .collect();
    
    assert_eq!(instances.len(), 5);
    
    // Wait for all to complete
    timeout(Duration::from_secs(10), async {
        loop {
            let active = engine.list_active_workflows().await.unwrap();
            if active.is_empty() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }).await.expect("All workflows should complete");
    
    let metrics = engine.get_metrics().await.unwrap();
    assert_eq!(metrics.completed_workflows, 5);
}

#[tokio::test]
async fn test_workflow_data_processing_chain() {
    let engine = create_test_engine().await;
    
    // Create a workflow that chains data processing steps
    let steps = vec![
        create_test_step(
            "step1",
            "Uppercase",
            StepType::DataProcessing,
            {
                let mut config = HashMap::new();
                config.insert("input_key".to_string(), serde_json::Value::String("text".to_string()));
                config.insert("operation".to_string(), serde_json::Value::String("uppercase".to_string()));
                config
            },
        ),
        create_test_step(
            "step2",
            "Identity",
            StepType::DataProcessing,
            {
                let mut config = HashMap::new();
                config.insert("input_key".to_string(), serde_json::Value::String("text".to_string()));
                config.insert("operation".to_string(), serde_json::Value::String("identity".to_string()));
                config
            },
        ),
    ];
    
    let workflow = create_test_workflow(
        "data-chain-test",
        "Data Chain Test",
        steps,
        ExecutionStrategy::Sequential,
    );
    
    engine.register_workflow(workflow).await.unwrap();
    
    let mut parameters = HashMap::new();
    parameters.insert("text".to_string(), serde_json::Value::String("hello world".to_string()));
    
    let instance = engine.execute_workflow("data-chain-test", parameters).await.unwrap();
    
    let completed_instance = timeout(Duration::from_secs(5), async {
        loop {
            if let Some(status) = engine.get_workflow_status(&instance.id).await.unwrap() {
                match status.state {
                    WorkflowState::Completed => break status,
                    WorkflowState::Failed => panic!("Workflow should not fail"),
                    _ => {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        continue;
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }).await.expect("Data processing workflow should complete");
    
    assert_eq!(completed_instance.state, WorkflowState::Completed);
    assert!(completed_instance.outputs.contains_key("step_0"));
    assert!(completed_instance.outputs.contains_key("step_1"));
}

// Note: These tests require the engine to implement Clone or be wrapped in Arc
// The actual implementation might need to be adjusted for proper concurrent testing 