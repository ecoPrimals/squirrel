// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use crate::api::ai::AiRouter;
use crate::api::ai::adapters::AiProvider;
use crate::api::ai::adapters::test_mocks::{TestAiAdapter, TextBehavior};
use serde_json::json;
use std::sync::Arc;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn make_server_with_adapter(sock: &str, adapter: TestAiAdapter) -> JsonRpcServer {
    JsonRpcServer::with_ai_router(
        sock.to_string(),
        Arc::new(AiRouter::from_adapters_for_test(vec![Arc::new(
            AiProvider::JsonRpcTestAi(adapter),
        )])),
    )
}

/// `capability.discover` appends AI capability names when the AI router has providers.
#[tokio::test]
async fn test_handle_discover_capabilities_adds_ai_methods_with_router() -> TestResult {
    let adapter = TestAiAdapter::text_only("test-provider", "Test", TextBehavior::Unreachable);
    let server = make_server_with_adapter("/tmp/jsonrpc-discover-ai.sock", adapter);

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
    let adapter = TestAiAdapter::text_only("echo-p", "Echo", TextBehavior::Echo);
    let server = make_server_with_adapter("/tmp/jsonrpc-ai-query-ok.sock", adapter);

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
    let adapter = TestAiAdapter::text_only(
        "fail-p",
        "Fail",
        TextBehavior::Fail("router failed as expected"),
    );
    let server = make_server_with_adapter("/tmp/jsonrpc-ai-query-fail.sock", adapter);

    let err = server
        .handle_query_ai(Some(json!({"prompt": "x"})))
        .await
        .expect_err("expected router error");
    assert!(err.message.contains("router failed"));
}

#[tokio::test]
async fn test_handle_list_providers_with_router_non_empty() -> TestResult {
    let adapter =
        TestAiAdapter::text_only("listed-p", "Listed", TextBehavior::Unreachable).with_cost(0.02);
    let server = make_server_with_adapter("/tmp/jsonrpc-ai-list.sock", adapter);

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

// === parse_signal_plan unit tests ===

#[test]
fn parse_signal_plan_valid_json_array() {
    let input = r#"[{"tier":"t1","signal":"data.fetch","params":{"key":"v"},"reason":"r"}]"#;
    let result = JsonRpcServer::parse_signal_plan(input);
    let steps = result.expect("valid JSON");
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].signal, "data.fetch");
    assert_eq!(steps[0].tier, "t1");
    assert_eq!(steps[0].reason.as_deref(), Some("r"));
}

#[test]
fn parse_signal_plan_strips_markdown_fences() {
    let input = "```json\n[{\"tier\":\"t1\",\"signal\":\"a.b\",\"params\":{}}]\n```";
    let steps = JsonRpcServer::parse_signal_plan(input).expect("fenced JSON");
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].signal, "a.b");
}

#[test]
fn parse_signal_plan_invalid_json_returns_error() {
    let input = "not json at all";
    let err = JsonRpcServer::parse_signal_plan(input).expect_err("bad JSON");
    assert!(err.message.contains("Failed to parse signal plan"));
}

#[test]
fn parse_signal_plan_empty_array_ok() {
    let steps = JsonRpcServer::parse_signal_plan("[]").expect("empty array");
    assert!(steps.is_empty());
}

#[test]
fn parse_signal_plan_multiple_steps() {
    let input = r#"[
        {"tier":"t1","signal":"a.b","params":{}},
        {"tier":"t2","signal":"c.d","params":{"x":1},"reason":"step 2"}
    ]"#;
    let steps = JsonRpcServer::parse_signal_plan(input).expect("multi-step");
    assert_eq!(steps.len(), 2);
    assert_eq!(steps[1].tier, "t2");
}

// === parse_signal_tools_toml unit tests ===

#[test]
fn parse_signal_tools_toml_valid() {
    let toml_str = r#"
[[tools]]
name = "data.fetch"
tier = "t1"
description = "Fetch data from a source"

[[tools]]
name = "compute.run"
tier = "t2"
description = "Run a computation"
"#;
    let tools = JsonRpcServer::parse_signal_tools_toml(toml_str).expect("valid TOML");
    assert_eq!(tools.len(), 2);
    assert_eq!(tools[0].name, "data.fetch");
    assert_eq!(tools[1].tier, "t2");
}

#[test]
fn parse_signal_tools_toml_missing_tools_array() {
    let toml_str = r#"[config]
key = "value"
"#;
    let err = JsonRpcServer::parse_signal_tools_toml(toml_str).expect_err("no [[tools]]");
    assert!(err.message.contains("missing [[tools]] array"));
}

#[test]
fn parse_signal_tools_toml_invalid_toml() {
    let err = JsonRpcServer::parse_signal_tools_toml("{{bad}}").expect_err("bad TOML");
    assert!(err.message.contains("Failed to parse signal_tools.toml"));
}

#[test]
fn parse_signal_tools_toml_missing_fields_get_defaults() {
    let toml_str = r#"
[[tools]]
name = "ok.tool"
tier = "t1"
description = "valid"

[[tools]]
tier = "t2"
description = "missing name"
"#;
    let tools = JsonRpcServer::parse_signal_tools_toml(toml_str).expect("partial parse");
    assert_eq!(
        tools.len(),
        2,
        "both tools parsed (missing fields get defaults)"
    );
    assert_eq!(tools[0].name, "ok.tool");
    assert_eq!(tools[1].name, "unknown");
}

// === signal_plan integration via router ===

#[tokio::test]
async fn signal_plan_mode_dispatches_to_router_and_parses_steps() {
    let plan_json = r#"[{"tier":"t1","signal":"data.fetch","params":{},"reason":"test"}]"#;
    let adapter = TestAiAdapter::text_only("plan-p", "Planner", TextBehavior::Static(plan_json));
    let server = make_server_with_adapter("/tmp/jsonrpc-signal-plan.sock", adapter);

    let result = server
        .handle_query_ai(Some(json!({
            "prompt": "fetch my data",
            "mode": "signal_plan",
            "tools": [{"name": "data.fetch", "tier": "t1", "description": "Fetch data"}]
        })))
        .await
        .expect("signal_plan should succeed");

    assert!(result.get("success").and_then(|v| v.as_bool()) == Some(true));
    let plan = result.get("plan").and_then(|v| v.as_array()).expect("plan");
    assert_eq!(plan.len(), 1);
    assert_eq!(
        plan[0].get("signal").and_then(|v| v.as_str()),
        Some("data.fetch")
    );
}

#[tokio::test]
async fn signal_plan_no_tools_errors() {
    let adapter = TestAiAdapter::text_only("plan-notools", "P", TextBehavior::Unreachable);
    let server = make_server_with_adapter("/tmp/jsonrpc-signal-plan-notools.sock", adapter);

    let err = server
        .handle_query_ai(Some(json!({
            "prompt": "hi",
            "mode": "signal_plan"
        })))
        .await
        .expect_err("should fail without tools");
    assert!(err.message.contains("No signal tools"));
}

#[tokio::test]
async fn signal_plan_no_router_errors() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-signal-plan-norouter.sock".to_string());

    let err = server
        .handle_query_ai(Some(json!({
            "prompt": "hi",
            "mode": "signal_plan",
            "tools": [{"name": "x.y", "tier": "t1", "description": "d"}]
        })))
        .await
        .expect_err("should fail without router");
    assert!(err.message.contains("AI router not configured"));
}

// === request_tracker assertion after RPC dispatch ===

#[tokio::test]
async fn request_tracker_increments_after_rpc_dispatch() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-tracker.sock".to_string());

    assert_eq!(server.request_tracker.total_requests(), 0);

    let req = r#"{"jsonrpc":"2.0","method":"system.ping","id":1}"#;
    server
        .handle_request_or_batch(req)
        .await
        .expect("ping should succeed");

    assert!(
        server.request_tracker.total_requests() >= 1,
        "tracker should record at least 1 request after dispatch"
    );
}

#[tokio::test]
async fn request_tracker_counts_errors() {
    let server = JsonRpcServer::new("/tmp/jsonrpc-tracker-err.sock".to_string());

    let req = r#"{"jsonrpc":"2.0","method":"nonexistent.method","id":1}"#;
    server
        .handle_request_or_batch(req)
        .await
        .expect("should return error response, not None");

    assert!(
        server.request_tracker.total_requests() >= 1,
        "tracker should count error requests"
    );
    assert!(
        server.request_tracker.total_errors() >= 1,
        "tracker should count errors"
    );
}
