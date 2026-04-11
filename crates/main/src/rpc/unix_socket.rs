// SPDX-License-Identifier: AGPL-3.0-or-later
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
    /// Overrides `BIOMEOS_INSECURE` (BTSP guard).
    pub biomeos_insecure: Option<bool>,
}

impl SocketConfig {
    /// Build config by reading the environment (the default path).
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            squirrel_socket: std::env::var("SQUIRREL_SOCKET").ok(),
            biomeos_socket_path: std::env::var("BIOMEOS_SOCKET_PATH").ok(),
            primal_socket: std::env::var("PRIMAL_SOCKET").ok(),
            family_id: std::env::var("SQUIRREL_FAMILY_ID")
                .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
                .or_else(|_| std::env::var("FAMILY_ID"))
                .ok(),
            node_id: std::env::var("SQUIRREL_NODE_ID").ok(),
            biomeos_insecure: std::env::var("BIOMEOS_INSECURE")
                .ok()
                .map(|v| v == "1" || v == "true"),
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

    // Tier 4: XDG runtime directory (family-scoped per PRIMAL_SELF_KNOWLEDGE_STANDARD)
    if let Some(xdg_path) = get_xdg_socket_path(&family_id) {
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
            .unwrap_or_else(|| crate::niche::PRIMAL_ID.to_string())
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
fn get_xdg_socket_path(family_id: &str) -> Option<String> {
    let uid = nix::unistd::getuid();
    let xdg_runtime_dir = format!("/run/user/{uid}");

    if Path::new(&xdg_runtime_dir).exists() {
        if let Err(e) = ensure_biomeos_directory() {
            warn!("Failed to create biomeos directory: {}", e);
            return None;
        }

        let family_scoped = !family_id.is_empty() && family_id != "default";
        let filename = if family_scoped {
            format!("squirrel-{family_id}.sock")
        } else {
            "squirrel.sock".to_string()
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

/// PRIMAL_IPC_PROTOCOL: capability-domain symlink `ai.sock` → `<socket basename>` in the same directory.
#[cfg(unix)]
const CAPABILITY_DOMAIN_SYMLINK_NAME: &str = "ai.sock";

/// True if `ai_sock` is a symlink whose target lives in the same directory as `socket_path`
/// (relative basename or absolute path with the same parent). Used to clean stale PRIMAL_IPC symlinks
/// without deleting unrelated `ai.sock` entries.
#[cfg(unix)]
fn capability_symlink_points_to_same_directory(socket_path: &Path, ai_sock: &Path) -> bool {
    let Ok(meta) = std::fs::symlink_metadata(ai_sock) else {
        return false;
    };
    if !meta.file_type().is_symlink() {
        return false;
    }
    let Ok(target) = std::fs::read_link(ai_sock) else {
        return false;
    };
    let Some(parent) = socket_path.parent() else {
        return false;
    };
    if target.is_absolute() {
        target.parent() == Some(parent)
    } else {
        use std::path::Component;
        target.components().count() == 1
            && matches!(target.components().next(), Some(Component::Normal(_)))
    }
}

/// Remove stale `ai.sock` before rebinding the filesystem socket (next startup / prepare).
#[cfg(unix)]
fn remove_stale_capability_domain_symlink(socket_path: &Path) {
    let Some(parent) = socket_path.parent() else {
        return;
    };
    let ai_sock = parent.join(CAPABILITY_DOMAIN_SYMLINK_NAME);
    if capability_symlink_points_to_same_directory(socket_path, &ai_sock)
        && let Err(e) = std::fs::remove_file(&ai_sock)
    {
        warn!(
            "Could not remove stale capability symlink {}: {}",
            ai_sock.display(),
            e
        );
    }
}

/// After binding the filesystem socket, create `ai.sock` → `<basename>` (PRIMAL_IPC_PROTOCOL).
/// Removes an existing `ai.sock` in that directory first. Non-Unix: no-op `Ok(())`.
#[cfg(unix)]
pub fn try_create_capability_domain_symlink(filesystem_socket_path: &str) -> std::io::Result<()> {
    let path = Path::new(filesystem_socket_path);
    let Some(parent) = path.parent() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "socket path has no parent directory",
        ));
    };
    let Some(target_name) = path.file_name() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "socket path has no file name",
        ));
    };
    let link_path = parent.join(CAPABILITY_DOMAIN_SYMLINK_NAME);
    if link_path.exists() {
        std::fs::remove_file(&link_path)?;
    }
    std::os::unix::fs::symlink(target_name, &link_path)?;
    info!(
        "Capability-domain symlink {} → {} (PRIMAL_IPC_PROTOCOL)",
        link_path.display(),
        target_name.to_string_lossy()
    );
    Ok(())
}

#[cfg(not(unix))]
pub fn try_create_capability_domain_symlink(_filesystem_socket_path: &str) -> std::io::Result<()> {
    Ok(())
}

/// Best-effort removal of `ai.sock` when it is our same-directory capability symlink.
#[cfg(unix)]
pub fn cleanup_capability_domain_symlink(filesystem_socket_path: &str) {
    let path = Path::new(filesystem_socket_path);
    let Some(parent) = path.parent() else {
        return;
    };
    let ai_sock = parent.join(CAPABILITY_DOMAIN_SYMLINK_NAME);
    if capability_symlink_points_to_same_directory(path, &ai_sock)
        && let Err(e) = std::fs::remove_file(&ai_sock)
        && e.kind() != std::io::ErrorKind::NotFound
    {
        warn!(
            "Could not remove capability symlink {}: {}",
            ai_sock.display(),
            e
        );
    }
}

#[cfg(not(unix))]
pub fn cleanup_capability_domain_symlink(_filesystem_socket_path: &str) {}

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

    #[cfg(unix)]
    remove_stale_capability_domain_symlink(path);

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
    cleanup_capability_domain_symlink(socket_path);
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

/// BTSP Protocol Standard §Security Model: refuse to start when both
/// `FAMILY_ID` (non-default) and `BIOMEOS_INSECURE=1` are set.
///
/// Production mode (`FAMILY_ID` set) requires BTSP authentication on every
/// socket connection. `BIOMEOS_INSECURE=1` skips BTSP and is only valid
/// in development (no `FAMILY_ID`). Setting both is a configuration error.
///
/// Checks `SQUIRREL_FAMILY_ID` first, then falls back to `FAMILY_ID`
/// (following the primal-specific env var precedence from
/// `PRIMAL_SELF_KNOWLEDGE_STANDARD.md` §4).
///
/// # Errors
///
/// Returns `Err` with a human-readable message when both are set.
pub fn validate_insecure_guard() -> Result<(), String> {
    let config = SocketConfig::from_env();
    let has_family = config
        .family_id
        .as_deref()
        .is_some_and(|v| !v.is_empty() && v != "default");
    validate_insecure_guard_with(has_family, config.biomeos_insecure.unwrap_or(false))
}

/// Injectable variant for testing without env var side effects.
pub fn validate_insecure_guard_with(has_family: bool, insecure: bool) -> Result<(), String> {
    if has_family && insecure {
        return Err("FATAL: FAMILY_ID and BIOMEOS_INSECURE=1 cannot coexist. \
             Production mode (FAMILY_ID set) requires BTSP authentication. \
             Remove BIOMEOS_INSECURE to run in production, or unset FAMILY_ID for development."
            .to_owned());
    }
    Ok(())
}

#[cfg(test)]
#[path = "unix_socket_tests.rs"]
mod tests;
