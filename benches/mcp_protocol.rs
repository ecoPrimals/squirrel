// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use squirrel_mcp::protocol::{McpProtocol, ProtocolMessage, MessageType, ProtocolConfig};
use squirrel_mcp::transport::{Transport, TransportConfig, MockTransport};
use squirrel_mcp::client::{McpClient, ClientConfig};
use squirrel_mcp::server::{McpServer, ServerConfig};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use serde_json::json;
use uuid::Uuid;

/// Benchmark message serialization/deserialization
fn benchmark_message_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_serialization");
    
    // Test different message sizes
    for payload_size in [100, 1000, 10000, 100000].iter() {
        group.throughput(Throughput::Bytes(*payload_size as u64));
        
        let large_payload = "x".repeat(*payload_size);
        let message = ProtocolMessage {
            id: Uuid::new_v4().to_string(),
            message_type: MessageType::Request,
            method: "test_method".to_string(),
            params: Some(json!({
                "data": large_payload,
                "timestamp": chrono::Utc::now(),
                "request_id": Uuid::new_v4()
            })),
            result: None,
            error: None,
        };
        
        group.bench_with_input(
            BenchmarkId::new("serialize", payload_size),
            &message,
            |b, message| {
                b.iter(|| {
                    let serialized = black_box(serde_json::to_string(message).unwrap());
                    black_box(serialized);
                });
            },
        );
        
        let serialized_message = serde_json::to_string(&message).unwrap();
        group.bench_with_input(
            BenchmarkId::new("deserialize", payload_size),
            &serialized_message,
            |b, serialized| {
                b.iter(|| {
                    let deserialized: ProtocolMessage = black_box(
                        serde_json::from_str(serialized).unwrap()
                    );
                    black_box(deserialized);
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark protocol message handling
fn benchmark_protocol_handling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("protocol_handling");
    
    // Test different message rates
    for messages_per_batch in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*messages_per_batch as u64));
        
        group.bench_with_input(
            BenchmarkId::new("message_processing", messages_per_batch),
            messages_per_batch,
            |b, &messages_per_batch| {
                b.to_async(&rt).iter(|| async {
                    let config = ProtocolConfig::default();
                    let protocol = McpProtocol::new(config).await.unwrap();
                    
                    // Process a batch of messages
                    for i in 0..messages_per_batch {
                        let message = ProtocolMessage {
                            id: format!("msg-{}", i),
                            message_type: MessageType::Request,
                            method: "benchmark_method".to_string(),
                            params: Some(json!({
                                "index": i,
                                "timestamp": chrono::Utc::now()
                            })),
                            result: None,
                            error: None,
                        };
                        
                        let _ = black_box(protocol.process_message(message).await);
                    }
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark client-server communication
fn benchmark_client_server_communication(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("client_server_communication");
    
    // Test different request patterns
    for concurrent_requests in [1, 10, 50, 100].iter() {
        group.throughput(Throughput::Elements(*concurrent_requests as u64));
        
        group.bench_with_input(
            BenchmarkId::new("request_response", concurrent_requests),
            concurrent_requests,
            |b, &concurrent_requests| {
                b.to_async(&rt).iter(|| async {
                    // Set up mock transport
                    let transport = Arc::new(MockTransport::new());
                    
                    // Create client and server
                    let client_config = ClientConfig::default();
                    let server_config = ServerConfig::default();
                    
                    let client = McpClient::new(client_config, transport.clone()).await.unwrap();
                    let server = McpServer::new(server_config, transport.clone()).await.unwrap();
                    
                    // Start server
                    let _server_handle = tokio::spawn(async move {
                        server.start().await.unwrap();
                    });
                    
                    // Send concurrent requests
                    let mut handles = Vec::new();
                    for i in 0..concurrent_requests {
                        let client_clone = client.clone();
                        handles.push(tokio::spawn(async move {
                            let request = json!({
                                "method": "benchmark_request",
                                "params": {
                                    "request_id": i,
                                    "data": format!("test-data-{}", i)
                                }
                            });
                            
                            client_clone.send_request("benchmark_method", request).await
                        }));
                    }
                    
                    // Wait for all requests to complete
                    for handle in handles {
                        let _ = black_box(handle.await.unwrap());
                    }
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark transport layer performance
fn benchmark_transport_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("transport_performance");
    
    // Test different message sizes over transport
    for message_size in [1024, 10240, 102400].iter() {
        group.throughput(Throughput::Bytes(*message_size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("transport_send_receive", message_size),
            message_size,
            |b, &message_size| {
                b.to_async(&rt).iter(|| async {
                    let transport = MockTransport::new();
                    let large_data = "x".repeat(message_size);
                    
                    let message = ProtocolMessage {
                        id: Uuid::new_v4().to_string(),
                        message_type: MessageType::Request,
                        method: "transport_test".to_string(),
                        params: Some(json!({
                            "data": large_data
                        })),
                        result: None,
                        error: None,
                    };
                    
                    // Simulate send and receive
                    transport.send(&message).await.unwrap();
                    let received = transport.receive().await.unwrap();
                    
                    black_box(received);
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark protocol overhead and efficiency
fn benchmark_protocol_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("protocol_overhead");
    
    group.bench_function("minimal_message", |b| {
        b.to_async(&rt).iter(|| async {
            let message = ProtocolMessage {
                id: "test-id".to_string(),
                message_type: MessageType::Request,
                method: "ping".to_string(),
                params: None,
                result: None,
                error: None,
            };
            
            // Measure end-to-end processing time
            let start = std::time::Instant::now();
            
            let serialized = serde_json::to_string(&message).unwrap();
            let _: ProtocolMessage = serde_json::from_str(&serialized).unwrap();
            
            let duration = start.elapsed();
            black_box(duration);
        });
    });
    
    group.bench_function("complex_message", |b| {
        b.to_async(&rt).iter(|| async {
            let message = ProtocolMessage {
                id: Uuid::new_v4().to_string(),
                message_type: MessageType::Request,
                method: "complex_operation".to_string(),
                params: Some(json!({
                    "nested_object": {
                        "array": vec![1, 2, 3, 4, 5],
                        "string_data": "complex data with unicode: 🚀",
                        "timestamp": chrono::Utc::now(),
                        "metadata": {
                            "version": "1.0.0",
                            "client_id": Uuid::new_v4()
                        }
                    }
                })),
                result: None,
                error: None,
            };
            
            // Measure end-to-end processing time
            let start = std::time::Instant::now();
            
            let serialized = serde_json::to_string(&message).unwrap();
            let _: ProtocolMessage = serde_json::from_str(&serialized).unwrap();
            
            let duration = start.elapsed();
            black_box(duration);
        });
    });
    
    group.finish();
}

/// Benchmark error handling in protocol
fn benchmark_protocol_error_handling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("protocol_error_handling");
    
    group.bench_function("malformed_messages", |b| {
        b.to_async(&rt).iter(|| async {
            let config = ProtocolConfig::default();
            let protocol = McpProtocol::new(config).await.unwrap();
            
            // Test various malformed messages
            let malformed_messages = vec![
                "invalid json",
                r#"{"incomplete": true"#,
                r#"{"id": null, "method": ""}"#,
                r#"{"id": "", "method": null}"#,
            ];
            
            for malformed in malformed_messages {
                let result = protocol.parse_message(malformed).await;
                black_box(result);
            }
        });
    });
    
    group.bench_function("timeout_handling", |b| {
        b.to_async(&rt).iter(|| async {
            let transport = MockTransport::with_latency(Duration::from_millis(100));
            let mut config = ClientConfig::default();
            config.request_timeout = Duration::from_millis(50); // Shorter than transport latency
            
            let client = McpClient::new(config, Arc::new(transport)).await.unwrap();
            
            // This should timeout
            let result = client.send_request("slow_method", json!({})).await;
            black_box(result);
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_message_serialization,
    benchmark_protocol_handling,
    benchmark_client_server_communication,
    benchmark_transport_performance,
    benchmark_protocol_overhead,
    benchmark_protocol_error_handling
);
criterion_main!(benches); 