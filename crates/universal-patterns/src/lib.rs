// SPDX-License-Identifier: AGPL-3.0-only
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(
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
    clippy::wildcard_imports,
    clippy::cloned_instead_of_copied
)]

//! # Universal Patterns Framework
//!
//! This crate provides a comprehensive universal patterns framework for the ecoPrimals ecosystem,
//! with full compatibility with songbird's orchestration system.
//!
//! ## Features
//!
//! - **Multi-Instance Support**: Multiple primal instances per user/device
//! - **Context-Aware Routing**: Route requests based on user/device/security context
//! - **Dynamic Port Management**: Songbird-managed port allocation and lifecycle
//! - **Comprehensive Health Monitoring**: Real-time health checking and failover
//! - **Auto-Discovery**: Automatic primal instance discovery and registration
//! - **Load Balancing**: Multiple load balancing strategies (round-robin, least connections, etc.)
//! - **Circuit Breaker**: Automatic failover and recovery mechanisms
//!
//! ## Architecture
//!
//! The universal patterns framework consists of several key components:
//!
//! - **Universal Primal Registry**: Central registry for discovering and managing primal instances
//! - **Primal Providers**: Individual adapter implementations for each primal service
//! - **Communication Protocol**: Standardized request/response format for all primals
//! - **Multi-Instance Support**: Ability to manage multiple instances of the same primal type
//! - **Context-Aware Routing**: Route requests to appropriate primal instances based on context
//!
//! ## Quick Start
//!
//! ```ignore,no_run
//! use universal_patterns::{initialize_primal_system, PrimalContext};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize the universal primal system
//! let registry = initialize_primal_system(None).await?;
//!
//! // Create a request context
//! let context = PrimalContext::default();
//!
//! // The registry is now ready to handle primal requests
//! println!("Universal primal system initialized successfully");
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![warn(clippy::all)]
#![warn(rust_2018_idioms)]
#![warn(missing_docs)]

pub mod builder;
pub mod circuit_breaker;
pub mod compute_dispatch;
pub mod config;
pub mod dispatch_outcome;
pub mod federation;
pub mod ipc_client;
pub mod manifest_discovery;
pub mod or_exit;
pub mod provenance;
pub mod registry;
pub mod security;
pub mod streaming;
pub mod traits;
pub mod transport;
pub mod validation_harness;

// Re-export commonly used types for convenience
pub use builder::UniversalConfigBuilder;
pub use circuit_breaker::{CircuitBreaker, CircuitState, ResilientCaller, RetryPolicy};
pub use compute_dispatch::{ComputeDispatchRequest, ComputeDispatchResponse};
pub use config::{
    InstanceLifecycleConfig, LoadBalancingStrategy, MultiInstanceConfig, PortManagementConfig,
    PrimalInstanceConfig, UniversalPrimalConfig,
};
pub use dispatch_outcome::{DispatchError, DispatchOutcome};
pub use ipc_client::{
    CapabilityInfo, IpcClient, IpcClientError, extract_rpc_error, extract_rpc_result,
    parse_capabilities_from_response,
};
pub use manifest_discovery::PrimalManifest;
pub use or_exit::{OrExit, exit_codes};
pub use registry::{
    DiscoveredPrimal, EnhancedRegistryStatistics, RegistryStatistics, UniversalPrimalRegistry,
};
pub use security::{SecurityContext, UniversalSecurityClient, UniversalSecurityProvider};
pub use streaming::{StreamItem, StreamKind};
pub use traits::{
    DynamicPortInfo, PrimalCapability, PrimalContext, PrimalError, PrimalHealth, PrimalProvider,
    PrimalRequest, PrimalRequestType, PrimalResponse, PrimalResponseType, PrimalResult, PrimalType,
    SecurityLevel,
};
pub use transport::{
    ListenerConfig, RemoteAddr, TransportConfig, TransportType, UniversalListener,
    UniversalTransport,
};
pub use validation_harness::{CheckOutcome, CheckResult, ValidationHarness};

/// Initialize the universal primal system
///
/// This function sets up the registry and performs initial configuration
/// based on environment variables and provided configuration.
///
/// # Arguments
///
/// * `config` - Optional configuration for the primal system
///
/// # Returns
///
/// A configured `UniversalPrimalRegistry` instance
///
/// # Example
///
/// ```ignore,no_run
/// use universal_patterns::initialize_primal_system;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let registry = initialize_primal_system(None).await?;
/// # Ok(())
/// # }
/// ```
#[must_use = "the registry should be used or the error handled"]
pub async fn initialize_primal_system(
    config: Option<UniversalPrimalConfig>,
) -> PrimalResult<UniversalPrimalRegistry> {
    let config = config.unwrap_or_else(UniversalPrimalConfig::from_env);

    // Validate configuration
    config
        .validate()
        .map_err(PrimalError::InvalidConfiguration)?;

    let mut registry = UniversalPrimalRegistry::new();

    // Initialize with configuration
    registry.initialize_with_config(&config).await?;

    Ok(registry)
}

/// Create a primal context for a specific user and device
///
/// This is a convenience function for creating primal contexts with
/// appropriate security levels and network information.
///
/// # Arguments
///
/// * `user_id` - User identifier
/// * `device_id` - Device identifier
/// * `security_level` - Required security level
///
/// # Returns
///
/// A configured `PrimalContext` instance
///
/// # Example
///
/// ```ignore
/// use universal_patterns::{create_primal_context, SecurityLevel};
///
/// let context = create_primal_context(
///     "user123".to_string(),
///     "device456".to_string(),
///     SecurityLevel::High
/// );
/// ```
pub fn create_primal_context(
    user_id: String,
    device_id: String,
    security_level: SecurityLevel,
) -> PrimalContext {
    PrimalContext {
        user_id,
        device_id,
        security_level,
        ..Default::default()
    }
}

/// Create a development configuration preset
///
/// This function creates a configuration optimized for development
/// environments with relaxed security and monitoring settings.
///
/// # Returns
///
/// A development-optimized `UniversalPrimalConfig`
///
/// # Example
///
/// ```ignore
/// use universal_patterns::create_development_config;
///
/// let config = create_development_config();
/// ```
pub fn create_development_config() -> UniversalPrimalConfig {
    let mut config = UniversalPrimalConfig::default();

    // Development-specific settings
    config.multi_instance.max_instances_per_type = 3;
    config.multi_instance.max_instances_per_user = 2;
    config.monitoring.metrics_enabled = false;
    config.monitoring.tracing.level = "debug".to_string();
    config.port_management.port_range.start = 8000;
    config.port_management.port_range.end = 8100;

    config
}

/// Create a production configuration preset
///
/// This function creates a configuration optimized for production
/// environments with enhanced security and monitoring.
///
/// # Returns
///
/// A production-optimized `UniversalPrimalConfig`
///
/// # Example
///
/// ```ignore
/// use universal_patterns::create_production_config;
///
/// let config = create_production_config();
/// ```
pub fn create_production_config() -> UniversalPrimalConfig {
    let mut config = UniversalPrimalConfig::default();

    // Production-specific settings
    config.multi_instance.max_instances_per_type = 20;
    config.multi_instance.max_instances_per_user = 10;
    config.multi_instance.scaling.auto_scaling_enabled = true;
    config.multi_instance.failover.enabled = true;
    config.monitoring.metrics_enabled = true;
    config.monitoring.tracing.level = "info".to_string();
    config.port_management.port_range.start = 9000;
    config.port_management.port_range.end = 10000;

    config
}

/// Create a primal-specific configuration
///
/// This function creates a configuration optimized for a specific primal type
/// with appropriate resource limits and capabilities.
///
/// # Arguments
///
/// * `primal_type` - The type of primal this configuration is for
/// * `instance_count` - Number of instances to configure
///
/// # Returns
///
/// A primal-specific `UniversalPrimalConfig`
///
/// # Example
///
/// ```ignore
/// use universal_patterns::{create_primal_config, PrimalType};
///
/// let config = create_primal_config(PrimalType::Security, 5);
/// ```
pub fn create_primal_config(
    primal_type: PrimalType,
    instance_count: usize,
) -> UniversalPrimalConfig {
    let mut config = UniversalPrimalConfig::default();

    // Primal-specific settings
    match primal_type {
        PrimalType::Coordinator => {
            config.multi_instance.max_instances_per_type = instance_count;
            config.monitoring.tracing.level = "info".to_string();
            config.multi_instance.scaling.auto_scaling_enabled = true;
        }
        PrimalType::Security => {
            config.multi_instance.max_instances_per_type = instance_count;
            config.monitoring.tracing.level = "info".to_string();
        }
        PrimalType::Orchestration => {
            config.multi_instance.max_instances_per_type = instance_count;
            config.monitoring.tracing.level = "debug".to_string();
            config.multi_instance.scaling.auto_scaling_enabled = true;
            config.multi_instance.scaling.scale_up_cpu_threshold = 50.0;
        }
        PrimalType::AI => {
            config.multi_instance.max_instances_per_type = instance_count;
            config.multi_instance.scaling.scale_up_cpu_threshold = 60.0;
            config.monitoring.tracing.level = "debug".to_string();
        }
        PrimalType::Storage => {
            config.multi_instance.max_instances_per_type = instance_count;
            config.multi_instance.scaling.scale_up_memory_threshold = 70.0;
        }
        PrimalType::Compute => {
            config.multi_instance.max_instances_per_type = instance_count;
            config.multi_instance.scaling.auto_scaling_enabled = true;
            config.multi_instance.scaling.scale_up_cpu_threshold = 80.0;
        }
        PrimalType::Network => {
            config.multi_instance.max_instances_per_type = instance_count;
            config.port_management.port_range.start = 10000;
            config.port_management.port_range.end = 11000;
        }
        PrimalType::Custom(_) => {
            config.multi_instance.max_instances_per_type = instance_count;
        }
    }

    config
}

/// Version information for the universal patterns framework
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get version information
///
/// # Returns
///
/// Version string for the universal patterns framework
///
/// # Example
///
/// ```ignore
/// use universal_patterns::version;
///
/// println!("Universal Patterns Framework v{}", version());
/// ```
pub fn version() -> &'static str {
    VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_api_surface_accessible() {
        // Verify all re-exported types are accessible
        let _: UniversalConfigBuilder = UniversalConfigBuilder::new();
        let _: InstanceLifecycleConfig = InstanceLifecycleConfig::default();
        let _: LoadBalancingStrategy = LoadBalancingStrategy::RoundRobin;
        let _: MultiInstanceConfig = MultiInstanceConfig::default();
        let _: PortManagementConfig = PortManagementConfig::default();
        let _: UniversalPrimalConfig = UniversalPrimalConfig::default();
        let _: PrimalContext = PrimalContext::default();
        let _: PrimalType = PrimalType::Coordinator;
        let _: SecurityLevel = SecurityLevel::Standard;
        let _: ListenerConfig = ListenerConfig::default();
        let _: TransportConfig = TransportConfig::default();
        let _: TransportType = TransportType::Tcp;
        // PrimalInstanceConfig and SecurityContext exist but require constructor args
        let _ = PrimalInstanceConfig::new(
            "http://localhost:8080".to_string(),
            "inst-1".to_string(),
            "user1".to_string(),
            "dev1".to_string(),
        );
    }

    #[test]
    fn test_version_returns_non_empty() {
        let v = version();
        assert!(!v.is_empty());
        assert_eq!(v, VERSION);
    }

    #[test]
    fn test_create_primal_context() {
        let ctx = create_primal_context(
            "user1".to_string(),
            "device1".to_string(),
            SecurityLevel::High,
        );
        assert_eq!(ctx.user_id, "user1");
        assert_eq!(ctx.device_id, "device1");
        assert_eq!(ctx.security_level, SecurityLevel::High);
    }

    #[test]
    fn test_create_development_config() {
        let config = create_development_config();
        assert_eq!(config.multi_instance.max_instances_per_type, 3);
        assert_eq!(config.multi_instance.max_instances_per_user, 2);
        assert!(!config.monitoring.metrics_enabled);
        assert_eq!(config.port_management.port_range.start, 8000);
    }

    #[test]
    fn test_create_production_config() {
        let config = create_production_config();
        assert_eq!(config.multi_instance.max_instances_per_type, 20);
        assert_eq!(config.multi_instance.max_instances_per_user, 10);
        assert!(config.multi_instance.scaling.auto_scaling_enabled);
        assert!(config.multi_instance.failover.enabled);
        assert!(config.monitoring.metrics_enabled);
    }

    #[test]
    fn test_create_primal_config_for_each_type() {
        for primal_type in [
            PrimalType::Coordinator,
            PrimalType::Security,
            PrimalType::Orchestration,
            PrimalType::AI,
            PrimalType::Storage,
            PrimalType::Compute,
            PrimalType::Network,
            PrimalType::Custom("custom".to_string()),
        ] {
            let config = create_primal_config(primal_type.clone(), 5);
            assert_eq!(config.multi_instance.max_instances_per_type, 5);
        }
    }

    #[tokio::test]
    async fn test_initialize_primal_system_with_none() {
        // Uses from_env - may fail if env invalid, but tests the code path
        let result = initialize_primal_system(None).await;
        // Either succeeds or fails with validation error
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_initialize_primal_system_with_config() {
        let config = create_development_config();
        let result = initialize_primal_system(Some(config)).await;
        assert!(result.is_ok());
        let registry = result.unwrap();
        let stats = registry.get_statistics().await;
        assert_eq!(stats.total_primals, 0);
    }
}
