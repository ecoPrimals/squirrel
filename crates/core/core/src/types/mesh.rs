// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Mesh-related types for federation and load balancing.

use ecosystem_api::PrimalType;

use super::core::{HealthStatus, TaskPriority, TaskType};

/// Result of agent coordination.
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
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LoadBalanceResult {
    /// Task distribution per node.
    pub distribution: std::collections::HashMap<String, u32>,
    /// Balance quality score.
    pub balance_score: f64,
    /// Time to rebalance.
    pub rebalance_time: std::time::Duration,
}

/// Service mesh load balancer configuration.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ServiceMeshLoadBalancerConfig {
    /// Load balancer endpoint URL.
    pub endpoint: String,
    /// Whether integration is enabled.
    pub enabled: bool,
    /// Strategy when mesh is unavailable.
    pub fallback_strategy: MeshLoadBalancingStrategy,
    /// Timeout for coordination.
    pub coordination_timeout: std::time::Duration,
}

/// Load balancing strategy for service mesh.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MeshLoadBalancingStrategy {
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
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct McpLoadBalancerConfig {
    /// Local routing strategy.
    pub local_strategy: MeshLoadBalancingStrategy,
    /// Optional service mesh integration.
    pub service_mesh_integration: Option<ServiceMeshLoadBalancerConfig>,
    /// Whether federation is enabled.
    pub federation_enabled: bool,
    /// Whether cross-primal routing is enabled.
    pub cross_primal_routing: bool,
}

/// Load balancer statistics.
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
    pub service_stats: std::collections::HashMap<String, MeshServiceStats>,
    /// MCP routing stats.
    pub mcp_routing_stats: McpRoutingStats,
    /// Federation stats if enabled.
    pub federation_stats: Option<MeshFederationStats>,
}

/// Per-service statistics for load balancer.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MeshServiceStats {
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

/// Federation statistics for load balancer.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MeshFederationStats {
    /// Active nodes.
    pub nodes_active: u32,
    /// Total capacity.
    pub total_capacity: u32,
    /// Load per node.
    pub load_distribution: std::collections::HashMap<String, f64>,
}

/// Routing statistics for a node.
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

/// Ecosystem-wide load distribution from service mesh.
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
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ScaleEventType {
    ScaleUp,
    ScaleDown,
    Rebalance,
    Emergency,
}

/// Recommendation from scaling analysis.
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

/// Requirements for scaling operation.
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

/// A Squirrel instance in the federation.
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
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FederationTopology {
    Star,
    Ring,
    Mesh,
    Tree,
}

/// Status of the federation.
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
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum InstanceStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
    Unknown,
}

/// Configuration for federation (mesh trait API).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MeshFederationConfig {
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

impl Default for MeshFederationConfig {
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

/// Federation load balancer.
#[derive(Debug)]
pub struct FederationLoadBalancer {
    #[expect(dead_code, reason = "Load metrics for federation load balancing")]
    load_metrics: std::sync::Arc<LoadMetrics>,
    #[expect(dead_code, reason = "Balancing strategy for federation routing")]
    balancing_strategy: MeshLoadBalancingStrategy,
}

impl FederationLoadBalancer {
    pub const fn new(load_metrics: std::sync::Arc<LoadMetrics>) -> Self {
        Self {
            load_metrics,
            balancing_strategy: MeshLoadBalancingStrategy::RoundRobin,
        }
    }
}
