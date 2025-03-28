//! Transport Migration Utilities
//!
//! This file contains utility functions that can help with migrating from the old transport
//! implementation to the new one. These functions can be used as examples or directly included
//! in your codebase to help with the migration process.

use mcp::error::Result;
use mcp::message::MCPMessage;
use std::sync::Arc;

/// Encapsulates the transport type to allow gradual migration
pub enum MigratingTransport {
    /// Using the old transport implementation (will be removed in the future)
    #[cfg(feature = "legacy-transport")]
    Old(mcp::transport_old::Transport),
    
    /// Using the new transport implementation
    New(Arc<dyn mcp::transport::Transport>),
}

impl MigratingTransport {
    /// Create a new transport instance
    ///
    /// This function creates a transport instance using either the old or new implementation,
    /// based on the `use_new` parameter. This allows for gradual migration by controlling
    /// which implementation is used at runtime.
    #[cfg(feature = "legacy-transport")]
    pub fn new(address: &str, use_new: bool) -> Self {
        if use_new {
            // Create new TCP transport
            let mut config = mcp::transport::tcp::TcpTransportConfig::default();
            config.remote_address = Some(address.to_string());
            
            let transport = Arc::new(mcp::transport::tcp::TcpTransport::new(config));
            Self::New(transport)
        } else {
            // Create old transport
            let mut config = mcp::transport_old::TransportConfig::default();
            config.remote_address = Some(address.to_string());
            
            let transport = mcp::transport_old::Transport::new(config);
            Self::Old(transport)
        }
    }
    
    /// Create a new transport instance (when legacy-transport is disabled)
    #[cfg(not(feature = "legacy-transport"))]
    pub fn new(address: &str, _use_new: bool) -> Self {
        // Only new implementation is available
        let mut config = mcp::transport::tcp::TcpTransportConfig::default();
        config.remote_address = Some(address.to_string());
        
        let transport = Arc::new(mcp::transport::tcp::TcpTransport::new(config));
        Self::New(transport)
    }
    
    /// Connect to the transport target
    pub async fn connect(&self) -> Result<()> {
        match self {
            #[cfg(feature = "legacy-transport")]
            Self::Old(transport) => transport.connect().await,
            Self::New(transport) => transport.connect().await,
        }
    }
    
    /// Check if the transport is connected
    pub async fn is_connected(&self) -> bool {
        match self {
            #[cfg(feature = "legacy-transport")]
            Self::Old(transport) => transport.state == mcp::transport_old::TransportState::Connected,
            Self::New(transport) => transport.is_connected().await.unwrap_or(false),
        }
    }
    
    /// Send a message through the transport
    pub async fn send_message(&self, message: MCPMessage) -> Result<()> {
        match self {
            #[cfg(feature = "legacy-transport")]
            Self::Old(transport) => transport.send_message(message).await,
            Self::New(transport) => transport.send_message(message).await,
        }
    }
    
    /// Receive a message from the transport
    pub async fn receive_message(&self) -> Result<MCPMessage> {
        match self {
            #[cfg(feature = "legacy-transport")]
            Self::Old(transport) => transport.receive_message().await,
            Self::New(transport) => transport.receive_message().await,
        }
    }
    
    /// Disconnect from the transport
    pub async fn disconnect(&self) -> Result<()> {
        match self {
            #[cfg(feature = "legacy-transport")]
            Self::Old(transport) => transport.disconnect().await,
            Self::New(transport) => transport.disconnect().await,
        }
    }
}

/// Create a pair of in-memory transports for testing
///
/// This function creates a pair of in-memory transports that can be used for testing.
/// It uses the new implementation regardless of whether the legacy-transport feature is enabled.
pub fn create_memory_transport_pair() -> (
    Arc<dyn mcp::transport::Transport>,
    Arc<dyn mcp::transport::Transport>
) {
    let (client, server) = mcp::transport::memory::MemoryChannel::create_pair();
    (Arc::new(client), Arc::new(server))
}

/// Convert an old transport to a new one (if using the legacy-transport feature)
///
/// This function converts an old transport to a new one using the compatibility layer.
#[cfg(feature = "legacy-transport")]
pub fn convert_transport(
    old_transport: &mcp::transport_old::Transport
) -> Result<Arc<dyn mcp::transport::Transport>> {
    mcp::transport_old::create_new_tcp_transport(old_transport)
}

/// Detect the best transport implementation based on the address
///
/// This function creates the most appropriate transport implementation based on the address.
pub fn create_transport_for_address(address: &str) -> Arc<dyn mcp::transport::Transport> {
    if address.starts_with("ws://") || address.starts_with("wss://") {
        // WebSocket transport
        let mut config = mcp::transport::websocket::WebSocketTransportConfig::default();
        config.url = address.to_string();
        Arc::new(mcp::transport::websocket::WebSocketTransport::new(config))
    } else {
        // TCP transport by default
        let mut config = mcp::transport::tcp::TcpTransportConfig::default();
        config.remote_address = Some(address.to_string());
        Arc::new(mcp::transport::tcp::TcpTransport::new(config))
    }
} 