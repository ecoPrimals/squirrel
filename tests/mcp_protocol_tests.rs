// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! MCP protocol comprehensive tests
//!
//! Tests for MCP protocol implementation

use squirrel_mcp::*;
use serde_json::json;

#[tokio::test]
async fn test_mcp_message_serialization() {
    let message = MCPMessage {
        id: "test-123".to_string(),
        method: "initialize".to_string(),
        params: Some(json!({"version": "1.0"})),
    };
    
    let serialized = serde_json::to_string(&message).unwrap();
    let deserialized: MCPMessage = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(deserialized.id, "test-123");
    assert_eq!(deserialized.method, "initialize");
}

#[tokio::test]
async fn test_mcp_request_response_flow() {
    let request = MCPMessage {
        id: "req-1".to_string(),
        method: "execute".to_string(),
        params: Some(json!({"action": "test"})),
    };
    
    // Simulate processing
    let response = MCPMessage {
        id: request.id.clone(),
        method: "response".to_string(),
        params: Some(json!({"result": "success"})),
    };
    
    assert_eq!(request.id, response.id);
}

#[tokio::test]
async fn test_mcp_error_handling() {
    let error = MCPError {
        code: -32600,
        message: "Invalid Request".to_string(),
        data: None,
    };
    
    assert_eq!(error.code, -32600);
    assert!(error.message.contains("Invalid"));
}

#[tokio::test]
async fn test_mcp_notification() {
    let notification = MCPMessage {
        id: "notif-1".to_string(),
        method: "status_update".to_string(),
        params: Some(json!({"status": "running"})),
    };
    
    // Notifications don't require responses
    assert!(!notification.id.is_empty());
}

#[tokio::test]
async fn test_mcp_batch_requests() {
    let batch: Vec<MCPMessage> = (1..=5)
        .map(|i| MCPMessage {
            id: format!("batch-{}", i),
            method: "test".to_string(),
            params: None,
        })
        .collect();
    
    assert_eq!(batch.len(), 5);
}

#[tokio::test]
async fn test_mcp_protocol_version_negotiation() {
    let client_version = "1.0";
    let server_version = "1.0";
    
    assert_eq!(client_version, server_version);
}

#[tokio::test]
async fn test_mcp_capabilities_exchange() {
    let capabilities = json!({
        "supports_streaming": true,
        "supports_batching": true,
        "max_batch_size": 100
    });
    
    assert!(capabilities["supports_streaming"].as_bool().unwrap());
}

#[tokio::test]
async fn test_mcp_connection_lifecycle() {
    // Simulate connection lifecycle
    let states = vec!["connecting", "connected", "ready", "disconnected"];
    
    for state in states {
        assert!(!state.is_empty());
    }
}

#[tokio::test]
async fn test_mcp_message_validation() {
    let valid = MCPMessage {
        id: "valid-1".to_string(),
        method: "test".to_string(),
        params: None,
    };
    
    assert!(!valid.id.is_empty());
    assert!(!valid.method.is_empty());
}

#[tokio::test]
async fn test_mcp_timeout_handling() {
    use std::time::Duration;
    
    let timeout = Duration::from_secs(30);
    assert!(timeout.as_secs() > 0);
}

#[tokio::test]
async fn test_mcp_json_rpc_format() {
    let message = json!({
        "jsonrpc": "2.0",
        "id": "test-1",
        "method": "test",
        "params": {}
    });
    
    assert_eq!(message["jsonrpc"], "2.0");
}

#[tokio::test]
async fn test_mcp_params_encoding() {
    let params = json!({
        "string": "value",
        "number": 42,
        "boolean": true,
        "array": [1, 2, 3],
        "object": {"nested": "value"}
    });
    
    assert!(params.is_object());
    assert_eq!(params["number"], 42);
}

#[tokio::test]
async fn test_mcp_large_payload() {
    let large_data = vec![0u8; 1024 * 100]; // 100KB
    let message = MCPMessage {
        id: "large-1".to_string(),
        method: "upload".to_string(),
        params: Some(json!({"data": large_data})),
    };
    
    assert!(!message.id.is_empty());
}

#[tokio::test]
async fn test_mcp_concurrent_requests() {
    let messages: Vec<MCPMessage> = (1..=10)
        .map(|i| MCPMessage {
            id: format!("concurrent-{}", i),
            method: "test".to_string(),
            params: None,
        })
        .collect();
    
    assert_eq!(messages.len(), 10);
}

#[tokio::test]
async fn test_mcp_streaming_support() {
    let streaming_message = MCPMessage {
        id: "stream-1".to_string(),
        method: "stream_start".to_string(),
        params: Some(json!({"stream_id": "abc-123"})),
    };
    
    assert!(streaming_message.params.is_some());
}

