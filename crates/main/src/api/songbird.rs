//! Songbird service mesh integration endpoints
//!
//! Handles registration and heartbeat communication with Songbird orchestrator.

use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Reply;

use crate::ecosystem::EcosystemManager;

use super::server::ServerState;
use super::types::{SongbirdHeartbeatResponse, SongbirdRegistrationResponse};

/// Handle Songbird registration
pub async fn handle_songbird_register(
    _ecosystem_manager: Arc<EcosystemManager>,
) -> Result<impl Reply, warp::Rejection> {
    // Note: Songbird registration requires a SquirrelPrimalProvider instance.
    // In a full implementation, this would be passed through the API context.
    // For now, we return a placeholder response indicating the intent.

    let response = SongbirdRegistrationResponse {
        status: "pending".to_string(),
        message: "Songbird registration requires primal provider context".to_string(),
    };

    Ok(warp::reply::json(&response))
}

/// Handle Songbird heartbeat
pub async fn handle_songbird_heartbeat(
    state: Arc<RwLock<ServerState>>,
) -> Result<impl Reply, warp::Rejection> {
    // Update heartbeat timestamp
    {
        let mut state_guard = state.write().await;
        state_guard.last_songbird_heartbeat = Some(Utc::now());
        state_guard.service_mesh_registered = true;
    }

    let response = SongbirdHeartbeatResponse {
        status: "ok".to_string(),
    };

    Ok(warp::reply::json(&response))
}

#[cfg(test)]
#[path = "songbird_tests.rs"]
mod tests;
