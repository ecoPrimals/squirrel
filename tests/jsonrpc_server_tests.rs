// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Comprehensive tests for JSON-RPC server
//!
//! This test suite includes:
//! - Unit tests for JSON-RPC protocol
//! - E2E tests for Unix socket communication
//! - Chaos tests for fault tolerance
//! - Performance tests

use serde_json::{json, Value};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::timeout;

/// Helper to send JSON-RPC request and get response
async fn send_jsonrpc_request(
    socket_path: &str,
    method: &str,
    params: Option<Value>,
    id: i32,
) -> Result<Value, String> {
    let stream = UnixStream::connect(socket_path)
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    let request = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": id
    });

    let mut request_str = serde_json::to_string(&request).unwrap();
    request_str.push('\n');

    let (read_half, mut write_half) = stream.into_split();
    write_half
        .write_all(request_str.as_bytes())
        .await
        .map_err(|e| format!("Failed to write: {}", e))?;

    let mut reader = BufReader::new(read_half);
    let mut response_line = String::new();

    timeout(Duration::from_secs(5), reader.read_line(&mut response_line))
        .await
        .map_err(|_| "Request timeout".to_string())?
        .map_err(|e| format!("Failed to read: {}", e))?;

    serde_json::from_str(&response_line).map_err(|e| format!("Failed to parse response: {}", e))
}

// ============================================================================
// UNIT TESTS - JSON-RPC Protocol
// ============================================================================

#[cfg(test)]
mod unit_tests {
    use super::*;
    use squirrel::rpc::jsonrpc_server::{JsonRpcRequest, JsonRpcResponse, JsonRpcError, error_codes};
    use std::sync::Arc;

    #[test]
    fn test_jsonrpc_request_valid() {
        let json_str = r#"{"jsonrpc":"2.0","method":"health","id":1}"#;
        let request: Result<JsonRpcRequest, _> = serde_json::from_str(json_str);
        assert!(request.is_ok());
        
        let req = request.unwrap();
        assert_eq!(req.jsonrpc.as_ref(), "2.0");
        assert_eq!(req.method.as_ref(), "health");
        assert_eq!(req.id, Some(json!(1)));
    }

    #[test]
    fn test_jsonrpc_request_with_params() {
        let json_str = r#"{"jsonrpc":"2.0","method":"query_ai","params":{"prompt":"Hello"},"id":1}"#;
        let request: Result<JsonRpcRequest, _> = serde_json::from_str(json_str);
        assert!(request.is_ok());
        
        let req = request.unwrap();
        assert_eq!(req.method.as_ref(), "query_ai");
        assert!(req.params.is_some());
    }

    #[test]
    fn test_jsonrpc_response_success() {
        let response = JsonRpcResponse {
            jsonrpc: Arc::from("2.0"),
            result: Some(json!({"status": "healthy"})),
            error: None,
            id: json!(1),
        };
        
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"result\""));
        assert!(!json.contains("\"error\""));
    }

    #[test]
    fn test_jsonrpc_response_error() {
        let response = JsonRpcResponse {
            jsonrpc: Arc::from("2.0"),
            result: None,
            error: Some(JsonRpcError {
                code: error_codes::METHOD_NOT_FOUND,
                message: "Method not found".to_string(),
                data: None,
            }),
            id: json!(1),
        };
        
        let json = serde_json::to_string(&response).unwrap();
        assert!(!json.contains("\"result\""));
        assert!(json.contains("\"error\""));
        assert!(json.contains("-32601")); // METHOD_NOT_FOUND code
    }

    #[test]
    fn test_jsonrpc_error_codes() {
        assert_eq!(error_codes::PARSE_ERROR, -32700);
        assert_eq!(error_codes::INVALID_REQUEST, -32600);
        assert_eq!(error_codes::METHOD_NOT_FOUND, -32601);
        assert_eq!(error_codes::INVALID_PARAMS, -32602);
        assert_eq!(error_codes::INTERNAL_ERROR, -32603);
    }

    #[test]
    fn test_server_metrics_initialization() {
        use squirrel::rpc::jsonrpc_server::ServerMetrics;
        
        let metrics = ServerMetrics::new();
        assert_eq!(metrics.requests_handled, 0);
        assert_eq!(metrics.errors, 0);
        assert!(metrics.uptime_seconds() >= 0);
        assert!(metrics.avg_response_time_ms().is_none());
    }

    #[test]
    fn test_server_metrics_avg_calculation() {
        use squirrel::rpc::jsonrpc_server::ServerMetrics;
        
        let mut metrics = ServerMetrics::new();
        metrics.requests_handled = 10;
        metrics.total_response_time_ms = 500;
        
        assert_eq!(metrics.avg_response_time_ms(), Some(50.0));
    }
}

// ============================================================================
// E2E TESTS - Unix Socket Communication
// ============================================================================

#[cfg(test)]
mod e2e_tests {
    use super::*;
    use squirrel::rpc::JsonRpcServer;
    use tempfile::tempdir;

    async fn start_test_server(socket_path: String) -> Arc<JsonRpcServer> {
        let server = Arc::new(JsonRpcServer::new(socket_path.clone()));
        let server_clone = Arc::clone(&server);
        
        tokio::spawn(async move {
            let _ = server_clone.start().await;
        });
        
        // Give server time to bind
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        server
    }

    #[tokio::test]
    async fn test_e2e_health_check() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test_health.sock");
        let socket_str = socket_path.to_str().unwrap().to_string();
        
        let _server = start_test_server(socket_str.clone()).await;
        
        let response = send_jsonrpc_request(&socket_str, "health", None, 1)
            .await
            .expect("Failed to get health check response");
        
        assert_eq!(response["jsonrpc"], "2.0");
        assert!(response["result"].is_object());
        assert_eq!(response["result"]["status"], "healthy");
        assert!(response["result"]["uptime_seconds"].is_number());
    }

    #[tokio::test]
    async fn test_e2e_list_providers_no_ai_router() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test_providers.sock");
        let socket_str = socket_path.to_str().unwrap().to_string();
        
        let _server = start_test_server(socket_str.clone()).await;
        
        let response = send_jsonrpc_request(&socket_str, "list_providers", None, 2)
            .await
            .expect("Failed to get providers response");
        
        assert_eq!(response["jsonrpc"], "2.0");
        assert!(response["result"].is_object());
        assert_eq!(response["result"]["total"], 0);
        assert!(response["result"]["providers"].is_array());
    }

    #[tokio::test]
    async fn test_e2e_announce_capabilities() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test_announce.sock");
        let socket_str = socket_path.to_str().unwrap().to_string();
        
        let _server = start_test_server(socket_str.clone()).await;
        
        let params = json!({
            "capabilities": ["ai.inference", "ai.routing", "tool.orchestration"]
        });
        
        let response = send_jsonrpc_request(&socket_str, "announce_capabilities", Some(params), 3)
            .await
            .expect("Failed to announce capabilities");
        
        assert_eq!(response["jsonrpc"], "2.0");
        assert!(response["result"].is_object());
        assert_eq!(response["result"]["success"], true);
    }

    #[tokio::test]
    async fn test_e2e_method_not_found() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test_not_found.sock");
        let socket_str = socket_path.to_str().unwrap().to_string();
        
        let _server = start_test_server(socket_str.clone()).await;
        
        let response = send_jsonrpc_request(&socket_str, "nonexistent_method", None, 4)
            .await
            .expect("Failed to get error response");
        
        assert_eq!(response["jsonrpc"], "2.0");
        assert!(response["error"].is_object());
        assert_eq!(response["error"]["code"], -32601); // METHOD_NOT_FOUND
    }

    #[tokio::test]
    async fn test_e2e_multiple_sequential_requests() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test_sequential.sock");
        let socket_str = socket_path.to_str().unwrap().to_string();
        
        let _server = start_test_server(socket_str.clone()).await;
        
        // Send multiple requests
        for i in 1..=5 {
            let response = send_jsonrpc_request(&socket_str, "health", None, i)
                .await
                .expect("Failed to get response");
            
            assert_eq!(response["id"], i);
            assert_eq!(response["result"]["status"], "healthy");
        }
    }

    #[tokio::test]
    async fn test_e2e_concurrent_requests() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test_concurrent.sock");
        let socket_str = socket_path.to_str().unwrap().to_string();
        
        let _server = start_test_server(socket_str.clone()).await;
        
        // Send concurrent requests
        let mut handles = vec![];
        for i in 1..=10 {
            let socket_clone = socket_str.clone();
            let handle = tokio::spawn(async move {
                send_jsonrpc_request(&socket_clone, "health", None, i).await
            });
            handles.push(handle);
        }
        
        // Wait for all to complete
        for (idx, handle) in handles.into_iter().enumerate() {
            let response = handle.await.unwrap().expect("Request failed");
            assert_eq!(response["jsonrpc"], "2.0");
            assert_eq!(response["result"]["status"], "healthy");
            assert_eq!(response["id"], (idx + 1) as i32);
        }
    }
}

// ============================================================================
// CHAOS TESTS - Fault Tolerance
// ============================================================================

#[cfg(test)]
mod chaos_tests {
    use super::*;
    use squirrel::rpc::JsonRpcServer;
    use tempfile::tempdir;

    async fn start_test_server(socket_path: String) -> Arc<JsonRpcServer> {
        let server = Arc::new(JsonRpcServer::new(socket_path.clone()));
        let server_clone = Arc::clone(&server);
        
        tokio::spawn(async move {
            let _ = server_clone.start().await;
        });
        
        tokio::time::sleep(Duration::from_millis(100)).await;
        server
    }

    #[tokio::test]
    async fn test_chaos_malformed_json() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test_malformed.sock");
        let socket_str = socket_path.to_str().unwrap().to_string();
        
        let _server = start_test_server(socket_str.clone()).await;
        
        let stream = UnixStream::connect(&socket_str).await.unwrap();
        let (read_half, mut write_half) = stream.into_split();
        
        // Send malformed JSON
        write_half.write_all(b"{not valid json}\n").await.unwrap();
        
        let mut reader = BufReader::new(read_half);
        let mut response_line = String::new();
        reader.read_line(&mut response_line).await.unwrap();
        
        let response: Value = serde_json::from_str(&response_line).unwrap();
        assert_eq!(response["jsonrpc"], "2.0");
        assert!(response["error"].is_object());
        assert_eq!(response["error"]["code"], -32700); // PARSE_ERROR
    }

    #[tokio::test]
    async fn test_chaos_invalid_jsonrpc_version() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test_invalid_version.sock");
        let socket_str = socket_path.to_str().unwrap().to_string();
        
        let _server = start_test_server(socket_str.clone()).await;
        
        let stream = UnixStream::connect(&socket_str).await.unwrap();
        let (read_half, mut write_half) = stream.into_split();
        
        // Send request with invalid version
        let request = json!({
            "jsonrpc": "1.0",
            "method": "health",
            "id": 1
        });
        
        let mut request_str = serde_json::to_string(&request).unwrap();
        request_str.push('\n');
        write_half.write_all(request_str.as_bytes()).await.unwrap();
        
        let mut reader = BufReader::new(read_half);
        let mut response_line = String::new();
        reader.read_line(&mut response_line).await.unwrap();
        
        let response: Value = serde_json::from_str(&response_line).unwrap();
        assert!(response["error"].is_object());
        assert_eq!(response["error"]["code"], -32600); // INVALID_REQUEST
    }

    #[tokio::test]
    async fn test_chaos_missing_params() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test_missing_params.sock");
        let socket_str = socket_path.to_str().unwrap().to_string();
        
        let _server = start_test_server(socket_str.clone()).await;
        
        // Try to query AI without params
        let response = send_jsonrpc_request(&socket_str, "query_ai", None, 1)
            .await
            .expect("Failed to get error response");
        
        assert!(response["error"].is_object());
        assert_eq!(response["error"]["code"], -32602); // INVALID_PARAMS
    }

    #[tokio::test]
    async fn test_chaos_connection_drop() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test_drop.sock");
        let socket_str = socket_path.to_str().unwrap().to_string();
        
        let _server = start_test_server(socket_str.clone()).await;
        
        // Connect and immediately drop
        {
            let _stream = UnixStream::connect(&socket_str).await.unwrap();
            // Stream dropped here
        }
        
        // Server should still be responsive
        let response = send_jsonrpc_request(&socket_str, "health", None, 1)
            .await
            .expect("Server should still respond after connection drop");
        
        assert_eq!(response["result"]["status"], "healthy");
    }

    #[tokio::test]
    async fn test_chaos_partial_write() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test_partial.sock");
        let socket_str = socket_path.to_str().unwrap().to_string();
        
        let _server = start_test_server(socket_str.clone()).await;
        
        let stream = UnixStream::connect(&socket_str).await.unwrap();
        let (_, mut write_half) = stream.into_split();
        
        // Write incomplete JSON (no newline, no closing brace)
        write_half.write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"health\"").await.unwrap();
        drop(write_half);
        
        // Server should handle this gracefully and still work
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        let response = send_jsonrpc_request(&socket_str, "health", None, 1)
            .await
            .expect("Server should recover from partial write");
        
        assert_eq!(response["result"]["status"], "healthy");
    }

    #[tokio::test]
    async fn test_chaos_rapid_connect_disconnect() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test_rapid.sock");
        let socket_str = socket_path.to_str().unwrap().to_string();
        
        let _server = start_test_server(socket_str.clone()).await;
        
        // Rapidly connect and disconnect
        for _ in 0..20 {
            let _stream = UnixStream::connect(&socket_str).await.unwrap();
            // Immediately drop
        }
        
        // Server should still be healthy
        let response = send_jsonrpc_request(&socket_str, "health", None, 1)
            .await
            .expect("Server should handle rapid connections");
        
        assert_eq!(response["result"]["status"], "healthy");
    }

    #[tokio::test]
    async fn test_chaos_large_payload() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test_large.sock");
        let socket_str = socket_path.to_str().unwrap().to_string();
        
        let _server = start_test_server(socket_str.clone()).await;
        
        // Create a very large prompt (1MB)
        let large_prompt = "a".repeat(1_000_000);
        let params = json!({
            "prompt": large_prompt,
            "provider": "auto"
        });
        
        // Should handle gracefully (either accept or reject with proper error)
        let response = send_jsonrpc_request(&socket_str, "query_ai", Some(params), 1)
            .await
            .expect("Server should handle large payload");
        
        assert_eq!(response["jsonrpc"], "2.0");
        // Either result or error, but must respond
        assert!(response["result"].is_object() || response["error"].is_object());
    }

    #[tokio::test]
    async fn test_chaos_unicode_in_params() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test_unicode.sock");
        let socket_str = socket_path.to_str().unwrap().to_string();
        
        let _server = start_test_server(socket_str.clone()).await;
        
        let params = json!({
            "capabilities": ["🦀 Rust", "🐿️ Squirrel", "🌍 Universal", "日本語", "العربية"]
        });
        
        let response = send_jsonrpc_request(&socket_str, "announce_capabilities", Some(params), 1)
            .await
            .expect("Server should handle Unicode");
        
        assert_eq!(response["result"]["success"], true);
    }

    #[tokio::test]
    async fn test_chaos_empty_method() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test_empty_method.sock");
        let socket_str = socket_path.to_str().unwrap().to_string();
        
        let _server = start_test_server(socket_str.clone()).await;
        
        let response = send_jsonrpc_request(&socket_str, "", None, 1)
            .await
            .expect("Server should handle empty method");
        
        assert!(response["error"].is_object());
        assert_eq!(response["error"]["code"], -32601); // METHOD_NOT_FOUND
    }
}

// ============================================================================
// PERFORMANCE TESTS
// ============================================================================

#[cfg(test)]
mod performance_tests {
    use super::*;
    use squirrel::rpc::JsonRpcServer;
    use tempfile::tempdir;

    async fn start_test_server(socket_path: String) -> Arc<JsonRpcServer> {
        let server = Arc::new(JsonRpcServer::new(socket_path.clone()));
        let server_clone = Arc::clone(&server);
        
        tokio::spawn(async move {
            let _ = server_clone.start().await;
        });
        
        tokio::time::sleep(Duration::from_millis(100)).await;
        server
    }

    #[tokio::test]
    async fn test_perf_throughput() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test_throughput.sock");
        let socket_str = socket_path.to_str().unwrap().to_string();
        
        let _server = start_test_server(socket_str.clone()).await;
        
        let start = std::time::Instant::now();
        let num_requests = 100;
        
        for i in 0..num_requests {
            let _ = send_jsonrpc_request(&socket_str, "health", None, i).await;
        }
        
        let elapsed = start.elapsed();
        let requests_per_sec = num_requests as f64 / elapsed.as_secs_f64();
        
        println!("Throughput: {:.2} requests/sec", requests_per_sec);
        assert!(requests_per_sec > 50.0, "Throughput should be > 50 req/sec");
    }

    #[tokio::test]
    async fn test_perf_latency() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test_latency.sock");
        let socket_str = socket_path.to_str().unwrap().to_string();
        
        let _server = start_test_server(socket_str.clone()).await;
        
        let mut latencies = vec![];
        
        for i in 0..50 {
            let start = std::time::Instant::now();
            let _ = send_jsonrpc_request(&socket_str, "health", None, i).await;
            latencies.push(start.elapsed().as_millis());
        }
        
        let avg_latency: u128 = latencies.iter().sum::<u128>() / latencies.len() as u128;
        let max_latency = *latencies.iter().max().unwrap();
        
        println!("Average latency: {}ms", avg_latency);
        println!("Max latency: {}ms", max_latency);
        
        assert!(avg_latency < 50, "Average latency should be < 50ms");
        assert!(max_latency < 200, "Max latency should be < 200ms");
    }
}

