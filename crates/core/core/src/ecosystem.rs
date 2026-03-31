// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors
//!
//! Ecosystem coordination: service lifecycle, peer discovery, and task routing.

#[path = "ecosystem_coordination.rs"]
mod ecosystem_coordination;
#[path = "ecosystem_service.rs"]
mod ecosystem_service;
#[path = "ecosystem_state.rs"]
mod ecosystem_state;
#[path = "ecosystem_types.rs"]
mod ecosystem_types;

pub use ecosystem_service::EcosystemService;
pub use ecosystem_state::ServiceStatus;

#[cfg(test)]
#[path = "ecosystem_tests.rs"]
mod tests;
