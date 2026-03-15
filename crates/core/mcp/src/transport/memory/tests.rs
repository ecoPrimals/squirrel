// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use crate::transport::Transport;
use tokio::time::timeout;
use std::time::Duration;
use crate::types::MessageType;

#[tokio::test]
async fn test_memory_transport_create() {
    // Create a channel
    let channel = MemoryChannel::new(100, Some(10));
    
    // Create config
    let config = MemoryTransportConfig {
        name: "test".to_string(),
        ..Default::default()
    };
    
    // Create a transport
    let transport = channel.create_transport(config);
    
    // Verify initial state
    assert!(!transport.is_connected().await);
    assert_eq!(transport.get_metadata().transport_type, "memory");
}

#[tokio::test]
async fn test_memory_channel_create_pair() {
    // Create a channel with pair
    let (transport_a, transport_b) = MemoryChannel::create_pair();
    
    // Verify initial state
    assert!(!transport_a.is_connected().await);
    assert!(!transport_b.is_connected().await);
    
    // Get metadata
    let metadata_a = transport_a.get_metadata();
    let metadata_b = transport_b.get_metadata();
    
    assert_eq!(metadata_a.transport_type, "memory");
    assert_eq!(metadata_b.transport_type, "memory");
}

#[tokio::test]
async fn test_memory_channel_create_pair_arc() {
    // Create a pair of Arc-wrapped transports using our new method
    let (client, server) = MemoryChannel::create_pair_arc();
    
    // Connect both sides
    client.connect().await.unwrap();
    server.connect().await.unwrap();
    
    // Check that both are connected
    assert!(client.is_connected().await);
    assert!(server.is_connected().await);
    
    // Test message sending/receiving
    let client_msg = crate::types::MCPMessage::new(
        crate::types::MessageType::Command,
        serde_json::json!({ "action": "test_arc_pair" }),
    );
    
    // Send message from client to server
    client.send_message(client_msg.clone()).await.unwrap();
    
    // Receive on server side with timeout to prevent hanging
    let received = timeout(
        Duration::from_secs(1),
        server.receive_message()
    ).await.unwrap().unwrap();
    
    // Verify message contents
    assert_eq!(received.id.0, client_msg.id.0);
    assert_eq!(received.type_, client_msg.type_);
    assert_eq!(
        received.payload.get("action").and_then(|v| v.as_str()),
        Some("test_arc_pair")
    );
    
    // Test in the other direction
    let server_msg = crate::types::MCPMessage::new(
        crate::types::MessageType::Response,
        serde_json::json!({ "status": "success" }),
    );
    
    // Send message from server to client
    server.send_message(server_msg.clone()).await.unwrap();
    
    // Receive on client side
    let received = timeout(
        Duration::from_secs(1),
        client.receive_message()
    ).await.unwrap().unwrap();
    
    // Verify message contents
    assert_eq!(received.id.0, server_msg.id.0);
    assert_eq!(received.type_, server_msg.type_);
    assert_eq!(
        received.payload.get("status").and_then(|v| v.as_str()),
        Some("success")
    );
    
    // Test disconnect
    client.disconnect().await.unwrap();
    server.disconnect().await.unwrap();
    
    // Verify disconnected state
    assert!(!client.is_connected().await);
    assert!(!server.is_connected().await);
}

#[tokio::test]
async fn test_memory_transport_creation_directly() {
    // Create a pair of transports without Arc wrapping
    let (mut client, mut server) = MemoryChannel::create_pair();
    
    // Connect both sides
    client.connect().await.unwrap();
    server.connect().await.unwrap();
    
    // Verify connected state
    assert!(client.is_connected().await);
    assert!(server.is_connected().await);
    
    // Get metadata and verify
    let client_meta = client.get_metadata();
    let server_meta = server.get_metadata();
    
    assert_eq!(client_meta.transport_type, "memory");
    assert_eq!(server_meta.transport_type, "memory");
    assert!(client_meta.peer_addr.is_none());
    assert!(server_meta.peer_addr.is_none());
    
    // Disconnect and verify
    client.disconnect().await.unwrap();
    server.disconnect().await.unwrap();
    
    assert!(!client.is_connected().await);
    assert!(!server.is_connected().await);
}

#[tokio::test]
async fn test_message_history() {
    // Create channel directly to test history
    let buffer_size = 100;
    let max_history = Some(5); // Small history for testing
    let channel = MemoryChannel::new(buffer_size, max_history);
    
    // Create transport pair
    let config_a = MemoryTransportConfig {
        name: "history-test-client".to_string(),
        ..Default::default()
    };
    
    let config_b = MemoryTransportConfig {
        name: "history-test-server".to_string(),
        ..Default::default()
    };
    
    let (mut client, mut server) = channel.create_transport_pair(Some(config_a), Some(config_b));
    
    // Connect both sides
    client.connect().await.unwrap();
    server.connect().await.unwrap();
    
    // Send multiple messages to test history
    for i in 0..10 {
        let msg = crate::types::MCPMessage::new(
            crate::types::MessageType::Command,
            serde_json::json!({ "sequence": i }),
        );
        
        client.send_message(msg).await.unwrap();
        
        // Receive on the other end to keep channels clear
        let _ = server.receive_message().await.unwrap();
    }
    
    // Verify history size is limited by max_history
    let history = channel.get_history().await;
    assert_eq!(history.len(), max_history.unwrap());
    
    // Verify history contains only the most recent messages
    let seq_values: Vec<i64> = history.iter()
        .filter_map(|msg| msg.payload.get("sequence")?.as_i64())
        .collect();
    
    // Should have messages 5-9 (most recent 5)
    assert_eq!(seq_values, vec![5, 6, 7, 8, 9]);
    
    // Test history clearing
    channel.clear_history().await;
    let empty_history = channel.get_history().await;
    assert_eq!(empty_history.len(), 0);
}

#[tokio::test]
async fn test_memory_transport_pair() {
    // Create a channel
    let channel = MemoryChannel::new(100, Some(10));
    
    // Create config for both sides
    let config_a = MemoryTransportConfig {
        name: "client".to_string(),
        ..Default::default()
    };
    
    let config_b = MemoryTransportConfig {
        name: "server".to_string(),
        ..Default::default()
    };
    
    // Create a pair of transports
    let channel_clone = channel.clone();
    let (client, server) = channel_clone.create_transport_pair(Some(config_a), Some(config_b));
    
    // Connect both sides
    client.connect().await.unwrap();
    server.connect().await.unwrap();
    
    // Check that both are connected
    assert!(client.is_connected().await);
    assert!(server.is_connected().await);
    
    // Send message from client to server
    let client_msg = MCPMessage::new(
        MessageType::Command,
        serde_json::json!({ "action": "test" }),
    );
    
    client.send_message(client_msg.clone()).await.unwrap();
    
    // Receive on server side
    let received = tokio::time::timeout(
        std::time::Duration::from_secs(1), 
        server.receive_message()
    ).await.unwrap().unwrap();
    
    // Verify message contents
    assert_eq!(received.id.0, client_msg.id.0);
    assert_eq!(received.type_, client_msg.type_);
    
    // Send response
    let server_msg = MCPMessage::new(
        MessageType::Response,
        serde_json::json!({ "result": "ok" }),
    );
    
    server.send_message(server_msg.clone()).await.unwrap();
    
    // Receive on client side
    let received = tokio::time::timeout(
        std::time::Duration::from_secs(1), 
        client.receive_message()
    ).await.unwrap().unwrap();
    
    // Verify message contents
    assert_eq!(received.id.0, server_msg.id.0);
    assert_eq!(received.type_, server_msg.type_);
    
    // Check history (which should have both messages)
    let history = channel.get_history().await;
    assert_eq!(history.len(), 2);
}

#[tokio::test]
async fn test_memory_transport_with_latency() {
    // Create a channel
    let channel = MemoryChannel::new(100, Some(10));
    
    // Create config for both sides
    let config_a = MemoryTransportConfig {
        name: "client".to_string(),
        simulated_latency_ms: Some(100),
        ..Default::default()
    };
    
    let config_b = MemoryTransportConfig {
        name: "server".to_string(),
        ..Default::default()
    };
    
    // Create a pair of transports
    let channel_clone = channel.clone();
    let (client, server) = channel_clone.create_transport_pair(Some(config_a), Some(config_b));
    
    // Connect both sides
    client.connect().await.unwrap();
    server.connect().await.unwrap();
    
    // Send message from client to server (should have latency)
    let start = tokio::time::Instant::now();
    let client_msg = MCPMessage::new(
        MessageType::Command,
        serde_json::json!({ "action": "test" }),
    );
    
    client.send_message(client_msg.clone()).await.unwrap();
    
    // Receive on server side
    let _ = server.receive_message().await.unwrap();
    let elapsed = start.elapsed();
    
    // Should take at least the simulated latency
    assert!(elapsed >= tokio::time::Duration::from_millis(100));
} 