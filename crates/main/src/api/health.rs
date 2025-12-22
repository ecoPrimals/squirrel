//! Health check endpoint handlers
//!
//! Provides health, liveness, and readiness probes following ecosystem standards.

use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Reply;

use crate::ecosystem::EcosystemManager;

use super::server::ServerState;
use super::types::{EcosystemHealthStatus, HealthResponse, ServiceMeshHealthStatus};

/// Handle comprehensive health check with ecosystem integration
pub async fn handle_health_check(
    state: Arc<RwLock<ServerState>>,
    ecosystem_manager: Arc<EcosystemManager>,
) -> Result<impl Reply, warp::Rejection> {
    let state_guard = state.read().await;
    let uptime = Utc::now()
        .signed_duration_since(state_guard.started_at)
        .num_seconds();

    // Get discovered primals count from registry manager
    let discovered_primals = ecosystem_manager
        .registry_manager
        .get_discovered_services()
        .await
        .len() as u32;

    // Get active integrations from registry manager
    let active_integrations = ecosystem_manager
        .registry_manager
        .get_active_integrations()
        .await;

    // Calculate ecosystem health score (simplified)
    let ecosystem_health_score = if discovered_primals > 0 { 0.8 } else { 0.5 };

    let response = HealthResponse {
        status: "healthy".to_string(),
        timestamp: Utc::now(),
        uptime_seconds: uptime as u64,
        service_mesh: ServiceMeshHealthStatus {
            registered: state_guard.service_mesh_registered,
            last_heartbeat: state_guard.last_songbird_heartbeat,
            connection_status: if state_guard.service_mesh_registered {
                "connected".to_string()
            } else {
                "not_registered".to_string()
            },
            load_balancing_active: state_guard.service_mesh_registered,
        },
        ecosystem: EcosystemHealthStatus {
            discovered_primals,
            active_integrations,
            cross_primal_status: "operational".to_string(),
            ecosystem_health_score,
        },
        metadata: HashMap::new(),
    };

    Ok(warp::reply::json(&response))
}

/// Handle liveness probe (always returns 200 if running)
pub async fn handle_health_live(
    state: Arc<RwLock<ServerState>>,
) -> Result<impl Reply, warp::Rejection> {
    let _state_guard = state.read().await;

    let response = serde_json::json!({
        "status": "alive",
        "timestamp": Utc::now().to_rfc3339(),
    });

    Ok(warp::reply::json(&response))
}

/// Handle readiness probe (checks ecosystem connectivity)
pub async fn handle_health_ready(
    state: Arc<RwLock<ServerState>>,
    ecosystem_manager: Arc<EcosystemManager>,
) -> Result<impl Reply, warp::Rejection> {
    let state_guard = state.read().await;

    // Check if we have discovered any primals (indicates ecosystem connectivity)
    let discovered_count = ecosystem_manager
        .registry_manager
        .get_discovered_services()
        .await
        .len();

    let is_ready = discovered_count > 0 || !state_guard.service_mesh_registered;

    let response = serde_json::json!({
        "status": if is_ready { "ready" } else { "not_ready" },
        "timestamp": Utc::now().to_rfc3339(),
        "details": {
            "discovered_primals": discovered_count,
            "service_mesh_registered": state_guard.service_mesh_registered,
        },
    });

    Ok(warp::reply::json(&response))
}

#[cfg(test)]
#[path = "health_tests.rs"]
mod tests;
