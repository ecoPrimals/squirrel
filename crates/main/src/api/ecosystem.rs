//! Ecosystem integration endpoint handlers
//!
//! Handles ecosystem status, primal discovery, and service mesh integration.

use std::collections::HashMap;
use std::sync::Arc;
use warp::Reply;

use crate::ecosystem::EcosystemManager;

use super::types::{
    CrossPrimalCommunicationResponse, EcosystemStatusResponse, LoadBalancingResponse,
    PrimalStatusResponse, ServiceInfo, ServiceMeshIntegrationStatus, ServiceMeshStatusResponse,
    ServicesResponse,
};

/// Handle ecosystem status endpoint
pub async fn handle_ecosystem_status(
    ecosystem_manager: Arc<EcosystemManager>,
) -> Result<impl Reply, warp::Rejection> {
    // registry_manager removed - use ecosystem discovery
    // TODO: Implement via ecosystem discovery
    let registered_primals: Vec<String> = Vec::new();
    let capabilities: HashMap<String, Vec<String>> = HashMap::new();

    let response = EcosystemStatusResponse {
        registered_primals,
        capabilities,
    };

    Ok(warp::reply::json(&response))
}

/// Handle service mesh status endpoint
pub async fn handle_service_mesh_status(
    _ecosystem_manager: Arc<EcosystemManager>,
) -> Result<impl Reply, warp::Rejection> {
    // Simplified response - in full implementation would query actual mesh status
    let response = ServiceMeshStatusResponse {
        load_balancing: LoadBalancingResponse {
            active: true,
            algorithm: "round_robin".to_string(),
        },
        cross_primal_communication: CrossPrimalCommunicationResponse {
            enabled: true,
            protocol: "http".to_string(),
        },
    };

    Ok(warp::reply::json(&response))
}

/// Handle primals list endpoint
pub async fn handle_primals_list(
    _ecosystem_manager: Arc<EcosystemManager>,
    _base_url: String,
) -> Result<impl Reply, warp::Rejection> {
    // registry_manager removed - use ecosystem discovery
    // TODO: Implement via ecosystem discovery
    let primals: Vec<PrimalStatusResponse> = Vec::new();
    let response = PrimalsResponse { primals };
    Ok(warp::reply::json(&response))
}

/// Handle primal status endpoint for a specific primal
pub async fn handle_primal_status(
    primal_name: String,
    _ecosystem_manager: Arc<EcosystemManager>,
    _base_url: String,
) -> Result<impl Reply, warp::Rejection> {
    // registry_manager removed - use ecosystem discovery
    // TODO: Implement via ecosystem discovery
    let error = serde_json::json!({
        "error": "Registry manager removed - ecosystem discovery not yet implemented",
        "primal": primal_name,
    });
    Ok(warp::reply::json(&error))
}

/// Handle services list endpoint
pub async fn handle_services(
    ecosystem_manager: Arc<EcosystemManager>,
    _base_url: String,
) -> Result<impl Reply, warp::Rejection> {
    // registry_manager removed - use ecosystem discovery
    let services_data: Vec<serde_json::Value> = Vec::new(); // TODO: Implement via ecosystem discovery

    let services: Vec<ServiceInfo> = services_data
        .iter()
        .map(|s| ServiceInfo {
            name: format!("{:?}", s.primal_type),
            endpoint: s.endpoint.to_string(),
            health: format!("{:?}", s.health_status),
        })
        .collect();

    let response = ServicesResponse {
        services,
        service_mesh_integration: ServiceMeshIntegrationStatus {
            songbird_registered: true,
            health_reporting_active: true,
        },
    };

    Ok(warp::reply::json(&response))
}

/// Handle tower discovery endpoint
///
/// Discovers other Squirrel instances through Songbird's capability system.
///
/// # Primal Self-Knowledge
///
/// This endpoint:
/// 1. Queries Songbird for all Squirrel instances
/// 2. Filters out THIS instance
/// 3. Returns information about OTHER instances
///
/// GET /api/ecosystem/towers
pub async fn handle_towers_discovery(
    ecosystem_manager: Arc<EcosystemManager>,
) -> Result<impl Reply, warp::Rejection> {
    use crate::ecosystem::EcosystemPrimalType;
    use serde::{Deserialize, Serialize};

    /// Tower information (another Squirrel instance)
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TowerInfo {
        /// Unique tower ID
        tower_id: String,
        /// Tower endpoint URL
        endpoint: String,
        /// GPU capabilities (if available)
        gpu_capabilities: Option<GpuCapabilities>,
        /// Tower health status
        health: String,
        /// Tower metadata
        metadata: HashMap<String, String>,
    }

    /// GPU capabilities of a tower
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct GpuCapabilities {
        /// Number of GPUs
        gpu_count: u32,
        /// Total VRAM in GB
        vram_total_gb: u32,
        /// GPU model
        gpu_model: Option<String>,
        /// GPU vendor
        gpu_vendor: Option<String>,
        /// Supports model splitting
        supports_model_splitting: bool,
    }

    /// Response for tower discovery
    #[derive(Debug, Serialize)]
    struct TowersResponse {
        /// List of discovered towers (excluding THIS instance)
        towers: Vec<TowerInfo>,
        /// Total number of towers (including THIS instance)
        total_count: usize,
        /// This instance's tower ID
        this_tower_id: String,
    }

    tracing::info!("Discovering towers through Songbird capability system");

    // Query Songbird for all Squirrel instances
    let discovered_services = ecosystem_manager
        .find_services_by_type(EcosystemPrimalType::Squirrel)
        .await
        .map_err(|e| {
            tracing::error!("Failed to discover Squirrel instances: {}", e);
            warp::reject::reject()
        })?;

    tracing::debug!(
        "Found {} Squirrel instance(s) in ecosystem",
        discovered_services.len()
    );

    // Get THIS instance's ID
    let this_tower_id = ecosystem_manager.config.service_id.clone();

    // Convert discovered services to TowerInfo, excluding THIS instance
    let mut towers = Vec::new();
    for service in &discovered_services {
        // Skip THIS instance
        if service.service_id.as_ref() == this_tower_id {
            tracing::debug!("Skipping this instance: {}", this_tower_id);
            continue;
        }

        // Parse GPU capabilities from metadata
        let gpu_count: u32 = service
            .metadata
            .get("gpu_count")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let gpu_capabilities = if gpu_count > 0 {
            let vram_total_gb: u32 = service
                .metadata
                .get("vram_total_gb")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            Some(GpuCapabilities {
                gpu_count,
                vram_total_gb,
                gpu_model: service
                    .metadata
                    .get("gpu_model")
                    .map(std::string::ToString::to_string),
                gpu_vendor: service
                    .metadata
                    .get("gpu_vendor")
                    .map(std::string::ToString::to_string),
                supports_model_splitting: vram_total_gb >= 8,
            })
        } else {
            None
        };

        towers.push(TowerInfo {
            tower_id: service.service_id.to_string(),
            endpoint: service.endpoint.to_string(),
            gpu_capabilities,
            health: format!("{:?}", service.health_status),
            metadata: service
                .metadata
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        });
    }

    tracing::info!(
        "Discovered {} other tower(s) (total: {})",
        towers.len(),
        discovered_services.len()
    );

    let response = TowersResponse {
        towers,
        total_count: discovered_services.len(),
        this_tower_id,
    };

    Ok(warp::reply::json(&response))
}

#[cfg(test)]
#[path = "ecosystem_tests.rs"]
mod tests;
