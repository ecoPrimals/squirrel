use std::sync::Mutex;
use std::time::Duration;
use std::collections::HashMap;
use tokio::test;
use std::sync::Arc;
use uuid::Uuid;

use crate::observability::tracing::{
    Span, SpanStatus, SpanEvent,
    external::{
        SpanExporter, ExternalTracingConfig, 
        OpenTelemetryExporter, ExternalTracer,
        JaegerExporter, ZipkinExporter
    }
};
use crate::observability::{ObservabilityError, ObservabilityResult};

// Import test helpers for checking services
#[path = "../tests/test_helpers.rs"]
mod test_helpers;

/// A mock exporter for testing that just stores spans
pub struct MockExporter {
    spans: Mutex<Vec<Span>>,
}

impl MockExporter {
    /// Create a new mock exporter
    pub fn new() -> Self {
        Self {
            spans: Mutex::new(Vec::new()),
        }
    }
    
    /// Get the spans that were exported
    pub fn get_exported_spans(&self) -> Vec<Span> {
        let spans = self.spans.lock().unwrap();
        println!("MockExporter.get_exported_spans(): Returning {} spans", spans.len());
        spans.clone()
    }
}

#[async_trait::async_trait]
impl SpanExporter for MockExporter {
    async fn export_spans(&self, spans: Vec<Span>) -> Result<(), ObservabilityError> {
        println!("MockExporter.export_spans(): Received {} spans", spans.len());
        for (i, span) in spans.iter().enumerate() {
            println!("  Span {}: name={}, status={:?}", i, span.name(), span.status());
        }
        
        let mut stored_spans = self.spans.lock().unwrap();
        println!("MockExporter.export_spans(): Before adding, storage has {} spans", stored_spans.len());
        stored_spans.extend(spans);
        println!("MockExporter.export_spans(): After adding, storage has {} spans", stored_spans.len());
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<(), ObservabilityError> {
        println!("MockExporter.shutdown() called");
        Ok(())
    }
}

/// A simplified tracer for testing with MockExporter
pub struct MockTracer {
    /// Current span buffer
    spans: Arc<Mutex<Vec<Span>>>,
    /// Active span
    current_span: Arc<Mutex<Option<String>>>,
    /// Exporter for sending spans
    exporter: Arc<MockExporter>,
}

impl MockTracer {
    /// Create a new tracer with mock exporter
    pub fn new(_name: &str, exporter: Arc<MockExporter>) -> Self {
        Self {
            spans: Arc::new(Mutex::new(Vec::new())),
            current_span: Arc::new(Mutex::new(None)),
            exporter,
        }
    }
    
    /// Create a new span
    pub fn create_span(&self, name: &str) -> Span {
        let trace_id = format!("trace-{}", Uuid::new_v4());
        Span::new(name, &trace_id, None)
    }
    
    /// Create a child span of the current span
    pub fn create_child_span(&self, name: &str) -> Span {
        let current = self.current_span.lock().unwrap();
        let trace_id = format!("trace-{}", Uuid::new_v4());
        
        if let Some(parent_id) = current.as_ref() {
            Span::new(name, &trace_id, Some(parent_id.clone()))
        } else {
            Span::new(name, &trace_id, None)
        }
    }
    
    /// Start a span and make it the current span
    pub fn start_span(&self, span: Span) -> SpanGuard {
        // Set as current span
        let mut current = self.current_span.lock().unwrap();
        *current = Some(span.id().to_string());
        
        // Add to spans list
        let mut spans = self.spans.lock().unwrap();
        let span_id = span.id().to_string();
        spans.push(span);
        
        // Return a guard that will end the span when dropped
        SpanGuard {
            span_id,
            tracer: self.clone(),
        }
    }
    
    /// Add an attribute to the current span
    pub fn add_span_attribute(&self, key: &str, value: &str) {
        let current_id = self.current_span.lock().unwrap();
        if let Some(span_id) = current_id.as_ref() {
            let mut spans = self.spans.lock().unwrap();
            if let Some(span) = spans.iter_mut().find(|s| s.id() == span_id) {
                span.add_attribute(key, value);
            }
        }
    }
    
    /// Add an event to the current span
    pub fn add_span_event(&self, name: &str, attributes: HashMap<String, String>) {
        let current_id = self.current_span.lock().unwrap();
        if let Some(span_id) = current_id.as_ref() {
            let mut spans = self.spans.lock().unwrap();
            if let Some(span) = spans.iter_mut().find(|s| s.id() == span_id) {
                span.add_event(name, attributes);
            }
        }
    }
    
    /// End a span by ID
    pub fn end_span(&self, span_id: &str) {
        let mut spans = self.spans.lock().unwrap();
        if let Some(span) = spans.iter_mut().find(|s| s.id() == span_id) {
            span.end();
        }
        
        // Clear current span if it matches
        let mut current = self.current_span.lock().unwrap();
        if let Some(current_id) = current.as_ref() {
            if current_id == span_id {
                *current = None;
            }
        }
    }
    
    /// Force flush all completed spans to the exporter
    pub async fn force_flush(&self) -> Result<(), ObservabilityError> {
        let spans_to_export = {
            let spans = self.spans.lock().unwrap();
            spans.iter()
                .filter(|span| span.status() == SpanStatus::Success || span.status() == SpanStatus::Error)
                .cloned()
                .collect::<Vec<_>>()
        };
        
        if !spans_to_export.is_empty() {
            self.exporter.export_spans(spans_to_export).await?;
        }
        
        Ok(())
    }
}

impl Clone for MockTracer {
    fn clone(&self) -> Self {
        Self {
            spans: Arc::clone(&self.spans),
            current_span: Arc::clone(&self.current_span),
            exporter: Arc::clone(&self.exporter),
        }
    }
}

/// A guard that will end the span when dropped
pub struct SpanGuard {
    /// ID of the span being guarded
    span_id: String,
    /// Reference to the tracer
    tracer: MockTracer,
}

impl Drop for SpanGuard {
    fn drop(&mut self) {
        self.tracer.end_span(&self.span_id);
    }
}

/// Test that spans can be created and exported using a mock exporter
#[test]
async fn test_external_tracing_with_mock_exporter() {
    // Create a mock exporter
    let mock_exporter = MockExporter::new();
    
    // Create a test span directly
    let mut span = Span::new("test_operation", "trace1", None);
    
    // Add attributes to the span
    span.add_attribute("test_key", "test_value");
    
    // End the span
    span.end();
    
    // Export span directly
    let spans_to_export = vec![span];
    mock_exporter.export_spans(spans_to_export).await.expect("Failed to export spans");
    
    // Get exported spans from mock exporter
    let exported_spans = mock_exporter.get_exported_spans();
    println!("test_external_tracing_with_mock_exporter: retrieved {} spans from exporter", exported_spans.len());
    
    // Verify that one span was exported
    assert_eq!(exported_spans.len(), 1);
    
    // Verify the exported span has the expected properties
    let exported_span = &exported_spans[0];
    assert_eq!(exported_span.name(), "test_operation");
    assert_eq!(exported_span.status(), SpanStatus::Success);
    
    // Check attributes
    let attrs = exported_span.attributes();
    assert_eq!(attrs.len(), 1);
    assert!(attrs.contains_key("test_key"));
    assert_eq!(attrs.get("test_key").unwrap(), "test_value");
}

/// Test that multiple spans can be created and exported together
#[test]
async fn test_external_tracing_multiple_spans() {
    // Create a mock exporter
    let mock_exporter = MockExporter::new();
    
    // Create multiple spans directly
    let mut span1 = Span::new("operation_1", "trace1", None);
    let mut span2 = Span::new("operation_2", "trace1", None);
    let mut span3 = Span::new("operation_3", "trace1", None);
    
    // End spans
    span1.end();
    span2.end();
    span3.end();
    
    // Export spans directly
    let spans_to_export = vec![span1, span2, span3];
    mock_exporter.export_spans(spans_to_export).await.expect("Failed to export spans");
    
    // Get exported spans from mock exporter
    let exported_spans = mock_exporter.get_exported_spans();
    
    // Verify that all spans were exported
    assert_eq!(exported_spans.len(), 3);
    
    // Verify operation names
    let operation_names: Vec<&str> = exported_spans.iter()
        .map(|span| span.name())
        .collect();
    
    assert_eq!(operation_names.len(), 3);
    assert!(operation_names.contains(&"operation_1"));
    assert!(operation_names.contains(&"operation_2"));
    assert!(operation_names.contains(&"operation_3"));
}

/// Test span with events
#[test]
async fn test_external_tracing_span_with_events() {
    // Create a mock exporter
    let mock_exporter = MockExporter::new();
    
    // Create a test span directly
    let mut span = Span::new("span_with_events", "trace1", None);
    
    // Add events and attributes
    let mut event1_attrs = HashMap::new();
    event1_attrs.insert("event_key_1".to_string(), "event_value_1".to_string());
    span.add_event("event_1", event1_attrs);
    
    // Also add a traceable attribute to the span
    span.add_attribute("event1_added", "true");
    
    // Add event 2
    let mut event2_attrs = HashMap::new();
    event2_attrs.insert("event_key_2".to_string(), "event_value_2".to_string());
    span.add_event("event_2", event2_attrs);
    
    // Also add a traceable attribute to the span
    span.add_attribute("event2_added", "true");
    
    // End the span
    span.end();
    
    // Export span directly
    let spans_to_export = vec![span];
    mock_exporter.export_spans(spans_to_export).await.expect("Failed to export spans");
    
    // Get exported spans from mock exporter
    let exported_spans = mock_exporter.get_exported_spans();
    
    // Verify that one span was exported
    assert_eq!(exported_spans.len(), 1);
    
    // Verify the exported span has the events
    let exported_span = &exported_spans[0];
    
    // Check that the span has 2 events
    let events = exported_span.events();
    assert_eq!(events.len(), 2, "Expected 2 events in the span");
    
    // Check for the marker attributes that indicate events were added
    let attrs = exported_span.attributes();
    assert!(attrs.contains_key("event1_added"), "Event 1 marker attribute not found");
    assert!(attrs.contains_key("event2_added"), "Event 2 marker attribute not found");
    assert_eq!(attrs.get("event1_added"), Some(&"true".to_string()));
    assert_eq!(attrs.get("event2_added"), Some(&"true".to_string()));
}

/// Test tracing with parent-child span relationship
#[test]
async fn test_external_tracing_parent_child_spans() {
    // Create a mock exporter
    let mock_exporter = MockExporter::new();
    
    // Create parent span
    let trace_id = "trace-1";
    let mut parent_span = Span::new("parent_operation", trace_id, None);
    
    // Get the parent ID to use for the child span
    let parent_id = parent_span.id().to_string();
    
    // Create child span with parent ID
    let mut child_span = Span::new("child_operation", trace_id, Some(parent_id));
    
    // End spans
    parent_span.end();
    child_span.end();
    
    // Export spans directly
    let spans_to_export = vec![parent_span, child_span];
    mock_exporter.export_spans(spans_to_export).await.expect("Failed to export spans");
    
    // Get exported spans from mock exporter
    let exported_spans = mock_exporter.get_exported_spans();
    
    // Verify that two spans were exported
    assert_eq!(exported_spans.len(), 2);
    
    // Find parent and child spans
    let parent_span = exported_spans.iter()
        .find(|span| span.name() == "parent_operation")
        .expect("Parent span not found");
    
    let child_span = exported_spans.iter()
        .find(|span| span.name() == "child_operation")
        .expect("Child span not found");
    
    // Verify parent-child relationship
    let parent_id = parent_span.id();
    let child_parent_id = child_span.parent_id();
    
    assert!(child_parent_id.is_some());
    assert_eq!(child_parent_id.unwrap(), parent_id);
}

/// Test OpenTelemetry exporter configuration
#[test]
async fn test_opentelemetry_exporter_config() {
    // Create configuration
    let config = ExternalTracingConfig {
        endpoint_url: "http://localhost:4317".to_string(),
        auth_token: Some("test_token".to_string()),
        flush_interval_seconds: 5,
        max_buffer_size: 100,
        add_standard_attributes: true,
        service_name: "test_service".to_string(),
        environment: "test_env".to_string(),
    };
    
    // Create exporter
    let _exporter = OpenTelemetryExporter::new(config.clone());
    
    // No assertions needed - just confirming it compiles and runs
}

/// Test Jaeger exporter configuration
#[test]
async fn test_jaeger_exporter_config() {
    // Create configuration
    let config = ExternalTracingConfig {
        endpoint_url: "http://localhost:14268/api/traces".to_string(),
        auth_token: Some("test_token".to_string()),
        flush_interval_seconds: 5,
        max_buffer_size: 100,
        add_standard_attributes: true,
        service_name: "test_service".to_string(),
        environment: "test_env".to_string(),
    };
    
    // Create exporter
    let _exporter = JaegerExporter::new(config.clone());
    
    // No assertions needed - just confirming it compiles and runs
}

/// Test Zipkin exporter configuration
#[test]
async fn test_zipkin_exporter_config() {
    // Create configuration
    let config = ExternalTracingConfig {
        endpoint_url: "http://localhost:9411/api/v2/spans".to_string(),
        auth_token: Some("test_token".to_string()),
        flush_interval_seconds: 5,
        max_buffer_size: 100,
        add_standard_attributes: true,
        service_name: "test_service".to_string(),
        environment: "test_env".to_string(),
    };
    
    // Create exporter
    let _exporter = ZipkinExporter::new(config.clone());
    
    // No assertions needed - just confirming it compiles and runs
}

/// Test OpenTelemetry exporter
/// This test will be skipped if OpenTelemetry collector is not available
#[test]
async fn test_opentelemetry_exporter() {
    // Start Docker services and check if OpenTelemetry is available
    if !test_helpers::start_docker_services().await {
        println!("Skipping test_opentelemetry_exporter: Docker services not started");
        return;
    }
    
    if !test_helpers::check_otel_services().await {
        println!("Skipping test_opentelemetry_exporter: OpenTelemetry collector not available");
        return;
    }

    // Create configuration
    let config = ExternalTracingConfig {
        endpoint_url: "http://localhost:4318/v1/traces".to_string(),
        auth_token: None,
        flush_interval_seconds: 5,
        max_buffer_size: 100,
        add_standard_attributes: true,
        service_name: "test_service".to_string(),
        environment: "test".to_string(),
    };
    
    // Create exporter and start a background flush task
    let exporter = OpenTelemetryExporter::new(config);
    let _flush_task = exporter.start_flush_task();
    
    // Create and export a test span
    let mut span = Span::new("otel_test_span", "trace-123", None);
    span.add_attribute("test_key", "test_value");
    span.end(); // Mark as completed
    
    // Export the span
    exporter.export_spans(vec![span]).await.expect("Failed to export span");
    
    // Wait for the span to be processed
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    println!("OpenTelemetry test completed - check collector logs or UI to verify");
}

/// Test Jaeger exporter
/// This test will be skipped if Jaeger is not available
#[test]
async fn test_jaeger_exporter() {
    // Start Docker services and check if Jaeger is available
    if !test_helpers::start_docker_services().await {
        println!("Skipping test_jaeger_exporter: Docker services not started");
        return;
    }
    
    if !test_helpers::check_jaeger_services().await {
        println!("Skipping test_jaeger_exporter: Jaeger not available");
        return;
    }

    // Create configuration
    let config = ExternalTracingConfig {
        endpoint_url: "http://localhost:14268/api/traces".to_string(),
        auth_token: None,
        flush_interval_seconds: 5,
        max_buffer_size: 100,
        add_standard_attributes: true,
        service_name: "test_service".to_string(),
        environment: "test".to_string(),
    };
    
    // Create exporter and start a background flush task
    let exporter = JaegerExporter::new(config);
    let _flush_task = exporter.start_flush_task();
    
    // Create and export a test span
    let mut span = Span::new("jaeger_test_span", "trace-123", None);
    span.add_attribute("test_key", "test_value");
    span.end(); // Mark as completed
    
    // Export the span
    exporter.export_spans(vec![span]).await.expect("Failed to export span");
    
    // Wait for the span to be processed
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    println!("Jaeger test completed - check Jaeger UI to verify at http://localhost:16686");
}

/// Test Zipkin exporter
/// This test will be skipped if Zipkin is not available
#[test]
async fn test_zipkin_exporter() {
    // Start Docker services and check if Zipkin is available
    if !test_helpers::start_docker_services().await {
        println!("Skipping test_zipkin_exporter: Docker services not started");
        return;
    }
    
    if !test_helpers::check_zipkin_services().await {
        println!("Skipping test_zipkin_exporter: Zipkin not available");
        return;
    }

    // Create configuration
    let config = ExternalTracingConfig {
        endpoint_url: "http://localhost:9411/api/v2/spans".to_string(),
        auth_token: None,
        flush_interval_seconds: 5,
        max_buffer_size: 100,
        add_standard_attributes: true,
        service_name: "test_service".to_string(),
        environment: "test".to_string(),
    };
    
    // Create exporter and start a background flush task
    let exporter = ZipkinExporter::new(config);
    let _flush_task = exporter.start_flush_task();
    
    // Create and export a test span
    let mut span = Span::new("zipkin_test_span", "trace-123", None);
    span.add_attribute("test_key", "test_value");
    span.end(); // Mark as completed
    
    // Export the span
    exporter.export_spans(vec![span]).await.expect("Failed to export span");
    
    // Wait for the span to be processed
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    println!("Zipkin test completed - check Zipkin UI to verify at http://localhost:9411");
}

/// Mock version of OpenTelemetry exporter test using MockExporter
/// This test runs without requiring an actual OpenTelemetry collector
#[test]
async fn test_opentelemetry_exporter_mock() {
    // Create a mock exporter
    let mock_exporter = Arc::new(MockExporter::new());
    
    // Create a stub tracer with mock exporter
    let tracer = MockTracer::new("opentelemetry-mock", mock_exporter.clone());
    
    // Create span using the tracer
    let span = tracer.create_span("otel_test_operation");
    let _span_guard = tracer.start_span(span);
    
    // Add some attributes via the tracer
    tracer.add_span_attribute("test_key", "test_value");
    
    // Add an event
    let mut event_attrs = HashMap::new();
    event_attrs.insert("event_key".to_string(), "event_value".to_string());
    tracer.add_span_event("test_event", event_attrs);
    
    // End the span (implicitly done by span_guard drop)
    drop(_span_guard);
    
    // Manually flush spans to the exporter
    tracer.force_flush().await.expect("Failed to flush spans");
    
    // Get exported spans from mock exporter
    let exported_spans = mock_exporter.get_exported_spans();
    
    // Verify that at least one span was exported
    assert!(!exported_spans.is_empty(), "No spans were exported");
    
    // Find our test span
    let test_span = exported_spans.iter()
        .find(|span| span.name() == "otel_test_operation")
        .expect("Test span not found in exported spans");
    
    // Verify attributes
    let attrs = test_span.attributes();
    assert!(attrs.contains_key("test_key"), "Span is missing expected attribute");
    assert_eq!(attrs.get("test_key").unwrap(), "test_value");
    
    // Verify events
    let events = test_span.events();
    assert!(!events.is_empty(), "No events found in span");
    assert!(events.len() > 0, "Expected at least one event");
}

/// Mock version of Jaeger exporter test using MockExporter
/// This test runs without requiring an actual Jaeger collector
#[test]
async fn test_jaeger_exporter_mock() {
    // Create a mock exporter
    let mock_exporter = Arc::new(MockExporter::new());
    
    // Create a stub tracer with mock exporter
    let tracer = MockTracer::new("jaeger-mock", mock_exporter.clone());
    
    // Create a parent span
    let parent_span = tracer.create_span("jaeger_parent_operation");
    let parent_guard = tracer.start_span(parent_span);
    
    // Add attributes to the parent span
    tracer.add_span_attribute("service", "test_service");
    
    // Create a child span
    let child_span = tracer.create_child_span("jaeger_child_operation");
    let child_guard = tracer.start_span(child_span);
    
    // Add attributes to the child span
    tracer.add_span_attribute("operation_type", "subtask");
    
    // End the child span
    drop(child_guard);
    
    // End the parent span
    drop(parent_guard);
    
    // Manually flush spans to the exporter
    tracer.force_flush().await.expect("Failed to flush spans");
    
    // Get exported spans from mock exporter
    let exported_spans = mock_exporter.get_exported_spans();
    
    // Verify that at least two spans were exported
    assert!(exported_spans.len() >= 2, "Expected at least 2 spans");
    
    // Find the parent and child spans
    let parent_span = exported_spans.iter()
        .find(|span| span.name() == "jaeger_parent_operation")
        .expect("Parent span not found");
        
    let child_span = exported_spans.iter()
        .find(|span| span.name() == "jaeger_child_operation")
        .expect("Child span not found");
    
    // Verify attributes
    assert_eq!(parent_span.attributes().get("service").unwrap(), "test_service");
    assert_eq!(child_span.attributes().get("operation_type").unwrap(), "subtask");
    
    // Verify parent-child relationship
    let parent_id = parent_span.id();
    let child_parent_id = child_span.parent_id().expect("Child has no parent ID");
    assert_eq!(child_parent_id, parent_id, "Child span's parent ID doesn't match parent span ID");
}

/// Mock version of Zipkin exporter test using MockExporter
/// This test runs without requiring an actual Zipkin collector
#[test]
async fn test_zipkin_exporter_mock() {
    // Create a mock exporter
    let mock_exporter = Arc::new(MockExporter::new());
    
    // Create a stub tracer with mock exporter
    let tracer = MockTracer::new("zipkin-mock", mock_exporter.clone());
    
    // Create spans for a distributed trace
    let span1 = tracer.create_span("zipkin_service_a");
    let _guard1 = tracer.start_span(span1);
    
    // Add span attributes
    tracer.add_span_attribute("component", "service_a");
    tracer.add_span_attribute("http.method", "GET");
    tracer.add_span_attribute("http.status_code", "200");
    
    // Add an event to record significant timing
    let mut event_attrs = HashMap::new();
    event_attrs.insert("database.query_time_ms".to_string(), "15".to_string());
    tracer.add_span_event("database.query", event_attrs);
    
    // End the span
    drop(_guard1);
    
    // Create another span
    let span2 = tracer.create_span("zipkin_service_b");
    let _guard2 = tracer.start_span(span2);
    tracer.add_span_attribute("component", "service_b");
    drop(_guard2);
    
    // Manually flush spans to the exporter
    tracer.force_flush().await.expect("Failed to flush spans");
    
    // Get exported spans from mock exporter
    let exported_spans = mock_exporter.get_exported_spans();
    
    // Verify that spans were exported
    assert!(exported_spans.len() >= 2, "Expected at least 2 spans");
    
    // Find the spans
    let service_a_span = exported_spans.iter()
        .find(|span| span.name() == "zipkin_service_a")
        .expect("Service A span not found");
        
    let service_b_span = exported_spans.iter()
        .find(|span| span.name() == "zipkin_service_b")
        .expect("Service B span not found");
    
    // Verify attributes
    assert_eq!(service_a_span.attributes().get("component").unwrap(), "service_a");
    assert_eq!(service_a_span.attributes().get("http.method").unwrap(), "GET");
    assert_eq!(service_a_span.attributes().get("http.status_code").unwrap(), "200");
    
    assert_eq!(service_b_span.attributes().get("component").unwrap(), "service_b");
    
    // Verify that service_a has events
    let events = service_a_span.events();
    assert!(!events.is_empty(), "No events found in service_a span");
    assert!(events.len() > 0, "Expected at least one event in service_a");
}

/// Test that spans can be directly exported to a MockExporter
#[test]
async fn test_direct_span_export() {
    // Create a mock exporter
    let mock_exporter = MockExporter::new();
    
    // Create a span directly
    let mut span = Span::new("direct_test", "trace1", None);
    span.add_attribute("test_key", "test_value");
    
    // End the span
    span.end();
    
    // Export the span directly
    let spans_to_export = vec![span];
    mock_exporter.export_spans(spans_to_export).await.expect("Failed to export spans");
    
    // Get exported spans from mock exporter
    let exported_spans = mock_exporter.get_exported_spans();
    
    // Verify that one span was exported
    assert_eq!(exported_spans.len(), 1);
    
    // Verify the exported span has the expected properties
    let exported_span = &exported_spans[0];
    assert_eq!(exported_span.name(), "direct_test");
    assert_eq!(exported_span.status(), SpanStatus::Success);
    
    // Check attributes
    let attrs = exported_span.attributes();
    assert_eq!(attrs.len(), 1);
    assert!(attrs.contains_key("test_key"));
    assert_eq!(attrs.get("test_key").unwrap(), "test_value");
} 