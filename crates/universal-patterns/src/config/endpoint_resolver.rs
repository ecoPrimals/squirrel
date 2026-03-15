// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Capability-Based Endpoint Resolution
//!
//! **Philosophy**: TRUE PRIMAL - No hardcoded endpoints, discover everything at runtime.
//!
//! This resolver integrates with the biomeOS socket standardization to provide:
//! 1. Unix socket discovery (preferred for local primals)
//! 2. HTTP/network fallback (for remote or legacy systems)
//! 3. Environment-based configuration (explicit overrides)
//! 4. Service mesh integration (future)
//!
//! ## Architecture
//!
//! ```text
//! Endpoint Resolution Priority:
//!
//! 1. EXPLICIT_ENDPOINT env var (highest priority)
//! 2. Unix socket discovery (/run/user/<uid>/biomeos/<primal>.sock)
//! 3. Service mesh discovery (query mesh for endpoint)
//! 4. Network discovery (mDNS, Consul, etc.)
//! 5. Fallback defaults (with warnings)
//! ```ignore
//!
//! ## Usage
//!
//! ```ignore
//! use squirrel_universal_patterns::config::EndpointResolver;
//!
//! // Create resolver
//! let resolver = EndpointResolver::new();
//!
//! // Resolve endpoint for a capability/primal
//! let endpoint = resolver.resolve(universal_constants::capabilities::SERVICE_MESH_CAPABILITY).await?;
//!
//! // Use the endpoint
//! match endpoint {
//!     Endpoint::UnixSocket(path) => {
//!         // Connect via Unix socket (preferred!)
//!         let stream = UnixStream::connect(path).await?;
//!     }
//!     Endpoint::Http(url) => {
//!         // Connect via HTTP (fallback)
//!         let response = client.get(url).send().await?;
//!     }
//! }
//! ```

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Endpoint types for primal communication
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Endpoint {
    /// Unix domain socket (preferred for local communication)
    UnixSocket(PathBuf),

    /// HTTP/HTTPS URL (for remote or legacy communication)
    Http(String),

    /// WebSocket URL
    WebSocket(String),
}

impl Endpoint {
    /// Create a Unix socket endpoint
    pub fn unix_socket<P: Into<PathBuf>>(path: P) -> Self {
        Self::UnixSocket(path.into())
    }

    /// Create an HTTP endpoint
    pub fn http<S: Into<String>>(url: S) -> Self {
        Self::Http(url.into())
    }

    /// Create a WebSocket endpoint
    pub fn websocket<S: Into<String>>(url: S) -> Self {
        Self::WebSocket(url.into())
    }

    /// Get a string representation of the endpoint
    pub fn as_str(&self) -> String {
        match self {
            Self::UnixSocket(path) => format!("unix://{}", path.display()),
            Self::Http(url) => url.clone(),
            Self::WebSocket(url) => url.clone(),
        }
    }

    /// Check if this is a Unix socket endpoint
    pub fn is_unix_socket(&self) -> bool {
        matches!(self, Self::UnixSocket(_))
    }

    /// Check if this is an HTTP endpoint
    pub fn is_http(&self) -> bool {
        matches!(self, Self::Http(_))
    }

    /// Check if this is a WebSocket endpoint
    pub fn is_websocket(&self) -> bool {
        matches!(self, Self::WebSocket(_))
    }
}

/// Resolution strategy for discovering endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResolutionStrategy {
    /// Prefer Unix sockets, fall back to network
    /// TRUE PRIMAL: Prefer local Unix sockets for efficiency and security
    #[default]
    PreferSocket,

    /// Prefer network, fall back to Unix socket
    PreferNetwork,

    /// Only use Unix sockets (fail if not available)
    SocketOnly,

    /// Only use network (fail if not available)
    NetworkOnly,
}

/// Capability-based endpoint resolver
///
/// Resolves endpoints for primals and services using multiple discovery
/// mechanisms with intelligent fallback.
pub struct EndpointResolver {
    /// Resolution strategy
    strategy: ResolutionStrategy,

    /// Cached endpoint mappings
    cache: Arc<RwLock<std::collections::HashMap<String, Endpoint>>>,

    /// Enable warnings for fallback usage
    warn_on_fallback: bool,
}

impl EndpointResolver {
    /// Create a new endpoint resolver with default settings
    ///
    /// Uses `PreferSocket` strategy and enables fallback warnings.
    pub fn new() -> Self {
        Self {
            strategy: ResolutionStrategy::default(),
            cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
            warn_on_fallback: true,
        }
    }

    /// Create a resolver with a specific strategy
    pub fn with_strategy(strategy: ResolutionStrategy) -> Self {
        Self {
            strategy,
            cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
            warn_on_fallback: true,
        }
    }

    /// Set whether to warn on fallback usage
    pub fn warn_on_fallback(mut self, enabled: bool) -> Self {
        self.warn_on_fallback = enabled;
        self
    }

    /// Resolve an endpoint for a primal or service
    ///
    /// ## Arguments
    /// - `name`: Capability or service name (e.g., "service-mesh", "security", "compute")
    ///
    /// ## Returns
    /// - `Ok(Endpoint)` if endpoint found
    /// - `Err(String)` if no endpoint available
    ///
    /// ## Resolution Order
    /// 1. Check cache (fast path)
    /// 2. Try explicit environment variable (`{NAME}_ENDPOINT`)
    /// 3. Try Unix socket discovery (standard path)
    /// 4. Try network discovery (service mesh, mDNS, etc.)
    /// 5. Use fallback defaults (with warnings)
    pub async fn resolve(&self, name: &str) -> Result<Endpoint, String> {
        // 1. Check cache
        {
            let cache = self.cache.read().await;
            if let Some(endpoint) = cache.get(name) {
                debug!("Endpoint cache hit for '{}': {}", name, endpoint.as_str());
                return Ok(endpoint.clone());
            }
        }

        // 2. Try explicit environment variable
        let env_var = format!("{}_ENDPOINT", name.to_uppercase());
        if let Ok(explicit_endpoint) = std::env::var(&env_var) {
            info!(
                "Using explicit endpoint from {}: {}",
                env_var, explicit_endpoint
            );

            let endpoint = Self::parse_endpoint(&explicit_endpoint)?;
            self.cache_endpoint(name, endpoint.clone()).await;
            return Ok(endpoint);
        }

        // 3. Apply resolution strategy
        let endpoint = match self.strategy {
            ResolutionStrategy::PreferSocket => self
                .try_unix_socket(name)
                .or_else(|| self.try_network(name))
                .unwrap_or_else(|| self.fallback_endpoint(name)),
            ResolutionStrategy::PreferNetwork => self
                .try_network(name)
                .or_else(|| self.try_unix_socket(name))
                .unwrap_or_else(|| self.fallback_endpoint(name)),
            ResolutionStrategy::SocketOnly => self
                .try_unix_socket(name)
                .ok_or_else(|| format!("Unix socket not found for '{}'", name))?,
            ResolutionStrategy::NetworkOnly => self
                .try_network(name)
                .ok_or_else(|| format!("Network endpoint not found for '{}'", name))?,
        };

        self.cache_endpoint(name, endpoint.clone()).await;
        Ok(endpoint)
    }

    /// Try to discover Unix socket for a primal
    ///
    /// Follows biomeOS socket standardization:
    /// `/run/user/<uid>/biomeos/<primal>.sock`
    fn try_unix_socket(&self, name: &str) -> Option<Endpoint> {
        // First check environment variable for socket path
        let socket_env = format!("{}_SOCKET", name.to_uppercase());
        if let Ok(socket_path) = std::env::var(&socket_env) {
            let path = PathBuf::from(socket_path);
            if path.exists() {
                debug!("Found Unix socket via {}: {}", socket_env, path.display());
                return Some(Endpoint::UnixSocket(path));
            }
        }

        // Try standard biomeOS path
        // Use nix crate for safe getuid() or std::env for portable solution
        let uid = std::env::var("UID")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or_else(|| {
                // Fallback: try to get UID from XDG_RUNTIME_DIR path
                if let Ok(xdg_runtime) = std::env::var("XDG_RUNTIME_DIR")
                    && let Some(uid_str) = xdg_runtime.strip_prefix("/run/user/")
                    && let Some(uid_part) = uid_str.split('/').next()
                    && let Ok(uid) = uid_part.parse::<u32>()
                {
                    return uid;
                }
                // Last resort: assume UID 1000 (common for first user)
                warn!("Could not determine UID, assuming 1000");
                1000
            });

        let standard_path = PathBuf::from(format!("/run/user/{}/biomeos/{}.sock", uid, name));

        if standard_path.exists() {
            debug!(
                "Found Unix socket at standard path: {}",
                standard_path.display()
            );
            return Some(Endpoint::UnixSocket(standard_path));
        }

        // Try XDG runtime directory
        if let Ok(xdg_runtime) = std::env::var("XDG_RUNTIME_DIR") {
            let xdg_path = PathBuf::from(format!("{}/biomeos/{}.sock", xdg_runtime, name));
            if xdg_path.exists() {
                debug!("Found Unix socket in XDG runtime: {}", xdg_path.display());
                return Some(Endpoint::UnixSocket(xdg_path));
            }
        }

        debug!("No Unix socket found for '{}'", name);
        None
    }

    /// Try to discover network endpoint
    ///
    /// Checks:
    /// 1. Service mesh discovery (query mesh)
    /// 2. Default network ports by service type
    fn try_network(&self, name: &str) -> Option<Endpoint> {
        // Check for explicit port environment variable
        let port_env = format!("{}_PORT", name.to_uppercase());
        if let Ok(port_str) = std::env::var(&port_env)
            && let Ok(port) = port_str.parse::<u16>()
        {
            let url = format!("http://localhost:{}", port);
            debug!("Found network endpoint via {}: {}", port_env, url);
            return Some(Endpoint::Http(url));
        }

        // NOTE(phase2): Query service mesh for endpoint - requires mesh integration
        // if let Some(endpoint) = query_service_mesh(name) {
        //     return Some(Endpoint::Http(endpoint));
        // }

        debug!("No network endpoint found for '{}'", name);
        None
    }

    /// Generate fallback endpoint (with warnings)
    ///
    /// Uses capability-based discovery constants. Primal names are accepted
    /// for backward compatibility but map to capability ports.
    fn fallback_endpoint(&self, name: &str) -> Endpoint {
        use universal_constants::capabilities;
        use universal_constants::deployment::ports;

        // Capability-based: use capability constants for discovery.
        // TRUE PRIMAL: No hardcoded primal names in production routing.
        let port = match name.to_lowercase().as_str() {
            n if n == capabilities::SERVICE_MESH_CAPABILITY || n == "network" => {
                ports::service_mesh()
            }
            n if n == capabilities::SECURITY_CAPABILITY => ports::security_service(),
            n if n == capabilities::COMPUTE_CAPABILITY => ports::compute_service(),
            n if n == capabilities::STORAGE_CAPABILITY => ports::storage_service(),
            n if n == capabilities::SELF_PRIMAL_NAME => ports::api_gateway(), // AI orchestration (us!)
            "websocket" | "ws" => ports::websocket(),
            "http" => 8081,
            "admin" => 8082,
            "metrics" => 9090,
            "discovery" => 8500,
            _ => {
                if self.warn_on_fallback {
                    warn!(
                        "Unknown service '{}' - no fallback port, using 0 (OS will allocate)",
                        name
                    );
                }
                0
            }
        };

        if port > 0 && self.warn_on_fallback {
            warn!(
                "Using fallback network endpoint for '{}': http://localhost:{} \
                 (set {}_ENDPOINT or {}_SOCKET for explicit configuration)",
                name,
                port,
                name.to_uppercase(),
                name.to_uppercase()
            );
        }

        if port == 0 {
            // Return a placeholder that will cause a connection error
            // This forces the caller to explicitly configure
            Endpoint::Http(format!("http://localhost:0?error=no-fallback-for-{}", name))
        } else {
            Endpoint::Http(format!("http://localhost:{}", port))
        }
    }

    /// Parse endpoint string into Endpoint type
    fn parse_endpoint(s: &str) -> Result<Endpoint, String> {
        if s.starts_with("unix://") || s.starts_with('/') {
            let path = s.strip_prefix("unix://").unwrap_or(s);
            Ok(Endpoint::UnixSocket(PathBuf::from(path)))
        } else if s.starts_with("ws://") || s.starts_with("wss://") {
            Ok(Endpoint::WebSocket(s.to_string()))
        } else if s.starts_with("http://") || s.starts_with("https://") {
            Ok(Endpoint::Http(s.to_string()))
        } else {
            Err(format!("Invalid endpoint format: {}", s))
        }
    }

    /// Cache an endpoint for future lookups
    async fn cache_endpoint(&self, name: &str, endpoint: Endpoint) {
        let mut cache = self.cache.write().await;
        cache.insert(name.to_string(), endpoint);
    }

    /// Clear the endpoint cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Invalidate a specific cached endpoint
    pub async fn invalidate(&self, name: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(name);
    }
}

impl Default for EndpointResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_types() {
        let unix = Endpoint::unix_socket("/tmp/test.sock");
        assert!(unix.is_unix_socket());
        assert!(!unix.is_http());

        let http = Endpoint::http("http://localhost:8080");
        assert!(http.is_http());
        assert!(!http.is_unix_socket());

        let ws = Endpoint::websocket("ws://localhost:8080");
        assert!(ws.is_websocket());
    }

    #[test]
    fn test_endpoint_as_str() {
        let unix = Endpoint::unix_socket("/tmp/test.sock");
        assert!(unix.as_str().contains("/tmp/test.sock"));

        let http = Endpoint::http("http://localhost:8080");
        assert_eq!(http.as_str(), "http://localhost:8080");
    }

    #[test]
    fn test_parse_endpoint() {
        let unix1 = EndpointResolver::parse_endpoint("unix:///tmp/test.sock").unwrap();
        assert!(unix1.is_unix_socket());

        let unix2 = EndpointResolver::parse_endpoint("/tmp/test.sock").unwrap();
        assert!(unix2.is_unix_socket());

        let http = EndpointResolver::parse_endpoint("http://localhost:8080").unwrap();
        assert!(http.is_http());

        let ws = EndpointResolver::parse_endpoint("ws://localhost:8080").unwrap();
        assert!(ws.is_websocket());
    }

    #[test]
    fn test_resolution_strategy() {
        let resolver = EndpointResolver::with_strategy(ResolutionStrategy::PreferSocket);
        assert_eq!(resolver.strategy, ResolutionStrategy::PreferSocket);

        let resolver = EndpointResolver::with_strategy(ResolutionStrategy::NetworkOnly);
        assert_eq!(resolver.strategy, ResolutionStrategy::NetworkOnly);
    }

    #[tokio::test]
    async fn test_explicit_endpoint_env_var() {
        unsafe { std::env::set_var("TEST_PRIMAL_ENDPOINT", "http://localhost:9999") };

        let resolver = EndpointResolver::new();
        let endpoint = resolver.resolve("test_primal").await.unwrap();

        assert_eq!(
            endpoint,
            Endpoint::Http("http://localhost:9999".to_string())
        );

        unsafe { std::env::remove_var("TEST_PRIMAL_ENDPOINT") };
    }

    #[tokio::test]
    async fn test_cache() {
        let resolver = EndpointResolver::new();

        unsafe { std::env::set_var("CACHE_TEST_ENDPOINT", "http://localhost:7777") };

        // First resolution - should cache
        let endpoint1 = resolver.resolve("cache_test").await.unwrap();

        // Change environment (cache should return old value)
        unsafe { std::env::set_var("CACHE_TEST_ENDPOINT", "http://localhost:8888") };
        let endpoint2 = resolver.resolve("cache_test").await.unwrap();

        assert_eq!(endpoint1, endpoint2);

        // Invalidate cache
        resolver.invalidate("cache_test").await;

        // Should now get new value
        let endpoint3 = resolver.resolve("cache_test").await.unwrap();
        assert_eq!(
            endpoint3,
            Endpoint::Http("http://localhost:8888".to_string())
        );

        unsafe { std::env::remove_var("CACHE_TEST_ENDPOINT") };
    }

    #[tokio::test]
    async fn test_fallback_for_capability_based_services() {
        use universal_constants::capabilities;

        // Use PreferNetwork strategy (will use network first, then fallback if needed)
        let resolver = EndpointResolver::with_strategy(ResolutionStrategy::PreferNetwork)
            .warn_on_fallback(false);

        // Capability-based discovery: use capability constants
        let service_mesh = resolver
            .resolve(capabilities::SERVICE_MESH_CAPABILITY)
            .await
            .unwrap();
        assert!(!service_mesh.as_str().is_empty());

        let security = resolver
            .resolve(capabilities::SECURITY_CAPABILITY)
            .await
            .unwrap();
        assert!(!security.as_str().is_empty());

        let self_primal = resolver
            .resolve(capabilities::SELF_PRIMAL_NAME)
            .await
            .unwrap();
        assert!(!self_primal.as_str().is_empty());
    }
}
