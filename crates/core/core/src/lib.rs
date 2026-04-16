// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: Core coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![warn(missing_docs)]

//! Compatibility module for legacy `squirrel_core` references
//!
//! This module re-exports types from `squirrel_context` for backward compatibility.

// Core modules for Squirrel MCP ecosystem coordination
/// HTTP API server, handlers, and wire types.
#[cfg(feature = "http-api")]
pub mod api;
/// HTTP API surface for Squirrel MCP when the `http-api` feature is enabled.
#[cfg(feature = "http-api")]
mod api_types;
/// Configuration loading and validation.
pub mod config;
/// Primal coordination primitives for mesh deployments.
#[cfg(feature = "mesh")]
pub mod coordination;
/// Service and primal discovery.
pub mod discovery;
/// Ecosystem coordination, discovery, and task routing across primals.
#[cfg(feature = "mesh")]
pub mod ecosystem;
/// Error types for core operations.
pub mod error;
/// Multi-node federation, scaling, and load coordination.
#[cfg(feature = "mesh")]
pub mod federation;
/// Manifest parsing and validation.
pub mod manifest;
/// Health, metrics, and operational event recording.
pub mod monitoring;
/// MCP task routing, agents, and context management.
#[cfg(feature = "mesh")]
pub mod routing;
/// Pluggable service registry and discovery queries.
#[cfg(feature = "mesh")]
pub mod service_discovery;
/// Swarm-level coordination for multi-instance orchestration (Phase 2).
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

/// Core type definitions.
pub mod types;

pub use types::*;

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

    /// Required capability is not available on this primal; discover via IPC.
    #[error("Capability unavailable: {capability}. {hint}")]
    CapabilityUnavailable {
        /// Capability id (e.g. `http.client`, `federation:probe-node`).
        capability: String,
        /// How to resolve (registry, socket env, service-mesh delegation, etc.).
        hint: String,
    },

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

/// Trait for coordinating tasks across primal ecosystem.
#[expect(
    async_fn_in_trait,
    reason = "internal trait — all impls are Send + Sync"
)]
pub trait PrimalCoordinator: Send + Sync {
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
#[expect(
    async_fn_in_trait,
    reason = "internal trait — all impls are Send + Sync"
)]
/// Routes MCP tasks and coordinates agents and capacity within the mesh.
pub trait McpRouter: Send + Sync {
    /// Routes a single MCP task to an appropriate handler or primal.
    async fn route_task(&self, task: McpTask) -> Result<TaskResponse>;
    /// Registers and coordinates the given agents for multi-agent work.
    async fn coordinate_agents(&self, agents: Vec<AgentSpec>) -> Result<CoordinationResult>;
    /// Scales capacity up or down according to the given requirements.
    async fn scale_capacity(&self, requirements: ScaleRequirements) -> Result<ScaleResult>;
}

#[cfg(feature = "mesh")]
#[expect(
    async_fn_in_trait,
    reason = "internal trait — all impls are Send + Sync"
)]
/// Spawns instances, joins federation nodes, and rebalances load across the swarm.
pub trait SwarmManager: Send + Sync {
    /// Creates a new Squirrel instance from the supplied configuration.
    async fn spawn_squirrel(&self, config: SquirrelConfig) -> Result<SquirrelInstance>;
    /// Adds the given nodes to the federation and returns aggregate results.
    async fn federate_nodes(&self, nodes: Vec<NodeSpec>) -> Result<FederationResult>;
    /// Recomputes load distribution using the provided load snapshot.
    async fn balance_load(&self, load: LoadMetrics) -> Result<LoadBalanceResult>;
}

// Re-export canonical PrimalType from ecosystem-api for ecosystem-wide consistency
pub use ecosystem_api::PrimalType;

/// Squirrel MCP protocol version.
pub const SQUIRREL_MCP_VERSION: &str = "2.2.0";
/// Primal type identifier for Squirrel.
pub const PRIMAL_TYPE: &str = "squirrel";

// Backward-compat aliases for mesh types (distinct from federation/routing/service_discovery types)
#[cfg(feature = "mesh")]
pub use types::{
    MeshFederationConfig, MeshFederationStats, MeshLoadBalancingStrategy, MeshServiceStats,
};

// Deprecated alias for backward compatibility (`ServiceMeshLoadBalancerConfig` is the canonical name, re-exported from [`types`]).
#[cfg(feature = "mesh")]
#[deprecated(
    since = "0.2.0",
    note = "Use ServiceMeshLoadBalancerConfig (service mesh is discovered by capability at runtime)"
)]
/// Deprecated alias for [`ServiceMeshLoadBalancerConfig`].
pub type SongbirdLoadBalancerConfig = types::ServiceMeshLoadBalancerConfig;

// Service Mesh Load Balancer Integration Trait (Capability-Based)
#[cfg(feature = "mesh")]
#[expect(
    async_fn_in_trait,
    reason = "internal trait — all impls are Send + Sync"
)]
/// Integrates Squirrel with an external service mesh for registration, routing, and scaling signals.
pub trait ServiceMeshLoadBalancerIntegration: Send + Sync {
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
    since = "0.2.0",
    note = "Use ServiceMeshLoadBalancerIntegration (discover mesh capabilities at runtime)"
)]
/// Deprecated marker trait; use [`ServiceMeshLoadBalancerIntegration`] instead.
pub trait SongbirdLoadBalancerIntegration: ServiceMeshLoadBalancerIntegration {}

// Enhanced MCP Router with Service Mesh Integration
#[cfg(feature = "mesh")]
#[expect(
    async_fn_in_trait,
    reason = "internal trait — all impls are Send + Sync"
)]
/// Combines mesh routing with service mesh registration, stats, and cross-primal coordination.
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
