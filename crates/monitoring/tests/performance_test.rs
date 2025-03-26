//! Performance tests for the monitoring functionality.
//!
//! These tests verify that the monitoring system can handle high volumes of data
//! and concurrent operations, and measure its performance characteristics.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio::time;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use serde_json::json;
use tokio::sync::Mutex;
use tokio::time::sleep;
use serde_json::Value;

use squirrel_core::error::Result;
use squirrel_core::error::SquirrelError;
use squirrel_monitoring::websocket::{WebSocketServer, WebSocketConfig, WebSocketInterface};
use squirrel_monitoring::metrics::{Metric, MetricType, MetricValue, DefaultMetricCollector};

mod mock_data_generator;
use mock_data_generator::MonitoringTestHarness;
use mock_data_generator::{MockMetricConfig, MockMetricGenerator, DataPattern};

/// Test metric collection performance under high load
#[tokio::test]
async fn test_metric_collection_performance() -> Result<()> {
    // Initialize metric collector
    let collector = DefaultMetricCollector::new();
    collector.initialize().await?;
    
    // Measure performance of collecting a large number of metrics
    const NUM_METRICS: usize = 10_000;
    let start = Instant::now();
    
    for i in 0..NUM_METRICS {
        let metric = Metric::new(
            format!("test_metric_{}", i),
            i as f64,
            MetricType::Gauge,
            std::collections::HashMap::new(),
        );
        collector.record_metric(metric).await?;
    }
    
    let duration = start.elapsed();
    println!("Recorded {} metrics in {:?} ({} metrics/sec)", 
        NUM_METRICS, 
        duration, 
        NUM_METRICS as f64 / duration.as_secs_f64()
    );
    
    // Performance should be above a minimum threshold (adjust based on expected performance)
    assert!(NUM_METRICS as f64 / duration.as_secs_f64() > 1000.0, 
        "Metric collection performance below threshold");
    
    Ok(())
}

/// Test WebSocket performance with high-frequency updates
#[ignore]
#[tokio::test]
async fn test_websocket_performance() -> Result<()> {
    // Create and start WebSocket server
    let config = WebSocketConfig {
        host: "127.0.0.1".to_string(),
        port: 8767,
        update_interval: 1,
        max_connections: 100,
        enable_compression: false,
        auth_required: false,
    };
    
    let server = Arc::new(WebSocketServer::new(config.clone()));
    server.start().await?;
    
    // Allow time for server to start
    sleep(Duration::from_millis(500)).await;
    
    // Create WebSocket clients
    const NUM_CLIENTS: usize = 10;
    let message_count = Arc::new(Mutex::new(0));
    
    // Connect clients
    let mut client_handles = Vec::new();
    for i in 0..NUM_CLIENTS {
        let url = format!("ws://{}:{}/ws", config.host, config.port);
        let message_counter = Arc::clone(&message_count);
        
        let handle = tokio::spawn(async move {
            // Connect to WebSocket server
            let (mut ws_stream, _) = connect_async(&url).await
                .expect("Failed to connect to WebSocket server");
            println!("Client {} connected", i);
            
            // Subscribe to a topic
            let topic = format!("performance_topic_{}", i % 3);
            let subscribe_msg = serde_json::json!({
                "action": "subscribe",
                "topic": topic,
            }).to_string();
            
            ws_stream.send(Message::Text(subscribe_msg)).await
                .expect("Failed to send subscription message");
            
            // Listen for messages
            let (write, mut read) = ws_stream.split();
            
            // Count messages received
            while let Some(msg) = read.next().await {
                if let Ok(Message::Text(_)) = msg {
                    let mut counter = message_counter.lock().await;
                    *counter += 1;
                }
            }
        });
        
        client_handles.push(handle);
    }
    
    // Update component data rapidly
    const NUM_UPDATES: usize = 100;
    let start = Instant::now();
    
    // Update component data for different topics
    for i in 0..NUM_UPDATES {
        for topic_id in 0..3 {
            let topic = format!("performance_topic_{}", topic_id);
            server.update_component_data(&topic, serde_json::json!({
                "value": i,
                "timestamp": chrono::Utc::now().timestamp(),
            })).await?;
        }
        
        if i % 10 == 0 {
            sleep(Duration::from_millis(10)).await;
        }
    }
    
    let duration = start.elapsed();
    
    // Allow time for messages to be processed
    sleep(Duration::from_secs(1)).await;
    
    let messages_received = *message_count.lock().await;
    println!("Sent {} updates in {:?}, clients received {} messages", 
        NUM_UPDATES * 3, 
        duration,
        messages_received
    );
    
    // Expect a reasonable number of messages to be received
    assert!(messages_received > 0, "No messages were received by clients");
    
    // Clean up
    server.stop().await?;
    
    Ok(())
}

/// Test performance under concurrent operations
#[tokio::test]
async fn test_concurrent_performance() -> Result<()> {
    // Configuration
    const NUM_CONCURRENT_OPERATIONS: usize = 50;
    const OPERATIONS_PER_TASK: usize = 100;
    
    // Create a WebSocket server
    let config = WebSocketConfig {
        host: "127.0.0.1".to_string(),
        port: 8768, // Different port from other tests
        update_interval: 1,
        max_connections: 200, // Higher limit for concurrent test
        enable_compression: false,
        auth_required: false,
    };
    
    // Create and start the server
    let server = Arc::new(WebSocketServer::new(config.clone()));
    server.start().await?;
    
    // Allow time for server to start
    sleep(Duration::from_millis(500)).await;
    
    // Create a counter for completed operations
    let operations_completed = Arc::new(AtomicUsize::new(0));
    let connection_successes = Arc::new(AtomicUsize::new(0));
    let message_counter = Arc::new(AtomicUsize::new(0));
    
    // Spawn concurrent client tasks
    let start_time = Instant::now();
    let mut handles = Vec::new();
    
    for task_id in 0..NUM_CONCURRENT_OPERATIONS {
        let server_clone = Arc::clone(&server);
        let operations_completed_clone = Arc::clone(&operations_completed);
        let connection_successes_clone = Arc::clone(&connection_successes);
        let message_counter_clone = Arc::clone(&message_counter);
        
        let handle = tokio::spawn(async move {
            // Create a new client connection
            let url = format!("ws://{}:{}/ws", config.host, config.port);
            
            // Try to connect to the server
            match connect_async(&url).await {
                Ok((mut ws_stream, _)) => {
                    // Successful connection
                    connection_successes_clone.fetch_add(1, Ordering::SeqCst);
                    
                    // Subscribe to a topic specific to this task
                    let topic = format!("concurrent_topic_{}", task_id % 10);
                    let subscribe_msg = serde_json::json!({
                        "action": "subscribe",
                        "topic": topic.clone(),
                    }).to_string();
                    
                    if let Err(e) = ws_stream.send(Message::Text(subscribe_msg)).await {
                        println!("Task {} failed to send subscription: {:?}", task_id, e);
                        return;
                    }
                    
                    // Split the WebSocket stream
                    let (mut write, mut read) = ws_stream.split();
                    
                    // Spawn a task to handle incoming messages
                    let msg_counter = Arc::clone(&message_counter_clone);
                    let read_task = tokio::spawn(async move {
                        while let Some(msg) = read.next().await {
                            if let Ok(Message::Text(_)) = msg {
                                msg_counter.fetch_add(1, Ordering::SeqCst);
                            }
                        }
                    });
                    
                    // Perform operations
                    for op_id in 0..OPERATIONS_PER_TASK {
                        // Simulate a mix of operations
                        match op_id % 3 {
                            0 => {
                                // Update component data
                                if let Err(e) = server_clone.update_component_data(
                                    &topic, 
                                    json!({
                                        "value": op_id,
                                        "timestamp": chrono::Utc::now().timestamp(),
                                    })
                                ).await {
                                    println!("Error updating component data: {:?}", e);
                                }
                            },
                            1 => {
                                // Send a data request
                                let request = json!({
                                    "action": "get_data",
                                    "topic": topic,
                                    "request_id": format!("req_{}_{}", task_id, op_id),
                                }).to_string();
                                
                                if let Err(e) = write.send(Message::Text(request)).await {
                                    println!("Task {} failed to send data request: {:?}", task_id, e);
                                }
                            },
                            _ => {
                                // Get health status 
                                let request = json!({
                                    "action": "get_health",
                                    "request_id": format!("health_{}_{}", task_id, op_id),
                                }).to_string();
                                
                                if let Err(e) = write.send(Message::Text(request)).await {
                                    println!("Task {} failed to send health request: {:?}", task_id, e);
                                }
                            }
                        }
                        
                        // Increment the operation counter
                        operations_completed_clone.fetch_add(1, Ordering::SeqCst);
                        
                        // Small delay to prevent flooding
                        if op_id % 10 == 0 {
                            sleep(Duration::from_millis(1)).await;
                        }
                    }
                    
                    // Cancel the read task
                    read_task.abort();
                    
                    println!("Task {} completed {} operations", task_id, OPERATIONS_PER_TASK);
                },
                Err(e) => {
                    println!("Task {} failed to connect: {:?}", task_id, e);
                }
            }
        });
        
        handles.push(handle);
        
        // Small delay between spawning clients to prevent connection failures
        sleep(Duration::from_millis(10)).await;
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        let _ = handle.await;
    }
    
    let total_time = start_time.elapsed();
    let completed = operations_completed.load(Ordering::SeqCst);
    let connections = connection_successes.load(Ordering::SeqCst);
    let messages = message_counter.load(Ordering::SeqCst);
    
    println!("Concurrent performance test complete:");
    println!("- {} concurrent tasks ({} successful connections)", NUM_CONCURRENT_OPERATIONS, connections);
    println!("- {} operations per task", OPERATIONS_PER_TASK);
    println!("- {} total operations", completed);
    println!("- {} total messages received", messages);
    println!("- Total time: {:?}", total_time);
    println!("- Operations per second: {}", (completed as f64 / total_time.as_secs_f64()) as u64);
    
    // Clean up
    server.stop().await?;
    
    // Success if we completed operations and had some successful connections
    assert!(completed > 0, "No operations were completed");
    assert!(connections > 0, "No connections were successful");
    
    Ok(())
}

/// Test memory usage under load
#[tokio::test]
async fn test_memory_usage() -> Result<()> {
    // Create a test harness
    let harness = MonitoringTestHarness::new();
    
    // Record initial memory usage
    let initial_memory = measure_memory_usage();
    println!("Initial memory usage: {} bytes", initial_memory);
    
    // Generate a large number of metrics
    const NUM_METRICS_BATCHES: usize = 1000;
    let metrics_batches = harness.generate_metrics(NUM_METRICS_BATCHES);
    let total_metrics: usize = metrics_batches.iter().map(|batch| batch.len()).sum();
    
    // Record memory after metrics generation
    let metrics_memory = measure_memory_usage();
    println!("Memory after generating {} metrics: {} bytes", total_metrics, metrics_memory);
    println!("Memory increase: {} bytes", metrics_memory.saturating_sub(initial_memory));
    
    // Generate health status data
    const NUM_HEALTH_UPDATES: usize = 1000;
    let mut health_statuses = Vec::with_capacity(NUM_HEALTH_UPDATES);
    
    for _ in 0..NUM_HEALTH_UPDATES {
        health_statuses.push(harness.generate_health_status());
    }
    
    // Record memory after health status generation
    let health_memory = measure_memory_usage();
    println!("Memory after generating {} health updates: {} bytes", NUM_HEALTH_UPDATES, health_memory);
    println!("Memory increase: {} bytes", health_memory.saturating_sub(metrics_memory));
    
    // Generate alerts
    const NUM_ALERTS: usize = 1000;
    let alerts = harness.generate_alerts(NUM_ALERTS);
    
    // Record memory after alert generation
    let alerts_memory = measure_memory_usage();
    println!("Memory after generating {} alerts: {} bytes", alerts.len(), alerts_memory);
    println!("Memory increase: {} bytes", alerts_memory.saturating_sub(health_memory));
    
    // Calculate memory usage per item
    let metrics_per_batch = metrics_batches.first().map(|batch| batch.len()).unwrap_or(0);
    if metrics_per_batch > 0 {
        let memory_per_metric = (metrics_memory.saturating_sub(initial_memory)) as f64 / (total_metrics as f64);
        println!("Estimated memory per metric: {:.2} bytes", memory_per_metric);
    }
    
    if !health_statuses.is_empty() {
        let components_per_status = health_statuses.first().map(|status| status.len()).unwrap_or(0);
        let memory_per_component = (health_memory.saturating_sub(metrics_memory)) as f64 / (NUM_HEALTH_UPDATES as f64 * components_per_status as f64);
        println!("Estimated memory per component health: {:.2} bytes", memory_per_component);
    }
    
    if !alerts.is_empty() {
        let memory_per_alert = (alerts_memory.saturating_sub(health_memory)) as f64 / (alerts.len() as f64);
        println!("Estimated memory per alert: {:.2} bytes", memory_per_alert);
    }
    
    Ok(())
}

/// Helper function to estimate current memory usage
fn measure_memory_usage() -> usize {
    // In a real implementation, this would use platform-specific
    // methods to measure actual memory usage
    
    // For now, we'll return a dummy value
    // This should be replaced with actual memory measurement
    
    // On Linux, could read /proc/self/statm
    // On Windows, could use GetProcessMemoryInfo
    // Or use a crate like psutil
    
    // Placeholder implementation
    std::thread::sleep(std::time::Duration::from_millis(10)); // Allow memory to stabilize
    1_000_000 // Return 1MB as a placeholder
} 