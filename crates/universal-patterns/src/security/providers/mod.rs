// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Universal Security Capability Definitions
//!
//! This module defines security capabilities and traits that any security service
//! can implement, following the Universal Capability-Based Adapter Pattern.
//!
//! Instead of hardcoding specific provider names, we define what capabilities
//! security services should provide and how they integrate universally.

mod beardog;
mod boxed;
mod local;
mod registry;
mod types;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg_attr(
    not(test),
    expect(unused_imports, reason = "Test-only re-exports for providers/tests.rs")
)]
#[expect(
    deprecated,
    reason = "legacy wire id constants re-exported for backward compatibility"
)]
pub use beardog::{BEARDOG_SECURITY_SERVICE_ID, SECURITY_PRIMARY_SERVICE_ID};
#[expect(
    deprecated,
    reason = "legacy type and factory aliases re-exported for backward compatibility"
)]
pub use beardog::{
    BeardogIntegration, BeardogSecurityProvider, SECURITY_SERVICE_ID, SecurityProviderFactory,
    SecurityProviderIntegration,
};
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
pub use types::*;
