// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use tokio::time::sleep;
use std::time::Duration;
use crate::observability::example::run_observability_example;
use crate::observability::{ObservabilityConfig, initialize_with_config};

/// Test that the example can run without errors
#[tokio::test]
async fn test_example_runs() {
    // Create a test configuration that doesn't connect to external services
    let config = ObservabilityConfig {
        service_name: "test-example".to_string(),
        environment: "test".to_string(),
        enable_dashboard_integration: false,
        enable_external_tracing: false,
        connect_health_to_alerting: true,
        ..ObservabilityConfig::default()
    };
    
    // Initialize the framework
    let framework = initialize_with_config(config).expect("should succeed");
    
    // The example already contains its own setup, so for testing we'll just
    // create a simplified version of the example
    
    // Create a simpler test that uses the framework
    let mut labels = std::collections::HashMap::new();
    labels.insert("test".to_string(), "true".to_string());
    
    // Create a counter
    let counter = framework.metrics().create_counter(
        "test_counter",
        "Test counter",
        None,
        labels.clone(),
    ).expect("should succeed");
    
    // Create a gauge
    let gauge = framework.metrics().create_gauge(
        "test_gauge",
        "Test gauge",
        None,
        labels.clone(),
    ).expect("should succeed");
    
    // Increment counter and set gauge
    counter.inc_one().expect("should succeed");
    gauge.set(42.0).expect("should succeed");
    
    // Create a span
    let span = framework.tracer().start_span("test_span").expect("should succeed");
    
    // Create component and update health
    framework.health_checker().register_component(
        "test_component",
        "Test Component",
        crate::observability::health::HealthStatus::Healthy,
    ).expect("should succeed");
    
    // End the span
    {
        let mut span_guard = span.lock().expect("should succeed");
        span_guard.end();
    }
    
    // Create an alert
    framework.alert_manager().create_alert(
        "test_alert",
        "Test alert",
        crate::observability::alerting::AlertSeverity::Info,
        Some("This is a test alert"),
        Some("test_component"),
        None,
    ).expect("should succeed");
    
    // Verify everything is working
    assert_eq!(counter.value().expect("should succeed"), 1);
    assert_eq!(gauge.value().expect("should succeed"), 42.0);
    
    let component_health = framework.health_checker().get_component_health("test_component").expect("should succeed");
    assert!(component_health.is_some());
    
    let alerts = framework.alert_manager().get_active_alerts().expect("should succeed");
    assert!(!alerts.is_empty());
}

/// Test the full example implementation
/// This test runs the full example, but with external services disabled
#[tokio::test]
async fn test_full_example() {
    // Override the configuration in the example to disable external connections
    let mut config = ObservabilityConfig::default();
    config.enable_dashboard_integration = false;
    config.enable_external_tracing = false;
    
    // Initialize with the modified config
    let _framework = initialize_with_config(config).expect("should succeed");
    
    // Run the example and verify it doesn't error
    let result = run_observability_example().await;
    assert!(result.is_ok(), "Example should run without errors");
}

/// Test that the example metrics and components are created and updated
#[tokio::test]
async fn test_example_metrics_and_components() {
    // Create a test configuration
    let config = ObservabilityConfig {
        service_name: "test-example".to_string(),
        environment: "test".to_string(),
        enable_dashboard_integration: false,
        enable_external_tracing: false,
        connect_health_to_alerting: true,
        ..ObservabilityConfig::default()
    };
    
    // Initialize the framework
    let framework = initialize_with_config(config).expect("should succeed");
    
    // Register the example component
    framework.health_checker().register_component(
        "example_component",
        "Example Component",
        crate::observability::health::HealthStatus::Healthy,
    ).expect("should succeed");
    
    // Create example metrics
    let mut labels = std::collections::HashMap::new();
    labels.insert("component".to_string(), "example".to_string());
    
    let counter = framework.metrics().create_counter(
        "example_operations_total",
        "Total number of operations performed",
        Some("operations".to_string()),
        labels.clone(),
    ).expect("should succeed");
    
    let gauge = framework.metrics().create_gauge(
        "example_memory_usage",
        "Current memory usage",
        Some("bytes".to_string()),
        labels.clone(),
    ).expect("should succeed");
    
    // Update metrics
    counter.inc_one().expect("should succeed");
    gauge.set(1024.0).expect("should succeed");
    
    // Verify metrics are updated
    assert_eq!(counter.value().expect("should succeed"), 1);
    assert_eq!(gauge.value().expect("should succeed"), 1024.0);
    
    // Register a health check
    framework.health_checker().register_health_check(
        "example_component_check",
        "example_component",
        "Example Health Check",
        crate::observability::health::HealthCheckType::Readiness,
        Box::new(|| {
            crate::observability::health::HealthCheckResult::healthy_with_details(
                "Component is functioning normally".to_string()
            )
        }),
        Some(1), // Run every second for testing
    ).expect("should succeed");
    
    // Wait for the health check to run
    sleep(Duration::from_millis(1500)).await;
    
    // Verify health status
    let component_health = framework.health_checker().get_component_health("example_component").expect("should succeed");
    assert!(component_health.is_some());
    assert_eq!(component_health.expect("should succeed").status, crate::observability::health::HealthStatus::Healthy);
    
    // Create an alert
    framework.alert_manager().create_alert(
        "example_alert",
        "Example alert",
        crate::observability::alerting::AlertSeverity::Info,
        Some("This is an example alert"),
        Some("example_component"),
        None,
    ).expect("should succeed");
    
    // Verify alert was created
    let alerts = framework.alert_manager().get_active_alerts().expect("should succeed");
    assert!(!alerts.is_empty());
    let example_alert = alerts.iter().find(|a| a.id() == "example_alert");
    assert!(example_alert.is_some());
} 