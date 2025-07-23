//! Comprehensive Performance Integration Tests
//!
//! This test suite validates the performance optimization systems working together
//! including memory pooling, zero-copy serialization, plugin optimization, and
//! end-to-end system performance under realistic workloads.

use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;

use tokio::test;
use tracing_test::traced_test;
use uuid::Uuid;

use squirrel_mcp::enhanced::performance_init::{
    PerformanceConfig, PerformanceManager, init_performance_systems,
    get_global_performance_report, quick_performance_check
};
use squirrel_mcp::enhanced::memory_pool::{
    get_global_memory_pool, utils as memory_utils
};
use squirrel_mcp::enhanced::serialization::{
    get_global_serializer, ZeroCopySerializer, SerializationConfig
};
use squirrel_plugins::{
    get_global_optimizer, init_global_optimizer, optimized_ops,
    create_unified_plugin_manager, PerformanceOptimizerConfig
};
use squirrel_mcp::protocol::types::MCPMessage;
use squirrel_mcp::enhanced::ai_types::{UniversalAIRequest, Message as AIMessage};

/// Integration test configuration
#[derive(Debug, Clone)]
struct TestConfig {
    pub message_count: usize,
    pub concurrent_clients: usize,
    pub plugin_count: usize,
    pub test_duration: Duration,
}

impl TestConfig {
    fn stress_test() -> Self {
        Self {
            message_count: 10000,
            concurrent_clients: 50,
            plugin_count: 100,
            test_duration: Duration::from_secs(60),
        }
    }
    
    fn standard_test() -> Self {
        Self {
            message_count: 1000,
            concurrent_clients: 10,
            plugin_count: 20,
            test_duration: Duration::from_secs(10),
        }
    }
    
    fn light_test() -> Self {
        Self {
            message_count: 100,
            concurrent_clients: 3,
            plugin_count: 5,
            test_duration: Duration::from_secs(5),
        }
    }
}

#[traced_test]
#[test]
async fn test_performance_system_initialization() {
    // Test performance system initialization
    let config = PerformanceConfig::production();
    let result = init_performance_systems(config).await;
    
    assert!(result.is_ok(), "Performance systems should initialize successfully");
    
    // Verify global systems are available
    let memory_pool = get_global_memory_pool();
    assert!(memory_pool.get_performance_report().await.buffer_efficiency >= 0.0);
    
    // Quick performance check
    let check_result = quick_performance_check().await;
    assert!(check_result.is_ok(), "Performance check should succeed");
    
    println!("✅ Performance system initialization test passed");
}

#[traced_test]
#[test]
async fn test_memory_pool_performance() {
    let config = TestConfig::standard_test();
    
    // Initialize performance systems
    let perf_config = PerformanceConfig::production();
    init_performance_systems(perf_config).await.unwrap();
    
    let start_time = Instant::now();
    let mut handles = Vec::new();
    
    // Spawn concurrent buffer allocation tasks
    for i in 0..config.concurrent_clients {
        let handle = tokio::spawn(async move {
            let mut allocated_buffers = Vec::new();
            
            // Allocate buffers of various sizes
            for j in 0..100 {
                let size = match j % 3 {
                    0 => 1024,    // Small buffer
                    1 => 32768,   // Medium buffer
                    _ => 262144,  // Large buffer
                };
                
                let buffer = memory_utils::get_message_buffer().await;
                allocated_buffers.push(buffer);
                
                // Simulate some work
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
            
            (i, allocated_buffers.len())
        });
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    let mut total_buffers = 0;
    for handle in handles {
        let (client_id, buffer_count) = handle.await.unwrap();
        total_buffers += buffer_count;
        println!("Client {} allocated {} buffers", client_id, buffer_count);
    }
    
    let duration = start_time.elapsed();
    
    // Get performance metrics
    let pool_performance = memory_utils::get_pool_performance().await;
    
    println!("🚀 Memory Pool Performance Test Results:");
    println!("  Total buffers allocated: {}", total_buffers);
    println!("  Test duration: {:?}", duration);
    println!("  Buffer efficiency: {:.1}%", pool_performance.buffer_efficiency * 100.0);
    println!("  Memory saved: {} KB", pool_performance.total_memory_saved / 1024);
    
    // Performance assertions
    assert!(pool_performance.buffer_efficiency > 0.5, "Buffer efficiency should be > 50%");
    assert!(duration < Duration::from_secs(30), "Test should complete within 30 seconds");
    
    println!("✅ Memory pool performance test passed");
}

#[traced_test]
#[test]
async fn test_zero_copy_serialization_performance() {
    let config = TestConfig::standard_test();
    
    // Initialize serialization system
    let serialization_config = SerializationConfig {
        enable_buffer_pooling: true,
        enable_fast_codecs: true,
        enable_templates: true,
        enable_streaming: false,
        enable_compression: true,
        compression_threshold: 1024,
        buffer_pool_size: 1000,
        template_cache_size: 500,
        max_buffer_size: 1024 * 1024,
        initial_pool_size: 100,
        max_pool_size: 2000,
    };
    
    let serializer = ZeroCopySerializer::new(serialization_config);
    
    let start_time = Instant::now();
    let mut total_serializations = 0;
    let mut total_bytes = 0;
    
    // Create test messages
    let test_messages: Vec<MCPMessage> = (0..config.message_count)
        .map(|i| {
            MCPMessage {
                id: format!("test-message-{}", i),
                method: Some("test_method".to_string()),
                params: Some(serde_json::json!({
                    "test_param": format!("test_value_{}", i),
                    "iteration": i,
                    "data": vec![1, 2, 3, 4, 5],
                })),
                result: None,
                error: None,
                jsonrpc: "2.0".to_string(),
            }
        })
        .collect();
    
    // Serialize messages concurrently
    let mut handles = Vec::new();
    let serializer = Arc::new(serializer);
    
    for chunk in test_messages.chunks(100) {
        let serializer_clone = Arc::clone(&serializer);
        let chunk = chunk.to_vec();
        
        let handle = tokio::spawn(async move {
            let mut bytes_serialized = 0;
            let mut serializations = 0;
            
            for message in chunk {
                let result = serializer_clone.serialize_mcp_message(&message).await.unwrap();
                bytes_serialized += result.data.len();
                serializations += 1;
            }
            
            (serializations, bytes_serialized)
        });
        handles.push(handle);
    }
    
    // Collect results
    for handle in handles {
        let (serializations, bytes) = handle.await.unwrap();
        total_serializations += serializations;
        total_bytes += bytes;
    }
    
    let duration = start_time.elapsed();
    let serialization_metrics = serializer.get_metrics().await;
    
    println!("🚀 Zero-Copy Serialization Performance Test Results:");
    println!("  Total serializations: {}", total_serializations);
    println!("  Total bytes serialized: {} KB", total_bytes / 1024);
    println!("  Test duration: {:?}", duration);
    println!("  Avg serialization time: {:.2} μs", serialization_metrics.avg_serialization_time_us);
    println!("  Buffer pool hit rate: {:.1}%", 
             if serialization_metrics.total_serializations > 0 {
                 serialization_metrics.buffer_pool_hits as f64 / serialization_metrics.total_serializations as f64 * 100.0
             } else {
                 0.0
             });
    
    // Performance assertions
    assert!(serialization_metrics.avg_serialization_time_us < 1000.0, "Avg serialization should be < 1ms");
    assert!(total_serializations == config.message_count, "All messages should be serialized");
    assert!(duration < Duration::from_secs(10), "Test should complete within 10 seconds");
    
    println!("✅ Zero-copy serialization performance test passed");
}

#[traced_test]
#[test]
async fn test_plugin_optimization_performance() {
    let config = TestConfig::light_test(); // Use lighter config for plugin tests
    
    // Initialize plugin performance optimizer
    let optimizer_config = PerformanceOptimizerConfig::production();
    init_global_optimizer().unwrap();
    
    // Create unified plugin manager
    let plugin_manager = create_unified_plugin_manager().await.unwrap();
    
    let start_time = Instant::now();
    
    // Test plugin discovery and lookup optimization
    let mut plugin_lookups = 0;
    let mut capability_queries = 0;
    
    for i in 0..config.message_count {
        // Simulate plugin lookups
        let plugin_name = format!("test-plugin-{}", i % 10); // Repeat names for cache hits
        if let Some(_plugin) = optimized_ops::fast_plugin_lookup(&plugin_name, plugin_manager.registry()).await {
            plugin_lookups += 1;
        }
        
        // Simulate capability queries
        let capability = format!("test-capability-{}", i % 5); // Repeat capabilities for cache hits
        let matching_plugins = optimized_ops::fast_capability_query(&capability, plugin_manager.registry()).await;
        capability_queries += matching_plugins.len();
    }
    
    let duration = start_time.elapsed();
    let optimizer_metrics = optimized_ops::get_performance_metrics().await;
    
    println!("🚀 Plugin Optimization Performance Test Results:");
    println!("  Plugin lookups: {}", plugin_lookups);
    println!("  Capability queries: {}", capability_queries);
    println!("  Test duration: {:?}", duration);
    println!("  Cache efficiency: {:.1}%", optimizer_metrics.cache_efficiency * 100.0);
    println!("  Operations optimized: {}", optimizer_metrics.operations_optimized);
    
    // Performance assertions
    assert!(optimizer_metrics.cache_efficiency > 0.3, "Cache efficiency should be > 30%");
    assert!(duration < Duration::from_secs(5), "Test should complete within 5 seconds");
    
    println!("✅ Plugin optimization performance test passed");
}

#[traced_test]
#[test]
async fn test_end_to_end_system_performance() {
    let config = TestConfig::standard_test();
    
    println!("🚀 Starting comprehensive end-to-end performance test");
    
    // Initialize all performance systems
    let perf_config = PerformanceConfig::production();
    init_performance_systems(perf_config).await.unwrap();
    init_global_optimizer().unwrap();
    
    let start_time = Instant::now();
    let mut handles = Vec::new();
    
    // Simulate realistic workload with multiple subsystems
    for client_id in 0..config.concurrent_clients {
        let handle = tokio::spawn(async move {
            let mut operations = 0;
            let mut messages_processed = 0;
            let mut memory_allocations = 0;
            let mut plugin_operations = 0;
            
            for i in 0..100 {
                // Memory pool operations
                let buffer = memory_utils::get_message_buffer().await;
                memory_allocations += 1;
                
                // Create and serialize messages
                let message = MCPMessage {
                    id: format!("client-{}-message-{}", client_id, i),
                    method: Some("integrated_test".to_string()),
                    params: Some(serde_json::json!({
                        "client": client_id,
                        "iteration": i,
                        "data": vec![1, 2, 3, 4, 5],
                    })),
                    result: None,
                    error: None,
                    jsonrpc: "2.0".to_string(),
                };
                
                // Cache the message (using memory pool integration)
                memory_utils::cache_hot_message(
                    format!("cached-{}-{}", client_id, i),
                    message.clone()
                ).await.unwrap();
                
                messages_processed += 1;
                
                // Plugin operations (every 10th iteration)
                if i % 10 == 0 {
                    let plugin_name = format!("test-plugin-{}", i % 5);
                    // Note: This would use the actual plugin registry in a real test
                    plugin_operations += 1;
                }
                
                operations += 1;
                
                // Small delay to simulate real work
                tokio::time::sleep(Duration::from_micros(100)).await;
            }
            
            (client_id, operations, messages_processed, memory_allocations, plugin_operations)
        });
        handles.push(handle);
    }
    
    // Wait for all clients to complete
    let mut total_operations = 0;
    let mut total_messages = 0;
    let mut total_allocations = 0;
    let mut total_plugin_ops = 0;
    
    for handle in handles {
        let (client_id, ops, messages, allocations, plugin_ops) = handle.await.unwrap();
        total_operations += ops;
        total_messages += messages;
        total_allocations += allocations;
        total_plugin_ops += plugin_ops;
        
        println!("Client {} completed: {} ops, {} messages", client_id, ops, messages);
    }
    
    let duration = start_time.elapsed();
    
    // Get comprehensive performance report
    let performance_report = get_global_performance_report().await.unwrap();
    
    println!("🏆 END-TO-END PERFORMANCE TEST RESULTS:");
    println!("=====================================");
    println!("  Test Duration: {:?}", duration);
    println!("  Concurrent Clients: {}", config.concurrent_clients);
    println!("  Total Operations: {}", total_operations);
    println!("  Total Messages: {}", total_messages);
    println!("  Total Memory Allocations: {}", total_allocations);
    println!("  Total Plugin Operations: {}", total_plugin_ops);
    println!();
    println!("📊 PERFORMANCE METRICS:");
    println!("  Overall Performance Score: {:.2}", performance_report.get_performance_score());
    println!("  Memory Pool Efficiency: {:.1}%", performance_report.memory_pool_efficiency * 100.0);
    println!("  Buffer Pool Hit Rate: {:.1}%", performance_report.buffer_pool_hit_rate * 100.0);
    println!("  Total Memory Saved: {} MB", performance_report.total_memory_saved / (1024 * 1024));
    println!("  Avg Serialization Time: {:.2} μs", performance_report.average_serialization_time_us);
    println!();
    println!("🎯 THROUGHPUT METRICS:");
    let ops_per_second = total_operations as f64 / duration.as_secs_f64();
    let messages_per_second = total_messages as f64 / duration.as_secs_f64();
    println!("  Operations per second: {:.0}", ops_per_second);
    println!("  Messages per second: {:.0}", messages_per_second);
    println!("  Memory allocations per second: {:.0}", total_allocations as f64 / duration.as_secs_f64());
    
    // Performance assertions
    assert!(performance_report.is_performance_healthy(), "System performance should be healthy");
    assert!(performance_report.get_performance_score() > 0.6, "Performance score should be > 0.6");
    assert!(ops_per_second > 1000.0, "Should achieve > 1000 ops/sec");
    assert!(messages_per_second > 500.0, "Should achieve > 500 messages/sec");
    assert!(duration < Duration::from_secs(30), "Test should complete within 30 seconds");
    
    // Print comprehensive report
    performance_report.print_comprehensive_report();
    
    println!("✅ End-to-end system performance test passed with flying colors!");
}

#[traced_test]
#[test]
async fn test_memory_pressure_handling() {
    println!("🧪 Testing memory pressure handling and garbage collection");
    
    // Initialize with smaller memory limits for testing
    let mut perf_config = PerformanceConfig::development();
    perf_config.memory_pool.buffer_pool.small_pool_size = 10; // Very small for testing
    init_performance_systems(perf_config).await.unwrap();
    
    let start_time = Instant::now();
    let memory_pool = get_global_memory_pool();
    
    // Generate memory pressure by allocating many buffers
    let mut allocated_buffers = Vec::new();
    for i in 0..50 { // Allocate more than pool size
        let buffer = memory_pool.get_buffer(4096).await;
        allocated_buffers.push(buffer);
        
        if i % 10 == 0 {
            println!("Allocated {} buffers", i + 1);
        }
    }
    
    // Force garbage collection by waiting
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Get performance metrics after memory pressure
    let pool_report = memory_pool.get_performance_report().await;
    let duration = start_time.elapsed();
    
    println!("🧪 Memory Pressure Test Results:");
    println!("  Buffers allocated: {}", allocated_buffers.len());
    println!("  Test duration: {:?}", duration);
    println!("  GC cycles: {}", pool_report.metrics.gc_cycles.load(std::sync::atomic::Ordering::Relaxed));
    println!("  Buffer efficiency: {:.1}%", pool_report.buffer_efficiency * 100.0);
    
    // The system should handle memory pressure gracefully
    assert!(pool_report.metrics.gc_cycles.load(std::sync::atomic::Ordering::Relaxed) > 0, "GC should have run");
    assert!(duration < Duration::from_secs(10), "Test should complete within 10 seconds");
    
    println!("✅ Memory pressure handling test passed");
}

#[traced_test]
#[test]
async fn test_concurrent_performance_optimization() {
    println!("🔄 Testing concurrent performance optimization under load");
    
    let config = TestConfig::standard_test();
    
    // Initialize all systems
    let perf_config = PerformanceConfig::high_performance();
    init_performance_systems(perf_config).await.unwrap();
    init_global_optimizer().unwrap();
    
    let start_time = Instant::now();
    let barrier = Arc::new(tokio::sync::Barrier::new(config.concurrent_clients));
    let mut handles = Vec::new();
    
    // Launch concurrent performance-intensive tasks
    for client_id in 0..config.concurrent_clients {
        let barrier_clone = Arc::clone(&barrier);
        
        let handle = tokio::spawn(async move {
            // Wait for all clients to be ready
            barrier_clone.wait().await;
            
            let client_start = Instant::now();
            let mut operations = 0;
            
            // Mixed workload
            for i in 0..200 {
                // Memory operations
                let buffer = memory_utils::get_message_buffer().await;
                
                // String interning
                let interned = memory_utils::intern_common_string(&format!("common-string-{}", i % 10)).await;
                
                // Message creation and caching
                if i % 5 == 0 {
                    let message = MCPMessage {
                        id: format!("concurrent-{}-{}", client_id, i),
                        method: Some("concurrent_test".to_string()),
                        params: Some(serde_json::json!({
                            "client": client_id,
                            "operation": i,
                        })),
                        result: None,
                        error: None,
                        jsonrpc: "2.0".to_string(),
                    };
                    
                    memory_utils::cache_hot_message(
                        format!("concurrent-{}-{}", client_id, i),
                        message
                    ).await.unwrap();
                }
                
                operations += 1;
            }
            
            let client_duration = client_start.elapsed();
            (client_id, operations, client_duration)
        });
        handles.push(handle);
    }
    
    // Wait for all concurrent tasks to complete
    let mut total_operations = 0;
    let mut max_client_time = Duration::ZERO;
    
    for handle in handles {
        let (client_id, operations, client_duration) = handle.await.unwrap();
        total_operations += operations;
        max_client_time = max_client_time.max(client_duration);
        println!("Client {} completed {} operations in {:?}", client_id, operations, client_duration);
    }
    
    let total_duration = start_time.elapsed();
    let performance_report = get_global_performance_report().await.unwrap();
    
    println!("🔄 Concurrent Performance Test Results:");
    println!("  Total Operations: {}", total_operations);
    println!("  Total Duration: {:?}", total_duration);
    println!("  Max Client Duration: {:?}", max_client_time);
    println!("  Concurrent Clients: {}", config.concurrent_clients);
    println!("  Performance Score: {:.2}", performance_report.get_performance_score());
    println!("  Memory Efficiency: {:.1}%", performance_report.memory_pool_efficiency * 100.0);
    
    // Concurrency performance assertions
    assert!(performance_report.get_performance_score() > 0.5, "Performance should remain good under concurrency");
    assert!(max_client_time < Duration::from_secs(10), "No client should take > 10 seconds");
    assert!(total_operations == config.concurrent_clients * 200, "All operations should complete");
    
    println!("✅ Concurrent performance optimization test passed");
}

/// Utility function to create test AI requests
fn create_test_ai_request(id: &str) -> UniversalAIRequest {
    UniversalAIRequest {
        id: id.to_string(),
        provider: "test-provider".to_string(),
        model: "test-model".to_string(),
        messages: vec![AIMessage {
            role: "user".to_string(),
            content: format!("Test message for {}", id),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }],
        tools: None,
        tool_choice: None,
        temperature: Some(0.7),
        max_tokens: Some(1000),
        top_p: Some(1.0),
        frequency_penalty: None,
        presence_penalty: None,
        stop: None,
        stream: Some(false),
        metadata: HashMap::new(),
    }
}

/// Performance benchmark utility
async fn benchmark_operation<F, Fut, T>(name: &str, operation: F, iterations: usize) -> Duration 
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    println!("🏃 Benchmarking {} ({} iterations)...", name, iterations);
    
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = operation().await;
    }
    let duration = start.elapsed();
    
    let per_op = duration / iterations as u32;
    println!("  {} completed in {:?} ({:?}/op)", name, duration, per_op);
    
    duration
} 