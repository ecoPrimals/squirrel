// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Core types for Squirrel ecosystem coordination.

use ecosystem_api::PrimalType;

/// Endpoint for a primal in the ecosystem.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PrimalEndpoint {
    /// Unique primal identifier.
    pub id: String,
    /// Type of primal (MCP, AI, etc.).
    pub primal_type: PrimalType,
    /// Network endpoint URL.
    pub endpoint: String,
    /// Capabilities this primal provides.
    pub capabilities: Vec<String>,
    /// Current health status.
    pub health: HealthStatus,
    /// Additional metadata.
    pub metadata: std::collections::HashMap<String, String>,
}

/// Health status of a primal or component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HealthStatus {
    /// Fully operational.
    Healthy,
    /// Partially degraded but functional.
    Degraded,
    /// Not operational.
    Unhealthy,
    /// Status unknown.
    Unknown,
}

/// Task to be coordinated across the ecosystem.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Task {
    /// Unique task identifier.
    pub id: String,
    /// Type of task.
    pub task_type: TaskType,
    /// Execution priority.
    pub priority: TaskPriority,
    /// Resource and capability requirements.
    pub requirements: TaskRequirements,
    /// Task context/payload.
    pub context: serde_json::Value,
    /// Optional deadline.
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
}

/// Type of coordinated task.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TaskType {
    /// MCP protocol coordination.
    McpCoordination,
    /// AI task routing.
    AiTaskRouting,
    /// Context management operation.
    ContextManagement,
    /// Storage operation.
    StorageOperation,
    /// Security validation.
    SecurityValidation,
    /// Compute execution.
    ComputeExecution,
    /// Service discovery.
    ServiceDiscovery,
    /// Federation management.
    FederationManagement,
}

/// Priority level for task execution.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum TaskPriority {
    /// Must run immediately.
    Critical,
    /// High priority.
    High,
    /// Normal priority.
    Normal,
    /// Low priority.
    Low,
    /// Background/opportunistic.
    Background,
}

/// Resource and capability requirements for a task.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TaskRequirements {
    /// Required CPU (fraction 0-1).
    pub cpu: Option<f64>,
    /// Required memory in bytes.
    pub memory: Option<u64>,
    /// Required storage in bytes.
    pub storage: Option<u64>,
    /// Required network bandwidth.
    pub network: Option<f64>,
    /// Required capabilities.
    pub required_capabilities: Vec<String>,
    /// Preferred primal types.
    pub preferred_primals: Vec<PrimalType>,
    /// Additional constraints.
    pub constraints: std::collections::HashMap<String, String>,
}

/// Result of task execution.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TaskResult {
    /// Task identifier.
    pub id: String,
    /// Final status.
    pub status: TaskStatus,
    /// Result payload if successful.
    pub result: Option<serde_json::Value>,
    /// Error message if failed.
    pub error: Option<String>,
    /// Time taken to execute.
    pub execution_time: std::time::Duration,
    /// ID of primal that executed it.
    pub executed_by: Option<String>,
}

/// Status of a task in the coordination pipeline.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TaskStatus {
    /// Waiting in queue.
    Queued,
    /// Currently executing.
    Running,
    /// Completed successfully.
    Completed,
    /// Execution failed.
    Failed,
    /// Cancelled.
    Cancelled,
    /// Retrying after failure.
    Retrying,
}

/// MCP-specific task for routing.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct McpTask {
    /// Task identifier.
    pub id: String,
    /// Target agent if specified.
    pub agent_id: Option<String>,
    /// Task payload.
    pub payload: serde_json::Value,
    /// Optional context.
    pub context: Option<serde_json::Value>,
    /// Hints for routing.
    pub routing_hints: Vec<String>,
    /// Context requirements.
    pub context_requirements: Option<ContextRequirements>,
    /// Raw MCP request.
    pub mcp_request: serde_json::Value,
}

/// Response from task execution.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TaskResponse {
    /// Response identifier.
    pub id: String,
    /// Result payload.
    pub result: serde_json::Value,
    /// Agent that executed the task.
    pub agent_id: String,
    /// Execution duration.
    pub execution_time: std::time::Duration,
    /// Updated context if any.
    pub context: Option<serde_json::Value>,
    /// Original task ID.
    pub task_id: String,
    /// Raw response.
    pub response: serde_json::Value,
    /// Response metadata.
    pub metadata: ResponseMetadata,
}

/// Context requirements for task execution.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ContextRequirements {
    /// Whether context must persist.
    pub persistent_context: bool,
    /// Shared context key-value pairs.
    pub shared_context: std::collections::HashMap<String, String>,
    /// Names of shared contexts.
    pub shared_contexts: Vec<String>,
    /// Required context keys.
    pub required_context: std::collections::HashMap<String, String>,
    /// Context keys to include.
    pub context_keys: Vec<String>,
}

/// Metadata attached to task response.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ResponseMetadata {
    /// Whether context was updated.
    pub context_updated: bool,
    /// Processing time.
    pub processing_time: std::time::Duration,
    /// Agent version if known.
    pub agent_version: Option<String>,
}

/// Specification for an agent in coordination.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AgentSpec {
    /// Agent identifier.
    pub id: String,
    /// Agent endpoint URL.
    pub endpoint: String,
    /// Agent capabilities.
    pub capabilities: Vec<String>,
    /// Optional weight for load balancing.
    pub weight: Option<f64>,
    /// Max concurrent tasks.
    pub max_concurrent_tasks: u32,
    /// Additional metadata.
    pub metadata: std::collections::HashMap<String, String>,
}

/// Task waiting in queue.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct QueuedTask {
    /// The task.
    pub task: McpTask,
    /// When queued.
    pub queued_at: chrono::DateTime<chrono::Utc>,
    /// Priority.
    pub priority: TaskPriority,
    /// Current retry count.
    pub retry_count: u32,
    /// Max retries allowed.
    pub max_retries: u32,
}
