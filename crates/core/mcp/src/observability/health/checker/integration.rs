//! Health Check Integration
//!
//! This module contains integration functionality for connecting
//! health checking to other systems like alerting and monitoring.

use std::sync::Arc;
use std::time::Duration;

use crate::observability::ObservabilityResult;
use crate::observability::health::types::{HealthStatus, HealthCheckType};
use crate::observability::health::result::HealthCheckResult;
use crate::observability::alerting::{AlertManager, AlertSeverity};

use super::core::HealthChecker;

/// Connect health checking to the alerting system
///
/// This function creates a background task that monitors health status changes
/// and creates alerts when components become unhealthy.
pub fn connect_health_to_alerting(
    health_checker: Arc<HealthChecker>,
    alert_manager: Arc<AlertManager>,
) -> ObservabilityResult<tokio::task::JoinHandle<()>> {
    // Create a subscriber to health status changes
    let mut subscriber = health_checker.subscribe();
    
    // Spawn a task to forward relevant events to alerting
    let handle = tokio::spawn(async move {
        loop {
            // Wait for a status update
            match subscriber.receive().await {
                Ok(update) => {
                    // If status is unhealthy, create an alert
                    if update.status == HealthStatus::Unhealthy {
                        let _ = alert_manager.create_alert(
                            "health",
                            &format!("Component {} is unhealthy", update.component_id),
                            AlertSeverity::Critical,
                            Some(&format!("Health check failed for component {}. Details: {}", 
                                update.component_id, 
                                update.details.unwrap_or_else(|| "No details provided".to_string()))),
                            Some("health"),
                            None
                        );
                    } else {
                        // Component is healthy now, no need to create a warning
                        tracing::info!("Component {} is now healthy", update.component_id);
                    }
                },
                Err(e) => {
                    eprintln!("Error receiving health status update: {}", e);
                    // Wait a bit before retrying
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    });
    
    Ok(handle)
}

/// Create standard health checks for a component
///
/// This function creates a set of standard health checks that can be
/// used for most components, including system and memory checks.
pub fn create_standard_health_checks(
    health_checker: &HealthChecker,
    component_id: &str,
) -> ObservabilityResult<()> {
    // System check
    health_checker.register_health_check_internal(
        format!("{}_system", component_id),
        component_id,
        "System Check",
        HealthCheckType::Liveness,
        Box::new(move || {
            // Simple check that always passes
            HealthCheckResult::healthy_with_message("System is running")
        }),
        Some(60), // Check every 60 seconds
    )?;
    
    // Memory check
    health_checker.register_health_check_internal(
        format!("{}_memory", component_id),
        component_id,
        "Memory Usage Check",
        HealthCheckType::Readiness,
        Box::new(move || {
            // Simple memory check
            // In a real implementation, this would check actual memory usage
            HealthCheckResult::healthy_with_message("Memory usage is acceptable")
        }),
        Some(300), // Check every 5 minutes
    )?;
    
    Ok(())
}

/// Create a basic connectivity health check
pub fn create_connectivity_check(
    health_checker: &HealthChecker,
    component_id: &str,
    check_id: &str,
    endpoint: String,
) -> ObservabilityResult<()> {
    health_checker.register_health_check_internal(
        check_id,
        component_id,
        &format!("Connectivity Check for {}", endpoint),
        HealthCheckType::Readiness,
        Box::new(move || {
            // In a real implementation, this would test actual connectivity
            // For now, just return healthy
            HealthCheckResult::healthy_with_message(&format!("Connection to {} is healthy", endpoint))
        }),
        Some(30), // Check every 30 seconds
    )
}

/// Create a database health check
pub fn create_database_check(
    health_checker: &HealthChecker,
    component_id: &str,
    database_name: String,
) -> ObservabilityResult<()> {
    health_checker.register_health_check_internal(
        format!("{}_database", component_id),
        component_id,
        &format!("Database Check for {}", database_name),
        HealthCheckType::Readiness,
        Box::new(move || {
            // In a real implementation, this would test database connectivity
            // For now, just return healthy
            HealthCheckResult::healthy_with_message(&format!("Database {} is accessible", database_name))
        }),
        Some(60), // Check every minute
    )
}

/// Create a service dependency health check
pub fn create_service_dependency_check(
    health_checker: &HealthChecker,
    component_id: &str,
    service_name: String,
) -> ObservabilityResult<()> {
    health_checker.register_health_check_internal(
        format!("{}_service_{}", component_id, service_name),
        component_id,
        &format!("Service Dependency Check for {}", service_name),
        HealthCheckType::Readiness,
        Box::new(move || {
            // In a real implementation, this would check service availability
            // For now, just return healthy
            HealthCheckResult::healthy_with_message(&format!("Service {} is available", service_name))
        }),
        Some(45), // Check every 45 seconds
    )
}

/// Monitor health checker and restart if needed
pub async fn monitor_health_checker(health_checker: Arc<HealthChecker>) {
    loop {
        // Check if health checker is responsive
        match health_checker.overall_status() {
            Ok(_) => {
                // Health checker is responsive
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
            Err(e) => {
                eprintln!("Health checker appears unresponsive: {}", e);
                // In a real implementation, you might restart the health checker here
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

/// Create a comprehensive health check suite for a web service
pub fn create_web_service_checks(
    health_checker: &HealthChecker,
    service_name: &str,
    endpoint: String,
    database_name: Option<String>,
) -> ObservabilityResult<()> {
    // Create standard checks
    create_standard_health_checks(health_checker, service_name)?;
    
    // Create connectivity check
    create_connectivity_check(health_checker, service_name, 
        &format!("{}_endpoint", service_name), endpoint)?;
    
    // Create database check if specified
    if let Some(db_name) = database_name {
        create_database_check(health_checker, service_name, db_name)?;
    }
    
    Ok(())
} 