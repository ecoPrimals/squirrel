// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! WebSocket connection types and handler for plugin management real-time updates.

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use super::PluginManagementAPI;

/// WebSocket connection for real-time updates
#[derive(Debug, Clone)]
pub struct WebSocketConnection {
    /// Connection ID
    pub id: Uuid,
    /// Connection metadata
    pub metadata: HashMap<String, String>,
    /// Subscribed event types
    pub subscriptions: Vec<String>,
}

/// WebSocket handler for real-time plugin updates
pub struct PluginWebSocketHandler {
    pub(super) api: Arc<PluginManagementAPI>,
}

impl PluginWebSocketHandler {
    /// Creates a new WebSocket handler backed by the plugin management API.
    #[must_use]
    pub const fn new(api: Arc<PluginManagementAPI>) -> Self {
        Self { api }
    }

    /// Handle WebSocket connection
    ///
    /// # Errors
    ///
    /// Returns [`anyhow::Error`] if the connection cannot be registered.
    pub async fn handle_connection(&self, connection_id: Uuid) -> Result<()> {
        let connection = WebSocketConnection {
            id: connection_id,
            metadata: HashMap::new(),
            subscriptions: vec![
                "plugin.install".to_string(),
                "plugin.uninstall".to_string(),
                "plugin.start".to_string(),
                "plugin.stop".to_string(),
                "plugin.config.update".to_string(),
            ],
        };

        self.api
            .websocket_connections
            .write()
            .await
            .insert(connection_id, connection);

        Ok(())
    }

    /// Handle WebSocket disconnection
    ///
    /// # Errors
    ///
    /// Returns [`anyhow::Error`] if the connection state cannot be updated.
    pub async fn handle_disconnection(&self, connection_id: Uuid) -> Result<()> {
        self.api
            .websocket_connections
            .write()
            .await
            .remove(&connection_id);

        Ok(())
    }
}
