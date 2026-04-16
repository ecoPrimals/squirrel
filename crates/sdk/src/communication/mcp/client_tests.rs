// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![expect(
    clippy::expect_used,
    reason = "MCP client tests use expect on known-good connection paths"
)]

use super::*;
use serde_json::json;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::JsValue;

#[test]
fn message_category_serde_roundtrip() {
    let c = MessageCategory::ToolInvocation;
    let s = serde_json::to_string(&c).expect("should succeed");
    assert_eq!(s, "\"tool_invocation\"");
    let back: MessageCategory = serde_json::from_str(&s).expect("should succeed");
    assert_eq!(back, c);

    let unknown: MessageCategory = serde_json::from_str("\"unknown\"").expect("should succeed");
    assert_eq!(unknown, MessageCategory::Generic);
}

#[test]
fn processing_strategy_serde_roundtrip() {
    let p = ProcessingStrategy::Streaming;
    let s = serde_json::to_string(&p).expect("should succeed");
    let back: ProcessingStrategy = serde_json::from_str(&s).expect("should succeed");
    assert_eq!(back, p);

    let unknown: ProcessingStrategy = serde_json::from_str("\"unknown\"").expect("should succeed");
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
    let v = serde_json::to_value(&msg).expect("should succeed");
    let msg2: AiMcpMessage = serde_json::from_value(v).expect("should succeed");
    assert_eq!(msg2.id, msg.id);

    let resp = MessageResponse {
        success: true,
        data: json!({}),
        message_type: "r".to_string(),
        timestamp: 1,
    };
    let s = serde_json::to_string(&resp).expect("should succeed");
    let r2: MessageResponse = serde_json::from_str(&s).expect("should succeed");
    assert_eq!(r2.message_type, resp.message_type);
}

#[test]
fn mcp_client_constructors_and_state() {
    let c = McpClient::with_server_url("unix:///tmp/squirrel-mcp-test-9999.sock");
    assert_eq!(
        c.config.server_url,
        "unix:///tmp/squirrel-mcp-test-9999.sock"
    );
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
    c.disconnect().await.expect("should succeed");
}

#[test]
fn classify_message_type_aliases_and_generic() {
    let c = McpClient::new();
    assert_eq!(
        c.test_classify_message_type("event")
            .expect("should succeed"),
        MessageCategory::Notification
    );
    assert_eq!(
        c.test_classify_message_type("function_call")
            .expect("should succeed"),
        MessageCategory::ToolInvocation
    );
    assert_eq!(
        c.test_classify_message_type("file_request")
            .expect("should succeed"),
        MessageCategory::ResourceAccess
    );
    assert_eq!(
        c.test_classify_message_type("chat_completion")
            .expect("should succeed"),
        MessageCategory::Completion
    );
    assert_eq!(
        c.test_classify_message_type("state_change")
            .expect("should succeed"),
        MessageCategory::StateManagement
    );
    assert_eq!(
        c.test_classify_message_type("ping")
            .expect("should succeed"),
        MessageCategory::SystemHealth
    );
    assert_eq!(
        c.test_classify_message_type("anything_else")
            .expect("should succeed"),
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

// `validate_and_process_payload` / `serialize_response_to_js` use `js_sys::JSON` and only run on wasm32.
#[cfg(target_arch = "wasm32")]
mod validate_payload_wasm {
    use super::*;
    use serde_json::json;
    use wasm_bindgen::prelude::JsValue;

    #[test]
    fn validate_payload_rejects_undefined_stringify_path() {
        let c = McpClient::new();
        let err = c
            .test_validate_and_process_payload("custom", JsValue::undefined())
            .unwrap_err();
        let s = format!("{err:?}");
        assert!(
            s.contains("Payload") || s.contains("stringify") || s.contains("undefined"),
            "unexpected error: {s}"
        );
    }

    #[test]
    fn validate_payload_rejects_invalid_json() {
        let c = McpClient::new();
        let payload = JsValue::from_str("not json {{{");
        let err = c
            .test_validate_and_process_payload("custom", payload)
            .unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("parse") || msg.contains("Payload"),
            "expected parse-related error: {msg}"
        );
    }

    #[test]
    fn validate_payload_tool_resource_completion_and_generic() {
        let c = McpClient::new();
        let tool = serde_wasm_bindgen::to_value(&json!({"tool": "t", "arguments": {}})).unwrap();
        let p = c
            .test_validate_and_process_payload("tool_call", tool)
            .expect("tool_call");
        assert_eq!(p.validation_status, "validated_tool_call");

        let res = serde_wasm_bindgen::to_value(&json!({"uri": "x"})).unwrap();
        let p2 = c
            .test_validate_and_process_payload("resource_request", res)
            .expect("resource");
        assert_eq!(p2.validation_status, "validated_resource_request");

        let comp = serde_wasm_bindgen::to_value(&json!({"prompt": "p"})).unwrap();
        let p3 = c
            .test_validate_and_process_payload("completion_request", comp)
            .expect("completion");
        assert_eq!(p3.validation_status, "validated_completion_request");

        let generic_payload = serde_wasm_bindgen::to_value(&json!({"a": 1})).unwrap();
        let p4 = c
            .test_validate_and_process_payload("anything", generic_payload)
            .expect("generic");
        assert_eq!(p4.validation_status, "passed");
        assert!(
            p4.processing_hints
                .iter()
                .any(|h| h == "generic_processing")
        );
    }

    #[test]
    fn serialize_response_round_trips() {
        let c = McpClient::new();
        let resp = MessageResponse {
            success: true,
            data: json!({"k": "v"}),
            message_type: "m".to_string(),
            timestamp: 0,
        };
        let js = c.test_serialize_response_to_js(resp).expect("serialize");
        let s = js_sys::JSON::stringify(&js)
            .expect("stringify")
            .as_string()
            .expect("as_string");
        let v: serde_json::Value = serde_json::from_str(&s).expect("json");
        assert_eq!(v["message_type"], "m");
        assert_eq!(v["success"], true);
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
    .expect("should succeed");

    let js = client
        .send_message("tool_call", payload)
        .await
        .expect("should succeed");
    let s = js_sys::JSON::stringify(&js)
        .expect("should succeed")
        .as_string()
        .expect("should succeed");
    let v: serde_json::Value = serde_json::from_str(&s).expect("should succeed");
    assert_eq!(v["success"], true);
    assert_eq!(v["message_type"], "tool_result");
}

#[cfg(target_arch = "wasm32")]
#[tokio::test]
async fn send_message_generic_and_notification() {
    let mut client = McpClient::new();
    let p1 = serde_wasm_bindgen::to_value(&json!({"x": 1})).expect("should succeed");
    let r1 = client
        .send_message("custom", p1)
        .await
        .expect("should succeed");
    let s1 = js_sys::JSON::stringify(&r1)
        .expect("should succeed")
        .as_string()
        .expect("should succeed");
    assert!(s1.contains("generic_response"));

    let p2 = serde_wasm_bindgen::to_value(&json!({"note": "n"})).expect("should succeed");
    let r2 = client
        .send_message("notification", p2)
        .await
        .expect("should succeed");
    let s2 = js_sys::JSON::stringify(&r2)
        .expect("should succeed")
        .as_string()
        .expect("should succeed");
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
    let p = serde_wasm_bindgen::to_value(&json!({"uri": "file:///a"})).expect("should succeed");
    let r = client
        .send_message("resource_request", p)
        .await
        .expect("should succeed");
    let txt = js_sys::JSON::stringify(&r)
        .expect("should succeed")
        .as_string()
        .expect("should succeed");
    assert!(txt.contains("Processed"));

    let mut c2 = McpClient::new();
    let p2 = serde_wasm_bindgen::to_value(&json!({"prompt": "p"})).expect("should succeed");
    let r2 = c2
        .send_message("completion_request", p2)
        .await
        .expect("should succeed");
    let txt2 = js_sys::JSON::stringify(&r2)
        .expect("should succeed")
        .as_string()
        .expect("should succeed");
    assert!(txt2.contains("Processed"));
}

#[cfg(target_arch = "wasm32")]
#[tokio::test]
async fn send_message_health_and_context_update() {
    let mut client = McpClient::new();
    let p = serde_wasm_bindgen::to_value(&json!({})).expect("should succeed");
    let r = client
        .send_message("health_check", p)
        .await
        .expect("should succeed");
    assert!(
        js_sys::JSON::stringify(&r)
            .expect("should succeed")
            .as_string()
            .expect("should succeed")
            .contains("generic_response")
    );

    let mut c2 = McpClient::new();
    let r2 = c2
        .send_message(
            "context_update",
            serde_wasm_bindgen::to_value(&json!({"ctx": 1})).expect("should succeed"),
        )
        .await
        .expect("should succeed");
    assert!(
        js_sys::JSON::stringify(&r2)
            .expect("should succeed")
            .as_string()
            .expect("should succeed")
            .contains("generic_response")
    );
}
