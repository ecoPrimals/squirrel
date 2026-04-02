// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Core traits for universal primal integration
//!
//! This module defines the trait interfaces that enable primals to
//! integrate with the ecosystem and provide services.

use serde_json::Value;

use crate::error::PrimalError;

use super::context::PrimalContext;
use super::endpoints::{DynamicPortInfo, PrimalEndpoints};
use super::health::{PrimalDependency, PrimalHealth};
use super::messages::{EcosystemRequest, EcosystemResponse, PrimalRequest, PrimalResponse};
use super::service::ServiceCapability;
use super::service_mesh::ServiceMeshStatus;
use super::types::{PrimalCapability, PrimalType};

/// Universal result type for all primal operations
pub type UniversalResult<T> = Result<T, PrimalError>;

/// Universal trait that ANY primal can implement for ecosystem integration
///
/// This trait defines the standard interface for all primals in the Squirrel
/// ecosystem, enabling discovery, health monitoring, and inter-primal communication.
///
/// # Examples
///
/// ```ignore
/// use squirrel::universal::{UniversalPrimalProvider, PrimalRequest, PrimalResponse};
///
/// struct MyPrimal { /* fields */ }
///
/// impl UniversalPrimalProvider for MyPrimal {
///     fn primal_id(&self) -> &str { "my_primal" }
///     fn instance_id(&self) -> &str { "instance_1" }
///     // ... implement remaining required methods per trait signature
/// }
/// ```
pub trait UniversalPrimalProvider: Send + Sync {
    /// Returns the primal's unique identifier.
    fn primal_id(&self) -> &str;
    /// Returns the instance identifier for this primal instance.
    fn instance_id(&self) -> &str;
    /// Returns the primal's execution context.
    fn context(&self) -> &PrimalContext;
    /// Returns the type of primal (e.g., Compute, Storage).
    fn primal_type(&self) -> PrimalType;
    /// Returns the capabilities this primal provides.
    fn capabilities(&self) -> Vec<PrimalCapability>;
    /// Returns dependencies on other primals or services.
    fn dependencies(&self) -> Vec<PrimalDependency>;
    /// Performs a health check and returns status.
    async fn health_check(&self) -> PrimalHealth;
    /// Returns the primal's exposed endpoints.
    fn endpoints(&self) -> PrimalEndpoints;
    /// Handles an incoming primal-to-primal request.
    async fn handle_primal_request(
        &self,
        request: PrimalRequest,
    ) -> UniversalResult<PrimalResponse>;
    /// Initializes the primal with the given configuration.
    async fn initialize(&mut self, config: Value) -> UniversalResult<()>;
    /// Gracefully shuts down the primal.
    async fn shutdown(&mut self) -> UniversalResult<()>;
    /// Returns whether this primal can serve the given context.
    fn can_serve_context(&self, context: &PrimalContext) -> bool;
    /// Returns dynamic port information if applicable.
    fn dynamic_port_info(&self) -> Option<DynamicPortInfo>;
    /// Registers this primal with the service mesh.
    async fn register_with_service_mesh(
        &mut self,
        service_mesh_endpoint: &str,
    ) -> UniversalResult<String>;
    /// Deregisters this primal from the service mesh.
    async fn deregister_from_service_mesh(&mut self) -> UniversalResult<()>;
    /// Returns the current service mesh integration status.
    fn get_service_mesh_status(&self) -> ServiceMeshStatus;
    /// Handles an ecosystem-level request.
    async fn handle_ecosystem_request(
        &self,
        request: EcosystemRequest,
    ) -> UniversalResult<EcosystemResponse>;
    /// Reports health status to the ecosystem.
    async fn report_health(&self, health: PrimalHealth) -> UniversalResult<()>;
    /// Updates the primal's advertised capabilities.
    async fn update_capabilities(&self, capabilities: Vec<PrimalCapability>)
    -> UniversalResult<()>;
}

/// Universal security provider trait
pub trait UniversalSecurityProvider: Send + Sync {
    /// Associated session type
    type Session;
    /// Associated error type
    type Error;

    /// Authenticates credentials and returns a session.
    async fn authenticate(&self, credentials: Value) -> Result<Self::Session, Self::Error>;
    /// Checks if the session is authorized for the resource and action.
    async fn authorize(
        &self,
        session_id: &str,
        resource: &str,
        action: &str,
    ) -> Result<bool, Self::Error>;
    /// Performs a health check on the security provider.
    async fn health_check(&self) -> Result<bool, Self::Error>;

    /// Get session by ID
    async fn get_session(&self, session_id: &str) -> Result<Option<Self::Session>, Self::Error>;

    /// Revoke a session
    async fn revoke_session(&self, session_id: &str) -> Result<(), Self::Error>;

    /// Get provider capabilities
    async fn get_capabilities(&self) -> Result<Vec<ServiceCapability>, Self::Error>;
}
