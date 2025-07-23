//! Core configuration modules for Squirrel MCP
//!
//! This module provides the fundamental configuration structures and utilities
//! used throughout the application for managing settings, defaults, and environment-specific
//! configurations.

pub mod ai;
pub mod defaults;
pub mod ecosystem;
pub mod manager;
pub mod network;
pub mod observability;
pub mod ports;
pub mod security;
pub mod service_endpoints;
pub mod types; // New port management configuration

// Re-export commonly used types
pub use ai::AIConfig;
pub use defaults::ConfigDefaults;
pub use ecosystem::EcosystemConfig;
pub use manager::{ConfigManager, DefaultConfigManager};
pub use network::{
    defaults as network_defaults, DevelopmentConfig, NetworkConfig, ServiceEndpoints,
};
pub use observability::ObservabilityConfig;
pub use ports::{
    DevelopmentPortConfig, EcosystemPortConfig, HealthCheckConfig, LoadBalancingConfig,
    PortAllocationSettings, PortAllocationStrategy, PortConfig, PrimalsPortConfig,
    ProductionPortConfig, ServiceProtocol, SquirrelPorts,
};
pub use security::SecurityConfig;
pub use service_endpoints::{get_service_endpoints, GlobalServiceEndpoints};
pub use types::{
    AIServiceConfig, AppConfig, BiomeOSEndpoints, Config, DatabaseConfig,
    ExtendedObservabilityConfig, ExternalServiceConfig,
};

// Network configuration alias for backward compatibility
pub use network::NetworkConfig as NetworkEndpointConfig;
