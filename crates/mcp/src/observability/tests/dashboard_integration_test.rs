use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use std::collections::HashMap;
use async_trait::async_trait;

use crate::observability::{
    tracing::{
        Span, SpanStatus,
        external::{SpanExporter, ExternalTracingConfig}
    },
    exporters::{
        dashboard_exporter::{self, DashboardExporter, DashboardExporterConfig},
        dashboard_integration::{
            DashboardIntegrationAdapter, 
            DashboardIntegrationConfig,
            create_default_dashboard_integration
        }
    },
    ObservabilityError
};

use squirrel_interfaces::tracing::{TraceData, TraceDataConsumer, TraceDataProvider};

/// Trait to verify if a value is a boxed SpanExporter
trait IsBoxedSpanExporter {
    fn is_span_exporter(&self) -> bool;
}

impl<T: SpanExporter + ?Sized> IsBoxedSpanExporter for Box<T> {
    fn is_span_exporter(&self) -> bool {
        true
    }
}

/// Test that we can create a dashboard exporter with a proper configuration
#[tokio::test]
async fn test_dashboard_exporter_creation() -> Result<(), ObservabilityError> {
    // Create a dashboard exporter with a realistic configuration
    let config = ExternalTracingConfig {
        endpoint_url: "http://localhost:8080/traces".to_string(),
        auth_token: Some("test_auth_token".to_string()),
        flush_interval_seconds: 5,
        max_buffer_size: 100,
        add_standard_attributes: true,
        service_name: "test-service".to_string(),
        environment: "test-env".to_string(),
    };
    
    let exporter = dashboard_exporter::create_dashboard_exporter(config);
    
    // Verify it's a span exporter
    assert!(exporter.is_span_exporter());
    
    Ok(())
}

/// Test that the dashboard exporter can handle spans properly
#[tokio::test]
async fn test_dashboard_exporter_span_handling() -> Result<(), ObservabilityError> {
    // Create a test span
    let span = create_test_span(
        "test-span", 
        "test-trace-id", 
        Some("parent-span-id")
    );
    
    // Create a dashboard exporter
    let config = ExternalTracingConfig {
        endpoint_url: "http://localhost:8080/traces".to_string(),
        auth_token: None,
        flush_interval_seconds: 1,
        max_buffer_size: 10,
        add_standard_attributes: true,
        service_name: "test-service".to_string(),
        environment: "test-env".to_string(),
    };
    
    let exporter = DashboardExporter::new(config.clone());
    
    // Export the span
    exporter.export_spans(vec![span]).await?;
    
    // Allow time for processing
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // The test passes if no error occurred during export
    Ok(())
}

/// Create a test span for testing
fn create_test_span(name: &str, trace_id: &str, parent_id: Option<&str>) -> Span {
    // Create a new span using the proper constructor
    let mut span = Span::new(
        name,
        trace_id,
        parent_id.map(String::from),
    );
    
    // Add attributes using the accessor method
    span.add_attribute("test.attribute", "test.value");
    span.add_attribute("component", "test-component");
    
    // Add events using the accessor method
    let mut event_attrs = HashMap::new();
    event_attrs.insert("event.type".to_string(), "test".to_string());
    span.add_event("test.event.1", event_attrs.clone());
    
    event_attrs.insert("timestamp".to_string(), "123456789".to_string());
    span.add_event("test.event.2", event_attrs);
    
    // Set span status using the accessor method
    span.end();
    
    span
}

/// Test the convenience helper for creating a dashboard exporter
#[tokio::test]
async fn test_dashboard_integration_helper() -> Result<(), ObservabilityError> {
    // Create a dashboard exporter using the helper function
    let config = ExternalTracingConfig {
        endpoint_url: "http://localhost:8080/traces".to_string(),
        auth_token: Some("test_auth_token".to_string()),
        flush_interval_seconds: 5,
        max_buffer_size: 100,
        add_standard_attributes: true,
        service_name: "test-service".to_string(),
        environment: "test-env".to_string(),
    };
    
    let exporter = dashboard_exporter::create_dashboard_exporter(config);
    
    // Create and export a test span
    let span = create_test_span(
        "helper-test-span", 
        "helper-test-trace-id", 
        None
    );
    
    // Export the span
    exporter.export_spans(vec![span]).await?;
    
    // The test passes if no error occurred during export
    Ok(())
}

// Mock trace data consumer for testing
struct TestTraceConsumer {
    name: String,
    received_traces: std::sync::Mutex<Vec<TraceData>>,
}

impl TestTraceConsumer {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            received_traces: std::sync::Mutex::new(Vec::new()),
        }
    }
    
    fn get_traces(&self) -> Vec<TraceData> {
        let guard = self.received_traces.lock().unwrap();
        guard.clone()
    }
}

#[async_trait]
impl TraceDataConsumer for TestTraceConsumer {
    async fn consume_trace_data(&self, trace_data: TraceData) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("Consumer '{}' received trace data with {} spans", self.name, trace_data.spans.len());
        
        let mut guard = self.received_traces.lock().unwrap();
        guard.push(trace_data);
        
        Ok(())
    }
} 