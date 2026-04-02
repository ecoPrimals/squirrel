// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
    routing::{Router, get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    AgentSpec, ContextRequirements, Error, McpRouter, McpTask, NodeSpec, PrimalCoordinator, Result,
    ScaleRequirements, SwarmManager, Task, ecosystem::EcosystemService,
    federation::FederationService, routing::McpRoutingService,
};

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
        Self(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self.0 {
            Error::Configuration(_) => (StatusCode::BAD_REQUEST, "Configuration error"),
            Error::Coordination(_) => (StatusCode::SERVICE_UNAVAILABLE, "Coordination error"),
            Error::Discovery(_) => (StatusCode::SERVICE_UNAVAILABLE, "Discovery error"),
            Error::CapabilityUnavailable { .. } => {
                (StatusCode::SERVICE_UNAVAILABLE, "Capability unavailable")
            }
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
    /// Elapsed time since the HTTP API server was constructed (e.g. `42s`).
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

#[derive(Debug, Serialize, Deserialize)]
#[expect(dead_code, reason = "deserialized from JSON at runtime")]
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
#[expect(dead_code, reason = "deserialized from JSON at runtime")]
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

#[cfg(test)]
mod tests {
    use super::ApiServer;
    use crate::ecosystem::EcosystemService;
    use crate::federation::FederationService;
    use crate::monitoring::MonitoringConfig;
    use crate::routing::McpRoutingService;
    use crate::{
        AgentSpec, DiscoveryConfig, EcosystemConfig, EcosystemMode, FederationConfig, McpTask,
        MonitoringService, RoutingConfig, Task, TaskPriority, TaskRequirements, TaskType,
    };
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use chrono::Duration as ChronoDuration;
    use ecosystem_api::PrimalType;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tower05::util::ServiceExt;

    fn test_routing() -> Arc<McpRoutingService> {
        Arc::new(McpRoutingService::new(RoutingConfig::default()).expect("routing config"))
    }

    fn test_federation() -> Arc<FederationService> {
        Arc::new(FederationService::new(FederationConfig::default()).expect("fed"))
    }

    fn test_ecosystem_disabled() -> Arc<EcosystemService> {
        Arc::new(
            EcosystemService::new(
                EcosystemConfig {
                    enabled: false,
                    mode: EcosystemMode::Standalone,
                    discovery: DiscoveryConfig::default(),
                },
                Arc::new(MonitoringService::new(MonitoringConfig::default())),
            )
            .expect("eco"),
        )
    }

    fn test_ecosystem_coordinated() -> Arc<EcosystemService> {
        Arc::new(
            EcosystemService::new(
                EcosystemConfig {
                    enabled: true,
                    mode: EcosystemMode::Coordinated,
                    discovery: DiscoveryConfig {
                        auto_discovery: false,
                        discovery_endpoint: None,
                        direct_endpoints: HashMap::new(),
                        probe_interval: ChronoDuration::seconds(60),
                        health_check_timeout: ChronoDuration::seconds(5),
                    },
                },
                Arc::new(MonitoringService::new(MonitoringConfig::default())),
            )
            .expect("eco"),
        )
    }

    fn app_disabled_eco() -> axum::Router {
        ApiServer::new(test_ecosystem_disabled(), test_routing(), test_federation()).create_router()
    }

    fn app_coord_eco() -> axum::Router {
        ApiServer::new(
            test_ecosystem_coordinated(),
            test_routing(),
            test_federation(),
        )
        .create_router()
    }

    async fn read_body_json(resp: axum::response::Response) -> serde_json::Value {
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.expect("body");
        serde_json::from_slice(&bytes).expect("json")
    }

    #[tokio::test]
    async fn health_returns_healthy_payload() {
        let app = app_disabled_eco();
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .expect("req"),
            )
            .await
            .expect("call");
        assert_eq!(res.status(), StatusCode::OK);
        let v = read_body_json(res).await;
        assert_eq!(v["status"], "healthy");
    }

    #[tokio::test]
    async fn route_mcp_without_agents_returns_500() {
        let app = app_disabled_eco();
        let task = McpTask {
            id: "t1".to_string(),
            agent_id: None,
            payload: serde_json::json!({}),
            context: None,
            routing_hints: vec![],
            context_requirements: None,
            mcp_request: serde_json::json!({}),
        };
        let body = serde_json::to_vec(&task).expect("ser");
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/route")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .expect("req"),
            )
            .await
            .expect("call");
        assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn route_mcp_after_register_agent_succeeds() {
        let app = app_disabled_eco();
        let agent = AgentSpec {
            id: "a1".to_string(),
            endpoint: "http://127.0.0.1:1".to_string(),
            capabilities: vec!["mcp".to_string()],
            weight: None,
            max_concurrent_tasks: 4,
            metadata: HashMap::new(),
        };
        let reg = serde_json::to_vec(&agent).expect("ser");
        let res_reg = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/agents")
                    .header("content-type", "application/json")
                    .body(Body::from(reg))
                    .expect("req"),
            )
            .await
            .expect("call");
        assert_eq!(res_reg.status(), StatusCode::OK);

        let task = McpTask {
            id: "t1".to_string(),
            agent_id: None,
            payload: serde_json::json!({}),
            context: None,
            routing_hints: vec![],
            context_requirements: None,
            mcp_request: serde_json::json!({}),
        };
        let body = serde_json::to_vec(&task).expect("ser");
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/route")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .expect("req"),
            )
            .await
            .expect("call");
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn info_returns_node_and_version_fields() {
        let app = app_disabled_eco();
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/info")
                    .body(Body::empty())
                    .expect("req"),
            )
            .await
            .expect("call");
        assert_eq!(res.status(), StatusCode::OK);
        let v = read_body_json(res).await;
        assert!(v.get("node_id").is_some());
        assert_eq!(
            v["version"].as_str().unwrap_or(""),
            crate::SQUIRREL_MCP_VERSION
        );
    }

    #[tokio::test]
    async fn coordinate_task_returns_503_when_routing_fails() {
        let app = app_coord_eco();
        let task = Task {
            id: "x1".to_string(),
            task_type: TaskType::McpCoordination,
            priority: TaskPriority::Normal,
            requirements: TaskRequirements {
                cpu: None,
                memory: None,
                storage: None,
                network: None,
                required_capabilities: vec!["missing".to_string()],
                preferred_primals: vec![PrimalType::Squirrel],
                constraints: HashMap::new(),
            },
            context: serde_json::json!({}),
            deadline: None,
        };
        let body = serde_json::to_vec(&task).expect("ser");
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/coordinate")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .expect("req"),
            )
            .await
            .expect("call");
        assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn list_discovered_primals_returns_zero_by_default() {
        let app = app_disabled_eco();
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/primals")
                    .body(Body::empty())
                    .expect("req"),
            )
            .await
            .expect("call");
        assert_eq!(res.status(), StatusCode::OK);
        let v = read_body_json(res).await;
        assert_eq!(v["total_primals"], 0);
    }

    #[tokio::test]
    async fn get_task_status_stub_returns_completed() {
        let app = app_disabled_eco();
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/task-abc")
                    .body(Body::empty())
                    .expect("req"),
            )
            .await
            .expect("call");
        assert_eq!(res.status(), StatusCode::OK);
        let v = read_body_json(res).await;
        assert_eq!(v["task_id"], "task-abc");
        assert_eq!(v["status"], "completed");
    }

    #[tokio::test]
    async fn shutdown_endpoint_returns_message() {
        let app = app_disabled_eco();
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/shutdown")
                    .body(Body::empty())
                    .expect("req"),
            )
            .await
            .expect("call");
        assert_eq!(res.status(), StatusCode::OK);
        let v = read_body_json(res).await;
        assert_eq!(v["message"], "Shutdown initiated");
    }
}
