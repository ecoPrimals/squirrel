use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::test;
use async_trait::async_trait;

use crate::observability::tracing::{
    Span, SpanStatus,
    external::{SpanExporter, ExternalTracingConfig},
};
use crate::observability::exporters::dashboard_exporter::DashboardExporter;
use crate::observability::ObservabilityError;

use squirrel_interfaces::tracing::{
    TraceData, TraceDataProvider, TraceDataConsumer
};

/// A test implementation of TraceDataConsumer that records received traces
struct TestTraceDataConsumer {
    pub traces: std::sync::Mutex<Vec<TraceData>>,
}

impl TestTraceDataConsumer {
    fn new() -> Self {
        Self {
            traces: std::sync::Mutex::new(Vec::new()),
        }
    }
    
    fn get_traces(&self) -> Vec<TraceData> {
        let traces = self.traces.lock().unwrap();
        traces.clone()
    }
}

#[async_trait]
impl TraceDataConsumer for TestTraceDataConsumer {
    async fn consume_trace_data(&self, trace_data: TraceData) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("TestTraceDataConsumer received trace data with {} spans", trace_data.spans.len());
        let mut traces = self.traces.lock().unwrap();
        traces.push(trace_data);
        Ok(())
    }
}

/// Create a test span for testing
fn create_test_span(name: &str, trace_id: &str, parent_id: Option<&str>) -> Span {
    let mut span = Span::new(
        name,
        trace_id,
        parent_id.map(String::from),
    );
    
    // Add some attributes
    span.add_attribute("component", "test");
    span.add_attribute("operation", "test_operation");
    
    // Add an event
    let mut event_attributes = HashMap::new();
    event_attributes.insert("event_type".to_string(), "test_event".to_string());
    span.add_event("test_event", event_attributes);
    
    // End the span
    span.end();
    
    span
}

/// Test that the DashboardExporter correctly implements TraceDataProvider
#[test]
async fn test_dashboard_exporter_trace_data_provider() {
    let config = ExternalTracingConfig {
        endpoint_url: "http://localhost:3000".to_string(),
        auth_token: None,
        flush_interval_seconds: 1,
        max_buffer_size: 10,
        add_standard_attributes: true,
        service_name: "test-service".to_string(),
        environment: "test".to_string(),
    };
    
    let exporter = DashboardExporter::new(config.clone());
    
    // Create and export test spans
    let trace_id = "trace-123";
    let parent_span = create_test_span("parent_operation", trace_id, None);
    let child_span = create_test_span("child_operation", trace_id, Some(parent_span.id()));
    
    // Export the spans
    exporter.export_spans(vec![parent_span.clone(), child_span.clone()]).await.unwrap();
    
    // Verify that we can retrieve trace data
    let traces = exporter.get_trace_data().await.unwrap();
    assert!(!traces.is_empty(), "Expected trace data to be non-empty");
    
    // Verify we can retrieve by trace ID
    let trace = exporter.get_trace_by_id(trace_id).await.unwrap();
    assert!(trace.is_some(), "Expected to find trace by ID");
    
    if let Some(trace) = trace {
        assert_eq!(trace.service, "test-service");
        assert_eq!(trace.environment, "test");
        assert_eq!(trace.spans.len(), 2);
        
        // Verify spans
        let span_names: Vec<String> = trace.spans.iter().map(|s| s.name.clone()).collect();
        assert!(span_names.contains(&"parent_operation".to_string()));
        assert!(span_names.contains(&"child_operation".to_string()));
    }
}

/// Test that the dashboard exporter can send spans to a consumer
#[test]
async fn test_dashboard_exporter_with_consumer() {
    let config = ExternalTracingConfig {
        endpoint_url: "http://localhost:3000".to_string(),
        auth_token: None,
        flush_interval_seconds: 1,
        max_buffer_size: 10,
        add_standard_attributes: true,
        service_name: "test-service".to_string(),
        environment: "test".to_string(),
    };
    
    let exporter = DashboardExporter::new(config.clone());
    let consumer = Arc::new(TestTraceDataConsumer::new());
    
    // Create and export test spans
    let trace_id = "trace-123";
    let span1 = create_test_span("operation_1", trace_id, None);
    let span2 = create_test_span("operation_2", trace_id, Some(span1.id()));
    let span3 = create_test_span("operation_3", trace_id, Some(span2.id()));
    
    // Export the spans
    exporter.export_spans(vec![span1, span2, span3]).await.unwrap();
    
    // Get the trace data
    let traces = exporter.get_trace_data().await.unwrap();
    assert_eq!(traces.len(), 1);
    
    // Now send the trace data to the consumer
    for trace in traces {
        consumer.consume_trace_data(trace).await.unwrap();
    }
    
    // Verify the consumer received the trace data
    let consumer_traces = consumer.get_traces();
    assert_eq!(consumer_traces.len(), 1);
    assert_eq!(consumer_traces[0].spans.len(), 3);
    
    // Verify span names
    let span_names: Vec<String> = consumer_traces[0].spans.iter()
        .map(|s| s.name.clone())
        .collect();
    
    assert!(span_names.contains(&"operation_1".to_string()));
    assert!(span_names.contains(&"operation_2".to_string()));
    assert!(span_names.contains(&"operation_3".to_string()));
    
    // Verify trace metadata
    assert_eq!(consumer_traces[0].service, "test-service");
    assert_eq!(consumer_traces[0].environment, "test");
} 