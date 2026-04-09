// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
    routing::{Router, get, post},
};
use std::sync::Arc;

use crate::{
    AgentSpec, Error, McpRouter, McpTask, NodeSpec, PrimalCoordinator, Result, ScaleRequirements,
    SwarmManager, Task, ecosystem::EcosystemService, federation::FederationService,
    routing::McpRoutingService,
};

pub use crate::api_types::*;

/// API Server for the MCP Routing Service
pub struct ApiServer {
    ecosystem_service: Arc<EcosystemService>,
    routing_service: Arc<McpRoutingService>,
    federation_service: Arc<FederationService>,
    /// Wall-clock time when this API server was constructed (used for `/info` uptime).
    api_started_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
struct AppState {
    ecosystem: Arc<EcosystemService>,
    routing: Arc<McpRoutingService>,
    federation: Arc<FederationService>,
    api_started_at: chrono::DateTime<chrono::Utc>,
}

impl ApiServer {
    /// Create a new API server
    #[must_use]
    pub fn new(
        ecosystem_service: Arc<EcosystemService>,
        routing_service: Arc<McpRoutingService>,
        federation_service: Arc<FederationService>,
    ) -> Self {
        Self {
            ecosystem_service,
            routing_service,
            federation_service,
            api_started_at: chrono::Utc::now(),
        }
    }

    /// Create the API router
    pub fn create_router(&self) -> Router {
        let state = AppState {
            ecosystem: self.ecosystem_service.clone(),
            routing: self.routing_service.clone(),
            federation: self.federation_service.clone(),
            api_started_at: self.api_started_at,
        };

        Router::new()
            // Health and status endpoints
            .route("/health", get(health_check))
            .route("/status", get(get_status))
            .route("/info", get(get_info))
            // MCP routing endpoints
            .route("/api/v1/route", post(route_mcp_task))
            .route("/api/v1/agents", get(list_agents))
            .route("/api/v1/agents", post(register_agent))
            .route("/api/v1/stats", get(get_routing_stats))
            // Federation endpoints
            .route("/api/v1/federation", get(get_federation_info))
            .route("/api/v1/federation/join", post(join_federation))
            .route("/api/v1/federation/nodes", get(list_federation_nodes))
            .route("/api/v1/federation/scale", post(scale_federation))
            .route("/api/v1/federation/stats", get(get_federation_stats))
            // Primal coordination endpoints
            .route("/api/v1/primals/discover", post(discover_primals))
            .route("/api/v1/primals", get(list_discovered_primals))
            .route("/api/v1/coordinate", post(coordinate_task))
            // Task management endpoints
            .route("/api/v1/tasks", post(submit_task))
            .route("/api/v1/tasks/:task_id", get(get_task_status))
            // Administrative endpoints
            .route("/api/v1/shutdown", post(shutdown_service))
            .with_state(state)
        // Layers commented out to avoid compatibility issues
        // // .layer(CorsLayer::permissive())
        // // .layer(TraceLayer::new_for_http())
    }

    /// Start the API server
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if binding to the address or serving the HTTP stack fails.
    pub async fn start(&self, bind_addr: &str) -> Result<()> {
        let router = self.create_router();

        let listener = tokio::net::TcpListener::bind(bind_addr)
            .await
            .map_err(Error::Io)?;

        tracing::info!("Starting Squirrel MCP API server on {}", bind_addr);

        axum::serve(listener, router)
            .await
            .map_err(|e| Error::Io(std::io::Error::other(e)))?;

        Ok(())
    }
}

// Health and status handlers

/// Health check endpoint
async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let Ok(ecosystem_health) = state.ecosystem.health_check().await else {
        return Json(HealthResponse {
            status: "unhealthy".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            ecosystem_status: "Error".to_string(),
            version: crate::SQUIRREL_MCP_VERSION.to_string(),
        })
        .into_response();
    };

    let response = HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        ecosystem_status: format!("{ecosystem_health:?}"),
        version: crate::SQUIRREL_MCP_VERSION.to_string(),
    };

    Json(response).into_response()
}

/// Get routing status
async fn get_status(State(state): State<AppState>) -> impl IntoResponse {
    let Ok(ecosystem_health) = state.ecosystem.health_check().await else {
        return Json(StatusResponse {
            status: "inactive".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            ecosystem_health: "Error".to_string(),
            version: crate::SQUIRREL_MCP_VERSION.to_string(),
        })
        .into_response();
    };

    let response = StatusResponse {
        status: "active".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        ecosystem_health: format!("{ecosystem_health:?}"),
        version: crate::SQUIRREL_MCP_VERSION.to_string(),
    };

    Json(response).into_response()
}

/// Get system information
async fn get_info(State(state): State<AppState>) -> impl IntoResponse {
    let federation_stats = state.federation.get_federation_stats();
    let uptime = chrono::Utc::now().signed_duration_since(state.api_started_at);
    let uptime_secs = uptime.num_seconds().max(0);
    let uptime_str = format!("{uptime_secs}s");

    let response = InfoResponse {
        node_id: federation_stats.node_id,
        version: crate::SQUIRREL_MCP_VERSION.to_string(),
        uptime: uptime_str,
        federation_status: format!("{:?}", federation_stats.status),
    };

    Json(response).into_response()
}

// MCP routing handlers

/// Route MCP task
async fn route_mcp_task(
    State(state): State<AppState>,
    Json(task): Json<McpTask>,
) -> impl IntoResponse {
    let response = match state.routing.route_task(task).await {
        Ok(result) => result,
        Err(e) => return ApiError(e).into_response(),
    };

    Json(response).into_response()
}

/// List registered agents
async fn list_agents(State(state): State<AppState>) -> impl IntoResponse {
    let routing_stats = state.routing.get_stats();

    let response = AgentsResponse {
        total_agents: routing_stats.registered_agents,
        active_tasks: routing_stats.active_tasks,
        completed_tasks: routing_stats.completed_tasks,
        failed_tasks: routing_stats.failed_tasks,
        average_response_time: routing_stats.average_response_time,
    };

    Json(response).into_response()
}

/// Register agent
async fn register_agent(
    State(state): State<AppState>,
    Json(agent): Json<AgentSpec>,
) -> impl IntoResponse {
    let result = match state.routing.register_agent(agent) {
        Ok(()) => RegisterResponse {
            success: true,
            message: "Agent registered successfully".to_string(),
            agent_count: 1,
        },
        Err(e) => return ApiError(e).into_response(),
    };

    Json(result).into_response()
}

/// Get routing statistics
async fn get_routing_stats(State(state): State<AppState>) -> impl IntoResponse {
    let routing_stats = state.routing.get_stats();
    Json(routing_stats)
}

// Federation handlers

/// Get federation information
async fn get_federation_info(State(state): State<AppState>) -> impl IntoResponse {
    let federation_stats = state.federation.get_federation_stats();

    ResponseJson(federation_stats).into_response()
}

/// Join federation
async fn join_federation(
    State(state): State<AppState>,
    Json(request): Json<JoinFederationRequest>,
) -> impl IntoResponse {
    let node_id = request
        .metadata
        .get("node_id")
        .cloned()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let zone = request.metadata.get("zone").cloned();
    let capacity = request
        .metadata
        .get("capacity")
        .and_then(|s| s.parse().ok())
        .unwrap_or(100_u32);

    if let Some(ref requested_fed) = request.federation_id {
        let current = state.federation.get_federation_stats().federation_id;
        if requested_fed != &current {
            tracing::debug!(
                requested = %requested_fed,
                current = %current,
                "Join request federation_id does not match local federation; proceeding with local registration"
            );
        }
    }

    let spec = NodeSpec {
        id: node_id,
        region: request.region,
        zone,
        endpoint: request.node_endpoint,
        capabilities: request.capabilities,
        capacity,
    };

    match state.federation.federate_nodes(vec![spec]).await {
        Ok(result) => {
            let response = FederationResponse {
                success: matches!(result.status, crate::FederationStatus::Active),
                federation_id: result.federation_id,
                node_count: result.nodes_joined,
                status: format!("{:?}", result.status),
            };
            Json(response).into_response()
        }
        Err(e) => ApiError(e).into_response(),
    }
}

/// List federation nodes
async fn list_federation_nodes(State(state): State<AppState>) -> impl IntoResponse {
    let federation_stats = state.federation.get_federation_stats();

    let response = NodesResponse {
        node_count: federation_stats.federation_nodes,
        nodes: vec![], // Would include actual node details
    };
    Json(response).into_response()
}

/// Scale federation
async fn scale_federation(
    State(state): State<AppState>,
    Json(requirements): Json<ScaleRequirements>,
) -> impl IntoResponse {
    let result = match state.routing.scale_capacity(requirements).await {
        Ok(r) => r,
        Err(e) => return ApiError(e).into_response(),
    };

    let response = ScaleResponse {
        scaling_triggered: result.scaling_triggered,
        target_instances: result.target_instances,
        current_instances: result.current_instances,
        status: result.scaling_status,
    };

    Json(response).into_response()
}

/// Get federation statistics
async fn get_federation_stats(State(state): State<AppState>) -> impl IntoResponse {
    let federation_stats = state.federation.get_federation_stats();
    Json(federation_stats)
}

// Ecosystem coordination handlers

/// Discover primals
#[expect(
    clippy::cast_possible_truncation,
    reason = "API response handling; value ranges audited"
)]
async fn discover_primals(State(state): State<AppState>) -> impl IntoResponse {
    let primals = match state.ecosystem.discover_primals().await {
        Ok(p) => p,
        Err(e) => return ApiError(e).into_response(),
    };
    let response = DiscoveryResponse {
        discovered_count: primals.len() as u32,
        primals,
    };

    Json(response).into_response()
}

/// Coordinate a task
async fn coordinate_task(
    State(state): State<AppState>,
    Json(task): Json<Task>,
) -> impl IntoResponse {
    let result = match state.ecosystem.coordinate_task(task).await {
        Ok(r) => r,
        Err(e) => return ApiError(e).into_response(),
    };
    Json(result).into_response()
}

/// List discovered primals
#[expect(
    clippy::cast_possible_truncation,
    reason = "API response handling; value ranges audited"
)]
async fn list_discovered_primals(State(state): State<AppState>) -> impl IntoResponse {
    let primals = state.ecosystem.get_discovered_primals();

    let response = PrimalsResponse {
        total_primals: primals.len() as u32,
        primals,
    };

    Json(response).into_response()
}

// Task management handlers

/// Submit a task
async fn submit_task(
    State(state): State<AppState>,
    Json(request): Json<TaskSubmissionRequest>,
) -> impl IntoResponse {
    let task = Task {
        id: request
            .task_id
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        task_type: request.task_type,
        priority: request.priority,
        requirements: request.requirements,
        context: request.context,
        deadline: request.deadline,
    };

    let result = match state.ecosystem.coordinate_task(task).await {
        Ok(r) => r,
        Err(e) => return ApiError(e).into_response(),
    };

    let response = TaskSubmissionResponse {
        task_id: result.id,
        status: format!("{:?}", result.status),
        message: "Task submitted successfully".to_string(),
    };

    Json(response).into_response()
}

/// Task status endpoint — returns 404 until Phase 2 persistence is wired.
///
/// When task persistence is available (via capability discovery), this will
/// query the task store. Until then, callers receive a clear "not available"
/// response rather than a misleading "completed" stub.
async fn get_task_status(
    State(_state): State<AppState>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let response = TaskStatusResponse {
        task_id,
        status: "unknown".to_string(),
        result: Some(serde_json::json!({
            "error": "task_tracking_unavailable",
            "message": "Task status tracking requires persistence backend (Phase 2)"
        })),
        error: None,
    };

    (StatusCode::NOT_FOUND, Json(response)).into_response()
}

// Administrative handlers

/// Shutdown service
async fn shutdown_service(State(_state): State<AppState>) -> impl IntoResponse {
    // This would trigger graceful shutdown
    let response = ShutdownResponse {
        message: "Shutdown initiated".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    Json(response).into_response()
}

#[cfg(test)]
#[path = "api_tests.rs"]
mod tests;
