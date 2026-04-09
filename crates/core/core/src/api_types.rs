// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(missing_docs)]

use axum::{
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};

use crate::{ContextRequirements, Error};

/// Maps [`Error`] to HTTP responses for Axum handlers.
pub struct ApiError(pub(crate) Error);

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
pub struct HealthResponse {
    pub(crate) status: String,
    pub(crate) timestamp: String,
    pub(crate) ecosystem_status: String,
    pub(crate) version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    pub(crate) status: String,
    pub(crate) timestamp: String,
    pub(crate) ecosystem_health: String,
    pub(crate) version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfoResponse {
    pub(crate) node_id: String,
    pub(crate) version: String,
    /// Elapsed time since the HTTP API server was constructed (e.g. `42s`).
    pub(crate) uptime: String,
    pub(crate) federation_status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentsResponse {
    pub(crate) total_agents: u32,
    pub(crate) active_tasks: u64,
    pub(crate) completed_tasks: u64,
    pub(crate) failed_tasks: u64,
    pub(crate) average_response_time: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentInfo {
    pub(crate) id: String,
    pub(crate) endpoint: String,
    pub(crate) capabilities: Vec<String>,
    pub(crate) status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub(crate) success: bool,
    pub(crate) message: String,
    pub(crate) agent_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FederationResponse {
    pub(crate) success: bool,
    pub(crate) federation_id: String,
    pub(crate) node_count: u32,
    pub(crate) status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodesResponse {
    pub(crate) node_count: u32,
    pub(crate) nodes: Vec<NodeInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeInfo {
    pub(crate) id: String,
    pub(crate) endpoint: String,
    pub(crate) region: Option<String>,
    pub(crate) capabilities: Vec<String>,
    pub(crate) status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScaleResponse {
    pub(crate) scaling_triggered: bool,
    pub(crate) target_instances: u32,
    pub(crate) current_instances: u32,
    pub(crate) status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscoveryResponse {
    pub(crate) discovered_count: u32,
    pub(crate) primals: Vec<crate::PrimalEndpoint>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrimalsResponse {
    pub(crate) total_primals: u32,
    pub(crate) primals: Vec<crate::PrimalEndpoint>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskSubmissionResponse {
    pub(crate) task_id: String,
    pub(crate) status: String,
    pub(crate) message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskStatusResponse {
    pub(crate) task_id: String,
    pub(crate) status: String,
    pub(crate) result: Option<serde_json::Value>,
    pub(crate) error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShutdownResponse {
    pub(crate) message: String,
    pub(crate) timestamp: String,
}

// Request types

#[derive(Debug, Serialize, Deserialize)]
pub struct McpTaskRequest {
    pub(crate) task_id: Option<String>,
    pub(crate) agent_id: String,
    pub(crate) payload: serde_json::Value,
    pub(crate) context: Option<serde_json::Value>,
    pub(crate) routing_hints: Option<Vec<String>>,
    pub(crate) context_requirements: Option<ContextRequirements>,
    pub(crate) mcp_request: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskSubmissionRequest {
    pub(crate) task_id: Option<String>,
    pub(crate) task_type: crate::TaskType,
    pub(crate) priority: crate::TaskPriority,
    pub(crate) requirements: crate::TaskRequirements,
    pub(crate) context: serde_json::Value,
    pub(crate) deadline: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinFederationRequest {
    pub(crate) federation_id: Option<String>,
    pub(crate) node_endpoint: String,
    pub(crate) capabilities: Vec<String>,
    pub(crate) region: Option<String>,
    pub(crate) metadata: std::collections::HashMap<String, String>,
}
