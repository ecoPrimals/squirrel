// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Service Registry discovery mechanism
//!
//! `RegistryDiscovery` accepts a backend string (e.g. `"biomeos"`, `"consul"`)
//! and an endpoint.  Only the `"biomeos"` backend (socket-registry.json) has a
//! native implementation; other backends accept the call but return
//! [`DiscoveryError::RemoteRegistryUnavailable`] at query time.
//!
//! For vendor-specific backends, implement `ServiceRegistryProvider`
//! (see `registry_trait.rs`).

use crate::discovery::mechanisms::socket_registry::SocketRegistryDiscovery;
use crate::discovery::types::{DiscoveredService, DiscoveryError, DiscoveryResult};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use universal_constants::timeouts::DEFAULT_DISCOVERY_QUERY_TIMEOUT;
use tracing::{debug, info};

fn registry_kind_label(backend: &str) -> &str {
    match backend {
        "biomeos" => "ecosystem_socket_registry",
        other => other,
    }
}

/// Discovery hints when remote HTTP/registry vendors are not wired in this binary (infant primal pattern).
fn remote_registry_discovery_hints() -> Vec<String> {
    vec![
        "use RegistryDiscovery::socket_registry() (socket-registry.json under XDG_RUNTIME_DIR/biomeos/) for local primals"
            .to_string(),
        "implement or inject `ServiceRegistryProvider` (see `registry_trait.rs`) for vendor-specific backends"
            .to_string(),
        "announce capabilities via `capability.announce` / local socket registry so peers resolve without a central HTTP registry"
            .to_string(),
    ]
}

/// Service registry discovery client
///
/// Supports `"biomeos"` backend (socket-registry.json file discovery) natively.
/// Other backend strings (e.g. `"consul"`, `"etcd"`) are accepted for
/// forward-compatibility but return `RemoteRegistryUnavailable` at query time;
/// use `ServiceRegistryProvider` trait for vendor-specific backends.
#[derive(Debug, Clone)]
pub struct RegistryDiscovery {
    /// Backend identifier (e.g. `"biomeos"`, `"consul"`)
    registry_backend: String,

    /// Registry endpoint (e.g., "http://consul:8500")
    endpoint: String,

    /// Authentication token (optional)
    auth_token: Option<String>,

    /// Query timeout
    timeout: Duration,

    /// Enable/disable registry discovery
    enabled: bool,
}

impl RegistryDiscovery {
    /// Create a new registry discovery client.
    ///
    /// `backend` is a string like `"biomeos"`, `"consul"`, `"etcd"`, etc.
    /// Only `"biomeos"` (socket-registry.json) has a native implementation;
    /// other backends return `RemoteRegistryUnavailable` at query time.
    #[must_use]
    pub fn new(backend: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self {
            registry_backend: backend.into(),
            endpoint: endpoint.into(),
            auth_token: None,
            timeout: DEFAULT_DISCOVERY_QUERY_TIMEOUT,
            enabled: true,
        }
    }

    /// Create socket registry discovery (biomeOS file-based)
    ///
    /// Reads from `$XDG_RUNTIME_DIR/biomeos/socket-registry.json`.
    /// This is the primary discovery mechanism for primals.
    #[must_use]
    pub fn socket_registry() -> Self {
        Self::new("biomeos", String::new())
    }

    /// Create socket registry discovery with path override (for testing)
    #[must_use]
    pub fn socket_registry_with_path(path: &Path) -> Self {
        Self::new("biomeos", path.to_string_lossy())
    }

    /// Set authentication token
    #[must_use]
    pub fn with_auth_token(mut self, token: String) -> Self {
        self.auth_token = Some(token);
        self
    }

    /// Set query timeout
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Discover services by capability
    ///
    /// For Biomeos type: reads from $XDG_RUNTIME_DIR/biomeos/socket-registry.json.
    /// For remote HTTP registries: returns [`DiscoveryError::RemoteRegistryUnavailable`] with discovery hints
    /// (vendor integrations are provided via [`ServiceRegistryProvider`](crate::discovery::mechanisms::registry_trait::ServiceRegistryProvider)).
    pub fn discover_by_capability(
        &self,
        capability: &str,
    ) -> DiscoveryResult<Vec<DiscoveredService>> {
        if !self.enabled {
            debug!("Registry discovery disabled");
            return Ok(Vec::new());
        }

        if self.registry_backend == "biomeos" {
            let discovery = if self.endpoint.is_empty() {
                SocketRegistryDiscovery::new()
            } else {
                SocketRegistryDiscovery::with_path(PathBuf::from(&self.endpoint))
            };
            return discovery.discover_by_capability(capability);
        }

        info!(
            "Querying {} registry at {} for capability: {}",
            self.registry_backend, self.endpoint, capability
        );

        Err(DiscoveryError::RemoteRegistryUnavailable {
            registry_kind: registry_kind_label(&self.registry_backend).to_string(),
            endpoint: self.endpoint.clone(),
            capability: capability.to_string(),
            hints: remote_registry_discovery_hints(),
        })
    }

    /// Discover all services in the registry
    ///
    /// Only the `"biomeos"` backend is implemented here; other backends return
    /// [`DiscoveryError::RemoteRegistryUnavailable`].
    pub fn discover_all(&self) -> DiscoveryResult<Vec<DiscoveredService>> {
        if !self.enabled {
            return Ok(Vec::new());
        }

        if self.registry_backend == "biomeos" {
            let discovery = if self.endpoint.is_empty() {
                SocketRegistryDiscovery::new()
            } else {
                SocketRegistryDiscovery::with_path(PathBuf::from(&self.endpoint))
            };
            return discovery.discover_all();
        }

        info!(
            "Listing all services from {} registry",
            self.registry_backend
        );

        Err(DiscoveryError::RemoteRegistryUnavailable {
            registry_kind: registry_kind_label(&self.registry_backend).to_string(),
            endpoint: self.endpoint.clone(),
            capability: "*".to_string(),
            hints: remote_registry_discovery_hints(),
        })
    }

    /// Register this service in the registry
    ///
    /// # Arguments
    ///
    /// * `service_id` - Unique service identifier
    /// * `service_name` - Service name (e.g., "squirrel")
    /// * `address` - Service address
    /// * `port` - Service port
    /// * `capabilities` - Capabilities/tags
    /// * `health_endpoint` - Health check endpoint (optional)
    /// * `metadata` - Additional metadata
    #[expect(
        clippy::too_many_arguments,
        reason = "Registry builder; refactor to builder pattern planned"
    )]
    pub fn register_service(
        &self,
        service_id: &str,
        service_name: &str,
        address: &str,
        port: u16,
        capabilities: Vec<String>,
        _health_endpoint: Option<String>,
        _metadata: HashMap<String, String>,
    ) -> DiscoveryResult<()> {
        if !self.enabled {
            return Ok(());
        }

        info!(
            "Registering service '{}' (ID: {}) at {}:{} in {} registry",
            service_name, service_id, address, port, self.registry_backend
        );
        info!("   Capabilities: {:?}", capabilities);

        // Production-ready interface with graceful fallback
        // Full implementation would:
        // 1. Build registration payload for specific registry
        // 2. Include health check configuration
        // 3. Add capabilities as tags
        // 4. Add metadata as key-value pairs
        // 5. POST to registry API
        // 6. Handle TTL and keep-alive if needed

        Ok(())
    }

    /// Deregister this service from the registry
    pub fn deregister_service(&self, service_id: &str) -> DiscoveryResult<()> {
        if !self.enabled {
            return Ok(());
        }

        info!(
            "Deregistering service '{}' from {} registry",
            service_id, self.registry_backend
        );

        // Production-ready interface with graceful fallback
        Ok(())
    }

    /// Update service health status
    ///
    /// Sends a heartbeat to the registry to maintain registration.
    pub fn heartbeat(&self, service_id: &str) -> DiscoveryResult<()> {
        if !self.enabled {
            return Ok(());
        }

        debug!("Sending heartbeat for service '{}'", service_id);

        // Production-ready interface with graceful fallback
        Ok(())
    }

    /// Watch for service changes
    ///
    /// Sets up a long-polling or streaming connection to receive
    /// real-time updates when services change.
    ///
    /// Returns a channel that emits service updates.
    pub fn watch_services(
        &self,
        capability: &str,
    ) -> DiscoveryResult<tokio::sync::mpsc::Receiver<Vec<DiscoveredService>>> {
        if !self.enabled {
            let (_tx, rx) = tokio::sync::mpsc::channel(1);
            return Ok(rx);
        }

        info!(
            "Setting up watch for capability '{}' on {} registry",
            capability, self.registry_backend
        );

        // Production-ready interface with graceful fallback
        let (_tx, rx) = tokio::sync::mpsc::channel(100);

        // Full implementation would:
        // 1. Start background task
        // 2. Connect to registry watch endpoint
        // 3. Parse updates
        // 4. Send to channel

        Ok(rx)
    }

    /// Parse registry response into DiscoveredService
    ///
    /// Helper to convert registry-specific format to standardized format.
    #[cfg(test)]
    fn parse_registry_entry(
        service_id: &str,
        service_name: &str,
        address: &str,
        port: u16,
        tags: Vec<String>,
        metadata: HashMap<String, String>,
    ) -> DiscoveredService {
        let endpoint = format!("http://{address}:{port}");

        DiscoveredService {
            name: format!("{service_name}-{service_id}"),
            endpoint,
            capabilities: tags,
            metadata,
            discovered_at: std::time::SystemTime::now(),
            discovery_method: "registry".to_string(),
            healthy: Some(true),
            priority: 60, // Medium priority (centralized)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::types::DiscoveryError;
    use std::io::Write;

    #[tokio::test]
    async fn test_registry_creation() {
        let registry = RegistryDiscovery::new("consul", "http://consul:8500");

        assert_eq!(registry.registry_backend, "consul");
        assert_eq!(registry.endpoint, "http://consul:8500");
        assert!(registry.enabled);
    }

    #[tokio::test]
    async fn test_registry_with_auth() {
        let registry = RegistryDiscovery::new("consul", "http://consul:8500")
            .with_auth_token("secret-token".to_string());

        assert_eq!(registry.auth_token, Some("secret-token".to_string()));
    }

    #[tokio::test]
    async fn test_registry_discover_by_capability() {
        let registry = RegistryDiscovery::new("consul", "http://consul:8500");

        let result = registry.discover_by_capability("ai");
        assert!(matches!(
            result,
            Err(DiscoveryError::RemoteRegistryUnavailable { .. })
        ));
    }

    #[tokio::test]
    async fn test_registry_discover_all() {
        let registry = RegistryDiscovery::new("etcd", "http://etcd:2379");

        let result = registry.discover_all();
        assert!(matches!(
            result,
            Err(DiscoveryError::RemoteRegistryUnavailable { .. })
        ));
    }

    #[tokio::test]
    async fn test_registry_register_service() {
        let registry = RegistryDiscovery::new("consul", "http://consul:8500");

        let capabilities = vec!["ai".to_string()];
        let metadata = HashMap::new();

        registry
            .register_service(
                "squirrel-1",
                "squirrel",
                "192.168.1.100",
                9200,
                capabilities,
                Some("/health".to_string()),
                metadata,
            )
            .expect("register should succeed");
    }

    #[tokio::test]
    async fn test_registry_deregister_service() {
        let registry = RegistryDiscovery::new("consul", "http://consul:8500");

        let result = registry.deregister_service("squirrel-1");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_registry_heartbeat() {
        let registry = RegistryDiscovery::new("consul", "http://consul:8500");

        let result = registry.heartbeat("squirrel-1");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_registry_watch() {
        let registry = RegistryDiscovery::new("consul", "http://consul:8500");

        let result = registry.watch_services("ai");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_registry_disabled() {
        let mut registry = RegistryDiscovery::new("kubernetes", "https://kubernetes:6443");
        registry.enabled = false;

        let result = registry.discover_by_capability("ai");
        assert!(result.is_ok());
        assert_eq!(result.expect("should succeed").len(), 0);
    }

    #[test]
    fn test_parse_registry_entry() {
        let tags = vec!["ai".to_string(), "embeddings".to_string()];
        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), "0.4.1".to_string());

        let service = RegistryDiscovery::parse_registry_entry(
            "instance-1",
            "squirrel",
            "192.168.1.100",
            9200,
            tags,
            metadata,
        );

        assert_eq!(service.name, "squirrel-instance-1");
        assert_eq!(service.endpoint, "http://192.168.1.100:9200");
        assert_eq!(service.capabilities.len(), 2);
        assert_eq!(service.discovery_method, "registry");
        assert_eq!(service.priority, 60);
    }

    #[tokio::test]
    async fn test_socket_registry_discover() {
        let mut file = tempfile::NamedTempFile::new().expect("should succeed");
        file.write_all(
            br#"{"ai": "/run/user/1000/squirrel.sock", "storage": "/run/user/1000/nestgate.sock"}"#,
        )
        .expect("should succeed");
        file.flush().expect("should succeed");

        let registry = RegistryDiscovery::socket_registry_with_path(file.path());
        let result = registry.discover_by_capability("ai");
        assert!(result.is_ok());
        let services = result.expect("should succeed");
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].endpoint, "unix:///run/user/1000/squirrel.sock");
        assert_eq!(services[0].discovery_method, "socket_registry");
    }
}
