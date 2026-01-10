//! Service Discovery Client - Capability-Based Discovery
//!
//! This module provides a unified interface for discovering services in the ecosystem
//! without hardcoding primal names, endpoints, or capabilities.
//!
//! ## Evolution from Hardcoded to Dynamic
//!
//! **Old Pattern (Hardcoded)**:
//! ```ignore
//! let songbird_url = "http://localhost:3001";
//! let response = client.get(songbird_url).send().await?;
//! ```
//!
//! **New Pattern (Capability-Based)**:
//! ```ignore
//! let discovery = ServiceDiscoveryClient::new();
//! let coordinator = discovery
//!     .discover_by_capability(&Capability::Coordination)
//!     .await?;
//! let response = client.get(&coordinator.endpoint).send().await?;
//! ```
//!
//! ## Multi-Level Discovery
//!
//! 1. **Environment Variables** (highest priority)
//!    - `{SERVICE}_ENDPOINT` - Direct endpoint override
//!    - `SERVICE_DISCOVERY_DOMAIN` - DNS domain for discovery
//!
//! 2. **DNS-Based Discovery**
//!    - mDNS: `{service}.local`
//!    - Production: `{service}.{domain}`
//!
//! 3. **Capability Registry**
//!    - Query by capability, not name
//!    - Supports multiple providers
//!
//! 4. **Local Fallback** (development only)
//!    - Graceful degradation
//!    - Local stub implementations

use crate::error::PrimalError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Discovered service information
#[derive(Debug, Clone)]
pub struct DiscoveredServiceInfo {
    /// Service identifier
    pub service_id: String,
    /// Primary endpoint
    pub endpoint: String,
    /// Service capabilities
    pub capabilities: Vec<String>,
    /// Service health status
    pub health: ServiceHealth,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Service health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceHealth {
    /// Service is healthy and responding
    Healthy,
    /// Service is degraded but operational
    Degraded,
    /// Service is unavailable
    Unavailable,
    /// Health status unknown
    Unknown,
}

/// Service discovery client trait
///
/// This trait defines the interface for discovering services in the ecosystem.
/// Implementations can use different discovery mechanisms (DNS, registry, etc.)
#[async_trait::async_trait]
pub trait ServiceDiscovery: Send + Sync {
    /// Discover a service by capability
    ///
    /// Returns the first available service that provides the requested capability.
    async fn discover_by_capability<'a>(
        &'a self,
        capability: &'a str,
    ) -> Result<DiscoveredServiceInfo, PrimalError>;

    /// Discover all services providing a capability
    ///
    /// Returns all available services that provide the requested capability.
    /// Useful for load balancing across multiple instances.
    async fn discover_all_by_capability<'a>(
        &'a self,
        capability: &'a str,
    ) -> Result<Vec<DiscoveredServiceInfo>, PrimalError>;

    /// Check if a capability is available
    ///
    /// Fast check without full discovery overhead.
    async fn is_capability_available<'a>(&'a self, capability: &'a str) -> bool;

    /// Get service by explicit ID
    ///
    /// For cases where service ID is known (e.g., from configuration).
    async fn get_service_by_id<'a>(
        &'a self,
        service_id: &'a str,
    ) -> Result<DiscoveredServiceInfo, PrimalError>;

    /// Refresh discovery cache
    ///
    /// Forces a refresh of the service discovery cache.
    async fn refresh<'a>(&'a self) -> Result<(), PrimalError>;
}

/// Standard capability names
///
/// These are well-known capability identifiers used across the ecosystem.
pub mod capabilities {
    /// Coordination and service mesh capability (Songbird)
    pub const COORDINATION: &str = "coordination";

    /// Security and authentication capability (BearDog)
    pub const SECURITY: &str = "security";

    /// Storage and persistence capability (NestGate)
    pub const STORAGE: &str = "storage";

    /// Workload management capability (ToadStool)
    pub const WORKLOAD: &str = "workload";

    /// AI intelligence capability (Squirrel)
    pub const AI_INTELLIGENCE: &str = "ai_intelligence";

    /// MCP protocol capability
    pub const MCP_PROTOCOL: &str = "mcp_protocol";

    /// Context management capability
    pub const CONTEXT_MANAGEMENT: &str = "context_management";
}

/// Configuration for service discovery
///
/// This allows tests to provide configuration without mutating global environment variables,
/// making tests truly concurrent and race-free.
#[derive(Debug, Clone)]
pub struct ServiceDiscoveryConfig {
    /// Environment mode ("production", "development", etc.)
    pub environment: Option<String>,
    /// DNS domain for service discovery
    pub dns_domain: Option<String>,
    /// Explicit endpoint overrides by capability
    pub endpoint_overrides: HashMap<String, String>,
    /// Development fallback configuration (host and port overrides)
    pub dev_fallback: Option<DevFallbackConfig>,
}

/// Development fallback configuration
#[derive(Debug, Clone)]
pub struct DevFallbackConfig {
    /// Host overrides by capability
    pub host_overrides: HashMap<String, String>,
    /// Port overrides by capability
    pub port_overrides: HashMap<String, u16>,
}

impl Default for ServiceDiscoveryConfig {
    fn default() -> Self {
        Self {
            environment: None,
            dns_domain: None,
            endpoint_overrides: HashMap::new(),
            dev_fallback: None,
        }
    }
}

/// Ecosystem service discovery client implementation
///
/// This implementation uses the ecosystem registry for service discovery
/// with multi-level fallback strategy.
pub struct EcosystemServiceDiscovery {
    /// Cached discovered services
    cache: Arc<RwLock<HashMap<String, DiscoveredServiceInfo>>>,
    /// HTTP client for health checks
    http_client: reqwest::Client,
    /// Configuration (injected for testing, reads env vars if not provided)
    config: ServiceDiscoveryConfig,
}

impl EcosystemServiceDiscovery {
    /// Create a new ecosystem service discovery client
    ///
    /// Reads configuration from environment variables. For testing with custom
    /// configuration, use `new_with_config()` instead.
    ///
    /// # Implementation Note
    ///
    /// This uses `unwrap_or_default()` for the HTTP client, which is safe because:
    /// - `reqwest::Client::default()` always provides a valid client
    /// - The default client has reasonable timeout and connection settings
    /// - Build failures are extremely rare (only on invalid TLS config)
    ///
    /// If custom TLS or advanced features are needed, use `new_with_client()` instead.
    pub fn new() -> Self {
        Self::new_with_config(ServiceDiscoveryConfig::default())
    }

    /// Create a new discovery client with custom configuration
    ///
    /// **Use this in tests** to provide configuration without mutating global environment variables.
    /// This makes tests truly concurrent and race-free.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = ServiceDiscoveryConfig {
    ///     environment: Some("production".to_string()),
    ///     ..Default::default()
    /// };
    /// let discovery = EcosystemServiceDiscovery::new_with_config(config);
    /// ```
    pub fn new_with_config(config: ServiceDiscoveryConfig) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap_or_else(|e| {
                    // Log the error for diagnostics, but use default client
                    tracing::warn!(
                        "Failed to build custom HTTP client: {}. Using default client.",
                        e
                    );
                    reqwest::Client::default()
                }),
            config,
        }
    }

    /// Create a new discovery client with a custom HTTP client
    ///
    /// Use this when you need fine-grained control over HTTP client behavior,
    /// such as custom TLS configuration, proxy settings, or connection pooling.
    pub fn new_with_client(http_client: reqwest::Client) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            http_client,
            config: ServiceDiscoveryConfig::default(),
        }
    }

    /// Discover service using multi-level strategy
    async fn discover_service(
        &self,
        capability: &str,
    ) -> Result<DiscoveredServiceInfo, PrimalError> {
        debug!("Discovering service for capability: {}", capability);

        // Level 1: Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(service) = cache.get(capability) {
                if service.health == ServiceHealth::Healthy {
                    debug!("Found healthy service in cache: {}", service.service_id);
                    return Ok(service.clone());
                }
            }
        }

        // Level 2: Environment variable override
        if let Some(service) = self.discover_from_env(capability).await? {
            self.cache_service(capability, &service).await;
            return Ok(service);
        }

        // Level 3: DNS-based discovery
        if let Some(service) = self.discover_from_dns(capability).await? {
            self.cache_service(capability, &service).await;
            return Ok(service);
        }

        // Level 4: Capability registry lookup
        // Future: Query ecosystem-wide capability registry
        // Currently skipped - relies on env vars and local config

        // Level 5: Local fallback (development only)
        if let Some(service) = self.discover_local_fallback(capability).await? {
            warn!(
                "Using local fallback for capability '{}': {}",
                capability, service.endpoint
            );
            self.cache_service(capability, &service).await;
            return Ok(service);
        }

        Err(PrimalError::ServiceDiscoveryFailed(format!(
            "No service found for capability: {}",
            capability
        )))
    }

    /// Discover service from environment variables or config
    async fn discover_from_env(
        &self,
        capability: &str,
    ) -> Result<Option<DiscoveredServiceInfo>, PrimalError> {
        // Priority 1: Check config overrides (for testing)
        if let Some(endpoint) = self.config.endpoint_overrides.get(capability) {
            debug!(
                "Discovered service from config override: {} = {}",
                capability, endpoint
            );

            let health = self.check_service_health(endpoint).await;

            return Ok(Some(DiscoveredServiceInfo {
                service_id: format!("{}-from-config", capability),
                endpoint: endpoint.clone(),
                capabilities: vec![capability.to_string()],
                health,
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("source".to_string(), "config".to_string());
                    meta
                },
            }));
        }

        // Priority 2: Check environment variables (production)
        let env_key = format!("{}_ENDPOINT", capability.to_uppercase());

        if let Ok(endpoint) = std::env::var(&env_key) {
            info!(
                "Discovered service from environment: {} = {}",
                env_key, endpoint
            );

            // Perform health check
            let health = self.check_service_health(&endpoint).await;

            return Ok(Some(DiscoveredServiceInfo {
                service_id: format!("{}-from-env", capability),
                endpoint,
                capabilities: vec![capability.to_string()],
                health,
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("source".to_string(), "environment".to_string());
                    meta
                },
            }));
        }

        Ok(None)
    }

    /// Discover service from DNS
    async fn discover_from_dns(
        &self,
        capability: &str,
    ) -> Result<Option<DiscoveredServiceInfo>, PrimalError> {
        // Use config dns_domain if provided, otherwise check env var
        let dns_domain = self
            .config
            .dns_domain
            .clone()
            .or_else(|| std::env::var("SERVICE_DISCOVERY_DOMAIN").ok())
            .unwrap_or_else(|| "local".to_string());

        if dns_domain == "local" {
            // Use mDNS pattern: {capability}.local
            let endpoint = format!("http://{}.local", capability);
            let health = self.check_service_health(&endpoint).await;

            if health != ServiceHealth::Unavailable {
                info!("Discovered service via mDNS: {}", endpoint);
                return Ok(Some(DiscoveredServiceInfo {
                    service_id: format!("{}-mdns", capability),
                    endpoint,
                    capabilities: vec![capability.to_string()],
                    health,
                    metadata: HashMap::new(),
                }));
            }
        } else {
            // Use DNS: {capability}.{domain}
            let endpoint = format!("http://{}.{}", capability, dns_domain);
            let health = self.check_service_health(&endpoint).await;

            if health != ServiceHealth::Unavailable {
                info!("Discovered service via DNS: {}", endpoint);
                return Ok(Some(DiscoveredServiceInfo {
                    service_id: format!("{}-dns", capability),
                    endpoint,
                    capabilities: vec![capability.to_string()],
                    health,
                    metadata: HashMap::new(),
                }));
            }
        }

        Ok(None)
    }

    /// Local fallback for development
    ///
    /// Maps capabilities to development services with environment variable overrides:
    /// - `DEV_{CAPABILITY}_HOST`: Override host (default: localhost)
    /// - `DEV_{CAPABILITY}_PORT`: Override port (default: capability-specific)
    ///
    /// Examples:
    /// - `DEV_COORDINATION_PORT=9001` - Override coordination service port
    /// - `DEV_SECURITY_HOST=192.168.1.100` - Override security service host
    async fn discover_local_fallback(
        &self,
        capability: &str,
    ) -> Result<Option<DiscoveredServiceInfo>, PrimalError> {
        // Check environment mode (config takes priority over env var)
        let environment = self
            .config
            .environment
            .clone()
            .or_else(|| std::env::var("ENVIRONMENT").ok())
            .unwrap_or_else(|| "development".to_string());

        // Only enable fallback in development
        if environment != "development" {
            return Ok(None);
        }

        // Default development ports for each capability
        let default_port = match capability {
            capabilities::COORDINATION => 8081,
            capabilities::SECURITY => 8083,
            capabilities::STORAGE => 8084,
            capabilities::WORKLOAD => 8082,
            capabilities::AI_INTELLIGENCE => 8080,
            _ => return Ok(None),
        };

        // Check config overrides first, then env vars
        let port = if let Some(dev_config) = &self.config.dev_fallback {
            dev_config
                .port_overrides
                .get(capability)
                .copied()
                .or_else(|| {
                    std::env::var(format!("DEV_{}_PORT", capability.to_uppercase()))
                        .ok()
                        .and_then(|p| p.parse().ok())
                })
                .unwrap_or(default_port)
        } else {
            std::env::var(format!("DEV_{}_PORT", capability.to_uppercase()))
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(default_port)
        };

        let host = if let Some(dev_config) = &self.config.dev_fallback {
            dev_config
                .host_overrides
                .get(capability)
                .cloned()
                .or_else(|| std::env::var(format!("DEV_{}_HOST", capability.to_uppercase())).ok())
                .unwrap_or_else(|| "localhost".to_string())
        } else {
            std::env::var(format!("DEV_{}_HOST", capability.to_uppercase()))
                .unwrap_or_else(|_| "localhost".to_string())
        };

        let endpoint = format!("http://{}:{}", host, port);
        let health = self.check_service_health(&endpoint).await;

        debug!(
            "Development fallback for capability '{}': {} (host={}, port={})",
            capability, endpoint, host, port
        );

        Ok(Some(DiscoveredServiceInfo {
            service_id: format!("{}-local", capability),
            endpoint,
            capabilities: vec![capability.to_string()],
            health,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("fallback".to_string(), "true".to_string());
                meta.insert("configurable".to_string(), "true".to_string());
                meta
            },
        }))
    }

    /// Check service health
    async fn check_service_health(&self, endpoint: &str) -> ServiceHealth {
        let health_url = format!("{}/health", endpoint);

        match self.http_client.get(&health_url).send().await {
            Ok(response) if response.status().is_success() => ServiceHealth::Healthy,
            Ok(_) => ServiceHealth::Degraded,
            Err(_) => ServiceHealth::Unavailable,
        }
    }

    /// Cache discovered service
    async fn cache_service(&self, capability: &str, service: &DiscoveredServiceInfo) {
        let mut cache = self.cache.write().await;
        cache.insert(capability.to_string(), service.clone());
    }
}

#[async_trait::async_trait]
impl ServiceDiscovery for EcosystemServiceDiscovery {
    async fn discover_by_capability<'a>(
        &'a self,
        capability: &'a str,
    ) -> Result<DiscoveredServiceInfo, PrimalError> {
        self.discover_service(capability).await
    }

    async fn discover_all_by_capability<'a>(
        &'a self,
        capability: &'a str,
    ) -> Result<Vec<DiscoveredServiceInfo>, PrimalError> {
        // Future: Query registry for all instances of a capability
        // Currently returns single instance (primary service)
        let service = self.discover_service(capability).await?;
        Ok(vec![service])
    }

    async fn is_capability_available<'a>(&'a self, capability: &'a str) -> bool {
        self.discover_service(capability).await.is_ok()
    }

    async fn get_service_by_id<'a>(
        &'a self,
        service_id: &'a str,
    ) -> Result<DiscoveredServiceInfo, PrimalError> {
        let cache = self.cache.read().await;
        for service in cache.values() {
            if service.service_id == service_id {
                return Ok(service.clone());
            }
        }

        Err(PrimalError::ServiceDiscoveryFailed(format!(
            "Service not found: {}",
            service_id
        )))
    }

    async fn refresh<'a>(&'a self) -> Result<(), PrimalError> {
        let mut cache = self.cache.write().await;
        cache.clear();
        info!("Service discovery cache cleared");
        Ok(())
    }
}

impl Default for EcosystemServiceDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

// Tests are in discovery_client_tests.rs for better organization
#[cfg(test)]
#[path = "discovery_client_tests.rs"]
mod tests;
