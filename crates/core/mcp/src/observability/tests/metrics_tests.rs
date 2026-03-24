// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for metrics functionality in the observability framework

use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::observability::metrics::{MetricsRegistry, Counter, Gauge, Histogram, MetricType, Labels};
use crate::observability::ObservabilityError;
use crate::observability::ObservabilityResult;

/// Test basic counter operations
#[test]
fn test_counter_basic_operations() {
    let registry = MetricsRegistry::new();
    registry.initialize().expect("should succeed");
    
    // Create a counter
    let counter = registry.create_counter(
        "test_counter",
        "Test counter description",
        None,
        HashMap::new()
    ).expect("should succeed");
    
    // Initial value should be 0
    assert_eq!(counter.value().expect("should succeed"), 0);
    
    // Increment by 1
    counter.inc_one().expect("should succeed");
    assert_eq!(counter.value().expect("should succeed"), 1);
    
    // Increment by a specific amount
    counter.inc(5).expect("should succeed");
    assert_eq!(counter.value().expect("should succeed"), 6);
    
    // Multiple increments
    counter.inc_one().expect("should succeed");
    counter.inc_one().expect("should succeed");
    counter.inc(2).expect("should succeed");
    assert_eq!(counter.value().expect("should succeed"), 10);
}

/// Test counters with labels
#[test]
fn test_counter_with_labels() {
    let registry = MetricsRegistry::new();
    registry.initialize().expect("should succeed");
    
    // Create counters with different labels
    let mut labels1 = HashMap::new();
    labels1.insert("service".to_string(), "api".to_string());
    
    let mut labels2 = HashMap::new();
    labels2.insert("service".to_string(), "database".to_string());
    
    let counter1 = registry.create_counter(
        "request_count",
        "Count of requests",
        None,
        labels1
    ).expect("should succeed");
    
    let counter2 = registry.create_counter(
        "request_count",
        "Count of requests",
        None,
        labels2
    ).expect("should succeed");
    
    // Increment each counter differently
    counter1.inc(10).expect("should succeed");
    counter2.inc(5).expect("should succeed");
    
    // Values should be independent
    assert_eq!(counter1.value().expect("should succeed"), 10);
    assert_eq!(counter2.value().expect("should succeed"), 5);
    
    // Check metric properties
    assert_eq!(counter1.metric().name(), "request_count");
    assert_eq!(counter1.metric().description(), "Count of requests");
    assert_eq!(counter1.metric().unit().expect("should succeed"), "requests");
    assert_eq!(counter1.metric().labels().get("service").expect("should succeed"), "api");
}

/// Test counter retrieval
#[test]
fn test_counter_retrieval() {
    let registry = MetricsRegistry::new();
    
    // Create a counter first
    let counter = registry.create_counter(
        "retrieval_counter",
        "Test counter for retrieval",
        None,
        HashMap::new()
    ).expect("should succeed");
    
    // Retrieve the counter
    let retrieved_counter = registry.get_counter("retrieval_counter").expect("should succeed");
    assert!(retrieved_counter.is_some());
    
    // Try to get a non-existent counter
    let non_existent = registry.get_counter("non_existent").expect("should succeed");
    assert!(non_existent.is_none());
}

/// Test basic gauge operations
#[test]
fn test_gauge_basic_operations() {
    let registry = MetricsRegistry::new();
    registry.initialize().expect("should succeed");
    
    // Create a gauge
    let gauge = registry.create_gauge(
        "test_gauge",
        "Test gauge description",
        None,
        HashMap::new()
    ).expect("should succeed");
    
    // Initial value should be 0
    assert!((gauge.value().expect("should succeed") - 0.0).abs() < f64::EPSILON);
    
    // Set to a specific value
    gauge.set(3.14).expect("should succeed");
    assert!((gauge.value().expect("should succeed") - 3.14).abs() < f64::EPSILON);
    
    // Increment by amount
    gauge.inc(1.0).expect("should succeed");
    assert!((gauge.value().expect("should succeed") - 4.14).abs() < f64::EPSILON);
    
    // Decrement by amount
    gauge.dec(2.0).expect("should succeed");
    assert!((gauge.value().expect("should succeed") - 2.14).abs() < f64::EPSILON);
}

/// Test gauge with negative values
#[test]
fn test_gauge_negative_values() {
    let registry = MetricsRegistry::new();
    registry.initialize().expect("should succeed");
    
    // Create a gauge
    let gauge = registry.create_gauge(
        "negative_gauge",
        "Gauge that can go negative",
        None,
        HashMap::new()
    ).expect("should succeed");
    
    // Set to a positive value
    gauge.set(10.0).expect("should succeed");
    assert!((gauge.value().expect("should succeed") - 10.0).abs() < f64::EPSILON);
    
    // Decrement below zero
    gauge.dec(15.0).expect("should succeed");
    assert!((gauge.value().expect("should succeed") - (-5.0)).abs() < f64::EPSILON);
    
    // Increment back to positive
    gauge.inc(7.5).expect("should succeed");
    assert!((gauge.value().expect("should succeed") - 2.5).abs() < f64::EPSILON);
}

/// Test gauge retrieval
#[test]
fn test_gauge_retrieval() {
    let registry = MetricsRegistry::new();
    
    // Create a gauge first
    let gauge = registry.create_gauge(
        "retrieval_gauge",
        "Test gauge for retrieval",
        None,
        HashMap::new(),
    ).expect("should succeed");
    
    // Retrieve the gauge
    let retrieved_gauge = registry.get_gauge("retrieval_gauge").expect("should succeed");
    assert!(retrieved_gauge.is_some());
    
    // Try to get a non-existent gauge
    let non_existent = registry.get_gauge("non_existent").expect("should succeed");
    assert!(non_existent.is_none());
}

/// Test basic histogram operations
#[test]
fn test_histogram_basic_operations() {
    let registry = MetricsRegistry::new();
    registry.initialize().expect("should succeed");
    
    // Create a histogram with specific buckets
    let buckets = vec![1.0, 5.0, 10.0, 50.0, 100.0, 500.0, 1000.0];
    let histogram = registry.create_histogram(
        "test_histogram",
        "Test histogram description",
        None,
        HashMap::new(),
        buckets
    ).expect("should succeed");
    
    // Observe some values
    histogram.observe(3.0).expect("should succeed");    // Falls in the 1.0-5.0 bucket
    histogram.observe(7.5).expect("should succeed");    // Falls in the 5.0-10.0 bucket
    histogram.observe(100.0).expect("should succeed");  // Falls in the 100.0-500.0 bucket
    histogram.observe(1.5).expect("should succeed");    // Falls in the 1.0-5.0 bucket
    histogram.observe(0.5).expect("should succeed");    // Falls in the 0.0-1.0 bucket
    
    // Check count and sum
    assert_eq!(histogram.count().expect("should succeed"), 5);
    assert!((histogram.sum().expect("should succeed") - 112.5).abs() < f64::EPSILON);
    
    // Check buckets
    let buckets = histogram.buckets().expect("should succeed");
    
    // Verify bucket counts
    // We can't be sure of the exact order, so we need to find each bucket by bound
    let bucket_1 = buckets.iter().find(|b| (b.upper_bound - 1.0).abs() < f64::EPSILON).expect("should succeed");
    let bucket_5 = buckets.iter().find(|b| (b.upper_bound - 5.0).abs() < f64::EPSILON).expect("should succeed");
    let bucket_10 = buckets.iter().find(|b| (b.upper_bound - 10.0).abs() < f64::EPSILON).expect("should succeed");
    let bucket_100 = buckets.iter().find(|b| (b.upper_bound - 100.0).abs() < f64::EPSILON).expect("should succeed");
    let bucket_500 = buckets.iter().find(|b| (b.upper_bound - 500.0).abs() < f64::EPSILON).expect("should succeed");
    
    assert_eq!(bucket_1.count, 1);   // 0.5
    assert_eq!(bucket_5.count, 2);   // 1.5, 3.0
    assert_eq!(bucket_10.count, 1);  // 7.5
    assert_eq!(bucket_100.count, 1); // 100.0
    assert_eq!(bucket_500.count, 0); // No values in this bucket
}

/// Test histogram with extreme values
#[test]
fn test_histogram_extreme_values() {
    let registry = MetricsRegistry::new();
    registry.initialize().expect("should succeed");
    
    // Create a histogram with specific buckets
    let buckets = vec![10.0, 100.0, 1000.0];
    let histogram = registry.create_histogram(
        "extreme_histogram",
        "Histogram for testing extreme values",
        None,
        HashMap::new(),
        buckets
    ).expect("should succeed");
    
    // Observe an extreme value (beyond all buckets)
    histogram.observe(5000.0).expect("should succeed");
    
    // Check count and sum
    assert_eq!(histogram.count().expect("should succeed"), 1);
    assert!((histogram.sum().expect("should succeed") - 5000.0).abs() < f64::EPSILON);
    
    // All defined buckets should have count 0
    let buckets = histogram.buckets().expect("should succeed");
    for bucket in buckets {
        // The observation is higher than all bucket upper bounds
        if bucket.upper_bound <= 1000.0 {
            assert_eq!(bucket.count, 0);
        } else {
            // There should be an implicit +Inf bucket
            assert_eq!(bucket.count, 1);
        }
    }
}

/// Test histogram retrieval
#[test]
fn test_histogram_retrieval() {
    let registry = MetricsRegistry::new();
    
    // Create a histogram first
    let histogram = registry.create_histogram(
        "retrieval_histogram",
        "Test histogram for retrieval",
        None,
        HashMap::new(),
        vec![0.1, 1.0, 10.0, 100.0], // bucket boundaries
    ).expect("should succeed");
    
    // Retrieve the histogram
    let retrieved_histogram = registry.get_histogram("retrieval_histogram").expect("should succeed");
    assert!(retrieved_histogram.is_some());
    
    // Try to get a non-existent histogram
    let non_existent = registry.get_histogram("non_existent").expect("should succeed");
    assert!(non_existent.is_none());
}

/// Test different metric namespaces
#[test]
fn test_metric_namespaces() {
    let registry = MetricsRegistry::new();
    registry.initialize().expect("should succeed");
    
    // Set a namespace for the registry
    let mut config = registry.get_config().expect("should succeed");
    config.namespace = Some("app".to_string());
    registry.set_config(&config).expect("should succeed");
    
    // Create metrics with and without namespaces
    let counter1 = registry.create_counter(
        "counter1",
        "Counter without additional namespace",
        None,
        HashMap::new()
    ).expect("should succeed");
    
    let counter2 = registry.create_counter_with_namespace(
        "api",
        "counter2",
        "Counter with additional namespace",
        None,
        HashMap::new()
    ).expect("should succeed");
    
    // Check that names are correctly namespaced
    assert_eq!(counter1.metric().name(), "app_counter1");
    assert_eq!(counter2.metric().name(), "app_api_counter2");
    
    // Make sure we can retrieve the counters with their full names
    let retrieved1 = registry.get_counter("app_counter1").expect("should succeed");
    let retrieved2 = registry.get_counter("app_api_counter2").expect("should succeed");
    
    assert!(retrieved1.is_some());
    assert!(retrieved2.is_some());
}

/// Test error handling for invalid metrics
#[test]
fn test_metrics_error_handling() {
    let registry = MetricsRegistry::new();
    registry.initialize().expect("should succeed");
    
    // Try to create a counter with an empty name
    let result = registry.create_counter(
        "",
        "Counter with empty name",
        None,
        HashMap::new()
    );
    assert!(result.is_err());
    
    // Try to create metrics with invalid labels
    let mut invalid_labels = HashMap::new();
    invalid_labels.insert("".to_string(), "value".to_string());
    
    let result = registry.create_counter(
        "invalid_labels_counter",
        "Counter with invalid labels",
        None,
        invalid_labels
    );
    assert!(result.is_err());
}

/// Test metric creation with the same name but different labels
#[test]
fn test_metric_label_validation() {
    let registry = MetricsRegistry::new();
    registry.initialize().expect("should succeed");
    
    // Valid labels
    let mut valid_labels = HashMap::new();
    valid_labels.insert("service".to_string(), "api".to_string());
    valid_labels.insert("environment".to_string(), "production".to_string());
    
    // Creating a metric with valid labels should succeed
    let result = registry.create_counter(
        "valid_labels_counter",
        "Counter with valid labels",
        None,
        valid_labels
    );
    assert!(result.is_ok());
    
    // Invalid label key (contains invalid character)
    let mut invalid_key_labels = HashMap::new();
    invalid_key_labels.insert("invalid.key".to_string(), "value".to_string());
    
    let result = registry.create_counter(
        "invalid_key_counter",
        "Counter with invalid label key",
        None,
        invalid_key_labels
    );
    assert!(result.is_err());
    
    // Empty label value should be accepted
    let mut empty_value_labels = HashMap::new();
    empty_value_labels.insert("key".to_string(), "".to_string());
    
    let result = registry.create_counter(
        "empty_value_counter",
        "Counter with empty label value",
        None,
        empty_value_labels
    );
    assert!(result.is_ok());
}

/// Test concurrent access to metrics
#[test]
fn test_metric_concurrency() {
    let registry = MetricsRegistry::new();
    registry.initialize().expect("should succeed");
    
    // Create a counter
    let counter = registry.create_counter(
        "concurrent_counter",
        "Counter for concurrency testing",
        None,
        HashMap::new()
    ).expect("should succeed");
    
    // Create a gauge
    let gauge = registry.create_gauge(
        "concurrent_gauge",
        "Gauge for concurrency testing",
        None,
        HashMap::new()
    ).expect("should succeed");
    
    // Create a histogram
    let histogram = registry.create_histogram(
        "concurrent_histogram",
        "Histogram for concurrency testing",
        None,
        HashMap::new(),
        vec![1.0, 10.0, 100.0]
    ).expect("should succeed");
    
    // Test concurrency with 10 threads
    let num_threads = 10;
    let num_iterations = 100;
    let mut handles = vec![];
    
    for i in 0..num_threads {
        let counter_clone = counter.clone();
        let gauge_clone = gauge.clone();
        let histogram_clone = histogram.clone();
        
        let handle = thread::spawn(move || {
            for j in 0..num_iterations {
                counter_clone.inc_one().expect("should succeed");
                gauge_clone.inc(0.1).expect("should succeed");
                histogram_clone.observe((i * j) as f64).expect("should succeed");
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("should succeed");
    }
    
    // Verify final values
    assert_eq!(counter.value().expect("should succeed"), (num_threads * num_iterations) as u64);
    assert!((gauge.value().expect("should succeed") - (num_threads * num_iterations) as f64 * 0.1).abs() < f64::EPSILON);
    assert_eq!(histogram.count().expect("should succeed"), (num_threads * num_iterations) as u64);
}

/// Test registry enumeration of metrics
#[test]
fn test_registry_enumeration() {
    let registry = MetricsRegistry::new();
    registry.initialize().expect("should succeed");
    
    // Create metrics of different types
    registry.create_counter("test_counter", "A test counter", None, HashMap::new()).expect("should succeed");
    registry.create_gauge("test_gauge", "A test gauge", None, HashMap::new()).expect("should succeed");
    registry.create_histogram("test_histogram", "A test histogram", None, HashMap::new(), vec![1.0, 10.0, 100.0]).expect("should succeed");
    
    // Get all metrics in the registry
    let all_metrics = registry.list_metrics().expect("should succeed");
    
    // Count metrics by type (simplified version - just count the metrics we know we created)
    let counter_count = registry.counter_names().expect("should succeed").len();
    let gauge_count = registry.gauge_names().expect("should succeed").len();
    let histogram_count = registry.histogram_names().expect("should succeed").len();
    
    // We should have created at least one of each type
    assert!(counter_count >= 1, "Should have at least 1 counter");
    assert!(gauge_count >= 1, "Should have at least 1 gauge");
    assert!(histogram_count >= 1, "Should have at least 1 histogram");
}

/// Test registry lifecycle
#[test]
fn test_registry_lifecycle() {
    // Create and initialize registry
    let registry = MetricsRegistry::new();
    registry.initialize().expect("should succeed");
    
    // Create a counter
    let counter = registry.create_counter(
        "lifecycle_counter",
        "Counter for lifecycle testing",
        None,
        HashMap::new()
    ).expect("should succeed");
    
    counter.inc(5).expect("should succeed");
    assert_eq!(counter.value().expect("should succeed"), 5);
    
    // Shutdown the registry
    registry.shutdown().expect("should succeed");
    
    // Creating new metrics after shutdown should fail
    let result = registry.create_counter(
        "post_shutdown_counter",
        "This counter should not be created",
        None,
        HashMap::new()
    );
    
    assert!(result.is_err());
} 