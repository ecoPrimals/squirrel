// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use serde_json::json;

fn make_server() -> JsonRpcServer {
    JsonRpcServer::new("/tmp/test.sock".to_string())
}

#[derive(Debug, serde::Deserialize)]
struct TestParams {
    name: String,
    count: u32,
}

#[tokio::test]
async fn test_parse_params_valid() {
    let server = make_server();
    let params = Some(json!({"name": "test", "count": 42}));
    let result: TestParams = server.parse_params(params).unwrap();
    assert_eq!(result.name, "test");
    assert_eq!(result.count, 42);
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
async fn test_error_response() {
    let server = make_server();
    let response = server.error_response(json!(1), -32000, "Custom error");
    assert_eq!(response.jsonrpc.as_ref(), "2.0");
    assert!(response.result.is_none());
    assert!(response.error.is_some());
    let err = response.error.unwrap();
    assert_eq!(err.code, -32000);
    assert_eq!(err.message, "Custom error");
    assert_eq!(response.id, json!(1));
}

#[tokio::test]
async fn test_handle_health() {
    let server = make_server();
    let result = server.handle_health().await.unwrap();
    assert!(result.get("status").and_then(|v| v.as_str()) == Some("healthy"));
    assert!(result.get("version").is_some());
    assert!(result.get("uptime_seconds").is_some());
}

#[tokio::test]
async fn test_handle_metrics() {
    let server = make_server();
    let result = server.handle_metrics().await.unwrap();
    assert!(result.get("requests_handled").is_some());
    assert!(result.get("errors").is_some());
    assert!(result.get("uptime_seconds").is_some());
    assert!(result.get("success_rate").is_some());
}

#[tokio::test]
async fn test_handle_ping() {
    let server = make_server();
    let result = server.handle_ping().await.unwrap();
    assert_eq!(
        result.get("pong").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert!(result.get("timestamp").is_some());
}

#[tokio::test]
async fn test_handle_discover_capabilities() {
    let server = make_server();
    let result = server.handle_discover_capabilities().await.unwrap();
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
}

#[tokio::test]
async fn test_handle_capability_list() {
    let server = make_server();
    let result = server.handle_capability_list().await.unwrap();
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

    let ai_query = methods.get("ai.query").unwrap().as_object().unwrap();
    assert!(ai_query.contains_key("cost"));
    assert!(ai_query.contains_key("depends_on"));
}

#[tokio::test]
async fn test_handle_announce_capabilities_valid() {
    let server = make_server();
    let params = Some(json!({"capabilities": ["ai.inference", "tool.execute"]}));
    let result = server.handle_announce_capabilities(params).await.unwrap();
    assert_eq!(
        result.get("success").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert!(
        result
            .get("message")
            .unwrap()
            .as_str()
            .unwrap()
            .contains('2')
    );
}

#[tokio::test]
async fn test_handle_announce_capabilities_missing_params() {
    let server = make_server();
    let result = server.handle_announce_capabilities(None).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, error_codes::INVALID_PARAMS);
}

#[tokio::test]
async fn test_handle_list_providers_no_router() {
    let server = make_server();
    let result = server.handle_list_providers(None).await.unwrap();
    assert_eq!(
        result.get("total").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert!(
        result
            .get("providers")
            .and_then(|v| v.as_array())
            .unwrap()
            .is_empty()
    );
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
async fn test_handle_batch_empty() {
    let server = make_server();
    let result = server.handle_request_or_batch("[]").await;
    assert!(result.is_some());
    let parsed: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
    assert!(parsed.get("error").is_some());
}

#[tokio::test]
async fn test_handle_batch_single() {
    let server = make_server();
    let batch = r#"[{"jsonrpc":"2.0","method":"system.ping","id":1}]"#;
    let result = server.handle_request_or_batch(batch).await;
    assert!(result.is_some());
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&result.unwrap()).unwrap();
    assert_eq!(parsed.len(), 1);
    assert!(parsed[0].get("result").is_some());
}

#[tokio::test]
async fn test_handle_batch_multi() {
    let server = make_server();
    let batch = r#"[
        {"jsonrpc":"2.0","method":"system.ping","id":1},
        {"jsonrpc":"2.0","method":"system.health","id":2}
    ]"#;
    let result = server.handle_request_or_batch(batch).await;
    assert!(result.is_some());
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&result.unwrap()).unwrap();
    assert_eq!(parsed.len(), 2);
}

#[tokio::test]
async fn test_handle_lifecycle_register() {
    let server = make_server();
    let result = server.handle_lifecycle_register().await.unwrap();
    assert_eq!(
        result.get("success").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert!(
        result
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap()
            .contains("squirrel")
    );
    assert!(result.get("version").is_some());
}

#[tokio::test]
async fn test_handle_lifecycle_status() {
    let server = make_server();
    let result = server.handle_lifecycle_status().await.unwrap();
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
}

#[tokio::test]
async fn test_handle_discover_peers() {
    let server = make_server();
    let result = server.handle_discover_peers(None).await.unwrap();
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
}

#[tokio::test]
async fn test_handle_single_request_ping() {
    let server = make_server();
    let req = r#"{"jsonrpc":"2.0","method":"system.ping","id":42}"#;
    let result = server.handle_request_or_batch(req).await;
    assert!(result.is_some());
    let parsed: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
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
}

#[tokio::test]
async fn test_handle_parse_error() {
    let server = make_server();
    let result = server.handle_request_or_batch("not valid json {{{").await;
    assert!(result.is_some());
    let parsed: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
    assert!(parsed.get("error").is_some());
    assert_eq!(
        parsed
            .get("error")
            .and_then(|e| e.get("code"))
            .and_then(serde_json::Value::as_i64),
        Some(i64::from(error_codes::PARSE_ERROR))
    );
}

#[tokio::test]
async fn test_handle_invalid_jsonrpc_version() {
    let server = make_server();
    let req = r#"{"jsonrpc":"1.0","method":"system.ping","id":1}"#;
    let result = server.handle_request_or_batch(req).await;
    assert!(result.is_some());
    let parsed: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
    assert!(parsed.get("error").is_some());
    assert!(
        parsed
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("")
            .contains("2.0")
    );
}

#[tokio::test]
async fn test_handle_announce_with_primal_and_socket() {
    let server = make_server();
    let params = Some(json!({
        "capabilities": ["tool.remote"],
        "primal": "songbird-1",
        "socket_path": "/tmp/songbird.sock",
        "tools": ["tool.remote"]
    }));
    let result = server.handle_announce_capabilities(params).await.unwrap();
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
}

#[tokio::test]
async fn test_handle_list_tools() {
    let server = make_server();
    let result = server.handle_list_tools().await.unwrap();
    assert!(result.get("tools").and_then(|v| v.as_array()).is_some());
    assert!(
        result
            .get("total")
            .and_then(serde_json::Value::as_u64)
            .is_some()
    );
}

#[tokio::test]
async fn test_handle_context_create() {
    let server = make_server();
    let params = Some(json!({"session_id": "test-session-123", "metadata": {"key": "value"}}));
    let result = server.handle_context_create(params).await.unwrap();
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
}

#[tokio::test]
async fn test_handle_context_create_auto_session_id() {
    let server = make_server();
    let result = server.handle_context_create(None).await.unwrap();
    assert!(
        result
            .get("id")
            .and_then(|v| v.as_str())
            .is_some_and(|s| !s.is_empty())
    );
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
async fn test_handle_request_context_create() {
    let server = make_server();
    let req =
        r#"{"jsonrpc":"2.0","method":"context.create","params":{"session_id":"req-test"},"id":1}"#;
    let result = server.handle_request_or_batch(req).await;
    assert!(result.is_some());
    let parsed: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
    assert!(parsed.get("result").is_some());
    assert!(parsed.get("result").and_then(|r| r.get("id")).is_some());
}

#[tokio::test]
async fn test_handle_request_tool_list() {
    let server = make_server();
    let req = r#"{"jsonrpc":"2.0","method":"tool.list","id":1}"#;
    let result = server.handle_request_or_batch(req).await;
    assert!(result.is_some());
    let parsed: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
    assert!(parsed.get("result").is_some());
    assert!(
        parsed
            .get("result")
            .and_then(|r| r.get("tools"))
            .and_then(|v| v.as_array())
            .is_some()
    );
}

#[tokio::test]
async fn test_handle_context_update_valid() {
    let server = make_server();
    let create_params =
        Some(json!({"session_id": "update-test-session", "metadata": {"key": "v1"}}));
    let create_result = server.handle_context_create(create_params).await.unwrap();
    let ctx_id = create_result
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap()
        .to_string();

    let update_params = Some(json!({"id": ctx_id, "data": {"updated": true}}));
    let update_result = server.handle_context_update(update_params).await.unwrap();
    assert_eq!(
        update_result.get("id").and_then(|v| v.as_str()),
        Some(ctx_id.as_str())
    );
    assert!(
        update_result
            .get("version")
            .and_then(serde_json::Value::as_u64)
            .unwrap()
            >= 1
    );
}

#[tokio::test]
async fn test_handle_context_summarize_valid() {
    let server = make_server();
    let create_params = Some(json!({"session_id": "summarize-test-session"}));
    let create_result = server.handle_context_create(create_params).await.unwrap();
    let ctx_id = create_result
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap()
        .to_string();

    let summarize_params = Some(json!({"id": ctx_id}));
    let summarize_result = server
        .handle_context_summarize(summarize_params)
        .await
        .unwrap();
    assert_eq!(
        summarize_result.get("id").and_then(|v| v.as_str()),
        Some(ctx_id.as_str())
    );
    assert!(
        summarize_result
            .get("summary")
            .and_then(|v| v.as_str())
            .unwrap()
            .contains("Context")
    );
}

#[tokio::test]
async fn test_handle_execute_tool_system_health() {
    let server = make_server();
    let params = Some(json!({"tool": "system.health", "args": {}}));
    let result = server.handle_execute_tool(params).await.unwrap();
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
            .unwrap()
            .contains("healthy")
    );
}

#[tokio::test]
async fn test_handle_execute_tool_unknown_tool() {
    let server = make_server();
    let params = Some(json!({"tool": "nonexistent.tool", "args": {}}));
    let result = server.handle_execute_tool(params).await.unwrap();
    assert_eq!(
        result.get("success").and_then(serde_json::Value::as_bool),
        Some(false)
    );
    assert!(
        result
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap()
            .contains("not found")
    );
}

#[tokio::test]
async fn test_handle_request_method_not_found() {
    let server = make_server();
    let req = r#"{"jsonrpc":"2.0","method":"unknown.method","id":1}"#;
    let result = server.handle_request_or_batch(req).await;
    assert!(result.is_some());
    let parsed: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
    assert!(parsed.get("error").is_some());
    assert_eq!(
        parsed
            .get("error")
            .and_then(|e| e.get("code"))
            .and_then(serde_json::Value::as_i64),
        Some(i64::from(error_codes::METHOD_NOT_FOUND))
    );
}

#[tokio::test]
async fn test_handle_batch_notification_no_response() {
    let server = make_server();
    let batch = r#"[{"jsonrpc":"2.0","method":"system.ping"}]"#;
    let result = server.handle_request_or_batch(batch).await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_handle_request_tool_execute() {
    let server = make_server();
    let req = r#"{"jsonrpc":"2.0","method":"tool.execute","params":{"tool":"system.health","args":{}},"id":1}"#;
    let result = server.handle_request_or_batch(req).await;
    assert!(result.is_some());
    let parsed: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
    assert!(parsed.get("result").is_some());
    assert_eq!(
        parsed
            .get("result")
            .and_then(|r| r.get("success"))
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
}

/// `capability.discover` appends AI capability names when the AI router has providers.
#[tokio::test]
async fn test_handle_discover_capabilities_adds_ai_methods_with_router() {
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
    let v = server.handle_discover_capabilities().await.unwrap();
    let arr = v
        .get("capabilities")
        .and_then(serde_json::Value::as_array)
        .expect("capabilities array");
    let strs: Vec<&str> = arr.iter().filter_map(|x| x.as_str()).collect();
    assert!(strs.contains(&"ai.inference"));
    assert!(strs.contains(&"ai.text_generation"));
}

/// `handle_query_ai` success path when `ai_router` is configured.
#[tokio::test]
async fn test_handle_query_ai_with_router_success() {
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
async fn test_handle_list_providers_with_router_non_empty() {
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
}
