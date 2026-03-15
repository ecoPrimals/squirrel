// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::observability::health::{
    HealthChecker, HealthStatus, HealthCheckType, 
    HealthCheckResult, ComponentHealth,
    connect_health_to_alerting, create_standard_health_checks
};
use crate::observability::alerting::{AlertManager, AlertSeverity};
use crate::observability::ObservabilityResult;

#[tokio::test]
async fn test_health_checker_creation() {
    let health_checker = HealthChecker::new();
    assert!(health_checker.initialize().await.is_ok());
    health_checker.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_component_registration() {
    let health_checker = HealthChecker::new();
    health_checker.initialize().await.unwrap();
    
    // Register a component
    let result = health_checker.register_component(
        "test_component",
        "Test Component",
        HealthStatus::Healthy,
    ).await;
    assert!(result.is_ok());
    
    // Get component health
    let component_health = health_checker.get_component_health("test_component").await.unwrap();
    assert!(component_health.is_some());
    
    let health = component_health.unwrap();
    assert_eq!(health.component_id, "test_component");
    assert_eq!(health.name, "Test Component");
    assert_eq!(health.status, HealthStatus::Healthy);
    
    health_checker.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_component_status_update() {
    let health_checker = HealthChecker::new();
    health_checker.initialize().await.unwrap();
    
    // Register a component
    health_checker.register_component(
        "test_component",
        "Test Component",
        HealthStatus::Healthy,
    ).await.unwrap();
    
    // Update status
    let result = health_checker.update_component_status(
        "test_component",
        HealthStatus::Degraded,
        Some("Service is experiencing high latency".to_string()),
    ).await;
    assert!(result.is_ok());
    
    // Get updated health
    let component_health = health_checker.get_component_health("test_component").await.unwrap().unwrap();
    assert_eq!(component_health.status, HealthStatus::Degraded);
    assert_eq!(component_health.details, Some("Service is experiencing high latency".to_string()));
    
    // Cleanup
    health_checker.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_health_check_registration() {
    let health_checker = HealthChecker::new();
    health_checker.initialize().await.unwrap();
    
    // Register a component
    health_checker.register_component(
        "test_component",
        "Test Component",
        HealthStatus::Healthy,
    ).await.unwrap();
    
    // Register a health check
    let result = health_checker.register_health_check(
        "test_check",
        "test_component",
        "Test Check",
        HealthCheckType::Liveness,
        Box::new(|| HealthCheckResult::healthy()),
        None,
    ).await;
    assert!(result.is_ok());
    
    // Cleanup
    health_checker.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_system_health_status() {
    let health_checker = HealthChecker::new();
    health_checker.initialize().await.unwrap();
    
    // Register multiple components with different statuses
    health_checker.register_component(
        "component1",
        "Component 1",
        HealthStatus::Healthy,
    ).await.unwrap();
    
    health_checker.register_component(
        "component2",
        "Component 2",
        HealthStatus::Healthy,
    ).await.unwrap();
    
    // System should be healthy when all components are healthy
    let system_status = health_checker.get_system_health_status().await.unwrap();
    assert_eq!(system_status, HealthStatus::Healthy);
    
    // Update one component to degraded
    health_checker.update_component_status(
        "component1",
        HealthStatus::Degraded,
        None,
    ).await.unwrap();
    
    // System should be degraded when at least one component is degraded
    let system_status = health_checker.get_system_health_status().await.unwrap();
    assert_eq!(system_status, HealthStatus::Degraded);
    
    // Update one component to unhealthy
    health_checker.update_component_status(
        "component2",
        HealthStatus::Unhealthy,
        None,
    ).await.unwrap();
    
    // System should be unhealthy when at least one component is unhealthy
    let system_status = health_checker.get_system_health_status().await.unwrap();
    assert_eq!(system_status, HealthStatus::Unhealthy);
    
    // Cleanup
    health_checker.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_health_check_execution() {
    let health_checker = HealthChecker::new();
    health_checker.initialize().await.unwrap();
    
    // Register a component
    health_checker.register_component(
        "test_component",
        "Test Component",
        HealthStatus::Unknown,
    ).await.unwrap();
    
    // Track check execution count
    let execution_count = Arc::new(AtomicUsize::new(0));
    let execution_count_clone = execution_count.clone();
    
    // Register a health check that increments the counter
    health_checker.register_health_check(
        "test_check",
        "test_component",
        "Test Check",
        HealthCheckType::Liveness,
        Box::new(move || {
            execution_count_clone.fetch_add(1, Ordering::SeqCst);
            HealthCheckResult::healthy()
        }),
        Some(1), // Run every 1 second
    ).await.unwrap();
    
    // Wait for the check to run at least once
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Check that the check was executed
    assert!(execution_count.load(Ordering::SeqCst) > 0);
    
    // Check that the component status was updated
    let component_health = health_checker.get_component_health("test_component").await.unwrap().unwrap();
    assert_eq!(component_health.status, HealthStatus::Healthy);
    
    // Cleanup
    health_checker.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_health_status_subscription() {
    let health_checker = Arc::new(HealthChecker::new());
    health_checker.initialize().await.unwrap();
    
    // Register a component
    health_checker.register_component(
        "test_component",
        "Test Component",
        HealthStatus::Healthy,
    ).await.unwrap();
    
    // Subscribe to status changes
    let mut status_rx = health_checker.subscribe_to_status_changes();
    
    // Spawn a task to receive status changes
    let status_change_received = Arc::new(tokio::sync::Notify::new());
    let status_change_received_clone = status_change_received.clone();
    
    tokio::spawn(async move {
        if let Ok(event) = status_rx.recv().await {
            assert_eq!(event.system_status, HealthStatus::Degraded);
            assert!(event.component_statuses.contains_key("test_component"));
            assert_eq!(event.component_statuses["test_component"].status, HealthStatus::Degraded);
            status_change_received_clone.notify_one();
        }
    });
    
    // Update component status to trigger event
    health_checker.update_component_status(
        "test_component",
        HealthStatus::Degraded,
        Some("Service is degraded".to_string()),
    ).await.unwrap();
    
    // Wait for status change to be received or timeout
    tokio::select! {
        _ = status_change_received.notified() => {
            // Status change was received
        }
        _ = tokio::time::sleep(Duration::from_secs(2)) => {
            panic!("Timed out waiting for status change event");
        }
    }
    
    // Cleanup
    health_checker.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_health_alerting_integration() {
    let health_checker = Arc::new(HealthChecker::new());
    let alert_manager = Arc::new(AlertManager::new());
    
    health_checker.initialize().await.unwrap();
    
    // Register a component
    health_checker.register_component(
        "test_component",
        "Test Component",
        HealthStatus::Healthy,
    ).await.unwrap();
    
    // Connect health to alerting
    let _task = connect_health_to_alerting(
        health_checker.clone(),
        alert_manager.clone(),
    ).unwrap();
    
    // Update component to unhealthy to trigger alert
    health_checker.update_component_status(
        "test_component",
        HealthStatus::Unhealthy,
        Some("Component has failed".to_string()),
    ).await.unwrap();
    
    // Wait a bit for the alert to be created
    sleep(Duration::from_millis(100)).await;
    
    // Check that an alert was created
    let alerts = alert_manager.get_active_alerts().unwrap();
    assert!(!alerts.is_empty(), "Should have created at least one alert");
    
    // Find the component alert
    let component_alert = alerts.iter().find(|a| {
        a.component.as_deref() == Some("test_component")
    });
    assert!(component_alert.is_some(), "Should have created an alert for the component");
    assert_eq!(component_alert.unwrap().severity, AlertSeverity::Critical);
    
    // Update component back to healthy
    health_checker.update_component_status(
        "test_component",
        HealthStatus::Healthy,
        Some("Component has recovered".to_string()),
    ).await.unwrap();
    
    // Wait a bit for the alert to be resolved
    sleep(Duration::from_millis(100)).await;
    
    // Check that the alert was resolved
    let active_alerts = alert_manager.get_active_alerts().unwrap();
    let component_alert = active_alerts.iter().find(|a| {
        a.component.as_deref() == Some("test_component")
    });
    assert!(component_alert.is_none(), "Component alert should have been resolved");
    
    health_checker.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_standard_health_checks() {
    let health_checker = HealthChecker::new();
    health_checker.initialize().await.unwrap();
    
    // Register a component
    health_checker.register_component(
        "test_component",
        "Test Component",
        HealthStatus::Healthy,
    ).await.unwrap();
    
    // Create standard health checks
    let result = create_standard_health_checks(
        &health_checker,
        "test_component",
    );
    assert!(result.is_ok());
    
    // Now health checker should have standard health checks registered
    // Wait for a short time and check that the component still exists
    tokio::time::sleep(Duration::from_millis(100)).await;
    let component_health = health_checker.get_component_health("test_component").await.unwrap();
    assert!(component_health.is_some());
    
    health_checker.shutdown().await.unwrap();
}

#[test]
fn test_component_health() {
    // Test ComponentHealth struct directly
    let mut health = ComponentHealth::new(
        "test_component",
        "Test Component",
        HealthStatus::Healthy,
    );
    
    // Add details
    health = health.with_details("Component is functioning normally");
    
    // Add metadata
    health = health.with_metadata("version", "1.0.0");
    health = health.with_metadata("region", "us-west");
    
    // Verify
    assert_eq!(health.component_id, "test_component");
    assert_eq!(health.name, "Test Component");
    assert_eq!(health.status, HealthStatus::Healthy);
    assert_eq!(health.details, Some("Component is functioning normally".to_string()));
    assert_eq!(health.metadata.get("version"), Some(&"1.0.0".to_string()));
    assert_eq!(health.metadata.get("region"), Some(&"us-west".to_string()));
    
    // Update status
    health.update_status(HealthStatus::Degraded, Some("Component is experiencing issues".to_string()));
    
    // Verify updated status
    assert_eq!(health.status, HealthStatus::Degraded);
    assert_eq!(health.details, Some("Component is experiencing issues".to_string()));
    
    // Test helper methods
    assert!(!health.is_healthy());
    assert!(health.is_degraded());
    assert!(!health.is_unhealthy());
}

#[test]
fn test_health_check_results() {
    // Test HealthCheckResult factory methods
    let healthy = HealthCheckResult::healthy();
    assert_eq!(healthy.status, HealthStatus::Healthy);
    assert_eq!(healthy.details, None);
    
    let healthy_with_details = HealthCheckResult::healthy_with_details("All systems nominal");
    assert_eq!(healthy_with_details.status, HealthStatus::Healthy);
    assert_eq!(healthy_with_details.details, Some("All systems nominal".to_string()));
    
    let degraded = HealthCheckResult::degraded("High latency detected");
    assert_eq!(degraded.status, HealthStatus::Degraded);
    assert_eq!(degraded.details, Some("High latency detected".to_string()));
    
    let unhealthy = HealthCheckResult::unhealthy("Service unavailable");
    assert_eq!(unhealthy.status, HealthStatus::Unhealthy);
    assert_eq!(unhealthy.details, Some("Service unavailable".to_string()));
    
    let unknown = HealthCheckResult::unknown();
    assert_eq!(unknown.status, HealthStatus::Unknown);
    assert_eq!(unknown.details, None);
} 