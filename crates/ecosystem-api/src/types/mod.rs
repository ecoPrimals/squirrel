// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Shared types for ecosystem integration
//!
//! This module contains all the standardized types used across the ecoPrimals
//! ecosystem for communication through the Songbird service mesh.
//!
//! Submodules group related types:
//! - `security` — Security context, levels, and configuration
//! - `context` — Primal context and network location
//! - `primal` — Primal types, capabilities, and dependencies
//! - `request` — Request/response formats
//! - `health` — Health status and resource usage
//! - `registration` — Service registration and resource specs
//! - `service_mesh` — Service mesh status

mod context;
mod health;
mod primal;
mod registration;
mod request;
mod security;
mod service_mesh;

#[cfg(test)]
#[allow(
    clippy::module_inception,
    reason = "Nested `types` module groups generated and hand-written type definitions"
)]
mod tests;

// Re-export all types for backward compatibility
pub use context::{NetworkLocation, PrimalContext};
pub use health::{HealthCheckConfig, HealthStatus, PrimalEndpoints, PrimalHealth, ResourceUsage};
pub use primal::{PrimalCapability, PrimalDependency, PrimalType};
pub use registration::{
    DynamicPortInfo, EcosystemServiceRegistration, ResourceSpec, ServiceCapabilities,
    ServiceEndpoints,
};
pub use request::{
    EcosystemRequest, EcosystemResponse, PrimalRequest, PrimalResponse, ResponseStatus,
};
pub use security::{SecurityConfig, SecurityContext, SecurityLevel};
pub use service_mesh::ServiceMeshStatus;
