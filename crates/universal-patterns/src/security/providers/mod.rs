// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Universal Security Capability Definitions
//!
//! This module defines security capabilities and traits that any security service
//! can implement, following the Universal Capability-Based Adapter Pattern.
//!
//! Instead of hardcoding specific provider names, we define what capabilities
//! security services should provide and how they integrate universally.

mod boxed;
mod local;
mod registry;
mod security_provider;
mod types;

#[cfg(test)]
mod tests_integration;
#[cfg(test)]
mod tests_registry;
#[cfg(test)]
mod tests_types;

pub use boxed::UniversalSecurityProviderBox;
pub use local::LocalSecurityProvider;
#[cfg_attr(
    not(test),
    expect(
        unused_imports,
        reason = "Re-export surface for consumers; unused in this module"
    )
)]
pub use registry::{UniversalSecurityRegistry, capabilities_match, register_security_service};
#[cfg_attr(
    not(test),
    expect(
        unused_imports,
        reason = "Test-only re-exports for providers test modules"
    )
)]
pub use security_provider::{SECURITY_PRIMARY_SERVICE_ID, SECURITY_SERVICE_ID};
pub use security_provider::{SecurityProviderFactory, SecurityProviderIntegration};
pub use types::*;
