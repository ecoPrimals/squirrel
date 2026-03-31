// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Universal Security Capability Definitions
//!
//! This module defines security capabilities and traits that any security service
//! can implement, following the Universal Capability-Based Adapter Pattern.
//!
//! Instead of hardcoding specific provider names, we define what capabilities
//! security services should provide and how they integrate universally.

#![allow(dead_code)] // Public capability surface for external consumers

mod beardog;
mod local;
mod registry;
mod types;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

pub use beardog::{BeardogIntegration, BeardogSecurityProvider};
pub use local::LocalSecurityProvider;
// Used by `providers/tests.rs` and external callers; not referenced from non-test lib code.
#[allow(unused_imports)]
pub use registry::{UniversalSecurityRegistry, capabilities_match, register_security_service};
pub use types::*;
