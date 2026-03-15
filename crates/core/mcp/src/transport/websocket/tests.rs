// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for WebSocket transport.

use super::*;

#[tokio::test]
async fn test_websocket_transport_create() {
    // Create a config with environment-based or default WebSocket port
    let test_url = std::env::var("TEST_WEBSOCKET_URL").unwrap_or_else(|_| {
        use universal_constants::network::get_service_port;
        let port = get_service_port("websocket");
        format!("ws://localhost:{}", port)
    });

    let config = WebSocketConfig {
        url: test_url.clone(),
        ..Default::default()
    };

    // Create transport
    let transport = WebSocketTransport::new(config);

    // Ensure it starts disconnected
    assert!(!transport.is_connected().await);

    // Get metadata
    let metadata = transport.get_metadata().await;
    assert_eq!(
        metadata
            .additional_info
            .get("transport_type")
            .unwrap_or(&"".to_string()),
        "websocket"
    );
    // Verify peer_addr is set (actual value depends on test environment)
    assert!(
        metadata.additional_info.get("peer_addr").is_some(),
        "peer_addr should be set in additional_info"
    );
}

#[tokio::test]
async fn test_websocket_transport_send_raw() {
    // Create a config with environment-based or default WebSocket port
    let test_url = std::env::var("TEST_WEBSOCKET_URL").unwrap_or_else(|_| {
        use universal_constants::network::get_service_port;
        let port = get_service_port("websocket");
        format!("ws://localhost:{}", port)
    });

    let config = WebSocketConfig {
        url: test_url,
        ..Default::default()
    };

    // Create transport
    let transport = WebSocketTransport::new(config);

    // Mock the connection state for testing
    {
        let mut state = transport.connection_state.lock().await;
        *state = WebSocketState::Connected;
    }

    // Test data to send
    let data = b"Hello WebSocket Raw Data!";

    // Since we're mocked as connected but not actually connected,
    // this should fail gracefully with a specific error
    let result = transport.send_raw(data).await;
    assert!(result.is_err());

    // We expect a specific error type - either ConnectionClosed or SendError
    if let Err(e) = result {
        let e_str = format!("{e:?}");
        assert!(
            e_str.contains("ConnectionClosed") || e_str.contains("SendError"),
            "Expected ConnectionClosed or SendError, got: {e:?}",
        );
    }
}

#[tokio::test]
async fn test_websocket_message_buffering() {
    // Create config
    let config = WebSocketConfig {
        url: "ws://localhost:8080".to_string(),
        ..Default::default()
    };

    let transport = WebSocketTransport::new(config);

    // Create a test message
    use crate::protocol::types::{MCPMessageId, MCPVersion};
    let test_message = MCPMessage {
        jsonrpc: MCPVersion::V2_0,
        id: MCPMessageId(1),
        method: Some("test.method".to_string()),
        params: None,
        result: None,
        error: None,
    };

    // Transport is disconnected, message should be buffered
    assert!(!transport.is_connected().await);

    let buffer_result = transport.buffer_message(test_message.clone()).await;
    assert!(
        buffer_result.is_ok(),
        "Should buffer message when disconnected"
    );

    // Verify message is in buffer
    {
        let buffer = transport.message_buffer.lock().await;
        assert_eq!(buffer.len(), 1, "Buffer should contain 1 message");
    }

    // Buffer multiple messages
    for i in 2..10 {
        let msg = MCPMessage {
            jsonrpc: MCPVersion::V2_0,
            id: MCPMessageId(i),
            method: Some("test.method".to_string()),
            params: None,
            result: None,
            error: None,
        };
        let _ = transport.buffer_message(msg).await;
    }

    // Verify buffer contains all messages
    {
        let buffer = transport.message_buffer.lock().await;
        assert_eq!(buffer.len(), 9, "Buffer should contain 9 messages");
    }
}

#[tokio::test]
async fn test_websocket_buffer_overflow() {
    // Create config
    let config = WebSocketConfig {
        url: "ws://localhost:8080".to_string(),
        ..Default::default()
    };

    let transport = WebSocketTransport::new(config);

    // Create test message
    use crate::protocol::types::{MCPMessageId, MCPVersion};
    let test_message = MCPMessage {
        jsonrpc: MCPVersion::V2_0,
        id: MCPMessageId(1),
        method: Some("test.method".to_string()),
        params: None,
        result: None,
        error: None,
    };

    // Buffer 1001 messages (buffer max is 1000)
    for i in 0..1001 {
        let msg = MCPMessage {
            jsonrpc: MCPVersion::V2_0,
            id: MCPMessageId(i),
            method: Some("test.method".to_string()),
            params: None,
            result: None,
            error: None,
        };
        let _ = transport.buffer_message(msg).await;
    }

    // Buffer should be capped at 1000 (oldest dropped)
    {
        let buffer = transport.message_buffer.lock().await;
        assert_eq!(buffer.len(), 1000, "Buffer should be capped at 1000");

        // First message should be id=1 (oldest was id=0, dropped)
        if let Some(first_msg) = buffer.first() {
            assert_eq!(first_msg.id.0, 1, "Oldest message should be dropped");
        }
    }
}

#[tokio::test]
async fn test_websocket_reconnection_counter() {
    // Create config with specific reconnection settings
    let config = WebSocketConfig {
        url: "ws://localhost:9999".to_string(), // Invalid port to force failure
        max_reconnect_attempts: 3,
        reconnect_delay_ms: 10, // Fast for testing
        ..Default::default()
    };

    let transport = WebSocketTransport::new(config);

    // Verify initial counter is 0
    {
        let attempts = transport.reconnection_attempts.lock().await;
        assert_eq!(*attempts, 0, "Initial reconnection attempts should be 0");
    }

    // Note: We can't test actual reconnection without a running WebSocket server
    // This test verifies the structure is in place
    // Full reconnection testing would be done in integration tests
}

#[tokio::test]
async fn test_websocket_keepalive_configuration() {
    // Test with keepalive enabled
    let config_with_keepalive = WebSocketConfig {
        url: "ws://localhost:8080".to_string(),
        ping_interval: Some(30), // 30 second ping interval
        ..Default::default()
    };

    let transport = WebSocketTransport::new(config_with_keepalive);
    assert!(
        transport.config.ping_interval.is_some(),
        "Keepalive should be enabled"
    );
    assert_eq!(transport.config.ping_interval.unwrap(), 30);

    // Test with keepalive disabled
    let config_without_keepalive = WebSocketConfig {
        url: "ws://localhost:8080".to_string(),
        ping_interval: None, // No keepalive
        ..Default::default()
    };

    let transport = WebSocketTransport::new(config_without_keepalive);
    assert!(
        transport.config.ping_interval.is_none(),
        "Keepalive should be disabled"
    );
}
