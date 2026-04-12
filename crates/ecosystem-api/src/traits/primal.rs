// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Core primal and ecosystem integration traits.

use std::future::Future;
use std::pin::Pin;

use crate::error::{EcosystemError, UniversalResult};
use crate::types::{
    DynamicPortInfo, EcosystemRequest, EcosystemResponse, HealthStatus, PrimalCapability,
    PrimalContext, PrimalDependency, PrimalEndpoints, PrimalHealth, PrimalRequest, PrimalResponse,
    PrimalType, ServiceCapabilities, ServiceMeshStatus,
};

/// Universal primal provider trait - ALL PRIMALS MUST IMPLEMENT
///
/// This trait defines the standard interface for all primals in the ecosystem.
/// It provides the foundation for service discovery, health monitoring, and
/// inter-primal communication.
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
    fn health_check(&self) -> Pin<Box<dyn Future<Output = PrimalHealth> + Send + '_>>;

    /// Get primal API endpoints
    fn endpoints(&self) -> PrimalEndpoints;

    /// Handle inter-primal communication
    fn handle_primal_request(
        &self,
        request: PrimalRequest,
    ) -> Pin<Box<dyn Future<Output = UniversalResult<PrimalResponse>> + Send + '_>>;

    /// Initialize the primal with configuration
    fn initialize(
        &mut self,
        config: serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = UniversalResult<()>> + Send + '_>>;

    /// Shutdown the primal gracefully
    fn shutdown(&mut self) -> Pin<Box<dyn Future<Output = UniversalResult<()>> + Send + '_>>;

    /// Check if this primal can serve the given context
    fn can_serve_context(&self, context: &PrimalContext) -> bool;

    /// Get dynamic port information (managed by service mesh)
    fn dynamic_port_info(&self) -> Option<DynamicPortInfo>;

    /// Register with service mesh
    fn register_with_service_mesh(
        &mut self,
        service_mesh_endpoint: &str,
    ) -> Pin<Box<dyn Future<Output = UniversalResult<String>> + Send + '_>>;

    /// Deregister from service mesh
    fn deregister_from_service_mesh(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = UniversalResult<()>> + Send + '_>>;

    /// Get service mesh status
    fn get_service_mesh_status(&self) -> ServiceMeshStatus;

    /// Handle ecosystem request (standardized format)
    fn handle_ecosystem_request(
        &self,
        request: EcosystemRequest,
    ) -> Pin<Box<dyn Future<Output = UniversalResult<EcosystemResponse>> + Send + '_>>;

    /// Update capabilities dynamically
    fn update_capabilities(
        &self,
        capabilities: Vec<PrimalCapability>,
    ) -> Pin<Box<dyn Future<Output = UniversalResult<()>> + Send + '_>>;

    /// Report health status
    fn report_health(
        &self,
        health: PrimalHealth,
    ) -> Pin<Box<dyn Future<Output = UniversalResult<()>> + Send + '_>>;
}
/// Ecosystem integration trait - ALL PRIMALS MUST IMPLEMENT
///
/// This trait handles communication with the broader ecosystem through
/// the service mesh. It provides standardized request/response
/// handling and service lifecycle management.
#[expect(
    async_fn_in_trait,
    reason = "internal trait — all impls are Send + Sync"
)]
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
