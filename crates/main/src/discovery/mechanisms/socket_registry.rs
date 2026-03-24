// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Socket Registry discovery - biomeOS capability-based discovery
//!
//! Reads from `$XDG_RUNTIME_DIR/biomeos/socket-registry.json` as specified in
//! SOCKET_REGISTRY_SPEC.md. This is how primals discover each other at runtime
//! — by reading the socket registry that biomeOS maintains.
//!
//! **Primal pattern**: Self-knowledge only; discover others at runtime.

use crate::discovery::types::{DiscoveredService, DiscoveryResult};
use dashmap::DashMap;
use nix::unistd::Uid;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tracing::{debug, info};

/// Default cache TTL for socket registry entries
const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(30);

/// Cached registry entry with TTL
struct CachedRegistry {
    entries: HashMap<String, String>,
    loaded_at: Instant,
}

/// Socket registry discovery - file-system based capability discovery
///
/// Reads from `$XDG_RUNTIME_DIR/biomeos/socket-registry.json` (or
/// `/run/user/<uid>/biomeos/socket-registry.json` if XDG_RUNTIME_DIR is unset).
pub struct SocketRegistryDiscovery {
    /// Optional path override (for testing)
    path_override: Option<PathBuf>,

    /// Cache of registry contents
    cache: Arc<DashMap<(), CachedRegistry>>,

    /// Cache TTL
    cache_ttl: Duration,
}

impl SocketRegistryDiscovery {
    /// Create a new socket registry discovery with default path
    #[must_use]
    pub fn new() -> Self {
        Self {
            path_override: None,
            cache: Arc::new(DashMap::new()),
            cache_ttl: DEFAULT_CACHE_TTL,
        }
    }

    /// Create with custom path (for testing)
    #[must_use]
    pub fn with_path(path: PathBuf) -> Self {
        Self {
            path_override: Some(path),
            cache: Arc::new(DashMap::new()),
            cache_ttl: DEFAULT_CACHE_TTL,
        }
    }

    /// Set cache TTL
    #[must_use]
    pub const fn with_cache_ttl(mut self, ttl: Duration) -> Self {
        self.cache_ttl = ttl;
        self
    }

    /// Get the registry file path
    fn registry_path(&self) -> PathBuf {
        if let Some(ref p) = self.path_override {
            return p.clone();
        }

        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            return PathBuf::from(runtime_dir)
                .join("biomeos")
                .join("socket-registry.json");
        }

        // Fallback: /run/user/<uid>/biomeos/socket-registry.json
        let uid = Uid::current();
        PathBuf::from(format!(
            "/run/user/{}/biomeos/socket-registry.json",
            uid.as_raw()
        ))
    }

    /// Read and parse the socket registry file
    ///
    /// Returns empty map on file-not-found (graceful degradation).
    fn read_registry(&self) -> HashMap<String, String> {
        let path = self.registry_path();

        // Check cache first
        if let Some(entry) = self.cache.get(&())
            && entry.loaded_at.elapsed() < self.cache_ttl
        {
            return entry.entries.clone();
        }

        let result = self.read_registry_from_disk(&path);

        if let Ok(ref entries) = result {
            self.cache.insert(
                (),
                CachedRegistry {
                    entries: entries.clone(),
                    loaded_at: Instant::now(),
                },
            );
        }

        result.unwrap_or_default()
    }

    fn read_registry_from_disk(
        &self,
        path: &PathBuf,
    ) -> Result<HashMap<String, String>, std::io::Error> {
        let contents = std::fs::read_to_string(path)?;
        let parsed: HashMap<String, String> = serde_json::from_str(&contents).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid socket registry JSON: {e}"),
            )
        })?;
        Ok(parsed)
    }

    /// Discover services by capability
    pub fn discover_by_capability(
        &self,
        capability: &str,
    ) -> DiscoveryResult<Vec<DiscoveredService>> {
        let registry = self.read_registry();

        if registry.is_empty() {
            debug!(
                "Socket registry empty or unavailable at {}",
                self.registry_path().display()
            );
            return Ok(Vec::new());
        }

        let capability_lower = capability.to_lowercase();

        // Exact match first
        if let Some(socket_path) = registry.get(&capability_lower) {
            let service = self.socket_path_to_service(&capability_lower, socket_path);
            info!(
                "✓ Discovered '{}' via socket registry: {}",
                capability, socket_path
            );
            return Ok(vec![service]);
        }

        // Prefix match for vendor.capability (e.g., "ai" matches "ai.inference")
        let matching: Vec<_> = registry
            .iter()
            .filter(|(k, _)| {
                k.as_str() == capability_lower.as_str()
                    || k.starts_with(&format!("{capability_lower}."))
            })
            .collect();

        if matching.is_empty() {
            debug!("Capability '{}' not found in socket registry", capability);
            return Ok(Vec::new());
        }

        let services: Vec<DiscoveredService> = matching
            .into_iter()
            .map(|(cap, path)| self.socket_path_to_service(cap, path))
            .collect();

        info!(
            "✓ Discovered {} service(s) for '{}' via socket registry",
            services.len(),
            capability
        );

        Ok(services)
    }

    /// Discover all services in the registry
    pub fn discover_all(&self) -> DiscoveryResult<Vec<DiscoveredService>> {
        let registry = self.read_registry();

        if registry.is_empty() {
            return Ok(Vec::new());
        }

        let services: Vec<DiscoveredService> = registry
            .iter()
            .map(|(cap, path)| self.socket_path_to_service(cap, path))
            .collect();

        Ok(services)
    }

    /// Convert socket path to DiscoveredService
    ///
    /// Uses unix:// scheme for Unix socket endpoints.
    fn socket_path_to_service(&self, capability: &str, socket_path: &str) -> DiscoveredService {
        let endpoint = format!("unix://{socket_path}");

        DiscoveredService {
            name: format!("{capability}-provider"),
            endpoint,
            capabilities: vec![capability.to_string()],
            metadata: HashMap::new(),
            discovered_at: SystemTime::now(),
            discovery_method: "socket_registry".to_string(),
            healthy: Some(true),
            priority: 65, // Medium-high (local, maintained by biomeOS)
        }
    }

    /// Clear the cache (e.g., for testing)
    pub fn clear_cache(&self) {
        self.cache.clear();
    }
}

impl Default for SocketRegistryDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

/// Discover from socket registry - shared helper for fallback from mDNS/DNS-SD
///
/// Used when mDNS/DNS-SD cannot perform real discovery (no pure-Rust impl available).
pub fn discover_from_socket_registry(capability: &str) -> DiscoveryResult<Vec<DiscoveredService>> {
    let discovery = SocketRegistryDiscovery::new();
    discovery.discover_by_capability(capability)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_socket_registry_path() {
        let discovery = SocketRegistryDiscovery::new();
        let path = discovery.registry_path();
        assert!(path.ends_with("socket-registry.json"));
        assert!(path.to_string_lossy().contains("biomeos"));
    }

    #[test]
    fn test_socket_registry_with_custom_path() {
        let path = PathBuf::from("/tmp/test-registry.json");
        let discovery = SocketRegistryDiscovery::with_path(path.clone());
        assert_eq!(discovery.registry_path(), path);
    }

    #[test]
    fn test_discover_from_empty_registry() {
        let mut file = NamedTempFile::new().expect("should succeed");
        file.write_all(b"{}").expect("should succeed");
        file.flush().expect("should succeed");

        let discovery = SocketRegistryDiscovery::with_path(PathBuf::from(file.path()));
        let result = discovery.discover_by_capability("ai");
        assert!(result.is_ok());
        assert!(result.expect("should succeed").is_empty());
    }

    #[test]
    fn test_discover_from_registry() {
        let mut file = NamedTempFile::new().expect("should succeed");
        file.write_all(
            br#"{"ai": "/run/user/1000/squirrel.sock", "storage": "/run/user/1000/nestgate.sock"}"#,
        )
        .expect("should succeed");
        file.flush().expect("should succeed");

        let discovery = SocketRegistryDiscovery::with_path(PathBuf::from(file.path()))
            .with_cache_ttl(Duration::from_secs(60));
        let result = discovery.discover_by_capability("ai");
        assert!(result.is_ok());
        let services = result.expect("should succeed");
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].endpoint, "unix:///run/user/1000/squirrel.sock");
        assert_eq!(services[0].capabilities, vec!["ai"]);
        assert_eq!(services[0].discovery_method, "socket_registry");
    }

    #[test]
    fn test_discover_all() {
        let mut file = NamedTempFile::new().expect("should succeed");
        file.write_all(
            br#"{"ai": "/run/user/1000/squirrel.sock", "storage": "/run/user/1000/nestgate.sock"}"#,
        )
        .expect("should succeed");
        file.flush().expect("should succeed");

        let discovery = SocketRegistryDiscovery::with_path(PathBuf::from(file.path()));
        let result = discovery.discover_all();
        assert!(result.is_ok());
        let services = result.expect("should succeed");
        assert_eq!(services.len(), 2);
    }

    #[test]
    fn test_discover_nonexistent_file() {
        let discovery = SocketRegistryDiscovery::with_path(PathBuf::from("/nonexistent/path.json"));
        let result = discovery.discover_by_capability("ai");
        assert!(result.is_ok());
        assert!(result.expect("should succeed").is_empty());
    }
}
