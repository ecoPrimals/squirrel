//! Route definitions for the legacy API server
//!
//! This module constructs all HTTP routes and combines them with middleware
//! following warp's filter composition pattern.

use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Filter;

use crate::ecosystem::EcosystemManager;
use crate::shutdown::ShutdownManager;
use crate::MetricsCollector;

use super::filters::*;
use super::handlers::*;
use super::state::{increment_request_count, ServerState};

/// Build all API routes with middleware
pub fn build_routes(
    state: Arc<RwLock<ServerState>>,
    ecosystem_manager: Arc<EcosystemManager>,
    metrics_collector: Arc<MetricsCollector>,
    shutdown_manager: Arc<ShutdownManager>,
    base_url: String,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    // Health check routes
    let health = warp::path!("health")
        .and(warp::get())
        .and(with_state(state.clone()))
        .and(with_ecosystem_manager(ecosystem_manager.clone()))
        .and_then(handle_health_check);

    let health_live = warp::path!("health" / "live")
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(handle_health_live);

    let health_ready = warp::path!("health" / "ready")
        .and(warp::get())
        .and(with_state(state.clone()))
        .and(with_ecosystem_manager(ecosystem_manager.clone()))
        .and_then(handle_health_ready);

    // Ecosystem API endpoints
    let ecosystem_status = warp::path!("api" / "v1" / "ecosystem" / "status")
        .and(warp::get())
        .and(with_ecosystem_manager(ecosystem_manager.clone()))
        .and_then(handle_ecosystem_status);

    let service_mesh_status = warp::path!("api" / "v1" / "service-mesh" / "status")
        .and(warp::get())
        .and(with_ecosystem_manager(ecosystem_manager.clone()))
        .and_then(handle_service_mesh_status);

    let primals_list = warp::path!("api" / "v1" / "primals")
        .and(warp::get())
        .and(with_ecosystem_manager(ecosystem_manager.clone()))
        .and(with_base_url(base_url.clone()))
        .and_then(handle_primals_list);

    let primal_status = warp::path!("api" / "v1" / "primals" / String)
        .and(warp::get())
        .and(with_ecosystem_manager(ecosystem_manager.clone()))
        .and(with_base_url(base_url.clone()))
        .and_then(handle_primal_status);

    let metrics = warp::path!("api" / "v1" / "metrics")
        .and(warp::get())
        .and(with_state(state.clone()))
        .and(with_metrics_collector(metrics_collector.clone()))
        .and_then(handle_metrics);

    let services = warp::path!("api" / "v1" / "services")
        .and(warp::get())
        .and(with_ecosystem_manager(ecosystem_manager.clone()))
        .and(with_base_url(base_url))
        .and_then(handle_services);

    // Songbird integration endpoints
    let songbird_register = warp::path!("api" / "v1" / "songbird" / "register")
        .and(warp::post())
        .and(with_ecosystem_manager(ecosystem_manager.clone()))
        .and_then(handle_songbird_register);

    let songbird_heartbeat = warp::path!("api" / "v1" / "songbird" / "heartbeat")
        .and(warp::post())
        .and(with_state(state.clone()))
        .and_then(handle_songbird_heartbeat);

    // Shutdown endpoint
    let shutdown = warp::path!("api" / "v1" / "shutdown")
        .and(warp::post())
        .and(with_shutdown_manager(shutdown_manager))
        .and_then(handle_shutdown);

    // Request counting middleware
    let request_counter = warp::any()
        .and(with_state(state.clone()))
        .and_then(|state: Arc<RwLock<ServerState>>| async move {
            increment_request_count(state).await;
            Ok::<_, warp::Rejection>(())
        })
        .untuple_one();

    // Combine all routes with middleware
    request_counter
        .and(
            health
                .or(health_live)
                .or(health_ready)
                .or(ecosystem_status)
                .or(service_mesh_status)
                .or(primals_list)
                .or(primal_status)
                .or(metrics)
                .or(services)
                .or(songbird_register)
                .or(songbird_heartbeat)
                .or(shutdown),
        )
        .with(warp::cors().allow_any_origin())
        .with(warp::log("api"))
}

