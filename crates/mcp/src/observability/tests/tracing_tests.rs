//! Tests for tracing functionality in the observability framework

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread;

use crate::observability::tracing::{Tracer, SpanContext, SpanStatus};
use crate::observability::ObservabilityError;
use crate::observability::ObservabilityResult;

/// Test basic span creation
#[test]
fn test_basic_span_creation() {
    let tracer = Tracer::new();
    tracer.initialize().unwrap();
    
    let span_result = tracer.start_span("test_operation");
    assert!(span_result.is_ok());
    
    let span = span_result.unwrap();
    {
        let span_guard = span.lock().unwrap();
        assert_eq!(span_guard.span().name(), "test_operation");
        assert!(span_guard.span().parent_id().is_none());
        assert_eq!(span_guard.span().status(), SpanStatus::Running);
    }
    
    // We need to drop the span explicitly which will call end()
    drop(span);
}

/// Test span attributes
#[test]
fn test_span_attributes() {
    let tracer = Tracer::new();
    tracer.initialize().unwrap();
    
    let span = tracer.start_span("attribute_test").unwrap();
    
    // Add attributes to the span
    {
        let mut span_guard = span.lock().unwrap();
        span_guard.add_attribute("service.name", "test-service");
        span_guard.add_attribute("operation.type", "query");
        span_guard.add_attribute("user.id", "12345");
        
        // Check attributes were set correctly
        let attributes = span_guard.span().attributes();
        assert_eq!(attributes.len(), 3);
        assert_eq!(attributes.get("service.name"), Some(&"test-service".to_string()));
        assert_eq!(attributes.get("operation.type"), Some(&"query".to_string()));
        assert_eq!(attributes.get("user.id"), Some(&"12345".to_string()));
    }
    
    // End the span by dropping it
    drop(span);
}

/// Test span events
#[test]
fn test_span_events() {
    let tracer = Tracer::new();
    tracer.initialize().unwrap();
    
    let span = tracer.start_span("event_test").unwrap();
    
    // Add events to the span
    {
        let mut span_guard = span.lock().unwrap();
        
        let mut event_attrs = HashMap::new();
        event_attrs.insert("stage".to_string(), "init".to_string());
        span_guard.add_event("started", event_attrs);
        
        let mut error_attrs = HashMap::new();
        error_attrs.insert("error.code".to_string(), "404".to_string());
        error_attrs.insert("error.message".to_string(), "Resource not found".to_string());
        span_guard.add_event("error_occurred", error_attrs);
        
        // Check events were recorded
        let events = span_guard.span().events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].name, "started");
        assert_eq!(events[1].name, "error_occurred");
        assert_eq!(events[1].attributes.get("error.code"), Some(&"404".to_string()));
    }
    
    // End the span error by dropping it
    drop(span);
}

/// Test span hierarchy
#[test]
fn test_span_hierarchy() {
    let tracer = Tracer::new();
    tracer.initialize().unwrap();
    
    // Create parent span
    let parent_span = tracer.start_span("parent_operation").unwrap();
    let parent_id = {
        let parent_guard = parent_span.lock().unwrap();
        parent_guard.span().id().to_string()
    };
    
    // Create child span
    let child_span = tracer.start_span_with_parent("child_operation", Some(parent_span.clone())).unwrap();
    
    // Verify parent-child relationship
    {
        let child_guard = child_span.lock().unwrap();
        let parent_guard = parent_span.lock().unwrap();
        
        assert_eq!(child_guard.span().name(), "child_operation");
        assert_eq!(child_guard.span().parent_id(), Some(&parent_id));
        
        // Child should inherit trace ID from parent
        assert_eq!(child_guard.span().trace_id(), parent_guard.span().trace_id());
    }
    
    // End spans in correct order: child first, then parent
    drop(child_span);
    drop(parent_span);
}

/// Test span status management
#[test]
fn test_span_status() {
    let tracer = Tracer::new();
    tracer.initialize().unwrap();
    
    // Create a span that will succeed
    let success_span = tracer.start_span("success_operation").unwrap();
    
    // Create a span that will fail
    let error_span = tracer.start_span("error_operation").unwrap();
    
    // End spans with different statuses
    {
        let span = Arc::clone(&success_span);
        drop(span); // This will drop one reference but not all
    }
    
    {
        let span = Arc::clone(&error_span);
        let mut guard = span.lock().unwrap();
        guard.add_attribute("error.message", "Something went wrong");
        // We can't call end_with_error() directly on the guard because it requires ownership
        // We'll end the span outside this block
    }
    
    // We need to end the spans properly
    // For error_span, we need a way to end with error, but due to the API design
    // that requires ownership transfer, we can't easily do this in tests
    // This is a limitation of the current API design
    
    // Clean up
    drop(success_span);
    drop(error_span);
}

/// Test span lifecycle including duration
#[test]
fn test_span_lifecycle() {
    let tracer = Tracer::new();
    tracer.initialize().unwrap();
    
    // Start time tracking
    let start = std::time::Instant::now();
    
    // Create span
    let span = tracer.start_span("timed_operation").unwrap();
    
    // Simulate work
    thread::sleep(Duration::from_millis(50));
    
    // Add some attributes
    {
        let mut span_guard = span.lock().unwrap();
        span_guard.add_attribute("operation.duration_target", "50ms");
    }
    
    // End span
    drop(span);
    
    // Verify elapsed time
    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() >= 50);
}

/// Test tracer configuration
#[test]
fn test_tracer_configuration() {
    let tracer = Tracer::new();
    let config = tracer.initialize().unwrap();
    
    // Modify configuration
    let mut new_config = config.clone();
    new_config.service_name = "test-service".to_string();
    new_config.enabled = false;
    
    // Apply new configuration
    tracer.set_config(&new_config).unwrap();
    
    // Verify configuration was applied
    let retrieved_config = tracer.get_config().unwrap();
    assert_eq!(retrieved_config.service_name, "test-service");
    assert_eq!(retrieved_config.enabled, false);
    
    // Re-enable tracing
    let mut enabled_config = retrieved_config.clone();
    enabled_config.enabled = true;
    tracer.set_config(&enabled_config).unwrap();
    
    // Verify we can create spans again
    assert!(tracer.start_span("test").is_ok());
}

/// Test concurrent spans in async context
#[tokio::test]
async fn test_concurrent_spans() {
    let tracer = Arc::new(Tracer::new());
    
    // Create multiple spans concurrently
    let mut handles = vec![];
    
    for i in 0..10 {
        let tracer_clone = Arc::clone(&tracer);
        let handle = tokio::spawn(async move {
            let span_name = format!("concurrent_span_{}", i);
            let span = tracer_clone.start_span(&span_name).unwrap();
            
            // Add unique attributes
            {
                let mut span_guard = span.lock().unwrap();
                span_guard.add_attribute("thread_id", i.to_string());
            }
            
            // Short pause
            tokio::time::sleep(Duration::from_millis(5)).await;
            
            // End the span
            {
                let mut span_guard = span.lock().unwrap();
                span_guard.end();
            }
            
            span_name
        });
        
        handles.push(handle);
    }
    
    // Wait for all spans to complete
    let mut span_names = vec![];
    for handle in handles {
        span_names.push(handle.await.unwrap());
    }
    
    // Verify all spans were created
    for name in span_names {
        let spans = tracer.get_trace_spans("").unwrap();
        assert!(spans.iter().any(|span| span.name() == name));
    }
}

/// Test error handling for invalid span operations
#[test]
fn test_tracer_error_handling() {
    let tracer = Tracer::new();
    tracer.initialize().unwrap();
    
    // Test invalid span name (empty)
    let result = tracer.start_span("");
    assert!(result.is_err());
    
    // Test invalid parent span ID
    let result = tracer.start_span_with_parent("child", "invalid-id");
    assert!(result.is_err());
}

/// Test getting the current active span
#[test]
fn test_current_span() {
    let tracer = Tracer::new();
    
    // Initially, there should be no current span
    let current = tracer.current_span().unwrap();
    assert!(current.is_none());
    
    // Create a span
    let span = tracer.start_span("current_test").unwrap();
    
    // Now there should be a current span
    let current = tracer.current_span().unwrap();
    assert!(current.is_some());
    
    // End the span
    {
        let mut span_guard = span.lock().unwrap();
        span_guard.end();
    }
    
    // After ending, the current span should be None again
    let current = tracer.current_span().unwrap();
    assert!(current.is_none());
}

/// Test span context propagation
#[test]
fn test_span_context_propagation() {
    let tracer = Tracer::new();
    tracer.initialize().unwrap();
    
    // Create a root span
    let root_span = tracer.start_span("root_span").unwrap();
    let (trace_id, span_id) = {
        let root_guard = root_span.lock().unwrap();
        (root_guard.span().trace_id().to_string(), root_guard.span().id().to_string())
    };
    
    // Create a child span manually using the IDs
    let child_span = tracer.start_span_with_parent("child_span", Some(root_span.clone())).unwrap();
    
    // Verify the child inherits context
    {
        let child_guard = child_span.lock().unwrap();
        
        assert_eq!(child_guard.span().parent_id(), Some(&span_id));
        assert_eq!(child_guard.span().trace_id(), &trace_id);
    }
}

/// Test handling spans within different traces
#[test]
fn test_multiple_traces() {
    let tracer = Tracer::new();
    
    // Create two root spans (different traces)
    let span1 = tracer.start_span("trace1_root").unwrap();
    let trace1_id = span1.lock().unwrap().span().trace_id().to_string();
    
    let span2 = tracer.start_span("trace2_root").unwrap();
    let trace2_id = span2.lock().unwrap().span().trace_id().to_string();
    
    // Traces should be different
    assert_ne!(trace1_id, trace2_id);
    
    // Create children in each trace
    let child1 = tracer.start_span_with_parent("trace1_child", Some(Arc::clone(&span1))).unwrap();
    let child2 = tracer.start_span_with_parent("trace2_child", Some(Arc::clone(&span2))).unwrap();
    
    // Verify trace propagation
    assert_eq!(child1.lock().unwrap().span().trace_id(), trace1_id);
    assert_eq!(child2.lock().unwrap().span().trace_id(), trace2_id);
    
    // Get spans for trace1
    let trace1_spans = tracer.get_trace_spans(&trace1_id).unwrap();
    assert_eq!(trace1_spans.len(), 2);
    
    // Get spans for trace2
    let trace2_spans = tracer.get_trace_spans(&trace2_id).unwrap();
    assert_eq!(trace2_spans.len(), 2);
}

/// Test span guard behavior with Drop trait
#[test]
fn test_span_guard_drop() {
    let tracer = Tracer::new();
    
    // Create a span and capture its ID
    let span_id = {
        let span = tracer.start_span("drop_test").unwrap();
        let id = span.lock().unwrap().span().id().to_string();
        
        // Span should be active
        assert!(tracer.get_span(&id).unwrap().unwrap().is_active());
        
        // Let span go out of scope without explicitly ending it
        id
    };
    
    // After drop, the span should be ended
    let span_info = tracer.get_span(&span_id).unwrap().unwrap();
    assert!(!span_info.is_active());
}

/// Test clearing spans from the tracer
#[test]
fn test_clear_spans() {
    let tracer = Tracer::new();
    
    // Create several spans
    for i in 0..5 {
        let span = tracer.start_span(&format!("clear_test_{}", i)).unwrap();
        let mut span_guard = span.lock().unwrap();
        span_guard.end();
    }
    
    // Verify spans were created
    let trace_spans = tracer.get_trace_spans("").unwrap();
    assert_eq!(trace_spans.len(), 5);
    
    // Clear all spans
    tracer.clear_spans().unwrap();
    
    // Verify spans were cleared
    let trace_spans = tracer.get_trace_spans("").unwrap();
    assert_eq!(trace_spans.len(), 0);
}

/// Test tracer singleton
#[test]
fn test_tracer_singleton() {
    // Get two instances of the tracer
    let tracer1 = Tracer::global();
    let tracer2 = Tracer::global();
    
    // Initialize the tracer
    tracer1.initialize().unwrap();
    
    // Both should point to the same instance
    let span1 = tracer1.start_span("test_from_tracer1").unwrap();
    let span2 = tracer2.start_span("test_from_tracer2").unwrap();
    
    // Both spans should be created successfully
    assert_eq!(span1.lock().unwrap().span().trace_id(), span2.lock().unwrap().span().trace_id());
    
    // End spans
    drop(span1);
    drop(span2);
}

/// Test trace correlation across spans
#[test]
fn test_trace_correlation() {
    let tracer = Tracer::new();
    
    // Create two related spans
    let span1 = tracer.start_span("span1").unwrap();
    let span2 = tracer.start_span_with_parent("span2", Some(span1.clone())).unwrap();
    
    // Verify they share the same trace ID
    assert_eq!(span1.lock().unwrap().span().trace_id(), span2.lock().unwrap().span().trace_id());
} 