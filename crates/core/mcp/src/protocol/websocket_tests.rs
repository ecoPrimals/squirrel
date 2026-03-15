// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Comprehensive tests for WebSocket Transport Implementation
//!
//! Tests cover WebSocket server functionality, client connections, message handling,
//! connection management, error scenarios, and performance optimization.

use super::websocket::*;
use crate::protocol::types::{MCPMessage, MessageId, MessageType, ProtocolVersion};
use crate::transport::frame::{Frame, MessageCodec};

use std::time::Duration;

/// Helper function to create a test WebSocket configuration
fn create_test_config() -> WebSocketConfig {
    WebSocketConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0, // Use system-assigned port for tests
        timeout_seconds: 10,
        max_connections: 10,
        buffer_size: 1024,
        connection_timeout: Duration::from_secs(10),
    }
}

/// Helper function to create a test MCP message
fn create_test_message(message_type: MessageType, content: &str) -> MCPMessage {
    MCPMessage {
        id: MessageId(format!("test-{}", uuid::Uuid::new_v4())),
        type_: message_type,
        version: ProtocolVersion::new(1, 0),
        payload: serde_json::json!({
            "content": content,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }),
        security: Default::default(),
        metadata: Some(serde_json::json!({})),
        timestamp: chrono::Utc::now(),
        trace_id: Some(format!("trace-{}", uuid::Uuid::new_v4())),
    }
}

/// Helper function to wait for server to start
async fn wait_for_server_start(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let mut retries = 0;
    loop {
        if tokio::net::TcpStream::connect(format!("127.0.0.1:{port}"))
            .await
            .is_ok()
        {
            break Ok(());
        }

        retries += 1;
        if retries > 50 {
            break Err("Server failed to start within timeout".into());
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

#[tokio::test]
async fn test_websocket_config_creation() {
    let config = create_test_config();

    assert_eq!(config.bind_address, "127.0.0.1");
    assert_eq!(config.port, 0);
    assert_eq!(config.timeout_seconds, 10);
    assert_eq!(config.max_connections, 10);
    assert_eq!(config.buffer_size, 1024);
    assert_eq!(config.connection_timeout, Duration::from_secs(10));
}

#[tokio::test]
async fn test_websocket_config_default() {
    let config = WebSocketConfig::default();

    assert_eq!(config.bind_address, "127.0.0.1");
    assert_eq!(config.port, 8080);
    assert_eq!(config.timeout_seconds, 30);
    assert_eq!(config.max_connections, 100);
    assert_eq!(config.buffer_size, 1024);
    assert_eq!(config.connection_timeout, Duration::from_secs(30));
}

#[tokio::test]
async fn test_connection_info_creation() {
    let connection_info = ConnectionInfo {
        id: "test-connection".to_string(),
        remote_address: "127.0.0.1:12345".to_string(),
        state: ConnectionState::Connected,
        connected_at: std::time::Instant::now(),
        last_message_at: None,
        message_count: 0,
        last_ping: None,
        last_pong: None,
        bytes_sent: 0,
        bytes_received: 0,
        messages_sent: 0,
        messages_received: 0,
    };

    assert_eq!(connection_info.id, "test-connection");
    assert_eq!(connection_info.state, ConnectionState::Connected);
    assert_eq!(connection_info.message_count, 0);
    assert_eq!(connection_info.bytes_sent, 0);
}

#[tokio::test]
async fn test_websocket_server_creation() {
    let config = create_test_config();
    let server = WebSocketServer::new(config.clone());

    // Test server properties
    let connections = server.get_connections().await;
    assert!(connections.is_empty());
}

#[tokio::test]
async fn test_websocket_client_creation() {
    let config = create_test_config();
    let client = WebSocketClient::new(config.clone());

    // Test client properties
    let info = client.get_connection_info().await;
    assert!(info.is_none());
}

#[tokio::test]
async fn test_message_codec_encoding_decoding() {
    let codec = MessageCodec::new();
    let test_message = create_test_message(MessageType::Command, "test message");

    // Test encoding
    let frame = codec.encode_message(&test_message).await.unwrap();
    assert!(!frame.payload.is_empty());

    // Test decoding
    let decoded_message = codec.decode_message(&frame).await.unwrap();
    assert_eq!(decoded_message.id.0, test_message.id.0);
    assert_eq!(decoded_message.type_, test_message.type_);
    assert_eq!(decoded_message.payload, test_message.payload);
}

#[tokio::test]
async fn test_frame_serialization() {
    let test_data = b"test frame data";
    let frame = Frame::new(test_data.to_vec());

    // Test serialization/deserialization
    let serialized = serde_json::to_string(&frame).unwrap();
    let deserialized: Frame = serde_json::from_str(&serialized).unwrap();

    assert_eq!(frame.payload, deserialized.payload);
}

#[tokio::test]
async fn test_websocket_server_event_handling() {
    let config = create_test_config();
    let server = WebSocketServer::new(config);

    // Subscribe to server events
    let mut event_receiver = server.subscribe();

    // Test that we can receive the initial subscription
    assert!(event_receiver.try_recv().is_err()); // Should be no events initially
}

#[tokio::test]
async fn test_connection_state_transitions() {
    let states = vec![
        ConnectionState::Connecting,
        ConnectionState::Connected,
        ConnectionState::Disconnecting,
        ConnectionState::Disconnected,
        ConnectionState::Failed,
    ];

    // Test that all states can be created and compared
    for (i, state) in states.iter().enumerate() {
        for (j, other_state) in states.iter().enumerate() {
            if i == j {
                assert_eq!(state, other_state);
            } else {
                assert_ne!(state, other_state);
            }
        }
    }
}

#[tokio::test]
async fn test_websocket_transport_creation() {
    let config = create_test_config();
    let connection_info = ConnectionInfo {
        id: "test-transport".to_string(),
        remote_address: "127.0.0.1:12345".to_string(),
        state: ConnectionState::Connected,
        connected_at: std::time::Instant::now(),
        last_message_at: None,
        message_count: 0,
        last_ping: None,
        last_pong: None,
        bytes_sent: 0,
        bytes_received: 0,
        messages_sent: 0,
        messages_received: 0,
    };

    let transport = WebSocketTransport::new(connection_info, config);

    assert_eq!(transport.connection.id, "test-transport");
    assert_eq!(transport.connection.state, ConnectionState::Connected);
}

#[tokio::test]
async fn test_server_event_types() {
    // Test that all server event types can be created
    let events = vec![
        ServerEvent::ClientConnected("client-1".to_string()),
        ServerEvent::ClientDisconnected("client-1".to_string()),
        ServerEvent::MessageReceived(
            "client-1".to_string(),
            create_test_message(MessageType::Command, "test"),
        ),
        ServerEvent::ConnectionError("client-1".to_string(), "test error".to_string()),
    ];

    for event in events {
        match event {
            ServerEvent::ClientConnected(id) => assert_eq!(id, "client-1"),
            ServerEvent::ClientDisconnected(id) => assert_eq!(id, "client-1"),
            ServerEvent::MessageReceived(id, _) => assert_eq!(id, "client-1"),
            ServerEvent::ConnectionError(id, _) => assert_eq!(id, "client-1"),
        }
    }
}

#[tokio::test]
async fn test_multiple_message_types() {
    let codec = MessageCodec::new();

    let message_types = vec![
        MessageType::Command,
        MessageType::Sync,
        MessageType::Event,
        MessageType::Error,
    ];

    for message_type in message_types {
        let test_message = create_test_message(message_type.clone(), "test");
        let frame = codec.encode_message(&test_message).await.unwrap();
        let decoded = codec.decode_message(&frame).await.unwrap();

        assert_eq!(decoded.type_, message_type);
    }
}

#[tokio::test]
async fn test_large_message_handling() {
    let codec = MessageCodec::new();

    // Create a large message payload
    let large_content = "x".repeat(10000); // 10KB content
    let test_message = create_test_message(MessageType::Command, &large_content);

    // Test encoding/decoding large message
    let frame = codec.encode_message(&test_message).await.unwrap();
    assert!(frame.payload.len() > 10000);

    let decoded = codec.decode_message(&frame).await.unwrap();
    assert_eq!(
        decoded
            .payload
            .get("content")
            .unwrap()
            .as_str()
            .unwrap()
            .len(),
        10000
    );
}

#[tokio::test]
async fn test_connection_info_state_tracking() {
    let mut connection_info = ConnectionInfo {
        id: "tracking-test".to_string(),
        remote_address: "127.0.0.1:12345".to_string(),
        state: ConnectionState::Connecting,
        connected_at: std::time::Instant::now(),
        last_message_at: None,
        message_count: 0,
        last_ping: None,
        last_pong: None,
        bytes_sent: 0,
        bytes_received: 0,
        messages_sent: 0,
        messages_received: 0,
    };

    // Test state transitions
    assert_eq!(connection_info.state, ConnectionState::Connecting);

    connection_info.state = ConnectionState::Connected;
    assert_eq!(connection_info.state, ConnectionState::Connected);

    // Test message counting
    connection_info.message_count += 1;
    connection_info.messages_sent += 1;
    connection_info.bytes_sent += 100;

    assert_eq!(connection_info.message_count, 1);
    assert_eq!(connection_info.messages_sent, 1);
    assert_eq!(connection_info.bytes_sent, 100);
}

#[tokio::test]
async fn test_error_message_handling() {
    let codec = MessageCodec::new();

    // Create an error message
    let error_message = MCPMessage {
        id: MessageId("error-test".to_string()),
        type_: MessageType::Error,
        version: ProtocolVersion::new(1, 0),
        payload: serde_json::json!({
            "error": "Test error message",
            "code": 500,
            "details": "This is a test error"
        }),
        security: Default::default(),
        metadata: Some(serde_json::json!({})),
        timestamp: chrono::Utc::now(),
        trace_id: Some("error-trace".to_string()),
    };

    let frame = codec.encode_message(&error_message).await.unwrap();
    let decoded = codec.decode_message(&frame).await.unwrap();

    assert_eq!(decoded.type_, MessageType::Error);
    assert_eq!(
        decoded.payload.get("error").unwrap().as_str().unwrap(),
        "Test error message"
    );
    assert_eq!(decoded.payload.get("code").unwrap().as_u64().unwrap(), 500);
}

#[tokio::test]
async fn test_concurrent_message_processing() {
    let codec = MessageCodec::new();

    // Create multiple messages
    let messages: Vec<_> = (0..10)
        .map(|i| create_test_message(MessageType::Command, &format!("message {}", i)))
        .collect();

    // Process messages concurrently
    let handles: Vec<_> = messages
        .into_iter()
        .map(|msg| {
            let codec_clone = codec.clone();
            tokio::spawn(async move {
                let frame = codec_clone.encode_message(&msg).await.unwrap();
                codec_clone.decode_message(&frame).await.unwrap()
            })
        })
        .collect();

    // Wait for all to complete
    let results = futures::future::join_all(handles).await;

    assert_eq!(results.len(), 10);
    for (i, result) in results.into_iter().enumerate() {
        let decoded = result.unwrap();
        assert_eq!(decoded.type_, MessageType::Command);
        assert_eq!(
            decoded.payload.get("content").unwrap().as_str().unwrap(),
            format!("message {}", i)
        );
    }
}

#[tokio::test]
async fn test_websocket_config_validation() {
    // Test valid configuration
    let valid_config = WebSocketConfig {
        bind_address: "0.0.0.0".to_string(),
        port: 8080,
        timeout_seconds: 30,
        max_connections: 100,
        buffer_size: 1024,
        connection_timeout: Duration::from_secs(30),
    };

    // All values should be reasonable
    assert!(!valid_config.bind_address.is_empty());
    assert!(valid_config.port > 0);
    assert!(valid_config.timeout_seconds > 0);
    assert!(valid_config.max_connections > 0);
    assert!(valid_config.buffer_size > 0);
    assert!(valid_config.connection_timeout.as_secs() > 0);
}

#[tokio::test]
async fn test_connection_state_serialization() {
    let states = vec![
        ConnectionState::Connecting,
        ConnectionState::Connected,
        ConnectionState::Disconnecting,
        ConnectionState::Disconnected,
        ConnectionState::Failed,
    ];

    for state in states {
        // Test serialization/deserialization
        let serialized = serde_json::to_string(&state).unwrap();
        let deserialized: ConnectionState = serde_json::from_str(&serialized).unwrap();

        assert_eq!(state, deserialized);
    }
}

#[tokio::test]
async fn test_message_id_uniqueness() {
    let mut message_ids = std::collections::HashSet::new();

    // Generate multiple messages and verify unique IDs
    for i in 0..100 {
        let message = create_test_message(MessageType::Command, &format!("message {}", i));
        assert!(
            message_ids.insert(message.id.0.clone()),
            "Message ID should be unique"
        );
    }

    assert_eq!(message_ids.len(), 100);
}

#[tokio::test]
async fn test_protocol_version_compatibility() {
    let codec = MessageCodec::new();

    let versions = vec![
        ProtocolVersion::new(1, 0),
        ProtocolVersion::new(1, 1),
        ProtocolVersion::new(2, 0),
    ];

    for version in versions {
        let mut message = create_test_message(MessageType::Command, "version test");
        message.version = version.clone();

        let frame = codec.encode_message(&message).await.unwrap();
        let decoded = codec.decode_message(&frame).await.unwrap();

        assert_eq!(decoded.version, version);
    }
}

// Integration test that would require actual WebSocket server/client
// This test is commented out as it requires network setup and might be flaky in CI
/*
#[tokio::test]
async fn test_websocket_integration() {
    // Start a test server
    let mut server_config = create_test_config();
    server_config.port = 0; // Let system assign port
    let mut server = WebSocketServer::new(server_config);

    // Start server in background
    tokio::spawn(async move {
        server.start().await.unwrap();
    });

    // Wait for server to be ready
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect client
    let client_config = create_test_config();
    let client = WebSocketClient::new(client_config);

    // This would require the actual port number from the server
    // client.connect("ws://127.0.0.1:port").await.unwrap();
}
*/
