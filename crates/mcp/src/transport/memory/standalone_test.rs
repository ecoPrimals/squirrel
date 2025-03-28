#[cfg(test)]
mod memory_transport_tests {
    use crate::transport::memory::{MemoryChannel, MemoryTransport, MemoryTransportConfig};
    use crate::transport::Transport;
    use crate::types::MCPMessage;
    use std::sync::Arc;
    use tokio::sync::mpsc;
    use tokio::time::Duration;
    
    #[tokio::test]
    async fn test_basic_memory_transport() {
        // Create a simple memory transport config
        let config_a = MemoryTransportConfig {
            name: "client".to_string(),
            ..Default::default()
        };
        
        let config_b = MemoryTransportConfig {
            name: "server".to_string(),
            ..Default::default()
        };
        
        // Create a memory channel
        let channel = MemoryChannel::new(100, Some(10));
        
        // Create a transport pair
        let (mut client, mut server) = channel.create_transport_pair(Some(config_a), Some(config_b));
        
        // Connect both transports
        assert!(client.connect().await.is_ok());
        assert!(server.connect().await.is_ok());
        
        // Verify both are connected
        assert!(client.is_connected().await);
        assert!(server.is_connected().await);
        
        // Test sending a message from client to server
        let test_msg = MCPMessage::new(
            crate::types::MessageType::Command,
            serde_json::json!({ "action": "test" }),
        );
        
        assert!(client.send_message(test_msg.clone()).await.is_ok());
        
        // Receive on server side
        let received = server.receive_message().await.unwrap();
        
        // Verify message
        assert_eq!(received.id.0, test_msg.id.0);
        assert_eq!(received.type_, test_msg.type_);
        
        // Disconnect
        assert!(client.disconnect().await.is_ok());
        assert!(server.disconnect().await.is_ok());
        
        // Verify disconnected
        assert!(!client.is_connected().await);
        assert!(!server.is_connected().await);
    }
    
    #[tokio::test]
    async fn test_create_pair_arc() {
        // Test the Arc-wrapped transports
        let (client, server) = MemoryChannel::create_pair_arc();
        
        // Connect both transports
        assert!(client.connect().await.is_ok());
        assert!(server.connect().await.is_ok());
        
        // Verify both are connected
        assert!(client.is_connected().await);
        assert!(server.is_connected().await);
        
        // Test sending a message from client to server
        let test_msg = MCPMessage::new(
            crate::types::MessageType::Command,
            serde_json::json!({ "action": "test_arc" }),
        );
        
        assert!(client.send_message(test_msg.clone()).await.is_ok());
        
        // Use tokio timeout to prevent test from hanging
        let receive_future = server.receive_message();
        let received = tokio::time::timeout(
            Duration::from_secs(1),
            receive_future
        ).await.unwrap().unwrap();
        
        // Verify message
        assert_eq!(received.id.0, test_msg.id.0);
        assert_eq!(received.type_, test_msg.type_);
        
        // Disconnect
        assert!(client.disconnect().await.is_ok());
        assert!(server.disconnect().await.is_ok());
        
        // Verify disconnected
        assert!(!client.is_connected().await);
        assert!(!server.is_connected().await);
    }
} 