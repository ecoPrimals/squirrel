// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: Universal pattern mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

#![expect(
    clippy::unused_async,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::missing_const_for_fn,
    clippy::doc_markdown,
    clippy::uninlined_format_args,
    clippy::use_self,
    clippy::redundant_closure_for_method_calls,
    clippy::needless_pass_by_value,
    clippy::return_self_not_must_use,
    clippy::match_same_arms,
    clippy::significant_drop_tightening,
    clippy::cast_possible_truncation,
    clippy::derive_partial_eq_without_eq,
    clippy::option_if_let_else,
    clippy::cloned_instead_of_copied,
    reason = "Large pattern library; progressive lint and doc tightening"
)]

//! # Universal Patterns — Transport & IPC
//!
//! Transport, IPC client, manifest discovery, and operational utilities
//! for the ecoPrimals ecosystem.
//!
//! C8 excision removed federation, registry, security, config, traits,
//! builder, circuit_breaker, compute_dispatch, dispatch_outcome, streaming,
//! and validation_harness modules (upstream absorption scaffolding).

#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "test code needs direct assertions"
    )
)]
#![warn(clippy::all)]
#![warn(rust_2018_idioms)]
#![warn(missing_docs)]

pub mod ipc_client;
pub mod manifest_discovery;
pub mod or_exit;
pub mod provenance;
pub mod transport;

pub use ipc_client::{
    CapabilityInfo, IpcClient, IpcClientError, extract_rpc_error, extract_rpc_result,
    parse_capabilities_from_response,
};
pub use manifest_discovery::PrimalManifest;
pub use or_exit::{OrExit, exit_codes};
pub use transport::{
    ListenerConfig, RemoteAddr, TransportConfig, TransportType, UniversalListener,
    UniversalTransport,
};

/// Version information for the universal patterns framework
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get version information
pub fn version() -> &'static str {
    VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_returns_non_empty() {
        let v = version();
        assert!(!v.is_empty());
        assert_eq!(v, VERSION);
    }
}
