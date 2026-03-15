// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Mock Transport for Testing
//!
//! Provides a mock transport implementation for testing MCP client
//! without requiring actual network connections.

use async_trait::async_trait;
use squirrel_mcp::error::Result;
use squirrel_mcp::transport::Transport;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

/// Mock transport for testing
pub struct MockTransport {
    /// Sender for outgoing messages
    tx: mpsc::UnboundedSender<Vec<u8>>,
    /// Receiver for incoming messages
    rx: Arc<Mutex<mpsc::UnboundedReceiver<Vec<u8>>>>,
    /// Sent messages (for verification)
    sent_messages: Arc<Mutex<Vec<Vec<u8>>>>,
    /// Flag to simulate connection state
    connected: Arc<Mutex<bool>>,
}

impl MockTransport {
    /// Create a new mock transport
    pub fn new() -> (Self, MockTransportController) {
        let (tx, rx) = mpsc::unbounded_channel();
        let (controller_tx, controller_rx) = mpsc::unbounded_channel();
        
        let sent_messages = Arc::new(Mutex::new(Vec::new()));
        let connected = Arc::new(Mutex::new(false));

        let transport = Self {
            tx: controller_tx.clone(),
            rx: Arc::new(Mutex::new(rx)),
            sent_messages: Arc::clone(&sent_messages),
            connected: Arc::clone(&connected),
        };

        let controller = MockTransportController {
            tx: tx.clone(),
            rx: Arc::new(Mutex::new(controller_rx)),
            sent_messages,
            connected,
        };

        (transport, controller)
    }

    /// Set connected state
    pub async fn set_connected(&self, state: bool) {
        let mut connected = self.connected.lock().await;
        *connected = state;
    }
}

impl Default for MockTransport {
    fn default() -> Self {
        Self::new().0
    }
}

#[async_trait]
impl Transport for MockTransport {
    async fn send(&self, data: Vec<u8>) -> Result<()> {
        let connected = self.connected.lock().await;
        if !*connected {
            return Err(squirrel_mcp::MCPError::Connection(
                squirrel_mcp::error::connection::ConnectionError::NotConnected
            ));
        }
        drop(connected);

        // Store sent message
        {
            let mut sent = self.sent_messages.lock().await;
            sent.push(data.clone());
        }

        // Send to controller
        self.tx.send(data).map_err(|_| {
            squirrel_mcp::MCPError::General("Failed to send message".to_string())
        })?;

        Ok(())
    }

    async fn receive(&self) -> Result<Vec<u8>> {
        let connected = self.connected.lock().await;
        if !*connected {
            return Err(squirrel_mcp::MCPError::Connection(
                squirrel_mcp::error::connection::ConnectionError::NotConnected
            ));
        }
        drop(connected);

        let mut rx = self.rx.lock().await;
        rx.recv().await.ok_or_else(|| {
            squirrel_mcp::MCPError::General("No message received".to_string())
        })
    }

    async fn close(&self) -> Result<()> {
        let mut connected = self.connected.lock().await;
        *connected = false;
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        *self.connected.lock().await
    }
}

/// Controller for mock transport (test side)
pub struct MockTransportController {
    /// Sender for injecting incoming messages
    tx: mpsc::UnboundedSender<Vec<u8>>,
    /// Receiver for sent messages
    rx: Arc<Mutex<mpsc::UnboundedReceiver<Vec<u8>>>>,
    /// Reference to sent messages
    sent_messages: Arc<Mutex<Vec<Vec<u8>>>>,
    /// Connection state
    connected: Arc<Mutex<bool>>,
}

impl MockTransportController {
    /// Send a message to the transport (simulating server response)
    pub async fn send_message(&self, data: Vec<u8>) -> Result<()> {
        self.tx.send(data).map_err(|_| {
            squirrel_mcp::MCPError::General("Failed to inject message".to_string())
        })
    }

    /// Receive a message sent by the transport
    pub async fn receive_sent_message(&self) -> Option<Vec<u8>> {
        let mut rx = self.rx.lock().await;
        rx.recv().await
    }

    /// Get all sent messages
    pub async fn get_sent_messages(&self) -> Vec<Vec<u8>> {
        let sent = self.sent_messages.lock().await;
        sent.clone()
    }

    /// Clear sent messages
    pub async fn clear_sent_messages(&self) {
        let mut sent = self.sent_messages.lock().await;
        sent.clear();
    }

    /// Set connection state
    pub async fn set_connected(&self, state: bool) {
        let mut connected = self.connected.lock().await;
        *connected = state;
    }

    /// Check connection state
    pub async fn is_connected(&self) -> bool {
        *self.connected.lock().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_transport_creation() {
        let (transport, _controller) = MockTransport::new();
        assert!(!transport.is_connected().await);
    }

    #[tokio::test]
    async fn test_mock_transport_send_receive() {
        let (transport, controller) = MockTransport::new();
        
        // Set connected
        transport.set_connected(true).await;
        controller.set_connected(true).await;

        // Send message from transport
        let message = b"test message".to_vec();
        transport.send(message.clone()).await.expect("test: should succeed");

        // Receive on controller side
        let received = controller.receive_sent_message().await;
        assert_eq!(received, Some(message));
    }

    #[tokio::test]
    async fn test_mock_transport_inject_message() {
        let (transport, controller) = MockTransport::new();
        
        // Set connected
        transport.set_connected(true).await;
        controller.set_connected(true).await;

        // Inject message from controller
        let message = b"server response".to_vec();
        controller.send_message(message.clone()).await.expect("test: should succeed");

        // Receive on transport side
        let received = transport.receive().await.expect("test: should succeed");
        assert_eq!(received, message);
    }

    #[tokio::test]
    async fn test_mock_transport_not_connected() {
        let (transport, _controller) = MockTransport::new();

        // Try to send without connection
        let result = transport.send(b"test".to_vec()).await;
        assert!(result.is_err());

        // Try to receive without connection
        let result = transport.receive().await;
        assert!(result.is_err());
    }
}

