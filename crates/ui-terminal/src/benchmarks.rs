//! Benchmarks for UI terminal components

use std::collections::HashMap;
use std::time::{Instant, Duration};
use chrono::{DateTime, Utc};

use crate::util::{CompressedTimeSeries, CachedMetrics, CachedWidget, CachedMap};
use crate::widgets::protocol::ProtocolWidget;

/// Run a timed test function multiple times and return statistics
fn bench_function<F>(name: &str, iterations: usize, f: F) -> (Duration, Duration, Duration) 
where
    F: Fn() -> ()
{
    println!("Benchmarking: {}", name);
    
    let mut durations = Vec::with_capacity(iterations);
    
    // Warmup
    for _ in 0..5 {
        f();
    }
    
    // Actual benchmark
    for i in 0..iterations {
        let start = Instant::now();
        f();
        let elapsed = start.elapsed();
        durations.push(elapsed);
        
        if i % 10 == 0 {
            print!(".");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    }
    println!(" done!");
    
    // Calculate statistics
    durations.sort();
    
    let min = durations.first().unwrap_or(&Duration::from_secs(0)).clone();
    let max = durations.last().unwrap_or(&Duration::from_secs(0)).clone();
    
    let total: Duration = durations.iter().sum();
    let avg = total / durations.len() as u32;
    
    println!("  Min: {:?}", min);
    println!("  Avg: {:?}", avg);
    println!("  Max: {:?}", max);
    
    (min, avg, max)
}

/// Benchmark the CompressedTimeSeries
pub fn bench_compressed_time_series() {
    println!("\n=== Compressed Time Series Benchmark ===");
    
    // Create test data - 10,000 points
    let now = Utc::now();
    let mut points = Vec::with_capacity(10_000);
    
    for i in 0..10_000 {
        let timestamp = now + chrono::Duration::milliseconds(i * 100);
        let value = (i as f64 / 100.0).sin() + 1.0; // Value between 0 and 2
        points.push((timestamp, value));
    }
    
    // Benchmark regular Vec storage
    bench_function("Vec storage - insert 10k points", 10, || {
        let mut regular_vec = Vec::with_capacity(10_000);
        for point in &points {
            regular_vec.push(*point);
        }
    });
    
    // Benchmark CompressedTimeSeries
    bench_function("CompressedTimeSeries - insert 10k points", 10, || {
        let mut ts = CompressedTimeSeries::<f64>::new(10_000);
        for (timestamp, value) in &points {
            ts.add(*timestamp, *value);
        }
    });
    
    // Create filled time series for read tests
    let mut ts = CompressedTimeSeries::<f64>::new(10_000);
    for (timestamp, value) in &points {
        ts.add(*timestamp, *value);
    }
    
    // Benchmark read operations
    bench_function("CompressedTimeSeries - read all points", 100, || {
        let _all_points = ts.points();
    });
    
    bench_function("CompressedTimeSeries - downsample to 100 points", 100, || {
        let _downsampled = ts.downsample(100);
    });
    
    // Compare memory usage
    let regular_size = std::mem::size_of::<(DateTime<Utc>, f64)>() * 10_000;
    let compressed_size = std::mem::size_of_val(&ts);
    
    println!("Memory usage:");
    println!("  Vec<(DateTime<Utc>, f64)>: {} bytes", regular_size);
    println!("  CompressedTimeSeries: {} bytes", compressed_size);
    println!("  Compression ratio: {:.2}x", regular_size as f64 / compressed_size as f64);
}

/// Benchmark cached metrics
pub fn bench_cached_metrics() {
    println!("\n=== Cached Metrics Benchmark ===");
    
    let heavy_computation = || {
        // Simulate an expensive computation
        let mut result = 0;
        for i in 0..1_000_000 {
            result += i;
        }
        result
    };
    
    // Benchmark without caching
    bench_function("Without caching", 10, || {
        for _ in 0..100 {
            heavy_computation();
        }
    });
    
    // Benchmark with caching
    bench_function("With CachedMetrics", 10, || {
        let mut cache = CachedMetrics::<i32>::new(1000);
        
        for _ in 0..100 {
            if let Some(value) = cache.get() {
                // Use cached value
                let _ = value;
            } else {
                // Compute and cache
                let result = heavy_computation();
                cache.update(result);
            }
        }
    });
}

/// Benchmark protocol widget rendering
pub fn bench_protocol_widget() {
    println!("Benchmarking protocol widget...");
    
    // Create test data
    let connection_history = create_connection_history();
    let metrics_history = create_metrics_history();
    let recent_messages = create_recent_messages();
    let protocol_errors = create_protocol_errors();
    let mut performance_metrics = HashMap::new();
    performance_metrics.insert("Throughput".to_string(), 150.5);
    performance_metrics.insert("Response Time".to_string(), 42.8);
    performance_metrics.insert("Error Rate".to_string(), 0.2);
    performance_metrics.insert("Network Latency".to_string(), 8.3);
    
    // Create protocol data
    let protocol_data = create_test_protocol_data();
    
    // Create connection health 
    let connection_health = create_test_connection_health();
    
    // Create widget instance for the benchmark
    let mut widget = ProtocolWidget::new(&protocol_data, "Protocol")
        .with_connection_health(&connection_health)
        .with_connection_history(&connection_history)
        .with_metrics_history(&metrics_history)
        .with_recent_messages(&recent_messages)
        .with_protocol_errors(&protocol_errors)
        .with_performance_metrics(&performance_metrics);
    
    // Benchmark rendering just once
    bench_function("Render protocol widget once", 10, || {
        let mut widget = ProtocolWidget::new(&protocol_data, "Protocol")
            .with_connection_health(&connection_health)
            .with_connection_history(&connection_history)
            .with_metrics_history(&metrics_history)
            .with_recent_messages(&recent_messages)
            .with_protocol_errors(&protocol_errors)
            .with_performance_metrics(&performance_metrics);
            
        let _ = widget.get_statistics_items();
    });
    
    // Benchmark multiple renders with caching
    bench_function("Render protocol widget 10 times with caching", 10, || {
        let mut widget = ProtocolWidget::new(&protocol_data, "Protocol")
            .with_connection_health(&connection_health)
            .with_connection_history(&connection_history)
            .with_metrics_history(&metrics_history)
            .with_recent_messages(&recent_messages)
            .with_protocol_errors(&protocol_errors)
            .with_performance_metrics(&performance_metrics);
            
        for _ in 0..10 {
            let _ = widget.get_statistics_items();
        }
    });
    
    // Simulate update patterns and measure cache performance
    bench_function("Update protocol widget and render", 10, || {
        // Initial render
        let mut widget = ProtocolWidget::new(&protocol_data, "Protocol")
            .with_connection_health(&connection_health)
            .with_connection_history(&connection_history)
            .with_metrics_history(&metrics_history)
            .with_recent_messages(&recent_messages)
            .with_protocol_errors(&protocol_errors)
            .with_performance_metrics(&performance_metrics);
            
        let _ = widget.get_statistics_items();
        
        // Cycle through tabs
        widget.next_tab();
        let _ = widget.get_statistics_items();
        
        widget.next_tab();
        let _ = widget.get_statistics_items();
        
        widget.next_tab();
        let _ = widget.get_statistics_items();
        
        // Back to first tab
        widget.set_tab(0);
        let _ = widget.get_statistics_items();
    });
}

// Helper function to create test protocol data
fn create_test_protocol_data() -> dashboard_core::data::ProtocolData {
    dashboard_core::data::ProtocolData {
        name: "Test Protocol".to_string(),
        protocol_type: "TCP".to_string(),
        version: "1.0".to_string(),
        connected: true,
        last_connected: Some(chrono::Utc::now()),
        status: "connected".to_string(),
        error: None,
        retry_count: 0,
        metrics: {
            let mut metrics = HashMap::new();
            metrics.insert("latency".to_string(), 8.3);
            metrics.insert("throughput".to_string(), 150.5);
            metrics
        },
    }
}

// Helper function to create test connection health
fn create_test_connection_health() -> widgets::protocol::ConnectionHealth {
    widgets::protocol::ConnectionHealth {
        status: widgets::protocol::ConnectionStatus::Connected,
        uptime_seconds: 3600,
        last_status_change: chrono::Utc::now() - chrono::Duration::seconds(3600),
        connected_since: Some(chrono::Utc::now() - chrono::Duration::seconds(3600)),
        reconnect_count: 0,
        last_error: None,
        health_score: 0.87,
    }
}

// Helper function to create test connection history
fn create_connection_history() -> Vec<widgets::protocol::ConnectionEvent> {
    let now = chrono::Utc::now();
    let mut events = Vec::new();
    
    // Create events over the last 24 hours
    for i in 0..24 {
        let timestamp = now - chrono::Duration::hours(i);
        let event_type = if i % 4 == 0 {
            widgets::protocol::ConnectionEventType::Connected
        } else if i % 4 == 1 {
            widgets::protocol::ConnectionEventType::Disconnected
        } else if i % 4 == 2 {
            widgets::protocol::ConnectionEventType::Reconnecting
        } else {
            widgets::protocol::ConnectionEventType::ReconnectSuccess
        };
        
        events.push(widgets::protocol::ConnectionEvent {
            timestamp,
            event_type,
            message: format!("Connection event at {}", timestamp.format("%H:%M:%S")),
        });
    }
    
    events
}

// Helper function to create test metrics history
fn create_metrics_history() -> HashMap<String, Vec<(chrono::DateTime<chrono::Utc>, f64)>> {
    let now = chrono::Utc::now();
    let mut metrics_history = HashMap::new();
    
    // Create latency metrics
    let mut latency_data = Vec::new();
    for i in 0..100 {
        let timestamp = now - chrono::Duration::seconds(i as i64);
        let value = 10.0 + 5.0 * (i as f64 / 100.0 * std::f64::consts::PI * 2.0).sin();
        latency_data.push((timestamp, value));
    }
    metrics_history.insert("latency".to_string(), latency_data);
    
    // Create throughput metrics
    let mut throughput_data = Vec::new();
    for i in 0..100 {
        let timestamp = now - chrono::Duration::seconds(i as i64);
        let value = 100.0 + 50.0 * (i as f64 / 100.0 * std::f64::consts::PI).sin();
        throughput_data.push((timestamp, value));
    }
    metrics_history.insert("throughput".to_string(), throughput_data);
    
    metrics_history
}

// Helper function to create test recent messages
fn create_recent_messages() -> Vec<widgets::protocol::ProtocolMessage> {
    let now = chrono::Utc::now();
    let mut messages = Vec::new();
    
    for i in 0..20 {
        let timestamp = now - chrono::Duration::seconds(i as i64);
        let message_type = match i % 4 {
            0 => widgets::protocol::MessageType::Request,
            1 => widgets::protocol::MessageType::Response,
            2 => widgets::protocol::MessageType::Event,
            _ => widgets::protocol::MessageType::Heartbeat,
        };
        
        let direction = if i % 2 == 0 {
            widgets::protocol::MessageDirection::Incoming
        } else {
            widgets::protocol::MessageDirection::Outgoing
        };
        
        messages.push(widgets::protocol::ProtocolMessage {
            id: format!("msg-{}", i),
            timestamp,
            message_type,
            direction,
            size_bytes: 100 + (i * 10) as u64,
            payload: format!("Test message payload {}", i),
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("source".to_string(), format!("test-source-{}", i));
                metadata.insert("destination".to_string(), "dashboard".to_string());
                metadata
            },
        });
    }
    
    messages
}

// Helper function to create test protocol errors
fn create_protocol_errors() -> Vec<widgets::protocol::ProtocolError> {
    let now = chrono::Utc::now();
    let mut errors = Vec::new();
    
    for i in 0..5 {
        let timestamp = now - chrono::Duration::hours(i);
        
        errors.push(widgets::protocol::ProtocolError {
            timestamp,
            error_type: format!("ERROR_{}", i),
            message: format!("Protocol error {}: Connection refused", i),
            source: "test-client".to_string(),
            severity: if i % 2 == 0 { "critical" } else { "warning" }.to_string(),
            resolved: i > 2,
        });
    }
    
    errors
}

/// Run all benchmarks
pub fn run_all_benchmarks() {
    bench_compressed_time_series();
    bench_cached_metrics();
    bench_protocol_widget();
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    #[ignore] // Long-running benchmark, run manually
    fn test_benchmarks() {
        run_all_benchmarks();
    }
} 