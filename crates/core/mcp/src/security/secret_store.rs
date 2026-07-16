// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Platform-abstracted secret storage.
//!
//! `SecretStore` is the trait that credential/token persistence backends
//! implement.  Squirrel ships three built-in backends:
//!
//! | Backend | Config variant | Persistence |
//! |---------|----------------|-------------|
//! | [`InMemorySecretStore`] | `CredentialStorage::Memory` | None (process lifetime) |
//! | [`FileSecretStore`] | `CredentialStorage::File { path }` | Base64-encoded JSON |
//! | (IPC delegation) | `CredentialStorage::SecurityProvider` | External (BearDog) |
//!
//! Platform-specific backends (Android Keystore, Windows Credential Manager)
//! are future work — each will implement this same trait.

use crate::error::{MCPError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Platform-agnostic secret storage trait.
///
/// Backends persist secrets keyed by an opaque string name (e.g.
/// `"jwt_secret"`, `"family_seed"`).  Values are raw byte blobs — the
/// caller is responsible for encoding/decoding.
#[expect(
    async_fn_in_trait,
    reason = "Storage backends — async I/O; impls are Send + Sync"
)]
pub trait SecretStore: Send + Sync + std::fmt::Debug {
    /// Retrieve a secret by name. Returns `None` if not present.
    async fn get(&self, name: &str) -> Result<Option<Vec<u8>>>;

    /// Store (or overwrite) a secret by name.
    async fn set(&self, name: &str, value: Vec<u8>) -> Result<()>;

    /// Delete a secret by name. Returns `true` if the key existed.
    async fn delete(&self, name: &str) -> Result<bool>;

    /// List all stored secret names (not values).
    async fn list_keys(&self) -> Result<Vec<String>>;
}

// ---------------------------------------------------------------------------
// In-memory backend
// ---------------------------------------------------------------------------

/// Volatile in-memory secret store — secrets are lost when the process exits.
///
/// Suitable for development and single-process deployments where secrets are
/// injected via environment variables at startup and never need to persist.
#[derive(Debug, Clone)]
pub struct InMemorySecretStore {
    secrets: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl InMemorySecretStore {
    /// Create a new empty in-memory store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            secrets: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemorySecretStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SecretStore for InMemorySecretStore {
    async fn get(&self, name: &str) -> Result<Option<Vec<u8>>> {
        let map = self.secrets.read().await;
        Ok(map.get(name).cloned())
    }

    async fn set(&self, name: &str, value: Vec<u8>) -> Result<()> {
        let mut map = self.secrets.write().await;
        map.insert(name.to_owned(), value);
        Ok(())
    }

    async fn delete(&self, name: &str) -> Result<bool> {
        let mut map = self.secrets.write().await;
        Ok(map.remove(name).is_some())
    }

    async fn list_keys(&self) -> Result<Vec<String>> {
        let map = self.secrets.read().await;
        Ok(map.keys().cloned().collect())
    }
}

// ---------------------------------------------------------------------------
// File-based backend
// ---------------------------------------------------------------------------

/// Persistent header+payload container written to disk.
#[derive(Debug, Serialize, Deserialize)]
struct FileStoreEnvelope {
    version: u32,
    entries: HashMap<String, EncodedSecret>,
}

/// A single secret entry as stored on disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EncodedSecret {
    /// Base64-encoded secret value.
    value_b64: String,
}

/// File-backed secret store — secrets persist as a JSON file.
///
/// Wires up the dormant `CredentialStorage::File { path }` config variant.
/// File permissions are set to owner-only (`0o600`) on Unix.
#[derive(Debug, Clone)]
pub struct FileSecretStore {
    path: PathBuf,
    cache: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl FileSecretStore {
    /// Open or create a file-backed secret store at `path`.
    ///
    /// If the file exists, its contents are loaded into memory. Otherwise an
    /// empty store is created (the file is written on the first `set`).
    pub async fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let entries = if path.exists() {
            Self::load_from_disk(&path).await?
        } else {
            HashMap::new()
        };
        Ok(Self {
            path,
            cache: Arc::new(RwLock::new(entries)),
        })
    }

    async fn load_from_disk(path: &Path) -> Result<HashMap<String, Vec<u8>>> {
        use base64::Engine;
        let bytes = tokio::fs::read(path).await.map_err(|e| {
            MCPError::Internal(format!(
                "Failed to read secret store at {}: {e}",
                path.display()
            ))
        })?;
        let envelope: FileStoreEnvelope = serde_json::from_slice(&bytes).map_err(|e| {
            MCPError::Internal(format!(
                "Corrupt secret store at {}: {e}",
                path.display()
            ))
        })?;
        let mut map = HashMap::with_capacity(envelope.entries.len());
        for (key, entry) in envelope.entries {
            let decoded = base64::engine::general_purpose::STANDARD
                .decode(&entry.value_b64)
                .map_err(|e| {
                    MCPError::Internal(format!("Base64 decode error for key {key:?}: {e}"))
                })?;
            map.insert(key, decoded);
        }
        Ok(map)
    }

    async fn flush_to_disk(&self) -> Result<()> {
        use base64::Engine;
        let map = self.cache.read().await;
        let entries: HashMap<String, EncodedSecret> = map
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    EncodedSecret {
                        value_b64: base64::engine::general_purpose::STANDARD.encode(v),
                    },
                )
            })
            .collect();

        let envelope = FileStoreEnvelope {
            version: 1,
            entries,
        };

        let json = serde_json::to_vec_pretty(&envelope).map_err(|e| {
            MCPError::Internal(format!("Failed to serialize secret store: {e}"))
        })?;

        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                MCPError::Internal(format!(
                    "Failed to create directory {}: {e}",
                    parent.display()
                ))
            })?;
        }

        tokio::fs::write(&self.path, &json).await.map_err(|e| {
            MCPError::Internal(format!(
                "Failed to write secret store to {}: {e}",
                self.path.display()
            ))
        })?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            tokio::fs::set_permissions(&self.path, perms)
                .await
                .map_err(|e| {
                    MCPError::Internal(format!(
                        "Failed to set permissions on {}: {e}",
                        self.path.display()
                    ))
                })?;
        }

        Ok(())
    }
}

impl SecretStore for FileSecretStore {
    async fn get(&self, name: &str) -> Result<Option<Vec<u8>>> {
        let map = self.cache.read().await;
        Ok(map.get(name).cloned())
    }

    async fn set(&self, name: &str, value: Vec<u8>) -> Result<()> {
        {
            let mut map = self.cache.write().await;
            map.insert(name.to_owned(), value);
        }
        self.flush_to_disk().await
    }

    async fn delete(&self, name: &str) -> Result<bool> {
        let existed = {
            let mut map = self.cache.write().await;
            map.remove(name).is_some()
        };
        if existed {
            self.flush_to_disk().await?;
        }
        Ok(existed)
    }

    async fn list_keys(&self) -> Result<Vec<String>> {
        let map = self.cache.read().await;
        Ok(map.keys().cloned().collect())
    }
}

// ---------------------------------------------------------------------------
// Runtime-dispatched store (enum dispatch — dyn-compatible)
// ---------------------------------------------------------------------------

/// Runtime-selected secret store backend (enum dispatch).
///
/// Constructed via [`SecretStoreBackend::from_config`] to wire up the
/// `CredentialStorage` configuration variant.
#[derive(Debug, Clone)]
pub enum SecretStoreBackend {
    /// Volatile in-memory store.
    Memory(InMemorySecretStore),
    /// Persistent file-backed store.
    File(FileSecretStore),
}

impl SecretStoreBackend {
    /// Build a backend from the config `CredentialStorage` variant.
    ///
    /// `SecurityProvider` returns an in-memory store as a local cache — the real
    /// secret retrieval happens via `SecurityProviderClient` RPC to BearDog.
    pub async fn from_config(
        storage: &universal_patterns::config::CredentialStorage,
    ) -> Result<Self> {
        use universal_patterns::config::CredentialStorage;
        match storage {
            CredentialStorage::Memory | CredentialStorage::SecurityProvider => {
                Ok(Self::Memory(InMemorySecretStore::new()))
            }
            CredentialStorage::File { path } => {
                let store = FileSecretStore::open(path).await?;
                Ok(Self::File(store))
            }
        }
    }
}

impl SecretStore for SecretStoreBackend {
    async fn get(&self, name: &str) -> Result<Option<Vec<u8>>> {
        match self {
            Self::Memory(s) => s.get(name).await,
            Self::File(s) => s.get(name).await,
        }
    }

    async fn set(&self, name: &str, value: Vec<u8>) -> Result<()> {
        match self {
            Self::Memory(s) => s.set(name, value).await,
            Self::File(s) => s.set(name, value).await,
        }
    }

    async fn delete(&self, name: &str) -> Result<bool> {
        match self {
            Self::Memory(s) => s.delete(name).await,
            Self::File(s) => s.delete(name).await,
        }
    }

    async fn list_keys(&self) -> Result<Vec<String>> {
        match self {
            Self::Memory(s) => s.list_keys().await,
            Self::File(s) => s.list_keys().await,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn in_memory_get_set_delete_list() {
        let store = InMemorySecretStore::new();

        assert!(store.get("missing").await.unwrap().is_none());
        assert!(store.list_keys().await.unwrap().is_empty());

        store.set("key1", b"value1".to_vec()).await.unwrap();
        store.set("key2", b"value2".to_vec()).await.unwrap();

        assert_eq!(store.get("key1").await.unwrap().unwrap(), b"value1");
        assert_eq!(store.get("key2").await.unwrap().unwrap(), b"value2");

        let mut keys = store.list_keys().await.unwrap();
        keys.sort();
        assert_eq!(keys, vec!["key1", "key2"]);

        assert!(store.delete("key1").await.unwrap());
        assert!(!store.delete("key1").await.unwrap());
        assert!(store.get("key1").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn in_memory_overwrite() {
        let store = InMemorySecretStore::new();
        store.set("k", b"v1".to_vec()).await.unwrap();
        store.set("k", b"v2".to_vec()).await.unwrap();
        assert_eq!(store.get("k").await.unwrap().unwrap(), b"v2");
    }

    #[tokio::test]
    async fn in_memory_default() {
        let store = InMemorySecretStore::default();
        assert!(store.list_keys().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn file_store_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("secrets.json");

        {
            let store = FileSecretStore::open(&path).await.unwrap();
            store.set("jwt", b"secret123".to_vec()).await.unwrap();
            store.set("seed", b"\x00\x01\x02".to_vec()).await.unwrap();
            assert_eq!(store.get("jwt").await.unwrap().unwrap(), b"secret123");
        }

        {
            let store = FileSecretStore::open(&path).await.unwrap();
            assert_eq!(store.get("jwt").await.unwrap().unwrap(), b"secret123");
            assert_eq!(store.get("seed").await.unwrap().unwrap(), b"\x00\x01\x02");

            assert!(store.delete("jwt").await.unwrap());
            assert!(store.get("jwt").await.unwrap().is_none());
        }

        {
            let store = FileSecretStore::open(&path).await.unwrap();
            assert!(store.get("jwt").await.unwrap().is_none());
            assert!(store.get("seed").await.unwrap().is_some());
        }
    }

    #[tokio::test]
    async fn file_store_creates_parent_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a").join("b").join("secrets.json");

        let store = FileSecretStore::open(&path).await.unwrap();
        store.set("k", b"v".to_vec()).await.unwrap();
        assert!(path.exists());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn file_store_unix_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("secrets.json");

        let store = FileSecretStore::open(&path).await.unwrap();
        store.set("k", b"v".to_vec()).await.unwrap();

        let meta = std::fs::metadata(&path).unwrap();
        let mode = meta.permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "file should be owner-only");
    }

    #[tokio::test]
    async fn backend_memory_variant() {
        use universal_patterns::config::CredentialStorage;
        let backend = SecretStoreBackend::from_config(&CredentialStorage::Memory)
            .await
            .unwrap();
        backend.set("test", b"val".to_vec()).await.unwrap();
        assert_eq!(backend.get("test").await.unwrap().unwrap(), b"val");
    }

    #[tokio::test]
    async fn backend_file_variant() {
        use universal_patterns::config::CredentialStorage;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("backend-test.json");
        let backend = SecretStoreBackend::from_config(&CredentialStorage::File {
            path: path.clone(),
        })
        .await
        .unwrap();
        backend.set("hello", b"world".to_vec()).await.unwrap();

        let backend2 = SecretStoreBackend::from_config(&CredentialStorage::File { path })
            .await
            .unwrap();
        assert_eq!(backend2.get("hello").await.unwrap().unwrap(), b"world");
    }

    #[tokio::test]
    async fn backend_security_provider_variant() {
        use universal_patterns::config::CredentialStorage;
        let backend = SecretStoreBackend::from_config(&CredentialStorage::SecurityProvider)
            .await
            .unwrap();
        backend.set("cached", b"val".to_vec()).await.unwrap();
        assert_eq!(backend.get("cached").await.unwrap().unwrap(), b"val");
    }
}
