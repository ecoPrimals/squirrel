// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: AI coordination mechanics licensed under ORC
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
    )
)]
// Progressive lint tightening — categorized by remediation type.
//
// ARCHITECTURAL: These require trait/API redesign, not simple code edits.
// Each will be resolved as the module is refactored.
#![allow(
    // Architectural: docs require trait/API stabilization before completion
    missing_docs,
    clippy::missing_errors_doc,
    // Edition 2024 stabilisation noise
    async_fn_in_trait,
    // Genuine domain naming (e.g. ToadStool, BiomeOS)
    clippy::doc_markdown,
    clippy::struct_field_names,
    // Deprecated migration still in-flight
    deprecated,
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
pub mod biomeos_integration;
pub mod capabilities; // NEW - Capability traits (Phase 2)
// capability and capability_registry removed - HTTP-based legacy discovery
pub mod chaos;
pub mod compute_client;
pub mod config;
pub mod discovery; // NEW - Capability-based discovery (Phase 2)
pub mod ecosystem;
pub mod error;
/// Error handling utilities and safe operation wrappers.
pub mod error_handling;
pub mod hardware;
pub mod metrics; // Capability-based metrics and observability
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
pub mod primal_provider;
pub mod primal_pulse; // PrimalPulse - AI-powered ecosystem intelligence
/// Protocol definitions for inter-primal communication.
pub mod protocol;
pub mod resource_manager;
pub mod security;
pub mod security_client;
pub mod session;
pub mod shutdown;
pub mod storage_client;
pub mod universal;
pub mod universal_adapter;
pub mod universal_adapter_v2;
pub mod universal_primal_ecosystem;
pub mod universal_provider;

/// Universal adapters for capability-based primal integration
pub mod universal_adapters;

/// Tool execution and management
pub mod tool;

/// JSON-RPC and tarpc protocol implementation for inter-primal communication
pub mod rpc;

/// Benchmarking framework for performance measurement
pub mod benchmarking;

/// Graceful shutdown system
pub mod self_healing;

// Re-export commonly used types for convenience
pub use biomeos_integration::SquirrelBiomeOSIntegration;
pub use compute_client::{
    UniversalComputeClient, UniversalComputeRequest, UniversalComputeResponse,
};
pub use ecosystem::{
    ComponentHealth,
    EcosystemPrimalType,
    EcosystemRegistryEvent,
    // EcosystemRegistryManager removed - HTTP-based registry
    EcosystemServiceRegistration,
    EcosystemStatus,
    initialize_ecosystem_integration,
};
// error_handling::prelude removed - safe_operations deleted
pub use monitoring::performance::PerformanceTracker;
pub use optimization::zero_copy;
pub use primal_provider::SquirrelPrimalProvider;
pub use security::BeardogSecurityCoordinator;
pub use security_client::{
    UniversalSecurityClient, UniversalSecurityRequest, UniversalSecurityResponse,
};
pub use storage_client::{
    UniversalStorageClient, UniversalStorageRequest, UniversalStorageResponse,
};
// Universal types (selective re-exports to avoid conflicts)
pub use universal::{
    DynamicPortInfo, HealthStatus, NetworkLocation, PrimalCapability, PrimalContext,
    PrimalDependency, PrimalEndpoints, PrimalHealth, PrimalRequest, PrimalResponse, PrimalType,
    SecurityLevel, ServiceMeshStatus,
};

pub use universal_adapter::{AdapterStatus, UniversalAdapter}; // Updated exports
pub use universal_provider::UniversalSquirrelProvider;

// Ecosystem API re-exports (selective to avoid conflicts)
pub use ecosystem_api::{
    error::EcosystemError,
    traits::{EcosystemIntegration, UniversalPrimalProvider as UniversalProviderTrait},
    types::{EcosystemRequest, EcosystemResponse, ResponseStatus},
};

// Convenient type aliases for common patterns

/// Result type for ecosystem operations using `EcosystemError`.
pub type UniversalResult<T> = Result<T, EcosystemError>;

/// Result type for primal operations using `PrimalError`.
pub type PrimalResult<T> = Result<T, PrimalError>;

// Core error types (selective re-exports)
pub use error::PrimalError;

// Conditional re-exports based on feature flags (selective)
#[cfg(feature = "monitoring")]
pub use monitoring::{health::HealthMonitor, metrics::MetricsCollector};

#[cfg(feature = "ecosystem")]
pub use ecosystem::{EcosystemConfig, EcosystemManager};

/// Standard exports commonly used by ecosystem consumers
pub mod prelude {
    pub use super::{
        EcosystemConfig, EcosystemError, EcosystemIntegration, EcosystemManager,
        EcosystemServiceRegistration, EcosystemStatus, UniversalProviderTrait,
        UniversalSquirrelProvider,
    };
    pub use crate::error::PrimalError;
    pub use crate::universal::*;
}

// Backward compatibility alias
pub use benchmarking::BenchmarkSuite;
pub use monitoring::performance::PerformanceTracker as PerformanceMonitor;
pub use self_healing::SelfHealingManager;
pub use shutdown::ShutdownManager;

// Version information

/// Package version string from Cargo.toml.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Package name string from Cargo.toml.
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Initialize the Squirrel MCP system with comprehensive ecosystem integration
pub async fn initialize_squirrel_system(
    config: EcosystemConfig,
) -> Result<SquirrelSystem, crate::error::PrimalError> {
    // Initialize metrics collector
    let metrics_collector = std::sync::Arc::new(monitoring::metrics::MetricsCollector::new());

    // Initialize ecosystem manager
    let ecosystem_manager =
        initialize_ecosystem_integration(config, metrics_collector.clone()).await?;

    // Initialize monitoring system
    let monitoring_system =
        monitoring::MonitoringSystem::new(monitoring::MonitoringConfig::default());

    // Initialize self-healing system
    let self_healing_system =
        self_healing::SelfHealingManager::new(self_healing::SelfHealingConfig::default());

    // Initialize shutdown manager
    let shutdown_manager = shutdown::ShutdownManager::new();

    // Create comprehensive system
    let system = SquirrelSystem {
        ecosystem_manager,
        monitoring_system,
        self_healing_system,
        shutdown_manager,
        #[cfg(feature = "monitoring")]
        metrics_collector,
    };

    Ok(system)
}

/// Comprehensive Squirrel system with all integrated components
pub struct SquirrelSystem {
    /// Manages ecosystem registration and coordination.
    pub ecosystem_manager: EcosystemManager,
    /// Tracks system health and performance metrics.
    pub monitoring_system: monitoring::MonitoringSystem,
    /// Handles automatic recovery from component failures.
    pub self_healing_system: SelfHealingManager,
    /// Coordinates graceful shutdown across components.
    pub shutdown_manager: ShutdownManager,
    #[cfg(feature = "monitoring")]
    pub metrics_collector: std::sync::Arc<MetricsCollector>,
}

impl SquirrelSystem {
    /// Register with ecosystem
    pub async fn register_with_ecosystem(
        &self,
        provider: &SquirrelPrimalProvider,
    ) -> Result<(), crate::error::PrimalError> {
        self.ecosystem_manager
            .register_squirrel_service(provider)
            .await
    }

    /// Start ecosystem coordination by capabilities (NEW - Capability-Based)
    ///
    /// Initiates coordination requiring specific capabilities.
    ///
    /// # Arguments
    /// * `required_capabilities` - List of capabilities needed
    ///
    /// # Example
    /// ```ignore
    /// let session_id = squirrel
    ///     .start_coordination_by_capabilities(vec!["service_mesh", "security.auth"])
    ///     .await?;
    /// ```
    pub async fn start_coordination_by_capabilities(
        &self,
        required_capabilities: Vec<&str>,
    ) -> Result<String, crate::error::PrimalError> {
        let context = std::collections::HashMap::new();
        self.ecosystem_manager
            .start_coordination_by_capabilities(required_capabilities, context)
            .await
    }

    /// Get comprehensive system status
    pub async fn get_system_status(&self) -> SquirrelSystemStatus {
        let ecosystem_status = self.ecosystem_manager.get_ecosystem_status().await;
        let monitoring_status = self.monitoring_system.get_system_status().await;
        let self_healing_status = self.self_healing_system.get_health_status().await;
        let shutdown_requested = self.shutdown_manager.is_shutdown_requested();
        let mut status_info = serde_json::Map::new();
        status_info["shutdown_requested"] = serde_json::json!(shutdown_requested);

        // Calculate health scores from available data
        let monitoring_health_score = match monitoring_status.health {
            monitoring::HealthState::Healthy => 100.0,
            monitoring::HealthState::Warning => 60.0,
            monitoring::HealthState::Critical => 20.0,
            monitoring::HealthState::Unknown => 50.0,
        };

        let self_healing_health_score = {
            let healthy_components = self_healing_status
                .values()
                .filter(|c| matches!(c.status, self_healing::HealthStatus::Healthy))
                .count();
            let total_components = self_healing_status.len().max(1);
            (healthy_components as f64 / total_components as f64) * 100.0
        };

        SquirrelSystemStatus {
            ecosystem_status: ecosystem_status.clone(),
            monitoring_status,
            self_healing_status,
            shutdown_requested: self.shutdown_manager.is_shutdown_requested(),
            overall_health: (ecosystem_status.overall_health
                + monitoring_health_score
                + self_healing_health_score)
                / 3.0,
        }
    }

    /// Graceful shutdown of the entire system
    pub async fn shutdown(&self) -> Result<(), crate::error::PrimalError> {
        tracing::info!("Shutting down Squirrel system");

        // Shutdown in reverse order of initialization
        self.ecosystem_manager.shutdown().await?;
        self.monitoring_system.stop().await?;
        // Note: self_healing_system and shutdown_manager don't have shutdown methods
        // They are designed to be stopped by dropping the system

        tracing::info!("Squirrel system shutdown complete");
        Ok(())
    }
}

/// Comprehensive system status
#[derive(Debug, Clone)]
pub struct SquirrelSystemStatus {
    /// Current ecosystem integration state.
    pub ecosystem_status: ecosystem::EcosystemIntegrationStatus,
    /// Monitoring system health and metrics.
    pub monitoring_status: monitoring::SystemStatus,
    /// Per-component health from the self-healing system.
    pub self_healing_status: std::collections::HashMap<String, self_healing::ComponentHealth>,
    /// Whether a shutdown has been requested.
    pub shutdown_requested: bool,
    /// Aggregated health score (0–100).
    pub overall_health: f64,
}

/// Create a default Squirrel system for testing and development
pub async fn create_default_squirrel_system() -> Result<SquirrelSystem, crate::error::PrimalError> {
    let config = EcosystemConfig::default();
    initialize_squirrel_system(config).await
}
