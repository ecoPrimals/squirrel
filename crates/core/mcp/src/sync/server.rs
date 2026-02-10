// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

// use crate::generated::mcp_sync::{SyncRequest, SyncResponse};
use crate::sync::state::StateSyncManager;
use crate::sync::{proto_to_state_change, state_change_to_proto};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use tonic::{Request, Response, Status};
use tracing::{info, error, debug, warn};

// Define the trait ourselves since it's not being generated properly
#[tonic::async_trait]
pub trait SyncService {
    async fn sync(&self, request: Request<SyncRequest>) -> Result<Response<SyncResponse>, Status>;
}

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
    async fn validate_client(&self, client_id: &str) -> Result<(), Status> {
        // In a production system, this would check if the client is registered
        // and authorized to perform sync operations
        // For now, we'll just check that the client ID is not empty
        if client_id.is_empty() {
            return Err(Status::invalid_argument("Client ID cannot be empty"));
        }
        Ok(())
    }
}

// Implement our own SyncService trait
#[tonic::async_trait]
impl SyncService for MCPSyncServer {
    async fn sync(
        &self,
        request: Request<SyncRequest>
    ) -> Result<Response<SyncResponse>, Status> {
        let req = request.into_inner();
        let client_id = req.client_id.clone();
        let client_version = req.last_known_version;
        
        info!("Received sync request from client {} with version {}", client_id, client_version);
        
        // Validate client
        if let Err(e) = self.validate_client(&client_id).await {
            error!("Client validation failed: {}", e);
            return Err(e);
        }
        
        // Process client changes
        let mut applied_changes = 0;
        let total_changes = req.local_changes.len();
        
        for proto_change in &req.local_changes {
            match proto_to_state_change(proto_change.clone()) {
                Ok(mut state_change) => {
                    // Ensure the change has a version assigned
                    if state_change.version == 0 {
                        // Assign a new version if not provided
                        state_change.version = self.increment_version().await;
                    }
                    
                    debug!("Applying change for context {}: {:?}", 
                          state_change.id, state_change.operation);
                    
                    if let Err(e) = self.state_manager.apply_change(state_change).await {
                        warn!("Failed to apply client change: {}", e);
                        // Continue with other changes
                    } else {
                        applied_changes += 1;
                    }
                },
                Err(e) => {
                    error!("Failed to convert proto change: {}", e);
                    // Continue with other changes
                }
            }
        }
        
        info!("Applied {}/{} changes from client {}", applied_changes, total_changes, client_id);
        
        // Get changes to send back to client
        let mut remote_changes = Vec::new();
        let server_changes = match self.state_manager.get_changes_since(client_version).await {
            Ok(changes) => changes,
            Err(e) => {
                error!("Failed to get changes since version {}: {}", client_version, e);
                return Err(Status::internal(format!("Failed to retrieve changes: {}", e)));
            }
        };
        
        for state_change in server_changes {
            match state_change_to_proto(&state_change) {
                Ok(proto_change) => remote_changes.push(proto_change),
                Err(e) => {
                    error!("Failed to convert state change to proto: {}", e);
                    // Continue with other changes
                }
            }
        }
        
        // Get current server version
        let current_version = self.state_manager.get_current_version().await
            .map_err(|e| Status::internal(format!("Failed to get current version: {}", e)))?;
        
        // Update client's last known version
        self.update_client_version(&client_id, current_version).await;
        
        info!("Sending {} changes to client {}", remote_changes.len(), client_id);
        
        // Build response
        let response = SyncResponse {
            current_server_version: current_version,
            remote_changes,
            success: true,
            error_message: String::new(),
        };
        
        Ok(Response::new(response))
    }
} 