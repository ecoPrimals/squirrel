// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use serde_json::json;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn make_server() -> JsonRpcServer {
    JsonRpcServer::new("/tmp/test.sock".to_string())
}

#[derive(Debug, serde::Deserialize)]
struct TestParams {
    name: String,
    count: u32,
}

#[tokio::test]
async fn test_parse_params_valid() -> TestResult {
    let server = make_server();
    let params = Some(json!({"name": "test", "count": 42}));
    let result: TestParams = server.parse_params(params)?;
    assert_eq!(result.name, "test");
    assert_eq!(result.count, 42);
    Ok(())
}

#[tokio::test]
async fn test_parse_params_missing() {
    let server = make_server();
    let result: Result<TestParams, _> = server.parse_params(None);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, error_codes::INVALID_PARAMS);
    assert!(err.message.contains("Missing parameters"));
}

#[tokio::test]
async fn test_parse_params_invalid_type() {
    let server = make_server();
    let params = Some(json!({"name": "test", "count": "not-a-number"}));
    let result: Result<TestParams, _> = server.parse_params(params);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, error_codes::INVALID_PARAMS);
}

#[tokio::test]
async fn test_parse_params_wrong_structure() {
    let server = make_server();
    let params = Some(json!({"wrong": "structure"}));
    let result: Result<TestParams, _> = server.parse_params(params);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_method_not_found() {
    let server = make_server();
    let err = server.method_not_found("nonexistent.method");
    assert_eq!(err.code, error_codes::METHOD_NOT_FOUND);
    assert!(err.message.contains("nonexistent.method"));
}

#[tokio::test]
async fn test_error_response() -> TestResult {
    let server = make_server();
    let response = server.error_response(json!(1), -32000, "Custom error");
    assert_eq!(response.jsonrpc.as_ref(), "2.0");
    assert!(response.result.is_none());
    assert!(response.error.is_some());
    let err = response
        .error
        .expect("error_response should set error when result is none");
    assert_eq!(err.code, -32000);
    assert_eq!(err.message, "Custom error");
    assert_eq!(response.id, json!(1));
    Ok(())
}

#[tokio::test]
async fn test_handle_identity_get() -> TestResult {
    use serde_json::Value;
    use universal_constants::identity;

    let server = make_server();
    let result: Value = server.handle_identity_get().await?;
    assert_eq!(
        result.get("primal_id").and_then(|v| v.as_str()),
        Some(identity::PRIMAL_ID)
    );
    assert_eq!(
        result.get("domain").and_then(|v| v.as_str()),
        Some(identity::PRIMAL_DOMAIN)
    );
    assert_eq!(
        result.get("transport").and_then(|v| v.as_str()),
        Some("unix-socket")
    );
    assert_eq!(
        result.get("protocol").and_then(|v| v.as_str()),
        Some("json-rpc-2.0")
    );
    assert_eq!(
        result.get("license").and_then(|v| v.as_str()),
        Some("AGPL-3.0-or-later")
    );
    assert_eq!(
        result.get("jwt_issuer").and_then(|v| v.as_str()),
        Some(identity::JWT_ISSUER)
    );
    assert_eq!(
        result.get("jwt_audience").and_then(|v| v.as_str()),
        Some(identity::JWT_AUDIENCE)
    );
    assert_eq!(
        result.get("version").and_then(|v| v.as_str()),
        Some(env!("CARGO_PKG_VERSION"))
    );
    Ok(())
}

#[tokio::test]
async fn test_handle_health() -> TestResult {
    let server = make_server();
    let result = server.handle_health().await?;
    assert!(result.get("tier").is_some());
    assert!(result.get("alive").and_then(Value::as_bool) == Some(true));
    assert!(result.get("status").and_then(Value::as_str) == Some("ready"));
    assert!(result.get("version").is_some());
    assert!(result.get("uptime_seconds").is_some());
    Ok(())
}

#[tokio::test]
async fn test_handle_metrics() -> TestResult {
    let server = make_server();
    let result = server.handle_metrics().await?;
    assert!(result.get("requests_handled").is_some());
    assert!(result.get("errors").is_some());
    assert!(result.get("uptime_seconds").is_some());
    assert!(result.get("success_rate").is_some());
    Ok(())
}

#[tokio::test]
async fn test_handle_ping() -> TestResult {
    let server = make_server();
    let result = server.handle_ping().await?;
    assert_eq!(
        result.get("pong").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert!(result.get("timestamp").is_some());
    Ok(())
}

#[tokio::test]
async fn test_handle_discover_capabilities() -> TestResult {
    let server = make_server();
    let result = server.handle_discover_capabilities().await?;
    assert_eq!(
        result.get("primal").and_then(|v| v.as_str()),
        Some("squirrel")
    );
    let caps = result
        .get("capabilities")
        .and_then(|v| v.as_array())
        .expect("test: capabilities must be array");
    assert!(caps.iter().any(|c| c.as_str() == Some("ai.query")));
    assert!(caps.iter().any(|c| c.as_str() == Some("system.health")));

    assert!(
        result.get("cost_estimates").is_some(),
        "response must include cost_estimates"
    );
    assert!(
        result.get("operation_dependencies").is_some(),
        "response must include operation_dependencies"
    );
    let consumed = result
        .get("consumed_capabilities")
        .and_then(|v| v.as_array())
        .expect("test: consumed_capabilities must be array");
    assert!(
        consumed
            .iter()
            .any(|c| c.as_str() == Some("discovery.register")),
        "consumed_capabilities must include discovery.register"
    );
    Ok(())
}

#[tokio::test]
async fn test_handle_capability_list() -> TestResult {
    let server = make_server();
    let result = server.handle_capability_list().await?;
    assert_eq!(
        result.get("primal").and_then(|v| v.as_str()),
        Some("squirrel")
    );
    let methods = result
        .get("methods")
        .and_then(|v| v.as_object())
        .expect("test: methods must be object");
    assert!(methods.contains_key("ai.query"));
    assert!(methods.contains_key("capability.list"));

    let ai_query = methods
        .get("ai.query")
        .expect("ai.query method metadata")
        .as_object()
        .expect("ai.query should be object");
    assert!(ai_query.contains_key("cost"));
    assert!(ai_query.contains_key("depends_on"));
    Ok(())
}

#[tokio::test]
async fn test_handle_announce_capabilities_valid() -> TestResult {
    let server = make_server();
    let params = Some(json!({"capabilities": ["ai.inference", "tool.execute"]}));
    let result = server.handle_announce_capabilities(params).await?;
    assert_eq!(
        result.get("success").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert!(
        result
            .get("message")
            .expect("announce_capabilities message")
            .as_str()
            .expect("message should be string")
            .contains('2')
    );
    Ok(())
}

#[tokio::test]
async fn test_handle_announce_capabilities_missing_params() {
    let server = make_server();
    let result = server.handle_announce_capabilities(None).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, error_codes::INVALID_PARAMS);
}

#[tokio::test]
async fn test_handle_list_providers_no_router() -> TestResult {
    let server = make_server();
    let result = server.handle_list_providers(None).await?;
    assert_eq!(
        result.get("total").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert!(
        result
            .get("providers")
            .and_then(|v| v.as_array())
            .expect("providers array")
            .is_empty()
    );
    Ok(())
}

#[tokio::test]
async fn test_handle_query_ai_no_params() {
    let server = make_server();
    let result = server.handle_query_ai(None).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, error_codes::INVALID_PARAMS);
}

#[tokio::test]
async fn test_handle_query_ai_no_router() {
    let server = make_server();
    let params = Some(json!({"prompt": "Hello"}));
    let result = server.handle_query_ai(params).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .message
            .contains("AI router not configured")
    );
}

#[tokio::test]
async fn test_handle_batch_empty() -> TestResult {
    let server = make_server();
    let result = server.handle_request_or_batch("[]").await;
    assert!(result.is_some());
    let body = result.expect("empty batch should yield response");
    let parsed: serde_json::Value = serde_json::from_str(&body)?;
    assert!(parsed.get("error").is_some());
    Ok(())
}

#[tokio::test]
async fn test_handle_batch_single() -> TestResult {
    let server = make_server();
    let batch = r#"[{"jsonrpc":"2.0","method":"system.ping","id":1}]"#;
    let result = server.handle_request_or_batch(batch).await;
    assert!(result.is_some());
    let body = result.expect("single batch response");
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&body)?;
    assert_eq!(parsed.len(), 1);
    assert!(parsed[0].get("result").is_some());
    Ok(())
}

#[tokio::test]
async fn test_handle_batch_multi() -> TestResult {
    let server = make_server();
    let batch = r#"[
        {"jsonrpc":"2.0","method":"system.ping","id":1},
        {"jsonrpc":"2.0","method":"system.health","id":2}
    ]"#;
    let result = server.handle_request_or_batch(batch).await;
    assert!(result.is_some());
    let body = result.expect("multi batch response");
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&body)?;
    assert_eq!(parsed.len(), 2);
    Ok(())
}

#[tokio::test]
async fn test_handle_lifecycle_register() -> TestResult {
    let server = make_server();
    let result = server.handle_lifecycle_register().await?;
    assert_eq!(
        result.get("success").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert!(
        result
            .get("message")
            .and_then(|v| v.as_str())
            .expect("lifecycle message")
            .contains("squirrel")
    );
    assert!(result.get("version").is_some());
    Ok(())
}

#[tokio::test]
async fn test_handle_lifecycle_status() -> TestResult {
    let server = make_server();
    let result = server.handle_lifecycle_status().await?;
    assert_eq!(
        result.get("status").and_then(|v| v.as_str()),
        Some("healthy")
    );
    assert!(result.get("version").is_some());
    assert!(result.get("uptime_seconds").is_some());
    assert_eq!(
        result.get("service").and_then(|v| v.as_str()),
        Some("squirrel")
    );
    Ok(())
}

#[tokio::test]
async fn test_handle_discover_peers() -> TestResult {
    let server = make_server();
    let result = server.handle_discover_peers(None).await?;
    assert!(result.get("peers").and_then(|v| v.as_array()).is_some());
    assert!(
        result
            .get("total")
            .and_then(serde_json::Value::as_u64)
            .is_some()
    );
    assert_eq!(
        result.get("discovery_method").and_then(|v| v.as_str()),
        Some("socket_scan")
    );
    Ok(())
}

#[tokio::test]
async fn test_handle_single_request_ping() -> TestResult {
    let server = make_server();
    let req = r#"{"jsonrpc":"2.0","method":"system.ping","id":42}"#;
    let result = server.handle_request_or_batch(req).await;
    assert!(result.is_some());
    let body = result.expect("ping response");
    let parsed: serde_json::Value = serde_json::from_str(&body)?;
    assert_eq!(
        parsed
            .get("result")
            .and_then(|r| r.get("pong"))
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        parsed.get("id").and_then(serde_json::Value::as_i64),
        Some(42)
    );
    Ok(())
}

#[tokio::test]
async fn test_handle_parse_error() -> TestResult {
    let server = make_server();
    let result = server.handle_request_or_batch("not valid json {{{").await;
    assert!(result.is_some());
    let body = result.expect("parse error response");
    let parsed: serde_json::Value = serde_json::from_str(&body)?;
    assert!(parsed.get("error").is_some());
    assert_eq!(
        parsed
            .get("error")
            .and_then(|e| e.get("code"))
            .and_then(serde_json::Value::as_i64),
        Some(i64::from(error_codes::PARSE_ERROR))
    );
    Ok(())
}

#[tokio::test]
async fn test_handle_invalid_jsonrpc_version() -> TestResult {
    let server = make_server();
    let req = r#"{"jsonrpc":"1.0","method":"system.ping","id":1}"#;
    let result = server.handle_request_or_batch(req).await;
    assert!(result.is_some());
    let body = result.expect("invalid jsonrpc version response");
    let parsed: serde_json::Value = serde_json::from_str(&body)?;
    assert!(parsed.get("error").is_some());
    assert!(
        parsed
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("")
            .contains("2.0")
    );
    Ok(())
}

#[tokio::test]
async fn test_handle_announce_with_primal_and_socket() -> TestResult {
    let server = make_server();
    let params = Some(json!({
        "capabilities": ["tool.remote"],
        "primal": "songbird-1",
        "socket_path": "/tmp/songbird.sock",
        "tools": ["tool.remote"]
    }));
    let result = server.handle_announce_capabilities(params).await?;
    assert_eq!(
        result.get("success").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        result
            .get("tools_registered")
            .and_then(serde_json::Value::as_u64),
        Some(1)
    );
    Ok(())
}

#[tokio::test]
async fn test_handle_list_tools() -> TestResult {
    let server = make_server();
    let result = server.handle_list_tools().await?;
    assert!(result.get("tools").and_then(|v| v.as_array()).is_some());
    assert!(
        result
            .get("total")
            .and_then(serde_json::Value::as_u64)
            .is_some()
    );
    Ok(())
}

#[tokio::test]
async fn test_handle_context_create() -> TestResult {
    let server = make_server();
    let params = Some(json!({"session_id": "test-session-123", "metadata": {"key": "value"}}));
    let result = server.handle_context_create(params).await?;
    assert!(result.get("id").is_some());
    assert!(result.get("version").is_some());
    assert!(result.get("created_at").is_some());
    assert_eq!(
        result
            .get("metadata")
            .and_then(|m| m.get("key"))
            .and_then(|v| v.as_str()),
        Some("value")
    );
    Ok(())
}

#[tokio::test]
async fn test_handle_context_create_auto_session_id() -> TestResult {
    let server = make_server();
    let result = server.handle_context_create(None).await?;
    assert!(
        result
            .get("id")
            .and_then(|v| v.as_str())
            .is_some_and(|s| !s.is_empty())
    );
    Ok(())
}

#[tokio::test]
async fn test_handle_context_update_missing_params() {
    let server = make_server();
    let result = server.handle_context_update(None).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, error_codes::INVALID_PARAMS);
}

#[tokio::test]
async fn test_handle_context_summarize_missing_params() {
    let server = make_server();
    let result = server.handle_context_summarize(None).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, error_codes::INVALID_PARAMS);
}

#[tokio::test]
async fn test_handle_execute_tool_missing_params() {
    let server = make_server();
    let result = server.handle_execute_tool(None).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, error_codes::INVALID_PARAMS);
}

#[tokio::test]
async fn test_handle_execute_tool_missing_tool_param() {
    let server = make_server();
    let params = Some(json!({"args": {}}));
    let result = server.handle_execute_tool(params).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, error_codes::INVALID_PARAMS);
}

#[tokio::test]
async fn test_handle_request_context_create() -> TestResult {
    let server = make_server();
    let req =
        r#"{"jsonrpc":"2.0","method":"context.create","params":{"session_id":"req-test"},"id":1}"#;
    let result = server.handle_request_or_batch(req).await;
    assert!(result.is_some());
    let body = result.expect("context.create response");
    let parsed: serde_json::Value = serde_json::from_str(&body)?;
    assert!(parsed.get("result").is_some());
    assert!(parsed.get("result").and_then(|r| r.get("id")).is_some());
    Ok(())
}

#[tokio::test]
async fn test_handle_request_tool_list() -> TestResult {
    let server = make_server();
    let req = r#"{"jsonrpc":"2.0","method":"tool.list","id":1}"#;
    let result = server.handle_request_or_batch(req).await;
    assert!(result.is_some());
    let body = result.expect("tool.list response");
    let parsed: serde_json::Value = serde_json::from_str(&body)?;
    assert!(parsed.get("result").is_some());
    assert!(
        parsed
            .get("result")
            .and_then(|r| r.get("tools"))
            .and_then(|v| v.as_array())
            .is_some()
    );
    Ok(())
}

#[tokio::test]
async fn test_handle_context_update_valid() -> TestResult {
    let server = make_server();
    let create_params =
        Some(json!({"session_id": "update-test-session", "metadata": {"key": "v1"}}));
    let create_result = server.handle_context_create(create_params).await?;
    let ctx_id = create_result
        .get("id")
        .and_then(|v| v.as_str())
        .expect("context id from create")
        .to_string();

    let update_params = Some(json!({"id": ctx_id, "data": {"updated": true}}));
    let update_result = server.handle_context_update(update_params).await?;
    assert_eq!(
        update_result.get("id").and_then(|v| v.as_str()),
        Some(ctx_id.as_str())
    );
    assert!(
        update_result
            .get("version")
            .and_then(serde_json::Value::as_u64)
            .expect("version after update")
            >= 1
    );
    Ok(())
}

#[tokio::test]
async fn test_handle_context_summarize_valid() -> TestResult {
    let server = make_server();
    let create_params = Some(json!({"session_id": "summarize-test-session"}));
    let create_result = server.handle_context_create(create_params).await?;
    let ctx_id = create_result
        .get("id")
        .and_then(|v| v.as_str())
        .expect("context id from create")
        .to_string();

    let summarize_params = Some(json!({"id": ctx_id}));
    let summarize_result = server.handle_context_summarize(summarize_params).await?;
    assert_eq!(
        summarize_result.get("id").and_then(|v| v.as_str()),
        Some(ctx_id.as_str())
    );
    assert!(
        summarize_result
            .get("summary")
            .and_then(|v| v.as_str())
            .expect("summary text")
            .contains("Context")
    );
    Ok(())
}

#[tokio::test]
async fn test_handle_execute_tool_system_health() -> TestResult {
    let server = make_server();
    let params = Some(json!({"tool": "system.health", "args": {}}));
    let result = server.handle_execute_tool(params).await?;
    assert_eq!(
        result.get("tool").and_then(|v| v.as_str()),
        Some("system.health")
    );
    assert_eq!(
        result.get("success").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert!(
        result
            .get("output")
            .and_then(|v| v.as_str())
            .expect("tool output")
            .contains("healthy")
    );
    Ok(())
}

#[tokio::test]
async fn test_handle_execute_tool_unknown_tool() -> TestResult {
    let server = make_server();
    let params = Some(json!({"tool": "nonexistent.tool", "args": {}}));
    let result = server.handle_execute_tool(params).await?;
    assert_eq!(
        result.get("success").and_then(serde_json::Value::as_bool),
        Some(false)
    );
    assert!(
        result
            .get("error")
            .and_then(|v| v.as_str())
            .expect("error message for unknown tool")
            .contains("not found")
    );
    Ok(())
}

#[tokio::test]
async fn test_handle_request_method_not_found() -> TestResult {
    let server = make_server();
    let req = r#"{"jsonrpc":"2.0","method":"unknown.method","id":1}"#;
    let result = server.handle_request_or_batch(req).await;
    assert!(result.is_some());
    let body = result.expect("method not found response");
    let parsed: serde_json::Value = serde_json::from_str(&body)?;
    assert!(parsed.get("error").is_some());
    assert_eq!(
        parsed
            .get("error")
            .and_then(|e| e.get("code"))
            .and_then(serde_json::Value::as_i64),
        Some(i64::from(error_codes::METHOD_NOT_FOUND))
    );
    Ok(())
}

#[tokio::test]
async fn test_handle_batch_notification_no_response() {
    let server = make_server();
    let batch = r#"[{"jsonrpc":"2.0","method":"system.ping"}]"#;
    let result = server.handle_request_or_batch(batch).await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_handle_request_tool_execute() -> TestResult {
    let server = make_server();
    let req = r#"{"jsonrpc":"2.0","method":"tool.execute","params":{"tool":"system.health","args":{}},"id":1}"#;
    let result = server.handle_request_or_batch(req).await;
    assert!(result.is_some());
    let body = result.expect("tool.execute response");
    let parsed: serde_json::Value = serde_json::from_str(&body)?;
    assert!(parsed.get("result").is_some());
    assert_eq!(
        parsed
            .get("result")
            .and_then(|r| r.get("success"))
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    Ok(())
}

/// `capability.discover` appends AI capability names when the AI router has providers.
#[tokio::test]
async fn test_handle_discover_capabilities_adds_ai_methods_with_router() -> TestResult {
    use crate::api::ai::AiRouter;
    use crate::api::ai::adapters::{AiProviderAdapter, QualityTier};
    use crate::api::ai::types::{
        ImageGenerationRequest, ImageGenerationResponse, TextGenerationRequest,
        TextGenerationResponse,
    };
    use async_trait::async_trait;
    use std::sync::Arc;

    struct OneTextProvider;

    #[async_trait]
    impl AiProviderAdapter for OneTextProvider {
        fn provider_id(&self) -> &'static str {
            "test-provider"
        }
        fn provider_name(&self) -> &'static str {
            "Test"
        }
        fn is_local(&self) -> bool {
            true
        }
        fn cost_per_unit(&self) -> Option<f64> {
            None
        }
        fn avg_latency_ms(&self) -> u64 {
            0
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
            _request: TextGenerationRequest,
        ) -> Result<TextGenerationResponse, crate::error::PrimalError> {
            unreachable!("not called by discover")
        }
        async fn generate_image(
            &self,
            _request: ImageGenerationRequest,
        ) -> Result<ImageGenerationResponse, crate::error::PrimalError> {
            unreachable!("not called by discover")
        }
    }

    let server = JsonRpcServer::with_ai_router(
        "/tmp/jsonrpc-discover-ai.sock".to_string(),
        Arc::new(AiRouter::from_adapters_for_test(vec![Arc::new(
            OneTextProvider,
        )])),
    );
    let v = server.handle_discover_capabilities().await?;
    let arr = v
        .get("capabilities")
        .and_then(serde_json::Value::as_array)
        .expect("capabilities array");
    let strs: Vec<&str> = arr.iter().filter_map(|x| x.as_str()).collect();
    assert!(strs.contains(&"ai.inference"));
    assert!(strs.contains(&"ai.text_generation"));
    Ok(())
}

/// `handle_query_ai` success path when `ai_router` is configured.
#[tokio::test]
async fn test_handle_query_ai_with_router_success() -> TestResult {
    use crate::api::ai::AiRouter;
    use crate::api::ai::adapters::{AiProviderAdapter, QualityTier};
    use crate::api::ai::types::{
        ImageGenerationRequest, ImageGenerationResponse, TextGenerationRequest,
        TextGenerationResponse,
    };
    use crate::error::PrimalError;
    use async_trait::async_trait;
    use std::sync::Arc;

    struct EchoText;

    #[async_trait]
    impl AiProviderAdapter for EchoText {
        fn provider_id(&self) -> &'static str {
            "echo-p"
        }

        fn provider_name(&self) -> &'static str {
            "Echo"
        }

        fn is_local(&self) -> bool {
            true
        }

        fn cost_per_unit(&self) -> Option<f64> {
            Some(0.0)
        }

        fn avg_latency_ms(&self) -> u64 {
            1
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
                text: format!("reply:{}", request.prompt),
                provider_id: "echo-p".to_string(),
                model: "mock-model".to_string(),
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

    let server = JsonRpcServer::with_ai_router(
        "/tmp/jsonrpc-ai-query-ok.sock".to_string(),
        Arc::new(AiRouter::from_adapters_for_test(vec![Arc::new(EchoText)])),
    );
    let v = server
        .handle_query_ai(Some(json!({"prompt": "hello"})))
        .await
        .expect("query ok");
    assert_eq!(
        v.get("success").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        v.get("response").and_then(serde_json::Value::as_str),
        Some("reply:hello")
    );
    Ok(())
}

#[tokio::test]
async fn test_handle_query_ai_router_returns_error() {
    use crate::api::ai::AiRouter;
    use crate::api::ai::adapters::{AiProviderAdapter, QualityTier};
    use crate::api::ai::types::{
        ImageGenerationRequest, ImageGenerationResponse, TextGenerationRequest,
        TextGenerationResponse,
    };
    use crate::error::PrimalError;
    use async_trait::async_trait;
    use std::sync::Arc;

    struct FailText;

    #[async_trait]
    impl AiProviderAdapter for FailText {
        fn provider_id(&self) -> &'static str {
            "fail-p"
        }

        fn provider_name(&self) -> &'static str {
            "Fail"
        }

        fn is_local(&self) -> bool {
            true
        }

        fn cost_per_unit(&self) -> Option<f64> {
            None
        }

        fn avg_latency_ms(&self) -> u64 {
            1
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
            _request: TextGenerationRequest,
        ) -> Result<TextGenerationResponse, PrimalError> {
            Err(PrimalError::OperationFailed(
                "router failed as expected".to_string(),
            ))
        }

        async fn generate_image(
            &self,
            _request: ImageGenerationRequest,
        ) -> Result<ImageGenerationResponse, PrimalError> {
            Err(PrimalError::OperationFailed("no image".to_string()))
        }
    }

    let server = JsonRpcServer::with_ai_router(
        "/tmp/jsonrpc-ai-query-fail.sock".to_string(),
        Arc::new(AiRouter::from_adapters_for_test(vec![Arc::new(FailText)])),
    );
    let err = server
        .handle_query_ai(Some(json!({"prompt": "x"})))
        .await
        .expect_err("expected router error");
    assert!(err.message.contains("router failed"));
}

#[tokio::test]
async fn test_handle_list_providers_with_router_non_empty() -> TestResult {
    use crate::api::ai::AiRouter;
    use crate::api::ai::adapters::{AiProviderAdapter, QualityTier};
    use crate::api::ai::types::{
        ImageGenerationRequest, ImageGenerationResponse, TextGenerationRequest,
        TextGenerationResponse,
    };
    use async_trait::async_trait;
    use std::sync::Arc;

    struct Listed;

    #[async_trait]
    impl AiProviderAdapter for Listed {
        fn provider_id(&self) -> &'static str {
            "listed-p"
        }

        fn provider_name(&self) -> &'static str {
            "Listed"
        }

        fn is_local(&self) -> bool {
            true
        }

        fn cost_per_unit(&self) -> Option<f64> {
            Some(0.02)
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
            _request: TextGenerationRequest,
        ) -> Result<TextGenerationResponse, crate::error::PrimalError> {
            unreachable!("list only")
        }

        async fn generate_image(
            &self,
            _request: ImageGenerationRequest,
        ) -> Result<ImageGenerationResponse, crate::error::PrimalError> {
            unreachable!("list only")
        }
    }

    let server = JsonRpcServer::with_ai_router(
        "/tmp/jsonrpc-ai-list.sock".to_string(),
        Arc::new(AiRouter::from_adapters_for_test(vec![Arc::new(Listed)])),
    );
    let v = server.handle_list_providers(None).await.expect("list");
    assert_eq!(v.get("total").and_then(serde_json::Value::as_u64), Some(1));
    let providers = v
        .get("providers")
        .and_then(serde_json::Value::as_array)
        .expect("providers");
    assert_eq!(providers.len(), 1);
    assert_eq!(
        providers[0]
            .get("cost_tier")
            .and_then(serde_json::Value::as_str),
        Some("high")
    );
    Ok(())
}
