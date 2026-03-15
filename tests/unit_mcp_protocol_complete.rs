// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! # Comprehensive MCP Protocol Unit Tests
//!
//! Provides extensive unit test coverage for the MCP protocol implementation,
//! focusing on message handling, serialization, error cases, and edge conditions.

use squirrel_mcp::protocol::{McpMessage, McpMessageType, McpError};
use squirrel_mcp::types::Priority;
use serde_json::{json, Value};
use std::collections::HashMap;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// MESSAGE CREATION AND VALIDATION TESTS
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn test_mcp_message_new() {
    let msg = McpMessage::new(
        McpMessageType::Request,
        json!({"test": "data"}),
    );
    
    assert!(!msg.id.0.is_empty());
    assert_eq!(msg.message_type, McpMessageType::Request);
    assert_eq!(msg.payload.get("test").and_then(Value::as_str), Some("data"));
}

#[test]
fn test_mcp_message_with_metadata() {
    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), Value::String("squirrel".to_string()));
    metadata.insert("priority".to_string(), Value::String("high".to_string()));
    
    let msg = McpMessage::new(McpMessageType::Request, json!({}))
        .with_metadata(metadata.clone());
    
    assert_eq!(
        msg.metadata.get("source").and_then(Value::as_str),
        Some("squirrel")
    );
    assert_eq!(
        msg.metadata.get("priority").and_then(Value::as_str),
        Some("high")
    );
}

#[test]
fn test_mcp_message_with_priority() {
    let msg = McpMessage::new(McpMessageType::Request, json!({}));
    
    // Default priority should be Normal
    let priority = msg.metadata.get("priority")
        .and_then(Value::as_str)
        .unwrap_or("normal");
    
    assert_eq!(priority, "normal");
}

#[test]
fn test_mcp_message_types() {
    let request = McpMessage::new(McpMessageType::Request, json!({}));
    let response = McpMessage::new(McpMessageType::Response, json!({}));
    let event = McpMessage::new(McpMessageType::Event, json!({}));
    
    assert_eq!(request.message_type, McpMessageType::Request);
    assert_eq!(response.message_type, McpMessageType::Response);
    assert_eq!(event.message_type, McpMessageType::Event);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// SERIALIZATION AND DESERIALIZATION TESTS
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn test_mcp_message_serialization() {
    let msg = McpMessage::new(
        McpMessageType::Request,
        json!({"action": "test"}),
    );
    
    let serialized = serde_json::to_string(&msg)
        .expect("Failed to serialize message");
    
    assert!(serialized.contains("\"action\":\"test\""));
}

#[test]
fn test_mcp_message_deserialization() {
    let json_str = r#"{
        "id": {"0": "test-id-123"},
        "message_type": "Request",
        "payload": {"action": "ping"},
        "metadata": {}
    }"#;
    
    let msg: McpMessage = serde_json::from_str(json_str)
        .expect("Failed to deserialize message");
    
    assert_eq!(msg.id.0, "test-id-123");
    assert_eq!(msg.message_type, McpMessageType::Request);
    assert_eq!(
        msg.payload.get("action").and_then(Value::as_str),
        Some("ping")
    );
}

#[test]
fn test_mcp_message_roundtrip() {
    let original = McpMessage::new(
        McpMessageType::Response,
        json!({
            "status": "success",
            "data": {
                "items": [1, 2, 3],
                "total": 3
            }
        }),
    );
    
    let serialized = serde_json::to_string(&original)
        .expect("Serialization failed");
    
    let deserialized: McpMessage = serde_json::from_str(&serialized)
        .expect("Deserialization failed");
    
    assert_eq!(original.message_type, deserialized.message_type);
    assert_eq!(
        original.payload.get("status"),
        deserialized.payload.get("status")
    );
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// EDGE CASES AND ERROR HANDLING
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn test_mcp_message_empty_payload() {
    let msg = McpMessage::new(McpMessageType::Event, json!({}));
    
    assert!(msg.payload.is_object());
    assert_eq!(msg.payload.as_object().map(|o| o.len()), Some(0));
}

#[test]
fn test_mcp_message_large_payload() {
    let large_data: Vec<i32> = (0..10000).collect();
    let msg = McpMessage::new(
        McpMessageType::Request,
        json!({"data": large_data}),
    );
    
    let payload_array = msg.payload.get("data")
        .and_then(Value::as_array);
    
    assert_eq!(payload_array.map(|a| a.len()), Some(10000));
}

#[test]
fn test_mcp_message_nested_payload() {
    let nested = json!({
        "level1": {
            "level2": {
                "level3": {
                    "value": "deep"
                }
            }
        }
    });
    
    let msg = McpMessage::new(McpMessageType::Request, nested);
    
    let deep_value = msg.payload
        .get("level1")
        .and_then(|v| v.get("level2"))
        .and_then(|v| v.get("level3"))
        .and_then(|v| v.get("value"))
        .and_then(Value::as_str);
    
    assert_eq!(deep_value, Some("deep"));
}

#[test]
fn test_mcp_message_special_characters() {
    let special_data = json!({
        "unicode": "Hello 世界 🌍",
        "newlines": "line1\nline2\nline3",
        "quotes": "She said \"hello\"",
        "backslash": "path\\to\\file"
    });
    
    let msg = McpMessage::new(McpMessageType::Request, special_data);
    
    let serialized = serde_json::to_string(&msg)
        .expect("Failed to serialize");
    
    let deserialized: McpMessage = serde_json::from_str(&serialized)
        .expect("Failed to deserialize");
    
    assert_eq!(
        msg.payload.get("unicode"),
        deserialized.payload.get("unicode")
    );
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// METADATA HANDLING TESTS
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn test_mcp_message_metadata_update() {
    let mut msg = McpMessage::new(McpMessageType::Request, json!({}));
    
    let mut metadata = HashMap::new();
    metadata.insert("key1".to_string(), json!("value1"));
    msg = msg.with_metadata(metadata);
    
    assert_eq!(
        msg.metadata.get("key1").and_then(Value::as_str),
        Some("value1")
    );
}

#[test]
fn test_mcp_message_metadata_multiple_types() {
    let mut metadata = HashMap::new();
    metadata.insert("string".to_string(), json!("text"));
    metadata.insert("number".to_string(), json!(42));
    metadata.insert("boolean".to_string(), json!(true));
    metadata.insert("array".to_string(), json!([1, 2, 3]));
    
    let msg = McpMessage::new(McpMessageType::Request, json!({}))
        .with_metadata(metadata);
    
    assert_eq!(msg.metadata.get("string").and_then(Value::as_str), Some("text"));
    assert_eq!(msg.metadata.get("number").and_then(Value::as_i64), Some(42));
    assert_eq!(msg.metadata.get("boolean").and_then(Value::as_bool), Some(true));
    assert_eq!(
        msg.metadata.get("array").and_then(Value::as_array).map(|a| a.len()),
        Some(3)
    );
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// CLONING AND EQUALITY TESTS
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn test_mcp_message_clone() {
    let original = McpMessage::new(
        McpMessageType::Request,
        json!({"test": "data"}),
    );
    
    let cloned = original.clone();
    
    assert_eq!(original.id, cloned.id);
    assert_eq!(original.message_type, cloned.message_type);
    assert_eq!(original.payload, cloned.payload);
}

#[test]
fn test_mcp_message_debug_format() {
    let msg = McpMessage::new(
        McpMessageType::Request,
        json!({"action": "test"}),
    );
    
    let debug_string = format!("{:?}", msg);
    
    assert!(debug_string.contains("McpMessage"));
    assert!(debug_string.contains("Request"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// CONCURRENT ACCESS TESTS
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn test_mcp_message_concurrent_creation() {
    use std::sync::Arc;
    use tokio::task;
    
    let mut handles = vec![];
    
    for i in 0..100 {
        let handle = task::spawn(async move {
            McpMessage::new(
                McpMessageType::Request,
                json!({"index": i}),
            )
        });
        handles.push(handle);
    }
    
    let messages: Vec<McpMessage> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.expect("Task failed"))
        .collect();
    
    assert_eq!(messages.len(), 100);
    
    // Verify all messages have unique IDs
    let mut ids = std::collections::HashSet::new();
    for msg in messages {
        assert!(ids.insert(msg.id.0.clone()), "Duplicate ID found");
    }
}

#[tokio::test]
async fn test_mcp_message_concurrent_serialization() {
    use std::sync::Arc;
    use tokio::task;
    
    let msg = Arc::new(McpMessage::new(
        McpMessageType::Request,
        json!({"shared": "data"}),
    ));
    
    let mut handles = vec![];
    
    for _ in 0..50 {
        let msg_clone = Arc::clone(&msg);
        let handle = task::spawn(async move {
            serde_json::to_string(&*msg_clone)
        });
        handles.push(handle);
    }
    
    let results: Vec<String> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.expect("Task failed").expect("Serialization failed"))
        .collect();
    
    assert_eq!(results.len(), 50);
    
    // All serializations should be identical
    let first = &results[0];
    for result in &results[1..] {
        assert_eq!(first, result);
    }
}

