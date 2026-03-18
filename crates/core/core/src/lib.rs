// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Compatibility module for legacy `squirrel_core` references
//!
//! This module re-exports types from `squirrel_context` for backward compatibility.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![allow(missing_docs)]
#![allow(clippy::unused_self)]
#![allow(clippy::unused_async)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::branches_sharing_code)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::significant_drop_tightening)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::float_cmp)]
#![allow(clippy::needless_collect)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::significant_drop_in_scrutinee)]
#![allow(clippy::cast_sign_loss)]
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
        Self::Http(err.to_string())
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

// Deprecated alias for backward compatibility
#[cfg(feature = "mesh")]
#[deprecated(since = "0.1.0", note = "Use ServiceMeshLoadBalancerConfig instead")]
pub type SongbirdLoadBalancerConfig = types::ServiceMeshLoadBalancerConfig;

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
