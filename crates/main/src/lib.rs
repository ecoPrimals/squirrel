//! Squirrel Universal AI Primal
//!
//! A universal AI coordination primal that implements the standardized ecosystem
//! patterns for dynamic primal evolution and integration with the ecoPrimals ecosystem.
//!
//! This primal follows the universal adapter patterns defined by Songbird and
//! implements the EcosystemServiceRegistration standard for seamless integration.

#![deny(unsafe_code)]
#![warn(clippy::all)]
#![warn(rust_2018_idioms)]

// Core modules
pub mod api;
pub mod beardog;
pub mod biomeos_integration;
pub mod capability;
pub mod chaos;
pub mod compute_client;
pub mod config;
pub mod ecosystem;
pub mod error;
pub mod error_handling;
pub mod hardware;
pub mod monitoring;
pub mod observability;
pub mod optimization;
pub mod primal_provider;
pub mod protocol;
pub mod resource_manager;
pub mod security;
pub mod security_client;
pub mod session;
pub mod shutdown;
pub mod songbird;
pub mod storage_client;
pub mod toadstool;
pub mod universal;
pub mod universal_adapter;
pub mod universal_primal_ecosystem;
pub mod universal_provider;

/// Universal adapters for capability-based primal integration
pub mod universal_adapters;

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
    initialize_ecosystem_integration, ComponentHealth, EcosystemPrimalType, EcosystemRegistryEvent,
    EcosystemRegistryManager, EcosystemServiceRegistration, EcosystemStatus,
};
pub use error_handling::prelude::*;
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
pub type UniversalResult<T> = Result<T, EcosystemError>;
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
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
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
        metrics_collector,
    };

    Ok(system)
}

/// Comprehensive Squirrel system with all integrated components
pub struct SquirrelSystem {
    pub ecosystem_manager: EcosystemManager,
    pub monitoring_system: monitoring::MonitoringSystem,
    pub self_healing_system: SelfHealingManager,
    pub shutdown_manager: ShutdownManager,
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

    /// Start ecosystem coordination
    pub async fn start_coordination(
        &self,
        participants: Vec<EcosystemPrimalType>,
    ) -> Result<String, crate::error::PrimalError> {
        let context = std::collections::HashMap::new();
        self.ecosystem_manager
            .start_coordination(participants, context)
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
    pub ecosystem_status: ecosystem::EcosystemStatus,
    pub monitoring_status: monitoring::SystemStatus,
    pub self_healing_status: std::collections::HashMap<String, self_healing::ComponentHealth>,
    pub shutdown_requested: bool,
    pub overall_health: f64,
}

/// Create a default Squirrel system for testing and development
pub async fn create_default_squirrel_system() -> Result<SquirrelSystem, crate::error::PrimalError> {
    let config = EcosystemConfig::default();
    initialize_squirrel_system(config).await
}
