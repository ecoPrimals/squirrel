//! Simple example to test the actual monitoring API structure
//! 
//! This example creates instances of the monitoring system components and
//! prints their structure to help debug API compatibility issues.

use std::sync::Arc;

use squirrel_mcp::monitoring::{
    alerts::{Alert, AlertConfiguration, AlertSeverity, AlertState, AlertCondition, AlertAction},
    metrics::{Metric, MetricType, MetricValue, MetricsCollector},
    HealthStatus, AlertManager
};

fn main() {
    println!("=== MCP Monitoring API Test ===");
    
    // Test metric creation and usage
    test_metrics();
    
    // Test alert creation and usage
    test_alerts();
    
    println!("=== Test Complete ===");
}

fn test_metrics() {
    println!("\n--- Testing Metrics API ---");
    
    // Create a metrics collector
    let metrics_collector = Arc::new(MetricsCollector::new());
    println!("Created MetricsCollector: {:?}", metrics_collector);
    
    // Create a metric
    let metric = Metric::new(
        "test_metric", 
        "Test metric description", 
        MetricType::Gauge, 
        MetricValue::Float(42.0)
    ).with_label("component", "test");
    
    println!("Created Metric: {:?}", metric);
    
    // Register metric
    metrics_collector.register_metric(metric.clone());
    println!("Registered metric");
    
    // Get metric
    let retrieved = metrics_collector.get_metric(&metric.name);
    println!("Retrieved metric: {:?}", retrieved);
    
    // Update metric
    metrics_collector.update_metric(&metric.name, MetricValue::Float(84.0));
    println!("Updated metric");
    
    // Get updated metric
    let updated = metrics_collector.get_metric(&metric.name);
    println!("Updated metric value: {:?}", updated);
    
    // Test get_metric method availability
    println!("MetricsCollector has get_metric method: {}", 
        if MetricsCollector::new().get_metric("test").is_none() { "Yes" } else { "Unknown" });
}

fn test_alerts() {
    println!("\n--- Testing Alerts API ---");
    
    // Create an alert manager
    let alert_manager = Arc::new(AlertManager::new());
    println!("Created AlertManager: {:?}", alert_manager);
    
    // Create alert configuration
    let alert_config = AlertConfiguration {
        name: "test_alert".to_string(),
        description: "Test alert description".to_string(),
        condition: AlertCondition::Custom("test_condition".to_string()),
        severity: AlertSeverity::Warning,
        actions: vec![AlertAction::Log],
        check_interval_seconds: 60,
        minimum_interval_seconds: 300,
        enabled: true,
        labels: std::collections::HashMap::new(),
    };
    
    println!("Created AlertConfiguration: {:?}", alert_config);
    
    // Add alert to manager
    let alert_id = alert_manager.add_alert(alert_config.clone());
    println!("Added alert, got ID: {}", alert_id);
    
    // Get alert
    let alert = alert_manager.get_alert(&alert_id);
    println!("Retrieved alert: {:?}", alert);
    
    // Test extract_severity functionality
    if let Some(alert) = alert {
        println!("Alert severity: {:?}", alert.config.severity);
        println!("Alert message: {}", alert.config.description);
        println!("Alert ID: {}", alert.id);
    }
    
    // Test downcast_ref availability
    println!("AlertManager does NOT have downcast_ref");
} 