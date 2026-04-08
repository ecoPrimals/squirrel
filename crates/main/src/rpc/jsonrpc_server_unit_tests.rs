// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use crate::api::ai::adapters::{AiProviderAdapter, QualityTier};
use crate::api::ai::router::AiRouter;
use crate::api::ai::types::{
    ImageGenerationRequest, ImageGenerationResponse, TextGenerationRequest, TextGenerationResponse,
};
use crate::error::PrimalError;
use async_trait::async_trait;
use serde_json::{Value, json};
use std::sync::Arc;

#[test]
fn normalize_method_strips_squirrel_prefix() {
    assert_eq!(
        super::normalize_method("squirrel.system.health"),
        "system.health"
    );
}

#[test]
fn normalize_method_strips_mcp_prefix() {
    assert_eq!(
        super::normalize_method("mcp.system.health"),
        "system.health"
    );
}

#[test]
fn normalize_method_leaves_unprefixed_methods() {
    assert_eq!(super::normalize_method("system.health"), "system.health");
    assert_eq!(super::normalize_method("identity.get"), "identity.get");
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

/// Minimal text adapter for JSON-RPC + router integration tests.
struct JsonRpcMockTextAdapter;

#[async_trait]
impl AiProviderAdapter for JsonRpcMockTextAdapter {
    fn provider_id(&self) -> &'static str {
        "jsonrpc-mock-text"
    }

    fn provider_name(&self) -> &'static str {
        "JsonRpcMock"
    }

    fn is_local(&self) -> bool {
        true
    }

    fn cost_per_unit(&self) -> Option<f64> {
        Some(0.0)
    }

    fn avg_latency_ms(&self) -> u64 {
        5
    }

    fn quality_tier(&self) -> QualityTier {
        QualityTier::Standard
    }

    fn supports_text_generation(&self) -> bool {
        true
    }

    fn supports_image_generation(&self) -> bool {
        false
    }

    async fn generate_text(
        &self,
        request: TextGenerationRequest,
    ) -> Result<TextGenerationResponse, PrimalError> {
        Ok(TextGenerationResponse {
            text: format!("echo:{}", request.prompt),
            provider_id: self.provider_id().to_string(),
            model: request.model.unwrap_or_else(|| "mock".to_string()),
            usage: None,
            cost_usd: None,
            latency_ms: 1,
        })
    }

    async fn generate_image(
        &self,
        _request: ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse, PrimalError> {
        Err(PrimalError::OperationFailed("no image".to_string()))
    }
}

#[tokio::test]
async fn ai_query_dispatches_to_router_and_returns_echo() {
    let router = Arc::new(AiRouter::from_adapters_for_test(vec![Arc::new(
        JsonRpcMockTextAdapter,
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
        Some("echo:ping")
    );
    assert_eq!(
        v.pointer("/result/success").and_then(Value::as_bool),
        Some(true)
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
