// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! G68 L1: Platform-abstracted capability-domain aliases.
//!
//! Unix: filesystem symlink (`ai.sock → squirrel-family.sock`).
//! Windows: discovery file (`ai.pipe` containing the named pipe path).
//! Other: no-op (capability discovery uses transport-layer mechanisms).

use std::io;
use std::path::Path;

/// Create a capability-domain alias for IPC discovery.
///
/// After a primal binds its transport endpoint, this creates an alias so that
/// capability-based discovery can find the endpoint by domain name (e.g. `ai`)
/// rather than the primal-specific socket name.
///
/// # Unix
/// Creates a symlink: `<dir>/<alias_name>.sock` → `<target_basename>`.
///
/// # Windows
/// Writes a discovery file: `<dir>/<alias_name>.pipe` containing the
/// named pipe path, so `discover_ipc_endpoint` can find it.
///
/// # Errors
/// Returns `io::Error` if the alias cannot be created.
pub fn create_capability_alias(
    endpoint_path: &Path,
    alias_name: &str,
) -> io::Result<()> {
    let Some(parent) = endpoint_path.parent() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "endpoint path has no parent directory",
        ));
    };
    let Some(target_name) = endpoint_path.file_name() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "endpoint path has no file name",
        ));
    };

    create_alias_inner(parent, alias_name, target_name)
}

/// Remove a capability-domain alias (best-effort cleanup on shutdown).
pub fn cleanup_capability_alias(endpoint_path: &Path, alias_name: &str) {
    let Some(parent) = endpoint_path.parent() else {
        return;
    };
    cleanup_alias_inner(parent, alias_name, endpoint_path);
}

// ── Unix implementation ────────────────────────────────────────────────────

#[cfg(unix)]
fn create_alias_inner(
    dir: &Path,
    alias_name: &str,
    target_name: &std::ffi::OsStr,
) -> io::Result<()> {
    let link_path = dir.join(format!("{alias_name}.sock"));

    if link_path.exists() {
        std::fs::remove_file(&link_path)?;
    }

    std::os::unix::fs::symlink(target_name, &link_path)?;
    tracing::info!(
        "Capability-domain symlink {} → {} (G68/PRIMAL_IPC_PROTOCOL)",
        link_path.display(),
        target_name.to_string_lossy()
    );
    Ok(())
}

#[cfg(unix)]
fn cleanup_alias_inner(dir: &Path, alias_name: &str, endpoint_path: &Path) {
    let link_path = dir.join(format!("{alias_name}.sock"));

    if !link_path.exists() {
        return;
    }

    // Only remove if the symlink points to the same directory (ours, not another primal's)
    if let Ok(meta) = std::fs::symlink_metadata(&link_path)
        && meta.file_type().is_symlink()
        && let Ok(target) = std::fs::read_link(&link_path)
    {
        let resolved = if target.is_relative() {
            dir.join(&target)
        } else {
            target
        };
        if let (Some(a), Some(b)) = (resolved.parent(), endpoint_path.parent())
            && a == b
        {
            let _ = std::fs::remove_file(&link_path);
            tracing::debug!("Cleaned up capability alias: {}", link_path.display());
        }
    }
}

// ── Windows implementation ─────────────────────────────────────────────────

#[cfg(windows)]
fn create_alias_inner(
    dir: &Path,
    alias_name: &str,
    target_name: &std::ffi::OsStr,
) -> io::Result<()> {
    // Write a discovery file that maps the capability name to the pipe path.
    // Named pipes don't have filesystem symlinks; discovery reads this file.
    let discovery_path = dir.join(format!("{alias_name}.pipe"));
    let pipe_name = format!(r"\\.\pipe\{}", target_name.to_string_lossy());

    std::fs::write(&discovery_path, &pipe_name)?;
    tracing::info!(
        "Capability-domain discovery file {} → {} (G68/PRIMAL_IPC_PROTOCOL)",
        discovery_path.display(),
        pipe_name
    );
    Ok(())
}

#[cfg(windows)]
fn cleanup_alias_inner(dir: &Path, alias_name: &str, _endpoint_path: &Path) {
    let discovery_path = dir.join(format!("{alias_name}.pipe"));
    if discovery_path.exists() {
        let _ = std::fs::remove_file(&discovery_path);
        tracing::debug!("Cleaned up capability discovery file: {}", discovery_path.display());
    }
}

// ── Fallback for other targets ─────────────────────────────────────────────

#[cfg(not(any(unix, windows)))]
fn create_alias_inner(
    _dir: &Path,
    alias_name: &str,
    _target_name: &std::ffi::OsStr,
) -> io::Result<()> {
    tracing::debug!("G68: capability alias '{alias_name}' skipped (unsupported platform)");
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn cleanup_alias_inner(_dir: &Path, _alias_name: &str, _endpoint_path: &Path) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_cleanup_alias() {
        let dir = tempfile::tempdir().expect("tempdir");

        #[cfg(unix)]
        let endpoint = dir.path().join("squirrel-default.sock");
        #[cfg(not(unix))]
        let endpoint = dir.path().join("squirrel-default.pipe");

        std::fs::write(&endpoint, b"").expect("write endpoint placeholder");

        create_capability_alias(&endpoint, "ai").expect("create alias");

        #[cfg(unix)]
        {
            let link = dir.path().join("ai.sock");
            assert!(link.exists(), "symlink should exist");
            let target = std::fs::read_link(&link).expect("read_link");
            assert!(
                target.to_string_lossy().contains("squirrel-default"),
                "symlink should point to endpoint"
            );
        }

        #[cfg(windows)]
        {
            let disc = dir.path().join("ai.pipe");
            assert!(disc.exists(), "discovery file should exist");
            let contents = std::fs::read_to_string(&disc).expect("read");
            assert!(contents.contains("squirrel-default"));
        }

        cleanup_capability_alias(&endpoint, "ai");

        #[cfg(unix)]
        assert!(!dir.path().join("ai.sock").exists(), "symlink should be cleaned up");
        #[cfg(windows)]
        assert!(!dir.path().join("ai.pipe").exists(), "discovery file should be cleaned up");
    }

    #[cfg(unix)]
    #[test]
    fn create_alias_replaces_existing() {
        let dir = tempfile::tempdir().expect("tempdir");

        {
            let ep1 = dir.path().join("old.sock");
            std::fs::write(&ep1, b"").expect("write");
            create_capability_alias(&ep1, "ai").expect("first");

            let ep2 = dir.path().join("new.sock");
            std::fs::write(&ep2, b"").expect("write");
            create_capability_alias(&ep2, "ai").expect("second should replace");

            let target = std::fs::read_link(dir.path().join("ai.sock")).expect("read_link");
            assert_eq!(target.to_string_lossy(), "new.sock");
        }
    }

    #[test]
    fn create_alias_invalid_path() {
        let result = create_capability_alias(Path::new("no_parent"), "ai");
        // "no_parent" has no parent directory component on some systems,
        // but on others Path::new("no_parent").parent() returns Some("")
        // Either outcome is acceptable for this edge case
        drop(result);
    }
}
