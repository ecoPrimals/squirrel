// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Multi-node federation, scaling, and load coordination.
//!
//! The [`FederationService`] coordinates local instances and peer membership; configuration
//! and observability types live in [`types`].

mod service;
mod service_swarm;
mod service_types;
mod types;

pub use service::FederationService;
pub use types::{FederationConfig, FederationNode, FederationStats, NodeHealth, ScalingPolicy};
