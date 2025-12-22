//! HTTP request handlers for legacy API endpoints
//!
//! This module contains all the handler functions that process incoming
//! HTTP requests and generate responses for the legacy API endpoints.

use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Reply;

use crate::ecosystem::EcosystemManager;
use crate::shutdown::ShutdownManager;
use crate::universal::{CircuitBreakerStatus, LoadBalancingStatus, ServiceMeshStatus};
use crate::MetricsCollector;

use super::state::ServerState;
use super::types::*;

/// Handle comprehensive health check with ecosystem integration
pub async fn handle_health_check(
    state: Arc<RwLock<ServerState>>,
    ecosystem_manager: Arc<EcosystemManager>,
) -> Result<impl Reply, warp::Rejection> {
    let state = state.read().await;
    let uptime = Utc::now()
        .signed_duration_since(state.started_at)
        .num_seconds() as u64;

    let ecosystem_status = ecosystem_manager.get_ecosystem_status().await;
    let manager_status = ecosystem_manager.get_manager_status().await;

    let mut metadata = HashMap::new();
    metadata.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
    metadata.insert("primal_type".to_string(), "ai".to_string());
    metadata.insert(
        "capabilities".to_string(),
        "ai_coordination,mcp_protocol,context_awareness".to_string(),
    );

    let response = HealthResponse {
        status: "healthy".to_string(),
        timestamp: Utc::now(),
        uptime_seconds: uptime,
        service_mesh: ServiceMeshHealthStatus {
            registered: state.service_mesh_registered,
            last_heartbeat: state.last_songbird_heartbeat,
            connection_status: if state.service_mesh_registered {
                "connected"
            } else {
                "disconnected"
            }
            .to_string(),
            load_balancing_active: state.service_mesh_registered,
        },
        ecosystem: EcosystemHealthStatus {
            discovered_primals: ecosystem_status.discovered_services.len() as u32,
            active_integrations: ecosystem_status.active_integrations,
            cross_primal_status: ecosystem_status.status,
            ecosystem_health_score: manager_status.health_status.health_score,
        },
        metadata,
    };

    Ok(warp::reply::json(&response))
}

/// Handle liveness probe (simple check that service is running)
pub async fn handle_health_live(
    state: Arc<RwLock<ServerState>>,
) -> Result<impl Reply, warp::Rejection> {
    let state = state.read().await;
    let uptime = Utc::now()
        .signed_duration_since(state.started_at)
        .num_seconds() as u64;

    let mut metadata = HashMap::new();
    metadata.insert("uptime_seconds".to_string(), uptime.to_string());
    metadata.insert(
        "active_connections".to_string(),
        state.active_connections.to_string(),
    );

    let response = HealthResponse {
        status: "alive".to_string(),
        timestamp: Utc::now(),
        uptime_seconds: uptime,
        service_mesh: ServiceMeshHealthStatus {
            registered: state.service_mesh_registered,
            last_heartbeat: state.last_songbird_heartbeat,
            connection_status: "live".to_string(),
            load_balancing_active: false,
        },
        ecosystem: EcosystemHealthStatus {
            discovered_primals: 0,
            active_integrations: vec![],
            cross_primal_status: "live".to_string(),
            ecosystem_health_score: 1.0,
        },
        metadata,
    };

    Ok(warp::reply::json(&response))
}

/// Handle readiness probe (check if service is ready to accept traffic)
pub async fn handle_health_ready(
    state: Arc<RwLock<ServerState>>,
    ecosystem_manager: Arc<EcosystemManager>,
) -> Result<impl Reply, warp::Rejection> {
    let state = state.read().await;
    let manager_status = ecosystem_manager.get_manager_status().await;

    let ready =
        manager_status.status == "initialized" && manager_status.health_status.health_score > 0.5;

    let status = if ready { "ready" } else { "not_ready" };

    let response = HealthResponse {
        status: status.to_string(),
        timestamp: Utc::now(),
        uptime_seconds: Utc::now()
            .signed_duration_since(state.started_at)
            .num_seconds() as u64,
        service_mesh: ServiceMeshHealthStatus {
            registered: state.service_mesh_registered,
            last_heartbeat: state.last_songbird_heartbeat,
            connection_status: if ready { "ready" } else { "not_ready" }.to_string(),
            load_balancing_active: ready && state.service_mesh_registered,
        },
        ecosystem: EcosystemHealthStatus {
            discovered_primals: 0,
            active_integrations: vec![],
            cross_primal_status: status.to_string(),
            ecosystem_health_score: manager_status.health_status.health_score,
        },
        metadata: HashMap::new(),
    };

    Ok(warp::reply::json(&response))
}

/// Handle ecosystem status request
pub async fn handle_ecosystem_status(
    ecosystem_manager: Arc<EcosystemManager>,
) -> Result<impl Reply, warp::Rejection> {
    let ecosystem_status = ecosystem_manager.get_ecosystem_status().await;
    let manager_status = ecosystem_manager.get_manager_status().await;

    let mut metadata = HashMap::new();
    metadata.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
    metadata.insert("primal_type".to_string(), "ai".to_string());
    metadata.insert(
        "service_mesh_integration".to_string(),
        "songbird".to_string(),
    );

    let response = EcosystemStatusResponse {
        status: ecosystem_status.status,
        timestamp: Utc::now(),
        active_primals: ecosystem_status
            .discovered_services
            .iter()
            .map(|s| s.service_id.to_string())
            .collect(),
        service_discovery: if ecosystem_status.discovered_services.is_empty() {
            "discovering".to_string()
        } else {
            "active".to_string()
        },
        service_mesh_status: ServiceMeshStatusResponse {
            enabled: true,
            registered: manager_status.status == "initialized",
            load_balancing: LoadBalancingResponse {
                enabled: true,
                algorithm: "round_robin".to_string(),
                health_score: manager_status.health_status.health_score,
            },
            cross_primal_communication: CrossPrimalCommunicationResponse {
                enabled: true,
                active_connections: ecosystem_status.active_integrations.len() as u32,
                supported_protocols: vec!["http".to_string(), "grpc".to_string()],
            },
        },
        metadata,
    };

    Ok(warp::reply::json(&response))
}

/// Handle service mesh status request
pub async fn handle_service_mesh_status(
    ecosystem_manager: Arc<EcosystemManager>,
) -> Result<impl Reply, warp::Rejection> {
    let manager_status = ecosystem_manager.get_manager_status().await;

    let response = ServiceMeshStatus {
        registered: manager_status.status == "initialized",
        connected: true,
        songbird_endpoint: Some({
            use universal_constants::{builders, network};
            let port = network::get_port_from_env("SONGBIRD_PORT", 8080);
            builders::localhost_http(port)
        }),
        registration_time: manager_status.last_registration,
        last_heartbeat: Some(Utc::now()),
        mesh_version: "1.0.0".to_string(),
        instance_id: "squirrel-1".to_string(),
        load_balancing_enabled: true,
        circuit_breaker_status: CircuitBreakerStatus {
            open: false,
            failures: 0,
            last_failure: None,
            next_retry: None,
        },
        last_registration: manager_status.last_registration,
        mesh_health: manager_status.health_status.health_score.to_string(),
        active_connections: manager_status.active_registrations.len() as u32,
        load_balancing: LoadBalancingStatus {
            enabled: true,
            healthy: true,
            active_connections: manager_status.active_registrations.len() as u32,
            algorithm: "round_robin".to_string(),
            health_score: manager_status.health_status.health_score,
            last_check: Utc::now(),
        },
    };

    Ok(warp::reply::json(&response))
}

/// Handle list all discovered primals
pub async fn handle_primals_list(
    ecosystem_manager: Arc<EcosystemManager>,
    base_url: String,
) -> Result<impl Reply, warp::Rejection> {
    let ecosystem_status = ecosystem_manager.get_ecosystem_status().await;

    let mut primals = vec![PrimalStatusResponse {
        name: "squirrel".to_string(),
        status: "active".to_string(),
        timestamp: Utc::now(),
        endpoints: vec![base_url.clone()],
        metadata: {
            let mut map = HashMap::new();
            map.insert("type".to_string(), "ai".to_string());
            map.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
            map.insert(
                "capabilities".to_string(),
                "ai_coordination,mcp_protocol,service_mesh_integration".to_string(),
            );
            map
        },
    }];

    // Add discovered primals
    for service in ecosystem_status.discovered_services {
        primals.push(PrimalStatusResponse {
            name: service.service_id.to_string(),
            status: "active".to_string(),
            timestamp: Utc::now(),
            endpoints: vec![service.endpoint.to_string()],
            metadata: service
                .metadata
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<HashMap<String, String>>(),
        });
    }

    Ok(warp::reply::json(&primals))
}

/// Handle get specific primal status
pub async fn handle_primal_status(
    primal_name: String,
    ecosystem_manager: Arc<EcosystemManager>,
    base_url: String,
) -> Result<impl Reply, warp::Rejection> {
    let ecosystem_status = ecosystem_manager.get_ecosystem_status().await;

    let (status, endpoints, metadata) = match primal_name.as_str() {
        "squirrel" => ("active".to_string(), vec![base_url.clone()], {
            let mut map = HashMap::new();
            map.insert("type".to_string(), "ai".to_string());
            map.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
            map.insert(
                "capabilities".to_string(),
                "ai_coordination,mcp_protocol,service_mesh_integration".to_string(),
            );
            map.insert("service_mesh".to_string(), "songbird".to_string());
            map
        }),
        _ => {
            if let Some(service) = ecosystem_status
                .discovered_services
                .iter()
                .find(|s| s.service_id.as_ref() == primal_name)
            {
                (
                    "active".to_string(),
                    vec![service.endpoint.to_string()],
                    service
                        .metadata
                        .iter()
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect::<HashMap<String, String>>(),
                )
            } else {
                ("not_found".to_string(), vec![], HashMap::new())
            }
        }
    };

    let response = PrimalStatusResponse {
        name: primal_name,
        status,
        timestamp: Utc::now(),
        endpoints,
        metadata,
    };

    Ok(warp::reply::json(&response))
}

/// Handle metrics request
pub async fn handle_metrics(
    state: Arc<RwLock<ServerState>>,
    metrics_collector: Arc<MetricsCollector>,
) -> Result<impl Reply, warp::Rejection> {
    let state_guard = state.read().await;
    let metrics = metrics_collector
        .get_all_metrics()
        .await
        .map_err(|_| warp::reject::not_found())?;

    let response = MetricsResponse {
        timestamp: Utc::now(),
        system: {
            let mut map = HashMap::new();
            map.insert("uptime".to_string(), "running".to_string());
            map.insert("memory_usage".to_string(), "normal".to_string());
            map.insert("cpu_usage".to_string(), "normal".to_string());
            map
        },
        application: {
            let mut map = HashMap::new();
            map.insert(
                "active_sessions".to_string(),
                metrics.metrics.len().to_string(),
            );
            map.insert(
                "requests_processed".to_string(),
                state_guard.request_count.to_string(),
            );
            map.insert("errors".to_string(), "0".to_string());
            map
        },
        performance: {
            let mut map = HashMap::new();
            map.insert("avg_response_time".to_string(), "100ms".to_string());
            map.insert("throughput".to_string(), "0 req/s".to_string());
            map.insert("error_rate".to_string(), "0%".to_string());
            map
        },
        ecosystem: {
            let mut map = HashMap::new();
            map.insert("service_mesh_health".to_string(), "healthy".to_string());
            map.insert("cross_primal_calls".to_string(), "0".to_string());
            map.insert("load_balancing_efficiency".to_string(), "100%".to_string());
            map
        },
    };

    Ok(warp::reply::json(&response))
}

/// Handle services list request
pub async fn handle_services(
    ecosystem_manager: Arc<EcosystemManager>,
    base_url: String,
) -> Result<impl Reply, warp::Rejection> {
    let ecosystem_status = ecosystem_manager.get_ecosystem_status().await;

    let mut services = vec![
        ServiceInfo {
            name: "Squirrel AI".to_string(),
            service_type: "HTTP API".to_string(),
            endpoints: vec![base_url.clone()],
            health: "healthy".to_string(),
            metadata: {
                let mut map = HashMap::new();
                map.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
                map.insert("primal".to_string(), "squirrel".to_string());
                map.insert(
                    "capabilities".to_string(),
                    "ai_coordination,mcp_protocol,service_mesh_integration".to_string(),
                );
                map
            },
        },
        ServiceInfo {
            name: "Service Discovery".to_string(),
            service_type: "Registry".to_string(),
            endpoints: vec![],
            health: if ecosystem_status.discovered_services.is_empty() {
                "discovering"
            } else {
                "active"
            }
            .to_string(),
            metadata: {
                let mut map = HashMap::new();
                map.insert("status".to_string(), "ecosystem_integration".to_string());
                map.insert("service_mesh".to_string(), "songbird".to_string());
                map
            },
        },
    ];

    // Add discovered services
    for service in ecosystem_status.discovered_services {
        services.push(ServiceInfo {
            name: service.service_id.to_string(),
            service_type: format!("{:?}", service.primal_type),
            endpoints: vec![service.endpoint.to_string()],
            health: "discovered".to_string(),
            metadata: service
                .metadata
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<HashMap<String, String>>(),
        });
    }

    let response = ServicesResponse {
        timestamp: Utc::now(),
        services,
        registry_status: "active".to_string(),
        service_mesh_status: ServiceMeshIntegrationStatus {
            enabled: true,
            provider: "songbird".to_string(),
            load_balancing: true,
            cross_primal_communication: true,
        },
    };

    Ok(warp::reply::json(&response))
}

/// Handle Songbird registration request
pub async fn handle_songbird_register(
    _ecosystem_manager: Arc<EcosystemManager>,
) -> Result<impl Reply, warp::Rejection> {
    // Registration is now handled by the ecosystem-api integration
    let response = SongbirdRegistrationResponse {
        success: true,
        message: "Registered with Songbird service mesh via ecosystem-api".to_string(),
        service_id: "squirrel-ai".to_string(),
        timestamp: Utc::now(),
    };

    Ok(warp::reply::json(&response))
}

/// Handle Songbird heartbeat request
pub async fn handle_songbird_heartbeat(
    state: Arc<RwLock<ServerState>>,
) -> Result<impl Reply, warp::Rejection> {
    let mut state = state.write().await;
    state.last_songbird_heartbeat = Some(Utc::now());
    state.service_mesh_registered = true;

    let response = SongbirdHeartbeatResponse {
        acknowledged: true,
        timestamp: Utc::now(),
        next_heartbeat: Utc::now() + chrono::Duration::seconds(30),
    };

    Ok(warp::reply::json(&response))
}

/// Handle graceful shutdown request
pub async fn handle_shutdown(
    shutdown_manager: Arc<ShutdownManager>,
) -> Result<impl Reply, warp::Rejection> {
    tracing::info!("Received shutdown signal");
    match shutdown_manager.request_shutdown().await {
        Ok(report) => {
            tracing::info!("Shutdown initiated successfully, report: {:?}", report);
            Ok(warp::reply::json(&ShutdownResponse { acknowledged: true }))
        }
        Err(e) => {
            tracing::error!("Failed to initiate shutdown: {}", e);
            Err(warp::reject::not_found())
        }
    }
}

