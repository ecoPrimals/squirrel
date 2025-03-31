// Transport implementations for MCP (Machine Context Protocol)
//
// This module provides various transport implementations for the Machine Context Protocol.
// Each transport handles the low-level communication details while providing a consistent
// interface for sending and receiving MCPMessages across different communication channels.
//
// The module includes implementations for:
// - TCP/IP networking
// - WebSockets
// - Standard I/O (for process communication)
// - In-memory channels (for testing and internal communication)
//
// All transports implement the `Transport` trait, which provides a common interface
// for communication regardless of the underlying transport mechanism.
//
// ## Transport Migration Guide
//
// If you were previously using the legacy transport system (removed in version 0.3.0),
// here's how to migrate to the new system:
//
// 1. Replace `Transport` references with implementations of the new `Transport` trait
// 2. For TCP connections:
//    - Old: `Transport::new_tcp("127.0.0.1:9000")`
//    - New: `TcpTransport::new(TcpTransportConfig::default().with_remote_address("127.0.0.1:9000"))`
// 3. For WebSocket connections:
//    - Old: `Transport::new_websocket("ws://localhost:8000")`
//    - New: `WebSocketTransport::new(WebSocketConfig::default().with_url("ws://localhost:8000"))`
// 4. For in-memory testing:
//    - Old: `Transport::new_memory()`
//    - New: `let (transport1, transport2) = MemoryChannel::create_pair()`
// 5. For stdio communication:
//    - Old: `Transport::new_stdio()`
//    - New: `StdioTransport::new(StdioConfig::default())`
//
// All new transport implementations use interior mutability with &self methods, making them
// more compatible with Arc wrapping for thread-safe sharing.

use async_trait::async_trait;
use crate::message::Message;
use crate::protocol::MCPMessage;
use crate::security::types::EncryptionFormat;
use crate::types::CompressionFormat;
use std::net::SocketAddr;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use crate::error::{Result, MCPError};
use std::fmt::Debug;
use uuid;

/// MCP Frame implementation for message framing over byte streams
///
/// Provides a mechanism for framing messages over raw byte streams, ensuring
/// message boundaries are preserved during transport.
pub mod frame;

/// TCP transport implementation for MCP
///
/// Provides TCP/IP-based transport for reliable network communication between
/// MCP components.
pub mod tcp;

/// WebSocket transport implementation for MCP
///
/// Provides WebSocket-based transport for full-duplex communication channels
/// over a single TCP connection, with support for web integration.
pub mod websocket;

/// Standard I/O transport implementation for MCP
///
/// Provides communication via standard input/output streams, useful for
/// interprocess communication and command-line interfaces.
pub mod stdio;

/// In-memory transport implementation for testing
///
/// Provides in-memory message passing for testing purposes and internal
/// component communication without network overhead.
pub mod memory;

/// Transport trait defining the interface for different transport mechanisms
///
/// This trait defines the common interface that all transport implementations
/// must provide. It abstracts away the details of the underlying transport
/// mechanism, allowing MCP components to communicate without knowledge of
/// the specific transport being used.
///
/// ## Implementations
///
/// The following implementations are provided:
/// - `TcpTransport`: TCP/IP-based transport for reliable network communication
/// - `WebSocketTransport`: WebSocket-based transport for web integration
/// - `StdioTransport`: Standard I/O-based transport for interprocess communication
/// - `MemoryChannel`: In-memory transport for testing and internal communication
///
/// ## Design Notes
///
/// All methods in this trait operate on `&self` rather than `&mut self` to support
/// interior mutability and make the trait compatible with Arc wrapping for thread-safe
/// sharing. This is a key improvement over the legacy transport implementation.
///
/// ## Usage Example
///
/// ```rust,no_run
/// use squirrel_mcp::transport::{Transport, tcp::TcpTransport, tcp::TcpTransportConfig};
/// use std::sync::Arc;
///
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     // Create a TCP transport with a specific configuration
///     let config = TcpTransportConfig::default()
///         .with_remote_address("127.0.0.1:9000")
///         .with_connection_timeout(5000);
///
///     let mut transport = TcpTransport::new(config);
///
///     // Connect to the remote endpoint
///     transport.connect().await?;
///
///     // Wrap in Arc for safe sharing between threads
///     let transport = Arc::new(transport);
///
///     // Now the transport can be cloned and shared between threads
///     let transport_clone = Arc::clone(&transport);
///
///     // Spawn a task to listen for messages
///     tokio::spawn(async move {
///         while let Ok(message) = transport_clone.receive_message().await {
///             println!("Received message: {:?}", message);
///         }
///     });
///
///     // Use the original transport to send messages
///     let message = squirrel_mcp::types::MCPMessage::new_ping();
///     transport.send_message(message).await?;
///
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait Transport: Send + Sync + Debug {
    /// Send a message over the transport
    ///
    /// Sends an MCP message over the transport. This method blocks until the message
    /// is queued for sending, but may return before the message is actually
    /// delivered to the remote endpoint.
    ///
    /// # Arguments
    ///
    /// * `message` - The MCP message to send
    ///
    /// # Returns
    ///
    /// Result indicating success or error
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The transport is not connected
    /// - The message cannot be serialized
    /// - The connection is lost during the send operation
    async fn send_message(&self, message: MCPMessage) -> crate::error::Result<()>;

    /// Receive a message from the transport
    /// 
    /// Waits for and receives the next message from the transport. This method
    /// blocks until a message is available or an error occurs.
    ///
    /// # Returns
    ///
    /// Result containing the received message or an error
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The transport is not connected
    /// - The connection is lost while waiting for a message
    /// - The message cannot be deserialized
    /// - A timeout occurs while waiting for a message
    async fn receive_message(&self) -> crate::error::Result<MCPMessage>;

    /// Send raw bytes over the transport
    ///
    /// Sends raw bytes directly over the transport layer. This is useful for sending
    /// pre-serialized data or control messages that don't conform to the standard
    /// MCPMessage structure.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The raw byte slice to send
    ///
    /// # Returns
    ///
    /// Result indicating success or error
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The transport is not connected
    /// - The connection is lost during the send operation
    async fn send_raw(&self, bytes: &[u8]) -> crate::error::Result<()>;

    /// Connect to the transport target
    /// 
    /// Establishes a connection to the remote endpoint. This method must be called
    /// before sending or receiving messages. Implementation requires mutable access
    /// as it typically modifies internal connection state.
    ///
    /// # Returns
    ///
    /// Result indicating success or error
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The connection cannot be established
    /// - A timeout occurs while attempting to connect
    /// - The transport is already connected
    /// - The connection parameters are invalid
    async fn connect(&mut self) -> crate::error::Result<()>;

    /// Disconnect from the transport target
    ///
    /// Closes the connection to the remote endpoint. After calling this method,
    /// the transport will no longer be able to send or receive messages until
    /// connect is called again.
    ///
    /// # Returns
    ///
    /// Result indicating success or error
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The transport is not connected
    /// - An error occurs while closing the connection
    async fn disconnect(&self) -> crate::error::Result<()>;

    /// Check if the transport is connected
    ///
    /// # Returns
    ///
    /// True if the transport is currently connected, false otherwise
    #[must_use]
    async fn is_connected(&self) -> bool;

    /// Get metadata about the transport connection
    async fn get_metadata(&self) -> crate::transport::types::TransportMetadata;
}

pub mod types;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use chrono::Utc;
    use crate::transport::types::TransportMetadata;

    pub struct MockTransport {
        pub connected: Arc<AtomicBool>,
        pub metadata: TransportMetadata,
    }

    impl MockTransport {
        pub fn new() -> Self {
            let metadata = TransportMetadata {
                connection_id: uuid::Uuid::new_v4().to_string(),
                remote_address: None,
                local_address: None,
                connected_at: Utc::now(),
                last_activity: Utc::now(),
                encryption_format: None,
                compression_format: None,
                additional_info: HashMap::new(),
            };

            Self {
                connected: Arc::new(AtomicBool::new(false)),
                metadata,
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

    impl Transport for MockTransport {
        async fn send_message(&self, _message: MCPMessage) -> crate::error::Result<()> {
            if !self.is_connected().await {
                return Err(TransportError::NotConnected.into());
            }
            // Just simulate success in the mock
            Ok(())
        }

        async fn receive_message(&self) -> crate::error::Result<MCPMessage> {
            if !self.is_connected().await {
                return Err(TransportError::NotConnected.into());
            }
            // Return a simple test message
            Ok(MCPMessage::new(
                crate::protocol::types::MessageType::Response,
                "mock-response",
                serde_json::json!({"status": "success"}),
            ))
        }

        async fn send_raw(&self, _bytes: &[u8]) -> crate::error::Result<()> {
            if !self.is_connected().await {
                return Err(TransportError::NotConnected.into());
            }
            // Just simulate success in the mock
            Ok(())
        }

        async fn connect(&mut self) -> crate::error::Result<()> {
            self.connected.store(true, Ordering::SeqCst);
            Ok(())
        }

        async fn disconnect(&self) -> crate::error::Result<()> {
            self.connected.store(false, Ordering::SeqCst);
            Ok(())
        }

        async fn is_connected(&self) -> bool {
            self.connected.load(Ordering::SeqCst)
        }

        async fn get_metadata(&self) -> crate::transport::types::TransportMetadata {
            self.metadata.clone()
        }
    }

    #[tokio::test]
    async fn test_mock_transport() {
        let mut transport = MockTransport::new();
        assert!(!transport.is_connected().await);
        
        transport.connect().await.unwrap();
        assert!(transport.is_connected().await);
        
        let msg = transport.receive_message().await.unwrap();
        assert_eq!(msg.message_type(), &crate::protocol::types::MessageType::Response);
        
        transport.disconnect().await.unwrap();
        assert!(!transport.is_connected().await);
        
        // After disconnection, should fail
        assert!(transport.send_message(msg).await.is_err());
    }
} 