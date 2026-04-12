// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Primal Provider trait for songbird compatibility.

use std::future::Future;
use std::pin::Pin;

use super::{
    DynamicPortInfo, PrimalCapability, PrimalContext, PrimalDependency, PrimalEndpoints,
    PrimalHealth, PrimalRequest, PrimalResponse, PrimalResult, PrimalType,
};

/// Songbird-compatible Universal Primal Provider trait
/// This trait enables full compatibility with songbird's orchestration system
pub trait PrimalProvider: Send + Sync {
    /// Unique primal identifier (e.g., "beardog", "nestgate", "toadstool", "squirrel")
    fn primal_id(&self) -> &str;

    /// Instance identifier for multi-instance support (e.g., "beardog-user123", "beardog-device456")
    fn instance_id(&self) -> &str;

    /// User/device context this primal instance serves
    fn context(&self) -> &PrimalContext;

    /// Primal type category (e.g., Security, Storage, Compute, AI)
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
    ) -> Pin<Box<dyn Future<Output = PrimalResult<PrimalResponse>> + Send + '_>>;

    /// Initialize the primal with configuration
    fn initialize(
        &mut self,
        config: serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = PrimalResult<()>> + Send + '_>>;

    /// Shutdown the primal gracefully
    fn shutdown(&mut self) -> Pin<Box<dyn Future<Output = PrimalResult<()>> + Send + '_>>;

    /// Check if this primal can serve the given context
    fn can_serve_context(&self, context: &PrimalContext) -> bool;

    /// Get dynamic port information
    fn dynamic_port_info(&self) -> Option<DynamicPortInfo>;
}
