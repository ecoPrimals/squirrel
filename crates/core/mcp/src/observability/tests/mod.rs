// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Integration Tests for Observability Framework
//!
//! These tests verify the integration between different components of
//! the observability framework, including metrics, tracing, logging,
//! health checks, and alerting.

use std::collections::HashMap;
use std::sync::Arc;

use crate::observability::metrics::MetricsRegistry;
use crate::observability::tracing::Tracer;
use crate::observability::logging::{LogContext, Logger};
use crate::observability::health::{HealthChecker, HealthStatus, HealthCheckResult};
use crate::observability::alerting::{AlertManager, AlertSeverity, AlertType, AlertState};
use crate::observability::{ObservabilityResult, ObservabilityError};
use crate::observability::{ObservabilityFramework, ObservabilityConfig, initialize_with_config};

pub mod external_tracing_test;
pub mod dashboard_integration_test;

/// Test the integration between metrics and tracing components
#[test]
fn test_metrics_tracing_integration() -> ObservabilityResult<()> {
    // Initialize components
    let metrics = MetricsRegistry::new();
    metrics.initialize()?;
    
    let tracer = Tracer::new();
    tracer.initialize()?;
    
    // Create metrics
    let counter = metrics.create_counter(
        "request_count", 
        "Total number of requests", 
        Some("count".to_string()), 
        HashMap::new()
    )?;
    
    let histogram = metrics.create_histogram(
        "request_duration", 
        "Duration of requests in ms", 
        Some("ms".to_string()), 
        HashMap::new(),
        vec![10.0, 50.0, 100.0, 200.0, 500.0, 1000.0] // Required bucket boundaries
    )?;
    
    // Process simulated requests with tracing
    for i in 1..=10 {
        // Start a span for this request
        let span = tracer.start_span(format!("process_request_{}", i))?;
        
        {
            let mut span_guard = span.lock().map_err(|_| 
                ObservabilityError::TracingError("Failed to lock span".to_string()))?;
            
            // Add request details as span attributes
            span_guard.add_attribute("request_id", format!("req-{}", i));
            span_guard.add_attribute("request_type", "GET");
            
            // Record metrics
            counter.inc(1)?; // Using u64 instead of f64
            
            // Simulate processing time
            let duration = 50.0 + (i as f64 * 5.0);
            histogram.observe(duration)?;
            
            // Add event to span
            let mut event_attrs = HashMap::new();
            event_attrs.insert("duration_ms".to_string(), duration.to_string());
            span_guard.add_event("request_completed", event_attrs);
        }
        
        // Note: We don't call span.lock().unwrap().end() here because that would
        // move out of the MutexGuard. The drop handler will handle ending.
    }
    
    // Verify metrics
    assert_eq!(counter.value()?, 10);
    assert_eq!(histogram.count()?, 10);
    
    Ok(())
}

/// Test the integration between health checks and alerting
#[test]
fn test_health_alerts_integration() -> ObservabilityResult<()> {
    // Initialize components
    let health_checker = HealthChecker::new();
    health_checker.initialize()?;
    
    let alert_manager = AlertManager::new();
    alert_manager.initialize()?;
    
    // Register health checks
    let _db_check = health_checker.register_check_fn(
        "database_connection",
        "database",
        "Checks database connectivity",
        || {
            // Simulate a database that sometimes fails
            if rand::random::<f32>() < 0.3 {
                HealthCheckResult::unhealthy("Database connection failed")
                    .with_detail("error", "Connection timeout")
            } else {
                HealthCheckResult::healthy_with_message("Database connection successful")
                    .with_detail("latency_ms", "15")
            }
        }
    )?;
    
    // Execute health checks and trigger alerts based on results
    for _ in 0..5 {
        // Check the database component
        let _results = health_checker.check_component("database")?;
        
        // Get the overall status
        let status = health_checker.component_status("database")?;
        
        // If unhealthy, create an alert
        if status == HealthStatus::Unhealthy {
            let _alert = alert_manager.alert(
                "database",
                "Database health check failed",
                "The database connectivity check is reporting unhealthy status",
                AlertSeverity::Error,
                AlertType::HealthStatus
            )?;
        }
    }
    
    // Verify alerts
    let active_alerts = alert_manager.get_alerts(
        None, 
        None, 
        None, 
        Some(AlertState::Active)
    )?;
    
    println!("Generated {} active alerts", active_alerts.len());
    
    Ok(())
}

/// Test the integration between tracing and logging
#[test]
fn test_tracing_logging_integration() -> ObservabilityResult<()> {
    // Initialize components
    let tracer = Tracer::new();
    tracer.initialize()?;
    
    let logger = Logger::new();
    logger.initialize()?;
    
    // Create a trace context
    let span = tracer.start_span("process_payment")?;
    
    {
        let mut span_guard = span.lock().map_err(|_| 
            ObservabilityError::TracingError("Failed to lock span".to_string()))?;
        
        // Add span attributes
        span_guard.add_attribute("payment_id", "pmt-12345");
        span_guard.add_attribute("amount", "99.95");
        
        // Create log entries associated with this span
        let log_context = LogContext::new()
            .with_field("payment_id", "pmt-12345")
            .with_trace_id(span_guard.span().trace_id())
            .with_span_id(span_guard.span().id());
        
        // Log steps in the process
        logger.info(
            "Payment processing started", 
            "payment_service", 
            Some(&log_context)
        )?;
        
        // Add an event to the span
        let mut event_attrs = HashMap::new();
        event_attrs.insert("verification_status".to_string(), "success".to_string());
        span_guard.add_event("payment_verified", event_attrs);
        
        // Log more details
        let log_context = log_context.with_field("verification_time_ms", "150");
        logger.info(
            "Payment verification complete", 
            "payment_service", 
            Some(&log_context)
        )?;
        
        // Log completion
        logger.info(
            "Payment processing complete", 
            "payment_service", 
            Some(&log_context)
        )?;
    }
    
    Ok(())
}

/// Test a complete observability pipeline
#[test]
fn test_complete_observability_pipeline() -> ObservabilityResult<()> {
    // Initialize all components
    let metrics = MetricsRegistry::new();
    metrics.initialize()?;
    
    let tracer = Tracer::new();
    tracer.initialize()?;
    
    let logger = Logger::new();
    logger.initialize()?;
    
    let health_checker = HealthChecker::new();
    health_checker.initialize()?;
    
    let alert_manager = AlertManager::new();
    alert_manager.initialize()?;
    
    // Register health checks
    health_checker.register_check_fn(
        "api_service",
        "api",
        "Checks API service health",
        || HealthCheckResult::healthy_with_message("API service running")
    )?;
    
    // Create metrics
    let req_counter = metrics.create_counter(
        "api_requests_total", 
        "Total API requests", 
        Some("count".to_string()), 
        HashMap::new()
    )?;
    
    let error_counter = metrics.create_counter(
        "api_errors_total", 
        "Total API errors", 
        Some("count".to_string()), 
        HashMap::new()
    )?;
    
    // Process a simulated API request
    let span = tracer.start_span("process_api_request")?;
    
    {
        let mut span_guard = span.lock().map_err(|_| 
            ObservabilityError::TracingError("Failed to lock span".to_string()))?;
        
        // Add request details
        span_guard.add_attribute("request_id", "req-abc123");
        span_guard.add_attribute("client_ip", "192.168.1.1");
        
        // Update metrics
        req_counter.inc(1)?;
        
        // Create log context
        let log_context = LogContext::new()
            .with_field("request_id", "req-abc123")
            .with_trace_id(span_guard.span().trace_id())
            .with_span_id(span_guard.span().id());
        
        // Log request receipt
        logger.info(
            "API request received", 
            "api_service", 
            Some(&log_context)
        )?;
        
        // Check health before processing
        health_checker.check_component("api")?;
        let api_status = health_checker.component_status("api")?;
        
        if api_status != HealthStatus::Healthy {
            // Create alert if API is not healthy
            let _alert = alert_manager.alert(
                "api",
                "API health degraded during request",
                "The API service is not fully healthy while processing requests",
                AlertSeverity::Warning,
                AlertType::HealthStatus
            )?;
            
            // Log error and update metrics
            logger.error(
                "Cannot process request due to API health", 
                "api_service", 
                Some(&log_context)
            )?;
            error_counter.inc(1)?;
        } else {
            // Log successful processing
            logger.info(
                "API request processed successfully", 
                "api_service", 
                Some(&log_context)
            )?;
        }
    }
    
    Ok(())
}

/// Test metrics performance
#[test]
fn test_metrics_performance() -> ObservabilityResult<()> {
    let metrics = MetricsRegistry::new();
    metrics.initialize()?;
    
    let counter = metrics.create_counter(
        "perf_test_counter", 
        "Performance test counter", 
        Some("count".to_string()), 
        HashMap::new()
    )?;
    
    // Record many metrics operations in a tight loop
    let start = std::time::Instant::now();
    for _ in 1..=10000 {
        counter.inc(1)?;
    }
    let duration = start.elapsed();
    
    println!("Recorded 10000 counter increments in {:?}", duration);
    assert_eq!(counter.value()?, 10000);
    
    Ok(())
}

/// Test tracing performance
#[test]
fn test_tracing_performance() -> ObservabilityResult<()> {
    let tracer = Tracer::new();
    tracer.initialize()?;
    
    // Create and end spans in a tight loop
    let start = std::time::Instant::now();
    for i in 1..=1000 {
        let span = tracer.start_span(format!("perf_span_{}", i))?;
        
        {
            let mut span_guard = span.lock().map_err(|_| 
                ObservabilityError::TracingError("Failed to lock span".to_string()))?;
            span_guard.add_attribute("index", i.to_string());
        }
        
        // Span will be automatically ended via drop
    }
    let duration = start.elapsed();
    
    println!("Created and ended 1000 spans in {:?}", duration);
    
    Ok(())
}

/// Test logging performance
#[test]
fn test_logging_performance() -> ObservabilityResult<()> {
    let logger = Logger::new();
    logger.initialize()?;
    
    // Log many messages in a tight loop
    let start = std::time::Instant::now();
    for i in 1..=10000 {
        logger.info(
            &format!("Performance test log message {}", i),
            "performance_test",
            None
        )?;
    }
    let duration = start.elapsed();
    
    println!("Logged 10000 messages in {:?}", duration);
    
    Ok(())
}

/// Test error handling across all components
#[test]
fn test_error_handling() -> ObservabilityResult<()> {
    // Initialize components
    let metrics = MetricsRegistry::new();
    let tracer = Tracer::new();
    let logger = Logger::new();
    let health_checker = HealthChecker::new();
    let alert_manager = AlertManager::new();
    
    // Test error propagation
    let result = metrics.create_counter(
        "", // Invalid name
        "Test counter",
        None,
        HashMap::new()
    );
    assert!(result.is_err());
    
    // Test error handling in tracing
    let span_result = tracer.start_span("test_span");
    assert!(span_result.is_ok());
    
    Ok(())
}

// Integration tests for the Observability Framework
//
// This module contains integration tests that verify the functionality
// of the Observability Framework components working together.

// Missing modules - uncomment when implementations are available
// mod dashboard_integration_tests;
// mod framework_tests;

// Re-export test utilities for use in other test modules
// These are currently missing - uncomment when implemented
// pub use super::metrics::tests::helpers as metrics_test_helpers;
// pub use super::tracing::tests::helpers as tracing_test_helpers;
// pub use super::health::tests::helpers as health_test_helpers;
// pub use super::alerting::tests::helpers as alerting_test_helpers;

// Test utilities for the Observability Framework
pub mod helpers {
    use std::sync::Arc;
    use std::collections::HashMap;
    
    use crate::observability::{
        ObservabilityFramework, ObservabilityConfig,
        initialize_with_config,
    };
    
    /// Create a test framework for unit tests
    pub async fn create_test_framework() -> Arc<ObservabilityFramework> {
        // Create a minimal config for tests
        let config = ObservabilityConfig {
            enable_dashboard: false,
            enable_metrics: true,
            enable_health_checks: true,
            enable_alerting: true,
            // Use defaults for the rest
            ..Default::default()
        };
        
        // Initialize with the config
        Arc::new(initialize_with_config(config).await.unwrap())
    }
    
    /// Create a full test framework with all features enabled
    pub async fn create_full_test_framework() -> Arc<ObservabilityFramework> {
        // Create a full config for tests
        let config = ObservabilityConfig {
            enable_dashboard: true,
            enable_metrics: true,
            enable_health_checks: true,
            enable_alerting: true,
            default_log_level: log::Level::Debug,
            include_trace_context_in_logs: true,
            enable_tracing: true,
            // Use defaults for the rest
            ..Default::default()
        };
        
        // Initialize with the config
        Arc::new(initialize_with_config(config).await.unwrap())
    }
    
    /// Create test metrics for the framework
    pub fn create_test_metrics(framework: &ObservabilityFramework) {
        let mut labels = HashMap::new();
        labels.insert("component".to_string(), "test".to_string());
        
        // Create test metrics
        let _counter = framework.metrics.create_counter(
            "test_counter",
            "Test counter",
            None,
            labels.clone(),
        ).unwrap();
        
        let _gauge = framework.metrics.create_gauge(
            "test_gauge",
            "Test gauge",
            None,
            labels.clone(),
        ).unwrap();
        
        let _histogram = framework.metrics.create_histogram(
            "test_histogram",
            "Test histogram",
            None,
            labels,
            vec![0.1, 1.0, 10.0],
        ).unwrap();
    }
    
    /// Register test components for health checking
    pub fn register_test_components(framework: &ObservabilityFramework) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            framework.health_checker.register_component(
                "test_component_1",
                "Test Component 1",
                crate::observability::health::HealthStatus::Healthy,
            ).await.unwrap();
            
            framework.health_checker.register_component(
                "test_component_2",
                "Test Component 2",
                crate::observability::health::HealthStatus::Healthy,
            ).await.unwrap();
        });
    }
}

// Add example_tests module
// mod example_tests;

/// Test framework initialization and basic usage
#[tokio::test]
async fn test_framework_initialization() {
    // Create a test framework
    // let framework = create_test_framework().await;
    let framework = helpers::create_test_framework().await;
    
    // Create empty labels HashMap
    let labels = std::collections::HashMap::new();
    
    // Test metrics API
    let _counter = framework.metrics.create_counter(
        "test_counter",
        "Test counter",
        None,
        labels.clone(),
    ).unwrap();
    
    let _gauge = framework.metrics.create_gauge(
        "test_gauge",
        "Test gauge",
        None,
        labels.clone(),
    ).unwrap();
    
    let _histogram = framework.metrics.create_histogram(
        "test_histogram",
        "Test histogram",
        None,
        labels.clone(),
        vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0],
    ).unwrap();
    
    // Test health API
    framework.health_checker.register_component(
        "api",
        "API Service",
        HealthStatus::Healthy,
    ).await.unwrap();
    
    framework.health_checker.register_component(
        "database",
        "Database Service",
        HealthStatus::Healthy,
    ).await.unwrap();
    
    // Clean shutdown
    framework.shutdown().await.unwrap();
}

/// Test creating a basic observability framework
#[tokio::test]
async fn test_create_framework() {
    // Create a basic config with minimal settings
    let config = ObservabilityConfig {
        enable_dashboard: false,
        enable_metrics: true,
        enable_health_checks: true,
        enable_alerting: true,
        // Use defaults for the rest
        ..Default::default()
    };
    
    // Initialize the framework
    let framework = initialize_with_config(config).await.unwrap();
    assert!(framework.is_initialized());
    
    // Register a test component
    framework.health_checker.register_component(
        "test_component",
        "Test component for integration test",
        HealthStatus::Healthy,
    ).await.unwrap();
    
    // Create a test alert
    framework.alert_manager.create_alert(
        "test_alert_id",
        "Test alert",
        AlertSeverity::Info,
        Some("This is a test alert from integration tests"),
        Some("test_component"),
        None,
    ).unwrap();
    
    // Set a test metric
    framework.metrics.increment_counter("test_counter", 1.0, None).unwrap();
    
    // Clean shutdown
    framework.shutdown().await.unwrap();
}

pub mod alerting_tests;
pub mod logging_tests;
pub mod metrics_tests;
pub mod tracing_tests;
pub mod health;

pub mod test_helpers; 