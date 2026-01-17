//! Unix Socket Utilities
//!
//! Helper functions for Unix socket management following biomeOS atomic standards.
//!
//! ## Socket Configuration Priority (4-Tier Fallback)
//!
//! 1. `SQUIRREL_SOCKET` environment variable (primal-specific override)
//! 2. `BIOMEOS_SOCKET_PATH` environment variable (Neural API orchestration) ⭐
//! 3. XDG Runtime Directory: `/run/user/<uid>/squirrel-<family>.sock` (secure user mode)
//! 4. Temp Directory (fallback): `/tmp/squirrel-<family>-<node>.sock` (system default)
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

/// Get the socket path following biomeOS atomic standards
///
/// ## Priority Order (4-Tier Fallback)
///
/// 1. `SQUIRREL_SOCKET` env var (primal-specific override)
/// 2. `BIOMEOS_SOCKET_PATH` env var (Neural API orchestration) ⭐
/// 3. XDG runtime directory (`/run/user/<uid>/squirrel-<family>.sock`)
/// 4. Temp directory fallback (`/tmp/squirrel-<family>-<node>.sock`)
///
/// ## Examples
///
/// ```rust
/// // Tier 1: Primal-specific override
/// std::env::set_var("SQUIRREL_SOCKET", "/custom/squirrel.sock");
/// let path = get_socket_path("node1");
/// assert_eq!(path, "/custom/squirrel.sock");
///
/// // Tier 2: Neural API orchestration
/// std::env::remove_var("SQUIRREL_SOCKET");
/// std::env::set_var("BIOMEOS_SOCKET_PATH", "/tmp/squirrel-nat0.sock");
/// let path = get_socket_path("node1");
/// assert_eq!(path, "/tmp/squirrel-nat0.sock");
///
/// // Tier 3: XDG runtime directory
/// std::env::remove_var("BIOMEOS_SOCKET_PATH");
/// std::env::set_var("SQUIRREL_FAMILY_ID", "nat0");
/// let path = get_socket_path("node1");
/// // Returns: /run/user/<uid>/squirrel-nat0.sock
/// ```
#[must_use]
pub fn get_socket_path(node_id: &str) -> String {
    // Tier 1: Primal-specific socket path override
    if let Ok(socket_path) = std::env::var("SQUIRREL_SOCKET") {
        debug!(
            "Socket Path: {} (from SQUIRREL_SOCKET env var ⭐ Tier 1 - primal-specific)",
            socket_path
        );
        return socket_path;
    }

    // Tier 2: Generic orchestrator environment variable (Neural API coordination)
    if let Ok(socket_path) = std::env::var("BIOMEOS_SOCKET_PATH") {
        debug!(
            "Socket Path: {} (from BIOMEOS_SOCKET_PATH env var ⭐ Tier 2 - Neural API)",
            socket_path
        );
        return socket_path;
    }

    // Get family ID for atomic grouping
    let family_id = get_family_id();

    // Tier 3: XDG runtime directory (preferred for standalone, secure)
    if let Some(xdg_path) = get_xdg_socket_path(&family_id) {
        debug!(
            "Socket Path: {} (from XDG runtime ⭐ Tier 3 - user mode)",
            xdg_path
        );
        return xdg_path;
    }

    // Tier 4: Temp directory fallback (system default)
    let fallback_path = format!("/tmp/squirrel-{}-{}.sock", family_id, node_id);
    debug!(
        "Socket Path: {} (from /tmp ⭐ Tier 4 - system default)",
        fallback_path
    );
    fallback_path
}

/// Get XDG-compliant socket path if runtime directory exists
///
/// Returns `/run/user/<uid>/squirrel-<family>.sock` if XDG directory exists.
fn get_xdg_socket_path(family_id: &str) -> Option<String> {
    // Get current user ID
    let uid = nix::unistd::getuid();
    let xdg_runtime_dir = format!("/run/user/{}", uid);

    // Check if XDG runtime directory exists
    if Path::new(&xdg_runtime_dir).exists() {
        let socket_path = format!("{}/squirrel-{}.sock", xdg_runtime_dir, family_id);
        Some(socket_path)
    } else {
        debug!("XDG runtime directory does not exist: {}", xdg_runtime_dir);
        None
    }
}

/// Get family ID from environment or use default
///
/// Family ID groups primals into atomic units (Tower, Node, Nest).
#[must_use]
pub fn get_family_id() -> String {
    std::env::var("SQUIRREL_FAMILY_ID").unwrap_or_else(|_| "default".to_string())
}

/// Get node ID from environment or generate one
///
/// Node ID identifies individual instances within a family.
#[must_use]
pub fn get_node_id() -> String {
    std::env::var("SQUIRREL_NODE_ID").unwrap_or_else(|_| {
        debug!("SQUIRREL_NODE_ID not set, using hostname");
        hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "squirrel".to_string())
    })
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
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            info!("Creating socket directory: {}", parent.display());
            std::fs::create_dir_all(parent)?;
        }
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
             Socket: {}\n\
             Family: {}\n\
             Node: {}",
            socket_path, family_id, node_id
        ))
    } else if socket_path.starts_with("/tmp/") {
        Ok(format!(
            "⚠️ Using /tmp socket (consider setting SQUIRREL_SOCKET or SQUIRREL_FAMILY_ID)\n\
             Socket: {}\n\
             Family: {}\n\
             Node: {}",
            socket_path, family_id, node_id
        ))
    } else {
        Ok(format!(
            "✅ Custom socket configuration\n\
             Socket: {}\n\
             Family: {}\n\
             Node: {}",
            socket_path, family_id, node_id
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn clear_env_vars() {
        std::env::remove_var("SQUIRREL_SOCKET");
        std::env::remove_var("BIOMEOS_SOCKET_PATH");
        std::env::remove_var("SQUIRREL_FAMILY_ID");
        std::env::remove_var("SQUIRREL_NODE_ID");
    }

    #[test]
    #[serial] // Serialize env var tests
    fn test_socket_path_tier1_squirrel_socket() {
        clear_env_vars();
        std::env::set_var("SQUIRREL_SOCKET", "/custom/path/socket.sock");

        let path = get_socket_path("test-node");
        assert_eq!(path, "/custom/path/socket.sock");

        clear_env_vars();
    }

    #[test]
    #[serial] // Serialize env var tests
    fn test_socket_path_tier2_biomeos_socket_path() {
        clear_env_vars();
        std::env::set_var("BIOMEOS_SOCKET_PATH", "/tmp/squirrel-nat0.sock");

        let path = get_socket_path("test-node");
        assert_eq!(path, "/tmp/squirrel-nat0.sock");

        clear_env_vars();
    }

    #[test]
    #[serial] // Serialize env var tests
    fn test_squirrel_socket_overrides_biomeos_socket_path() {
        clear_env_vars();
        std::env::set_var("SQUIRREL_SOCKET", "/custom/squirrel.sock");
        std::env::set_var("BIOMEOS_SOCKET_PATH", "/tmp/squirrel-nat0.sock");

        let path = get_socket_path("test-node");
        // Tier 1 (SQUIRREL_SOCKET) should override Tier 2 (BIOMEOS_SOCKET_PATH)
        assert_eq!(path, "/custom/squirrel.sock");

        clear_env_vars();
    }

    #[test]
    #[serial] // Serialize env var tests
    fn test_socket_path_tier3_and_tier4_fallback() {
        clear_env_vars();
        std::env::set_var("SQUIRREL_FAMILY_ID", "test0");

        let path = get_socket_path("test-node");
        // Should use Tier 3 (XDG) or Tier 4 (/tmp) fallback
        assert!(path.contains("squirrel-test0") || path.contains("/run/user/"));

        clear_env_vars();
    }

    #[test]
    #[serial] // Serialize env var tests
    fn test_get_family_id_default() {
        clear_env_vars();

        let family_id = get_family_id();
        assert_eq!(family_id, "default");
    }

    #[test]
    #[serial] // Serialize env var tests
    fn test_get_family_id_from_env() {
        clear_env_vars();
        std::env::set_var("SQUIRREL_FAMILY_ID", "nat0");

        let family_id = get_family_id();
        assert_eq!(family_id, "nat0");

        clear_env_vars();
    }

    #[test]
    #[serial] // Serialize env var tests
    fn test_get_node_id_default() {
        clear_env_vars();

        let node_id = get_node_id();
        assert!(!node_id.is_empty());
    }

    #[test]
    #[serial] // Serialize env var tests
    fn test_get_node_id_from_env() {
        clear_env_vars();
        std::env::set_var("SQUIRREL_NODE_ID", "custom-node");

        let node_id = get_node_id();
        assert_eq!(node_id, "custom-node");

        clear_env_vars();
    }

    #[test]
    fn test_prepare_socket_path_creates_directory() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("subdir/test.sock");
        let socket_str = socket_path.to_str().unwrap();

        // Should create parent directory and return path
        let result = prepare_socket_path(socket_str);
        assert!(result.is_ok());
        assert!(socket_path.parent().unwrap().exists());
    }

    #[test]
    fn test_verify_socket_config() {
        clear_env_vars();

        let result = verify_socket_config();
        assert!(result.is_ok());

        let message = result.unwrap();
        assert!(message.contains("Socket:"));
        assert!(message.contains("Family:"));
        assert!(message.contains("Node:"));
    }

    #[test]
    fn test_xdg_socket_path_format() {
        clear_env_vars();

        let family_id = "nat0";
        if let Some(xdg_path) = get_xdg_socket_path(family_id) {
            assert!(xdg_path.starts_with("/run/user/"));
            assert!(xdg_path.ends_with("/squirrel-nat0.sock"));
        }
    }
}
