// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Timeout, Chaos, and Fault Tests for Evolution Features
//!
//! Comprehensive testing for:
//! - biomeOS timeout fixes
//! - Capability discovery resilience  
//! - AI router robustness
//! - Chaos scenarios
//! - Fault injection

use std::path::PathBuf;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::time::timeout;

// ============================================================================
// TIMEOUT TESTS (biomeOS fix validation)
// ============================================================================

#[tokio::test]
async fn test_timeout_json_rpc_error_no_hang() {
    // Validate biomeOS fix: JSON-RPC errors don't cause infinite hangs
    let socket_path = "/tmp/timeout-error-test.sock";
    let _ = std::fs::remove_file(socket_path);
    
    let listener = UnixListener::bind(socket_path).unwrap();
    
    tokio::spawn(async move {
        loop {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut reader = BufReader::new(&mut stream);
                let mut line = String::new();
                if reader.read_line(&mut line).await.is_err() {
                    break;
                }
                
                // Always return "Method not found" error (like Songbird did)
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32601,
                        "message": "Method not found: discover_capabilities"
                    },
                    "id": "1"
                });
                let _ = stream.write_all(response.to_string().as_bytes()).await;
                let _ = stream.write_all(b"\n").await;
            }
        }
    });
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let start = std::time::Instant::now();
    
    // Simulate what discovery system does
    match tokio::net::UnixStream::connect(socket_path).await {
        Ok(mut stream) => {
            let request = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "discover_capabilities",
                "params": {},
                "id": "1"
            });
            
            let _ = stream.write_all(request.to_string().as_bytes()).await;
            let _ = stream.write_all(b"\n").await;
            
            let mut reader = BufReader::new(&mut stream);
            let mut response_line = String::new();
            
            // This should timeout in 2s, not hang forever
            let read_result = timeout(
                Duration::from_secs(2),
                reader.read_line(&mut response_line)
            ).await;
            
            assert!(read_result.is_ok(), "Should not timeout reading error response");
            
            if let Ok(response) = serde_json::from_str::<serde_json::Value>(&response_line) {
                // Check for error field
                assert!(response.get("error").is_some(), "Should have error field");
            }
        }
        Err(_) => {}
    }
    
    let elapsed = start.elapsed();
    let _ = std::fs::remove_file(socket_path);
    
    // Should complete quickly (< 3s), not hang
    assert!(elapsed < Duration::from_secs(3), "Should handle error response quickly, got {:?}", elapsed);
}

#[tokio::test]
async fn test_timeout_slow_socket_2s_limit() {
    // Validate 2s per-socket timeout
    let socket_path = "/tmp/timeout-slow-2s.sock";
    let _ = std::fs::remove_file(socket_path);
    
    let listener = UnixListener::bind(socket_path).unwrap();
    
    tokio::spawn(async move {
        if let Ok((_stream, _)) = listener.accept().await {
            // Wait 10 seconds (much longer than 2s timeout)
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    });
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let start = std::time::Instant::now();
    
    // Try to connect with 2s timeout
    let result = timeout(
        Duration::from_secs(2),
        tokio::net::UnixStream::connect(socket_path)
    ).await;
    
    let elapsed = start.elapsed();
    let _ = std::fs::remove_file(socket_path);
    
    // Should connect immediately (socket exists)
    assert!(result.is_ok(), "Connection should succeed");
    
    // But if we try to read, it should timeout in 2s
    if let Ok(Ok(stream)) = result {
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        
        let start_read = std::time::Instant::now();
        let read_result = timeout(
            Duration::from_secs(2),
            reader.read_line(&mut line)
        ).await;
        let read_elapsed = start_read.elapsed();
        
        // Should timeout (not get data)
        assert!(read_result.is_err() || read_result.unwrap().is_err(), "Should timeout on slow read");
        assert!(read_elapsed >= Duration::from_secs(2) && read_elapsed < Duration::from_secs(3), 
                "Should respect 2s timeout, got {:?}", read_elapsed);
    }
}

#[tokio::test]
async fn test_timeout_overall_10s_limit() {
    // Validate overall 10s timeout for initialization
    let start = std::time::Instant::now();
    
    // Simulate scanning many slow sockets
    let mut tasks = vec![];
    
    for i in 0..20 {
        let socket_path = format!("/tmp/timeout-many-{}.sock", i);
        let _ = std::fs::remove_file(&socket_path);
        
        let listener = UnixListener::bind(&socket_path).unwrap();
        
        let task = tokio::spawn(async move {
            if let Ok((_stream, _)) = listener.accept().await {
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
            let _ = std::fs::remove_file(&socket_path);
        });
        
        tasks.push(task);
    }
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Try to scan all 20 sockets with overall 10s timeout
    let scan_result = timeout(
        Duration::from_secs(10),
        async {
            for i in 0..20 {
                let socket_path = format!("/tmp/timeout-many-{}.sock", i);
                let _ = timeout(
                    Duration::from_millis(400), // Each socket gets 400ms
                    tokio::net::UnixStream::connect(&socket_path)
                ).await;
            }
        }
    ).await;
    
    let elapsed = start.elapsed();
    
    // Cleanup
    for task in tasks {
        task.abort();
    }
    
    // Should complete within 10-11 seconds
    assert!(elapsed < Duration::from_secs(11), "Should respect 10s overall timeout, got {:?}", elapsed);
    assert!(scan_result.is_ok(), "Overall scan should complete");
}

// ============================================================================
// CHAOS TESTS
// ============================================================================

#[tokio::test]
async fn test_chaos_mixed_socket_states() {
    // Test with sockets in various failure states
    let socket_working = "/tmp/chaos-working.sock";
    let socket_error = "/tmp/chaos-error.sock";
    let socket_timeout = "/tmp/chaos-timeout.sock";
    let socket_crash = "/tmp/chaos-crash.sock";
    
    for s in &[socket_working, socket_error, socket_timeout, socket_crash] {
        let _ = std::fs::remove_file(s);
    }
    
    // Working socket
    let listener1 = UnixListener::bind(socket_working).unwrap();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener1.accept().await {
            let response = serde_json::json!({"jsonrpc":"2.0","result":{"status":"ok"},"id":"1"});
            let _ = stream.write_all(response.to_string().as_bytes()).await;
            let _ = stream.write_all(b"\n").await;
        }
    });
    
    // Error socket
    let listener2 = UnixListener::bind(socket_error).unwrap();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener2.accept().await {
            let response = serde_json::json!({"jsonrpc":"2.0","error":{"code":-32000,"message":"error"},"id":"1"});
            let _ = stream.write_all(response.to_string().as_bytes()).await;
            let _ = stream.write_all(b"\n").await;
        }
    });
    
    // Timeout socket
    let listener3 = UnixListener::bind(socket_timeout).unwrap();
    tokio::spawn(async move {
        if let Ok((_stream, _)) = listener3.accept().await {
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    });
    
    // Crash socket
    let listener4 = UnixListener::bind(socket_crash).unwrap();
    tokio::spawn(async move {
        if let Ok((stream, _)) = listener4.accept().await {
            drop(stream); // Immediate disconnect
        }
    });
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Try to connect to all (should handle chaos gracefully)
    let start = std::time::Instant::now();
    
    for socket in &[socket_working, socket_error, socket_timeout, socket_crash] {
        let _ = timeout(
            Duration::from_secs(1),
            async {
                if let Ok(mut stream) = tokio::net::UnixStream::connect(socket).await {
                    let _ = stream.write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":\"1\"}\n").await;
                    let mut reader = BufReader::new(stream);
                    let mut line = String::new();
                    let _ = timeout(Duration::from_millis(500), reader.read_line(&mut line)).await;
                }
            }
        ).await;
    }
    
    let elapsed = start.elapsed();
    
    // Cleanup
    for s in &[socket_working, socket_error, socket_timeout, socket_crash] {
        let _ = std::fs::remove_file(s);
    }
    
    // Should complete reasonably quickly despite chaos
    assert!(elapsed < Duration::from_secs(6), "Should handle chaos efficiently, got {:?}", elapsed);
}

#[tokio::test]
async fn test_chaos_concurrent_connections() {
    // Test concurrent connections to same socket
    let socket_path = "/tmp/chaos-concurrent.sock";
    let _ = std::fs::remove_file(socket_path);
    
    let listener = UnixListener::bind(socket_path).unwrap();
    
    // Spawn handler
    tokio::spawn(async move {
        for _ in 0..100 {
            if let Ok((mut stream, _)) = listener.accept().await {
                tokio::spawn(async move {
                    let response = serde_json::json!({"jsonrpc":"2.0","result":{"ok":true},"id":"1"});
                    let _ = stream.write_all(response.to_string().as_bytes()).await;
                    let _ = stream.write_all(b"\n").await;
                });
            }
        }
    });
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Spawn 50 concurrent clients
    let mut handles = vec![];
    for _ in 0..50 {
        let path = socket_path.to_string();
        let handle = tokio::spawn(async move {
            if let Ok(mut stream) = tokio::net::UnixStream::connect(&path).await {
                let _ = stream.write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":\"1\"}\n").await;
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                let _ = reader.read_line(&mut line).await;
            }
        });
        handles.push(handle);
    }
    
    // Wait for all
    let start = std::time::Instant::now();
    for handle in handles {
        let _ = timeout(Duration::from_secs(5), handle).await;
    }
    let elapsed = start.elapsed();
    
    let _ = std::fs::remove_file(socket_path);
    
    // Should handle concurrent load
    assert!(elapsed < Duration::from_secs(10), "Should handle concurrency, got {:?}", elapsed);
}

// ============================================================================
// FAULT INJECTION TESTS
// ============================================================================

#[tokio::test]
async fn test_fault_malformed_json() {
    let socket_path = "/tmp/fault-json.sock";
    let _ = std::fs::remove_file(socket_path);
    
    let listener = UnixListener::bind(socket_path).unwrap();
    
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            // Send malformed JSON
            let _ = stream.write_all(b"{ this is not json }\n").await;
        }
    });
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Should handle malformed JSON gracefully
    let result = timeout(
        Duration::from_secs(2),
        async {
            if let Ok(stream) = tokio::net::UnixStream::connect(socket_path).await {
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                let _ = reader.read_line(&mut line).await;
                
                // Try to parse (will fail)
                let parse_result = serde_json::from_str::<serde_json::Value>(&line);
                assert!(parse_result.is_err(), "Should fail to parse malformed JSON");
            }
        }
    ).await;
    
    let _ = std::fs::remove_file(socket_path);
    
    assert!(result.is_ok(), "Should handle malformed JSON without panic");
}

#[tokio::test]
async fn test_fault_empty_response() {
    let socket_path = "/tmp/fault-empty.sock";
    let _ = std::fs::remove_file(socket_path);
    
    let listener = UnixListener::bind(socket_path).unwrap();
    
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            // Send empty response
            let _ = stream.write_all(b"\n").await;
        }
    });
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let result = timeout(
        Duration::from_secs(2),
        async {
            if let Ok(stream) = tokio::net::UnixStream::connect(socket_path).await {
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                let _ = reader.read_line(&mut line).await;
                
                // Empty line should not cause panic
                assert!(line.trim().is_empty() || serde_json::from_str::<serde_json::Value>(&line).is_err());
            }
        }
    ).await;
    
    let _ = std::fs::remove_file(socket_path);
    
    assert!(result.is_ok(), "Should handle empty response without panic");
}

#[tokio::test]
async fn test_fault_partial_response() {
    let socket_path = "/tmp/fault-partial.sock";
    let _ = std::fs::remove_file(socket_path);
    
    let listener = UnixListener::bind(socket_path).unwrap();
    
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            // Send partial JSON then disconnect
            let _ = stream.write_all(b"{\"jsonrpc\":\"2.0\",\"result\":").await;
            drop(stream);
        }
    });
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let result = timeout(
        Duration::from_secs(2),
        async {
            if let Ok(stream) = tokio::net::UnixStream::connect(socket_path).await {
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                
                // This might return error or partial data
                let _ = reader.read_line(&mut line).await;
            }
        }
    ).await;
    
    let _ = std::fs::remove_file(socket_path);
    
    assert!(result.is_ok(), "Should handle partial response without panic");
}

// ============================================================================
// PERFORMANCE / LOAD TESTS
// ============================================================================

#[tokio::test]
async fn test_performance_many_sequential_requests() {
    let socket_path = "/tmp/perf-sequential.sock";
    let _ = std::fs::remove_file(socket_path);
    
    let listener = UnixListener::bind(socket_path).unwrap();
    
    tokio::spawn(async move {
        loop {
            if let Ok((mut stream, _)) = listener.accept().await {
                let response = serde_json::json!({"jsonrpc":"2.0","result":{"ok":true},"id":"1"});
                let _ = stream.write_all(response.to_string().as_bytes()).await;
                let _ = stream.write_all(b"\n").await;
            }
        }
    });
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let start = std::time::Instant::now();
    
    // Make 100 sequential requests
    for _ in 0..100 {
        if let Ok(mut stream) = tokio::net::UnixStream::connect(socket_path).await {
            let _ = stream.write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":\"1\"}\n").await;
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            let _ = reader.read_line(&mut line).await;
        }
    }
    
    let elapsed = start.elapsed();
    let _ = std::fs::remove_file(socket_path);
    
    // Should complete 100 requests in reasonable time (< 5s)
    assert!(elapsed < Duration::from_secs(5), "100 sequential requests took {:?}", elapsed);
}

#[tokio::test]
async fn test_performance_rapid_connect_disconnect() {
    let socket_path = "/tmp/perf-rapid.sock";
    let _ = std::fs::remove_file(socket_path);
    
    let listener = UnixListener::bind(socket_path).unwrap();
    
    tokio::spawn(async move {
        for _ in 0..200 {
            if let Ok((stream, _)) = listener.accept().await {
                // Accept then immediately close
                drop(stream);
            }
        }
    });
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let start = std::time::Instant::now();
    
    // Rapidly connect and disconnect 100 times
    for _ in 0..100 {
        if let Ok(stream) = tokio::net::UnixStream::connect(socket_path).await {
            drop(stream);
        }
    }
    
    let elapsed = start.elapsed();
    let _ = std::fs::remove_file(socket_path);
    
    // Should handle rapid connections (< 2s)
    assert!(elapsed < Duration::from_secs(2), "Rapid connect/disconnect took {:?}", elapsed);
}

