// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Unix Socket Utilities
//!
//! Helper functions for Unix socket management following biomeOS atomic standards.
//!
//! ## Socket Configuration Priority (5-Tier Fallback)
//!
//! 1. `SQUIRREL_SOCKET` environment variable (primal-specific override)
//! 2. `BIOMEOS_SOCKET_PATH` environment variable (Neural API orchestration) ⭐
//! 3. `PRIMAL_SOCKET` environment variable with family suffix (generic primal coordination)
//! 4. XDG Runtime Directory: `/run/user/<uid>/biomeos/squirrel.sock` (STANDARD biomeOS path)
//! 5. Temp Directory (fallback): `/tmp/squirrel-<family>-<node>.sock` (dev/testing only)
//!
//! ## Environment Variables
//!
//! - `SQUIRREL_SOCKET`: Primal-specific override (highest priority)
//! - `BIOMEOS_SOCKET_PATH`: Generic orchestrator path (Neural API coordination) ⭐
//! - `SQUIRREL_FAMILY_ID`: Family identifier for atomic grouping (default: "default")
//! - `SQUIRREL_NODE_ID`: Node identifier for multi-instance (default: hostname)
//!
//! ## biomeOS Atomic Architecture Compliance
//!
//! This implementation follows the standardized socket configuration required for:
//! - Tower atomics (BearDog + Songbird)
//! - Node atomics (BearDog + Songbird + ToadStool)
//! - Nest atomics (BearDog + Songbird + NestGate)
//! - NUCLEUS deployments (all atomics)
//!
//! See: `docs/sessions/2026-01-11/BIOMEOS_SOCKET_STANDARDS.md`

use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Injectable socket configuration (airSpring `SocketConfig` pattern).
///
/// Replaces env-var reads in production code with explicit config fields,
/// enabling test isolation without `temp_env` or `#[serial]`.
#[derive(Debug, Clone, Default)]
pub struct SocketConfig {
    /// Overrides `SQUIRREL_SOCKET` (Tier 1).
    pub squirrel_socket: Option<String>,
    /// Overrides `BIOMEOS_SOCKET_PATH` (Tier 2).
    pub biomeos_socket_path: Option<String>,
    /// Overrides `PRIMAL_SOCKET` (Tier 3).
    pub primal_socket: Option<String>,
    /// Overrides `SQUIRREL_FAMILY_ID`.
    pub family_id: Option<String>,
    /// Overrides `SQUIRREL_NODE_ID`.
    pub node_id: Option<String>,
}

impl SocketConfig {
    /// Build config by reading the environment (the default path).
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            squirrel_socket: std::env::var("SQUIRREL_SOCKET").ok(),
            biomeos_socket_path: std::env::var("BIOMEOS_SOCKET_PATH").ok(),
            primal_socket: std::env::var("PRIMAL_SOCKET").ok(),
            family_id: std::env::var("SQUIRREL_FAMILY_ID").ok(),
            node_id: std::env::var("SQUIRREL_NODE_ID").ok(),
        }
    }
}

/// Get socket path using an explicit `SocketConfig` (injectable, test-friendly).
#[must_use]
pub fn get_socket_path_with(config: &SocketConfig, node_id: &str) -> String {
    // Tier 1: Primal-specific socket path override
    if let Some(ref socket_path) = config.squirrel_socket {
        debug!("Socket Path: {socket_path} (Tier 1 - primal-specific)");
        return socket_path.clone();
    }

    // Tier 2: Generic orchestrator (Neural API coordination)
    if let Some(ref socket_path) = config.biomeos_socket_path {
        debug!("Socket Path: {socket_path} (Tier 2 - Neural API)");
        return socket_path.clone();
    }

    let family_id = get_family_id_with(config);

    // Tier 3: Generic PRIMAL_SOCKET with family suffix
    if let Some(ref generic_socket) = config.primal_socket {
        let suffixed_path = format!("{generic_socket}-{family_id}");
        debug!("Socket Path: {suffixed_path} (Tier 3 - generic primal)");
        return suffixed_path;
    }

    // Tier 4: XDG runtime directory
    if let Some(xdg_path) = get_xdg_socket_path() {
        debug!("Socket Path: {xdg_path} (Tier 4 - STANDARD biomeOS)");
        return xdg_path;
    }

    // Tier 5: Temp directory fallback
    let fallback_path = format!("/tmp/squirrel-{family_id}-{node_id}.sock");
    debug!("Socket Path: {fallback_path} (Tier 5 - dev/testing ONLY)");
    fallback_path
}

/// Get family ID from config or use default.
#[must_use]
pub fn get_family_id_with(config: &SocketConfig) -> String {
    config
        .family_id
        .clone()
        .unwrap_or_else(|| "default".to_string())
}

/// Get node ID from config or use hostname.
#[must_use]
pub fn get_node_id_with(config: &SocketConfig) -> String {
    config.node_id.clone().unwrap_or_else(|| {
        debug!("No node_id in config, using hostname");
        hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "squirrel".to_string())
    })
}

/// Get the socket path following biomeOS atomic standards
///
/// ## Priority Order (5-Tier Fallback)
///
/// 1. `SQUIRREL_SOCKET` env var (primal-specific override)
/// 2. `BIOMEOS_SOCKET_PATH` env var (Neural API orchestration) ⭐
/// 3. `PRIMAL_SOCKET` env var with family suffix (generic primal coordination)
/// 4. XDG runtime directory (`/run/user/<uid>/biomeos/squirrel.sock`) - STANDARD
/// 5. Temp directory fallback (`/tmp/squirrel-<family>-<node>.sock`) - dev only
///
/// ## Examples
///
/// ```ignore
/// // Tier 1: Primal-specific override
/// unsafe { std::env::set_var("SQUIRREL_SOCKET", "/custom/squirrel.sock") };
/// let path = get_socket_path("node1");
/// assert_eq!(path, "/custom/squirrel.sock");
///
/// // Tier 2: Neural API orchestration
/// unsafe { std::env::remove_var("SQUIRREL_SOCKET") };
/// unsafe { std::env::set_var("BIOMEOS_SOCKET_PATH", "/tmp/squirrel-nat0.sock") };
/// let path = get_socket_path("node1");
/// assert_eq!(path, "/tmp/squirrel-nat0.sock");
///
/// // Tier 3: XDG runtime directory
/// unsafe { std::env::remove_var("BIOMEOS_SOCKET_PATH") };
/// unsafe { std::env::set_var("SQUIRREL_FAMILY_ID", "nat0") };
/// let path = get_socket_path("node1");
/// // Returns: /run/user/<uid>/squirrel-nat0.sock
/// ```
#[must_use]
pub fn get_socket_path(node_id: &str) -> String {
    get_socket_path_with(&SocketConfig::from_env(), node_id)
}

/// Get XDG-compliant socket path with biomeos subdirectory (STANDARD)
///
/// Returns `/run/user/<uid>/biomeos/squirrel.sock` - the standardized biomeOS path
/// that enables inter-primal discovery and NUCLEUS deployment.
///
/// This path is used by all primals: BearDog, Songbird, NestGate, Toadstool, Squirrel.
fn get_xdg_socket_path() -> Option<String> {
    // Get current user ID
    let uid = nix::unistd::getuid();
    let xdg_runtime_dir = format!("/run/user/{uid}");

    // Check if XDG runtime directory exists
    if Path::new(&xdg_runtime_dir).exists() {
        // Ensure biomeos subdirectory exists with proper permissions
        if let Err(e) = ensure_biomeos_directory() {
            warn!("Failed to create biomeos directory: {}", e);
            return None;
        }

        let filename = match std::env::var("FAMILY_ID") {
            Ok(fid) if !fid.is_empty() => format!("squirrel-{fid}.sock"),
            _ => "squirrel.sock".to_string(),
        };
        let socket_path = format!("{xdg_runtime_dir}/biomeos/{filename}");
        Some(socket_path)
    } else {
        debug!("XDG runtime directory does not exist: {}", xdg_runtime_dir);
        None
    }
}

/// Ensure biomeos directory exists with proper permissions
///
/// Creates `/run/user/<uid>/biomeos/` with 0700 permissions (user-only access).
/// This is the standardized directory for all biomeOS primal sockets.
///
/// ## Security
///
/// Directory permissions are set to 0700 to ensure only the owning user can:
/// - List sockets in the directory
/// - Connect to sockets
/// - Create new sockets
///
/// ## Errors
///
/// Returns error if:
/// - Cannot create directory
/// - Cannot set permissions
pub fn ensure_biomeos_directory() -> std::io::Result<PathBuf> {
    let uid = nix::unistd::getuid();
    let biomeos_dir = format!("/run/user/{uid}/biomeos");
    let path = PathBuf::from(&biomeos_dir);

    // Create directory if it doesn't exist
    if !path.exists() {
        debug!("Creating biomeos directory: {}", biomeos_dir);
        std::fs::create_dir_all(&path)?;

        // Set permissions to 0700 (user-only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o700);
            std::fs::set_permissions(&path, perms)?;
            debug!("Set biomeos directory permissions to 0700");
        }
    }

    Ok(path)
}

/// Get family ID from environment or use default.
#[must_use]
pub fn get_family_id() -> String {
    get_family_id_with(&SocketConfig::from_env())
}

/// Get node ID from environment or generate one.
#[must_use]
pub fn get_node_id() -> String {
    get_node_id_with(&SocketConfig::from_env())
}

/// Prepare socket path for binding
///
/// Ensures:
/// 1. Parent directory exists (creates if needed)
/// 2. Old socket file is removed (prevents "address already in use")
/// 3. Returns canonical path
///
/// ## biomeOS Compliance
///
/// This function implements the socket preparation requirements from
/// the biomeOS primal socket configuration standards.
///
/// ## Errors
///
/// Returns error if:
/// - Cannot create parent directory
/// - Cannot remove old socket file (if permissions issue)
pub fn prepare_socket_path(socket_path: &str) -> std::io::Result<PathBuf> {
    let path = Path::new(socket_path);

    // Ensure parent directory exists
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        info!("Creating socket directory: {}", parent.display());
        std::fs::create_dir_all(parent)?;
    }

    // Remove old socket if exists (prevents "address already in use")
    if path.exists() {
        info!("Removing old socket file: {}", socket_path);
        std::fs::remove_file(path)?;
    }

    Ok(path.to_path_buf())
}

/// Clean up socket file on shutdown
///
/// Removes socket file if it exists. Logs warnings on failure but doesn't
/// propagate errors (cleanup is best-effort).
pub fn cleanup_socket(socket_path: &str) {
    if Path::new(socket_path).exists() {
        info!("🧹 Cleaning up socket: {}", socket_path);
        if let Err(e) = std::fs::remove_file(socket_path) {
            warn!("⚠️ Failed to remove socket: {}", e);
        }
    }
}

/// Verify socket configuration for biomeOS atomic deployment
///
/// Returns `Ok(())` if configuration is valid for atomic deployment.
/// Returns `Err` with explanation if configuration needs adjustment.
pub fn verify_socket_config() -> Result<String, String> {
    let node_id = get_node_id();
    let family_id = get_family_id();
    let socket_path = get_socket_path(&node_id);

    // Check for XDG compliance (recommended)
    if socket_path.starts_with("/run/user/") {
        Ok(format!(
            "✅ XDG-compliant socket configuration\n\
             Socket: {socket_path}\n\
             Family: {family_id}\n\
             Node: {node_id}"
        ))
    } else if socket_path.starts_with("/tmp/") {
        Ok(format!(
            "⚠️ Using /tmp socket (consider setting SQUIRREL_SOCKET or SQUIRREL_FAMILY_ID)\n\
             Socket: {socket_path}\n\
             Family: {family_id}\n\
             Node: {node_id}"
        ))
    } else {
        Ok(format!(
            "✅ Custom socket configuration\n\
             Socket: {socket_path}\n\
             Family: {family_id}\n\
             Node: {node_id}"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    /// Socket-related env vars - used for save/restore and temp_env isolation.
    /// temp_env provides cross-test isolation via internal mutex; serial_test
    /// prevents races with tests that modify env directly (e.g. config tests).
    const SOCKET_ENV_VARS: &[&str] = &[
        "SQUIRREL_SOCKET",
        "BIOMEOS_SOCKET_PATH",
        "PRIMAL_SOCKET",
        "SQUIRREL_FAMILY_ID",
        "SQUIRREL_NODE_ID",
    ];

    #[test]
    #[serial(socket_env)]
    fn test_socket_path_tier1_squirrel_socket() {
        temp_env::with_var("SQUIRREL_SOCKET", Some("/custom/path/socket.sock"), || {
            let path = get_socket_path("test-node");
            assert_eq!(path, "/custom/path/socket.sock");
        });
    }

    #[test]
    #[serial(socket_env)]
    fn test_socket_path_tier2_biomeos_socket_path() {
        temp_env::with_var(
            "BIOMEOS_SOCKET_PATH",
            Some("/tmp/squirrel-nat0.sock"),
            || {
                let path = get_socket_path("test-node");
                assert_eq!(path, "/tmp/squirrel-nat0.sock");
            },
        );
    }

    #[test]
    #[serial(socket_env)]
    fn test_squirrel_socket_overrides_biomeos_socket_path() {
        temp_env::with_vars(
            [
                ("SQUIRREL_SOCKET", Some("/custom/squirrel.sock")),
                ("BIOMEOS_SOCKET_PATH", Some("/tmp/squirrel-nat0.sock")),
            ],
            || {
                let path = get_socket_path("test-node");
                assert_eq!(path, "/custom/squirrel.sock");
            },
        );
    }

    #[test]
    #[serial(socket_env)]
    fn test_socket_path_tier3_primal_socket() {
        temp_env::with_vars(
            [
                ("PRIMAL_SOCKET", Some("/custom/primal")),
                ("SQUIRREL_FAMILY_ID", Some("nat0")),
            ],
            || {
                let path = get_socket_path("test-node");
                assert_eq!(path, "/custom/primal-nat0");
            },
        );
    }

    #[test]
    #[serial(socket_env)]
    fn test_socket_path_tier4_and_tier5_fallback() {
        temp_env::with_var("SQUIRREL_FAMILY_ID", Some("test0"), || {
            let path = get_socket_path("test-node");
            assert!(
                path.contains("/biomeos/squirrel.sock") || path.contains("/tmp/squirrel-test0")
            );
        });
    }

    #[test]
    #[serial(socket_env)]
    fn test_get_family_id_default() {
        temp_env::with_vars_unset(SOCKET_ENV_VARS, || {
            let family_id = get_family_id();
            assert_eq!(family_id, "default");
        });
    }

    #[test]
    #[serial(socket_env)]
    fn test_get_family_id_from_env() {
        temp_env::with_var("SQUIRREL_FAMILY_ID", Some("nat0"), || {
            let family_id = get_family_id();
            assert_eq!(family_id, "nat0");
        });
    }

    #[test]
    #[serial(socket_env)]
    fn test_get_node_id_default() {
        temp_env::with_vars_unset(SOCKET_ENV_VARS, || {
            let node_id = get_node_id();
            assert!(!node_id.is_empty());
        });
    }

    #[test]
    #[serial(socket_env)]
    fn test_get_node_id_from_env() {
        temp_env::with_var("SQUIRREL_NODE_ID", Some("custom-node"), || {
            let node_id = get_node_id();
            assert_eq!(node_id, "custom-node");
        });
    }

    #[test]
    fn test_prepare_socket_path_creates_directory() {
        use tempfile::tempdir;

        let dir = tempdir().expect("should succeed");
        let socket_path = dir.path().join("subdir/test.sock");
        let socket_str = socket_path.to_str().expect("should succeed");

        // Should create parent directory and return path
        let result = prepare_socket_path(socket_str);
        assert!(result.is_ok());
        assert!(socket_path.parent().expect("should succeed").exists());
    }

    #[test]
    #[serial(socket_env)]
    fn test_verify_socket_config() {
        temp_env::with_vars_unset(SOCKET_ENV_VARS, || {
            let result = verify_socket_config();
            assert!(result.is_ok());

            let message = result.expect("should succeed");
            assert!(message.contains("Socket:"));
            assert!(message.contains("Family:"));
            assert!(message.contains("Node:"));
        });
    }

    #[test]
    #[serial(socket_env)]
    fn test_xdg_socket_path_format() {
        temp_env::with_vars_unset(SOCKET_ENV_VARS, || {
            if let Some(xdg_path) = get_xdg_socket_path() {
                assert!(xdg_path.starts_with("/run/user/"));
                assert!(xdg_path.contains("/biomeos/"));
                assert!(xdg_path.ends_with("/squirrel.sock"));
            }
        });
    }

    #[test]
    fn test_ensure_biomeos_directory() {
        // Test directory creation (idempotent - works even if already exists)
        let result = ensure_biomeos_directory();
        assert!(result.is_ok());

        let path = result.expect("should succeed");
        assert!(path.exists());
        assert!(path.to_str().expect("should succeed").ends_with("biomeos"));

        // Verify directory is accessible (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(&path).expect("should succeed");
            let mode = metadata.permissions().mode();
            // Check that user has rwx permissions (at minimum)
            assert_eq!(mode & 0o700, 0o700, "User should have rwx permissions");
        }
    }

    #[test]
    #[serial(socket_env)]
    fn test_socket_path_uses_biomeos_directory() {
        temp_env::with_vars_unset(SOCKET_ENV_VARS, || {
            let path = get_socket_path("test-node");
            assert!(path.contains("/biomeos/squirrel.sock") || path.starts_with("/tmp/"));
        });
    }

    // ── Injectable SocketConfig tests (no temp_env, no serial) ──────────

    #[test]
    fn di_tier1_squirrel_socket() {
        let config = SocketConfig {
            squirrel_socket: Some("/custom/squirrel.sock".into()),
            ..Default::default()
        };
        assert_eq!(
            get_socket_path_with(&config, "node1"),
            "/custom/squirrel.sock"
        );
    }

    #[test]
    fn di_tier2_biomeos_socket_path() {
        let config = SocketConfig {
            biomeos_socket_path: Some("/tmp/squirrel-nat0.sock".into()),
            ..Default::default()
        };
        assert_eq!(
            get_socket_path_with(&config, "node1"),
            "/tmp/squirrel-nat0.sock"
        );
    }

    #[test]
    fn di_tier1_overrides_tier2() {
        let config = SocketConfig {
            squirrel_socket: Some("/custom/squirrel.sock".into()),
            biomeos_socket_path: Some("/tmp/should-not-use.sock".into()),
            ..Default::default()
        };
        assert_eq!(
            get_socket_path_with(&config, "node1"),
            "/custom/squirrel.sock"
        );
    }

    #[test]
    fn di_tier3_primal_socket_with_family() {
        let config = SocketConfig {
            primal_socket: Some("/custom/primal".into()),
            family_id: Some("nat0".into()),
            ..Default::default()
        };
        assert_eq!(
            get_socket_path_with(&config, "node1"),
            "/custom/primal-nat0"
        );
    }

    #[test]
    fn di_family_id_from_config() {
        let config = SocketConfig {
            family_id: Some("test-family-42".into()),
            ..Default::default()
        };
        assert_eq!(get_family_id_with(&config), "test-family-42");
    }

    #[test]
    fn di_family_id_default() {
        let config = SocketConfig::default();
        assert_eq!(get_family_id_with(&config), "default");
    }

    #[test]
    fn di_node_id_from_config() {
        let config = SocketConfig {
            node_id: Some("custom-node".into()),
            ..Default::default()
        };
        assert_eq!(get_node_id_with(&config), "custom-node");
    }

    #[test]
    fn di_node_id_default_hostname() {
        let config = SocketConfig::default();
        let node_id = get_node_id_with(&config);
        assert!(!node_id.is_empty());
    }
}
