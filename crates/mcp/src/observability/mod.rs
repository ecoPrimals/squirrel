//! # MCP Observability Framework
//! 
//! This module provides a comprehensive observability framework for the Machine Context Protocol,
//! including metrics collection, distributed tracing, structured logging, health checking,
//! and alerting capabilities.
//!
//! ## Core Features
//!
//! - **Metrics Collection**: Collect and expose metrics from various MCP components
//! - **Distributed Tracing**: Track request flows across components and services
//! - **Structured Logging**: Consistent, structured logging with context
//! - **Health Checking**: Component and system-level health monitoring
//! - **Alerting**: Event-based alerting for critical system conditions
//!
//! ## Architecture
//!
//! The Observability Framework follows a modular design with these key components:
//!
//! 1. **Metrics Registry**: Central registry for all system metrics
//! 2. **Tracer**: Distributed tracing implementation
//! 3. **Logger**: Structured logging with context propagation
//! 4. **Health Checker**: Component health status monitoring
//! 5. **Alert Manager**: Alert generation and routing

//! Observability Framework
//!
//! Provides a unified framework for application observability, including:
//! - Metrics collection
//! - Distributed tracing
//! - Structured logging
//! - Health checking
//! - Alerting
//!
//! This module serves as an integration point for all observability components
//! and provides a centralized API for applications to use.

pub mod metrics;
pub mod tracing;
pub mod logging;
pub mod health;
pub mod alerting;
pub mod exporters;
// TODO: Implement the monitoring bridge module
// pub mod monitoring_bridge;

#[cfg(test)]
pub mod tests;

use std::sync::Arc;
use thiserror::Error;

/// Errors that can occur in observability operations
#[derive(Error, Debug)]
pub enum ObservabilityError {
    #[error("Metrics error: {0}")]
    MetricsError(String),
    
    #[error("Tracing error: {0}")]
    TracingError(String),
    
    #[error("Logging error: {0}")]
    LoggingError(String),
    
    #[error("Health checking error: {0}")]
    HealthError(String),
    
    #[error("Alerting error: {0}")]
    AlertingError(String),
    
    #[error("Initialization error: {0}")]
    InitializationError(String),
    
    #[error("External system error: {0}")]
    External(String),
}

/// Result type for observability operations
pub type ObservabilityResult<T> = std::result::Result<T, ObservabilityError>;

/// Main entry point for the Observability Framework
/// 
/// Provides access to all observability components through a unified API.
#[derive(Clone)]
pub struct ObservabilityFramework {
    metrics_registry: Arc<metrics::MetricsRegistry>,
    tracer: Arc<tracing::Tracer>,
    health_checker: Arc<health::HealthChecker>,
    alert_manager: Arc<alerting::AlertManager>,
    // Logger is accessed through thread-local storage
}

impl ObservabilityFramework {
    /// Create a new ObservabilityFramework with default configuration
    pub fn new() -> ObservabilityResult<Self> {
        // Initialize metrics registry with default configuration
        let metrics_registry = Arc::new(metrics::MetricsRegistry::new());
        
        // Initialize tracer with default configuration
        let _tracer_config = tracing::TracerConfig {
            enabled: true,
            sampling_rate: 1.0, // Sample all traces
            max_spans: 1000,
        };
        let tracer = Arc::new(tracing::Tracer::new());
        
        // Initialize health checker with default configuration
        let health_checker = Arc::new(health::HealthChecker::new());
        
        // Initialize alert manager with default configuration
        let _alert_config = alerting::AlertManagerConfig {
            retention_time: std::time::Duration::from_secs(86400), // 24 hours
            max_alerts: 1000,
            notification_buffer: 100,
        };
        let alert_manager = Arc::new(alerting::AlertManager::new());
        
        // Initialize logger with default configuration (done globally)
        let logger_config = logging::LoggerConfig {
            default_level: logging::LogLevel::Info,
            component_levels: std::collections::HashMap::new(),
            include_trace_context: true,
        };
        
        // Create a logger instance to initialize
        let logger = logging::Logger::new();
        logger.set_config(logger_config)?;
        logger.initialize()?;
        
        Ok(Self {
            metrics_registry,
            tracer,
            health_checker,
            alert_manager,
        })
    }
    
    /// Get a reference to the metrics registry
    pub fn metrics(&self) -> &Arc<metrics::MetricsRegistry> {
        &self.metrics_registry
    }
    
    /// Get a reference to the tracer
    pub fn tracer(&self) -> &Arc<tracing::Tracer> {
        &self.tracer
    }
    
    /// Get a reference to the health checker
    pub fn health_checker(&self) -> &Arc<health::HealthChecker> {
        &self.health_checker
    }
    
    /// Get a reference to the alert manager
    pub fn alert_manager(&self) -> &Arc<alerting::AlertManager> {
        &self.alert_manager
    }
}

/// Initialize the Observability Framework with default configuration
///
/// This is the recommended way to initialize the framework for most applications.
pub fn initialize() -> ObservabilityResult<ObservabilityFramework> {
    ObservabilityFramework::new()
}

/// Error handling for observability
pub mod error;

/// Health check subsystem
// ... existing code ...

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::collections::HashMap;
    
    mod metrics_tests {
        use super::*;
        use crate::observability::metrics::{Counter, Gauge, Histogram, MetricsRegistry, Labels};
        
        #[test]
        fn test_counter_operations() {
            let registry = MetricsRegistry::new();
            let labels: Labels = HashMap::new();
            
            // Create a counter
            let counter = registry.create_counter(
                "test_counter", 
                "Test counter", 
                None, 
                labels.clone()
            ).unwrap();
            
            // Test increment operations
            counter.inc(5).unwrap();
            assert_eq!(counter.value().unwrap(), 5);
            
            counter.inc_one().unwrap();
            assert_eq!(counter.value().unwrap(), 6);
            
            // Create a counter with labels
            let mut label_keys: Labels = HashMap::new();
            label_keys.insert("label1".to_string(), "value1".to_string());
            label_keys.insert("label2".to_string(), "value2".to_string());
            
            let labeled_counter = registry.create_counter(
                "labeled_counter", 
                "Labeled counter", 
                None,
                label_keys
            ).unwrap();
            
            labeled_counter.inc(10).unwrap();
            assert_eq!(labeled_counter.value().unwrap(), 10);
            
            // Test get_counter retrieval
            let retrieved_counter = registry.get_counter("test_counter").unwrap().unwrap();
            assert_eq!(retrieved_counter.value().unwrap(), 6);
        }
        
        #[test]
        fn test_gauge_operations() {
            let registry = MetricsRegistry::new();
            let labels: Labels = HashMap::new();
            
            // Create a gauge
            let gauge = registry.create_gauge(
                "test_gauge", 
                "Test gauge", 
                None,
                labels
            ).unwrap();
            
            // Test set operation
            gauge.set(3.14).unwrap();
            assert!((gauge.value().unwrap() - 3.14).abs() < 1e-10);
            
            // Test increment and decrement
            gauge.inc(1.0).unwrap();
            assert!((gauge.value().unwrap() - 4.14).abs() < 1e-10);
            
            gauge.dec(2.0).unwrap();
            assert!((gauge.value().unwrap() - 2.14).abs() < 1e-10);
            
            // Test get_gauge retrieval
            let retrieved_gauge = registry.get_gauge("test_gauge").unwrap().unwrap();
            assert!((retrieved_gauge.value().unwrap() - 2.14).abs() < 1e-10);
        }
        
        #[test]
        fn test_histogram_operations() {
            let registry = MetricsRegistry::new();
            let labels: Labels = HashMap::new();
            
            // Create a histogram with bucket boundaries
            let bucket_boundaries = vec![1.0, 5.0, 10.0, 50.0, 100.0];
            let histogram = registry.create_histogram(
                "test_histogram", 
                "Test histogram", 
                None,
                labels,
                bucket_boundaries
            ).unwrap();
            
            // Record some values
            histogram.observe(3.0).unwrap();
            histogram.observe(7.0).unwrap();
            histogram.observe(99.0).unwrap();
            
            // Get the count
            assert_eq!(histogram.count().unwrap(), 3);
            
            // Get the sum
            assert_eq!(histogram.sum().unwrap(), 109.0);
            
            // Test get_histogram retrieval
            let retrieved_histogram = registry.get_histogram("test_histogram").unwrap().unwrap();
            assert_eq!(retrieved_histogram.count().unwrap(), 3);
        }
    }
    
    mod tracing_tests {
        use super::*;
        use crate::observability::tracing::{Tracer, TracerConfig};
        use std::thread;
        use std::time::Duration;
        
        #[test]
        fn test_span_creation() {
            let tracer = Tracer::new();
            
            // Create a span
            let span = tracer.start_span("test_span").unwrap();
            
            // Add some attributes to the span
            {
                let mut span_guard = span.lock().unwrap();
                span_guard.add_attribute("key1", "value1");
                span_guard.add_attribute("key2", "value2");
                
                // Add an event
                let mut attrs = HashMap::new();
                attrs.insert("event_key".to_string(), "event_value".to_string());
                span_guard.add_event("test_event", attrs);
                
                // Span should not be ended yet
                assert!(!span_guard.is_ended());
            }
            
            // End the span (via drop)
            drop(span);
            
            // Start a span with a parent
            let parent_span = tracer.start_span("parent_span").unwrap();
            let child_span = tracer.start_span_with_parent("child_span", Some(parent_span.clone())).unwrap();
            
            // We can't end the spans directly through the guard because end() takes ownership
            // Instead, we drop the spans which will end them automatically
            drop(child_span);
            drop(parent_span);
        }
        
        #[test]
        fn test_tracer_configuration() {
            let tracer = Tracer::new();
            
            // Set a custom configuration
            let config = TracerConfig {
                enabled: true,
                sampling_rate: 0.5,
                max_spans: 500,
            };
            
            tracer.set_config(config).unwrap();
            
            // Even with 50% sampling, we should be able to create some spans
            // Run enough iterations to have a high probability of success
            let mut spans_created = 0;
            for _ in 0..100 {
                match tracer.start_span("test_span") {
                    Ok(_) => spans_created += 1,
                    Err(_) => {}, // Sampling might reject this span
                }
            }
            
            // With 50% sampling rate, we expect around 50 spans to be created
            // Allow for some statistical variation (very unlikely to fail)
            assert!(spans_created > 10);
        }
    }
    
    mod health_tests {
        use super::*;
        use crate::observability::health::{HealthChecker, HealthStatus, HealthCheckResult};
        
        #[test]
        fn test_health_checks() {
            let health_checker = health::HealthChecker::new();
            
            // Create a health check that returns healthy
            let healthy_check = health::HealthCheck::new(
                "test_healthy",
                "test_component",
                "A test check that always returns healthy",
                Box::new(|| HealthCheckResult::healthy("Service is healthy"))
            );
            
            // Create a health check that returns unhealthy
            let unhealthy_check = health::HealthCheck::new(
                "test_unhealthy",
                "test_component",
                "A test check that always returns unhealthy",
                Box::new(|| HealthCheckResult::unhealthy("Service is unhealthy"))
            );
            
            // Register the checks
            let healthy_check_ref = health_checker.register_check(healthy_check).unwrap();
            let unhealthy_check_ref = health_checker.register_check(unhealthy_check).unwrap();
            
            // Execute the checks
            let healthy_result = healthy_check_ref.execute().unwrap();
            let unhealthy_result = unhealthy_check_ref.execute().unwrap();
            
            // Verify the results
            assert_eq!(healthy_result.status(), HealthStatus::Healthy);
            assert_eq!(unhealthy_result.status(), HealthStatus::Unhealthy);
            
            assert_eq!(healthy_result.message(), "Service is healthy");
            assert_eq!(unhealthy_result.message(), "Service is unhealthy");
        }
    }
    
    mod alerting_tests {
        use super::*;
        use crate::observability::alerting::{AlertManager, Alert, AlertSeverity, AlertType, AlertState};
        
        #[test]
        fn test_alert_creation() {
            let alert_manager = alerting::AlertManager::new();
            
            // Create an alert
            let alert = Alert::new(
                "test_component",
                "Test alert",
                "This is a test alert",
                AlertSeverity::Warning,
                AlertType::Custom
            );
            
            // Publish the alert
            let published_alert = alert_manager.publish_alert(alert).unwrap();
            
            // Verify the alert
            assert_eq!(published_alert.source(), "test_component");
            assert_eq!(published_alert.summary(), "Test alert");
            assert_eq!(published_alert.description(), "This is a test alert");
            assert_eq!(published_alert.severity(), AlertSeverity::Warning);
            assert_eq!(published_alert.alert_type(), AlertType::Custom);
            assert_eq!(published_alert.state(), AlertState::Active);
            
            // Get the alert by ID
            let retrieved_alert = alert_manager.get_alert(published_alert.id()).unwrap().unwrap();
            assert_eq!(retrieved_alert.id(), published_alert.id());
            
            // Acknowledge the alert
            assert!(alert_manager.acknowledge_alert(published_alert.id()).unwrap());
            
            // Get the alert again to verify it's acknowledged
            let acknowledged_alert = alert_manager.get_alert(published_alert.id()).unwrap().unwrap();
            assert_eq!(acknowledged_alert.state(), AlertState::Acknowledged);
            
            // Resolve the alert
            assert!(alert_manager.resolve_alert(published_alert.id()).unwrap());
            
            // Get the alert again to verify it's resolved
            let resolved_alert = alert_manager.get_alert(published_alert.id()).unwrap().unwrap();
            assert_eq!(resolved_alert.state(), AlertState::Resolved);
        }
        
        #[test]
        fn test_alert_filtering() {
            let alert_manager = alerting::AlertManager::new();
            
            // Create and publish multiple alerts with different properties
            let alert1 = Alert::new(
                "component1",
                "Alert 1",
                "Description 1",
                AlertSeverity::Info,
                AlertType::Custom
            );
            
            let alert2 = Alert::new(
                "component2",
                "Alert 2",
                "Description 2", 
                AlertSeverity::Warning,
                AlertType::ResourceUsage
            );
            
            let alert3 = Alert::new(
                "component1",
                "Alert 3",
                "Description 3",
                AlertSeverity::Critical,
                AlertType::HealthStatus
            );
            
            alert_manager.publish_alert(alert1).unwrap();
            alert_manager.publish_alert(alert2).unwrap();
            alert_manager.publish_alert(alert3).unwrap();
            
            // Filter by source
            let component1_alerts = alert_manager.get_alerts(Some("component1"), None, None, None).unwrap();
            assert_eq!(component1_alerts.len(), 2);
            
            // Filter by severity
            let critical_alerts = alert_manager.get_alerts(None, Some(AlertSeverity::Critical), None, None).unwrap();
            assert_eq!(critical_alerts.len(), 1);
            assert_eq!(critical_alerts[0].summary(), "Alert 3");
            
            // Filter by type
            let health_alerts = alert_manager.get_alerts(None, None, Some(AlertType::HealthStatus), None).unwrap();
            assert_eq!(health_alerts.len(), 1);
            
            // Filter by state (all are active initially)
            let active_alerts = alert_manager.get_alerts(None, None, None, Some(AlertState::Active)).unwrap();
            assert_eq!(active_alerts.len(), 3);
            
            // Acknowledge one alert and check filtering
            alert_manager.acknowledge_alert(component1_alerts[0].id()).unwrap();
            let acknowledged_alerts = alert_manager.get_alerts(None, None, None, Some(AlertState::Acknowledged)).unwrap();
            assert_eq!(acknowledged_alerts.len(), 1);
        }
    }
    
    mod logging_tests {
        use super::*;
        use crate::observability::logging::{Logger, LogLevel, LogContext};
        
        #[test]
        fn test_log_levels() {
            let logger = logging::Logger::new();
            
            // Set the default log level to Info
            logger.set_default_level(LogLevel::Info).unwrap();
            
            // These should be logged (Info level and above)
            logger.info("Info message", "test_component", None).unwrap();
            logger.warning("Warning message", "test_component", None).unwrap();
            logger.error("Error message", "test_component", None).unwrap();
            logger.critical("Critical message", "test_component", None).unwrap();
            
            // Set a component-specific level
            logger.set_component_level("debug_component", LogLevel::Debug).unwrap();
            
            // Debug should be logged for the debug_component but not for others
            // (In a real test, we would check if the logs were actually output)
        }
        
        #[test]
        fn test_logging_context() {
            // Create a context with fields
            let context = LogContext::new()
                .with_field("user_id", "12345")
                .with_field("request_id", "abc-123")
                .with_trace_id("trace-123")
                .with_span_id("span-456");
            
            // Log with context
            let logger = logging::Logger::new();
            logger.info("Message with context", "test_component", Some(&context)).unwrap();
            
            // Test thread-local context manager
            logging::ContextManager::set_context(context.clone()).unwrap();
            let retrieved_context = logging::ContextManager::current_context().unwrap();
            
            assert_eq!(retrieved_context.fields().get("user_id"), Some(&"12345".to_string()));
            assert_eq!(retrieved_context.trace_id(), Some("trace-123"));
            
            // Add a field to the current context
            logging::ContextManager::add_field("session_id", "session-789").unwrap();
            let updated_context = logging::ContextManager::current_context().unwrap();
            
            assert_eq!(updated_context.fields().get("session_id"), Some(&"session-789".to_string()));
            
            // Clear the context
            logging::ContextManager::clear_context().unwrap();
            let cleared_context = logging::ContextManager::current_context().unwrap();
            
            assert!(cleared_context.fields().is_empty());
            assert_eq!(cleared_context.trace_id(), None);
        }
    }
    
    mod framework_tests {
        use super::*;
        
        #[test]
        fn test_framework_initialization() {
            // Initialize the framework
            let framework = initialize().unwrap();
            
            // Check that all components are accessible and non-null
            assert!(framework.metrics().counter_names().is_ok());
            assert!(framework.tracer().initialize().is_ok());
            assert!(framework.health_checker().initialize().is_ok());
            assert!(framework.alert_manager().cleanup_old_alerts().is_ok());
            
            // Test using the components through the framework
            let counter = framework.metrics().create_counter(
                "framework_test_counter",
                "A counter created through the framework",
                None,
                HashMap::new()
            ).unwrap();
            
            counter.inc_one().unwrap();
            assert_eq!(counter.value().unwrap(), 1);
        }
    }
}

// Add a convenient function to create a dashboard exporter
/// Create a dashboard exporter for tracing visualization
pub fn create_dashboard_exporter(
    dashboard_url: &str, 
    service_name: &str,
    export_interval_secs: u64,
) -> ObservabilityResult<Box<dyn tracing::external::SpanExporter>> {
    use crate::observability::exporters::dashboard_exporter;
    use crate::observability::tracing::external::ExternalTracingConfig;
    
    let config = ExternalTracingConfig {
        endpoint_url: dashboard_url.to_string(),
        auth_token: None,
        flush_interval_seconds: export_interval_secs,
        max_buffer_size: 100,
        add_standard_attributes: true,
        service_name: service_name.to_string(),
        environment: "development".to_string(),
    };
    
    Ok(dashboard_exporter::create_dashboard_exporter(config))
} 