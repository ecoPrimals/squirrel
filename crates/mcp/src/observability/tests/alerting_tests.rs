//! Alerting Tests
//! 
//! This module contains tests for the alerting functionality in the observability framework,
//! including alert creation, management, notification, and recovery handling.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::observability::alerting::{
    AlertManager, Alert, AlertManagerConfig, AlertSeverity, AlertType, AlertState
};
use crate::observability::ObservabilityError;
use crate::observability::ObservabilityResult;

/// Test alert severity types
#[test]
fn test_alert_severity_types() {
    // Test severity ordering
    assert!(AlertSeverity::Info < AlertSeverity::Warning);
    assert!(AlertSeverity::Warning < AlertSeverity::Error);
    assert!(AlertSeverity::Error < AlertSeverity::Critical);
    
    // Test to_string representation
    assert_eq!(AlertSeverity::Info.to_string(), "INFO");
    assert_eq!(AlertSeverity::Warning.to_string(), "WARNING");
    assert_eq!(AlertSeverity::Error.to_string(), "ERROR");
    assert_eq!(AlertSeverity::Critical.to_string(), "CRITICAL");
}

/// Test alert type classifications
#[test]
fn test_alert_type_classifications() {
    // Test to_string representation
    assert_eq!(AlertType::HealthStatus.to_string(), "HEALTH_STATUS");
    assert_eq!(AlertType::ResourceUsage.to_string(), "RESOURCE_USAGE");
    assert_eq!(AlertType::Security.to_string(), "SECURITY");
    assert_eq!(AlertType::Performance.to_string(), "PERFORMANCE");
    assert_eq!(AlertType::Configuration.to_string(), "CONFIGURATION");
    assert_eq!(AlertType::Custom.to_string(), "CUSTOM");
}

/// Test alert creation
#[test]
fn test_alert_creation() {
    let alert = Alert::new(
        "test-component",
        "Test alert",
        "This is a test alert",
        AlertSeverity::Warning,
        AlertType::Custom,
    );

    assert_eq!(alert.source(), "test-component");
    assert_eq!(alert.summary(), "Test alert");
    assert_eq!(alert.description(), "This is a test alert");
    assert_eq!(alert.severity(), AlertSeverity::Warning);
    assert_eq!(alert.alert_type(), AlertType::Custom);
    assert_eq!(alert.state(), AlertState::Active);
    assert!(alert.labels().is_empty());
    assert!(alert.annotations().is_empty());
    
    // Test with labels and annotations
    let alert_with_metadata = Alert::new(
        "test-component",
        "Test alert with metadata",
        "This is a test alert with additional metadata",
        AlertSeverity::Warning,
        AlertType::Custom,
    )
    .with_label("environment", "production")
    .with_label("region", "us-west")
    .with_annotation("runbook", "https://example.com/runbooks/test-alert")
    .with_annotation("team", "platform");
    
    assert_eq!(alert_with_metadata.labels().len(), 2);
    assert_eq!(alert_with_metadata.labels().get("environment").unwrap(), "production");
    assert_eq!(alert_with_metadata.labels().get("region").unwrap(), "us-west");
    
    assert_eq!(alert_with_metadata.annotations().len(), 2);
    assert_eq!(alert_with_metadata.annotations().get("runbook").unwrap(), "https://example.com/runbooks/test-alert");
    assert_eq!(alert_with_metadata.annotations().get("team").unwrap(), "platform");
}

/// Test alert manager configuration
#[test]
fn test_alert_manager_config() {
    // Create default configuration
    let default_config = AlertManagerConfig::default();
    
    // Create custom configuration
    let custom_config = AlertManagerConfig {
        notification_buffer: 100,
        max_alerts: 1000,
    };
    
    assert_eq!(custom_config.notification_buffer, 100);
    assert_eq!(custom_config.max_alerts, 1000);
}

/// Test alert manager initialization
#[test]
fn test_alert_manager_initialization() -> ObservabilityResult<()> {
    // Create alert manager
    let alert_manager = AlertManager::new();
    
    // Initialize
    alert_manager.initialize()?;
    
    // Set custom config
    let custom_config = AlertManagerConfig {
        notification_buffer: 100,
        max_alerts: 1000,
    };
    
    alert_manager.set_config(custom_config)?;
    
    // Verify no alerts yet
    let alerts = alert_manager.get_alerts(None, None, None, None)?;
    assert_eq!(alerts.len(), 0);
    
    Ok(())
}

/// Test alert triggering and querying
#[test]
fn test_alert_triggering_and_querying() -> ObservabilityResult<()> {
    let alert_manager = AlertManager::new();
    
    // Trigger a basic alert
    let triggered = alert_manager.alert(
        "api-service",
        "API service is not responding",
        "The API service is not responding to health checks",
        AlertSeverity::Error,
        AlertType::HealthStatus
    )?;
    
    // Verify alert was created
    assert_eq!(triggered.source(), "api-service");
    assert_eq!(triggered.summary(), "API service is not responding");
    assert_eq!(triggered.severity(), AlertSeverity::Error);
    assert_eq!(triggered.alert_type(), AlertType::HealthStatus);
    assert_eq!(triggered.state(), AlertState::Active);
    
    // Get the alert by ID
    let retrieved = alert_manager.get_alert(triggered.id())?;
    assert!(retrieved.is_some());
    
    let retrieved_alert = retrieved.unwrap();
    assert_eq!(retrieved_alert.id(), triggered.id());
    
    // Get all active alerts
    let active_alerts = alert_manager.get_alerts(None, None, None, Some(AlertState::Active))?;
    assert_eq!(active_alerts.len(), 1);
    assert_eq!(active_alerts[0].id(), triggered.id());
    
    // Trigger a second alert
    let second_alert = alert_manager.alert(
        "database",
        "Database connection latency high",
        "Database queries are experiencing high latency",
        AlertSeverity::Warning,
        AlertType::Performance
    )?;
    
    // Get alerts filtered by source
    let api_alerts = alert_manager.get_alerts(Some("api-service"), None, None, None)?;
    assert_eq!(api_alerts.len(), 1);
    assert_eq!(api_alerts[0].id(), triggered.id());
    
    // Get alerts filtered by severity
    let warning_alerts = alert_manager.get_alerts(None, Some(AlertSeverity::Warning), None, None)?;
    assert_eq!(warning_alerts.len(), 1);
    assert_eq!(warning_alerts[0].id(), second_alert.id());
    
    // Get alerts filtered by type
    let performance_alerts = alert_manager.get_alerts(None, None, Some(AlertType::Performance), None)?;
    assert_eq!(performance_alerts.len(), 1);
    assert_eq!(performance_alerts[0].id(), second_alert.id());
    
    Ok(())
}

/// Test alert acknowledgment and resolution
#[test]
fn test_alert_acknowledgment_and_resolution() -> ObservabilityResult<()> {
    let alert_manager = AlertManager::new();
    
    // Create an alert
    let alert = alert_manager.alert(
        "service-a",
        "Service A degraded",
        "Service A is experiencing degraded performance",
        AlertSeverity::Warning,
        AlertType::Performance
    )?;
    
    // Acknowledge the alert
    let acknowledged = alert_manager.acknowledge_alert(alert.id())?;
    assert!(acknowledged);
    
    // Verify state changed
    let updated = alert_manager.get_alert(alert.id())?.unwrap();
    assert_eq!(updated.state(), AlertState::Acknowledged);
    
    // Create another alert
    let alert2 = alert_manager.alert(
        "service-b",
        "Service B down",
        "Service B is not responding",
        AlertSeverity::Critical,
        AlertType::HealthStatus
    )?;
    
    // Resolve the alert
    let resolved = alert_manager.resolve_alert(alert2.id())?;
    assert!(resolved);
    
    // Verify state changed
    let updated2 = alert_manager.get_alert(alert2.id())?.unwrap();
    assert_eq!(updated2.state(), AlertState::Resolved);
    
    // Get alerts by state
    let acknowledged_alerts = alert_manager.get_alerts(None, None, None, Some(AlertState::Acknowledged))?;
    assert_eq!(acknowledged_alerts.len(), 1);
    assert_eq!(acknowledged_alerts[0].id(), alert.id());
    
    let resolved_alerts = alert_manager.get_alerts(None, None, None, Some(AlertState::Resolved))?;
    assert_eq!(resolved_alerts.len(), 1);
    assert_eq!(resolved_alerts[0].id(), alert2.id());
    
    Ok(())
}

/// Test alert notification subscription
#[test]
fn test_alert_notification_subscription() -> ObservabilityResult<()> {
    let alert_manager = AlertManager::new();
    
    // Subscribe to alerts
    let mut subscriber = alert_manager.subscribe();
    
    // Trigger an alert
    let alert = alert_manager.alert(
        "notification-test",
        "Test notification",
        "Testing the notification system",
        AlertSeverity::Info,
        AlertType::Custom
    )?;
    
    // Receive the notification
    let received = subscriber.try_recv();
    assert!(received.is_ok());
    
    let notification = received.unwrap();
    assert_eq!(notification.id(), alert.id());
    
    Ok(())
}

/// Test concurrent alert management
#[tokio::test]
async fn test_concurrent_alert_management() -> ObservabilityResult<()> {
    let alert_manager = Arc::new(AlertManager::new());
    
    let mut handles = vec![];
    
    // Create 10 alerts concurrently
    for i in 0..10 {
        let manager = Arc::clone(&alert_manager);
        let handle = tokio::spawn(async move {
            let alert = manager.alert(
                &format!("component-{}", i),
                &format!("Alert {}", i),
                &format!("Description for alert {}", i),
                if i % 2 == 0 { AlertSeverity::Warning } else { AlertSeverity::Error },
                if i % 3 == 0 { AlertType::Performance } else { AlertType::Custom }
            ).unwrap();
            
            // If even number, resolve the alert
            if i % 2 == 0 {
                manager.resolve_alert(alert.id()).unwrap();
            }
            
            alert.id().to_string()
        });
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    let alert_ids = futures::future::join_all(handles).await;
    
    // Check results
    let active_alerts = alert_manager.get_alerts(None, None, None, Some(AlertState::Active))?;
    assert_eq!(active_alerts.len(), 5); // Odd-numbered alerts should be active
    
    let resolved_alerts = alert_manager.get_alerts(None, None, None, Some(AlertState::Resolved))?;
    assert_eq!(resolved_alerts.len(), 5); // Even-numbered alerts should be resolved
    
    Ok(())
}

/// Test alert eviction when exceeding max_alerts
#[test]
fn test_alert_eviction() -> ObservabilityResult<()> {
    // Create alert manager with small max_alerts
    let alert_manager = AlertManager::new();
    
    let small_config = AlertManagerConfig {
        notification_buffer: 10,
        max_alerts: 3, // Only keep 3 alerts
    };
    
    alert_manager.set_config(small_config)?;
    
    // Create 5 alerts
    for i in 0..5 {
        alert_manager.alert(
            &format!("component-{}", i),
            &format!("Alert {}", i),
            &format!("Description for alert {}", i),
            AlertSeverity::Info,
            AlertType::Custom
        )?;
    }
    
    // Check that only the latest 3 alerts are kept
    let all_alerts = alert_manager.get_alerts(None, None, None, None)?;
    assert_eq!(all_alerts.len(), 3);
    
    // The remaining alerts should be the most recent ones
    let sources: Vec<&str> = all_alerts.iter().map(|a| a.source()).collect();
    assert!(sources.contains(&"component-2"));
    assert!(sources.contains(&"component-3"));
    assert!(sources.contains(&"component-4"));
    
    Ok(())
} 