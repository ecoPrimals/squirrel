// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use tokio::time::Duration;

use super::*;

#[test]
fn client_construction_does_not_panic() {
    let client = IpcClient::new("/tmp/test-squirrel.sock");
    assert_eq!(client.socket_path, PathBuf::from("/tmp/test-squirrel.sock"));
    assert_eq!(client.request_timeout, Duration::from_secs(30));
    assert_eq!(client.connection_timeout, Duration::from_secs(5));
}

#[test]
fn client_timeout_configuration() {
    let client = IpcClient::new("/tmp/test.sock")
        .with_request_timeout(Duration::from_secs(60))
        .with_connection_timeout(Duration::from_secs(10));

    assert_eq!(client.request_timeout, Duration::from_secs(60));
    assert_eq!(client.connection_timeout, Duration::from_secs(10));
}

#[test]
fn socket_discovery_returns_xdg_path() {
    let path = IpcClient::discover_socket("test-service");
    let path_str = path.to_string_lossy();
    assert!(
        path_str.contains("biomeos") && path_str.ends_with("test-service.sock"),
        "expected XDG-compliant path, got: {path_str}"
    );
}

#[test]
fn request_id_increments() {
    let client = IpcClient::new("/tmp/test.sock");
    let id1 = client.next_id.fetch_add(1, Ordering::Relaxed);
    let id2 = client.next_id.fetch_add(1, Ordering::Relaxed);
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
}

#[test]
fn discover_fails_for_nonexistent_socket() {
    let result = IpcClient::discover("nonexistent-service-xyz");
    assert!(result.is_err());
}

#[test]
fn error_display_formatting() {
    let err = IpcClientError::Rpc {
        phase: IpcErrorPhase::JsonRpcError,
        code: -32601,
        message: "method not found".to_string(),
    };
    assert!(err.to_string().contains("-32601"));
    assert!(err.to_string().contains("method not found"));
}

#[test]
fn error_display_connection() {
    let err = IpcClientError::Connection {
        phase: IpcErrorPhase::Connect,
        message: "refused".to_string(),
    };
    assert!(err.to_string().contains("refused"));
    assert_eq!(err.phase(), IpcErrorPhase::Connect);
}

#[test]
fn error_display_timeout() {
    let err = IpcClientError::Timeout {
        phase: IpcErrorPhase::Read,
        duration: Duration::from_secs(5),
    };
    assert!(err.to_string().contains("5s"));
    assert_eq!(err.phase(), IpcErrorPhase::Read);
}

#[test]
fn error_display_not_found() {
    let err = IpcClientError::NotFound(PathBuf::from("/tmp/missing.sock"));
    assert!(err.to_string().contains("/tmp/missing.sock"));
    assert_eq!(err.phase(), IpcErrorPhase::Connect);
}

#[test]
fn error_retryable_by_phase() {
    assert!(
        IpcClientError::Connection {
            phase: IpcErrorPhase::Connect,
            message: "refused".into()
        }
        .is_retryable()
    );

    assert!(
        IpcClientError::Timeout {
            phase: IpcErrorPhase::Read,
            duration: Duration::from_secs(5)
        }
        .is_retryable()
    );

    assert!(
        !IpcClientError::Rpc {
            phase: IpcErrorPhase::JsonRpcError,
            code: -32601,
            message: "method not found".into()
        }
        .is_retryable()
    );
}

#[test]
fn error_constants() {
    assert_eq!(IpcClientError::PARSE_ERROR, -32700);
    assert_eq!(IpcClientError::INVALID_REQUEST, -32600);
    assert_eq!(IpcClientError::METHOD_NOT_FOUND, -32601);
    assert_eq!(IpcClientError::INVALID_PARAMS, -32602);
    assert_eq!(IpcClientError::INTERNAL_ERROR, -32603);
}

#[test]
fn http_response_serde() {
    let resp = HttpResponse {
        status: 200,
        headers: {
            let mut h = HashMap::new();
            h.insert("content-type".to_string(), "application/json".to_string());
            h
        },
        body: r#"{"ok": true}"#.to_string(),
    };
    let json = serde_json::to_string(&resp).expect("should succeed");
    let deser: HttpResponse = serde_json::from_str(&json).expect("should succeed");
    assert_eq!(deser.status, 200);
    assert_eq!(
        deser.headers.get("content-type").expect("should succeed"),
        "application/json"
    );
}

#[test]
fn capability_info_serde() {
    let info = CapabilityInfo {
        capability: "secure_http".to_string(),
        atomic_type: Some("Tower".to_string()),
        providers: vec![ProviderInfo {
            id: "provider-1".to_string(),
            socket: PathBuf::from("/tmp/p1.sock"),
            healthy: true,
            capabilities: vec!["http_proxy".to_string()],
        }],
        primary_socket: PathBuf::from("/tmp/p1.sock"),
    };
    let json = serde_json::to_string(&info).expect("should succeed");
    let deser: CapabilityInfo = serde_json::from_str(&json).expect("should succeed");
    assert_eq!(deser.capability, "secure_http");
    assert_eq!(deser.providers.len(), 1);
    assert!(deser.providers[0].healthy);
}

#[test]
fn routing_metrics_serde() {
    let metrics = RoutingMetrics {
        total_requests: 42,
        entries: vec![RoutingMetric {
            request_id: "req-1".to_string(),
            capability: "ai.query".to_string(),
            method: "POST".to_string(),
            latency_ms: 150,
            success: true,
            error: None,
        }],
    };
    let json = serde_json::to_string(&metrics).expect("should succeed");
    let deser: RoutingMetrics = serde_json::from_str(&json).expect("should succeed");
    assert_eq!(deser.total_requests, 42);
    assert_eq!(deser.entries.len(), 1);
    assert!(deser.entries[0].success);
}

#[test]
fn routing_metric_with_error() {
    let metric = RoutingMetric {
        request_id: "req-2".to_string(),
        capability: "storage".to_string(),
        method: "GET".to_string(),
        latency_ms: 5000,
        success: false,
        error: Some("timeout".to_string()),
    };
    let json = serde_json::to_string(&metric).expect("should succeed");
    let deser: RoutingMetric = serde_json::from_str(&json).expect("should succeed");
    assert!(!deser.success);
    assert_eq!(deser.error.as_deref(), Some("timeout"));
}

#[test]
fn client_builder_pattern() {
    let client = IpcClient::new("/tmp/test.sock")
        .with_request_timeout(Duration::from_secs(120))
        .with_connection_timeout(Duration::from_millis(500));

    assert_eq!(client.request_timeout, Duration::from_secs(120));
    assert_eq!(client.connection_timeout, Duration::from_millis(500));
    assert_eq!(client.socket_path, PathBuf::from("/tmp/test.sock"));
}

#[test]
fn parse_capabilities_new_format() {
    let resp = serde_json::json!({
        "primal": "test",
        "capabilities": ["ai.query", "system.health"]
    });
    let caps = parse_capabilities_from_response(&resp);
    assert_eq!(caps.len(), 2);
    assert!(caps.contains(&"ai.query".to_string()));
}

#[test]
fn parse_capabilities_legacy_format() {
    let resp = serde_json::json!({
        "primal": "test",
        "methods": {
            "ai.query": { "cost": { "latency_ms": 500 } },
            "system.health": { "cost": { "latency_ms": 1 } }
        }
    });
    let caps = parse_capabilities_from_response(&resp);
    assert_eq!(caps.len(), 2);
}

#[test]
fn parse_capabilities_prefers_new_format() {
    let resp = serde_json::json!({
        "capabilities": ["new.cap"],
        "methods": { "old.cap": {} }
    });
    let caps = parse_capabilities_from_response(&resp);
    assert_eq!(caps, vec!["new.cap"]);
}

#[test]
fn parse_capabilities_empty_response() {
    let resp = serde_json::json!({});
    let caps = parse_capabilities_from_response(&resp);
    assert!(caps.is_empty());
}

#[test]
fn parse_capabilities_nested_result() {
    let resp = serde_json::json!({
        "result": {
            "primal": "test",
            "capabilities": ["nested.cap"]
        }
    });
    let caps = parse_capabilities_from_response(&resp);
    assert_eq!(caps, vec!["nested.cap"]);
}

#[test]
fn parse_capabilities_double_nested_result() {
    let resp = serde_json::json!({
        "result": {
            "result": {
                "capabilities": ["deep.cap"]
            }
        }
    });
    let caps = parse_capabilities_from_response(&resp);
    assert_eq!(caps, vec!["deep.cap"]);
}

#[test]
fn parse_capabilities_nested_legacy_methods() {
    let resp = serde_json::json!({
        "result": {
            "methods": { "legacy.nested": {} }
        }
    });
    let caps = parse_capabilities_from_response(&resp);
    assert_eq!(caps, vec!["legacy.nested"]);
}

#[test]
fn parse_capabilities_double_nested_legacy_methods() {
    let resp = serde_json::json!({
        "result": {
            "result": {
                "methods": { "deep.legacy": {} }
            }
        }
    });
    let caps = parse_capabilities_from_response(&resp);
    assert_eq!(caps, vec!["deep.legacy"]);
}

#[test]
fn extract_rpc_error_from_error_response() {
    let resp = serde_json::json!({
        "jsonrpc": "2.0",
        "error": {
            "code": -32601,
            "message": "method not found",
            "data": {"method": "foo.bar"}
        },
        "id": 1
    });
    let err = extract_rpc_error(&resp).expect("should succeed");
    assert_eq!(err.code, -32601);
    assert_eq!(err.message, "method not found");
    assert!(err.is_method_not_found());
    assert!(!err.is_internal());
    assert!(err.data.is_some());
}

#[test]
fn extract_rpc_error_returns_none_for_success() {
    let resp = serde_json::json!({
        "jsonrpc": "2.0",
        "result": {"ok": true},
        "id": 1
    });
    assert!(extract_rpc_error(&resp).is_none());
}

#[test]
fn extract_rpc_error_internal() {
    let resp = serde_json::json!({
        "error": { "code": -32603, "message": "internal error" }
    });
    let err = extract_rpc_error(&resp).expect("should succeed");
    assert!(err.is_internal());
}

#[test]
fn extract_rpc_error_server_range() {
    let resp = serde_json::json!({
        "error": { "code": -32050, "message": "custom server error" }
    });
    let err = extract_rpc_error(&resp).expect("should succeed");
    assert!(err.is_server_error());
}

#[test]
fn rpc_error_display() {
    let err = RpcError {
        code: -32601,
        message: "method not found".into(),
        data: None,
    };
    assert!(err.to_string().contains("-32601"));
    assert!(err.to_string().contains("method not found"));
}

#[test]
fn extract_rpc_result_from_success() {
    let resp = serde_json::json!({
        "jsonrpc": "2.0",
        "result": {"ok": true},
        "id": 1
    });
    let result = extract_rpc_result(&resp).expect("should succeed");
    assert_eq!(result, serde_json::json!({"ok": true}));
}

#[test]
fn extract_rpc_result_from_error_returns_err() {
    let resp = serde_json::json!({
        "jsonrpc": "2.0",
        "error": {"code": -32601, "message": "method not found"},
        "id": 1
    });
    let err = extract_rpc_result(&resp).unwrap_err();
    assert_eq!(err.code, -32601);
    assert!(err.is_method_not_found());
}

#[test]
fn extract_rpc_result_missing_result_field() {
    let resp = serde_json::json!({"jsonrpc": "2.0", "id": 1});
    let err = extract_rpc_result(&resp).unwrap_err();
    assert!(err.message.contains("missing"));
}

#[test]
fn extract_rpc_result_null_result_is_ok() {
    let resp = serde_json::json!({
        "jsonrpc": "2.0",
        "result": null,
        "id": 1
    });
    let result = extract_rpc_result(&resp).expect("should succeed");
    assert!(result.is_null());
}

#[test]
fn extract_rpc_error_returns_none_when_code_not_integer() {
    let resp = serde_json::json!({
        "error": { "code": "not-a-number", "message": "bad" }
    });
    assert!(extract_rpc_error(&resp).is_none());
}

#[test]
fn parse_capabilities_filters_non_string_array_entries() {
    let resp = serde_json::json!({
        "capabilities": ["valid.cap", 42, null, "other.cap"]
    });
    let caps = parse_capabilities_from_response(&resp);
    assert_eq!(caps, vec!["valid.cap", "other.cap"]);
}

#[cfg(unix)]
mod unix_socket_call_tests {
    use super::*;
    use std::path::PathBuf;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixListener;
    use tokio::time::Duration;

    fn json_rpc_result(id: u64, result: serde_json::Value) -> Vec<u8> {
        let v = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result,
        });
        serde_json::to_vec(&v).expect("should succeed")
    }

    fn json_rpc_error(id: u64, error: serde_json::Value) -> Vec<u8> {
        let v = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": error,
        });
        serde_json::to_vec(&v).expect("should succeed")
    }

    #[tokio::test]
    async fn call_succeeds_with_mock_unix_server() {
        let dir = tempfile::tempdir().expect("should succeed");
        let sock_path = dir.path().join("ipc-test.sock");
        let _ = std::fs::remove_file(&sock_path);
        let listener = UnixListener::bind(&sock_path).expect("should succeed");
        let response = json_rpc_result(1, serde_json::json!({"echo": true}));
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("should succeed");
            let mut buf = vec![0u8; 16384];
            let _ = stream.read(&mut buf).await;
            stream.write_all(&response).await.expect("should succeed");
        });

        let client = IpcClient::new(&sock_path);
        let out = client
            .call("test.method", &serde_json::json!({ "x": 1 }))
            .await
            .expect("should succeed");
        assert_eq!(out, serde_json::json!({"echo": true}));
        server.await.expect("should succeed");
    }

    #[tokio::test]
    async fn call_returns_rpc_error_from_server() {
        let dir = tempfile::tempdir().expect("should succeed");
        let sock_path = dir.path().join("ipc-rpc-err.sock");
        let _ = std::fs::remove_file(&sock_path);
        let listener = UnixListener::bind(&sock_path).expect("should succeed");
        let response = json_rpc_error(
            1,
            serde_json::json!({ "code": -32601, "message": "method not found" }),
        );
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("should succeed");
            let mut buf = vec![0u8; 16384];
            let _ = stream.read(&mut buf).await;
            stream.write_all(&response).await.expect("should succeed");
        });

        let client = IpcClient::new(&sock_path);
        let err = client
            .call("x", &serde_json::Value::Null)
            .await
            .unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("-32601") || msg.contains("method not found"));
        server.await.expect("should succeed");
    }

    #[tokio::test]
    async fn call_rpc_error_uses_default_code_when_missing() {
        let dir = tempfile::tempdir().expect("should succeed");
        let sock_path = dir.path().join("ipc-default-code.sock");
        let _ = std::fs::remove_file(&sock_path);
        let listener = UnixListener::bind(&sock_path).expect("should succeed");
        let response = json_rpc_error(1, serde_json::json!({ "message": "oops" }));
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("should succeed");
            let mut buf = vec![0u8; 16384];
            let _ = stream.read(&mut buf).await;
            stream.write_all(&response).await.expect("should succeed");
        });

        let client = IpcClient::new(&sock_path);
        let err = client
            .call("x", &serde_json::Value::Null)
            .await
            .unwrap_err();
        assert!(format!("{err:#}").contains(&format!("{}", IpcClientError::INTERNAL_ERROR)));
        server.await.expect("should succeed");
    }

    #[tokio::test]
    async fn call_rpc_error_uses_unknown_message_when_missing() {
        let dir = tempfile::tempdir().expect("should succeed");
        let sock_path = dir.path().join("ipc-default-msg.sock");
        let _ = std::fs::remove_file(&sock_path);
        let listener = UnixListener::bind(&sock_path).expect("should succeed");
        let response = json_rpc_error(1, serde_json::json!({ "code": -32000 }));
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("should succeed");
            let mut buf = vec![0u8; 16384];
            let _ = stream.read(&mut buf).await;
            stream.write_all(&response).await.expect("should succeed");
        });

        let client = IpcClient::new(&sock_path);
        let err = client
            .call("x", &serde_json::Value::Null)
            .await
            .unwrap_err();
        assert!(format!("{err:#}").contains("unknown error"));
        server.await.expect("should succeed");
    }

    #[tokio::test]
    async fn call_fails_when_result_missing_on_success_envelope() {
        let dir = tempfile::tempdir().expect("should succeed");
        let sock_path = dir.path().join("ipc-no-result.sock");
        let _ = std::fs::remove_file(&sock_path);
        let listener = UnixListener::bind(&sock_path).expect("should succeed");
        let response = br#"{"jsonrpc":"2.0","id":1}"#.to_vec();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("should succeed");
            let mut buf = vec![0u8; 16384];
            let _ = stream.read(&mut buf).await;
            stream.write_all(&response).await.expect("should succeed");
        });

        let client = IpcClient::new(&sock_path);
        let err = client
            .call("x", &serde_json::Value::Null)
            .await
            .unwrap_err();
        assert!(format!("{err:#}").contains("result"));
        server.await.expect("should succeed");
    }

    #[tokio::test]
    async fn call_fails_on_invalid_json_response() {
        let dir = tempfile::tempdir().expect("should succeed");
        let sock_path = dir.path().join("ipc-bad-json.sock");
        let _ = std::fs::remove_file(&sock_path);
        let listener = UnixListener::bind(&sock_path).expect("should succeed");
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("should succeed");
            let mut buf = vec![0u8; 16384];
            let _ = stream.read(&mut buf).await;
            stream
                .write_all(b"not json at all")
                .await
                .expect("should succeed");
        });

        let client = IpcClient::new(&sock_path);
        let err = client
            .call("x", &serde_json::Value::Null)
            .await
            .unwrap_err();
        assert!(format!("{err:#}").to_lowercase().contains("parse"));
        server.await.expect("should succeed");
    }

    #[tokio::test]
    async fn call_fails_connection_refused() {
        let client = IpcClient::new("/nonexistent/path/to/ipc-does-not-exist.sock");
        let err = client
            .call("x", &serde_json::Value::Null)
            .await
            .unwrap_err();
        assert!(
            format!("{err:#}").to_lowercase().contains("connection")
                || format!("{err:#}").to_lowercase().contains("no such file")
        );
    }

    #[tokio::test]
    async fn call_times_out_when_server_never_responds() {
        let dir = tempfile::tempdir().expect("should succeed");
        let sock_path = dir.path().join("ipc-slow.sock");
        let _ = std::fs::remove_file(&sock_path);
        let listener = UnixListener::bind(&sock_path).expect("should succeed");
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("should succeed");
            let mut buf = vec![0u8; 16384];
            let _ = stream.read(&mut buf).await;
            // Never respond — client should hit its 80ms timeout without
            // the server needing to block for 60 seconds.
            std::future::pending::<()>().await;
            let _ = stream.write_all(b"{}").await;
        });

        let client = IpcClient::new(&sock_path).with_request_timeout(Duration::from_millis(80));
        let err = client
            .call("x", &serde_json::Value::Null)
            .await
            .unwrap_err();
        assert!(
            format!("{err:#}").to_lowercase().contains("timed out")
                || format!("{err:#}").to_lowercase().contains("timeout")
        );
        server.abort();
    }

    #[tokio::test]
    async fn proxy_http_parses_http_response_from_call() {
        let dir = tempfile::tempdir().expect("should succeed");
        let sock_path = dir.path().join("ipc-proxy.sock");
        let _ = std::fs::remove_file(&sock_path);
        let listener = UnixListener::bind(&sock_path).expect("should succeed");
        let http_payload = serde_json::json!({
            "status": 201,
            "headers": { "x-test": "1" },
            "body": "created"
        });
        let response = json_rpc_result(1, http_payload);
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("should succeed");
            let mut buf = vec![0u8; 32768];
            let _ = stream.read(&mut buf).await;
            stream.write_all(&response).await.expect("should succeed");
        });

        let client = IpcClient::new(&sock_path);
        let resp = client
            .proxy_http("POST", "https://example.test/", None, None)
            .await
            .expect("should succeed");
        assert_eq!(resp.status, 201);
        assert_eq!(resp.body, "created");
        assert_eq!(resp.headers.get("x-test").map(String::as_str), Some("1"));
        server.await.expect("should succeed");
    }

    #[tokio::test]
    async fn discover_capability_parses_capability_info() {
        let dir = tempfile::tempdir().expect("should succeed");
        let sock_path = dir.path().join("ipc-cap.sock");
        let _ = std::fs::remove_file(&sock_path);
        let listener = UnixListener::bind(&sock_path).expect("should succeed");
        let cap = CapabilityInfo {
            capability: "secure_http".into(),
            atomic_type: Some("Tower".into()),
            providers: vec![],
            primary_socket: PathBuf::from("/tmp/p.sock"),
        };
        let response = json_rpc_result(1, serde_json::to_value(&cap).expect("should succeed"));
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("should succeed");
            let mut buf = vec![0u8; 32768];
            let _ = stream.read(&mut buf).await;
            stream.write_all(&response).await.expect("should succeed");
        });

        let client = IpcClient::new(&sock_path);
        let info = client
            .discover_capability("secure_http")
            .await
            .expect("should succeed");
        assert_eq!(info.capability, "secure_http");
        assert_eq!(info.atomic_type.as_deref(), Some("Tower"));
        server.await.expect("should succeed");
    }

    #[tokio::test]
    async fn route_by_capability_returns_value() {
        let dir = tempfile::tempdir().expect("should succeed");
        let sock_path = dir.path().join("ipc-route.sock");
        let _ = std::fs::remove_file(&sock_path);
        let listener = UnixListener::bind(&sock_path).expect("should succeed");
        let response = json_rpc_result(1, serde_json::json!({ "ok": true, "n": 7 }));
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("should succeed");
            let mut buf = vec![0u8; 32768];
            let _ = stream.read(&mut buf).await;
            stream.write_all(&response).await.expect("should succeed");
        });

        let client = IpcClient::new(&sock_path);
        let v = client
            .route_by_capability("ai.query", "tool.run", serde_json::json!({ "a": 1 }))
            .await
            .expect("should succeed");
        assert_eq!(v["n"], 7);
        server.await.expect("should succeed");
    }

    #[tokio::test]
    async fn get_metrics_parses_routing_metrics() {
        let dir = tempfile::tempdir().expect("should succeed");
        let sock_path = dir.path().join("ipc-metrics.sock");
        let _ = std::fs::remove_file(&sock_path);
        let listener = UnixListener::bind(&sock_path).expect("should succeed");
        let metrics = RoutingMetrics {
            total_requests: 3,
            entries: vec![],
        };
        let response = json_rpc_result(1, serde_json::to_value(&metrics).expect("should succeed"));
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("should succeed");
            let mut buf = vec![0u8; 32768];
            let _ = stream.read(&mut buf).await;
            stream.write_all(&response).await.expect("should succeed");
        });

        let client = IpcClient::new(&sock_path);
        let m = client.get_metrics().await.expect("should succeed");
        assert_eq!(m.total_requests, 3);
        server.await.expect("should succeed");
    }

    #[test]
    fn discover_succeeds_when_xdg_runtime_socket_exists() {
        let dir = tempfile::tempdir().expect("should succeed");
        let runtime = dir.path();
        let biomeos = runtime.join("biomeos");
        std::fs::create_dir_all(&biomeos).expect("should succeed");
        let sock_path = biomeos.join("xdg_discover_test.sock");
        let _listener = std::os::unix::net::UnixListener::bind(&sock_path).expect("should succeed");

        temp_env::with_vars(
            [(
                "XDG_RUNTIME_DIR",
                Some(runtime.to_str().expect("utf8 temp path")),
            )],
            || {
                let client = IpcClient::discover("xdg_discover_test").expect("should succeed");
                assert_eq!(client.socket_path, sock_path);
            },
        );
    }

    #[tokio::test]
    async fn proxy_http_fails_on_malformed_http_payload() {
        let dir = tempfile::tempdir().expect("should succeed");
        let sock_path = dir.path().join("ipc-proxy-bad.sock");
        let _ = std::fs::remove_file(&sock_path);
        let listener = UnixListener::bind(&sock_path).expect("should succeed");
        let response = json_rpc_result(1, serde_json::json!("not an object"));
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("should succeed");
            let mut buf = vec![0u8; 32768];
            let _ = stream.read(&mut buf).await;
            stream.write_all(&response).await.expect("should succeed");
        });

        let client = IpcClient::new(&sock_path);
        let err = client
            .proxy_http("GET", "https://x.test/", None, None)
            .await
            .unwrap_err();
        assert!(format!("{err:#}").to_lowercase().contains("parse"));
        server.await.expect("should succeed");
    }
}
