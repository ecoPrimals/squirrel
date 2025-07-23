//! Universal Configuration System
//!
//! This module provides a universal configuration system that replaces hardcoded
//! primal endpoints with dynamic service discovery and configuration.
//!
//! ## Features
//!
//! - Environment variable-based configuration
//! - Dynamic service discovery configuration
//! - Service mesh integration
//! - Health check configuration
//! - Comprehensive validation
//! - Builder pattern for easy configuration
//!
//! ## Usage
//!
//! ```rust
//! use squirrel_mcp_config::universal::{UniversalServiceConfig, ServiceConfigBuilder, UniversalConfigBuilder, FromEnv};
//! use std::time::Duration;
//!
//! // Load from environment variables
//! let config = UniversalServiceConfig::from_env();
//!
//! // Or build programmatically
//! let config = UniversalConfigBuilder::new()
//!     .add_discovery_endpoint("http://localhost:8500".to_string())
//!     .unwrap()
//!     .add_service(
//!         "ai-service".to_string(),
//!         ServiceConfigBuilder::new()
//!             .add_endpoint("http://localhost:8080".to_string())
//!             .unwrap()
//!             .add_capability("chat".to_string())
//!             .build()
//!             .unwrap()
//!     )
//!     .unwrap()
//!     .build();
//! ```

mod builder;
pub mod environment;
mod types;
mod utils;
mod validation;

// Re-export public types
pub use builder::{ServiceConfigBuilder, UniversalConfigBuilder};
pub use types::*;
pub use utils::{parse_duration, validate_url};

// Re-export validation functionality
pub use validation::ValidationExt;

// Re-export environment functionality
pub use environment::FromEnv;

/// Universal Network Configuration Helper
///
/// This module provides utilities to replace hardcoded network addresses
/// with configurable values following the agnostic capability-based approach.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use super::core::service_endpoints::get_service_endpoints;

/// Universal network endpoint configuration
/// This replaces hardcoded localhost, 127.0.0.1, and port constants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalNetworkConfig {
    /// Base host for services (configurable)
    pub host: String,

    /// Service discovery endpoints
    pub service_endpoints: HashMap<String, ServiceEndpoint>,

    /// Environment-specific overrides
    pub environment_overrides: HashMap<String, String>,

    /// Default ports by service capability
    pub default_ports: HashMap<ServiceCapability, u16>,
}

/// Universal service endpoint that avoids hardcoding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    /// Service name (capability-based, not hardcoded to specific names)
    pub capability: ServiceCapability,

    /// Host (from environment or discovery)
    pub host: String,

    /// Port (from environment or service allocation)
    pub port: u16,

    /// Protocol scheme
    pub scheme: String,

    /// Path prefix (optional)
    pub path: Option<String>,

    /// Health check endpoint
    pub health_path: Option<String>,
}

/// Service capabilities instead of hardcoded names
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum ServiceCapability {
    /// Service mesh orchestration (was: Songbird)
    ServiceMesh,

    /// Security services (was: BearDog)
    Security,

    /// Compute services (was: ToadStool)
    Compute,

    /// Storage services (was: NestGate)
    Storage,

    /// AI coordination services (was: Squirrel)
    AICoordination,

    /// Operating system services (was: BiomeOS)
    OperatingSystem,

    /// Generic HTTP API
    HttpApi,

    /// WebSocket services
    WebSocket,

    /// Metrics and monitoring
    Monitoring,

    /// Health checks
    Health,

    /// Admin interfaces
    Admin,
}

impl Default for UniversalNetworkConfig {
    fn default() -> Self {
        let is_production = env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .eq_ignore_ascii_case("production");

        let default_host = if is_production {
            // In production, use service discovery or explicit config
            env::var("SERVICE_HOST").unwrap_or_else(|_| "0.0.0.0".to_string())
        } else {
            // In development, use host from centralized endpoints
            env::var("DEV_HOST").unwrap_or_else(|_| {
                get_service_endpoints().mcp_url()
                    .ok()
                    .and_then(|url| url.host_str().map(|h| h.to_string()))
                    .unwrap_or_else(|| "localhost".to_string())
            })
        };

        // Default port mappings (still configurable via environment)
        let mut default_ports = HashMap::new();
        default_ports.insert(ServiceCapability::ServiceMesh, 8080);
        default_ports.insert(ServiceCapability::Security, 8443);
        default_ports.insert(ServiceCapability::Compute, 8445);
        default_ports.insert(ServiceCapability::Storage, 8444);
        default_ports.insert(ServiceCapability::AICoordination, 8080);
        default_ports.insert(ServiceCapability::OperatingSystem, 5000);
        default_ports.insert(ServiceCapability::HttpApi, 8080);
        default_ports.insert(ServiceCapability::WebSocket, 8081);
        default_ports.insert(ServiceCapability::Monitoring, 9090);
        default_ports.insert(ServiceCapability::Health, 8082);
        default_ports.insert(ServiceCapability::Admin, 8083);

        Self {
            host: default_host,
            service_endpoints: HashMap::new(),
            environment_overrides: HashMap::new(),
            default_ports,
        }
    }
}

impl UniversalNetworkConfig {
    /// Create a new network configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Load service-specific endpoints from environment
        config.load_service_endpoints_from_env();

        // Load environment overrides
        config.load_environment_overrides();

        config
    }

    /// Load service endpoints from environment variables
    fn load_service_endpoints_from_env(&mut self) {
        // Service mesh (was Songbird)
        if let Ok(endpoint) = env::var("SERVICE_MESH_ENDPOINT") {
            if let Ok(parsed) = self.parse_endpoint(endpoint, ServiceCapability::ServiceMesh) {
                self.service_endpoints
                    .insert("service_mesh".to_string(), parsed);
            }
        }

        // Security service (was BearDog)
        if let Ok(endpoint) = env::var("SECURITY_SERVICE_ENDPOINT") {
            if let Ok(parsed) = self.parse_endpoint(endpoint, ServiceCapability::Security) {
                self.service_endpoints
                    .insert("security".to_string(), parsed);
            }
        }

        // Compute service (was ToadStool)
        if let Ok(endpoint) = env::var("COMPUTE_SERVICE_ENDPOINT") {
            if let Ok(parsed) = self.parse_endpoint(endpoint, ServiceCapability::Compute) {
                self.service_endpoints.insert("compute".to_string(), parsed);
            }
        }

        // Storage service (was NestGate)
        if let Ok(endpoint) = env::var("STORAGE_SERVICE_ENDPOINT") {
            if let Ok(parsed) = self.parse_endpoint(endpoint, ServiceCapability::Storage) {
                self.service_endpoints.insert("storage".to_string(), parsed);
            }
        }

        // AI coordination service (was Squirrel)
        if let Ok(endpoint) = env::var("AI_COORDINATION_ENDPOINT") {
            if let Ok(parsed) = self.parse_endpoint(endpoint, ServiceCapability::AICoordination) {
                self.service_endpoints
                    .insert("ai_coordination".to_string(), parsed);
            }
        }

        // Operating system service (was BiomeOS)
        if let Ok(endpoint) = env::var("OS_SERVICE_ENDPOINT") {
            if let Ok(parsed) = self.parse_endpoint(endpoint, ServiceCapability::OperatingSystem) {
                self.service_endpoints
                    .insert("operating_system".to_string(), parsed);
            }
        }
    }

    /// Load environment-specific overrides
    fn load_environment_overrides(&mut self) {
        if let Ok(overrides) = env::var("NETWORK_OVERRIDES") {
            for override_pair in overrides.split(',') {
                if let Some((key, value)) = override_pair.split_once('=') {
                    self.environment_overrides
                        .insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }
    }

    /// Parse endpoint URL into ServiceEndpoint
    fn parse_endpoint(
        &self,
        endpoint: String,
        capability: ServiceCapability,
    ) -> Result<ServiceEndpoint, String> {
        let url = url::Url::parse(&endpoint).map_err(|e| format!("Invalid URL: {}", e))?;

        let host = url.host_str()
            .map(|h| h.to_string())
            .unwrap_or_else(|| {
                // Use centralized service endpoint host as fallback
                get_service_endpoints()
                    .mcp_url()
                    .ok()
                    .and_then(|url| url.host_str().map(|h| h.to_string()))
                    .unwrap_or_else(|| "localhost".to_string())
            });
        let port = url
            .port()
            .unwrap_or(*self.default_ports.get(&capability).unwrap_or(&8080));
        let scheme = url.scheme().to_string();
        let path = if url.path() != "/" && !url.path().is_empty() {
            Some(url.path().to_string())
        } else {
            None
        };

        Ok(ServiceEndpoint {
            capability,
            host,
            port,
            scheme,
            path,
            health_path: Some("/health".to_string()),
        })
    }

    /// Get endpoint for a service capability (instead of hardcoded names)
    pub fn get_endpoint_for_capability(&self, capability: ServiceCapability) -> String {
        // First check for configured service endpoints
        for endpoint in self.service_endpoints.values() {
            if endpoint.capability == capability {
                return self.build_url(endpoint);
            }
        }

        // Fall back to default configuration
        let port = *self.default_ports.get(&capability).unwrap_or(&8080);
        let scheme = match capability {
            ServiceCapability::Security => "https",
            ServiceCapability::WebSocket => "ws",
            _ => "http",
        };

        format!("{}://{}:{}", scheme, self.host, port)
    }

    /// Build URL from service endpoint
    fn build_url(&self, endpoint: &ServiceEndpoint) -> String {
        let mut url = format!("{}://{}:{}", endpoint.scheme, endpoint.host, endpoint.port);
        if let Some(path) = &endpoint.path {
            url.push_str(path);
        }
        url
    }

    /// Get health check URL for a capability
    pub fn get_health_url_for_capability(&self, capability: ServiceCapability) -> String {
        let base_url = self.get_endpoint_for_capability(capability);
        format!("{}/health", base_url)
    }

    /// Get metrics URL for a capability  
    pub fn get_metrics_url_for_capability(&self, capability: ServiceCapability) -> String {
        let base_url = self.get_endpoint_for_capability(capability);
        format!("{}/metrics", base_url)
    }

    /// Get WebSocket URL for a capability
    pub fn get_websocket_url_for_capability(&self, capability: ServiceCapability) -> String {
        let endpoint = self.get_endpoint_for_capability(capability);
        // Convert http/https to ws/wss
        let ws_endpoint = endpoint
            .replace("http://", "ws://")
            .replace("https://", "wss://");
        format!("{}/ws", ws_endpoint)
    }

    /// Register a discovered service endpoint
    pub fn register_discovered_endpoint(&mut self, service_id: String, endpoint: ServiceEndpoint) {
        self.service_endpoints.insert(service_id, endpoint);
    }

    /// Get all configured endpoints
    pub fn get_all_endpoints(&self) -> &HashMap<String, ServiceEndpoint> {
        &self.service_endpoints
    }
}

/// Helper functions to replace hardcoded values
pub struct NetworkConfigHelper;

impl NetworkConfigHelper {
    /// Replace localhost:8080 with configurable endpoint for service mesh
    pub fn get_service_mesh_endpoint() -> String {
        env::var("SERVICE_MESH_ENDPOINT").unwrap_or_else(|_| {
            let config = UniversalNetworkConfig::from_env();
            config.get_endpoint_for_capability(ServiceCapability::ServiceMesh)
        })
    }

    /// Replace localhost:8443 with configurable endpoint for security service
    pub fn get_security_endpoint() -> String {
        env::var("SECURITY_SERVICE_ENDPOINT").unwrap_or_else(|_| {
            let config = UniversalNetworkConfig::from_env();
            config.get_endpoint_for_capability(ServiceCapability::Security)
        })
    }

    /// Replace localhost:8444 with configurable endpoint for storage service
    pub fn get_storage_endpoint() -> String {
        env::var("STORAGE_SERVICE_ENDPOINT").unwrap_or_else(|_| {
            let config = UniversalNetworkConfig::from_env();
            config.get_endpoint_for_capability(ServiceCapability::Storage)
        })
    }

    /// Replace localhost:8445 with configurable endpoint for compute service
    pub fn get_compute_endpoint() -> String {
        env::var("COMPUTE_SERVICE_ENDPOINT").unwrap_or_else(|_| {
            let config = UniversalNetworkConfig::from_env();
            config.get_endpoint_for_capability(ServiceCapability::Compute)
        })
    }

    /// Replace localhost:5000 with configurable endpoint for OS service
    pub fn get_os_endpoint() -> String {
        env::var("OS_SERVICE_ENDPOINT").unwrap_or_else(|_| {
            let config = UniversalNetworkConfig::from_env();
            config.get_endpoint_for_capability(ServiceCapability::OperatingSystem)
        })
    }

    /// Get health endpoint for any service capability
    pub fn get_health_endpoint_for(capability: ServiceCapability) -> String {
        let config = UniversalNetworkConfig::from_env();
        config.get_health_url_for_capability(capability)
    }

    /// Get metrics endpoint for any service capability
    pub fn get_metrics_endpoint_for(capability: ServiceCapability) -> String {
        let config = UniversalNetworkConfig::from_env();
        config.get_metrics_url_for_capability(capability)
    }

    /// Get WebSocket endpoint for any service capability
    pub fn get_websocket_endpoint_for(capability: ServiceCapability) -> String {
        let config = UniversalNetworkConfig::from_env();
        config.get_websocket_url_for_capability(capability)
    }
}

pub mod env_vars {
    //! Environment variable documentation
    //!
    //! Environment variables for network configuration
    //!
    //! ## Service Endpoints
    //! - `SERVICE_MESH_ENDPOINT`: Service mesh endpoint (default: http://localhost:8080)
    //! - `SECURITY_SERVICE_ENDPOINT`: Security service endpoint (default: https://localhost:8443)
    //! - `COMPUTE_SERVICE_ENDPOINT`: Compute service endpoint (default: http://localhost:8445)
    //! - `STORAGE_SERVICE_ENDPOINT`: Storage service endpoint (default: http://localhost:8444)
    //! - `AI_COORDINATION_ENDPOINT`: AI coordination endpoint (default: http://localhost:8080)
    //! - `OS_SERVICE_ENDPOINT`: OS service endpoint (default: http://localhost:5000)
    //!
    //! ## Environment Configuration
    //! - `ENVIRONMENT`: deployment environment (development/production)
    //! - `SERVICE_HOST`: default host for all services (default: localhost in dev, 0.0.0.0 in prod)
    //! - `DEV_HOST`: development host override
    //!
    //! ## Network Overrides
    //! - `NETWORK_OVERRIDES`: comma-separated key=value pairs for endpoint overrides
}
