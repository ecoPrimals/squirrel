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
//! ```rust
//! use universal_patterns::config::PortResolver;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create resolver (with optional ecosystem manager)
//! let resolver = PortResolver::new(None);
//!
//! // Resolve individual port
//! let http_port = resolver.resolve_port("http").await?;
//! println!("HTTP port: {}", http_port);
//!
//! // Resolve full endpoint
//! let metrics_endpoint = resolver.resolve_endpoint("metrics").await?;
//! println!("Metrics: {}", metrics_endpoint);
//! # Ok(())
//! # }
//! ```

use std::sync::Arc;
use universal_constants::{env_vars, network};

/// Port resolution error
#[derive(Debug, thiserror::Error)]
pub enum PortResolutionError {
    #[error("Unknown service: {0}")]
    UnknownService(String),

    #[error("Invalid port number: {0}")]
    InvalidPort(String),

    #[error("Service discovery failed: {0}")]
    DiscoveryFailed(String),
}

pub type Result<T> = std::result::Result<T, PortResolutionError>;

/// Port resolver with proper fallback chain
///
/// Resolves ports and endpoints following the priority:
/// 1. Environment variables (highest priority)
/// 2. Service discovery (if ecosystem manager available)
/// 3. Universal constants (last resort)
pub struct PortResolver {
    /// Optional ecosystem manager for service discovery
    /// If None, skip discovery step
    ecosystem: Option<Arc<dyn ServiceDiscovery>>,
}

/// Trait for service discovery (allows mocking in tests)
#[async_trait::async_trait]
pub trait ServiceDiscovery: Send + Sync {
    /// Discover service by name
    async fn discover_service(&self, name: &str) -> Option<DiscoveredService>;
}

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

impl PortResolver {
    /// Create a new port resolver without service discovery
    ///
    /// This resolver will use environment variables and constants only.
    pub fn new() -> Self {
        Self { ecosystem: None }
    }

    /// Create a port resolver with service discovery
    ///
    /// This resolver will try service discovery before falling back to constants.
    pub fn with_discovery(discovery: Arc<dyn ServiceDiscovery>) -> Self {
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
    /// ```rust
    /// # use universal_patterns::config::PortResolver;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let resolver = PortResolver::new();
    /// let http_port = resolver.resolve_port("http").await?;
    /// assert_eq!(http_port, 8080); // Default from constants
    /// # Ok(())
    /// # }
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
        if let Some(ecosystem) = &self.ecosystem {
            if let Some(discovered) = ecosystem.discover_service(service).await {
                return Ok(discovered.port);
            }
        }

        // 3. Universal constants (last resort)
        self.resolve_port_from_constants(service)
    }

    /// Resolve host with proper fallback chain
    ///
    /// Priority: environment variable → constants
    pub fn resolve_host(&self) -> String {
        std::env::var(env_vars::BIND_ADDRESS)
            .unwrap_or_else(|_| network::DEFAULT_LOCALHOST.to_string())
    }

    /// Resolve full endpoint (scheme + host + port)
    ///
    /// # Example
    ///
    /// ```rust
    /// # use universal_patterns::config::PortResolver;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let resolver = PortResolver::new();
    /// let endpoint = resolver.resolve_endpoint("http").await?;
    /// assert_eq!(endpoint, "http://localhost:8080");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn resolve_endpoint(&self, service: &str) -> Result<String> {
        self.resolve_endpoint_with_scheme(service, "http").await
    }

    /// Resolve endpoint with specific scheme
    ///
    /// # Example
    ///
    /// ```rust
    /// # use universal_patterns::config::PortResolver;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let resolver = PortResolver::new();
    /// let endpoint = resolver.resolve_endpoint_with_scheme("api", "https").await?;
    /// assert_eq!(endpoint, "https://localhost:8080");
    /// # Ok(())
    /// # }
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
    fn resolve_port_from_constants(&self, service: &str) -> Result<u16> {
        let port = match service {
            "http" => network::DEFAULT_HTTP_PORT,
            #[allow(deprecated)]
            "https" => network::DEV_SECURITY_SERVICE_PORT, // HTTPS typically used by security services
            "websocket" | "ws" => network::DEFAULT_WEBSOCKET_PORT,
            "metrics" => network::DEFAULT_METRICS_PORT,
            "admin" => network::DEFAULT_ADMIN_PORT,
            "grpc" => network::DEFAULT_GRPC_PORT,
            _ => return Err(PortResolutionError::UnknownService(service.to_string())),
        };
        Ok(port)
    }
}

impl Default for PortResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resolve_port_from_constants() {
        let resolver = PortResolver::new();

        assert_eq!(resolver.resolve_port("http").await.unwrap(), 8080);
        assert_eq!(resolver.resolve_port("https").await.unwrap(), 8443);
        assert_eq!(resolver.resolve_port("metrics").await.unwrap(), 9091);
    }

    #[tokio::test]
    async fn test_resolve_port_from_env() {
        std::env::set_var("TEST_PORT", "7777");

        let resolver = PortResolver::new();
        assert_eq!(resolver.resolve_port("test").await.unwrap(), 7777);

        std::env::remove_var("TEST_PORT");
    }

    #[tokio::test]
    async fn test_resolve_endpoint() {
        let resolver = PortResolver::new();
        let endpoint = resolver.resolve_endpoint("http").await.unwrap();

        assert_eq!(endpoint, "http://localhost:8080");
    }

    #[tokio::test]
    async fn test_resolve_endpoint_with_scheme() {
        let resolver = PortResolver::new();
        let endpoint = resolver
            .resolve_endpoint_with_scheme("https", "https")
            .await
            .unwrap();

        assert_eq!(endpoint, "https://localhost:8443");
    }

    #[tokio::test]
    async fn test_resolve_endpoint_from_env() {
        std::env::set_var("API_ENDPOINT", "https://api.example.com");

        let resolver = PortResolver::new();
        let endpoint = resolver.resolve_endpoint("api").await.unwrap();

        assert_eq!(endpoint, "https://api.example.com");

        std::env::remove_var("API_ENDPOINT");
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
            _ => panic!("Expected UnknownService error"),
        }
    }

    #[tokio::test]
    async fn test_invalid_port_env() {
        std::env::set_var("BADPORT_PORT", "not_a_number");

        let resolver = PortResolver::new();
        let result = resolver.resolve_port("badport").await;

        assert!(result.is_err());
        match result {
            Err(PortResolutionError::InvalidPort(_)) => {}
            _ => panic!("Expected InvalidPort error"),
        }

        std::env::remove_var("BADPORT_PORT");
    }

    // Mock service discovery for testing
    struct MockDiscovery {
        services: std::collections::HashMap<String, DiscoveredService>,
    }

    #[async_trait::async_trait]
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

        let port = resolver.resolve_port("api").await.unwrap();
        assert_eq!(port, 9999);
    }

    #[tokio::test]
    async fn test_fallback_chain() {
        // No env var, no discovery → should use constants
        let resolver = PortResolver::new();
        assert_eq!(resolver.resolve_port("http").await.unwrap(), 8080);

        // With env var → should use env var
        std::env::set_var("HTTP_PORT", "7070");
        let resolver2 = PortResolver::new();
        assert_eq!(resolver2.resolve_port("http").await.unwrap(), 7070);
        std::env::remove_var("HTTP_PORT");

        // After cleanup, back to constant
        let resolver3 = PortResolver::new();
        assert_eq!(resolver3.resolve_port("http").await.unwrap(), 8080);
    }
}
