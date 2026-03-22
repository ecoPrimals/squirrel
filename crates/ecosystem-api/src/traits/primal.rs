// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Core primal and ecosystem integration traits.

use crate::error::{EcosystemError, UniversalResult};
use crate::types::{
    DynamicPortInfo, EcosystemRequest, EcosystemResponse, HealthStatus, PrimalCapability,
    PrimalContext, PrimalDependency, PrimalEndpoints, PrimalHealth, PrimalRequest, PrimalResponse,
    PrimalType, ServiceCapabilities, ServiceMeshStatus,
};

use async_trait::async_trait;
/// Universal primal provider trait - ALL PRIMALS MUST IMPLEMENT
///
/// This trait defines the standard interface for all primals in the ecosystem.
/// It provides the foundation for service discovery, health monitoring, and
/// inter-primal communication.
#[async_trait]
pub trait UniversalPrimalProvider: Send + Sync {
    /// Unique primal identifier (e.g., "squirrel", "beardog", "nestgate")
    fn primal_id(&self) -> &str;

    /// Instance identifier for multi-instance support
    fn instance_id(&self) -> &str;

    /// User/device context this primal instance serves
    fn context(&self) -> &PrimalContext;

    /// Primal type category
    fn primal_type(&self) -> PrimalType;

    /// Capabilities this primal provides
    fn capabilities(&self) -> Vec<PrimalCapability>;

    /// What this primal needs from other primals
    fn dependencies(&self) -> Vec<PrimalDependency>;

    /// Health check for this primal
    async fn health_check(&self) -> PrimalHealth;

    /// Get primal API endpoints
    fn endpoints(&self) -> PrimalEndpoints;

    /// Handle inter-primal communication
    async fn handle_primal_request(
        &self,
        request: PrimalRequest,
    ) -> UniversalResult<PrimalResponse>;

    /// Initialize the primal with configuration
    async fn initialize(&mut self, config: serde_json::Value) -> UniversalResult<()>;

    /// Shutdown the primal gracefully
    async fn shutdown(&mut self) -> UniversalResult<()>;

    /// Check if this primal can serve the given context
    fn can_serve_context(&self, context: &PrimalContext) -> bool;

    /// Get dynamic port information (managed by service mesh)
    fn dynamic_port_info(&self) -> Option<DynamicPortInfo>;

    /// Register with service mesh
    async fn register_with_service_mesh(
        &mut self,
        service_mesh_endpoint: &str,
    ) -> UniversalResult<String>;

    /// Deregister from service mesh
    async fn deregister_from_service_mesh(&mut self) -> UniversalResult<()>;

    /// Get service mesh status
    fn get_service_mesh_status(&self) -> ServiceMeshStatus;

    /// Handle ecosystem request (standardized format)
    async fn handle_ecosystem_request(
        &self,
        request: EcosystemRequest,
    ) -> UniversalResult<EcosystemResponse>;

    /// Update capabilities dynamically
    async fn update_capabilities(&self, capabilities: Vec<PrimalCapability>)
    -> UniversalResult<()>;

    /// Report health status
    async fn report_health(&self, health: PrimalHealth) -> UniversalResult<()>;
}
/// Ecosystem integration trait - ALL PRIMALS MUST IMPLEMENT
///
/// This trait handles communication with the broader ecosystem through
/// the service mesh. It provides standardized request/response
/// handling and service lifecycle management.
#[async_trait]
pub trait EcosystemIntegration: Send + Sync {
    /// Register service with service mesh
    async fn register_with_service_mesh(&self) -> Result<String, EcosystemError>;

    /// Handle incoming requests from other services
    async fn handle_ecosystem_request(
        &self,
        request: EcosystemRequest,
    ) -> Result<EcosystemResponse, EcosystemError>;

    /// Report health status to Songbird
    async fn report_health(&self, health: HealthStatus) -> Result<(), EcosystemError>;

    /// Update service capabilities
    async fn update_capabilities(
        &self,
        capabilities: ServiceCapabilities,
    ) -> Result<(), EcosystemError>;

    /// Deregister from ecosystem
    async fn deregister(&self) -> Result<(), EcosystemError>;
}
