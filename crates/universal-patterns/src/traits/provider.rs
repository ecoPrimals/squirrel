// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Primal Provider trait for songbird compatibility.

use async_trait::async_trait;

use super::{
    DynamicPortInfo, PrimalCapability, PrimalContext, PrimalDependency, PrimalEndpoints,
    PrimalHealth, PrimalRequest, PrimalResponse, PrimalResult, PrimalType,
};

/// Songbird-compatible Universal Primal Provider trait
/// This trait enables full compatibility with songbird's orchestration system
#[async_trait]
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
    async fn health_check(&self) -> PrimalHealth;

    /// Get primal API endpoints
    fn endpoints(&self) -> PrimalEndpoints;

    /// Handle inter-primal communication
    async fn handle_primal_request(&self, request: PrimalRequest) -> PrimalResult<PrimalResponse>;

    /// Initialize the primal with configuration
    async fn initialize(&mut self, config: serde_json::Value) -> PrimalResult<()>;

    /// Shutdown the primal gracefully
    async fn shutdown(&mut self) -> PrimalResult<()>;

    /// Check if this primal can serve the given context
    fn can_serve_context(&self, context: &PrimalContext) -> bool;

    /// Get dynamic port information
    fn dynamic_port_info(&self) -> Option<DynamicPortInfo>;
}
