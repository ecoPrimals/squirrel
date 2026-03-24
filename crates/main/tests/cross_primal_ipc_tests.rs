// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Cross-primal IPC integration tests.
//!
//! Validates JSON-RPC 2.0 communication patterns that all ecoPrimals
//! primals must support — absorbed from ecosystem-wide testing patterns
//! in petalTongue v0.5.20 and rhizoCrypt v0.13.

#![expect(
    clippy::expect_used,
    reason = "Test code: explicit expect and local lint noise"
)]

use serde_json::{Value, json};
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};

/// Helper: send a JSON-RPC request and read the response over a Unix socket
async fn jsonrpc_roundtrip(socket_path: &std::path::Path, method: &str, params: Value) -> Value {
    let mut stream = UnixStream::connect(socket_path)
        .await
        .expect("should succeed");
    let request = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });
    let bytes = serde_json::to_vec(&request).expect("should succeed");
    stream.write_all(&bytes).await.expect("should succeed");
    stream.write_all(b"\n").await.expect("should succeed");
    stream.shutdown().await.expect("should succeed");

    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).await.expect("should succeed");
    serde_json::from_slice(&buf).expect("should succeed")
}

/// Helper: spawn a minimal echo JSON-RPC server on a Unix socket
fn spawn_mock_server(socket_path: PathBuf) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let listener = UnixListener::bind(&socket_path).expect("should succeed");
        // Accept connections in a loop
        loop {
            let Ok((mut stream, _)) = listener.accept().await else {
                break;
            };
            tokio::spawn(async move {
                let mut buf = Vec::new();
                let _ = stream.read_to_end(&mut buf).await;
                if buf.is_empty() {
                    return;
                }
                let request: Value = match serde_json::from_slice(&buf) {
                    Ok(v) => v,
                    Err(_) => return,
                };
                let method = request.get("method").and_then(Value::as_str).unwrap_or("");
                let id = request.get("id").cloned().unwrap_or(Value::Null);

                let result = match method {
                    "system.health" => json!({
                        "status": "healthy",
                        "primal": "mock-primal",
                        "uptime_secs": 42
                    }),
                    "capability.list" => json!({
                        "primal": "mock-primal",
                        "version": "0.1.0",
                        "domain": "test",
                        "capabilities": ["test.ping", "test.echo"],
                        "domains": ["test"],
                        "locality": {
                            "local": ["test.ping", "test.echo"],
                            "external": []
                        },
                        "methods": {
                            "test.ping": { "cost": { "latency_ms": 1 } },
                            "test.echo": { "cost": { "latency_ms": 1 } }
                        },
                        "consumed_capabilities": []
                    }),
                    "error.test" => {
                        let response = json!({
                            "jsonrpc": "2.0",
                            "error": { "code": -32601, "message": "method not found" },
                            "id": id
                        });
                        let resp_bytes = serde_json::to_vec(&response).expect("should succeed");
                        let _ = stream.write_all(&resp_bytes).await;
                        return;
                    }
                    _ => json!({ "echo": method }),
                };

                let response = json!({
                    "jsonrpc": "2.0",
                    "result": result,
                    "id": id
                });
                let resp_bytes = serde_json::to_vec(&response).expect("should succeed");
                let _ = stream.write_all(&resp_bytes).await;
            });
        }
    })
}

#[tokio::test]
async fn test_cross_primal_health_exchange() {
    let tmp = tempfile::TempDir::new().expect("should succeed");
    let sock = tmp.path().join("health.sock");
    let server = spawn_mock_server(sock.clone());
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let resp = jsonrpc_roundtrip(&sock, "system.health", json!({})).await;
    let result = resp.get("result").expect("should succeed");
    assert_eq!(result["status"], "healthy");
    assert_eq!(result["primal"], "mock-primal");

    server.abort();
}

#[tokio::test]
async fn test_capability_list_ecosystem_format() {
    let tmp = tempfile::TempDir::new().expect("should succeed");
    let sock = tmp.path().join("caplist.sock");
    let server = spawn_mock_server(sock.clone());
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let resp = jsonrpc_roundtrip(&sock, "capability.list", json!({})).await;
    let result = resp.get("result").expect("should succeed");

    // Ecosystem consensus: flat "capabilities" array must be present
    assert!(
        result
            .get("capabilities")
            .expect("should succeed")
            .is_array()
    );
    let caps = result["capabilities"].as_array().expect("should succeed");
    assert!(!caps.is_empty());

    // Domain introspection fields
    assert!(result.get("domains").expect("should succeed").is_array());
    assert!(result.get("locality").expect("should succeed").is_object());
    assert!(
        result["locality"]
            .get("local")
            .expect("should succeed")
            .is_array()
    );
    assert!(
        result["locality"]
            .get("external")
            .expect("should succeed")
            .is_array()
    );

    // Legacy "methods" object still present for backward compat
    assert!(result.get("methods").expect("should succeed").is_object());

    server.abort();
}

#[tokio::test]
async fn test_ipc_error_phase_propagation() {
    let tmp = tempfile::TempDir::new().expect("should succeed");
    let sock = tmp.path().join("error.sock");
    let server = spawn_mock_server(sock.clone());
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let resp = jsonrpc_roundtrip(&sock, "error.test", json!({})).await;
    let error = resp.get("error").expect("should succeed");
    assert_eq!(error["code"], -32601);
    assert!(
        error["message"]
            .as_str()
            .expect("should succeed")
            .contains("not found")
    );

    server.abort();
}

#[tokio::test]
async fn test_connect_error_phase() {
    use universal_patterns::ipc_client::{IpcClient, IpcClientError, IpcErrorPhase};

    // Attempt to connect to a non-existent socket — must yield Connect-phase error
    let result = IpcClient::discover("nonexistent-primal-xyz-test");
    let Err(err) = result else {
        unreachable!("discover should fail for nonexistent primal");
    };
    let ipc_err = err
        .downcast_ref::<IpcClientError>()
        .expect("IpcClientError");
    assert_eq!(ipc_err.phase(), IpcErrorPhase::Connect);
}

#[tokio::test]
async fn test_concurrent_ipc_requests() {
    let tmp = tempfile::TempDir::new().expect("should succeed");
    let sock = tmp.path().join("concurrent.sock");
    let server = spawn_mock_server(sock.clone());
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let mut handles = Vec::new();
    for i in 0..10 {
        let sock_path = sock.clone();
        handles.push(tokio::spawn(async move {
            let resp = jsonrpc_roundtrip(&sock_path, "system.health", json!({"seq": i})).await;
            assert!(resp.get("result").is_some(), "request {i} failed");
        }));
    }

    for h in handles {
        h.await.expect("should succeed");
    }

    server.abort();
}

#[tokio::test]
async fn test_graceful_disconnect() {
    let tmp = tempfile::TempDir::new().expect("should succeed");
    let sock = tmp.path().join("disconnect.sock");
    let server = spawn_mock_server(sock.clone());
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Connect and immediately drop without sending data
    {
        let _stream = UnixStream::connect(&sock).await.expect("should succeed");
        // drop immediately
    }
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Server should still be healthy — verify with a normal request
    let resp = jsonrpc_roundtrip(&sock, "system.health", json!({})).await;
    assert_eq!(resp["result"]["status"], "healthy");

    server.abort();
}
