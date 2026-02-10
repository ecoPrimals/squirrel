// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

// Example file showing how to use the enhanced ObservabilityFramework

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use crate::observability::{
    self,
    metrics::{Counter, Gauge, Histogram, Labels},
    health::{self, HealthStatus, HealthCheckResult},
    alerting::{AlertSeverity},
    ObservabilityConfig,
    ObservabilityResult,
    ObservabilityFramework,
};

/// Run a simple example showing ObservabilityFramework integration
pub async fn run_observability_example() -> ObservabilityResult<()> {
    // Create a custom configuration
    let config = ObservabilityConfig {
        service_name: "mcp-example".to_string(),
        environment: "development".to_string(),
        
        // Enable dashboard integration
        enable_dashboard: true,
        default_log_level: log::Level::Debug,
        include_trace_context_in_logs: true,
        enable_tracing: true,
        enable_metrics: true,
        enable_health_checks: true,
        enable_alerting: true,
        
        // Use other defaults
        ..ObservabilityConfig::default()
    };
    
    // Initialize the framework with custom configuration
    let framework = observability::initialize_with_config(config).await?;
    
    // Register custom components
    framework.health_checker.register_component(
        "example_component",
        "Example Component",
        HealthStatus::Unknown,
    ).await?;
    
    // Register custom metrics
    let mut labels = Labels::new();
    labels.insert("component".to_string(), "example".to_string());
    
    let operation_counter: Arc<Counter> = framework.metrics.create_counter(
        "example_operations_total",
        "Total number of operations performed",
        Some("operations".to_string()),
        labels.clone(),
    )?;
    
    let memory_gauge: Arc<Gauge> = framework.metrics.create_gauge(
        "example_memory_usage",
        "Current memory usage",
        Some("bytes".to_string()),
        labels.clone(),
    )?;
    
    let duration_histogram: Arc<Histogram> = framework.metrics.create_histogram(
        "example_operation_duration",
        "Duration of operations",
        Some("seconds".to_string()),
        labels.clone(),
        vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0],
    )?;
    
    // Register a health check for the example component
    framework.health_checker.register_health_check(
        "example_component",
        "example_health_check",
        "Example Health Check",
        Box::new(|| {
            // Check if the component is healthy based on metrics
            let component_metrics_ok = check_component_metrics();
            let component_connections_ok = check_component_connections();
            
            if component_metrics_ok && component_connections_ok {
                HealthCheckResult::healthy_with_message("Component is healthy")
            } else if component_metrics_ok {
                HealthCheckResult::degraded("Component connections are experiencing issues")
            } else if component_connections_ok {
                HealthCheckResult::degraded("Component metrics are outside normal range")
            } else {
                HealthCheckResult::unhealthy("Component metrics and connections are failing")
            }
        })
    )?;
    
    // Update component health status
    framework.health_checker.update_component_status(
        "example_component",
        HealthStatus::Healthy,
        Some("The example component has been initialized".to_string()),
    )?;
    
    // Create an alert
    let _ = framework.alert_manager.create_alert(
        "example_initialization",
        "Example Component Initialized",
        AlertSeverity::Info,
        Some("The example component has been initialized"),
        Some("example_component"),
        None
    )?;
    
    println!("ObservabilityFramework initialized and configured");
    println!("Running example workload...");
    
    // Simulate a workload with tracing, metrics, and health checks
    for i in 1..=10 {
        // Create a trace span for this operation
        let operation_span = framework.tracer.start_span(format!("example_operation_{}", i))?;
        {
            let span_guard = operation_span.lock().expect("span lock poisoned");
            println!("Starting operation {} with span ID: {}", i, span_guard.span().id());
        }
        
        // Update metrics
        operation_counter.inc_one()?;
        memory_gauge.set(1000.0 * i as f64)?;
        
        // Perform the "operation"
        let start_time = std::time::Instant::now();
        
        // Simulate some work with sub-spans
        {
            let sub_span = framework.tracer.start_span_with_parent(
                "sub_operation_a",
                Some(operation_span.clone()),
            )?;
            
            // Do some work
            sleep(Duration::from_millis(50)).await;
            
            // End the span properly by consuming it
            let span_guard = sub_span.lock().expect("span lock poisoned");
            drop(span_guard);
            // The span will be automatically ended when the Arc is dropped
        }
        
        {
            let sub_span = framework.tracer.start_span_with_parent(
                "sub_operation_b",
                Some(operation_span.clone()),
            )?;
            
            // Do some more work
            sleep(Duration::from_millis(100)).await;
            
            // End the span properly by consuming it
            let span_guard = sub_span.lock().expect("span lock poisoned");
            drop(span_guard);
            // The span will be automatically ended when the Arc is dropped
        }
        
        // Record the duration
        let duration = start_time.elapsed().as_secs_f64();
        duration_histogram.observe(duration)?;
        
        // End the main span
        {
            let span_guard = operation_span.lock().expect("span lock poisoned"); 
            drop(span_guard);
            // The span will be automatically ended when the Arc is dropped
        }
        
        // Update health status on some iterations
        if i % 3 == 0 {
            // Toggle health status
            let status = if i % 6 == 0 {
                HealthStatus::Degraded
            } else {
                HealthStatus::Healthy
            };
            
            let details = if status == HealthStatus::Healthy {
                Some(format!("Component healthy at iteration {}", i))
            } else {
                Some(format!("Component degraded at iteration {}", i))
            };
            
            framework.health_checker.update_component_status(
                "example_component",
                status,
                details,
            )?;
            
            println!("Updated health status: {:?}", status);
        }
        
        // Add some MCP-specific metrics
        framework.metrics.increment_counter("mcp_messages_processed_total", 1.0, None)?;
        framework.metrics.increment_counter("mcp_messages_sent", 1.0, None)?;
        framework.metrics.set_gauge("mcp_active_plugins", 3.0, None)?;
        
        if i % 5 == 0 {
            // Create an alert for the error
            framework.alert_manager.create_alert(
                &format!("example_error_{}", i),
                "Example error occurred",
                AlertSeverity::Warning,
                Some(&format!("An error occurred during iteration {}", i)),
                Some("example_component"),
                None,
            )?;
            
            println!("Recorded error and created alert");
        }
        
        println!("Completed operation {}", i);
        sleep(Duration::from_millis(500)).await;
    }
    
    // Final health status
    framework.health_checker.update_component_status(
        "example_component",
        HealthStatus::Healthy,
        Some("The example component has completed all operations".to_string()),
    )?;
    
    // Create a completion alert
    framework.alert_manager.create_alert(
        "example_completion",
        "Example component completed",
        AlertSeverity::Info,
        Some("The example component has completed all operations"),
        Some("example_component"),
        None,
    )?;
    
    println!("Example workload completed");
    println!("Observability data has been collected and exported");
    
    // Allow time for exporters to flush
    sleep(Duration::from_secs(2)).await;
    
    Ok(())
}

/// Example usage of the observability framework
pub async fn example_usage() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the framework
    let framework = ObservabilityFramework::new().await?;
    
    // Register a component for health monitoring
    framework.health_checker.register_component(
        "test_component",
        "Test Component",
        HealthStatus::Healthy,
    ).await?;
    
    // Update component health status
    framework.health_checker.update_component_status(
        "test_component",
        HealthStatus::Healthy,
        Some("Component is functioning normally".to_string()),
    )?;

    // Get component health
    let health = framework.health_checker.get_component_health("test_component")?;
    println!("Component health: {:?}", health);
    
    // Update component health with details
    framework.health_checker.update_component_status(
        "test_component",
        HealthStatus::Degraded,
        Some("Component is experiencing slowdown".to_string()),
    )?;

    // Update component health to unhealthy
    framework.health_checker.update_component_status(
        "test_component",
        HealthStatus::Unhealthy,
        Some("Component has failed".to_string()),
    )?;

    // Update component health back to healthy
    framework.health_checker.update_component_status(
        "test_component",
        HealthStatus::Healthy,
        Some("Component has recovered".to_string()),
    )?;
    
    // Create an alert
    framework.alert_manager.create_alert(
        "test_alert",
        "Test Alert",
        AlertSeverity::Warning,
        Some("This is a test alert"),
        Some("test_component"),
        None,
    )?;
    
    Ok(())
}

/// Initialize example component
pub fn initialize_example_component(framework: &ObservabilityFramework) -> ObservabilityResult<()> {
    // Register the example component
    framework.health_checker.register_component_sync(
        "example_component",
        "Example Component",
        HealthStatus::Unknown,
    )?;
    
    // Register standard health checks
    health::create_standard_health_checks(
        &framework.health_checker,
        "example_component",
    )?;
    
    // Update component status to healthy
    framework.health_checker.update_component_status(
        "example_component",
        HealthStatus::Healthy,
        Some("The example component has completed all operations".to_string()),
    )?;
    
    Ok(())
}

// Helper functions for the example
fn check_component_metrics() -> bool {
    // Simulate a check on component metrics
    // In a real implementation, this would actually check metrics
    true // For this example, we'll say metrics are good
}

fn check_component_connections() -> bool {
    // Simulate a check on component connections
    // In a real implementation, this would check actual connections
    true // For this example, we'll say connections are good
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observability::ObservabilityConfig;
    
    #[tokio::test]
    async fn test_example_usage() {
        // Create a test configuration
        let config = ObservabilityConfig {
            enable_dashboard: false,
            ..Default::default()
        };
        
        // Initialize the framework
        let framework = ObservabilityFramework::new_with_config(config).await.unwrap();
        
        // Run the example usage
        example_usage().await.unwrap();
        
        // Initialize example component
        initialize_example_component(&framework).unwrap();
    }
} 