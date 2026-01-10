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

#![deny(unsafe_code)] // ✅ ENFORCED: No unsafe code allowed
#![warn(clippy::all)]
#![warn(rust_2018_idioms)]
//! - **Primal Providers**: Individual adapter implementations for each primal service
//! - **Communication Protocol**: Standardized request/response format for all primals
//! - **Multi-Instance Support**: Ability to manage multiple instances of the same primal type
//! - **Context-Aware Routing**: Route requests to appropriate primal instances based on context
//!
//! ## Quick Start
//!
//! ```rust,no_run
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

#![deny(unsafe_code)]
#![warn(clippy::all)]
#![warn(rust_2018_idioms)]
#![warn(missing_docs)]

pub mod builder;
pub mod config;
pub mod federation;
pub mod registry;
pub mod security;
pub mod traits;

// Re-export commonly used types for convenience
pub use builder::UniversalConfigBuilder;
pub use config::{
    InstanceLifecycleConfig, LoadBalancingStrategy, MultiInstanceConfig, PortManagementConfig,
    PrimalInstanceConfig, UniversalPrimalConfig,
};
pub use registry::{
    DiscoveredPrimal, EnhancedRegistryStatistics, RegistryStatistics, UniversalPrimalRegistry,
};
pub use security::{SecurityContext, UniversalSecurityClient, UniversalSecurityProvider};
pub use traits::{
    DynamicPortInfo, PrimalCapability, PrimalContext, PrimalError, PrimalHealth, PrimalProvider,
    PrimalRequest, PrimalRequestType, PrimalResponse, PrimalResponseType, PrimalResult, PrimalType,
    SecurityLevel,
};

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
/// ```rust,no_run
/// use universal_patterns::initialize_primal_system;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let registry = initialize_primal_system(None).await?;
/// # Ok(())
/// # }
/// ```
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
/// ```rust
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
/// ```rust
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
/// ```rust
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
/// ```rust
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
/// ```rust
/// use universal_patterns::version;
///
/// println!("Universal Patterns Framework v{}", version());
/// ```
pub fn version() -> &'static str {
    VERSION
}
