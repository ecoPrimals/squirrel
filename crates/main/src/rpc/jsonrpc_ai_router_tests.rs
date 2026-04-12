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
