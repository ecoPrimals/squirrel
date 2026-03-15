// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! MCP Client API
//!
//! This module provides a high-level client API for interacting with the Machine Context Protocol.
//! It handles connection management, message sending/receiving, and event subscription.
//!
//! ## Features
//!
//! * Connection management with automatic reconnection
//! * Command/response handling with timeouts
//! * Event publishing and subscription
//! * Support for different transport mechanisms
//! * Secure communication with transport encryption
//!
//! ## Usage Examples
//!
//! ```rust,no_run
//! use squirrel_mcp::client::{MCPClient, ClientConfig};
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create client with default configuration
//!     let mut client = MCPClient::new(ClientConfig::default());
//!     
//!     // Connect to server
//!     client.connect().await?;
//!     
//!     // Send a command
//!     let response = client.send_command_with_content(
//!         "get_status",
//!         json!({
//!             "detail_level": "full"
//!         })
//!     ).await?;
//!     
//!     // Process response
//!     println!("Response: {:?}", response);
//!     
//!     // Subscribe to events
//!     let mut event_receiver = client.subscribe_to_events().await;
//!     
//!     // Disconnect when done
//!     client.disconnect().await?;
//!     
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod connection;
pub mod event;
pub mod session;

// Re-export main types
pub use config::ClientConfig;
pub use connection::{ClientState, ConnectionManager};
pub use event::{EventHandler, CompositeEventHandler, LoggingEventHandler, ChannelEventHandler, FilteringEventHandler, BatchingEventHandler};
pub use session::SessionManager;

use crate::error::{MCPError, Result};
use crate::message::Message;
use crate::session::Session;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

// Import configuration if available
#[cfg(feature = "config")]
use squirrel_mcp_config::unified::SquirrelUnifiedConfig;  // Migrated from deprecated Config type (ADR-008)

/// High-level MCP client that orchestrates all components
pub struct MCPClient {
    /// Configuration
    pub config: ClientConfig,
    /// Connection manager
    connection_manager: Arc<ConnectionManager>,
    /// Session manager
    session_manager: Arc<SessionManager>,
    /// Current session information
    session: Arc<RwLock<Option<Session>>>,
}

impl MCPClient {
    /// Create a new MCP client with the given configuration
    #[must_use]
    pub fn new(config: ClientConfig) -> Self {
        let connection_manager = Arc::new(ConnectionManager::new(config.clone()));
        let session_manager = Arc::new(SessionManager::new(config.clone(), Arc::clone(&connection_manager)));

        Self {
            config,
            connection_manager,
            session_manager,
            session: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a new client from environment variables
    pub fn from_env() -> Self {
        Self::new(ClientConfig::from_env())
    }

    /// Create a new client from global configuration
    #[cfg(feature = "config")]
    pub fn from_global_config(config: &Config) -> Self {
        Self::new(ClientConfig::from_global_config(config))
    }

    /// Get the last error encountered
    pub async fn get_last_error(&self) -> Option<MCPError> {
        self.connection_manager.get_last_error().await
    }

    /// Check if the client is connected
    pub async fn is_connected(&self) -> bool {
        self.connection_manager.is_connected().await
    }

    /// Connect to the MCP server
    pub async fn connect(&mut self) -> Result<()> {
        // Validate configuration
        self.config.validate()?;

        // Connect via connection manager
        let transport = self.connection_manager.connect().await?;

        // Start session manager
        self.session_manager.start().await?;

        // Create session
        let session = Session::new(
            self.config.client_id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            transport,
        );

        // Store session
        {
            let mut session_guard = self.session.write().await;
            *session_guard = Some(session.clone());
        }

        self.session_manager.set_session(Some(session)).await;

        Ok(())
    }

    /// Disconnect from the MCP server
    pub async fn disconnect(&self) -> Result<()> {
        // Stop session manager
        self.session_manager.stop().await?;

        // Clear session
        {
            let mut session_guard = self.session.write().await;
            *session_guard = None;
        }

        self.session_manager.set_session(None).await;

        // Disconnect via connection manager
        self.connection_manager.disconnect().await?;

        Ok(())
    }

    /// Get the current connection state
    pub async fn get_state(&self) -> ClientState {
        self.connection_manager.get_state().await
    }

    /// Send a command and wait for response
    pub async fn send_command(&self, command: &Message) -> Result<Message> {
        self.session_manager.send_command(command).await
    }

    /// Send a command with content
    pub async fn send_command_with_content<T>(&self, command_name: &str, content: T) -> Result<Message>
    where
        T: Into<serde_json::Value>,
    {
        self.session_manager.send_command_with_content(command_name, content).await
    }

    /// Send an event
    pub async fn send_event(&self, event: &Message) -> Result<()> {
        self.session_manager.send_event(event).await
    }

    /// Send an event with content
    pub async fn send_event_with_content<T>(&self, event_name: &str, content: T) -> Result<()>
    where
        T: Into<serde_json::Value>,
    {
        self.session_manager.send_event_with_content(event_name, content).await
    }

    /// Register an event handler
    pub async fn register_event_handler(&self, handler: Arc<dyn EventHandler>) -> Result<()> {
        self.session_manager.register_event_handler(handler).await
    }

    /// Subscribe to events
    pub async fn subscribe_to_events(&self) -> broadcast::Receiver<Option<Message>> {
        self.session_manager.subscribe_to_events().await
    }

    /// Get the current session
    pub async fn get_session(&self) -> Option<Session> {
        self.session.read().await.clone()
    }

    /// Get connection manager (for advanced use cases)
    pub fn connection_manager(&self) -> &Arc<ConnectionManager> {
        &self.connection_manager
    }

    /// Get session manager (for advanced use cases)
    pub fn session_manager(&self) -> &Arc<SessionManager> {
        &self.session_manager
    }
}

impl std::fmt::Debug for MCPClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MCPClient")
            .field("config", &self.config)
            .field("connection_manager", &"ConnectionManager")
            .field("session_manager", &"SessionManager")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config_default() {
        let config = ClientConfig::default();
        assert_eq!(config.server_address, "127.0.0.1:8080");
        assert_eq!(config.max_reconnect_attempts, 3);
    }

    #[test]
    fn test_client_creation() {
        let config = ClientConfig::new()
            .with_server_address("localhost:9000")
            .with_max_reconnect_attempts(5);
        
        let client = MCPClient::new(config);
        assert_eq!(client.config.server_address, "localhost:9000");
        assert_eq!(client.config.max_reconnect_attempts, 5);
    }

    #[test]
    fn test_client_from_env() {
        let client = MCPClient::from_env();
        assert!(!client.config.server_address.is_empty());
    }
} 