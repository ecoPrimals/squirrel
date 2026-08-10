// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Security orchestrator middleware tests and RPC dispatch edge-case regressions.

use super::*;
use serde_json::{Value, json};
use std::sync::Arc;

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
    let code = err.get("code").and_then(Value::as_i64).unwrap_or_default();
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
        .unwrap_or_default();
    assert_eq!(code, -32700, "malformed JSON → PARSE_ERROR");
}
