// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: Core coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Squirrel Universal AI Primal
//!
//! A universal AI coordination primal that implements the standardized ecosystem
//! patterns for dynamic primal evolution and integration with the ecoPrimals ecosystem.
//!
//! This primal follows the universal adapter patterns defined by the ecosystem registry
//! and implements the `EcosystemServiceRegistration` standard for seamless integration.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(rust_2018_idioms)]
// Test code may use unwrap/expect for assertion clarity, exact float comparisons for known constants
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::float_cmp,
        clippy::unreadable_literal,
        clippy::field_reassign_with_default,
        clippy::many_single_char_names,
        clippy::needless_pass_by_value,
        clippy::default_trait_access,
        reason = "test code needs direct assertions and concise literals"
    )
)]
// Progressive lint tightening — architectural lints deferred until trait/API stabilization.
#![expect(
    // Architectural: docs require trait/API stabilization before completion
    missing_docs,
    clippy::missing_errors_doc,
    // Edition 2024 stabilisation noise
    async_fn_in_trait,
    // Genuine domain naming (e.g. ToadStool, BiomeOS)
    clippy::doc_markdown,
    clippy::struct_field_names,
    // Numeric casts in metrics/scoring — audited per-site
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    // Patterns under active refactor
    clippy::similar_names,
    clippy::option_if_let_else,
    clippy::significant_drop_tightening,
    // ARCHITECTURAL: async trait conformance — 199 call-sites need trait redesign
    clippy::unused_async,
    // ARCHITECTURAL: method design — 21 call-sites need API restructuring
    clippy::unused_self,
    // ARCHITECTURAL: API surface — 24 call-sites, would break callers
    clippy::needless_pass_by_ref_mut,
    // ARCHITECTURAL: builder pattern — 28 call-sites
    clippy::return_self_not_must_use,
    // Noisy in domain code — many intentional patterns
    clippy::items_after_statements,
    clippy::struct_excessive_bools,
    reason = "Progressive lint tightening per CHANGELOG")]

// Core modules
pub mod api;
pub mod capabilities;
pub mod config;
pub mod discovery;
pub mod error;
pub mod hardware;
pub mod metrics;
pub mod monitoring;
/// Niche self-knowledge: identity, capabilities, costs, dependencies.
pub mod niche;
pub mod observability;
/// Zero-copy and performance optimization utilities.
pub mod optimization;
/// Orchestration primitives for multi-primal composition (deploy graphs).
pub mod orchestration;
/// Centralized primal name hints for socket discovery (TRUE PRIMAL pattern).
pub mod primal_names;
pub mod primal_pulse;
/// Protocol definitions for inter-primal communication.
pub mod protocol;
pub mod resource_manager;
pub mod security;
pub mod session;
pub mod shutdown;
#[cfg(test)]
mod shutdown_tests;

/// Universal adapters for capability-based primal integration
pub mod universal_adapters;

/// Tool execution and management
pub mod tool;

/// JSON-RPC and tarpc protocol implementation for inter-primal communication
pub mod rpc;

/// Transport abstraction — sourDough wire-compatible.
///
/// Provides `connect_transport()` for outbound IPC without raw TCP/UDS coupling.
pub mod transport;

/// Benchmarking framework for performance measurement
#[cfg(feature = "benchmarking")]
pub mod benchmarking;

/// Graceful shutdown system
pub mod self_healing;

// Core error types
pub use error::PrimalError;

/// Result type for primal operations using `PrimalError`.
pub type PrimalResult<T> = Result<T, PrimalError>;

// Monitoring (used by main.rs binary)
pub use monitoring::metrics::MetricsCollector;
pub use monitoring::performance::PerformanceTracker as PerformanceMonitor;
pub use self_healing::SelfHealingManager;
pub use shutdown::ShutdownManager;

// Version information

/// Package version string from Cargo.toml.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Package name string from Cargo.toml.
pub const NAME: &str = env!("CARGO_PKG_NAME");

// C8: SquirrelSystem / initialize_squirrel_system / create_default_squirrel_system REMOVED.
// Ecosystem registration, primal_provider, universal/, universal_primal_ecosystem/,
// compute_client/, storage_client/, security_client/, biomeos_integration/, error_handling/
// excised as upstream absorption (Songbird/BearDog/ToadStool scaffolding).
// Archived: /tmp/squirrel-c8-excision-archive.tar.gz
