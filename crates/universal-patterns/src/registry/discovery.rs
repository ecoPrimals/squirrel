// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! TRUE PRIMAL Discovery Implementation
//!
//! Capability-based primal discovery via Unix sockets.
//! No knowledge of specific primals (Songbird, BearDog, etc.)
//! Only discovers via capabilities.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::net::UnixStream;
use tracing::{debug, info, warn};

use crate::traits::{PrimalCapability, PrimalContext, PrimalHealth, PrimalResult, PrimalType};

use super::DiscoveredPrimal;

/// Unix socket discovery configuration
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Standard socket directory paths to scan
    pub socket_dirs: Vec<PathBuf>,
    /// Socket file patterns to match (e.g., "*.sock")
    pub socket_patterns: Vec<String>,
    /// Timeout for discovery probes (milliseconds)
    pub probe_timeout_ms: u64,
    /// Whether to skip unhealthy primals
    pub skip_unhealthy: bool,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            // Ecosystem XDG socket convention for TRUE PRIMAL discovery
            socket_dirs: vec![
                universal_constants::network::get_socket_dir(),
                PathBuf::from(universal_constants::network::BIOMEOS_SOCKET_FALLBACK_DIR),
            ],
            socket_patterns: vec![
                "*.sock".to_string(),
                "*-nat0.sock".to_string(), // Family-specific sockets
            ],
            probe_timeout_ms: 1000, // 1 second timeout
            skip_unhealthy: true,
        }
    }
}

/// Discovery result containing primal information
#[derive(Debug, Clone)]
pub struct DiscoveryResult {
    /// Socket path that was discovered
    pub socket_path: PathBuf,
    /// Primal information (if successfully probed)
    pub primal_info: Option<PrimalInfo>,
    /// Discovery status
    pub status: DiscoveryStatus,
}

/// Primal information from discovery probe
#[derive(Debug, Clone)]
pub struct PrimalInfo {
    /// Primal ID (e.g., "squirrel", "beardog")
    pub primal_id: String,
    /// Instance ID (e.g., "squirrel-user123")
    pub instance_id: String,
    /// Primal type
    pub primal_type: PrimalType,
    /// Capabilities provided
    pub capabilities: Vec<PrimalCapability>,
    /// Health status
    pub health: PrimalHealth,
}

/// Discovery status for a socket
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryStatus {
    /// Successfully discovered and probed
    Success,
    /// Socket found but probe failed
    ProbeFailed(String),
    /// Socket found but primal is unhealthy
    Unhealthy(String),
    /// Not a primal socket (wrong protocol)
    NotAPrimal,
    /// Timeout during probe
    Timeout,
}

/// TRUE PRIMAL discovery engine
pub struct PrimalDiscovery {
    config: DiscoveryConfig,
}

impl PrimalDiscovery {
    /// Create a new discovery engine with default configuration
    pub fn new() -> Self {
        Self {
            config: DiscoveryConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: DiscoveryConfig) -> Self {
        Self { config }
    }

    /// Discover all primals in standard locations
    ///
    /// This is TRUE PRIMAL discovery:
    /// - No knowledge of specific primals
    /// - Capability-based queries only
    /// - Unix socket communication
    pub async fn discover_all(&self) -> PrimalResult<Vec<DiscoveredPrimal>> {
        info!("🔍 Starting TRUE PRIMAL discovery (Unix socket scan)");

        let mut discovered = Vec::new();
        let mut socket_paths = Vec::new();

        // Scan all configured directories for Unix sockets
        for dir in &self.config.socket_dirs {
            if let Ok(sockets) = self.scan_directory(dir).await {
                socket_paths.extend(sockets);
            }
        }

        info!("Found {} potential primal sockets", socket_paths.len());

        // Probe each socket to get primal information
        for socket_path in socket_paths {
            match self.probe_socket(&socket_path).await {
                Ok(result) => {
                    if result.status == DiscoveryStatus::Success {
                        if let Some(primal_info) = result.primal_info {
                            // Convert to DiscoveredPrimal
                            discovered.push(DiscoveredPrimal {
                                id: primal_info.primal_id.clone(),
                                instance_id: primal_info.instance_id.clone(),
                                primal_type: primal_info.primal_type.clone(),
                                capabilities: primal_info.capabilities.clone(),
                                endpoint: socket_path.display().to_string(), // Unix socket path
                                health: primal_info.health.clone(),
                                context: PrimalContext {
                                    user_id: "system".to_string(), // Default context
                                    device_id: "local".to_string(),
                                    session_id: "discovery".to_string(),
                                    network_location: crate::traits::NetworkLocation {
                                        ip_address: "127.0.0.1".to_string(),
                                        subnet: None,
                                        network_id: None,
                                        geo_location: None,
                                    },
                                    security_level: crate::traits::SecurityLevel::Standard,
                                    metadata: HashMap::default(),
                                },
                                port_info: None, // Unix sockets don't use ports
                            });

                            info!(
                                "✅ Discovered primal: {} at {}",
                                primal_info.instance_id,
                                socket_path.display()
                            );
                        }
                    } else {
                        debug!(
                            "❌ Socket {} probe failed: {:?}",
                            socket_path.display(),
                            result.status
                        );
                    }
                }
                Err(e) => {
                    warn!("Error probing {}: {}", socket_path.display(), e);
                }
            }
        }

        info!("🎯 Discovery complete: found {} primals", discovered.len());
        Ok(discovered)
    }

    /// Discover primals with a specific capability
    pub async fn discover_by_capability(
        &self,
        capability: &PrimalCapability,
    ) -> PrimalResult<Vec<DiscoveredPrimal>> {
        let all_primals = self.discover_all().await?;

        // Filter by capability
        let matching = all_primals
            .into_iter()
            .filter(|primal| primal.capabilities.contains(capability))
            .collect();

        Ok(matching)
    }

    /// Scan a directory for Unix socket files
    async fn scan_directory(&self, dir: &Path) -> PrimalResult<Vec<PathBuf>> {
        let mut sockets = Vec::new();

        // Check if directory exists
        if !dir.exists() {
            debug!("Directory does not exist: {}", dir.display());
            return Ok(sockets);
        }

        // Read directory entries
        let mut entries = match fs::read_dir(dir).await {
            Ok(entries) => entries,
            Err(e) => {
                debug!("Cannot read directory {}: {}", dir.display(), e);
                return Ok(sockets);
            }
        };

        while let Some(entry) = entries.next_entry().await.ok().flatten() {
            let path = entry.path();

            // Check if it's a socket file
            #[cfg(unix)]
            {
                use std::os::unix::fs::FileTypeExt;
                if let Ok(metadata) = entry.metadata().await
                    && metadata.file_type().is_socket()
                    && self.matches_pattern(&path)
                {
                    sockets.push(path);
                }
            }

            // On non-Unix systems, check if filename matches pattern
            #[cfg(not(unix))]
            {
                if self.matches_pattern(&path) {
                    sockets.push(path);
                }
            }
        }

        Ok(sockets)
    }

    /// Check if a path matches our socket patterns
    fn matches_pattern(&self, path: &Path) -> bool {
        if self.config.socket_patterns.is_empty() {
            return true; // No patterns = match all
        }

        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        for pattern in &self.config.socket_patterns {
            // Simple pattern matching (*.sock, etc.)
            if let Some(suffix) = pattern.strip_prefix('*') {
                if filename.ends_with(suffix) {
                    return true;
                }
            } else if filename == pattern {
                return true;
            }
        }

        false
    }

    /// Probe a Unix socket to get primal information
    ///
    /// This uses a simple JSON-RPC "info" request to discover the primal.
    /// TRUE PRIMAL: no knowledge of what's on the other end!
    async fn probe_socket(&self, socket_path: &Path) -> PrimalResult<DiscoveryResult> {
        debug!("Probing socket: {}", socket_path.display());

        // Try to connect with timeout
        let connect_result = tokio::time::timeout(
            std::time::Duration::from_millis(self.config.probe_timeout_ms),
            UnixStream::connect(socket_path),
        )
        .await;

        let stream = match connect_result {
            Ok(Ok(stream)) => stream,
            Ok(Err(e)) => {
                return Ok(DiscoveryResult {
                    socket_path: socket_path.to_path_buf(),
                    primal_info: None,
                    status: DiscoveryStatus::ProbeFailed(format!("Connection failed: {}", e)),
                });
            }
            Err(_) => {
                return Ok(DiscoveryResult {
                    socket_path: socket_path.to_path_buf(),
                    primal_info: None,
                    status: DiscoveryStatus::Timeout,
                });
            }
        };

        // NOTE(phase2): JSON-RPC "info" request - requires primal info protocol definition
        // For now, this is a placeholder that will be completed with actual JSON-RPC protocol
        drop(stream); // Close connection

        // Placeholder: parse socket name for basic info
        let filename = socket_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        // Example: "squirrel-nat0.sock" -> primal_id: "squirrel", family: "nat0"
        let parts: Vec<&str> = filename.split('-').collect();
        if !parts.is_empty() {
            let primal_id = parts[0].to_string();
            let instance_id = filename.trim_end_matches(".sock").to_string();

            // Placeholder primal info (will be replaced with real JSON-RPC probe)
            let primal_info = PrimalInfo {
                primal_id,
                instance_id,
                primal_type: PrimalType::Custom("unknown".to_string()),
                capabilities: vec![], // Would be populated from JSON-RPC response
                health: PrimalHealth::Healthy,
            };

            return Ok(DiscoveryResult {
                socket_path: socket_path.to_path_buf(),
                primal_info: Some(primal_info),
                status: DiscoveryStatus::Success,
            });
        }

        Ok(DiscoveryResult {
            socket_path: socket_path.to_path_buf(),
            primal_info: None,
            status: DiscoveryStatus::NotAPrimal,
        })
    }
}

impl Default for PrimalDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl PrimalDiscovery {
    /// Exposes [`PrimalDiscovery::probe_socket`] for unit tests (connection failures, etc.).
    pub(crate) async fn probe_socket_for_test(
        &self,
        socket_path: &Path,
    ) -> PrimalResult<DiscoveryResult> {
        self.probe_socket(socket_path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_discovery_config_default() {
        let config = DiscoveryConfig::default();
        assert!(!config.socket_dirs.is_empty());
        assert_eq!(config.probe_timeout_ms, 1000);
        assert!(config.skip_unhealthy);
    }

    #[test]
    fn test_matches_pattern() {
        let discovery = PrimalDiscovery::new();

        // Test wildcard pattern
        assert!(discovery.matches_pattern(Path::new("/tmp/squirrel.sock")));
        assert!(discovery.matches_pattern(Path::new("/tmp/beardog-nat0.sock")));
    }

    #[test]
    fn test_matches_pattern_empty_patterns_matches_all() {
        let discovery = PrimalDiscovery::with_config(DiscoveryConfig {
            socket_patterns: vec![],
            ..DiscoveryConfig::default()
        });
        assert!(discovery.matches_pattern(Path::new("/any/path/file.txt")));
    }

    #[test]
    fn test_matches_pattern_exact_filename() {
        let discovery = PrimalDiscovery::with_config(DiscoveryConfig {
            socket_patterns: vec!["exact.sock".to_string()],
            ..DiscoveryConfig::default()
        });
        assert!(discovery.matches_pattern(Path::new("/tmp/exact.sock")));
        assert!(!discovery.matches_pattern(Path::new("/tmp/other.sock")));
    }

    #[test]
    fn test_matches_pattern_no_suffix_match() {
        let discovery = PrimalDiscovery::new();
        assert!(!discovery.matches_pattern(Path::new("/tmp/readme.md")));
    }

    #[tokio::test]
    async fn test_scan_directory_missing_dir_returns_empty() {
        let dir = TempDir::new().expect("should succeed");
        let missing = dir.path().join("nonexistent_subdir");
        let discovery = PrimalDiscovery::with_config(DiscoveryConfig {
            socket_dirs: vec![missing],
            ..DiscoveryConfig::default()
        });
        let out = discovery.discover_all().await.expect("should succeed");
        assert!(out.is_empty());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_discover_all_finds_listening_unix_socket() {
        let dir = TempDir::new().expect("should succeed");
        let socket_path = dir.path().join("probe-me.sock");
        let _listener = tokio::net::UnixListener::bind(&socket_path).expect("should succeed");

        let discovery = PrimalDiscovery::with_config(DiscoveryConfig {
            socket_dirs: vec![dir.path().to_path_buf()],
            socket_patterns: vec!["*.sock".to_string()],
            probe_timeout_ms: 2000,
            ..DiscoveryConfig::default()
        });

        let found = discovery.discover_all().await.expect("should succeed");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, "probe");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_probe_socket_rejects_non_socket_file() {
        let dir = TempDir::new().expect("should succeed");
        let path = dir.path().join("fake.sock");
        std::fs::write(&path, b"not-a-socket").expect("should succeed");

        let discovery = PrimalDiscovery::new();
        let result = discovery
            .probe_socket_for_test(&path)
            .await
            .expect("should succeed");
        assert!(
            matches!(result.status, DiscoveryStatus::ProbeFailed(_)),
            "expected ProbeFailed, got {:?}",
            result.status
        );
        assert!(result.primal_info.is_none());
    }

    #[tokio::test]
    async fn test_discover_all_no_crash() {
        let discovery = PrimalDiscovery::new();
        // Should not crash even if no sockets found
        let result = discovery.discover_all().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_discover_by_capability() {
        let discovery = PrimalDiscovery::new();
        let capability = PrimalCapability::Authentication {
            methods: vec!["password".to_string()],
        };

        // Should not crash even if nothing found
        let result = discovery.discover_by_capability(&capability).await;
        assert!(result.is_ok());
    }
}
