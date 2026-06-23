// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use crate::api::ai::adapters::AiProvider;
use crate::api::ai::adapters::test_mocks::JsonRpcMockTextAdapter;
use crate::api::ai::router::AiRouter;
use crate::niche::PRIMAL_ID;
use crate::rpc::jsonrpc_types::normalize_method;
use anyhow::Context;
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use universal_patterns::transport::UniversalTransport;

#[test]
fn normalize_method_strips_squirrel_prefix() {
    assert_eq!(normalize_method("squirrel.system.health"), "system.health");
}

#[test]
fn normalize_method_strips_mcp_prefix() {
    assert_eq!(normalize_method("mcp.system.health"), "system.health");
}

#[test]
fn normalize_method_leaves_unprefixed_methods() {
    assert_eq!(normalize_method("system.health"), "system.health");
    assert_eq!(normalize_method("identity.get"), "identity.get");
}

#[tokio::test]
async fn routing_squirrel_prefixed_system_health() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-norm-health.sock".to_string());
    let req = r#"{"jsonrpc":"2.0","method":"squirrel.system.health","id":1}"#;
    let raw = server
        .handle_request_or_batch(req)
        .await
        .expect("should succeed");
    let v: Value = serde_json::from_str(&raw).expect("should succeed");
    assert_eq!(
        v.pointer("/result/status").and_then(Value::as_str),
        Some("ready")
    );
    assert_eq!(
        v.pointer("/result/tier").and_then(Value::as_str),
        Some("ready")
    );
}

#[test]
fn test_jsonrpc_request_serialization() {
    let request = JsonRpcRequest {
        jsonrpc: Arc::from("2.0"),
        method: Arc::from("ai.query"),
        params: Some(json!({"prompt": "Hello"})),
        id: Some(json!(1)),
    };

    let json = serde_json::to_string(&request).expect("should succeed");
    let deserialized: JsonRpcRequest = serde_json::from_str(&json).expect("should succeed");

    assert_eq!(request.method, deserialized.method);
    assert_eq!(request.jsonrpc, deserialized.jsonrpc);
}

#[test]
fn test_jsonrpc_response_serialization() {
    let response = JsonRpcResponse {
        jsonrpc: Arc::from("2.0"),
        result: Some(json!({"status": "ok"})),
        error: None,
        id: json!(1),
    };

    let json = serde_json::to_string(&response).expect("should succeed");
    let deserialized: JsonRpcResponse = serde_json::from_str(&json).expect("should succeed");

    assert_eq!(response.jsonrpc, deserialized.jsonrpc);
    assert!(deserialized.result.is_some());
    assert!(deserialized.error.is_none());
}

#[test]
fn test_jsonrpc_error_serialization() {
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

    let json = serde_json::to_string(&response).expect("should succeed");
    let deserialized: JsonRpcResponse = serde_json::from_str(&json).expect("should succeed");

    assert!(deserialized.result.is_none());
    assert!(deserialized.error.is_some());
    assert_eq!(
        deserialized.error.expect("should succeed").code,
        error_codes::METHOD_NOT_FOUND
    );
}

#[test]
fn test_metrics_uptime() {
    let metrics = ServerMetrics::new();
    // uptime_seconds() returns u64, always >= 0
    let _ = metrics.uptime_seconds();
}

#[test]
fn test_metrics_avg_response_time() {
    let mut metrics = ServerMetrics::new();
    assert!(metrics.avg_response_time_ms().is_none());

    metrics.requests_handled = 10;
    metrics.total_response_time_ms = 1000;
    assert_eq!(metrics.avg_response_time_ms(), Some(100.0));
}

/// Exercise JSON-RPC method dispatch for each `handle_single_request` match arm.
#[tokio::test]
async fn routing_covers_ai_query_complete_chat_list_providers() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-routing.sock".to_string());
    for method in ["ai.query", "ai.complete", "ai.chat"] {
        let req =
            format!(r#"{{"jsonrpc":"2.0","method":"{method}","params":{{"prompt":"hi"}},"id":1}}"#);
        let raw = server
            .handle_request_or_batch(&req)
            .await
            .expect("should succeed");
        let v: Value = serde_json::from_str(&raw).expect("should succeed");
        assert!(
            v.get("error").is_some(),
            "{method} should error without AI router: {raw}"
        );
    }
    let req = r#"{"jsonrpc":"2.0","method":"ai.list_providers","id":2}"#;
    let raw = server
        .handle_request_or_batch(req)
        .await
        .expect("should succeed");
    let v: Value = serde_json::from_str(&raw).expect("should succeed");
    assert_eq!(v.pointer("/result/total").and_then(Value::as_u64), Some(0));

    let req = r#"{"jsonrpc":"2.0","method":"system.status","id":3}"#;
    let raw = server
        .handle_request_or_batch(req)
        .await
        .expect("should succeed");
    let v: Value = serde_json::from_str(&raw).expect("should succeed");
    assert_eq!(
        v.pointer("/result/status").and_then(Value::as_str),
        Some("healthy")
    );
}

#[tokio::test]
async fn routing_health_liveness_and_readiness() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-health.sock".to_string());
    for (method, key) in [("health.liveness", "alive"), ("health.readiness", "ready")] {
        let req = format!(r#"{{"jsonrpc":"2.0","method":"{method}","id":1}}"#);
        let raw = server
            .handle_request_or_batch(&req)
            .await
            .expect("should succeed");
        let v: Value = serde_json::from_str(&raw).expect("should succeed");
        assert!(
            v.pointer(&format!("/result/{key}")).is_some(),
            "{method}: {raw}"
        );
    }
}

#[tokio::test]
async fn routing_capabilities_list_aliases() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-cap.sock".to_string());
    for method in ["capabilities.list", "capability.list"] {
        let req = format!(r#"{{"jsonrpc":"2.0","method":"{method}","id":1}}"#);
        let raw = server
            .handle_request_or_batch(&req)
            .await
            .expect("should succeed");
        let v: Value = serde_json::from_str(&raw).expect("should succeed");
        assert!(
            v.pointer("/result/methods").is_some(),
            "Wire Standard: result.methods flat array required: {raw}"
        );
        let methods = v.pointer("/result/methods").expect("methods");
        assert!(methods.is_array(), "methods must be a flat string array");
        assert_eq!(
            v.pointer("/result/primal").and_then(Value::as_str),
            Some("squirrel"),
            "Wire Standard: result.primal required"
        );
        assert!(
            v.pointer("/result/version").is_some(),
            "Wire Standard: result.version required"
        );
    }
}

#[tokio::test]
async fn routing_capability_discover_and_announce() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-cap2.sock".to_string());
    let req = r#"{"jsonrpc":"2.0","method":"capability.discover","id":1}"#;
    let raw = server
        .handle_request_or_batch(req)
        .await
        .expect("should succeed");
    let v: Value = serde_json::from_str(&raw).expect("should succeed");
    assert!(v.pointer("/result/capabilities").is_some());

    let req = r#"{"jsonrpc":"2.0","method":"capability.announce","params":{"capabilities":["x"]},"id":2}"#;
    let raw = server
        .handle_request_or_batch(req)
        .await
        .expect("should succeed");
    let v: Value = serde_json::from_str(&raw).expect("should succeed");
    assert_eq!(
        v.pointer("/result/success").and_then(Value::as_bool),
        Some(true)
    );
}

#[tokio::test]
async fn routing_discovery_peers_lifecycle_graph() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-misc.sock".to_string());

    let req = r#"{"jsonrpc":"2.0","method":"discovery.peers","id":1}"#;
    let raw = server
        .handle_request_or_batch(req)
        .await
        .expect("should succeed");
    let v: Value = serde_json::from_str(&raw).expect("should succeed");
    assert!(v.pointer("/result/peers").is_some());

    for method in ["lifecycle.register", "lifecycle.status"] {
        let req = format!(r#"{{"jsonrpc":"2.0","method":"{method}","id":2}}"#);
        let raw = server
            .handle_request_or_batch(&req)
            .await
            .expect("should succeed");
        let v: Value = serde_json::from_str(&raw).expect("should succeed");
        assert!(v.get("result").is_some(), "{method}: {raw}");
    }

    let toml = r#"
[graph]
name = "t"
version = "1"

[[graph.node]]
name = "squirrel"
binary = "squirrel"
order = 1
"#;
    let parse_req = json!({
        "jsonrpc": "2.0",
        "method": "graph.parse",
        "params": { "graph_toml": toml },
        "id": 3
    })
    .to_string();
    let raw = server
        .handle_request_or_batch(&parse_req)
        .await
        .expect("should succeed");
    let v: Value = serde_json::from_str(&raw).expect("should succeed");
    assert!(v.pointer("/result/graph").is_some(), "{raw}");

    let validate_req = json!({
        "jsonrpc": "2.0",
        "method": "graph.validate",
        "params": { "graph_toml": toml },
        "id": 4
    })
    .to_string();
    let req = validate_req;
    let raw = server
        .handle_request_or_batch(&req)
        .await
        .expect("should succeed");
    let v: Value = serde_json::from_str(&raw).expect("should succeed");
    assert!(v.pointer("/result/valid").is_some(), "{raw}");
}

#[tokio::test]
async fn routing_graph_parse_invalid_returns_error_object() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-graph-bad.sock".to_string());
    let req =
        r#"{"jsonrpc":"2.0","method":"graph.parse","params":{"graph_toml":"not toml"},"id":1}"#;
    let raw = server
        .handle_request_or_batch(req)
        .await
        .expect("should succeed");
    let v: Value = serde_json::from_str(&raw).expect("should succeed");
    assert!(v.get("error").is_some());
    assert_eq!(
        v.pointer("/error/code")
            .and_then(Value::as_i64)
            .map(|c| c as i32),
        Some(error_codes::INVALID_PARAMS)
    );
}

#[tokio::test]
async fn single_request_parse_error_returns_invalid_json_shape() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-parse.sock".to_string());
    let req = r#"{"jsonrpc":"2.0","method":"system.ping","params":notjson,"id":1}"#;
    let raw = server
        .handle_request_or_batch(req)
        .await
        .expect("should succeed");
    let v: Value = serde_json::from_str(&raw).expect("should succeed");
    assert_eq!(
        v.pointer("/error/code")
            .and_then(Value::as_i64)
            .map(|c| c as i32),
        Some(error_codes::PARSE_ERROR)
    );
}

#[tokio::test]
async fn jsonrpc_request_round_trip_skips_none_id() {
    let r = JsonRpcRequest {
        jsonrpc: Arc::from("2.0"),
        method: Arc::from("system.ping"),
        params: None,
        id: None,
    };
    let s = serde_json::to_string(&r).expect("should succeed");
    assert!(!s.contains("id"));
    let back: JsonRpcRequest = serde_json::from_str(&s).expect("should succeed");
    assert_eq!(back.method.as_ref(), "system.ping");
}

#[tokio::test]
async fn routing_context_tool_and_system_metrics() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-ctx-tool.sock".to_string());

    let create = json!({
        "jsonrpc": "2.0",
        "method": "context.create",
        "params": { "session_id": "s-routing", "metadata": { "a": 1 } },
        "id": 1
    })
    .to_string();
    let raw = server
        .handle_request_or_batch(&create)
        .await
        .expect("should succeed");
    let v: Value = serde_json::from_str(&raw).expect("should succeed");
    let id = v
        .pointer("/result/id")
        .and_then(Value::as_str)
        .expect("context id");

    let update = json!({
        "jsonrpc": "2.0",
        "method": "context.update",
        "params": { "id": id, "data": { "b": 2 } },
        "id": 2
    })
    .to_string();
    let raw = server
        .handle_request_or_batch(&update)
        .await
        .expect("should succeed");
    let v: Value = serde_json::from_str(&raw).expect("should succeed");
    assert!(v.pointer("/result/version").is_some());

    let sum = json!({
        "jsonrpc": "2.0",
        "method": "context.summarize",
        "params": { "id": id },
        "id": 3
    })
    .to_string();
    let raw = server
        .handle_request_or_batch(&sum)
        .await
        .expect("should succeed");
    let v: Value = serde_json::from_str(&raw).expect("should succeed");
    assert!(v.pointer("/result/summary").is_some());

    let req = r#"{"jsonrpc":"2.0","method":"tool.list","id":4}"#;
    let raw = server
        .handle_request_or_batch(req)
        .await
        .expect("should succeed");
    let v: Value = serde_json::from_str(&raw).expect("should succeed");
    assert!(v.pointer("/result/tools").is_some());

    let exec = json!({
        "jsonrpc": "2.0",
        "method": "tool.execute",
        "params": { "tool": "system.health", "args": {} },
        "id": 5
    })
    .to_string();
    let raw = server
        .handle_request_or_batch(&exec)
        .await
        .expect("should succeed");
    let v: Value = serde_json::from_str(&raw).expect("should succeed");
    assert_eq!(
        v.pointer("/result/success").and_then(Value::as_bool),
        Some(true)
    );

    let req = r#"{"jsonrpc":"2.0","method":"system.metrics","id":6}"#;
    let raw = server
        .handle_request_or_batch(req)
        .await
        .expect("should succeed");
    let v: Value = serde_json::from_str(&raw).expect("should succeed");
    assert!(v.pointer("/result/requests_handled").is_some());
}

#[tokio::test]
async fn batch_request_mixed_methods_and_notifications() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-batch.sock".to_string());
    let batch = json!([
        {"jsonrpc":"2.0","method":"system.ping","id":1},
        {"jsonrpc":"2.0","method":"system.ping"},
        {"jsonrpc":"2.0","method":"system.health","id":2}
    ]);
    let raw = server
        .handle_request_or_batch(&batch.to_string())
        .await
        .expect("batch response");
    let arr: Vec<Value> = serde_json::from_str(&raw).expect("array");
    assert_eq!(arr.len(), 2);
    assert_eq!(
        arr[0].pointer("/result/pong").and_then(Value::as_bool),
        Some(true)
    );
}

#[tokio::test]
async fn batch_empty_returns_invalid_request() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-batch-empty.sock".to_string());
    let raw = server
        .handle_request_or_batch("[]")
        .await
        .expect("should succeed");
    let v: Value = serde_json::from_str(&raw).expect("should succeed");
    assert_eq!(
        v.pointer("/error/code")
            .and_then(Value::as_i64)
            .map(|c| c as i32),
        Some(error_codes::INVALID_REQUEST)
    );
}

#[tokio::test]
async fn notification_only_batch_returns_none() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-notify.sock".to_string());
    let batch = json!([
        {"jsonrpc":"2.0","method":"system.ping"},
        {"jsonrpc":"2.0","method":"system.ping"}
    ]);
    let out = server.handle_request_or_batch(&batch.to_string()).await;
    assert!(out.is_none());
}

#[tokio::test]
async fn top_level_parse_error_in_batch_path() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-top-parse.sock".to_string());
    let raw = server
        .handle_request_or_batch("not json")
        .await
        .expect("should succeed");
    let v: Value = serde_json::from_str(&raw).expect("should succeed");
    assert_eq!(
        v.pointer("/error/code")
            .and_then(Value::as_i64)
            .map(|c| c as i32),
        Some(error_codes::PARSE_ERROR)
    );
}

#[tokio::test]
async fn invalid_jsonrpc_version_rejected() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-ver.sock".to_string());
    let req = r#"{"jsonrpc":"1.0","method":"system.ping","id":1}"#;
    let raw = server
        .handle_request_or_batch(req)
        .await
        .expect("should succeed");
    let v: Value = serde_json::from_str(&raw).expect("should succeed");
    assert_eq!(
        v.pointer("/error/code")
            .and_then(Value::as_i64)
            .map(|c| c as i32),
        Some(error_codes::INVALID_REQUEST)
    );
}

#[tokio::test]
async fn unknown_method_not_found() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-unknown.sock".to_string());
    let req = r#"{"jsonrpc":"2.0","method":"no.such.method","id":1}"#;
    let raw = server
        .handle_request_or_batch(req)
        .await
        .expect("should succeed");
    let v: Value = serde_json::from_str(&raw).expect("should succeed");
    assert_eq!(
        v.pointer("/error/code")
            .and_then(Value::as_i64)
            .map(|c| c as i32),
        Some(error_codes::METHOD_NOT_FOUND)
    );
}

#[tokio::test]
async fn missing_method_invalid_request() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-missing-method.sock".to_string());
    let req = r#"{"jsonrpc":"2.0","id":1}"#;
    let raw = server
        .handle_request_or_batch(req)
        .await
        .expect("should succeed");
    let v: Value = serde_json::from_str(&raw).expect("should succeed");
    assert_eq!(
        v.pointer("/error/code")
            .and_then(Value::as_i64)
            .map(|c| c as i32),
        Some(error_codes::INVALID_REQUEST)
    );
}

#[tokio::test]
async fn empty_method_invalid_request() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-empty-method.sock".to_string());
    let req = r#"{"jsonrpc":"2.0","method":"","id":1}"#;
    let raw = server
        .handle_request_or_batch(req)
        .await
        .expect("should succeed");
    let v: Value = serde_json::from_str(&raw).expect("should succeed");
    assert_eq!(
        v.pointer("/error/code")
            .and_then(Value::as_i64)
            .map(|c| c as i32),
        Some(error_codes::INVALID_REQUEST)
    );
}

#[tokio::test]
async fn params_primitive_invalid_params() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-bad-params.sock".to_string());
    let req = r#"{"jsonrpc":"2.0","method":"system.ping","params":"nope","id":1}"#;
    let raw = server
        .handle_request_or_batch(req)
        .await
        .expect("should succeed");
    let v: Value = serde_json::from_str(&raw).expect("should succeed");
    assert_eq!(
        v.pointer("/error/code")
            .and_then(Value::as_i64)
            .map(|c| c as i32),
        Some(error_codes::INVALID_PARAMS)
    );
}

#[tokio::test]
async fn single_notification_returns_no_body() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-single-notify.sock".to_string());
    let req = r#"{"jsonrpc":"2.0","method":"system.ping"}"#;
    let out = server.handle_request_or_batch(req).await;
    assert!(out.is_none());
}

#[tokio::test]
async fn system_health_tier_becomes_healthy_after_prior_rpc() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-health-tier.sock".to_string());
    let ping = r#"{"jsonrpc":"2.0","method":"system.ping","id":1}"#;
    server.handle_request_or_batch(ping).await.expect("ping");
    let health = r#"{"jsonrpc":"2.0","method":"system.health","id":2}"#;
    let raw = server
        .handle_request_or_batch(health)
        .await
        .expect("health");
    let v: Value = serde_json::from_str(&raw).expect("should succeed");
    assert_eq!(
        v.pointer("/result/tier").and_then(Value::as_str),
        Some("healthy")
    );
    assert_eq!(
        v.pointer("/result/status").and_then(Value::as_str),
        Some("healthy")
    );
}

#[tokio::test]
async fn ai_query_dispatches_to_router_and_returns_echo() {
    let router = Arc::new(AiRouter::from_adapters_for_test(vec![Arc::new(
        AiProvider::JsonRpcMockText(JsonRpcMockTextAdapter),
    )]));
    let server = JsonRpcServer::with_ai_router("/tmp/jsonrpc-ai-ok.sock".to_string(), router);
    let req = r#"{"jsonrpc":"2.0","method":"ai.query","params":{"prompt":"ping"},"id":1}"#;
    let raw = server
        .handle_request_or_batch(req)
        .await
        .expect("should succeed");
    let v: Value = serde_json::from_str(&raw).expect("should succeed");
    assert_eq!(
        v.pointer("/result/response").and_then(Value::as_str),
        Some("echo:ping"),
        "ai.query routes through AiRouter and returns 'response' field"
    );
}

#[tokio::test]
async fn ai_query_without_router_returns_internal_error() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-ai-err.sock".to_string());
    let req = r#"{"jsonrpc":"2.0","method":"ai.query","params":{"prompt":"x"},"id":1}"#;
    let raw = server
        .handle_request_or_batch(req)
        .await
        .expect("should succeed");
    let v: Value = serde_json::from_str(&raw).expect("should succeed");
    assert_eq!(
        v.pointer("/error/code")
            .and_then(Value::as_i64)
            .map(|c| c as i32),
        Some(error_codes::INTERNAL_ERROR)
    );
}

#[tokio::test]
async fn handler_error_increments_metrics_errors() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-metrics.sock".to_string());
    let before = server.metrics.read().await.errors;
    let req = r#"{"jsonrpc":"2.0","method":"ai.query","params":{"prompt":"x"},"id":1}"#;
    server
        .handle_request_or_batch(req)
        .await
        .expect("should succeed");
    let after = server.metrics.read().await.errors;
    assert_eq!(after, before + 1);
}

#[test]
fn server_new_sets_socket_path_and_service_name() {
    let s = JsonRpcServer::new("/tmp/jsonrpc-cfg.sock".to_string());
    assert_eq!(s.socket_path, "/tmp/jsonrpc-cfg.sock");
    assert_eq!(s.service_name, PRIMAL_ID);
}

#[tokio::test]
async fn test_handle_jsonrpc_line_matches_handle_request_or_batch() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-alias.sock".to_string());
    let line = r#"{"jsonrpc":"2.0","method":"system.ping","id":1}"#;
    let a = server
        .test_handle_jsonrpc_line(line)
        .await
        .expect("hidden api");
    let b = server
        .handle_request_or_batch(line)
        .await
        .expect("batch path");
    let mut va: Value = serde_json::from_str(&a).expect("json a");
    let mut vb: Value = serde_json::from_str(&b).expect("json b");
    // system.ping embeds a fresh timestamp per call; compare everything else.
    if let Some(r) = va.pointer_mut("/result/timestamp") {
        *r = Value::Null;
    }
    if let Some(r) = vb.pointer_mut("/result/timestamp") {
        *r = Value::Null;
    }
    assert_eq!(va, vb);
}

#[tokio::test]
async fn single_request_scalar_not_object() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-scalar.sock".to_string());
    let raw = server
        .handle_request_or_batch("42")
        .await
        .expect("response");
    let v: Value = serde_json::from_str(&raw).expect("json");
    assert_eq!(
        v.pointer("/error/code")
            .and_then(Value::as_i64)
            .map(|c| c as i32),
        Some(error_codes::INVALID_REQUEST)
    );
}

#[tokio::test]
async fn method_field_not_string_invalid_request() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-method-type.sock".to_string());
    let raw = server
        .handle_request_or_batch(r#"{"jsonrpc":"2.0","method":99,"id":1}"#)
        .await
        .expect("response");
    let v: Value = serde_json::from_str(&raw).expect("json");
    assert_eq!(
        v.pointer("/error/code")
            .and_then(Value::as_i64)
            .map(|c| c as i32),
        Some(error_codes::INVALID_REQUEST)
    );
}

#[tokio::test]
async fn notification_wrong_jsonrpc_version_returns_no_response() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-notify-ver.sock".to_string());
    let out = server
        .handle_request_or_batch(r#"{"jsonrpc":"1.0","method":"system.ping"}"#)
        .await;
    assert!(out.is_none());
}

/// TCP loopback pair: server uses [`UniversalTransport::Tcp`], client uses raw [`tokio::net::TcpStream`].
async fn tcp_server_transport() -> (
    tokio::task::JoinHandle<anyhow::Result<()>>,
    tokio::net::TcpStream,
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    let server = Arc::new(JsonRpcServer::new("/tmp/jsonrpc-tcp.sock".to_string()));
    let jh = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.context("accept")?;
        server
            .handle_universal_connection(UniversalTransport::Tcp(stream))
            .await
    });
    let client = tokio::net::TcpStream::connect(addr).await.expect("connect");
    (jh, client)
}

#[tokio::test]
async fn universal_connection_eof_before_first_line_ok() {
    let (jh, client) = tcp_server_transport().await;
    drop(client);
    let res = jh.await.expect("join");
    assert!(res.is_ok(), "{res:?}");
}

#[tokio::test]
async fn universal_connection_jsonrpc_line_roundtrip() {
    let (jh, mut client) = tcp_server_transport().await;
    client
        .write_all(br#"{"jsonrpc":"2.0","method":"system.ping","id":1}"#)
        .await
        .expect("write");
    client.write_all(b"\n").await.expect("newline");
    client.flush().await.expect("flush");
    let mut reader = BufReader::new(&mut client);
    let mut line = String::new();
    reader.read_line(&mut line).await.expect("readline");
    let v: Value = serde_json::from_str(line.trim()).expect("json");
    assert_eq!(
        v.pointer("/result/pong").and_then(Value::as_bool),
        Some(true)
    );
    let client = reader.into_inner();
    client.shutdown().await.expect("shutdown");
    let _ = jh.await;
}

#[tokio::test]
async fn universal_connection_protocol_negotiation_jsonrpc_then_ping() {
    let (jh, mut client) = tcp_server_transport().await;
    client
        .write_all(b"PROTOCOLS: jsonrpc\n")
        .await
        .expect("protocols");
    client
        .write_all(br#"{"jsonrpc":"2.0","method":"system.ping","id":1}"#)
        .await
        .expect("rpc");
    client.write_all(b"\n").await.expect("newline");
    client.flush().await.expect("flush");
    let mut reader = BufReader::new(&mut client);
    let mut line = String::new();
    reader.read_line(&mut line).await.expect("proto line");
    assert!(
        line.starts_with("PROTOCOL:"),
        "expected PROTOCOL response, got {line:?}"
    );
    line.clear();
    reader.read_line(&mut line).await.expect("json line");
    let v: Value = serde_json::from_str(line.trim()).expect("json");
    assert_eq!(
        v.pointer("/result/pong").and_then(Value::as_bool),
        Some(true)
    );
    let client = reader.into_inner();
    client.shutdown().await.expect("shutdown");
    let _ = jh.await;
}

#[tokio::test]
async fn universal_connection_invalid_protocol_request_falls_back_to_jsonrpc() {
    let (jh, mut client) = tcp_server_transport().await;
    client
        .write_all(b"PROTOCOLS: not-a-real-protocol-list\n")
        .await
        .expect("bad proto");
    client
        .write_all(br#"{"jsonrpc":"2.0","method":"system.ping","id":1}"#)
        .await
        .expect("rpc");
    client.write_all(b"\n").await.expect("newline");
    client.flush().await.expect("flush");
    let mut reader = BufReader::new(&mut client);
    let mut line = String::new();
    reader.read_line(&mut line).await.expect("fallback proto");
    assert!(line.starts_with("PROTOCOL:"));
    line.clear();
    reader.read_line(&mut line).await.expect("json");
    let v: Value = serde_json::from_str(line.trim()).expect("json");
    assert_eq!(
        v.pointer("/result/pong").and_then(Value::as_bool),
        Some(true)
    );
    let client = reader.into_inner();
    client.shutdown().await.expect("shutdown");
    let _ = jh.await;
}

// -----------------------------------------------------------------------
// riboCipher signal acceptance (UDS) — Eukaryotic genetics model
// -----------------------------------------------------------------------

/// UDS loopback pair routed through `handle_uds_connection` (riboCipher + BTSP auto-detect).
async fn uds_server_transport() -> (
    tokio::task::JoinHandle<anyhow::Result<()>>,
    tokio::net::UnixStream,
) {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("ribo.sock");
    let listener = tokio::net::UnixListener::bind(&sock).expect("bind");
    let server = Arc::new(JsonRpcServer::new(sock.to_str().expect("utf8").to_string()));
    let jh = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.context("accept")?;
        let transport = UniversalTransport::UnixSocket(stream);
        JsonRpcServer::handle_uds_connection(server, transport).await
    });
    let client = tokio::net::UnixStream::connect(&sock)
        .await
        .expect("connect");
    std::mem::forget(dir);
    (jh, client)
}

#[tokio::test]
async fn uds_clear_signal_ndjson_health_roundtrip() {
    let (jh, mut client) = uds_server_transport().await;
    client.write_all(&[0xEC, 0x01]).await.expect("preamble");
    client
        .write_all(br#"{"jsonrpc":"2.0","method":"health","id":1}"#)
        .await
        .expect("rpc");
    client.write_all(b"\n").await.expect("newline");
    client.flush().await.expect("flush");
    client.shutdown().await.expect("shutdown");

    let mut buf = Vec::new();
    client.read_to_end(&mut buf).await.expect("read");
    let v: Value = serde_json::from_slice(&buf).expect("json");
    assert_eq!(
        v.pointer("/result/status").and_then(Value::as_str),
        Some("healthy")
    );
    let _ = jh.await;
}

#[tokio::test]
async fn uds_mito_signal_ndjson_health_roundtrip() {
    let (jh, mut client) = uds_server_transport().await;
    client.write_all(&[0xED, 0x01]).await.expect("preamble");
    client
        .write_all(br#"{"jsonrpc":"2.0","method":"health","id":1}"#)
        .await
        .expect("rpc");
    client.write_all(b"\n").await.expect("newline");
    client.flush().await.expect("flush");
    client.shutdown().await.expect("shutdown");

    let mut buf = Vec::new();
    client.read_to_end(&mut buf).await.expect("read");
    let v: Value = serde_json::from_slice(&buf).expect("json");
    assert_eq!(
        v.pointer("/result/status").and_then(Value::as_str),
        Some("healthy"),
        "mito-beacon (0xED) should be accepted identically to clear (0xEC): {v}"
    );
    let _ = jh.await;
}

#[tokio::test]
async fn uds_nuclear_signal_returns_json_error_for_ndjson() {
    let (jh, mut client) = uds_server_transport().await;
    // Nuclear signal + NDJSON protocol type → structured JSON-RPC error
    client
        .write_all(&[0xEE, 0x01])
        .await
        .expect("nuclear+ndjson");
    client.flush().await.expect("flush");

    let mut buf = Vec::new();
    let _ = client.read_to_end(&mut buf).await;
    assert!(
        !buf.is_empty(),
        "nuclear+ndjson should return a JSON-RPC error"
    );
    let v: Value = serde_json::from_slice(&buf).expect("valid json");
    assert_eq!(
        v.pointer("/error/code").and_then(Value::as_i64),
        Some(-32050),
        "error code must be -32050"
    );
    assert_eq!(
        v.pointer("/error/data/resolution").and_then(Value::as_str),
        Some("awaiting_beardog_keys"),
        "resolution field must guide client"
    );
    assert_eq!(
        v.pointer("/error/data/tier").and_then(Value::as_str),
        Some("nuclear"),
        "tier must identify nuclear lineage"
    );
    let res = jh.await.expect("join");
    assert!(res.is_ok(), "server should not error: {res:?}");
}

#[tokio::test]
async fn uds_nuclear_signal_btsp_protocol_closes_silently() {
    let (jh, mut client) = uds_server_transport().await;
    // Nuclear signal + BTSP protocol type → no JSON response, clean close
    client.write_all(&[0xEE, 0x02]).await.expect("nuclear+btsp");
    client.flush().await.expect("flush");

    let mut buf = Vec::new();
    let _ = client.read_to_end(&mut buf).await;
    assert!(
        buf.is_empty(),
        "nuclear+btsp should close silently (no NDJSON response)"
    );
    let res = jh.await.expect("join");
    assert!(res.is_ok(), "server should not error: {res:?}");
}

#[tokio::test]
async fn uds_raw_json_still_works_without_prefix() {
    let (jh, mut client) = uds_server_transport().await;
    client
        .write_all(br#"{"jsonrpc":"2.0","method":"system.ping","id":1}"#)
        .await
        .expect("rpc");
    client.write_all(b"\n").await.expect("newline");
    client.flush().await.expect("flush");
    client.shutdown().await.expect("shutdown");

    let mut buf = Vec::new();
    client.read_to_end(&mut buf).await.expect("read");
    let v: Value = serde_json::from_slice(&buf).expect("json");
    assert_eq!(
        v.pointer("/result/pong").and_then(Value::as_bool),
        Some(true),
        "raw JSON without riboCipher prefix should still work: {v}"
    );
    let _ = jh.await;
}

// ── Security orchestrator middleware tests ────────────────────────────────────

#[tokio::test]
async fn security_middleware_allows_normal_requests() {
    use crate::security::orchestrator::{SecurityOrchestrationConfig, SecurityOrchestrator};

    let orchestrator = SecurityOrchestrator::new(SecurityOrchestrationConfig::default())
        .await
        .expect("orchestrator");
    let server = JsonRpcServer::new("/tmp/jsonrpc-sec-allow.sock".to_string())
        .with_security_orchestrator(Arc::new(orchestrator));

    let req = r#"{"jsonrpc":"2.0","method":"system.health","id":1}"#;
    let raw = server
        .test_handle_jsonrpc_line(req)
        .await
        .expect("response");
    let v: Value = serde_json::from_str(&raw).expect("json");
    assert!(
        v.get("error").is_none(),
        "healthy request should pass security: {v}"
    );
    assert_eq!(
        v.pointer("/result/status").and_then(Value::as_str),
        Some("ready")
    );
}

#[tokio::test]
async fn security_middleware_maps_endpoint_types_correctly() {
    assert_eq!(
        JsonRpcServer::endpoint_type_for_method("health.check"),
        crate::security::rate_limiter::types::EndpointType::HealthCheck,
    );
    assert_eq!(
        JsonRpcServer::endpoint_type_for_method("ai.query"),
        crate::security::rate_limiter::types::EndpointType::Compute,
    );
    assert_eq!(
        JsonRpcServer::endpoint_type_for_method("inference.complete"),
        crate::security::rate_limiter::types::EndpointType::Compute,
    );
    assert_eq!(
        JsonRpcServer::endpoint_type_for_method("btsp.handshake"),
        crate::security::rate_limiter::types::EndpointType::Authentication,
    );
    assert_eq!(
        JsonRpcServer::endpoint_type_for_method("deploy.start"),
        crate::security::rate_limiter::types::EndpointType::Admin,
    );
    assert_eq!(
        JsonRpcServer::endpoint_type_for_method("context.create"),
        crate::security::rate_limiter::types::EndpointType::Api,
    );
}

#[tokio::test]
async fn security_middleware_extracts_text_inputs() {
    let params = json!({"prompt": "hello world", "model": "local"});
    let inputs = JsonRpcServer::extract_input_data(Some(&params));
    assert!(inputs.is_some(), "should extract prompt");
    let inputs = inputs.unwrap();
    assert_eq!(inputs.len(), 1);
    assert_eq!(inputs[0].0, "prompt");
    assert_eq!(inputs[0].1, "hello world");
}

#[tokio::test]
async fn security_middleware_no_inputs_for_empty_params() {
    let params = json!({"count": 5});
    let inputs = JsonRpcServer::extract_input_data(Some(&params));
    assert!(inputs.is_none(), "no text fields should yield None");
}

#[tokio::test]
async fn security_middleware_absent_when_not_configured() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-sec-none.sock".to_string());
    assert!(
        server.security_orchestrator.is_none(),
        "default server should have no security orchestrator"
    );
    let req = r#"{"jsonrpc":"2.0","method":"system.health","id":1}"#;
    let raw = server
        .test_handle_jsonrpc_line(req)
        .await
        .expect("response");
    let v: Value = serde_json::from_str(&raw).expect("json");
    assert!(v.get("error").is_none(), "should pass without orchestrator");
}

// ── Security middleware depth tests ─────────────────────────────────────────

#[tokio::test]
async fn security_middleware_extracts_multiple_text_fields() {
    let params = json!({
        "prompt": "generate code",
        "system_message": "you are helpful",
        "model": "local",
        "temperature": 0.7
    });
    let inputs = JsonRpcServer::extract_input_data(Some(&params));
    assert!(inputs.is_some());
    let inputs = inputs.unwrap();
    let field_names: Vec<&str> = inputs.iter().map(|(k, _, _)| k.as_str()).collect();
    assert!(field_names.contains(&"prompt"), "should extract prompt");
    assert!(
        field_names.contains(&"system_message"),
        "should extract system_message"
    );
    assert!(!field_names.contains(&"model"), "non-text fields excluded");
}

#[tokio::test]
async fn security_middleware_handles_null_params() {
    let inputs = JsonRpcServer::extract_input_data(None);
    assert!(inputs.is_none(), "null params → no inputs");
}

#[tokio::test]
async fn security_middleware_handles_non_object_params() {
    let params = json!([1, 2, 3]);
    let inputs = JsonRpcServer::extract_input_data(Some(&params));
    assert!(inputs.is_none(), "array params → no inputs");
}

#[tokio::test]
async fn security_middleware_endpoint_type_unknown_prefix_defaults_to_api() {
    assert_eq!(
        JsonRpcServer::endpoint_type_for_method("custom.operation"),
        crate::security::rate_limiter::types::EndpointType::Api,
    );
    assert_eq!(
        JsonRpcServer::endpoint_type_for_method(""),
        crate::security::rate_limiter::types::EndpointType::Api,
    );
}

#[tokio::test]
async fn security_middleware_allows_concurrent_health_checks() {
    use crate::security::orchestrator::{SecurityOrchestrationConfig, SecurityOrchestrator};

    let orchestrator = SecurityOrchestrator::new(SecurityOrchestrationConfig::default())
        .await
        .expect("orchestrator");
    let server = Arc::new(
        JsonRpcServer::new("/tmp/jsonrpc-sec-concurrent.sock".to_string())
            .with_security_orchestrator(Arc::new(orchestrator)),
    );

    let mut handles = Vec::new();
    for i in 0..10 {
        let srv = Arc::clone(&server);
        handles.push(tokio::spawn(async move {
            let req = format!(r#"{{"jsonrpc":"2.0","method":"system.health","id":{i}}}"#);
            let raw = srv.test_handle_jsonrpc_line(&req).await.expect("response");
            let v: Value = serde_json::from_str(&raw).expect("json");
            assert!(
                v.get("error").is_none(),
                "concurrent health check {i} should pass"
            );
        }));
    }
    for h in handles {
        h.await.expect("join");
    }
}

// ── RPC dispatch edge case tests ────────────────────────────────────────────

#[tokio::test]
async fn rpc_handles_batch_requests_as_error() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-batch.sock".to_string());
    let req = r#"[{"jsonrpc":"2.0","method":"system.health","id":1},{"jsonrpc":"2.0","method":"system.health","id":2}]"#;
    let raw = server
        .test_handle_jsonrpc_line(req)
        .await
        .expect("response");
    let v: Value = serde_json::from_str(&raw).expect("json");
    assert!(
        v.get("error").is_some() || v.is_array(),
        "batch should either error or return array: {v}"
    );
}

#[tokio::test]
async fn rpc_handles_missing_method_field() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-nomethod.sock".to_string());
    let req = r#"{"jsonrpc":"2.0","id":1}"#;
    let raw = server
        .test_handle_jsonrpc_line(req)
        .await
        .expect("response");
    let v: Value = serde_json::from_str(&raw).expect("json");
    assert!(v.get("error").is_some(), "missing method → error: {v}");
}

#[tokio::test]
async fn rpc_handles_unknown_method_gracefully() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-unknown.sock".to_string());
    let req = r#"{"jsonrpc":"2.0","method":"nonexistent.method","id":1}"#;
    let raw = server
        .test_handle_jsonrpc_line(req)
        .await
        .expect("response");
    let v: Value = serde_json::from_str(&raw).expect("json");
    let err = v.get("error").expect("should be error");
    let code = err.get("code").and_then(Value::as_i64).unwrap_or(0);
    assert_eq!(code, -32601, "unknown method → METHOD_NOT_FOUND");
}

#[tokio::test]
async fn rpc_preserves_request_id_on_error() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-idpreserve.sock".to_string());
    let req = r#"{"jsonrpc":"2.0","method":"nonexistent.method","id":42}"#;
    let raw = server
        .test_handle_jsonrpc_line(req)
        .await
        .expect("response");
    let v: Value = serde_json::from_str(&raw).expect("json");
    assert_eq!(
        v.get("id").and_then(Value::as_i64),
        Some(42),
        "id preserved"
    );
}

#[tokio::test]
async fn rpc_handles_string_id() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-strid.sock".to_string());
    let req = r#"{"jsonrpc":"2.0","method":"system.health","id":"abc-123"}"#;
    let raw = server
        .test_handle_jsonrpc_line(req)
        .await
        .expect("response");
    let v: Value = serde_json::from_str(&raw).expect("json");
    assert_eq!(
        v.get("id").and_then(Value::as_str),
        Some("abc-123"),
        "string id preserved"
    );
}

#[tokio::test]
async fn rpc_handles_notification_without_id() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-notify.sock".to_string());
    let req = r#"{"jsonrpc":"2.0","method":"system.health"}"#;
    let result = server.test_handle_jsonrpc_line(req).await;
    // Notifications may return None (no response) or a response without id
    if let Some(raw) = result {
        let v: Value = serde_json::from_str(&raw).expect("json");
        assert!(
            v.get("id").is_none() || v["id"].is_null(),
            "notification response should have null/absent id: {v}"
        );
    }
}

#[tokio::test]
async fn rpc_handles_empty_string_input() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-empty.sock".to_string());
    let result = server.test_handle_jsonrpc_line("").await;
    assert!(result.is_some(), "empty input should return parse error");
    let v: Value = serde_json::from_str(&result.unwrap()).expect("json");
    assert!(v.get("error").is_some(), "empty input → parse error");
}

#[tokio::test]
async fn rpc_handles_malformed_json() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-malformed.sock".to_string());
    let raw = server
        .test_handle_jsonrpc_line("{not json")
        .await
        .expect("response");
    let v: Value = serde_json::from_str(&raw).expect("json");
    let code = v
        .pointer("/error/code")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    assert_eq!(code, -32700, "malformed JSON → PARSE_ERROR");
}
