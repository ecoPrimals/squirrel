//! Tests for WebSocket functionality

use super::*;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::mpsc;
use serde_json::json;

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use serde_json::json;
    use tokio::sync::mpsc;
    use futures::StreamExt;
    
    use crate::websocket::{
        ConnectionManager,
        models::{ChannelCategory, WebSocketResponse, WebSocketEvent, WebSocketCommand},
        error::WebSocketError,
    };

    /// Test the connection manager creation and basic operations
    #[tokio::test]
    async fn test_connection_manager() {
        // Create a new connection manager
        let manager = ConnectionManager::new();
        
        // Create a channel for sending messages to the client
        let (tx, mut rx) = mpsc::channel::<Result<String, WebSocketError>>(10);
        
        // Register a connection
        let connection_id = manager
            .register_connection(Some("test_user".to_string()), vec!["user".to_string()], tx)
            .await;
        
        // Verify connection count
        assert_eq!(manager.connection_count().await, 1);
        
        // Subscribe to a channel
        manager
            .subscribe(&connection_id, ChannelCategory::Job, "test-job")
            .await
            .expect("Failed to subscribe");
        
        // Verify subscription count
        assert_eq!(
            manager
                .subscription_count(ChannelCategory::Job, "test-job")
                .await,
            1
        );
        
        // Broadcast a message to the channel
        let sent_count = manager
            .broadcast_to_channel(
                ChannelCategory::Job,
                "test-job",
                "test-event",
                json!({"message": "Hello, WebSocket!"}),
            )
            .await
            .expect("Failed to broadcast");
        
        // Verify that the message was sent to one connection
        assert_eq!(sent_count, 1);
        
        // Check if the message was received
        if let Some(Ok(message)) = rx.recv().await {
            let response: WebSocketResponse = serde_json::from_str(&message)
                .expect("Failed to parse response");
            
            assert!(response.success);
            assert_eq!(response.event, "test-event");
            
            if let serde_json::Value::Object(data) = response.data {
                assert_eq!(
                    data.get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default(),
                    "Hello, WebSocket!"
                );
            } else {
                panic!("Expected data to be an object");
            }
        } else {
            panic!("Did not receive a message");
        }
        
        // Unsubscribe from the channel
        manager
            .unsubscribe(&connection_id, ChannelCategory::Job, "test-job")
            .await
            .expect("Failed to unsubscribe");
        
        // Verify subscription count is now 0
        assert_eq!(
            manager
                .subscription_count(ChannelCategory::Job, "test-job")
                .await,
            0
        );
        
        // Remove the connection
        manager.remove_connection(&connection_id).await;
        
        // Verify connection count is now 0
        assert_eq!(manager.connection_count().await, 0);
    }
    
    /// Test subscribing to multiple channels
    #[tokio::test]
    async fn test_multiple_subscriptions() {
        // Create a new connection manager
        let manager = ConnectionManager::new();
        
        // Create a channel for sending messages
        let (tx, mut rx) = mpsc::channel::<Result<String, WebSocketError>>(10);
        
        // Register a connection
        let connection_id = manager
            .register_connection(Some("test_user".to_string()), vec!["user".to_string()], tx)
            .await;
        
        // Subscribe to multiple channels
        for channel in &["job-1", "job-2", "job-3"] {
            manager
                .subscribe(&connection_id, ChannelCategory::Job, channel)
                .await
                .expect("Failed to subscribe");
        }
        
        // Verify subscription count for each channel
        for channel in &["job-1", "job-2", "job-3"] {
            assert_eq!(
                manager
                    .subscription_count(ChannelCategory::Job, channel)
                    .await,
                1
            );
        }
        
        // Verify active subscriptions
        let subscriptions = manager.get_active_subscriptions().await;
        assert_eq!(subscriptions.len(), 3);
        
        // Broadcast to one channel
        manager
            .broadcast_to_channel(
                ChannelCategory::Job,
                "job-2",
                "update",
                json!({"status": "running"}),
            )
            .await
            .expect("Failed to broadcast");
        
        // Check received message
        if let Some(Ok(message)) = rx.recv().await {
            let response: WebSocketResponse = serde_json::from_str(&message)
                .expect("Failed to parse response");
            
            assert!(response.success);
            assert_eq!(response.event, "update");
            
            if let serde_json::Value::Object(data) = response.data {
                assert_eq!(
                    data.get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default(),
                    "running"
                );
            } else {
                panic!("Expected data to be an object");
            }
        } else {
            panic!("Did not receive a message");
        }
        
        // Cleanup
        manager.remove_connection(&connection_id).await;
    }
    
    /// Test sending messages to specific connections
    #[tokio::test]
    async fn test_send_to_connection() {
        // Create a new connection manager
        let manager = ConnectionManager::new();
        
        // Create channels for two connections
        let (tx1, mut rx1) = mpsc::channel::<Result<String, WebSocketError>>(10);
        let (tx2, mut rx2) = mpsc::channel::<Result<String, WebSocketError>>(10);
        
        // Register two connections
        let conn_id1 = manager
            .register_connection(Some("user1".to_string()), vec!["user".to_string()], tx1)
            .await;
        let conn_id2 = manager
            .register_connection(Some("user2".to_string()), vec!["user".to_string()], tx2)
            .await;
        
        // Send a message to the first connection
        let response1 = WebSocketResponse {
            success: true,
            event: "personal-message".to_string(),
            data: json!({"message": "Hello, User 1!"}),
            error: None,
            id: None,
        };
        
        manager
            .send_to_connection(&conn_id1, response1)
            .await
            .expect("Failed to send message");
        
        // Send a message to the second connection
        let response2 = WebSocketResponse {
            success: true,
            event: "personal-message".to_string(),
            data: json!({"message": "Hello, User 2!"}),
            error: None,
            id: None,
        };
        
        manager
            .send_to_connection(&conn_id2, response2)
            .await
            .expect("Failed to send message");
        
        // Check message received by first connection
        if let Some(Ok(message)) = rx1.recv().await {
            let response: WebSocketResponse = serde_json::from_str(&message)
                .expect("Failed to parse response");
            
            assert!(response.success);
            assert_eq!(response.event, "personal-message");
            
            if let serde_json::Value::Object(data) = response.data {
                assert_eq!(
                    data.get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default(),
                    "Hello, User 1!"
                );
            } else {
                panic!("Expected data to be an object");
            }
        } else {
            panic!("User 1 did not receive a message");
        }
        
        // Check message received by second connection
        if let Some(Ok(message)) = rx2.recv().await {
            let response: WebSocketResponse = serde_json::from_str(&message)
                .expect("Failed to parse response");
            
            assert!(response.success);
            assert_eq!(response.event, "personal-message");
            
            if let serde_json::Value::Object(data) = response.data {
                assert_eq!(
                    data.get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default(),
                    "Hello, User 2!"
                );
            } else {
                panic!("Expected data to be an object");
            }
        } else {
            panic!("User 2 did not receive a message");
        }
        
        // Cleanup
        manager.remove_connection(&conn_id1).await;
        manager.remove_connection(&conn_id2).await;
    }
    
    /// Test WebSocket command parsing
    #[test]
    fn test_command_parsing() {
        // Test parsing a valid subscribe command
        let json_str = r#"{
            "command": "subscribe",
            "id": "abc123",
            "params": {
                "category": "job",
                "channel": "job-123"
            }
        }"#;
        
        let command: WebSocketCommand = serde_json::from_str(json_str).unwrap();
        assert_eq!(command.command, "subscribe");
        assert_eq!(command.id, Some("abc123".to_string()));
        assert_eq!(command.params.len(), 2);
        assert_eq!(
            command.params.get("category").and_then(|v| v.as_str()),
            Some("job")
        );
        
        // Test parsing a ping command with no ID
        let json_str = r#"{
            "command": "ping",
            "params": {
                "data": "ping-data"
            }
        }"#;
        
        let command: WebSocketCommand = serde_json::from_str(json_str).unwrap();
        assert_eq!(command.command, "ping");
        assert_eq!(command.id, None);
        assert_eq!(
            command.params.get("data").and_then(|v| v.as_str()),
            Some("ping-data")
        );
    }
    
    /// Test the WebSocket event broadcasting
    #[tokio::test]
    async fn test_event_stream() {
        let manager = ConnectionManager::new();
        
        // Subscribe to events
        let mut event_stream = manager.subscribe_to_events();
        
        // Create a task to listen for events
        let event_task = tokio::spawn(async move {
            if let Some(event) = event_stream.next().await {
                return event;
            }
            panic!("No event received");
        });
        
        // Broadcast an event
        manager
            .broadcast_to_channel(
                ChannelCategory::System,
                "alerts",
                "system-alert",
                json!({"level": "warning", "message": "Test alert"}),
            )
            .await
            .expect("Failed to broadcast");
        
        // Wait for the event to be received
        let event = tokio::time::timeout(Duration::from_secs(1), event_task)
            .await
            .expect("Timed out waiting for event")
            .expect("Task failed");
        
        // Verify the event
        assert_eq!(event.event, "system-alert");
        assert_eq!(event.category, ChannelCategory::System);
        assert_eq!(event.channel, "alerts");
        
        if let serde_json::Value::Object(data) = event.data {
            assert_eq!(
                data.get("level").and_then(|v| v.as_str()).unwrap_or_default(),
                "warning"
            );
            assert_eq!(
                data.get("message").and_then(|v| v.as_str()).unwrap_or_default(),
                "Test alert"
            );
        } else {
            panic!("Expected data to be an object");
        }
    }
} 