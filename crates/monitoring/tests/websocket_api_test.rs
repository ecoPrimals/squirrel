use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use serde_json::Value;
use serde_json::json;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use squirrel_monitoring::websocket::{WebSocketConfig, server::WebSocketServer};
use squirrel_core::error::Result;

/// Helper function to set up a test WebSocket server
async fn setup_test_server() -> Result<Arc<WebSocketServer>> {
    let config = WebSocketConfig {
        host: "127.0.0.1".to_string(),
        port: 9876, // Use a test-specific port
        update_interval: 1,
        max_connections: 100,
        enable_compression: true,
        auth_required: false,
    };
    
    let server = Arc::new(WebSocketServer::new(config));
    server.start().await?;
    
    // Allow time for server to start
    time::sleep(Duration::from_millis(100)).await;
    
    Ok(server)
}

/// Test basic WebSocket connection
#[tokio::test]
async fn test_websocket_connection() -> Result<()> {
    let server = setup_test_server().await?;
    
    // Connect to the WebSocket server
    let url = format!("ws://{}:{}/ws", "127.0.0.1", 9876);
    let (ws_stream, _) = connect_async(url).await
        .map_err(|e| squirrel_core::error::SquirrelError::Generic(
            format!("Failed to connect: {}", e)
        ))?;
    
    // Split the WebSocket stream
    let (mut write, mut read) = ws_stream.split();
    
    // Send a ping message
    let ping_message = json!({
        "action": "ping"
    }).to_string();
    
    write.send(Message::Text(ping_message)).await
        .map_err(|e| squirrel_core::error::SquirrelError::Generic(
            format!("Failed to send: {}", e)
        ))?;
    
    // Cleanup
    server.stop().await?;
    
    Ok(())
}

/// Test topic subscription and message delivery
#[tokio::test]
async fn test_websocket_subscription() -> Result<()> {
    let server = setup_test_server().await?;
    
    // Connect to the WebSocket server
    let url = format!("ws://{}:{}/ws", "127.0.0.1", 9876);
    let (ws_stream, _) = connect_async(url).await
        .map_err(|e| squirrel_core::error::SquirrelError::Generic(
            format!("Failed to connect: {}", e)
        ))?;
    
    // Split the WebSocket stream
    let (mut write, mut read) = ws_stream.split();
    
    // Subscribe to a topic
    let test_topic = "test_topic";
    let subscribe_message = json!({
        "action": "subscribe",
        "topic": test_topic
    }).to_string();
    
    write.send(Message::Text(subscribe_message)).await
        .map_err(|e| squirrel_core::error::SquirrelError::Generic(
            format!("Failed to send: {}", e)
        ))?;
    
    // Allow time for subscription to be processed
    time::sleep(Duration::from_millis(100)).await;
    
    // Update component data
    let test_data = json!({
        "value": 42,
        "timestamp": chrono::Utc::now().timestamp()
    });
    
    server.update_component_data(test_topic, test_data.clone()).await?;
    
    // Receive the message
    let message = tokio::time::timeout(Duration::from_secs(2), read.next()).await
        .map_err(|_| squirrel_core::error::SquirrelError::Generic(
            "Timeout waiting for message".to_string()
        ))?
        .ok_or_else(|| squirrel_core::error::SquirrelError::Generic(
            "No message received".to_string()
        ))?
        .map_err(|e| squirrel_core::error::SquirrelError::Generic(
            format!("WebSocket error: {}", e)
        ))?;
    
    if let Message::Text(text) = message {
        let received: Value = serde_json::from_str(&text)
            .map_err(|e| squirrel_core::error::SquirrelError::Generic(
                format!("Invalid JSON: {}", e)
            ))?;
        
        assert_eq!(received["topic"], test_topic);
        assert_eq!(received["payload"]["value"], 42);
    } else {
        return Err(squirrel_core::error::SquirrelError::Generic(
            format!("Expected text message, got: {:?}", message)
        ));
    }
    
    // Cleanup
    server.stop().await?;
    
    Ok(())
}

/// Test unsubscribing from a topic
#[tokio::test]
async fn test_websocket_unsubscribe() -> Result<()> {
    let server = setup_test_server().await?;
    
    // Connect to the WebSocket server
    let url = format!("ws://{}:{}/ws", "127.0.0.1", 9876);
    let (ws_stream, _) = connect_async(url).await
        .map_err(|e| squirrel_core::error::SquirrelError::Generic(
            format!("Failed to connect: {}", e)
        ))?;
    
    // Split the WebSocket stream
    let (mut write, mut read) = ws_stream.split();
    
    // Subscribe to a topic
    let test_topic = "test_topic";
    let subscribe_message = json!({
        "action": "subscribe",
        "topic": test_topic
    }).to_string();
    
    write.send(Message::Text(subscribe_message)).await
        .map_err(|e| squirrel_core::error::SquirrelError::Generic(
            format!("Failed to send: {}", e)
        ))?;
    
    // Allow time for subscription to be processed
    time::sleep(Duration::from_millis(100)).await;
    
    // Update component data to verify subscription works
    let test_data = json!({
        "value": 42,
        "timestamp": chrono::Utc::now().timestamp()
    });
    
    server.update_component_data(test_topic, test_data.clone()).await?;
    
    // Receive the message to confirm subscription is working
    let message = tokio::time::timeout(Duration::from_secs(2), read.next()).await
        .map_err(|_| squirrel_core::error::SquirrelError::Generic(
            "Timeout waiting for message".to_string()
        ))?
        .ok_or_else(|| squirrel_core::error::SquirrelError::Generic(
            "No message received".to_string()
        ))?
        .map_err(|e| squirrel_core::error::SquirrelError::Generic(
            format!("WebSocket error: {}", e)
        ))?;
    
    // Unsubscribe from the topic
    let unsubscribe_message = json!({
        "action": "unsubscribe",
        "topic": test_topic
    }).to_string();
    
    write.send(Message::Text(unsubscribe_message)).await
        .map_err(|e| squirrel_core::error::SquirrelError::Generic(
            format!("Failed to send: {}", e)
        ))?;
    
    // Allow time for unsubscription to be processed
    time::sleep(Duration::from_millis(100)).await;
    
    // Update component data again
    let new_test_data = json!({
        "value": 100,
        "timestamp": chrono::Utc::now().timestamp()
    });
    
    server.update_component_data(test_topic, new_test_data.clone()).await?;
    
    // Try to receive a message, should timeout since we're unsubscribed
    let timeout_result = tokio::time::timeout(Duration::from_millis(500), read.next()).await;
    assert!(timeout_result.is_err(), "Expected timeout after unsubscribing");
    
    // Cleanup
    server.stop().await?;
    
    Ok(())
}

/// Test multiple subscriptions
#[tokio::test]
async fn test_multiple_subscriptions() -> Result<()> {
    let server = setup_test_server().await?;
    
    // Connect to the WebSocket server
    let url = format!("ws://{}:{}/ws", "127.0.0.1", 9876);
    let (ws_stream, _) = connect_async(url).await
        .map_err(|e| squirrel_core::error::SquirrelError::Generic(
            format!("Failed to connect: {}", e)
        ))?;
    
    // Split the WebSocket stream
    let (mut write, mut read) = ws_stream.split();
    
    // Subscribe to multiple topics
    let topics = vec!["topic1", "topic2", "topic3"];
    
    for topic in &topics {
        let subscribe_message = json!({
            "action": "subscribe",
            "topic": topic
        }).to_string();
        
        write.send(Message::Text(subscribe_message)).await
            .map_err(|e| squirrel_core::error::SquirrelError::Generic(
                format!("Failed to send: {}", e)
            ))?;
        
        // Allow time for subscription to be processed
        time::sleep(Duration::from_millis(50)).await;
    }
    
    // Update component data for each topic
    for (i, topic) in topics.iter().enumerate() {
        let test_data = json!({
            "value": i + 1,
            "timestamp": chrono::Utc::now().timestamp()
        });
        
        server.update_component_data(topic, test_data.clone()).await?;
    }
    
    // Receive messages for each topic
    let mut received_topics = Vec::new();
    
    for _ in 0..topics.len() {
        let message = tokio::time::timeout(Duration::from_secs(2), read.next()).await
            .map_err(|_| squirrel_core::error::SquirrelError::Generic(
                "Timeout waiting for message".to_string()
            ))?
            .ok_or_else(|| squirrel_core::error::SquirrelError::Generic(
                "No message received".to_string()
            ))?
            .map_err(|e| squirrel_core::error::SquirrelError::Generic(
                format!("WebSocket error: {}", e)
            ))?;
        
        if let Message::Text(text) = message {
            let received: Value = serde_json::from_str(&text)
                .map_err(|e| squirrel_core::error::SquirrelError::Generic(
                    format!("Invalid JSON: {}", e)
                ))?;
            
            received_topics.push(received["topic"].as_str().unwrap().to_string());
        }
    }
    
    // Verify we received messages for all topics
    for topic in topics {
        assert!(received_topics.contains(&topic.to_string()), "Missing message for topic: {}", topic);
    }
    
    // Cleanup
    server.stop().await?;
    
    Ok(())
}

/// Test multiple clients
#[tokio::test]
async fn test_multiple_clients() -> Result<()> {
    let server = setup_test_server().await?;
    
    // Connect multiple clients
    const CLIENT_COUNT: usize = 5;
    let url = format!("ws://{}:{}/ws", "127.0.0.1", 9876);
    
    let mut clients = Vec::with_capacity(CLIENT_COUNT);
    
    for _ in 0..CLIENT_COUNT {
        let (ws_stream, _) = connect_async(&url).await
            .map_err(|e| squirrel_core::error::SquirrelError::Generic(
                format!("Failed to connect: {}", e)
            ))?;
        
        let (write, read) = ws_stream.split();
        clients.push((write, read));
    }
    
    // Subscribe all clients to the same topic
    let test_topic = "shared_topic";
    
    for (mut write, _) in &mut clients {
        let subscribe_message = json!({
            "action": "subscribe",
            "topic": test_topic
        }).to_string();
        
        write.send(Message::Text(subscribe_message)).await
            .map_err(|e| squirrel_core::error::SquirrelError::Generic(
                format!("Failed to send: {}", e)
            ))?;
    }
    
    // Allow time for subscriptions to be processed
    time::sleep(Duration::from_millis(100)).await;
    
    // Update component data
    let test_data = json!({
        "value": 42,
        "timestamp": chrono::Utc::now().timestamp()
    });
    
    server.update_component_data(test_topic, test_data.clone()).await?;
    
    // Verify all clients receive the message
    for (i, (_, mut read)) in clients.iter_mut().enumerate() {
        let message = tokio::time::timeout(Duration::from_secs(2), read.next()).await
            .map_err(|_| squirrel_core::error::SquirrelError::Generic(
                format!("Timeout waiting for message for client {}", i)
            ))?
            .ok_or_else(|| squirrel_core::error::SquirrelError::Generic(
                format!("No message received for client {}", i)
            ))?
            .map_err(|e| squirrel_core::error::SquirrelError::Generic(
                format!("WebSocket error for client {}: {}", i, e)
            ))?;
        
        if let Message::Text(text) = message {
            let received: Value = serde_json::from_str(&text)
                .map_err(|e| squirrel_core::error::SquirrelError::Generic(
                    format!("Invalid JSON for client {}: {}", i, e)
                ))?;
            
            assert_eq!(received["topic"], test_topic);
            assert_eq!(received["payload"]["value"], 42);
        } else {
            return Err(squirrel_core::error::SquirrelError::Generic(
                format!("Expected text message for client {}, got: {:?}", i, message)
            ));
        }
    }
    
    // Cleanup
    server.stop().await?;
    
    Ok(())
}

/// Test high-frequency updates
#[tokio::test]
async fn test_high_frequency_updates() -> Result<()> {
    let server = setup_test_server().await?;
    
    // Connect to the WebSocket server
    let url = format!("ws://{}:{}/ws", "127.0.0.1", 9876);
    let (ws_stream, _) = connect_async(url).await
        .map_err(|e| squirrel_core::error::SquirrelError::Generic(
            format!("Failed to connect: {}", e)
        ))?;
    
    // Split the WebSocket stream
    let (mut write, mut read) = ws_stream.split();
    
    // Subscribe to a topic
    let test_topic = "high_frequency_topic";
    let subscribe_message = json!({
        "action": "subscribe",
        "topic": test_topic
    }).to_string();
    
    write.send(Message::Text(subscribe_message)).await
        .map_err(|e| squirrel_core::error::SquirrelError::Generic(
            format!("Failed to send: {}", e)
        ))?;
    
    // Allow time for subscription to be processed
    time::sleep(Duration::from_millis(100)).await;
    
    // Send multiple updates in quick succession
    const UPDATE_COUNT: usize = 10;
    
    for i in 0..UPDATE_COUNT {
        let test_data = json!({
            "value": i,
            "timestamp": chrono::Utc::now().timestamp()
        });
        
        server.update_component_data(test_topic, test_data.clone()).await?;
    }
    
    // Receive messages and count them
    let mut received_count = 0;
    let timeout = Duration::from_secs(5);
    let start = std::time::Instant::now();
    
    while received_count < UPDATE_COUNT && start.elapsed() < timeout {
        match tokio::time::timeout(Duration::from_millis(500), read.next()).await {
            Ok(Some(Ok(Message::Text(_)))) => {
                received_count += 1;
            },
            Ok(Some(Err(e))) => {
                return Err(squirrel_core::error::SquirrelError::Generic(
                    format!("WebSocket error: {}", e)
                ));
            },
            Ok(None) => {
                break;
            },
            Err(_) => {
                // Timeout, continue
            },
            _ => {}
        }
    }
    
    // Verify we received at least some updates
    // Note: We may not receive all updates due to batching or timing
    assert!(received_count > 0, "Did not receive any updates");
    println!("Received {} out of {} updates", received_count, UPDATE_COUNT);
    
    // Cleanup
    server.stop().await?;
    
    Ok(())
} 