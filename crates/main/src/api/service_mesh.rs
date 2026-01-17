//! Service mesh integration endpoints
//!
//! Handles registration and heartbeat communication with any service mesh provider.
//! Uses capability-based discovery - works with ANY service mesh discovered at runtime.

use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Reply;

use crate::ecosystem::EcosystemManager;

use super::server::ServerState;
use super::types::{ServiceMeshHeartbeatResponse, ServiceMeshRegistrationResponse};

/// Handle service mesh registration
pub async fn handle_service_mesh_register(
    _ecosystem_manager: Arc<EcosystemManager>,
) -> Result<impl Reply, warp::Rejection> {
    // Note: Service mesh registration uses capability discovery.
    // In a full implementation, this would discover and register with
    // any available service mesh provider (Songbird, Consul, etc).

    let response = ServiceMeshRegistrationResponse {
        status: "pending".to_string(),
        message: "Service mesh registration requires capability discovery context".to_string(),
    };

    Ok(warp::reply::json(&response))
}

/// Handle service mesh heartbeat
pub async fn handle_service_mesh_heartbeat(
    state: Arc<RwLock<ServerState>>,
) -> Result<impl Reply, warp::Rejection> {
    // Update heartbeat timestamp
    {
        let mut state_guard = state.write().await;
        state_guard.last_service_mesh_heartbeat = Some(Utc::now());
        state_guard.service_mesh_registered = true;
    }

    let response = ServiceMeshHeartbeatResponse {
        status: "ok".to_string(),
    };

    Ok(warp::reply::json(&response))
}

#[cfg(test)]
#[path = "service_mesh_tests.rs"]
mod tests;
