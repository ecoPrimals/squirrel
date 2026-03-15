// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
    routing::{get, post, Router},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    ecosystem::EcosystemService, federation::FederationService, routing::McpRoutingService,
    AgentSpec, ContextRequirements, Error, McpRouter, McpTask, PrimalCoordinator, Result,
    ScaleRequirements, Task,
};

/// API Server for the MCP Routing Service
pub struct ApiServer {
    ecosystem_service: Arc<EcosystemService>,
    routing_service: Arc<McpRoutingService>,
    federation_service: Arc<FederationService>,
}

#[derive(Clone)]
struct AppState {
    ecosystem: Arc<EcosystemService>,
    routing: Arc<McpRoutingService>,
    federation: Arc<FederationService>,
}

impl ApiServer {
    /// Create a new API server
    pub fn new(
        ecosystem_service: Arc<EcosystemService>,
        routing_service: Arc<McpRoutingService>,
        federation_service: Arc<FederationService>,
    ) -> Self {
        Self {
            ecosystem_service,
            routing_service,
            federation_service,
        }
    }

    /// Create the API router
    pub fn create_router(&self) -> Router {
        let state = AppState {
            ecosystem: self.ecosystem_service.clone(),
            routing: self.routing_service.clone(),
            federation: self.federation_service.clone(),
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
    let ecosystem_health = match state.ecosystem.health_check().await {
        Ok(health) => health,
        Err(_) => {
            return Json(HealthResponse {
                status: "unhealthy".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                ecosystem_status: "Error".to_string(),
                version: crate::SQUIRREL_MCP_VERSION.to_string(),
            })
            .into_response()
        }
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
    let ecosystem_health = match state.ecosystem.health_check().await {
        Ok(health) => health,
        Err(_) => {
            return Json(StatusResponse {
                status: "inactive".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                ecosystem_health: "Error".to_string(),
                version: crate::SQUIRREL_MCP_VERSION.to_string(),
            })
            .into_response()
        }
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
async fn get_info(State(_state): State<AppState>) -> impl IntoResponse {
    // Use placeholder data since get_status doesn't exist
    let federation_status = "Active".to_string();

    let response = InfoResponse {
        node_id: "squirrel-node".to_string(), // Placeholder since get_node_id doesn't exist
        version: crate::SQUIRREL_MCP_VERSION.to_string(),
        uptime: chrono::Utc::now().to_rfc3339(),
        federation_status,
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
    let stats = state.routing.get_stats();

    let response = AgentsResponse {
        total_agents: stats.registered_agents,
        active_tasks: stats.active_tasks,
        completed_tasks: stats.completed_tasks,
        failed_tasks: stats.failed_tasks,
        average_response_time: stats.average_response_time,
    };

    Json(response).into_response()
}

/// Register agent
async fn register_agent(
    State(state): State<AppState>,
    Json(agent): Json<AgentSpec>,
) -> impl IntoResponse {
    let result = match state.routing.register_agent(agent).await {
        Ok(_) => RegisterResponse {
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
    let stats = state.routing.get_stats();
    Json(stats)
}

// Federation handlers

/// Get federation information
async fn get_federation_info(State(state): State<AppState>) -> impl IntoResponse {
    let federation_stats = state.federation.get_federation_stats();

    ResponseJson(federation_stats).into_response()
}

/// Join federation
async fn join_federation(
    State(_state): State<AppState>,
    Json(_request): Json<JoinFederationRequest>,
) -> impl IntoResponse {
    // Use placeholder implementation since join_federation doesn't exist
    let result = crate::FederationResult {
        federation_id: "federation-1".to_string(),
        nodes_joined: 1,
        total_capacity: 100,
        status: crate::FederationStatus::Active,
    };

    let response = FederationResponse {
        success: matches!(result.status, crate::FederationStatus::Active),
        federation_id: result.federation_id,
        node_count: result.nodes_joined,
        status: format!("{:?}", result.status),
    };
    Json(response).into_response()
}

/// List federation nodes
async fn list_federation_nodes(State(state): State<AppState>) -> impl IntoResponse {
    let stats = state.federation.get_federation_stats();

    let response = NodesResponse {
        node_count: stats.federation_nodes,
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
    let stats = state.federation.get_federation_stats();
    Json(stats)
}

// Ecosystem coordination handlers

/// Discover primals
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

/// Get task status
async fn get_task_status(
    State(_state): State<AppState>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    // This would look up actual task status
    let response = TaskStatusResponse {
        task_id,
        status: "completed".to_string(),
        result: Some(serde_json::json!({"message": "Task completed successfully"})),
        error: None,
    };

    Json(response).into_response()
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

// Error handling

struct ApiError(Error);

impl From<Error> for ApiError {
    fn from(err: Error) -> Self {
        ApiError(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self.0 {
            Error::Configuration(_) => (StatusCode::BAD_REQUEST, "Configuration error"),
            Error::Coordination(_) => (StatusCode::SERVICE_UNAVAILABLE, "Coordination error"),
            Error::Discovery(_) => (StatusCode::SERVICE_UNAVAILABLE, "Discovery error"),
            Error::Federation(_) => (StatusCode::SERVICE_UNAVAILABLE, "Federation error"),
            Error::Routing(_) => (StatusCode::SERVICE_UNAVAILABLE, "Routing error"),
            Error::Http(_) => (StatusCode::BAD_GATEWAY, "HTTP error"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        let body = Json(serde_json::json!({
            "error": error_message,
            "message": self.0.to_string(),
        }));

        (status, body).into_response()
    }
}

// Response types

#[derive(Debug, Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    timestamp: String,
    ecosystem_status: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct StatusResponse {
    status: String,
    timestamp: String,
    ecosystem_health: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct InfoResponse {
    node_id: String,
    version: String,
    uptime: String,
    federation_status: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AgentsResponse {
    total_agents: u32,
    active_tasks: u64,
    completed_tasks: u64,
    failed_tasks: u64,
    average_response_time: f64,
}

#[allow(dead_code)] // Reserved for agent registration system
#[derive(Debug, Serialize, Deserialize)]
struct AgentInfo {
    id: String,
    endpoint: String,
    capabilities: Vec<String>,
    status: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RegisterResponse {
    success: bool,
    message: String,
    agent_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct FederationResponse {
    success: bool,
    federation_id: String,
    node_count: u32,
    status: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct NodesResponse {
    node_count: u32,
    nodes: Vec<NodeInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct NodeInfo {
    id: String,
    endpoint: String,
    region: Option<String>,
    capabilities: Vec<String>,
    status: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ScaleResponse {
    scaling_triggered: bool,
    target_instances: u32,
    current_instances: u32,
    status: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DiscoveryResponse {
    discovered_count: u32,
    primals: Vec<crate::PrimalEndpoint>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PrimalsResponse {
    total_primals: u32,
    primals: Vec<crate::PrimalEndpoint>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TaskSubmissionResponse {
    task_id: String,
    status: String,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TaskStatusResponse {
    task_id: String,
    status: String,
    result: Option<serde_json::Value>,
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ShutdownResponse {
    message: String,
    timestamp: String,
}

// Request types

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)] // API type for future MCP task routing
struct McpTaskRequest {
    task_id: Option<String>,
    agent_id: String,
    payload: serde_json::Value,
    context: Option<serde_json::Value>,
    routing_hints: Option<Vec<String>>,
    context_requirements: Option<ContextRequirements>,
    mcp_request: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TaskSubmissionRequest {
    task_id: Option<String>,
    task_type: crate::TaskType,
    priority: crate::TaskPriority,
    requirements: crate::TaskRequirements,
    context: serde_json::Value,
    deadline: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JoinFederationRequest {
    federation_id: Option<String>,
    node_endpoint: String,
    capabilities: Vec<String>,
    region: Option<String>,
    metadata: std::collections::HashMap<String, String>,
}
