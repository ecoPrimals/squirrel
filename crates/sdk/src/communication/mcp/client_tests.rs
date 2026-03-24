// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(clippy::unwrap_used, clippy::expect_used)] // Test code: explicit unwrap/expect and local lint noise

use super::*;
use serde_json::json;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::JsValue;

#[test]
fn message_category_serde_roundtrip() {
    let c = MessageCategory::ToolInvocation;
    let s = serde_json::to_string(&c).unwrap();
    assert_eq!(s, "\"tool_invocation\"");
    let back: MessageCategory = serde_json::from_str(&s).unwrap();
    assert_eq!(back, c);

    let unknown: MessageCategory = serde_json::from_str("\"unknown\"").unwrap();
    assert_eq!(unknown, MessageCategory::Generic);
}

#[test]
fn processing_strategy_serde_roundtrip() {
    let p = ProcessingStrategy::Streaming;
    let s = serde_json::to_string(&p).unwrap();
    let back: ProcessingStrategy = serde_json::from_str(&s).unwrap();
    assert_eq!(back, p);

    let unknown: ProcessingStrategy = serde_json::from_str("\"unknown\"").unwrap();
    assert_eq!(unknown, ProcessingStrategy::Standard);
}

#[test]
fn processed_payload_ai_message_and_response_serde() {
    let payload = ProcessedPayload {
        data: json!({"k": 1}),
        validation_status: "valid".to_string(),
        processing_hints: vec!["a".to_string()],
    };
    let msg = AiMcpMessage {
        id: "id1".to_string(),
        message_type: "t".to_string(),
        category: MessageCategory::Generic,
        payload,
        timestamp: 0,
        client_context: ClientContext {
            client_id: "c".to_string(),
            session_id: "s".to_string(),
            capabilities: vec!["x".to_string()],
        },
        processing_strategy: ProcessingStrategy::Standard,
    };
    let v = serde_json::to_value(&msg).unwrap();
    let msg2: AiMcpMessage = serde_json::from_value(v).unwrap();
    assert_eq!(msg2.id, msg.id);

    let resp = MessageResponse {
        success: true,
        data: json!({}),
        message_type: "r".to_string(),
        timestamp: 1,
    };
    let s = serde_json::to_string(&resp).unwrap();
    let r2: MessageResponse = serde_json::from_str(&s).unwrap();
    assert_eq!(r2.message_type, resp.message_type);
}

#[test]
fn mcp_client_constructors_and_state() {
    let c = McpClient::with_server_url("ws://127.0.0.1:9999");
    assert_eq!(c.config.server_url, "ws://127.0.0.1:9999");
    assert!(!c.connected());
    assert_eq!(c.state(), "Disconnected");

    let _: McpClient = McpClient::default();
    let n = McpClient::new();
    let cur = McpClient::current();
    let gl = McpClient::global();
    assert!(!n.connected() && !cur.connected() && !gl.connected());
}

#[tokio::test]
async fn disconnect_when_already_disconnected_ok() {
    let mut c = McpClient::new();
    c.disconnect().await.unwrap();
}

#[test]
fn classify_message_type_aliases_and_generic() {
    let c = McpClient::new();
    assert_eq!(
        c.test_classify_message_type("function_call").unwrap(),
        MessageCategory::ToolInvocation
    );
    assert_eq!(
        c.test_classify_message_type("file_request").unwrap(),
        MessageCategory::ResourceAccess
    );
    assert_eq!(
        c.test_classify_message_type("chat_completion").unwrap(),
        MessageCategory::Completion
    );
    assert_eq!(
        c.test_classify_message_type("state_change").unwrap(),
        MessageCategory::StateManagement
    );
    assert_eq!(
        c.test_classify_message_type("ping").unwrap(),
        MessageCategory::SystemHealth
    );
    assert_eq!(
        c.test_classify_message_type("anything_else").unwrap(),
        MessageCategory::Generic
    );
}

#[test]
fn processing_strategy_covers_all_categories() {
    let c = McpClient::new();
    let pairs = [
        (
            MessageCategory::ToolInvocation,
            ProcessingStrategy::Synchronous,
        ),
        (MessageCategory::ResourceAccess, ProcessingStrategy::Cached),
        (
            MessageCategory::Notification,
            ProcessingStrategy::Asynchronous,
        ),
        (MessageCategory::Completion, ProcessingStrategy::Streaming),
        (
            MessageCategory::StateManagement,
            ProcessingStrategy::Transactional,
        ),
        (MessageCategory::SystemHealth, ProcessingStrategy::Priority),
        (MessageCategory::Generic, ProcessingStrategy::Standard),
    ];
    for (cat, expected) in pairs {
        assert_eq!(c.test_determine_processing_strategy(&cat), expected);
    }
}

#[cfg(target_arch = "wasm32")]
#[tokio::test]
async fn send_message_tool_call_builds_response() {
    let mut client = McpClient::new();
    let payload = serde_wasm_bindgen::to_value(&json!({
        "tool": "demo_tool",
        "arguments": {"q": "hello"}
    }))
    .unwrap();

    let js = client.send_message("tool_call", payload).await.unwrap();
    let s = js_sys::JSON::stringify(&js).unwrap().as_string().unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert_eq!(v["success"], true);
    assert_eq!(v["message_type"], "tool_result");
}

#[cfg(target_arch = "wasm32")]
#[tokio::test]
async fn send_message_generic_and_notification() {
    let mut client = McpClient::new();
    let p1 = serde_wasm_bindgen::to_value(&json!({"x": 1})).unwrap();
    let r1 = client.send_message("custom", p1).await.unwrap();
    let s1 = js_sys::JSON::stringify(&r1).unwrap().as_string().unwrap();
    assert!(s1.contains("generic_response"));

    let p2 = serde_wasm_bindgen::to_value(&json!({"note": "n"})).unwrap();
    let r2 = client.send_message("notification", p2).await.unwrap();
    let s2 = js_sys::JSON::stringify(&r2).unwrap().as_string().unwrap();
    assert!(s2.contains("generic_response"));
}

#[cfg(target_arch = "wasm32")]
#[tokio::test]
async fn send_message_invalid_payload_errors() {
    let mut client = McpClient::new();
    let bad = JsValue::undefined();
    let err = client.send_message("custom", bad).await.unwrap_err();
    let s = format!("{err:?}");
    assert!(!s.is_empty());
}

#[cfg(target_arch = "wasm32")]
#[tokio::test]
async fn send_message_resource_and_completion_routes() {
    let mut client = McpClient::new();
    let p = serde_wasm_bindgen::to_value(&json!({"uri": "file:///a"})).unwrap();
    let r = client.send_message("resource_request", p).await.unwrap();
    let txt = js_sys::JSON::stringify(&r).unwrap().as_string().unwrap();
    assert!(txt.contains("Processed"));

    let mut c2 = McpClient::new();
    let p2 = serde_wasm_bindgen::to_value(&json!({"prompt": "p"})).unwrap();
    let r2 = c2.send_message("completion_request", p2).await.unwrap();
    let txt2 = js_sys::JSON::stringify(&r2).unwrap().as_string().unwrap();
    assert!(txt2.contains("Processed"));
}

#[cfg(target_arch = "wasm32")]
#[tokio::test]
async fn send_message_health_and_context_update() {
    let mut client = McpClient::new();
    let p = serde_wasm_bindgen::to_value(&json!({})).unwrap();
    let r = client.send_message("health_check", p).await.unwrap();
    assert!(
        js_sys::JSON::stringify(&r)
            .unwrap()
            .as_string()
            .unwrap()
            .contains("generic_response")
    );

    let mut c2 = McpClient::new();
    let r2 = c2
        .send_message(
            "context_update",
            serde_wasm_bindgen::to_value(&json!({"ctx": 1})).unwrap(),
        )
        .await
        .unwrap();
    assert!(
        js_sys::JSON::stringify(&r2)
            .unwrap()
            .as_string()
            .unwrap()
            .contains("generic_response")
    );
}
