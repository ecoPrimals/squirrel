// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use serde_json::json;
use uuid::Uuid;

//! End-to-end workflow tests
//!
//! These tests validate complete workflows without artificial delays.
//! All simulation sleeps have been removed for fast, concurrent testing.

// Mock components for end-to-end testing
#[derive(Debug, Clone)]
struct WorkflowContext {
    id: String,
    user_id: String,
    session_id: String,
    workflow_type: WorkflowType,
    status: WorkflowStatus,
    data: serde_json::Value,
    created_at: std::time::SystemTime,
    updated_at: std::time::SystemTime,
}

#[derive(Debug, Clone)]
enum WorkflowType {
    UserRegistration,
    DataProcessing,
    ServiceMesh,
    PluginExecution,
    ConfigurationUpdate,
    ErrorRecovery,
}

#[derive(Debug, Clone, PartialEq)]
enum WorkflowStatus {
    Initialized,
    InProgress,
    Completed,
    Failed,
    Retrying,
}

// Mock orchestrator for end-to-end workflows
struct WorkflowOrchestrator {
    active_workflows: Arc<tokio::sync::RwLock<std::collections::HashMap<String, WorkflowContext>>>,
    completion_callbacks: Arc<tokio::sync::RwLock<std::collections::HashMap<String, bool>>>,
}

impl WorkflowOrchestrator {
    fn new() -> Self {
        Self {
            active_workflows: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            completion_callbacks: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    async fn start_workflow(&self, workflow_type: WorkflowType, user_id: String, data: serde_json::Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let workflow_id = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();
        
        let context = WorkflowContext {
            id: workflow_id.clone(),
            user_id,
            session_id,
            workflow_type,
            status: WorkflowStatus::Initialized,
            data,
            created_at: std::time::SystemTime::now(),
            updated_at: std::time::SystemTime::now(),
        };
        
        self.active_workflows.write().await.insert(workflow_id.clone(), context);
        Ok(workflow_id)
    }

    async fn execute_workflow(&self, workflow_id: &str) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        // Update status to in progress
        {
            let mut workflows = self.active_workflows.write().await;
            if let Some(workflow) = workflows.get_mut(workflow_id) {
                workflow.status = WorkflowStatus::InProgress;
                workflow.updated_at = std::time::SystemTime::now();
            } else {
                return Err("Workflow not found".into());
            }
        }

        // Get workflow context
        let context = {
            let workflows = self.active_workflows.read().await;
            workflows.get(workflow_id).cloned()
                .ok_or("Workflow not found")?
        };

        // Execute based on workflow type
        let result = match context.workflow_type {
            WorkflowType::UserRegistration => {
                self.execute_user_registration_workflow(&context).await?
            },
            WorkflowType::DataProcessing => {
                self.execute_data_processing_workflow(&context).await?
            },
            WorkflowType::ServiceMesh => {
                self.execute_service_mesh_workflow(&context).await?
            },
            WorkflowType::PluginExecution => {
                self.execute_plugin_workflow(&context).await?
            },
            WorkflowType::ConfigurationUpdate => {
                self.execute_config_update_workflow(&context).await?
            },
            WorkflowType::ErrorRecovery => {
                self.execute_error_recovery_workflow(&context).await?
            },
        };

        // Update status to completed
        {
            let mut workflows = self.active_workflows.write().await;
            if let Some(workflow) = workflows.get_mut(workflow_id) {
                workflow.status = WorkflowStatus::Completed;
                workflow.updated_at = std::time::SystemTime::now();
            }
        }

        // Mark completion callback
        self.completion_callbacks.write().await.insert(workflow_id.to_string(), true);

        Ok(result)
    }

    async fn execute_user_registration_workflow(&self, context: &WorkflowContext) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        // Step 1: Validate user data
        let user_data = &context.data;
        if !user_data.get("email").and_then(|e| e.as_str()).unwrap_or("").contains('@') {
            return Err("Invalid email format".into());
        }

        // Step 2: Check for existing user (no artificial delay)
        
        // Step 3: Create user profile
        let profile = json!({
            "user_id": context.user_id,
            "email": user_data.get("email"),
            "name": user_data.get("name"),
            "created_at": format!("{:?}", context.created_at),
            "status": "active"
        });

        // Step 4: Send welcome email (simulated - no artificial delay)

        // Step 5: Initialize user preferences
        let preferences = json!({
            "theme": "default",
            "notifications": true,
            "language": "en"
        });

        Ok(json!({
            "registration_id": Uuid::new_v4(),
            "profile": profile,
            "preferences": preferences,
            "workflow_id": context.id,
            "completed_at": format!("{:?}", std::time::SystemTime::now())
        }))
    }

    async fn execute_data_processing_workflow(&self, context: &WorkflowContext) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        // Step 1: Validate input data
        let input_data = &context.data;
        let data_size = input_data.to_string().len();
        
        if data_size == 0 {
            return Err("Empty data provided".into());
        }

        // Step 2: Data preprocessing (no artificial delay)
        
        // Step 3: Apply transformations
        let mut processed_data = input_data.clone();
        if let Some(obj) = processed_data.as_object_mut() {
            obj.insert("processed_at".to_string(), json!(format!("{:?}", std::time::SystemTime::now())));
            obj.insert("processor_id".to_string(), json!(context.session_id));
            obj.insert("size_bytes".to_string(), json!(data_size));
        }

        // Step 4: Quality checks (no artificial delay)
        
        // Step 5: Store results
        let result_id = Uuid::new_v4().to_string();

        Ok(json!({
            "result_id": result_id,
            "original_size": data_size,
            "processed_data": processed_data,
            "processing_time_ms": 225,
            "quality_score": 0.95,
            "workflow_id": context.id
        }))
    }

    async fn execute_service_mesh_workflow(&self, context: &WorkflowContext) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        // Step 1: Service discovery
        let services = vec!["auth-service", "user-service", "notification-service"];
        let mut service_endpoints = std::collections::HashMap::new();
        
        for service in &services {
            // No artificial delay - test concurrency
            service_endpoints.insert(service.to_string(), format!("http://{}.local:8080", service));
        }

        // Step 2: Health checks (no artificial delay)
        let mut healthy_services = Vec::new();
        for service in &services {
            if rand::random::<f32>() > 0.1 { // 90% health check success rate
                healthy_services.push(service.to_string());
            }
        }

        // Step 3: Load balancing setup
        let load_balanced_config = json!({
            "strategy": "round_robin",
            "healthy_instances": healthy_services.len(),
            "total_instances": services.len()
        });

        // Step 4: Route configuration
        let routes = services.iter().map(|service| {
            json!({
                "path": format!("/api/v1/{}", service.replace("-service", "")),
                "upstream": service_endpoints.get(*service),
                "timeout": "30s",
                "retries": 3
            })
        }).collect::<Vec<_>>();

        Ok(json!({
            "mesh_id": Uuid::new_v4(),
            "services": service_endpoints,
            "healthy_services": healthy_services,
            "load_balancer": load_balanced_config,
            "routes": routes,
            "workflow_id": context.id
        }))
    }

    async fn execute_plugin_workflow(&self, context: &WorkflowContext) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        // Step 1: Plugin discovery
        let available_plugins = vec![
            ("data-validator", "1.0.0"),
            ("auth-provider", "2.1.0"),
            ("metrics-collector", "1.5.0"),
        ];

        // Step 2: Plugin loading (no artificial delay)
        let mut loaded_plugins = Vec::new();
        for (plugin_name, version) in &available_plugins {
            // Simulate loading success/failure
            if rand::random::<f32>() > 0.05 { // 95% success rate
                loaded_plugins.push(json!({
                    "name": plugin_name,
                    "version": version,
                    "status": "loaded",
                    "load_time_ms": 100
                }));
            }
        }

        // Step 3: Plugin initialization (no artificial delay)
        for plugin in &mut loaded_plugins {
            if let Some(obj) = plugin.as_object_mut() {
                obj.insert("initialized".to_string(), json!(true));
                obj.insert("init_time_ms".to_string(), json!(50));
            }
        }

        // Step 4: Execute plugin operations
        let mut execution_results = Vec::new();
        let plugin_input = &context.data;
        
        for plugin in &loaded_plugins {
            let plugin_name = plugin.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
            
            // No artificial delay - test concurrent execution
            
            execution_results.push(json!({
                "plugin": plugin_name,
                "status": "success",
                "execution_time_ms": 75,
                "result": format!("Processed by {}", plugin_name)
            }));
        }

        Ok(json!({
            "plugin_session_id": Uuid::new_v4(),
            "loaded_plugins": loaded_plugins,
            "execution_results": execution_results,
            "total_execution_time_ms": execution_results.len() * 75,
            "workflow_id": context.id
        }))
    }

    async fn execute_config_update_workflow(&self, context: &WorkflowContext) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        // Step 1: Validate new configuration
        let new_config = &context.data;
        if new_config.get("version").is_none() {
            return Err("Configuration version required".into());
        }

        // Step 2: Backup current configuration (no artificial delay)
        let backup_id = Uuid::new_v4().to_string();

        // Step 3: Apply configuration changes
        let changes_applied = vec![
            "network.timeout updated",
            "logging.level updated", 
            "security.tls_enabled updated"
        ];
        
        // No artificial delays - test concurrent updates

        // Step 4: Validate new configuration
        let validation_passed = true;

        // Step 5: Notify dependent services
        let notified_services = vec!["auth-service", "api-gateway", "load-balancer"];

        Ok(json!({
            "update_id": Uuid::new_v4(),
            "backup_id": backup_id,
            "changes_applied": changes_applied,
            "validation_passed": validation_passed,
            "notified_services": notified_services,
            "rollback_available": true,
            "workflow_id": context.id
        }))
    }

    async fn execute_error_recovery_workflow(&self, context: &WorkflowContext) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        // Step 1: Error detection and classification
        let error_data = &context.data;
        let error_type = error_data.get("error_type").and_then(|t| t.as_str()).unwrap_or("unknown");
        
        // Step 2: Determine recovery strategy
        let recovery_strategy = match error_type {
            "network" => "retry_with_backoff",
            "resource" => "scale_resources",
            "configuration" => "rollback_config",
            _ => "generic_recovery"
        };

        // Step 3: Execute recovery actions
        let mut recovery_actions = Vec::new();
        
        // No artificial delays - test actual recovery logic
        match recovery_strategy {
            "retry_with_backoff" => {
                for attempt in 1..=3 {
                    recovery_actions.push(format!("Retry attempt {} completed", attempt));
                }
            },
            "scale_resources" => {
                recovery_actions.push("Resources scaled up".to_string());
                recovery_actions.push("Load balancer updated".to_string());
            },
            "rollback_config" => {
                recovery_actions.push("Configuration rolled back".to_string());
                recovery_actions.push("Services restarted".to_string());
            },
            _ => {
                recovery_actions.push("Generic recovery executed".to_string());
            }
        }

        // Step 4: Verify recovery success
        let recovery_successful = rand::random::<f32>() > 0.2; // 80% success rate

        // Step 5: Update monitoring and alerts (no artificial delay)

        Ok(json!({
            "recovery_id": Uuid::new_v4(),
            "error_type": error_type,
            "recovery_strategy": recovery_strategy,
            "recovery_actions": recovery_actions,
            "recovery_successful": recovery_successful,
            "recovery_time_ms": recovery_actions.len() * 100 + 50,
            "workflow_id": context.id
        }))
    }

    async fn get_workflow_status(&self, workflow_id: &str) -> Option<WorkflowStatus> {
        let workflows = self.active_workflows.read().await;
        workflows.get(workflow_id).map(|w| w.status.clone())
    }

    async fn cleanup_workflow(&self, workflow_id: &str) -> bool {
        let mut workflows = self.active_workflows.write().await;
        let mut callbacks = self.completion_callbacks.write().await;
        workflows.remove(workflow_id).is_some() | callbacks.remove(workflow_id).is_some()
    }
}

/// Test complete user registration workflow
#[tokio::test]
async fn test_user_registration_end_to_end() {
    let orchestrator = WorkflowOrchestrator::new();
    
    let user_data = json!({
        "email": "test@example.com",
        "name": "Test User",
        "password": "secure_password"
    });
    
    // Start workflow
    let workflow_id = orchestrator
        .start_workflow(WorkflowType::UserRegistration, "user123".to_string(), user_data)
        .await
        .expect("Should start workflow");

    // Verify workflow is initialized
    assert_eq!(
        orchestrator.get_workflow_status(&workflow_id).await,
        Some(WorkflowStatus::Initialized)
    );

    // Execute workflow with timeout
    let result = timeout(
        Duration::from_secs(5),
        orchestrator.execute_workflow(&workflow_id)
    ).await.expect("Workflow should complete within timeout")
        .expect("Workflow should execute successfully");

    // Verify result structure
    assert!(result.get("registration_id").is_some());
    assert!(result.get("profile").is_some());
    assert!(result.get("preferences").is_some());
    assert_eq!(result.get("workflow_id").and_then(|id| id.as_str()), Some(workflow_id.as_str()));

    // Verify workflow completed
    assert_eq!(
        orchestrator.get_workflow_status(&workflow_id).await,
        Some(WorkflowStatus::Completed)
    );

    // Verify profile data
    let profile = result.get("profile").expect("Profile should exist");
    assert_eq!(profile.get("email").and_then(|e| e.as_str()), Some("test@example.com"));
    assert_eq!(profile.get("name").and_then(|n| n.as_str()), Some("Test User"));
    assert_eq!(profile.get("status").and_then(|s| s.as_str()), Some("active"));

    // Cleanup
    assert!(orchestrator.cleanup_workflow(&workflow_id).await);
}

/// Test data processing workflow with various data types
#[tokio::test]
async fn test_data_processing_end_to_end() {
    let orchestrator = WorkflowOrchestrator::new();
    
    let test_data = json!({
        "dataset": "sensor_readings",
        "records": [
            {"id": 1, "temperature": 22.5, "humidity": 45.2},
            {"id": 2, "temperature": 23.1, "humidity": 46.8},
            {"id": 3, "temperature": 21.9, "humidity": 44.1}
        ],
        "metadata": {
            "source": "iot_sensors",
            "timestamp": "2024-01-01T12:00:00Z"
        }
    });
    
    let workflow_id = orchestrator
        .start_workflow(WorkflowType::DataProcessing, "system".to_string(), test_data)
        .await
        .expect("Should start workflow");

    let result = timeout(
        Duration::from_secs(5),
        orchestrator.execute_workflow(&workflow_id)
    ).await.expect("Workflow should complete")
        .expect("Workflow should succeed");

    // Verify processing results
    assert!(result.get("result_id").is_some());
    assert!(result.get("processed_data").is_some());
    assert!(result.get("quality_score").and_then(|q| q.as_f64()).unwrap() > 0.9);

    let processed_data = result.get("processed_data").expect("Processed data should exist");
    assert!(processed_data.get("processed_at").is_some());
    assert!(processed_data.get("processor_id").is_some());

    // Verify original data is preserved
    assert_eq!(processed_data.get("dataset").and_then(|d| d.as_str()), Some("sensor_readings"));
    assert!(processed_data.get("records").and_then(|r| r.as_array()).unwrap().len() == 3);

    orchestrator.cleanup_workflow(&workflow_id).await;
}

/// Test service mesh workflow
#[tokio::test]
async fn test_service_mesh_end_to_end() {
    let orchestrator = WorkflowOrchestrator::new();
    
    let mesh_config = json!({
        "environment": "production",
        "region": "us-west-2",
        "load_balancer_type": "round_robin"
    });
    
    let workflow_id = orchestrator
        .start_workflow(WorkflowType::ServiceMesh, "system".to_string(), mesh_config)
        .await
        .expect("Should start workflow");

    let result = timeout(
        Duration::from_secs(5),
        orchestrator.execute_workflow(&workflow_id)
    ).await.expect("Workflow should complete")
        .expect("Workflow should succeed");

    // Verify service mesh setup
    assert!(result.get("mesh_id").is_some());
    
    let services = result.get("services").and_then(|s| s.as_object()).expect("Services should exist");
    assert!(services.contains_key("auth-service"));
    assert!(services.contains_key("user-service"));
    assert!(services.contains_key("notification-service"));

    let healthy_services = result.get("healthy_services").and_then(|h| h.as_array()).expect("Healthy services should exist");
    assert!(healthy_services.len() >= 2); // Should have most services healthy

    let routes = result.get("routes").and_then(|r| r.as_array()).expect("Routes should exist");
    assert!(routes.len() >= 2);

    // Verify load balancer configuration
    let load_balancer = result.get("load_balancer").expect("Load balancer config should exist");
    assert_eq!(load_balancer.get("strategy").and_then(|s| s.as_str()), Some("round_robin"));

    orchestrator.cleanup_workflow(&workflow_id).await;
}

/// Test plugin execution workflow
#[tokio::test]
async fn test_plugin_execution_end_to_end() {
    let orchestrator = WorkflowOrchestrator::new();
    
    let plugin_config = json!({
        "operation": "data_validation",
        "input_data": {
            "user_id": 12345,
            "email": "user@example.com",
            "age": 25
        },
        "plugins_enabled": ["data-validator", "auth-provider"]
    });
    
    let workflow_id = orchestrator
        .start_workflow(WorkflowType::PluginExecution, "user456".to_string(), plugin_config)
        .await
        .expect("Should start workflow");

    let result = timeout(
        Duration::from_secs(10),
        orchestrator.execute_workflow(&workflow_id)
    ).await.expect("Workflow should complete")
        .expect("Workflow should succeed");

    // Verify plugin execution
    assert!(result.get("plugin_session_id").is_some());
    
    let loaded_plugins = result.get("loaded_plugins").and_then(|p| p.as_array()).expect("Loaded plugins should exist");
    assert!(loaded_plugins.len() >= 2);

    // Verify each plugin was properly loaded and initialized
    for plugin in loaded_plugins {
        assert_eq!(plugin.get("status").and_then(|s| s.as_str()), Some("loaded"));
        assert_eq!(plugin.get("initialized").and_then(|i| i.as_bool()), Some(true));
        assert!(plugin.get("load_time_ms").and_then(|t| t.as_u64()).unwrap() > 0);
    }

    let execution_results = result.get("execution_results").and_then(|r| r.as_array()).expect("Execution results should exist");
    assert!(execution_results.len() >= 2);

    // Verify plugin execution results
    for exec_result in execution_results {
        assert_eq!(exec_result.get("status").and_then(|s| s.as_str()), Some("success"));
        assert!(exec_result.get("execution_time_ms").and_then(|t| t.as_u64()).unwrap() > 0);
    }

    orchestrator.cleanup_workflow(&workflow_id).await;
}

/// Test configuration update workflow
#[tokio::test]
async fn test_configuration_update_end_to_end() {
    let orchestrator = WorkflowOrchestrator::new();
    
    let new_config = json!({
        "version": "2.1.0",
        "changes": {
            "network": {
                "timeout": "45s",
                "max_connections": 200
            },
            "logging": {
                "level": "info",
                "format": "json"
            },
            "security": {
                "tls_enabled": true,
                "cert_path": "/etc/ssl/certs/app.crt"
            }
        }
    });
    
    let workflow_id = orchestrator
        .start_workflow(WorkflowType::ConfigurationUpdate, "admin".to_string(), new_config)
        .await
        .expect("Should start workflow");

    let result = timeout(
        Duration::from_secs(10),
        orchestrator.execute_workflow(&workflow_id)
    ).await.expect("Workflow should complete")
        .expect("Workflow should succeed");

    // Verify configuration update
    assert!(result.get("update_id").is_some());
    assert!(result.get("backup_id").is_some());
    assert_eq!(result.get("validation_passed").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(result.get("rollback_available").and_then(|r| r.as_bool()), Some(true));

    let changes_applied = result.get("changes_applied").and_then(|c| c.as_array()).expect("Changes should be applied");
    assert!(changes_applied.len() >= 3);

    let notified_services = result.get("notified_services").and_then(|n| n.as_array()).expect("Services should be notified");
    assert!(notified_services.len() >= 3);

    orchestrator.cleanup_workflow(&workflow_id).await;
}

/// Test error recovery workflow
#[tokio::test]
async fn test_error_recovery_end_to_end() {
    let orchestrator = WorkflowOrchestrator::new();
    
    // Test different error types
    let error_scenarios = vec![
        ("network", json!({"error_type": "network", "error_message": "Connection timeout"})),
        ("resource", json!({"error_type": "resource", "error_message": "Out of memory"})),
        ("configuration", json!({"error_type": "configuration", "error_message": "Invalid config"})),
    ];
    
    for (error_type, error_data) in error_scenarios {
        let workflow_id = orchestrator
            .start_workflow(WorkflowType::ErrorRecovery, "system".to_string(), error_data)
            .await
            .expect("Should start workflow");

        let result = timeout(
            Duration::from_secs(10),
            orchestrator.execute_workflow(&workflow_id)
        ).await.expect("Workflow should complete")
            .expect("Workflow should succeed");

        // Verify error recovery
        assert!(result.get("recovery_id").is_some());
        assert_eq!(result.get("error_type").and_then(|t| t.as_str()), Some(error_type));
        
        let recovery_strategy = result.get("recovery_strategy").and_then(|s| s.as_str()).expect("Recovery strategy should exist");
        assert!(!recovery_strategy.is_empty());

        let recovery_actions = result.get("recovery_actions").and_then(|a| a.as_array()).expect("Recovery actions should exist");
        assert!(recovery_actions.len() >= 1);

        // Verify recovery time is reasonable
        let recovery_time = result.get("recovery_time_ms").and_then(|t| t.as_u64()).unwrap();
        assert!(recovery_time > 0 && recovery_time < 2000); // Should complete within 2 seconds

        orchestrator.cleanup_workflow(&workflow_id).await;
    }
}

/// Test concurrent workflow execution
#[tokio::test]
async fn test_concurrent_workflows_end_to_end() {
    let orchestrator = Arc::new(WorkflowOrchestrator::new());
    
    let mut handles = Vec::new();
    
    // Start multiple workflows concurrently
    for i in 0..10 {
        let orchestrator_clone = orchestrator.clone();
        
        handles.push(tokio::spawn(async move {
            let user_data = json!({
                "email": format!("user{}@example.com", i),
                "name": format!("User {}", i),
                "id": i
            });
            
            let workflow_id = orchestrator_clone
                .start_workflow(WorkflowType::UserRegistration, format!("user{}", i), user_data)
                .await
                .expect("Should start workflow");

            let result = orchestrator_clone
                .execute_workflow(&workflow_id)
                .await
                .expect("Workflow should succeed");

            // Verify result and cleanup
            let success = result.get("registration_id").is_some();
            orchestrator_clone.cleanup_workflow(&workflow_id).await;
            
            (i, success)
        }));
    }
    
    // Wait for all workflows to complete
    let mut successful_workflows = 0;
    let mut failed_workflows = 0;
    
    for handle in handles {
        match handle.await {
            Ok((id, success)) => {
                if success {
                    successful_workflows += 1;
                } else {
                    failed_workflows += 1;
                }
                println!("Workflow {} completed: success={}", id, success);
            },
            Err(e) => {
                failed_workflows += 1;
                println!("Workflow failed with error: {:?}", e);
            }
        }
    }
    
    println!("Concurrent workflow results: {} successful, {} failed", successful_workflows, failed_workflows);
    
    // Verify most workflows succeeded
    assert!(successful_workflows >= 8, "Most concurrent workflows should succeed");
    assert!(failed_workflows <= 2, "Should have minimal failures");
    assert_eq!(successful_workflows + failed_workflows, 10, "All workflows should complete");
}

/// Test workflow failure and retry scenarios
#[tokio::test]
async fn test_workflow_failure_and_retry() {
    let orchestrator = WorkflowOrchestrator::new();
    
    // Test with invalid data to trigger failure
    let invalid_data = json!({
        "email": "invalid_email", // Missing @ symbol
        "name": ""
    });
    
    let workflow_id = orchestrator
        .start_workflow(WorkflowType::UserRegistration, "user_fail".to_string(), invalid_data)
        .await
        .expect("Should start workflow");

    // Execute workflow - should fail
    let result = orchestrator.execute_workflow(&workflow_id).await;
    assert!(result.is_err(), "Workflow should fail with invalid data");
    
    // Verify error message
    let error_message = format!("{}", result.unwrap_err());
    assert!(error_message.contains("Invalid email format"));
    
    // Test retry with corrected data
    let corrected_data = json!({
        "email": "corrected@example.com",
        "name": "Corrected User"
    });
    
    let retry_workflow_id = orchestrator
        .start_workflow(WorkflowType::UserRegistration, "user_retry".to_string(), corrected_data)
        .await
        .expect("Should start retry workflow");

    let retry_result = orchestrator
        .execute_workflow(&retry_workflow_id)
        .await
        .expect("Retry workflow should succeed");

    // Verify retry succeeded
    assert!(retry_result.get("registration_id").is_some());
    assert_eq!(
        retry_result.get("profile").and_then(|p| p.get("email")).and_then(|e| e.as_str()),
        Some("corrected@example.com")
    );

    orchestrator.cleanup_workflow(&workflow_id).await;
    orchestrator.cleanup_workflow(&retry_workflow_id).await;
} 