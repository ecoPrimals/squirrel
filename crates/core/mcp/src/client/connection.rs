// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Connection management for MCP clients
//!
//! This module handles client connection states, transport creation, and connection lifecycle management.

use crate::error::{MCPError, Result};
use crate::transport::Transport;
use crate::transport::tcp::{TcpTransport, TcpTransportConfig};
use crate::client::config::ClientConfig;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Client connection state
#[derive(Debug, Clone, PartialEq)]
pub enum ClientState {
    /// Client is disconnected
    Disconnected,
    /// Client is connecting
    Connecting,
    /// Client is connected
    Connected,
    /// Client is disconnecting
    Disconnecting,
    /// Client connection failed
    Failed,
}

impl Default for ClientState {
    fn default() -> Self {
        Self::Disconnected
    }
}

/// Connection manager for MCP clients
pub struct ConnectionManager {
    /// Configuration
    config: ClientConfig,
    /// Current transport
    transport: Arc<RwLock<Option<Arc<dyn Transport>>>>,
    /// Current connection state
    state: Arc<RwLock<ClientState>>,
    /// Last error encountered
    last_error: Arc<RwLock<Option<MCPError>>>,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new(config: ClientConfig) -> Self {
        Self {
            config,
            transport: Arc::new(RwLock::new(None)),
            state: Arc::new(RwLock::new(ClientState::default())),
            last_error: Arc::new(RwLock::new(None)),
        }
    }

    /// Get the current connection state
    pub async fn get_state(&self) -> ClientState {
        *self.state.read().await
    }

    /// Check if the client is connected
    pub async fn is_connected(&self) -> bool {
        matches!(*self.state.read().await, ClientState::Connected)
    }

    /// Get the last error encountered
    pub async fn get_last_error(&self) -> Option<MCPError> {
        self.last_error.read().await.clone()
    }

    /// Set the connection state
    pub async fn set_state(&self, new_state: ClientState) {
        let mut state = self.state.write().await;
        *state = new_state;
    }

    /// Set the last error
    pub async fn set_last_error(&self, error: Option<MCPError>) {
        let mut last_error = self.last_error.write().await;
        *last_error = error;
    }

    /// Connect to the server
    pub async fn connect(&self) -> Result<Arc<dyn Transport>> {
        // Set state to connecting
        self.set_state(ClientState::Connecting).await;

        // Clear previous error
        self.set_last_error(None).await;

        // Create transport
        match self.create_transport().await {
            Ok(transport) => {
                // Store transport
                let mut transport_guard = self.transport.write().await;
                *transport_guard = Some(transport.clone());
                
                // Set state to connected
                self.set_state(ClientState::Connected).await;
                
                Ok(transport)
            }
            Err(error) => {
                // Store error and set state to failed
                self.set_last_error(Some(error.clone())).await;
                self.set_state(ClientState::Failed).await;
                Err(error)
            }
        }
    }

    /// Disconnect from the server
    pub async fn disconnect(&self) -> Result<()> {
        // Set state to disconnecting
        self.set_state(ClientState::Disconnecting).await;

        // Get and clear transport
        let transport = {
            let mut transport_guard = self.transport.write().await;
            transport_guard.take()
        };

        // Disconnect transport if available
        if let Some(transport) = transport {
            if let Err(error) = transport.disconnect().await {
                self.set_last_error(Some(error.clone())).await;
                self.set_state(ClientState::Failed).await;
                return Err(error);
            }
        }

        // Set state to disconnected
        self.set_state(ClientState::Disconnected).await;
        Ok(())
    }

    /// Get the current transport
    pub async fn get_transport(&self) -> Option<Arc<dyn Transport>> {
        self.transport.read().await.clone()
    }

    /// Create a transport based on configuration
    async fn create_transport(&self) -> Result<Arc<dyn Transport>> {
        // Use custom transport if provided
        if let Some(transport) = &self.config.transport {
            return Ok(transport.clone());
        }

        // Create default TCP transport
        let tcp_config = TcpTransportConfig::default()
            .with_remote_address(&self.config.server_address)
            .with_connection_timeout(self.config.connection_timeout_ms());

        let transport = TcpTransport::new(tcp_config);

        // Connect the transport
        transport.connect().await.map_err(|e| {
            MCPError::ConnectionError(format!("Failed to connect transport: {}", e))
        })?;

        Ok(Arc::new(transport))
    }

    /// Handle connection error
    pub async fn handle_connection_error(&self, error: MCPError) -> Result<()> {
        self.set_last_error(Some(error.clone())).await;
        self.set_state(ClientState::Failed).await;
        
        // Clear transport
        let mut transport_guard = self.transport.write().await;
        *transport_guard = None;
        
        Err(error)
    }
}

/// Create a transport from configuration
pub fn create_transport_from_config(config: &ClientConfig) -> Arc<dyn Transport> {
    // Use custom transport if provided
    if let Some(transport) = &config.transport {
        return transport.clone();
    }

    // Create default TCP transport
    let tcp_config = TcpTransportConfig::default()
        .with_remote_address(&config.server_address)
        .with_connection_timeout(config.connection_timeout_ms());

    let transport = TcpTransport::new(tcp_config);
    Arc::new(transport)
} 