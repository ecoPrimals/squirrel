// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors
#![expect(deprecated, reason = "Backward compatibility during migration")]
#![expect(dead_code, reason = "Registry types awaiting full ecosystem wiring")]

//! Core types for the ecosystem registry manager
//!
//! Submodules group interning, serde helpers, discovery DTOs, API messages, events, and status.

mod api;
mod arc_serde;
mod discovered;
mod events;
mod health;
mod interning;
mod registry_state;
mod status;

pub use api::{PrimalApiRequest, PrimalApiResponse};
pub use discovered::DiscoveredService;
pub use events::EcosystemRegistryEvent;
pub use health::{HealthCheckResult, ServiceHealthStatus};
pub use interning::intern_registry_string;
pub use registry_state::RegistryState;
pub use status::{EcosystemStatus, PrimalStatus, ServiceStatus};

#[cfg(test)]
#[path = "types_tests.rs"]
mod tests;
