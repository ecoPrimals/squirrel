// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Compatibility module for legacy squirrel_core references
//!
//! This module re-exports types from squirrel_context for backward compatibility.

#![cfg_attr(not(test), forbid(unsafe_code))]
#![warn(missing_docs)]
// Core modules for Squirrel MCP ecosystem coordination
#[cfg(feature = "http-api")]
pub mod api;
/// Configuration loading and validation.
pub mod config;
#[cfg(feature = "mesh")]
pub mod coordination;
/// Service and primal discovery.
pub mod discovery;
#[cfg(feature = "mesh")]
pub mod ecosystem;
/// Error types for core operations.
pub mod error;
#[cfg(feature = "mesh")]
pub mod federation;
/// Manifest parsing and validation.
pub mod manifest;
pub mod monitoring;
#[cfg(feature = "mesh")]
pub mod routing;
#[cfg(feature = "mesh")]
pub mod service_discovery;
#[cfg(feature = "mesh")]
pub mod swarm;

// Re-export core types
#[cfg(feature = "http-api")]
pub use api::*;
pub use config::*;
#[cfg(feature = "mesh")]
pub use coordination::*;
pub use discovery::*;
#[cfg(feature = "mesh")]
pub use ecosystem::*;
pub use error::*;
#[cfg(feature = "mesh")]
pub use federation::*;
pub use manifest::*;
pub use monitoring::*;
#[cfg(feature = "mesh")]
pub use routing::*;
#[cfg(feature = "mesh")]
pub use service_discovery::*;
#[cfg(feature = "mesh")]
pub use swarm::*;

/// Result type alias for core operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in core ecosystem operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Configuration loading or validation failed.
    #[error("Configuration error: {0}")]
    Configuration(#[from] config::Error),

    /// Generic configuration error with message.
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Requested agent was not found.
    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    /// No agent available to handle the request.
    #[error("No agent available")]
    NoAgentAvailable,

    /// Context was not found.
    #[error("Context not found: {0}")]
    ContextNotFound(String),

    /// Context data was invalid.
    #[error("Invalid context: {0}")]
    InvalidContext(String),

    /// Coordination between primals failed.
    #[error("Coordination error: {0}")]
    Coordination(String),

    /// Service discovery failed.
    #[error("Discovery error: {0}")]
    Discovery(String),

    /// Federation operation failed.
    #[error("Federation error: {0}")]
    Federation(String),

    /// Task routing failed.
    #[error("Routing error: {0}")]
    Routing(String),

    /// Swarm coordination failed.
    #[error("Swarm error: {0}")]
    Swarm(String),

    /// Monitoring operation failed.
    #[error("Monitoring error: {0}")]
    Monitoring(String),

    /// HTTP request/response failed.
    #[error("HTTP error: {0}")]
    Http(String),

    /// JSON serialization/deserialization failed.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// I/O operation failed.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// Feature-gated reqwest::Error conversion (only available with http-client feature)
#[cfg(feature = "http-client")]
impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Http(err.to_string())
    }
}

/// Trait for coordinating tasks across primal ecosystem.
#[async_trait::async_trait]
pub trait PrimalCoordinator {
    /// Registers this node with the ecosystem.
    async fn register_with_ecosystem(&self) -> Result<()>;
    /// Discovers available primals in the ecosystem.
    async fn discover_primals(&self) -> Result<Vec<PrimalEndpoint>>;
    /// Coordinates execution of a task.
    async fn coordinate_task(&self, task: Task) -> Result<TaskResult>;
    /// Returns current health status.
    async fn health_check(&self) -> Result<HealthStatus>;
}

#[cfg(feature = "mesh")]
#[async_trait::async_trait]
pub trait McpRouter {
    async fn route_task(&self, task: McpTask) -> Result<TaskResponse>;
    async fn coordinate_agents(&self, agents: Vec<AgentSpec>) -> Result<CoordinationResult>;
    async fn scale_capacity(&self, requirements: ScaleRequirements) -> Result<ScaleResult>;
}

#[cfg(feature = "mesh")]
#[async_trait::async_trait]
pub trait SwarmManager {
    async fn spawn_squirrel(&self, config: SquirrelConfig) -> Result<SquirrelInstance>;
    async fn federate_nodes(&self, nodes: Vec<NodeSpec>) -> Result<FederationResult>;
    async fn balance_load(&self, load: LoadMetrics) -> Result<LoadBalanceResult>;
}

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

// Re-export canonical PrimalType from ecosystem-api for ecosystem-wide consistency
// This ensures all primals use the same type definitions for service discovery and routing
pub use ecosystem_api::PrimalType;

/// Health status of a primal or component.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

/// Result of agent coordination.
#[cfg(feature = "mesh")]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CoordinationResult {
    /// Successfully registered agents.
    pub registered_agents: u32,
    /// Failed registration count.
    pub failed_registrations: u32,
    /// Total agents.
    pub total_agents: u32,
    /// Status message.
    pub status: String,
}

/// Result of scaling operation.
#[cfg(feature = "mesh")]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScaleResult {
    /// Whether scaling was triggered.
    pub scaling_triggered: bool,
    /// Target instance count.
    pub target_instances: u32,
    /// Current instance count.
    pub current_instances: u32,
    /// Scaling status.
    pub scaling_status: String,
    /// Status message.
    pub message: String,
    /// New capacity after scaling.
    pub new_capacity: u32,
}

/// Load metrics for a node.
#[cfg(feature = "mesh")]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LoadMetrics {
    /// CPU usage (0-1).
    pub cpu_usage: f64,
    /// Memory usage (0-1).
    pub memory_usage: f64,
    /// Network usage.
    pub network_usage: f64,
    /// Active task count.
    pub active_tasks: u32,
    /// Queue length.
    pub queue_length: u32,
    /// Average response time.
    pub response_time: std::time::Duration,
    /// Error rate (0-1).
    pub error_rate: f64,
}

/// Result of load balancing.
#[cfg(feature = "mesh")]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LoadBalanceResult {
    /// Task distribution per node.
    pub distribution: std::collections::HashMap<String, u32>,
    /// Balance quality score.
    pub balance_score: f64,
    /// Time to rebalance.
    pub rebalance_time: std::time::Duration,
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

/// Squirrel MCP protocol version.
pub const SQUIRREL_MCP_VERSION: &str = "2.2.0";
/// Primal type identifier for Squirrel.
pub const PRIMAL_TYPE: &str = "squirrel";

/// Service mesh load balancer configuration.
#[cfg(feature = "mesh")]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ServiceMeshLoadBalancerConfig {
    /// Load balancer endpoint URL.
    pub endpoint: String,
    /// Whether integration is enabled.
    pub enabled: bool,
    /// Strategy when mesh is unavailable.
    pub fallback_strategy: LoadBalancingStrategy,
    /// Timeout for coordination.
    pub coordination_timeout: std::time::Duration,
}

// Deprecated alias for backward compatibility
#[cfg(feature = "mesh")]
#[deprecated(since = "0.1.0", note = "Use ServiceMeshLoadBalancerConfig instead")]
pub type SongbirdLoadBalancerConfig = ServiceMeshLoadBalancerConfig;

/// Load balancing strategy.
#[cfg(feature = "mesh")]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin,
    ResponseTimeBased,
    CapabilityBased,
    Adaptive,
    Random,
    HealthBased,
}

/// MCP load balancer configuration.
#[cfg(feature = "mesh")]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct McpLoadBalancerConfig {
    /// Local routing strategy.
    pub local_strategy: LoadBalancingStrategy,
    /// Optional service mesh integration.
    pub service_mesh_integration: Option<ServiceMeshLoadBalancerConfig>,
    /// Whether federation is enabled.
    pub federation_enabled: bool,
    /// Whether cross-primal routing is enabled.
    pub cross_primal_routing: bool,
}

/// Load balancer statistics.
#[cfg(feature = "mesh")]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LoadBalancerStats {
    /// Total requests.
    pub total_requests: u64,
    /// Successful requests.
    pub successful_requests: u64,
    /// Failed requests.
    pub failed_requests: u64,
    /// Average response time in seconds.
    pub average_response_time: f64,
    /// Active connections.
    pub active_connections: u64,
    /// Per-service stats.
    pub service_stats: std::collections::HashMap<String, ServiceStats>,
    /// MCP routing stats.
    pub mcp_routing_stats: McpRoutingStats,
    /// Federation stats if enabled.
    pub federation_stats: Option<FederationStats>,
}

/// Per-service statistics.
#[cfg(feature = "mesh")]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ServiceStats {
    /// Request count.
    pub requests: u64,
    /// Success count.
    pub successes: u64,
    /// Failure count.
    pub failures: u64,
    /// Average response time.
    pub average_response_time: f64,
    /// Active connections.
    pub active_connections: u64,
}

/// MCP routing statistics.
#[cfg(feature = "mesh")]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct McpRoutingStats {
    /// Registered agents.
    pub agents_registered: u32,
    /// Tasks routed locally.
    pub tasks_routed_locally: u64,
    /// Tasks routed to primals.
    pub tasks_routed_to_primals: u64,
    /// Tasks routed to federation.
    pub tasks_routed_to_federation: u64,
    /// Context operations.
    pub context_operations: u64,
}

/// Federation statistics.
#[cfg(feature = "mesh")]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FederationStats {
    /// Active nodes.
    pub nodes_active: u32,
    /// Total capacity.
    pub total_capacity: u32,
    /// Load per node.
    pub load_distribution: std::collections::HashMap<String, f64>,
}

/// Routing statistics for a node.
#[cfg(feature = "mesh")]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RoutingStats {
    /// Node identifier.
    pub node_id: String,
    /// Active tasks.
    pub active_tasks: u64,
    /// Completed tasks.
    pub completed_tasks: u64,
    /// Failed tasks.
    pub failed_tasks: u64,
    /// Queued tasks.
    pub queued_tasks: u64,
    /// Registered agents.
    pub registered_agents: u32,
    /// Average response time.
    pub average_response_time: f64,
    /// Federation nodes.
    pub federation_nodes: u32,
}

// Service Mesh Load Balancer Integration Trait (Capability-Based)
#[cfg(feature = "mesh")]
#[async_trait::async_trait]
pub trait ServiceMeshLoadBalancerIntegration {
    /// Register Squirrel MCP with service mesh load balancer
    async fn register_with_service_mesh(
        &self,
        config: &ServiceMeshLoadBalancerConfig,
    ) -> Result<()>;

    /// Find a capable primal for cross-primal task routing
    async fn find_capable_primal(&self, task: &McpTask) -> Result<Option<PrimalEndpoint>>;

    /// Report load metrics to service mesh for ecosystem-wide load balancing
    async fn report_load_metrics(&self, metrics: &LoadBalancerStats) -> Result<()>;

    /// Query service mesh for ecosystem load distribution recommendations
    async fn query_load_distribution(&self) -> Result<EcosystemLoadDistribution>;

    /// Coordinate with service mesh during scaling events
    async fn coordinate_scaling(&self, scale_event: &ScaleEvent) -> Result<ScaleRecommendation>;
}

// Deprecated alias for backward compatibility
#[cfg(feature = "mesh")]
#[deprecated(
    since = "0.1.0",
    note = "Use ServiceMeshLoadBalancerIntegration instead"
)]
pub trait SongbirdLoadBalancerIntegration: ServiceMeshLoadBalancerIntegration {}

/// Ecosystem-wide load distribution from service mesh.
#[cfg(feature = "mesh")]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EcosystemLoadDistribution {
    /// Recommended load per node.
    pub recommended_distribution: std::collections::HashMap<String, f64>,
    /// Overall ecosystem health.
    pub overall_ecosystem_health: HealthStatus,
    /// Scaling recommendations.
    pub scaling_recommendations: Vec<ScaleRecommendation>,
    /// Suggested cross-primal routes.
    pub cross_primal_routing_suggestions: Vec<CrossPrimalRoute>,
}

/// Scaling event from load balancer.
#[cfg(feature = "mesh")]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ScaleEvent {
    /// Type of scaling event.
    pub event_type: ScaleEventType,
    /// Current load metrics.
    pub current_load: LoadBalancerStats,
    /// Projected load if available.
    pub projected_load: Option<LoadBalancerStats>,
    /// Resource requirements.
    pub resource_requirements: ScaleRequirements,
}

/// Type of scaling event.
#[cfg(feature = "mesh")]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ScaleEventType {
    ScaleUp,
    ScaleDown,
    Rebalance,
    Emergency,
}

/// Recommendation from scaling analysis.
#[cfg(feature = "mesh")]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ScaleRecommendation {
    /// Action to take.
    pub action: ScaleAction,
    /// Priority of the recommendation.
    pub priority: TaskPriority,
    /// Estimated impact.
    pub estimated_impact: f64,
    /// Whether coordination is required.
    pub coordination_required: bool,
}

/// Scaling action to perform.
#[cfg(feature = "mesh")]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ScaleAction {
    /// Spawn N instances.
    SpawnInstances(u32),
    /// Terminate N instances.
    TerminateInstances(u32),
    /// Rebalance load.
    RebalanceLoad,
    /// Delegate to federation.
    DelegateToFederation,
    /// Request assistance from primal type.
    RequestPrimalAssistance(PrimalType),
}

/// Route for cross-primal task delegation.
#[cfg(feature = "mesh")]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CrossPrimalRoute {
    /// Source primal type.
    pub source_primal: PrimalType,
    /// Target primal type.
    pub target_primal: PrimalType,
    /// Task types for this route.
    pub task_types: Vec<TaskType>,
    /// Estimated benefit.
    pub estimated_benefit: f64,
}

// Enhanced MCP Router with Service Mesh Integration
#[cfg(feature = "mesh")]
#[async_trait::async_trait]
pub trait EnhancedMcpRouter: McpRouter + ServiceMeshLoadBalancerIntegration {
    /// Route task with service mesh coordination
    async fn route_task_with_service_mesh(&self, task: McpTask) -> Result<TaskResponse>;

    /// Get comprehensive routing statistics including service mesh coordination
    async fn get_comprehensive_stats(&self) -> Result<LoadBalancerStats>;

    /// Handle cross-primal task coordination
    async fn coordinate_cross_primal_task(
        &self,
        task: McpTask,
        target_primal: PrimalEndpoint,
    ) -> Result<TaskResponse>;
}

/// Requirements for scaling operation.
#[cfg(feature = "mesh")]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ScaleRequirements {
    /// Target capacity.
    pub target_capacity: u32,
    /// Minimum instances.
    pub min_instances: u32,
    /// Maximum instances.
    pub max_instances: u32,
    /// Scaling triggers.
    pub triggers: Vec<ScaleTrigger>,
}

/// Trigger that initiates scaling.
#[cfg(feature = "mesh")]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ScaleTrigger {
    /// CPU usage threshold.
    CpuThreshold(f64),
    /// Memory usage threshold.
    MemoryThreshold(f64),
    /// Queue length threshold.
    QueueLength(u32),
    /// Response time threshold.
    ResponseTime(std::time::Duration),
    /// Custom trigger.
    Custom(String),
}

/// Specification for a federation node.
#[cfg(feature = "mesh")]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NodeSpec {
    /// Node identifier.
    pub id: String,
    /// Region if applicable.
    pub region: Option<String>,
    /// Zone if applicable.
    pub zone: Option<String>,
    /// Node endpoint URL.
    pub endpoint: String,
    /// Node capabilities.
    pub capabilities: Vec<String>,
    /// Capacity (max concurrent tasks).
    pub capacity: u32,
}

#[cfg(feature = "mesh")]
impl Default for FederationConfig {
    fn default() -> Self {
        Self {
            node_id: "default-node".to_string(),
            port: 8080,
            federation_discovery_urls: Vec::new(),
            auto_scaling_enabled: true,
            min_instances: 1,
            max_instances: 10,
            scale_up_threshold: 0.7,
            scale_down_threshold: 0.3,
            health_check_interval: chrono::Duration::minutes(1),
            federation_timeout: chrono::Duration::seconds(30),
            federation_enabled: false,
        }
    }
}

/// A Squirrel instance in the federation.
#[cfg(feature = "mesh")]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SquirrelInstance {
    /// Instance identifier.
    pub id: String,
    /// Node this instance runs on.
    pub node_id: String,
    /// Instance endpoint.
    pub endpoint: String,
    /// Region if applicable.
    pub region: Option<String>,
    /// Zone if applicable.
    pub zone: Option<String>,
    /// Instance capabilities.
    pub capabilities: Vec<String>,
    /// Max capacity.
    pub capacity: u32,
    /// Current load.
    pub current_load: u32,
    /// Instance health.
    pub health: InstanceStatus,
    /// Last seen timestamp.
    pub last_seen: chrono::DateTime<chrono::Utc>,
    /// Metadata.
    pub metadata: std::collections::HashMap<String, String>,
}

/// Configuration for spawning a Squirrel instance.
#[cfg(feature = "mesh")]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SquirrelConfig {
    /// Node identifier.
    pub node_id: String,
    /// Port to bind.
    pub port: u16,
    /// Region if applicable.
    pub region: Option<String>,
    /// Zone if applicable.
    pub zone: Option<String>,
    /// Instance capabilities.
    pub capabilities: Vec<String>,
    /// Capacity.
    pub capacity: u32,
    /// Whether federation is enabled.
    pub federation_enabled: bool,
    /// Whether auto-scaling is enabled.
    pub auto_scaling_enabled: bool,
    /// Metadata.
    pub metadata: std::collections::HashMap<String, String>,
}

/// Result of federation operation.
#[cfg(feature = "mesh")]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FederationResult {
    /// Federation identifier.
    pub federation_id: String,
    /// Nodes that joined.
    pub nodes_joined: u32,
    /// Total capacity.
    pub total_capacity: u32,
    /// Federation status.
    pub status: FederationStatus,
}

/// Topology of the federation network.
#[cfg(feature = "mesh")]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FederationTopology {
    Star,
    Ring,
    Mesh,
    Tree,
}

/// Status of the federation.
#[cfg(feature = "mesh")]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FederationStatus {
    /// Federation is forming.
    Forming,
    /// Federation is active.
    Active,
    /// Federation is degraded.
    Degraded,
    /// Federation is inactive.
    Inactive,
    /// Federation has error.
    Error,
}

/// Status of an instance.
#[cfg(feature = "mesh")]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum InstanceStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
    Unknown,
}

/// Configuration for federation.
#[cfg(feature = "mesh")]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FederationConfig {
    /// Node identifier.
    pub node_id: String,
    /// Port.
    pub port: u16,
    /// URLs for federation discovery.
    pub federation_discovery_urls: Vec<String>,
    /// Whether auto-scaling is enabled.
    pub auto_scaling_enabled: bool,
    /// Minimum instances.
    pub min_instances: u32,
    /// Maximum instances.
    pub max_instances: u32,
    /// Scale-up threshold.
    pub scale_up_threshold: f64,
    /// Scale-down threshold.
    pub scale_down_threshold: f64,
    /// Health check interval.
    pub health_check_interval: chrono::Duration,
    /// Federation timeout.
    pub federation_timeout: chrono::Duration,
    /// Whether federation is enabled.
    pub federation_enabled: bool,
}

// Missing types referenced in federation.rs
#[cfg(feature = "mesh")]
#[derive(Debug)]
pub struct FederationLoadBalancer {
    #[expect(dead_code, reason = "Load metrics for federation load balancing")]
    load_metrics: std::sync::Arc<LoadMetrics>,
    #[expect(dead_code, reason = "Balancing strategy for federation routing")]
    balancing_strategy: LoadBalancingStrategy,
}

#[cfg(feature = "mesh")]
impl FederationLoadBalancer {
    pub fn new(load_metrics: std::sync::Arc<LoadMetrics>) -> Self {
        Self {
            load_metrics,
            balancing_strategy: LoadBalancingStrategy::RoundRobin,
        }
    }
}

// RoutingStats is already defined earlier in the file
