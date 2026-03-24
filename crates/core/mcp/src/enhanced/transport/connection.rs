// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Connection registry and lifecycle updates.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::info;

use crate::error::{Result, types::MCPError};

use super::types::{ConnectionInfo, ConnectionState, TransportConfig};

/// Connection Manager - Manages all connections
#[derive(Debug)]
pub struct ConnectionManager {
    pub(super) config: Arc<TransportConfig>,
    pub(super) connections: Arc<RwLock<HashMap<String, ConnectionInfo>>>,
    pub(super) connection_handlers: Arc<RwLock<HashMap<String, super::types::ConnectionHandler>>>,
}

impl ConnectionManager {
    pub async fn new(config: Arc<TransportConfig>) -> Result<Self> {
        Ok(Self {
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
            connection_handlers: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting Connection Manager");
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        info!("Stopping Connection Manager");
        Ok(())
    }

    pub async fn add_connection(&self, connection: ConnectionInfo) -> Result<()> {
        let mut connections = self.connections.write().await;
        connections.insert(connection.id.clone(), connection);
        Ok(())
    }

    pub async fn remove_connection(&self, connection_id: &str) -> Result<()> {
        let mut connections = self.connections.write().await;
        connections.remove(connection_id);
        Ok(())
    }

    pub async fn get_connection(&self, connection_id: &str) -> Result<ConnectionInfo> {
        let connections = self.connections.read().await;
        connections
            .get(connection_id)
            .cloned()
            .ok_or_else(|| MCPError::NotFound(format!("Connection not found: {}", connection_id)))
    }

    pub async fn list_connections(&self) -> Result<Vec<ConnectionInfo>> {
        let connections = self.connections.read().await;
        Ok(connections.values().cloned().collect())
    }

    pub async fn update_connection_state(
        &self,
        connection_id: &str,
        state: ConnectionState,
    ) -> Result<()> {
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.get_mut(connection_id) {
            connection.state = state;
            connection.last_activity = chrono::Utc::now();
        }
        Ok(())
    }
}
