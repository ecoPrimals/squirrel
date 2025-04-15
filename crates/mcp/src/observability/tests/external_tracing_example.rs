//! # External Tracing Integration Example
//!
//! This module demonstrates how to use the external tracing integration
//! to connect MCP's observability framework with external tracing systems.

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use async_trait::async_trait;

use crate::observability::tracing::{
    Span, SpanStatus,
    external::{
        SpanExporter, ExternalTracingConfig, 
        OpenTelemetryExporter, JaegerExporter, ZipkinExporter
    }
};
use crate::observability::metrics::MetricsCollector;
use crate::observability::ObservabilityResult;

// Import test helpers
mod test_helpers;
use test_helpers::{start_docker_services, check_otel_services, check_jaeger_services, check_zipkin_services};

// This tests the basic functionality of the external tracing integration
#[tokio::test]
async fn test_external_tracing_basic() {
    // Create a mock exporter that just stores spans
    let exporter = MockExporter::new();
    
    // Create a tracer with the mock exporter
    let mut tracer = ExternalTracer::new(exporter.clone());
    
    // Initialize the tracer
    tracer.initialize().await.expect("Failed to initialize tracer");
    
    // Create a new span
    let span = tracer.start_span("test_span").expect("Failed to create span");
    
    // Add attributes to the span
    {
        let mut span_guard = span.lock().unwrap();
        span_guard.add_attribute("test_key", "test_value");
    }
    
    // End the span
    {
        let span = Arc::clone(&span);
        let mut span_guard = span.lock().unwrap();
        span_guard.end();
    }
    
    // Export completed spans
    let exported_count = tracer.export_completed_spans().await.expect("Failed to export spans");
    assert_eq!(exported_count, 1);
    
    // Check that the span was exported
    let spans = exporter.get_spans();
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].name(), "test_span");
    assert_eq!(spans[0].status(), SpanStatus::Success);
    assert_eq!(spans[0].attributes().get("test_key"), Some(&"test_value".to_string()));
}

// This tests the OpenTelemetry exporter functionality
#[tokio::test]
async fn test_opentelemetry_exporter() {
    // Start Docker services and check if OpenTelemetry is available
    if !start_docker_services().await || !check_otel_services().await {
        println!("Skipping test_opentelemetry_exporter: OpenTelemetry collector not available");
        return;
    }

    // Create an OpenTelemetry exporter with a custom configuration
    let config = ExternalTracingConfig {
        endpoint_url: "http://localhost:4318/v1/traces".to_string(),
        service_name: "test-service".to_string(),
        environment: "test".to_string(),
        ..Default::default()
    };
    
    let metrics_collector = Arc::new(MetricsCollector::new("test"));
    let exporter = OpenTelemetryExporter::new(config).with_metrics(metrics_collector);
    
    // Start the background flush task
    let _flush_task = exporter.start_flush_task().expect("Failed to start flush task");
    
    // Create some test spans
    let span1 = create_test_span("span1", None);
    let span2 = create_test_span("span2", Some(span1.id().to_string()));
    
    // Export the spans
    exporter.export_spans(vec![span1, span2]).await.expect("Failed to export spans");
    
    // Wait for the spans to be exported
    sleep(Duration::from_secs(2)).await;
    
    // Note: we can't actually check that the spans were exported since this is a test
    // In a real application, you would check the OpenTelemetry collector's logs or UI
    println!("OpenTelemetry test completed - check collector logs or UI to verify");
}

// This tests the Jaeger exporter functionality
#[tokio::test]
async fn test_jaeger_exporter() {
    // Start Docker services and check if Jaeger is available
    if !start_docker_services().await || !check_jaeger_services().await {
        println!("Skipping test_jaeger_exporter: Jaeger not available");
        return;
    }

    // Create a Jaeger exporter with a custom configuration
    let config = ExternalTracingConfig {
        endpoint_url: "http://localhost:14268/api/traces".to_string(),
        service_name: "test-service".to_string(),
        environment: "test".to_string(),
        ..Default::default()
    };
    
    let metrics_collector = Arc::new(MetricsCollector::new("test"));
    let exporter = JaegerExporter::new(config).with_metrics(metrics_collector);
    
    // Start the background flush task
    let _flush_task = exporter.start_flush_task().expect("Failed to start flush task");
    
    // Create some test spans
    let span1 = create_test_span("span1", None);
    let span2 = create_test_span("span2", Some(span1.id().to_string()));
    
    // Export the spans
    exporter.export_spans(vec![span1, span2]).await.expect("Failed to export spans");
    
    // Wait for the spans to be exported
    sleep(Duration::from_secs(2)).await;
    
    // Note: we can't actually check that the spans were exported since this is a test
    // In a real application, you would check the Jaeger UI or API
    println!("Jaeger test completed - check Jaeger UI to verify");
}

// This tests the Zipkin exporter functionality
#[tokio::test]
async fn test_zipkin_exporter() {
    // Start Docker services and check if Zipkin is available
    if !start_docker_services().await || !check_zipkin_services().await {
        println!("Skipping test_zipkin_exporter: Zipkin not available");
        return;
    }

    // Create a Zipkin exporter with a custom configuration
    let config = ExternalTracingConfig {
        endpoint_url: "http://localhost:9411/api/v2/spans".to_string(),
        service_name: "test-service".to_string(),
        environment: "test".to_string(),
        ..Default::default()
    };
    
    let metrics_collector = Arc::new(MetricsCollector::new("test"));
    let exporter = ZipkinExporter::new(config).with_metrics(metrics_collector);
    
    // Start the background flush task
    let _flush_task = exporter.start_flush_task().expect("Failed to start flush task");
    
    // Create some test spans
    let span1 = create_test_span("span1", None);
    let span2 = create_test_span("span2", Some(span1.id().to_string()));
    
    // Export the spans
    exporter.export_spans(vec![span1, span2]).await.expect("Failed to export spans");
    
    // Wait for the spans to be exported
    sleep(Duration::from_secs(2)).await;
    
    // Note: we can't actually check that the spans were exported since this is a test
    // In a real application, you would check the Zipkin UI or API
    println!("Zipkin test completed - check Zipkin UI to verify");
}

// Helper function to create a test span
fn create_test_span(name: &str, parent_id: Option<String>) -> Span {
    let trace_id = uuid::Uuid::new_v4().to_string();
    let mut span = Span::new(name, trace_id, parent_id);
    
    // Add some attributes
    span.add_attribute("test_attribute", "test_value");
    
    // Add an event
    span.add_event("test_event", std::collections::HashMap::from([
        ("event_key".to_string(), "event_value".to_string())
    ]));
    
    // End the span
    span.end();
    
    span
}

// Mock exporter for testing
#[derive(Clone)]
struct MockExporter {
    spans: Arc<std::sync::Mutex<Vec<Span>>>,
}

impl MockExporter {
    fn new() -> Self {
        Self {
            spans: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }
    
    fn get_spans(&self) -> Vec<Span> {
        let spans = self.spans.lock().unwrap();
        spans.clone()
    }
}

#[async_trait::async_trait]
impl SpanExporter for MockExporter {
    async fn export_spans(&self, spans: Vec<Span>) -> crate::observability::ObservabilityResult<()> {
        let mut spans_guard = self.spans.lock().unwrap();
        spans_guard.extend(spans);
        Ok(())
    }
    
    async fn shutdown(&self) -> crate::observability::ObservabilityResult<()> {
        Ok(())
    }
} 