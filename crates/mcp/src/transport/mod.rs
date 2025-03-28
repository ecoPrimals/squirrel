use std::sync::Arc;
use async_trait::async_trait;
use crate::error::transport::TransportError;
use crate::types::MCPMessage;
use std::sync::atomic::AtomicBool;

pub mod frame;
pub mod tcp;
pub mod websocket;
pub mod stdio;
pub mod memory;

/// Metadata about a transport connection
#[derive(Debug, Clone)]
pub struct TransportMetadata {
    /// Type of transport (tcp, websocket, stdio, etc.)
    pub transport_type: String,
    
    /// Remote address for the connection, if applicable
    pub remote_address: String,
    
    /// Local address for the connection, if applicable
    pub local_address: Option<String>,
    
    /// Encryption format used by the transport
    pub encryption: crate::types::EncryptionFormat,
    
    /// Compression format used by the transport
    pub compression: crate::types::CompressionFormat,
}

/// Transport trait defining the interface for different transport mechanisms
#[async_trait]
pub trait Transport: Send + Sync {
    /// Send a message over the transport
    async fn send_message(&self, message: MCPMessage) -> Result<(), TransportError>;

    /// Receive a message from the transport
    /// 
    /// Note: Changed from &mut self to &self to allow better sharing with Arc
    async fn receive_message(&self) -> Result<MCPMessage, TransportError>;

    /// Connect to the transport target
    /// 
    /// Note: Changed from &mut self to &self to allow better sharing with Arc
    async fn connect(&self) -> Result<(), TransportError>;

    /// Disconnect from the transport target
    async fn disconnect(&self) -> Result<(), TransportError>;

    /// Check if the transport is connected
    async fn is_connected(&self) -> bool;

    /// Get transport metadata
    fn get_metadata(&self) -> TransportMetadata;
}

// Export Transport implementations
pub use tcp::TcpTransport;
pub use websocket::WebSocketTransport;
pub use stdio::StdioTransport;
pub use memory::MemoryTransport;
pub use memory::MemoryChannel;

#[cfg(test)]
mod tests {
    use super::*;
    
    // Mock Transport for testing
    pub struct MockTransport {
        pub connected: Arc<AtomicBool>,
        pub metadata: TransportMetadata,
    }
    
    impl MockTransport {
        pub fn new() -> Self {
            Self {
                connected: Arc::new(AtomicBool::new(false)),
                metadata: TransportMetadata {
                    transport_type: "mock".to_string(),
                    remote_address: "mock://localhost".to_string(),
                    local_address: None,
                    encryption: crate::types::EncryptionFormat::None,
                    compression: crate::types::CompressionFormat::None,
                },
            }
        }
    }
    
    impl Clone for MockTransport {
        fn clone(&self) -> Self {
            Self {
                connected: Arc::clone(&self.connected),
                metadata: self.metadata.clone(),
            }
        }
    }
    
    #[async_trait]
    impl Transport for MockTransport {
        async fn send_message(&self, _message: MCPMessage) -> Result<(), TransportError> {
            if !self.is_connected().await {
                return Err(TransportError::ConnectionClosed("Not connected".into()));
            }
            Ok(())
        }
        
        // Updated to use &self instead of &mut self
        async fn receive_message(&self) -> Result<MCPMessage, TransportError> {
            if !self.is_connected().await {
                return Err(TransportError::ConnectionClosed("Not connected".into()));
            }
            
            // Create an MCPMessage directly using the MCPMessage constructor
            Ok(crate::types::MCPMessage::new(
                crate::types::MessageType::Response, 
                serde_json::json!({
                    "content": "{}",
                    "source": "mock",
                    "destination": "test"
                })
            ))
        }
        
        // Updated to use &self instead of &mut self
        async fn connect(&self) -> Result<(), TransportError> {
            self.connected.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
        
        async fn disconnect(&self) -> Result<(), TransportError> {
            self.connected.store(false, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
        
        async fn is_connected(&self) -> bool {
            self.connected.load(std::sync::atomic::Ordering::SeqCst)
        }
        
        fn get_metadata(&self) -> TransportMetadata {
            self.metadata.clone()
        }
    }
    
    #[tokio::test]
    async fn test_mock_transport() {
        println!("Starting MockTransport test...");
        let transport = MockTransport::new(); // No longer needs to be mutable
        assert!(!transport.is_connected().await);
        
        transport.connect().await.unwrap();
        assert!(transport.is_connected().await);
        
        let metadata = transport.get_metadata();
        assert_eq!(metadata.transport_type, "mock");
        
        // Test receiving a message
        let message = transport.receive_message().await.unwrap();
        println!("Received message: {:?}", message);
        
        transport.disconnect().await.unwrap();
        assert!(!transport.is_connected().await);
        println!("MockTransport test completed successfully!");
    }
} 