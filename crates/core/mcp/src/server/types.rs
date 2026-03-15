// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Type definitions for MCP Server

use crate::error::Result;
use crate::message::Message;
use crate::session::Session;
use crate::transport::Transport;
use serde_json::Value;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

/// MCP Server state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    /// Server is stopped
    Stopped,
    /// Server is starting
    Starting,
    /// Server is running
    Running,
    /// Server is stopping
    Stopping,
    /// Server failed to start
    Failed,
}

/// Command handler for processing command messages
pub trait CommandHandler: Send + Sync + std::fmt::Debug {
    /// Handle a command message
    fn handle_command<'a>(
        &'a self,
        command: &'a Message,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<Message>>> + Send + 'a>>;

    /// Get the command types this handler can process
    fn supported_commands(&self) -> Vec<String>;

    /// Clone the handler into a new box
    fn clone_box(&self) -> Box<dyn CommandHandler>;
}

impl Clone for Box<dyn CommandHandler> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Connection handler for managing client connections
pub trait ConnectionHandler: Send + Sync {
    /// Handle a new client connection
    fn handle_connection<'a>(
        &'a self,
        client: ClientConnection,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>>;

    /// Handle client disconnection
    fn handle_disconnection<'a>(
        &'a self,
        client_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>>;

    /// Clone the handler into a new box
    fn clone_box(&self) -> Box<dyn ConnectionHandler>;
}

impl Clone for Box<dyn ConnectionHandler> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Client connection information
#[derive(Clone)]
pub struct ClientConnection {
    /// Client ID
    pub client_id: String,

    /// Client address
    pub address: SocketAddr,

    /// Client session
    pub session: Arc<Session>,

    /// Client transport
    pub transport: Arc<dyn Transport>,

    /// Connection time
    pub connected_at: chrono::DateTime<chrono::Utc>,

    /// Client metadata
    pub metadata: HashMap<String, Value>,
}

// Add manual Debug implementation
impl std::fmt::Debug for ClientConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientConnection")
            .field("client_id", &self.client_id)
            .field("address", &self.address)
            .field("session", &self.session)
            .field("connected_at", &self.connected_at)
            .field("metadata", &self.metadata)
            .field("transport", &"<Transport>")
            .finish()
    }
}

