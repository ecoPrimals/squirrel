// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! JSON-RPC Sync Service
//!
//! Provides sync server implementation using JSON-RPC handler pattern.

use crate::error::MCPError;
use crate::sync::state::StateSyncManager;
use crate::sync::{proto_to_state_change, state_change_to_proto};
use crate::sync::json_rpc_types::{SyncRequest, SyncResponse};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Server implementation for MCP synchronization
#[derive(Clone, Debug)]
pub struct MCPSyncServer {
    /// State manager for handling state changes
    state_manager: Arc<StateSyncManager>,
    /// Map of client IDs to their last known version
    client_versions: Arc<RwLock<HashMap<String, u64>>>,
    /// Current server version
    version: Arc<RwLock<u64>>,
}

impl MCPSyncServer {
    /// Creates a new sync server instance
    pub fn new(state_manager: Arc<StateSyncManager>) -> Self {
        Self {
            state_manager,
            client_versions: Arc::new(RwLock::new(HashMap::new())),
            version: Arc::new(RwLock::new(0)),
        }
    }

    /// Increments and returns the server version
    async fn increment_version(&self) -> u64 {
        let mut version = self.version.write().await;
        *version += 1;
        *version
    }

    /// Updates the version for a client
    async fn update_client_version(&self, client_id: &str, version: u64) {
        let mut client_versions = self.client_versions.write().await;
        client_versions.insert(client_id.to_string(), version);
    }

    /// Gets the last known version for a client
    async fn get_client_version(&self, client_id: &str) -> u64 {
        let client_versions = self.client_versions.read().await;
        *client_versions.get(client_id).unwrap_or(&0)
    }

    /// Validates that a client is registered and authorized to sync
    async fn validate_client(&self, client_id: &str) -> Result<(), MCPError> {
        if client_id.is_empty() {
            return Err(MCPError::InvalidArgument("Client ID cannot be empty".to_string()));
        }
        Ok(())
    }

    /// Handle a JSON-RPC sync request.
    ///
    /// Accepts a JSON-RPC request value, extracts the sync params, processes the sync,
    /// and returns a JSON-RPC response value.
    pub async fn handle_json_rpc_request(
        &self,
        request: serde_json::Value,
    ) -> Result<serde_json::Value, MCPError> {
        let params = request
            .get("params")
            .and_then(|p| p.as_object())
            .ok_or_else(|| MCPError::InvalidArgument("Missing params".to_string()))?;

        let req: SyncRequest = serde_json::from_value(serde_json::to_value(params)?)
            .map_err(|e| MCPError::InvalidArgument(format!("Invalid sync params: {}", e)))?;

        let response = self.sync_impl(req).await?;

        let id = request.get("id").cloned().unwrap_or(serde_json::Value::Null);
        let result = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": serde_json::to_value(&response).map_err(|e| MCPError::Serialization(e.to_string()))?
        });

        Ok(result)
    }

    /// Core sync logic (same as before, but returns SyncResponse directly)
    async fn sync_impl(&self, req: SyncRequest) -> Result<SyncResponse, MCPError> {
        let client_id = req.client_id.clone();
        let client_version = req.last_known_version;

        info!(
            "Received sync request from client {} with version {}",
            client_id, client_version
        );

        self.validate_client(&client_id).await?;

        let mut applied_changes = 0;
        let total_changes = req.local_changes.len();

        for proto_change in &req.local_changes {
            match proto_to_state_change(proto_change.clone()) {
                Ok(mut state_change) => {
                    if state_change.version == 0 {
                        state_change.version = self.increment_version().await;
                    }

                    debug!(
                        "Applying change for context {}: {:?}",
                        state_change.id, state_change.operation
                    );

                    if let Err(e) = self.state_manager.apply_change(state_change).await {
                        warn!("Failed to apply client change: {}", e);
                    } else {
                        applied_changes += 1;
                    }
                }
                Err(e) => {
                    error!("Failed to convert proto change: {}", e);
                }
            }
        }

        info!(
            "Applied {}/{} changes from client {}",
            applied_changes, total_changes, client_id
        );

        let mut remote_changes = Vec::new();
        let server_changes = self
            .state_manager
            .get_changes_since(client_version)
            .await
            .map_err(|e| MCPError::InternalError(format!("Failed to retrieve changes: {}", e)))?;

        for state_change in server_changes {
            match state_change_to_proto(&state_change) {
                Ok(proto_change) => remote_changes.push(proto_change),
                Err(e) => {
                    error!("Failed to convert state change to proto: {}", e);
                }
            }
        }

        let current_version = self
            .state_manager
            .get_current_version()
            .await
            .map_err(|e| MCPError::InternalError(format!("Failed to get current version: {}", e)))?;

        self.update_client_version(&client_id, current_version).await;

        info!("Sending {} changes to client {}", remote_changes.len(), client_id);

        Ok(SyncResponse {
            current_server_version: current_version,
            remote_changes,
            success: true,
            error_message: String::new(),
        })
    }
}
