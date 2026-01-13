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
/// ```no_run
/// use squirrel::universal::{UniversalPrimalProvider, PrimalRequest, PrimalResponse};
///
/// struct MyPrimal;
///
/// impl UniversalPrimalProvider for MyPrimal {
///     // Implement required methods...
///     # fn primal_id(&self) -> &str { "my_primal" }
///     # fn instance_id(&self) -> &str { "instance_1" }
///     # fn context(&self) -> &squirrel::universal::PrimalContext { todo!() }
///     # fn primal_type(&self) -> squirrel::universal::PrimalType { squirrel::universal::PrimalType::Compute }
///     # fn capabilities(&self) -> Vec<squirrel::universal::PrimalCapability> { vec![] }
///     # fn dependencies(&self) -> Vec<squirrel::universal::PrimalDependency> { vec![] }
///     # async fn health_check(&self) -> squirrel::universal::PrimalHealth { todo!() }
///     # fn endpoints(&self) -> squirrel::universal::PrimalEndpoints { todo!() }
///     # async fn handle_primal_request(&self, request: PrimalRequest) -> squirrel::universal::UniversalResult<PrimalResponse> { todo!() }
///     # async fn initialize(&mut self, config: serde_json::Value) -> squirrel::universal::UniversalResult<()> { Ok(()) }
///     # async fn shutdown(&mut self) -> squirrel::universal::UniversalResult<()> { Ok(()) }
/// }
/// ```
pub trait UniversalPrimalProvider: Send + Sync {
    fn primal_id(&self) -> &str;
    fn instance_id(&self) -> &str;
    fn context(&self) -> &PrimalContext;
    fn primal_type(&self) -> PrimalType;
    fn capabilities(&self) -> Vec<PrimalCapability>;
    fn dependencies(&self) -> Vec<PrimalDependency>;
    async fn health_check(&self) -> PrimalHealth;
    fn endpoints(&self) -> PrimalEndpoints;
    async fn handle_primal_request(
        &self,
        request: PrimalRequest,
    ) -> UniversalResult<PrimalResponse>;
    async fn initialize(&mut self, config: Value) -> UniversalResult<()>;
    async fn shutdown(&mut self) -> UniversalResult<()>;
    fn can_serve_context(&self, context: &PrimalContext) -> bool;
    fn dynamic_port_info(&self) -> Option<DynamicPortInfo>;
    async fn register_with_songbird(&mut self, songbird_endpoint: &str) -> UniversalResult<String>;
    async fn deregister_from_songbird(&mut self) -> UniversalResult<()>;
    fn get_service_mesh_status(&self) -> ServiceMeshStatus;
    async fn handle_ecosystem_request(
        &self,
        request: EcosystemRequest,
    ) -> UniversalResult<EcosystemResponse>;
    async fn report_health(&self, health: PrimalHealth) -> UniversalResult<()>;
    async fn update_capabilities(&self, capabilities: Vec<PrimalCapability>)
        -> UniversalResult<()>;
}

/// Universal security provider trait
pub trait UniversalSecurityProvider: Send + Sync {
    /// Associated session type
    type Session;
    /// Associated error type
    type Error;

    async fn authenticate(&self, credentials: Value) -> Result<Self::Session, Self::Error>;
    async fn authorize(
        &self,
        session_id: &str,
        resource: &str,
        action: &str,
    ) -> Result<bool, Self::Error>;
    async fn health_check(&self) -> Result<bool, Self::Error>;

    /// Get session by ID
    async fn get_session(&self, session_id: &str) -> Result<Option<Self::Session>, Self::Error>;

    /// Revoke a session
    async fn revoke_session(&self, session_id: &str) -> Result<(), Self::Error>;

    /// Get provider capabilities
    async fn get_capabilities(&self) -> Result<Vec<ServiceCapability>, Self::Error>;
}
