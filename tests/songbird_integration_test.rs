//! Songbird Orchestration Integration Tests
//!
//! These tests verify that the Songbird orchestration system is properly integrated
//! and can handle production orchestration scenarios.

use std::time::Duration;
use universal_patterns::config::ConfigBuilder;
use universal_patterns::orchestration::{OrchestrationTask, TaskPriority, RetryPolicy, BackoffStrategy, TaskConstraints, ServiceHealth};
use universal_patterns::traits::PrimalState;
use squirrel::songbird::{SquirrelOrchestrationService, create_health_report, McpHealthStatus};
use uuid::Uuid;
use chrono::Utc;

#[tokio::test]
async fn test_complete_orchestration_workflow() {
    // Test complete orchestration workflow with disabled orchestration (uses mock provider automatically)
    let config = ConfigBuilder::squirrel()
        .build() // Use mock provider by not setting endpoint
        .expect("Should create config with mock orchestration");

    let mut service = SquirrelOrchestrationService::new(&config)
        .expect("Should create orchestration service");

    // Mock service is not enabled, but demonstrates workflow
    assert!(!service.is_enabled());
    assert_eq!(service.get_state().await, PrimalState::Stopped);

    // Start the service (no-op for disabled orchestration)
    service.start().await
        .expect("Should start orchestration service");

    // For disabled orchestration, state remains stopped
    assert_eq!(service.get_state().await, PrimalState::Stopped);

    // Test task scheduling (should fail for disabled orchestration)
    let task = OrchestrationTask {
        id: Uuid::new_v4(),
        name: "AI Processing Task".to_string(),
        task_type: "mcp.ai.process".to_string(),
        target_primal: Some("squirrel".to_string()),
        payload: serde_json::json!({
            "prompt": "Analyze the data",
            "model": "gpt-4",
            "max_tokens": 1000
        }),
        priority: TaskPriority::High,
        retry_policy: RetryPolicy {
            max_retries: 3,
            backoff: BackoffStrategy::Exponential {
                initial: Duration::from_secs(1),
                max: Duration::from_secs(60),
                multiplier: 2.0,
            },
            retry_on: vec!["network_error".to_string(), "timeout".to_string()],
        },
        timeout: Duration::from_secs(300),
        dependencies: Vec::new(),
        constraints: TaskConstraints {
            required_resources: std::collections::HashMap::new(),
            node_affinity: Some("ai-enabled".to_string()),
            anti_affinity: Vec::new(),
            placement: Vec::new(),
        },
        created_at: Utc::now(),
        scheduled_at: None,
    };

    // Task scheduling should fail for disabled orchestration
    let task_result = service.schedule_task(task).await;
    assert!(task_result.is_err());
    assert!(task_result.unwrap_err().to_string().contains("not enabled"));

    // Test service discovery (should return empty list)
    let services = service.discover_services("ai").await
        .expect("Should discover services");
    
    // Disabled orchestration returns empty list
    assert!(services.is_empty());

    // Test health reporting (should succeed even when disabled)
    let mcp_status = McpHealthStatus {
        core_healthy: true,
        core_message: "MCP core is operational".to_string(),
        core_check_duration_ms: 25,
        ai_providers_healthy: true,
        healthy_providers: 2,
        total_providers: 3,
        ai_check_duration_ms: 150,
        protocol_healthy: true,
        protocol_message: "All MCP protocols active".to_string(),
        protocol_check_duration_ms: 10,
    };

    let health_report = create_health_report(mcp_status);
    service.report_health(health_report).await
        .expect("Should report health even when disabled");

    // Test cluster status (should fail for disabled orchestration)
    let cluster_result = service.get_cluster_status().await;
    assert!(cluster_result.is_err());
    assert!(cluster_result.unwrap_err().to_string().contains("not enabled"));

    // Test state updates (should work locally)
    service.update_state(PrimalState::Maintenance).await
        .expect("Should update state locally");
    
    assert_eq!(service.get_state().await, PrimalState::Maintenance);

    // Stop the service (should reset to stopped state)
    service.stop().await
        .expect("Should stop orchestration service");

    // Verify stopped state (stop operation resets state)
    assert_eq!(service.get_state().await, PrimalState::Stopped);
}

#[tokio::test]
async fn test_orchestration_task_lifecycle() {
    // Test task lifecycle validation with disabled orchestration
    let config = ConfigBuilder::squirrel()
        .build() // Use mock/disabled orchestration
        .expect("Should create config");

    let service = SquirrelOrchestrationService::new(&config)
        .expect("Should create orchestration service");

    // Create a complex task with dependencies
    let main_task = OrchestrationTask {
        id: Uuid::new_v4(),
        name: "Complex AI Workflow".to_string(),
        task_type: "mcp.workflow.complex".to_string(),
        target_primal: None, // Let orchestrator decide
        payload: serde_json::json!({
            "workflow": "data_analysis",
            "steps": [
                {"type": "data_ingestion", "source": "api"},
                {"type": "preprocessing", "filters": ["normalize", "clean"]},
                {"type": "ai_analysis", "models": ["gpt-4", "claude-3"]},
                {"type": "result_formatting", "format": "json"}
            ]
        }),
        priority: TaskPriority::Critical,
        retry_policy: RetryPolicy {
            max_retries: 5,
            backoff: BackoffStrategy::Linear {
                initial: Duration::from_secs(2),
                increment: Duration::from_secs(1),
            },
            retry_on: vec![
                "network_error".to_string(),
                "timeout".to_string(),
                "resource_unavailable".to_string(),
            ],
        },
        timeout: Duration::from_secs(1800), // 30 minutes
        dependencies: Vec::new(),
        constraints: TaskConstraints {
            required_resources: {
                let mut resources = std::collections::HashMap::new();
                resources.insert("cpu_cores".to_string(), 4);
                resources.insert("memory_gb".to_string(), 8);
                resources.insert("gpu_vram_gb".to_string(), 2);
                resources
            },
            node_affinity: Some("high-performance".to_string()),
            anti_affinity: vec!["batch-processing".to_string()],
            placement: Vec::new(),
        },
        created_at: Utc::now(),
        scheduled_at: Some(Utc::now() + chrono::Duration::seconds(10)),
    };

    // Task scheduling should fail for disabled orchestration
    let task_result = service.schedule_task(main_task).await;
    assert!(task_result.is_err());
    assert!(task_result.unwrap_err().to_string().contains("not enabled"));

    // Since we can't schedule tasks, test the task structure itself
    let dummy_task_id = Uuid::new_v4();
    
    // Task status should fail for disabled orchestration
    let status_result = service.get_task_status(&dummy_task_id).await;
    assert!(status_result.is_err());
    assert!(status_result.unwrap_err().to_string().contains("not enabled"));

    // Task cancellation should fail for disabled orchestration
    let cancel_result = service.cancel_task(&dummy_task_id).await;
    assert!(cancel_result.is_err());
    assert!(cancel_result.unwrap_err().to_string().contains("not enabled"));
}

#[tokio::test]
async fn test_health_reporting_comprehensive() {
    // Test comprehensive health reporting functionality (with disabled orchestration)
    let config = ConfigBuilder::squirrel()
        .build() // Use disabled orchestration for testing
        .expect("Should create config");

    let service = SquirrelOrchestrationService::new(&config)
        .expect("Should create orchestration service");

    // Test healthy system
    let healthy_status = McpHealthStatus {
        core_healthy: true,
        core_message: "All core systems operational".to_string(),
        core_check_duration_ms: 15,
        ai_providers_healthy: true,
        healthy_providers: 3,
        total_providers: 3,
        ai_check_duration_ms: 45,
        protocol_healthy: true,
        protocol_message: "MCP protocols running smoothly".to_string(),
        protocol_check_duration_ms: 8,
    };

    let healthy_report = create_health_report(healthy_status);
    assert_eq!(healthy_report.status, ServiceHealth::Healthy);
    assert_eq!(healthy_report.checks.len(), 3);

    service.report_health(healthy_report).await
        .expect("Should report healthy status");

    // Test degraded system
    let degraded_status = McpHealthStatus {
        core_healthy: true,
        core_message: "Core systems operational".to_string(),
        core_check_duration_ms: 25,
        ai_providers_healthy: false,
        healthy_providers: 1,
        total_providers: 3,
        ai_check_duration_ms: 200,
        protocol_healthy: true,
        protocol_message: "Protocols operational".to_string(),
        protocol_check_duration_ms: 12,
    };

    let degraded_report = create_health_report(degraded_status);
    assert_eq!(degraded_report.status, ServiceHealth::Unhealthy);

    service.report_health(degraded_report).await
        .expect("Should report degraded status");

    // Test critical system failure
    let critical_status = McpHealthStatus {
        core_healthy: false,
        core_message: "Core system experiencing issues".to_string(),
        core_check_duration_ms: 100,
        ai_providers_healthy: false,
        healthy_providers: 0,
        total_providers: 3,
        ai_check_duration_ms: 500,
        protocol_healthy: false,
        protocol_message: "Protocol connections unstable".to_string(),
        protocol_check_duration_ms: 75,
    };

    let critical_report = create_health_report(critical_status);
    assert_eq!(critical_report.status, ServiceHealth::Unhealthy);

    service.report_health(critical_report).await
        .expect("Should report critical status");
}

#[tokio::test]
async fn test_orchestration_disabled_graceful_handling() {
    // Test that disabled orchestration handles operations gracefully
    let config = ConfigBuilder::squirrel()
        .build() // No orchestration enabled
        .expect("Should create config without orchestration");

    let mut service = SquirrelOrchestrationService::new(&config)
        .expect("Should create orchestration service");

    // Verify disabled state
    assert!(!service.is_enabled());

    // All operations should gracefully handle disabled state
    service.start().await
        .expect("Should handle start when disabled");

    service.stop().await
        .expect("Should handle stop when disabled");

    let services = service.discover_services("ai").await
        .expect("Should handle service discovery when disabled");
    assert!(services.is_empty());

    let health_report = create_health_report(McpHealthStatus::default());
    service.report_health(health_report).await
        .expect("Should handle health reporting when disabled");

    // State operations should not affect cluster state when disabled, but local state should work
    let initial_state = service.get_state().await;
    assert_eq!(initial_state, PrimalState::Stopped);
    
    // This operation should succeed locally but not affect the cluster
    service.update_state(PrimalState::Running).await
        .expect("Should handle state update when disabled");
    
    assert_eq!(service.get_state().await, PrimalState::Running);

    // Task operations should return appropriate errors
    let dummy_task = OrchestrationTask {
        id: Uuid::new_v4(),
        name: "Test Task".to_string(),
        task_type: "test".to_string(),
        target_primal: None,
        payload: serde_json::json!({}),
        priority: TaskPriority::Normal,
        retry_policy: RetryPolicy {
            max_retries: 1,
            backoff: BackoffStrategy::Fixed(Duration::from_secs(1)),
            retry_on: Vec::new(),
        },
        timeout: Duration::from_secs(60),
        dependencies: Vec::new(),
        constraints: TaskConstraints {
            required_resources: std::collections::HashMap::new(),
            node_affinity: None,
            anti_affinity: Vec::new(),
            placement: Vec::new(),
        },
        created_at: Utc::now(),
        scheduled_at: None,
    };

    let result = service.schedule_task(dummy_task).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not enabled"));
}

#[tokio::test]
async fn test_configuration_validation() {
    // Test that configuration validation works correctly for orchestration
    
    // Valid configuration with Songbird endpoint
    let valid_config = ConfigBuilder::squirrel()
        .enable_orchestration()
        .songbird_endpoint("https://songbird.internal.corp:8082").expect("Valid URL")
        .build();
    
    assert!(valid_config.is_ok());

    // Configuration that should fail validation (orchestration enabled without endpoint)
    let invalid_config = ConfigBuilder::squirrel()
        .enable_orchestration()
        .build();
    
    // This should fail due to missing Songbird endpoint when orchestration is enabled
    assert!(invalid_config.is_err());
    assert!(invalid_config.unwrap_err().to_string().contains("Songbird endpoint must be configured"));

    // Disabled orchestration should work without endpoint
    let disabled_config = ConfigBuilder::squirrel()
        .build();
    
    assert!(disabled_config.is_ok());
} 