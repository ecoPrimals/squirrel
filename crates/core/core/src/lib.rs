// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

#![allow(clippy::missing_docs_in_private_items)]
//! Compatibility module for legacy squirrel_core references
//!
//! This module re-exports types from squirrel_context for backward compatibility.

#![forbid(unsafe_code)]
// Core modules for Squirrel MCP ecosystem coordination
pub mod api;
pub mod config;
pub mod coordination;
pub mod discovery;
pub mod ecosystem;
pub mod error;
pub mod federation;
pub mod manifest;
pub mod monitoring;
pub mod routing;
pub mod service_discovery;
pub mod swarm;

// Re-export core types
pub use api::*;
pub use config::*;
pub use coordination::*;
pub use discovery::*;
pub use ecosystem::*;
pub use error::*;
pub use federation::*;
pub use manifest::*;
pub use monitoring::*;
pub use routing::*;
pub use service_discovery::*;
pub use swarm::*;

// Core result types
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Configuration(#[from] config::Error),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    #[error("No agent available")]
    NoAgentAvailable,

    #[error("Context not found: {0}")]
    ContextNotFound(String),

    #[error("Invalid context: {0}")]
    InvalidContext(String),

    #[error("Coordination error: {0}")]
    Coordination(String),

    #[error("Discovery error: {0}")]
    Discovery(String),

    #[error("Federation error: {0}")]
    Federation(String),

    #[error("Routing error: {0}")]
    Routing(String),

    #[error("Swarm error: {0}")]
    Swarm(String),

    #[error("Monitoring error: {0}")]
    Monitoring(String),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// Feature-gated reqwest::Error conversion (only available with http-client feature)
#[cfg(feature = "http-client")]
impl From<reqwest::Error> for CoreError {
    fn from(err: reqwest::Error) -> Self {
        CoreError::Http(err.to_string())
    }
}

// Core traits for ecosystem coordination
#[async_trait::async_trait]
pub trait PrimalCoordinator {
    async fn register_with_ecosystem(&self) -> Result<()>;
    async fn discover_primals(&self) -> Result<Vec<PrimalEndpoint>>;
    async fn coordinate_task(&self, task: Task) -> Result<TaskResult>;
    async fn health_check(&self) -> Result<HealthStatus>;
}

#[async_trait::async_trait]
pub trait McpRouter {
    async fn route_task(&self, task: McpTask) -> Result<TaskResponse>;
    async fn coordinate_agents(&self, agents: Vec<AgentSpec>) -> Result<CoordinationResult>;
    async fn scale_capacity(&self, requirements: ScaleRequirements) -> Result<ScaleResult>;
}

#[async_trait::async_trait]
pub trait SwarmManager {
    async fn spawn_squirrel(&self, config: SquirrelConfig) -> Result<SquirrelInstance>;
    async fn federate_nodes(&self, nodes: Vec<NodeSpec>) -> Result<FederationResult>;
    async fn balance_load(&self, load: LoadMetrics) -> Result<LoadBalanceResult>;
}

// Core data structures
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PrimalEndpoint {
    pub id: String,
    pub primal_type: PrimalType,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub health: HealthStatus,
    pub metadata: std::collections::HashMap<String, String>,
}

// Re-export canonical PrimalType from ecosystem-api for ecosystem-wide consistency
// This ensures all primals use the same type definitions for service discovery and routing
pub use ecosystem_api::PrimalType;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Task {
    pub id: String,
    pub task_type: TaskType,
    pub priority: TaskPriority,
    pub requirements: TaskRequirements,
    pub context: serde_json::Value,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TaskType {
    McpCoordination,
    AiTaskRouting,
    ContextManagement,
    StorageOperation,
    SecurityValidation,
    ComputeExecution,
    ServiceDiscovery,
    FederationManagement,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum TaskPriority {
    Critical,
    High,
    Normal,
    Low,
    Background,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TaskRequirements {
    pub cpu: Option<f64>,
    pub memory: Option<u64>,
    pub storage: Option<u64>,
    pub network: Option<f64>,
    pub required_capabilities: Vec<String>,
    pub preferred_primals: Vec<PrimalType>,
    pub constraints: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TaskResult {
    pub id: String,
    pub status: TaskStatus,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time: std::time::Duration,
    pub executed_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TaskStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
    Retrying,
}

// MCP-specific types
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct McpTask {
    pub id: String,
    pub agent_id: Option<String>,
    pub payload: serde_json::Value,
    pub context: Option<serde_json::Value>,
    pub routing_hints: Vec<String>,
    pub context_requirements: Option<ContextRequirements>,
    pub mcp_request: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TaskResponse {
    pub id: String,
    pub result: serde_json::Value,
    pub agent_id: String,
    pub execution_time: std::time::Duration,
    pub context: Option<serde_json::Value>,
    pub task_id: String,
    pub response: serde_json::Value,
    pub metadata: ResponseMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ContextRequirements {
    pub persistent_context: bool,
    pub shared_context: std::collections::HashMap<String, String>,
    pub shared_contexts: Vec<String>,
    pub required_context: std::collections::HashMap<String, String>,
    pub context_keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ResponseMetadata {
    pub context_updated: bool,
    pub processing_time: std::time::Duration,
    pub agent_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AgentSpec {
    pub id: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub weight: Option<f64>,
    pub max_concurrent_tasks: u32,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CoordinationResult {
    pub registered_agents: u32,
    pub failed_registrations: u32,
    pub total_agents: u32,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScaleResult {
    pub scaling_triggered: bool,
    pub target_instances: u32,
    pub current_instances: u32,
    pub scaling_status: String,
    pub message: String,
    pub new_capacity: u32,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LoadMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub network_usage: f64,
    pub active_tasks: u32,
    pub queue_length: u32,
    pub response_time: std::time::Duration,
    pub error_rate: f64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LoadBalanceResult {
    pub distribution: std::collections::HashMap<String, u32>,
    pub balance_score: f64,
    pub rebalance_time: std::time::Duration,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct QueuedTask {
    pub task: McpTask,
    pub queued_at: chrono::DateTime<chrono::Utc>,
    pub priority: TaskPriority,
    pub retry_count: u32,
    pub max_retries: u32,
}

// Version and identity constants
pub const SQUIRREL_MCP_VERSION: &str = "2.2.0";
pub const PRIMAL_TYPE: &str = "squirrel";

// Service Mesh Load Balancer Integration Config
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ServiceMeshLoadBalancerConfig {
    pub endpoint: String,
    pub enabled: bool,
    pub fallback_strategy: LoadBalancingStrategy,
    pub coordination_timeout: std::time::Duration,
}

// Deprecated alias for backward compatibility
#[deprecated(since = "0.1.0", note = "Use ServiceMeshLoadBalancerConfig instead")]
pub type SongbirdLoadBalancerConfig = ServiceMeshLoadBalancerConfig;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin,
    ResponseTimeBased,
    CapabilityBased,
    Adaptive,
    // Songbird-compatible strategies
    Random,
    HealthBased,
}

// Enhanced MCP Load Balancer with Songbird coordination
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct McpLoadBalancerConfig {
    pub local_strategy: LoadBalancingStrategy,
    pub service_mesh_integration: Option<ServiceMeshLoadBalancerConfig>,
    pub federation_enabled: bool,
    pub cross_primal_routing: bool,
}

// Load balancer statistics compatible with Songbird
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LoadBalancerStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: f64,
    pub active_connections: u64,
    pub service_stats: std::collections::HashMap<String, ServiceStats>,
    // Squirrel-specific stats
    pub mcp_routing_stats: McpRoutingStats,
    pub federation_stats: Option<FederationStats>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ServiceStats {
    pub requests: u64,
    pub successes: u64,
    pub failures: u64,
    pub average_response_time: f64,
    pub active_connections: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct McpRoutingStats {
    pub agents_registered: u32,
    pub tasks_routed_locally: u64,
    pub tasks_routed_to_primals: u64,
    pub tasks_routed_to_federation: u64,
    pub context_operations: u64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FederationStats {
    pub nodes_active: u32,
    pub total_capacity: u32,
    pub load_distribution: std::collections::HashMap<String, f64>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RoutingStats {
    pub node_id: String,
    pub active_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub queued_tasks: u64,
    pub registered_agents: u32,
    pub average_response_time: f64,
    pub federation_nodes: u32,
}

// Service Mesh Load Balancer Integration Trait (Capability-Based)
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
#[deprecated(
    since = "0.1.0",
    note = "Use ServiceMeshLoadBalancerIntegration instead"
)]
pub trait SongbirdLoadBalancerIntegration: ServiceMeshLoadBalancerIntegration {}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EcosystemLoadDistribution {
    pub recommended_distribution: std::collections::HashMap<String, f64>,
    pub overall_ecosystem_health: HealthStatus,
    pub scaling_recommendations: Vec<ScaleRecommendation>,
    pub cross_primal_routing_suggestions: Vec<CrossPrimalRoute>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ScaleEvent {
    pub event_type: ScaleEventType,
    pub current_load: LoadBalancerStats,
    pub projected_load: Option<LoadBalancerStats>,
    pub resource_requirements: ScaleRequirements,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ScaleEventType {
    ScaleUp,
    ScaleDown,
    Rebalance,
    Emergency,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ScaleRecommendation {
    pub action: ScaleAction,
    pub priority: TaskPriority,
    pub estimated_impact: f64,
    pub coordination_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ScaleAction {
    SpawnInstances(u32),
    TerminateInstances(u32),
    RebalanceLoad,
    DelegateToFederation,
    RequestPrimalAssistance(PrimalType),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CrossPrimalRoute {
    pub source_primal: PrimalType,
    pub target_primal: PrimalType,
    pub task_types: Vec<TaskType>,
    pub estimated_benefit: f64,
}

// Enhanced MCP Router with Service Mesh Integration
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

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ScaleRequirements {
    pub target_capacity: u32,
    pub min_instances: u32,
    pub max_instances: u32,
    pub triggers: Vec<ScaleTrigger>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ScaleTrigger {
    CpuThreshold(f64),
    MemoryThreshold(f64),
    QueueLength(u32),
    ResponseTime(std::time::Duration),
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NodeSpec {
    pub id: String,
    pub region: Option<String>,
    pub zone: Option<String>,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub capacity: u32,
}

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

// Additional types for federation support
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SquirrelInstance {
    pub id: String,
    pub node_id: String,
    pub endpoint: String,
    pub region: Option<String>,
    pub zone: Option<String>,
    pub capabilities: Vec<String>,
    pub capacity: u32,
    pub current_load: u32,
    pub health: InstanceStatus,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SquirrelConfig {
    pub node_id: String,
    pub port: u16,
    pub region: Option<String>,
    pub zone: Option<String>,
    pub capabilities: Vec<String>,
    pub capacity: u32,
    pub federation_enabled: bool,
    pub auto_scaling_enabled: bool,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FederationResult {
    pub federation_id: String,
    pub nodes_joined: u32,
    pub total_capacity: u32,
    pub status: FederationStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FederationTopology {
    Star,
    Ring,
    Mesh,
    Tree,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FederationStatus {
    Forming,
    Active,
    Degraded,
    Inactive,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum InstanceStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FederationConfig {
    pub node_id: String,
    pub port: u16,
    pub federation_discovery_urls: Vec<String>,
    pub auto_scaling_enabled: bool,
    pub min_instances: u32,
    pub max_instances: u32,
    pub scale_up_threshold: f64,
    pub scale_down_threshold: f64,
    pub health_check_interval: chrono::Duration,
    pub federation_timeout: chrono::Duration,
    pub federation_enabled: bool,
}

// Missing types referenced in federation.rs
#[derive(Debug)]
pub struct FederationLoadBalancer {
    #[expect(dead_code, reason = "Load metrics for federation load balancing")]
    load_metrics: std::sync::Arc<LoadMetrics>,
    #[expect(dead_code, reason = "Balancing strategy for federation routing")]
    balancing_strategy: LoadBalancingStrategy,
}

impl FederationLoadBalancer {
    pub fn new(load_metrics: std::sync::Arc<LoadMetrics>) -> Self {
        Self {
            load_metrics,
            balancing_strategy: LoadBalancingStrategy::RoundRobin,
        }
    }
}

// RoutingStats is already defined earlier in the file
