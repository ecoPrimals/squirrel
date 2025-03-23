use super::*;
use std::time::Duration;
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio::net::TcpStream;
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream, connect_async, tungstenite::protocol::Message};
use url::Url;
use futures_util::{SinkExt, StreamExt};
use std::io::Read;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use flate2::read::GzDecoder;

#[tokio::test]
async fn test_server_init() {
    // ... existing code ...
}

#[tokio::test]
async fn test_client_connection() {
    // ... existing code ...
}

#[tokio::test]
async fn test_component_subscription() {
    // ... existing code ...
}

#[tokio::test]
async fn test_batched_updates() -> Result<(), Box<dyn std::error::Error>> {
    // Start the dashboard server
    let dashboard_addr = "127.0.0.1:9879";
    let server_handle = tokio::spawn(async move {
        let server = WebSocketServer::new();
        server.start(dashboard_addr.parse().unwrap()).await.unwrap();
    });

    // Give the server a moment to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Create a channel for simulating metric updates
    let (update_sender, _) = mpsc::channel(100);
    let sender_clone = update_sender.clone();

    // Connect a client
    let url = Url::parse(&format!("ws://{}/ws", dashboard_addr))?;
    let (mut ws_stream, _) = connect_async(url).await?;
    
    // Subscribe to a component
    let subscribe_msg = json!({
        "type": "subscribe",
        "componentId": "test_component"
    }).to_string();
    ws_stream.send(Message::Text(subscribe_msg)).await?;
    
    // Wait for subscription confirmation
    let msg = ws_stream.next().await.unwrap()?;
    assert!(msg.is_text());
    let msg_txt = msg.into_text()?;
    let parsed: Value = serde_json::from_str(&msg_txt)?;
    assert_eq!(parsed["type"], "subscription_confirmed");
    
    // Send multiple updates to trigger batching
    for i in 0..15 {
        let update = ComponentUpdate {
            component_id: "test_component".to_string(),
            data: json!({"value": i}),
            timestamp: chrono::Utc::now(),
        };
        sender_clone.send(update).await?;
    }
    
    // Wait a bit for updates to be batched and sent
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Receive the message, should be a batch
    let msg = ws_stream.next().await.unwrap()?;
    assert!(msg.is_text());
    let msg_txt = msg.into_text()?;
    let parsed: Value = serde_json::from_str(&msg_txt)?;
    
    // Verify we received a batch
    assert_eq!(parsed["type"], "batch");
    let updates = parsed["updates"].as_array().unwrap();
    assert!(updates.len() > 1, "Expected multiple updates in batch");
    
    // Cleanup
    ws_stream.close(None).await?;
    server_handle.abort();
    
    Ok(())
}

#[tokio::test]
async fn test_message_compression() -> Result<(), Box<dyn std::error::Error>> {
    // Start the dashboard server with a low compression threshold for testing
    let dashboard_addr = "127.0.0.1:9880";
    let server_handle = tokio::spawn(async move {
        let server = WebSocketServer::new();
        server.start(dashboard_addr.parse().unwrap()).await.unwrap();
    });

    // Give the server a moment to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Create a channel for simulating metric updates
    let (update_sender, _) = mpsc::channel(100);
    let sender_clone = update_sender.clone();

    // Connect a client
    let url = Url::parse(&format!("ws://{}/ws", dashboard_addr))?;
    let (mut ws_stream, _) = connect_async(url).await?;
    
    // Subscribe to a component
    let subscribe_msg = json!({
        "type": "subscribe",
        "componentId": "large_component"
    }).to_string();
    ws_stream.send(Message::Text(subscribe_msg)).await?;
    
    // Wait for subscription confirmation
    let msg = ws_stream.next().await.unwrap()?;
    assert!(msg.is_text());
    
    // Create a large update that will trigger compression
    // Generate a large JSON payload with repeated data
    let mut large_data = json!({
        "name": "Large Test Component",
        "description": "This is a test component with a large payload that should trigger compression"
    });
    
    // Add a large array to ensure the data exceeds the compression threshold
    let mut items = Vec::new();
    for i in 0..1000 {
        items.push(json!({
            "id": i,
            "name": format!("Item {}", i),
            "value": i * 10,
            "metadata": {
                "created_at": format!("2023-10-{:02}", (i % 30) + 1),
                "updated_at": format!("2023-11-{:02}", (i % 30) + 1),
                "tags": ["test", "large", "compression"]
            }
        }));
    }
    large_data["items"] = json!(items);
    
    let update = ComponentUpdate {
        component_id: "large_component".to_string(),
        data: large_data,
        timestamp: chrono::Utc::now(),
    };
    sender_clone.send(update).await?;
    
    // Wait for the update to be processed and sent
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Receive the message, should be compressed
    let msg = ws_stream.next().await.unwrap()?;
    assert!(msg.is_text());
    let msg_txt = msg.into_text()?;
    let parsed: Value = serde_json::from_str(&msg_txt)?;
    
    // Verify we received a compressed message
    assert_eq!(parsed["type"], "compressed");
    assert_eq!(parsed["compressed"], true);
    assert!(parsed["compressed_data"].is_string());
    
    // Test decompression
    let compressed_data = parsed["compressed_data"].as_str().unwrap();
    let decoded_data = BASE64.decode(compressed_data)?;
    let mut decoder = GzDecoder::new(&decoded_data[..]);
    let mut decompressed = String::new();
    decoder.read_to_string(&mut decompressed)?;
    
    // Parse and verify the decompressed data
    let decompressed_json: Value = serde_json::from_str(&decompressed)?;
    assert!(decompressed_json.is_object());
    assert!(decompressed_json.get("type").is_some());
    
    // Cleanup
    ws_stream.close(None).await?;
    server_handle.abort();
    
    Ok(())
}

// Helper function to create a connected WebSocket client
async fn connect_client(addr: &str) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn std::error::Error>> {
    let url = Url::parse(&format!("ws://{}/ws", addr))?;
    let (ws_stream, _) = connect_async(url).await?;
    Ok(ws_stream)
} 