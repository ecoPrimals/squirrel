// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! G68 L2: Platform-abstracted file access control.
//!
//! Unix: mode bits via `PermissionsExt`. Windows: `readonly` attribute +
//! inherited ACLs (full DACL manipulation deferred to `windows-sys` adoption).

use std::io;
use std::path::Path;

/// Semantic access level — the *intent*, not the platform mechanism.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessLevel {
    /// Owner can read, write, execute. No group/other access.
    /// Unix: `0o700`. Windows: inherited ACL (current user typically has full control).
    OwnerExclusive,

    /// Owner can read and write. No group/other access.
    /// Unix: `0o600`. Windows: inherited ACL, not marked readonly.
    OwnerReadWrite,

    /// Owner full, group read+execute. No other access.
    /// Unix: `0o750`. Windows: inherited ACL.
    GroupReadable,

    /// Specific Unix mode bits (passthrough for transport listener config).
    /// On Windows, maps to best-effort equivalent.
    #[cfg(unix)]
    Mode(u32),
}

/// Set access control on a filesystem path.
///
/// # Unix
/// Sets POSIX mode bits via `PermissionsExt`.
///
/// # Windows
/// Uses the `readonly` attribute for restrictive modes. Full DACL/ACL
/// manipulation requires `windows-sys` (tracked as G68-L2-DACL).
///
/// # Errors
/// Returns `io::Error` if the underlying permission-set call fails.
pub fn set_access(path: &Path, level: AccessLevel) -> io::Result<()> {
    set_access_inner(path, level)
}

/// Async variant of [`set_access`] for use in async contexts.
///
/// # Errors
/// Returns `io::Error` if the underlying permission-set call fails.
pub async fn set_access_async(path: &Path, level: AccessLevel) -> io::Result<()> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || set_access_inner(&path, level))
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
}

/// Check whether a file is world-accessible (security risk).
///
/// # Unix
/// Returns `true` if the world-write bit (`0o002`) is set.
///
/// # Windows
/// Returns `false` — files inherit parent ACL; world-writable is not
/// a meaningful concept without DACL inspection.
pub fn check_world_accessible(path: &Path) -> io::Result<bool> {
    check_world_accessible_inner(path)
}

// ── Unix implementation ────────────────────────────────────────────────────

#[cfg(unix)]
fn set_access_inner(path: &Path, level: AccessLevel) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mode = match level {
        AccessLevel::OwnerExclusive => 0o700,
        AccessLevel::OwnerReadWrite => 0o600,
        AccessLevel::GroupReadable => 0o750,
        AccessLevel::Mode(m) => m,
    };
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
}

#[cfg(unix)]
fn check_world_accessible_inner(path: &Path) -> io::Result<bool> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = std::fs::metadata(path)?;
    let mode = metadata.permissions().mode();
    Ok(mode & 0o002 != 0)
}

// ── Windows implementation ─────────────────────────────────────────────────

#[cfg(windows)]
fn set_access_inner(path: &Path, level: AccessLevel) -> io::Result<()> {
    let mut perms = std::fs::metadata(path)?.permissions();
    match level {
        AccessLevel::OwnerReadWrite | AccessLevel::GroupReadable => {
            perms.set_readonly(false);
        }
        AccessLevel::OwnerExclusive => {
            // Best-effort: mark readonly restricts casual modification.
            // Full user-only ACL requires windows-sys (G68-L2-DACL).
            perms.set_readonly(true);
        }
    }
    std::fs::set_permissions(path, perms)
}

#[cfg(windows)]
fn check_world_accessible_inner(_path: &Path) -> io::Result<bool> {
    // Windows files inherit parent ACL. Without DACL inspection (requires
    // windows-sys), we cannot determine world-accessibility. Return false
    // to avoid false-positive security warnings on Windows.
    Ok(false)
}

// ── Fallback for other targets ─────────────────────────────────────────────

#[cfg(not(any(unix, windows)))]
fn set_access_inner(_path: &Path, _level: AccessLevel) -> io::Result<()> {
    tracing::debug!("G68: no platform-specific access control on this target");
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn check_world_accessible_inner(_path: &Path) -> io::Result<bool> {
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn access_level_debug_display() {
        let level = AccessLevel::OwnerExclusive;
        let s = format!("{level:?}");
        assert!(s.contains("OwnerExclusive"));
    }

    #[test]
    fn set_access_owner_read_write() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("test.txt");
        std::fs::write(&file, b"data").expect("write");

        set_access(&file, AccessLevel::OwnerReadWrite).expect("set_access");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&file).expect("meta").permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
        }
    }

    #[test]
    fn set_access_owner_exclusive() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("test_exclusive.txt");
        std::fs::write(&file, b"data").expect("write");

        set_access(&file, AccessLevel::OwnerExclusive).expect("set_access");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&file).expect("meta").permissions().mode() & 0o777;
            assert_eq!(mode, 0o700);
        }
    }

    #[cfg(unix)]
    #[test]
    fn set_access_explicit_mode() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("test_mode.txt");
        std::fs::write(&file, b"data").expect("write");

        set_access(&file, AccessLevel::Mode(0o644)).expect("set_access");

        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&file).expect("meta").permissions().mode() & 0o777;
        assert_eq!(mode, 0o644);
    }

    #[test]
    fn check_world_accessible_normal_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("normal.txt");
        std::fs::write(&file, b"data").expect("write");

        set_access(&file, AccessLevel::OwnerReadWrite).expect("set_access");
        let world = check_world_accessible(&file).expect("check");
        assert!(!world);
    }

    #[cfg(unix)]
    #[test]
    fn check_world_accessible_world_writable() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("world.txt");
        std::fs::write(&file, b"data").expect("write");

        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o666)).expect("chmod");
        let world = check_world_accessible(&file).expect("check");
        assert!(world);
    }

    #[tokio::test]
    async fn set_access_async_works() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("async_test.txt");
        std::fs::write(&file, b"data").expect("write");

        set_access_async(&file, AccessLevel::OwnerReadWrite)
            .await
            .expect("set_access_async");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&file).expect("meta").permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
        }
    }
}
