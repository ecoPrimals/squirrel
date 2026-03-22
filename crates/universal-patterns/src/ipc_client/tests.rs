// SPDX-License-Identifier: AGPL-3.0-only
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
    let json = serde_json::to_string(&resp).unwrap();
    let deser: HttpResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.status, 200);
    assert_eq!(
        deser.headers.get("content-type").unwrap(),
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
    let json = serde_json::to_string(&info).unwrap();
    let deser: CapabilityInfo = serde_json::from_str(&json).unwrap();
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
    let json = serde_json::to_string(&metrics).unwrap();
    let deser: RoutingMetrics = serde_json::from_str(&json).unwrap();
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
    let json = serde_json::to_string(&metric).unwrap();
    let deser: RoutingMetric = serde_json::from_str(&json).unwrap();
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
    let err = extract_rpc_error(&resp).unwrap();
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
    let err = extract_rpc_error(&resp).unwrap();
    assert!(err.is_internal());
}

#[test]
fn extract_rpc_error_server_range() {
    let resp = serde_json::json!({
        "error": { "code": -32050, "message": "custom server error" }
    });
    let err = extract_rpc_error(&resp).unwrap();
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
    let result = extract_rpc_result(&resp).unwrap();
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
    let result = extract_rpc_result(&resp).unwrap();
    assert!(result.is_null());
}
