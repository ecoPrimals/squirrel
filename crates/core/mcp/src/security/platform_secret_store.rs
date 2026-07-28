// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Platform credential cache — file-based fallback for offline/bootstrap.
//!
//! `PlatformSecretStore` provides a file-backed [`SecretStore`] at an
//! OS-appropriate path.  It is a **local cache**, not a credential authority.
//!
//! - For production-grade HSM-backed credentials, use
//!   `CredentialStorage::SecurityProvider` which delegates to the security provider via IPC.
//! - `PlatformSecretStore` is appropriate for bootstrap credentials before
//!   the security provider is available, MCP session state, cache keys, and offline
//!   operation.
//!
//! Native credential store backends (Windows Credential Manager, Android
//! Keystore, macOS Keychain) are the **security provider's domain**.  When the
//! security provider ships those backends, squirrel accesses them via the `SecurityProvider` IPC path.
//!
//! ## Path selection
//!
//! | OS | Default path |
//! |----|-------------|
//! | Linux | `$XDG_DATA_HOME/squirrel/secrets.json` (typically `~/.local/share/squirrel/secrets.json`) |
//! | macOS | `~/Library/Application Support/squirrel/secrets.json` |
//! | Windows | `%APPDATA%\squirrel\secrets.json` |
//! | Android | `<app_data>/squirrel/secrets.json` |
//! | Other | `~/.squirrel/secrets.json` |

use super::secret_store::{FileSecretStore, SecretStore};
use crate::error::Result;
use std::path::{Path, PathBuf};
use tracing::info;

/// Metadata describing the platform credential cache.
#[derive(Debug, Clone)]
pub struct PlatformStoreInfo {
    /// Human-readable backend name (e.g. "XDG file store", "AppData file store")
    pub backend_name: &'static str,
    /// Whether secrets are encrypted at rest by the OS
    pub os_encrypted: bool,
    /// Whether the store is hardware-backed (always false — hw-backed is the security provider's domain)
    pub hardware_backed: bool,
    /// Whether secrets are scoped to the current user session
    pub session_scoped: bool,
    /// Path used for the file-based backend
    pub file_path: Option<PathBuf>,
}

/// Platform credential cache — file-backed at an OS-appropriate path.
///
/// This is a **local cache**, not a credential authority.  For HSM-backed
/// storage, use `CredentialStorage::SecurityProvider` (security provider IPC).
/// The inner backend is selected at construction time via [`PlatformSecretStore::detect`].
#[derive(Debug, Clone)]
pub struct PlatformSecretStore {
    inner: PlatformBackend,
    info: PlatformStoreInfo,
}

/// The concrete backend for the platform credential cache.
///
/// File-based only.  Native credential stores (Windows Credential Manager,
/// Android Keystore, macOS Keychain) are the security provider's domain — squirrel
/// accesses them via `CredentialStorage::SecurityProvider` IPC.
#[derive(Debug, Clone)]
enum PlatformBackend {
    /// File-based store at a platform-appropriate path.
    File(FileSecretStore),
}

impl PlatformSecretStore {
    /// Detect and open the best available credential store for the current OS.
    ///
    /// # Errors
    ///
    /// Returns an error if the file-based fallback cannot be opened (e.g.
    /// filesystem permissions).
    pub async fn detect() -> Result<Self> {
        let path = platform_secret_path();
        let info = platform_store_info(&path);

        info!(
            backend = info.backend_name,
            path = %path.display(),
            os_encrypted = info.os_encrypted,
            "Platform credential store initialized"
        );

        let store = FileSecretStore::open(&path).await?;
        Ok(Self {
            inner: PlatformBackend::File(store),
            info,
        })
    }

    /// Open a platform store at a specific path (for testing or override).
    ///
    /// # Errors
    ///
    /// Returns an error if the file store cannot be opened.
    pub async fn with_path(path: PathBuf) -> Result<Self> {
        let info = PlatformStoreInfo {
            backend_name: "file (explicit path)",
            os_encrypted: false,
            hardware_backed: false,
            session_scoped: false,
            file_path: Some(path.clone()),
        };
        let store = FileSecretStore::open(&path).await?;
        Ok(Self {
            inner: PlatformBackend::File(store),
            info,
        })
    }

    /// Get metadata about the current platform store.
    #[must_use]
    pub const fn info(&self) -> &PlatformStoreInfo {
        &self.info
    }
}

impl SecretStore for PlatformSecretStore {
    async fn get(&self, name: &str) -> Result<Option<Vec<u8>>> {
        match &self.inner {
            PlatformBackend::File(s) => s.get(name).await,
        }
    }

    async fn set(&self, name: &str, value: Vec<u8>) -> Result<()> {
        match &self.inner {
            PlatformBackend::File(s) => s.set(name, value).await,
        }
    }

    async fn delete(&self, name: &str) -> Result<bool> {
        match &self.inner {
            PlatformBackend::File(s) => s.delete(name).await,
        }
    }

    async fn list_keys(&self) -> Result<Vec<String>> {
        match &self.inner {
            PlatformBackend::File(s) => s.list_keys().await,
        }
    }
}

/// Resolve the platform-appropriate path for the credential file.
fn platform_secret_path() -> PathBuf {
    // XDG / AppData / Application Support via `dirs` crate
    if let Some(data_dir) = dirs::data_dir() {
        return data_dir.join("squirrel").join("secrets.json");
    }

    // Fallback: home directory
    if let Some(home) = dirs::home_dir() {
        return home.join(".squirrel").join("secrets.json");
    }

    // Last resort: current directory (should never happen in practice)
    PathBuf::from(".squirrel").join("secrets.json")
}

/// Build the info struct describing the current platform's capabilities.
fn platform_store_info(path: &Path) -> PlatformStoreInfo {
    PlatformStoreInfo {
        backend_name: platform_backend_name(),
        os_encrypted: platform_os_encrypted(),
        hardware_backed: false,
        session_scoped: platform_session_scoped(),
        file_path: Some(path.to_path_buf()),
    }
}

#[cfg(target_os = "linux")]
const fn platform_backend_name() -> &'static str {
    "XDG file store (Linux)"
}

#[cfg(target_os = "macos")]
const fn platform_backend_name() -> &'static str {
    "Application Support file store (macOS)"
}

#[cfg(target_os = "windows")]
const fn platform_backend_name() -> &'static str {
    "AppData file store (Windows)"
}

#[cfg(target_os = "android")]
const fn platform_backend_name() -> &'static str {
    "app-private file store (Android)"
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "windows",
    target_os = "android"
)))]
const fn platform_backend_name() -> &'static str {
    "file store (generic)"
}

#[cfg(target_os = "windows")]
const fn platform_os_encrypted() -> bool {
    // NTFS EFS can encrypt user profile dirs; not guaranteed
    false
}

#[cfg(target_os = "macos")]
const fn platform_os_encrypted() -> bool {
    // FileVault encrypts the entire disk; file-level encryption not guaranteed
    false
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
const fn platform_os_encrypted() -> bool {
    false
}

#[cfg(target_os = "windows")]
const fn platform_session_scoped() -> bool {
    // %APPDATA% is per-user, accessible only during user's session
    true
}

#[cfg(not(target_os = "windows"))]
const fn platform_session_scoped() -> bool {
    false
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn detect_returns_valid_store() {
        let store = PlatformSecretStore::detect().await.expect("detect");
        let info = store.info();
        assert!(!info.backend_name.is_empty());
        assert!(info.file_path.is_some());
    }

    #[tokio::test]
    async fn with_path_round_trip() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let path = dir.path().join("platform-test.json");

        let store = PlatformSecretStore::with_path(path.clone())
            .await
            .expect("open");

        assert_eq!(store.info().backend_name, "file (explicit path)");

        store
            .set("platform_key", b"platform_value".to_vec())
            .await
            .expect("set");
        assert_eq!(
            store.get("platform_key").await.expect("get").expect("some"),
            b"platform_value"
        );

        let mut keys = store.list_keys().await.expect("list");
        keys.sort();
        assert_eq!(keys, vec!["platform_key"]);

        assert!(store.delete("platform_key").await.expect("delete"));
        assert!(store.get("platform_key").await.expect("get").is_none());
    }

    #[tokio::test]
    async fn detect_persists_across_instances() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let path = dir.path().join("persist-test.json");

        {
            let store = PlatformSecretStore::with_path(path.clone())
                .await
                .expect("open");
            store
                .set("persist_key", b"persist_value".to_vec())
                .await
                .expect("set");
        }

        {
            let store = PlatformSecretStore::with_path(path).await.expect("reopen");
            assert_eq!(
                store.get("persist_key").await.expect("get").expect("some"),
                b"persist_value"
            );
        }
    }

    #[test]
    fn platform_secret_path_is_not_empty() {
        let path = platform_secret_path();
        assert!(
            path.to_str().is_some_and(|s| !s.is_empty()),
            "platform path should not be empty"
        );
        assert!(
            path.ends_with("secrets.json"),
            "path should end with secrets.json"
        );
    }

    #[test]
    fn platform_info_fields() {
        let path = platform_secret_path();
        let info = platform_store_info(&path);
        assert!(!info.backend_name.is_empty());
        assert!(!info.hardware_backed);
    }
}
