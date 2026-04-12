// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

// ! Port Resolution - Zero Hardcoding
//!
//! This module provides utilities for resolving ports and endpoints at runtime,
//! following a proper priority chain: environment variables → service discovery → constants.
//!
//! # Design Principles
//!
//! 1. **Environment First**: Always check environment variables first
//! 2. **Discovery Second**: Query service mesh if ecosystem is available
//! 3. **Constants Last**: Fall back to universal constants
//! 4. **No Hardcoding**: Never use inline literals
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use universal_patterns::config::PortResolver;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let resolver = PortResolver::new();
//!     let _http_port = resolver.resolve_port("http").await?;
//!     let _metrics_endpoint = resolver.resolve_endpoint("metrics").await?;
//!     Ok(())
//! }
//! ```

use std::sync::Arc;
use universal_constants::network;

/// Port resolution error
#[derive(Debug, thiserror::Error)]
pub enum PortResolutionError {
    /// Service name not found in registry or constants
    #[error("Unknown service: {0}")]
    UnknownService(String),

    /// Port number is invalid (outside 1-65535 range)
    #[error("Invalid port number: {0}")]
    InvalidPort(String),

    /// Service discovery mechanism failed
    #[error("Service discovery failed: {0}")]
    DiscoveryFailed(String),
}

/// Result type for port resolution operations
pub type Result<T> = std::result::Result<T, PortResolutionError>;

/// Default [`ServiceDiscovery`] implementation used by [`PortResolver`] when no external
/// discovery backend is configured.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoServiceDiscovery;

/// Discovered service information
#[derive(Debug, Clone)]
pub struct DiscoveredService {
    /// Host address of the service
    pub host: String,
    /// Port the service is listening on
    pub port: u16,
    /// Scheme (http, https, ws, etc.)
    pub scheme: String,
}

/// Trait for service discovery (allows mocking in tests)
#[expect(
    async_fn_in_trait,
    reason = "internal trait — all impls are Send + Sync"
)]
pub trait ServiceDiscovery: Send + Sync {
    /// Discover service by name
    async fn discover_service(&self, name: &str) -> Option<DiscoveredService>;
}

/// Port resolver with proper fallback chain
///
/// Resolves ports and endpoints following the priority:
/// 1. Environment variables (highest priority)
/// 2. Service discovery (if ecosystem manager available)
/// 3. Universal constants (last resort)
pub struct PortResolver<D: ServiceDiscovery = NoServiceDiscovery> {
    /// Optional ecosystem manager for service discovery
    /// If None, skip discovery step
    ecosystem: Option<Arc<D>>,
}

impl ServiceDiscovery for NoServiceDiscovery {
    async fn discover_service(&self, _name: &str) -> Option<DiscoveredService> {
        None
    }
}

impl PortResolver<NoServiceDiscovery> {
    /// Create a new port resolver without service discovery
    ///
    /// This resolver will use environment variables and constants only.
    pub fn new() -> Self {
        Self { ecosystem: None }
    }
}

impl<D: ServiceDiscovery> PortResolver<D> {
    /// Create a port resolver with service discovery
    ///
    /// This resolver will try service discovery before falling back to constants.
    pub fn with_discovery(discovery: Arc<D>) -> Self {
        Self {
            ecosystem: Some(discovery),
        }
    }

    /// Resolve port with proper fallback chain
    ///
    /// # Priority Order
    ///
    /// 1. Environment variable `{SERVICE}_PORT`
    /// 2. Service discovery (if ecosystem available)
    /// 3. Universal constants
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use universal_patterns::config::PortResolver;
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let resolver = PortResolver::new();
    ///     let _http_port = resolver.resolve_port("http").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn resolve_port(&self, service: &str) -> Result<u16> {
        // 1. Environment variable (highest priority)
        let env_var = format!("{}_PORT", service.to_uppercase());
        if let Ok(port_str) = std::env::var(&env_var) {
            return port_str
                .parse()
                .map_err(|_| PortResolutionError::InvalidPort(port_str));
        }

        // 2. Service discovery (if ecosystem available)
        if let Some(ecosystem) = &self.ecosystem
            && let Some(discovered) = ecosystem.discover_service(service).await
        {
            return Ok(discovered.port);
        }

        // 3. Universal constants (last resort)
        Self::resolve_port_from_constants(service)
    }

    /// Resolve host with proper fallback chain
    ///
    /// Priority: environment variable → constants
    pub fn resolve_host(&self) -> String {
        std::env::var("BIND_ADDRESS").unwrap_or_else(|_| network::DEFAULT_LOCALHOST.to_string())
    }

    /// Resolve full endpoint (scheme + host + port)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use universal_patterns::config::PortResolver;
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let resolver = PortResolver::new();
    ///     let _endpoint = resolver.resolve_endpoint("http").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn resolve_endpoint(&self, service: &str) -> Result<String> {
        self.resolve_endpoint_with_scheme(service, "http").await
    }

    /// Resolve endpoint with specific scheme
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use universal_patterns::config::PortResolver;
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let resolver = PortResolver::new();
    ///     let _endpoint = resolver.resolve_endpoint_with_scheme("security", "https").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn resolve_endpoint_with_scheme(
        &self,
        service: &str,
        scheme: &str,
    ) -> Result<String> {
        // Check if full endpoint is in environment first
        let env_var = format!("{}_ENDPOINT", service.to_uppercase());
        if let Ok(endpoint) = std::env::var(&env_var) {
            return Ok(endpoint);
        }

        // Otherwise construct from parts
        let port = self.resolve_port(service).await?;
        let host = self.resolve_host();
        Ok(format!("{}://{}:{}", scheme, host, port))
    }

    /// Resolve port from universal constants
    fn resolve_port_from_constants(service: &str) -> Result<u16> {
        let port = match service {
            "http" => network::get_service_port("http"),
            "https" | "security" => network::get_service_port("security"), // HTTPS/security services
            "websocket" | "ws" => network::get_service_port("websocket"),
            "metrics" => network::get_service_port("metrics"),
            "admin" => network::get_service_port("admin"),
            "tarpc" => network::DEFAULT_METRICS_LISTEN_PORT, // tarpc binary protocol (same port as former grpc)
            _ => return Err(PortResolutionError::UnknownService(service.to_string())),
        };
        Ok(port)
    }
}

impl Default for PortResolver<NoServiceDiscovery> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_port_from_constants() {
        temp_env::with_vars_unset(["HTTP_PORT", "WEBSOCKET_PORT", "METRICS_PORT"], || {
            let rt = tokio::runtime::Runtime::new().expect("should succeed");
            rt.block_on(async {
                let resolver = PortResolver::new();
                assert_eq!(
                    resolver.resolve_port("http").await.expect("should succeed"),
                    8081
                );
                assert_eq!(
                    resolver
                        .resolve_port("websocket")
                        .await
                        .expect("should succeed"),
                    8080
                );
                assert_eq!(
                    resolver
                        .resolve_port("metrics")
                        .await
                        .expect("should succeed"),
                    9090
                );
            });
        });
    }

    #[test]
    fn test_resolve_port_from_env() {
        temp_env::with_var("TEST_PORT", Some("7777"), || {
            let rt = tokio::runtime::Runtime::new().expect("should succeed");
            rt.block_on(async {
                let resolver = PortResolver::new();
                assert_eq!(
                    resolver.resolve_port("test").await.expect("should succeed"),
                    7777
                );
            });
        });
    }

    #[test]
    fn test_resolve_endpoint() {
        temp_env::with_vars_unset(["HTTP_ENDPOINT", "HTTP_PORT", "BIND_ADDRESS"], || {
            let rt = tokio::runtime::Runtime::new().expect("should succeed");
            rt.block_on(async {
                let resolver = PortResolver::new();
                let endpoint = resolver
                    .resolve_endpoint("http")
                    .await
                    .expect("should succeed");
                assert_eq!(endpoint, "http://localhost:8081");
            });
        });
    }

    #[test]
    fn test_resolve_endpoint_with_scheme() {
        temp_env::with_var_unset("SECURITY_ENDPOINT", || {
            let rt = tokio::runtime::Runtime::new().expect("should succeed");
            rt.block_on(async {
                let resolver = PortResolver::new();
                let endpoint = resolver
                    .resolve_endpoint_with_scheme("security", "https")
                    .await
                    .expect("should succeed");
                assert_eq!(endpoint, "https://localhost:8083");
            });
        });
    }

    #[test]
    fn test_resolve_endpoint_from_env() {
        temp_env::with_var("API_ENDPOINT", Some("https://api.example.com"), || {
            let rt = tokio::runtime::Runtime::new().expect("should succeed");
            rt.block_on(async {
                let resolver = PortResolver::new();
                let endpoint = resolver
                    .resolve_endpoint("api")
                    .await
                    .expect("should succeed");
                assert_eq!(endpoint, "https://api.example.com");
            });
        });
    }

    #[tokio::test]
    async fn test_unknown_service() {
        let resolver = PortResolver::new();
        let result = resolver.resolve_port("unknown_service_xyz").await;

        assert!(result.is_err());
        match result {
            Err(PortResolutionError::UnknownService(s)) => {
                assert_eq!(s, "unknown_service_xyz");
            }
            _ => unreachable!("Expected UnknownService error"),
        }
    }

    #[test]
    fn test_invalid_port_env() {
        temp_env::with_var("BADPORT_PORT", Some("not_a_number"), || {
            let rt = tokio::runtime::Runtime::new().expect("should succeed");
            rt.block_on(async {
                let resolver = PortResolver::new();
                let result = resolver.resolve_port("badport").await;
                assert!(result.is_err());
                match result {
                    Err(PortResolutionError::InvalidPort(_)) => {}
                    _ => unreachable!("Expected InvalidPort error"),
                }
            });
        });
    }

    // Mock service discovery for testing
    struct MockDiscovery {
        services: std::collections::HashMap<String, DiscoveredService>,
    }

    impl ServiceDiscovery for MockDiscovery {
        async fn discover_service(&self, name: &str) -> Option<DiscoveredService> {
            self.services.get(name).cloned()
        }
    }

    #[tokio::test]
    async fn test_resolve_with_discovery() {
        let mut services = std::collections::HashMap::new();
        services.insert(
            "api".to_string(),
            DiscoveredService {
                host: "api.discovered.local".to_string(),
                port: 9999,
                scheme: "https".to_string(),
            },
        );

        let discovery = Arc::new(MockDiscovery { services });
        let resolver = PortResolver::with_discovery(discovery);

        let port = resolver.resolve_port("api").await.expect("should succeed");
        assert_eq!(port, 9999);
    }

    #[test]
    fn test_fallback_chain() {
        temp_env::with_var_unset("HTTP_PORT", || {
            let rt = tokio::runtime::Runtime::new().expect("should succeed");
            let default_port = rt.block_on(async {
                let resolver = PortResolver::new();
                resolver.resolve_port("http").await.expect("should succeed")
            });

            // Nested: set HTTP_PORT and verify override, then unset and verify fallback
            temp_env::with_var("HTTP_PORT", Some("7070"), || {
                rt.block_on(async {
                    let resolver2 = PortResolver::new();
                    assert_eq!(
                        resolver2
                            .resolve_port("http")
                            .await
                            .expect("should succeed"),
                        7070
                    );
                });
            });

            rt.block_on(async {
                let resolver3 = PortResolver::new();
                assert_eq!(
                    resolver3
                        .resolve_port("http")
                        .await
                        .expect("should succeed"),
                    default_port
                );
            });
        });
    }
}
