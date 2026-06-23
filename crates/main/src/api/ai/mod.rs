// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! AI routing and provider selection.
//!
//! Single adapter path: `AiRouter` → `router_discovery` → `adapters::UniversalAiAdapter`.
//! Parallel adapter/bridge/discovery stacks removed in Wave 124 (1,811 lines of dead code).

pub mod action_registry;
pub mod adapters;
pub mod constraint_router;
pub mod constraints;
pub mod dignity;
pub mod http_provider_config;
pub mod router;
mod router_discovery;
mod router_init;
pub mod selector;
pub mod types;

pub use router::AiRouter;
