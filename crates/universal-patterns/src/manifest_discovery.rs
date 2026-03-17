// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Manifest-based primal discovery — fallback when Songbird is unavailable.
//!
//! Absorbed from rhizoCrypt v0.13. Scans `$XDG_RUNTIME_DIR/ecoPrimals/*.json`
//! manifest files that running primals write at startup. Each manifest declares
//! the primal's identity, socket path, and capabilities.
//!
//! # Discovery priority
//!
//! 1. Songbird service mesh (preferred)
//! 2. biomeOS socket scan
//! 3. **Manifest scan** (this module) — zero-network fallback

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// A primal manifest file written to `$XDG_RUNTIME_DIR/ecoPrimals/`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimalManifest {
    /// Primal identifier (e.g., "squirrel", "beardog").
    pub primal: String,
    /// Semantic version.
    pub version: String,
    /// Unix socket path.
    pub socket: PathBuf,
    /// Capabilities offered.
    pub capabilities: Vec<String>,
    /// PID of the running process (for liveness checks).
    #[serde(default)]
    pub pid: Option<u32>,
    /// ISO 8601 timestamp when the manifest was written.
    #[serde(default)]
    pub started_at: Option<String>,
    /// FAMILY_ID for multi-instance isolation.
    #[serde(default)]
    pub family_id: Option<String>,
}

/// Discover all primal manifests from the standard directory.
///
/// Returns manifests sorted by primal name. Skips malformed or unreadable files.
pub fn discover_manifests() -> Vec<PrimalManifest> {
    let dir = manifest_directory();
    scan_directory(&dir)
}

/// Discover manifests matching a specific capability.
pub fn discover_by_capability(capability: &str) -> Vec<PrimalManifest> {
    discover_manifests()
        .into_iter()
        .filter(|m| m.capabilities.iter().any(|c| c == capability))
        .collect()
}

/// Discover a specific primal by name.
pub fn discover_by_name(primal: &str) -> Option<PrimalManifest> {
    discover_manifests()
        .into_iter()
        .find(|m| m.primal == primal)
}

/// Write this primal's manifest to the standard directory.
///
/// Called at startup so other primals can discover us via manifest scan.
pub fn write_manifest(manifest: &PrimalManifest) -> std::io::Result<()> {
    let dir = manifest_directory();
    std::fs::create_dir_all(&dir)?;

    let filename = match &manifest.family_id {
        Some(fid) => format!("{}-{fid}.json", manifest.primal),
        None => format!("{}.json", manifest.primal),
    };

    let path = dir.join(filename);
    let json = serde_json::to_string_pretty(manifest)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

    std::fs::write(&path, json)?;
    debug!(path = %path.display(), primal = %manifest.primal, "wrote primal manifest");
    Ok(())
}

/// Remove this primal's manifest on shutdown.
pub fn remove_manifest(primal: &str, family_id: Option<&str>) -> std::io::Result<()> {
    let dir = manifest_directory();
    let filename = match family_id {
        Some(fid) => format!("{primal}-{fid}.json"),
        None => format!("{primal}.json"),
    };
    let path = dir.join(filename);
    if path.exists() {
        std::fs::remove_file(&path)?;
        debug!(path = %path.display(), "removed primal manifest");
    }
    Ok(())
}

/// Check if a manifest's process is still alive.
#[must_use]
pub fn is_alive(manifest: &PrimalManifest) -> bool {
    match manifest.pid {
        Some(pid) => Path::new(&format!("/proc/{pid}")).exists(),
        None => manifest.socket.exists(),
    }
}

fn manifest_directory() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(xdg).join("ecoPrimals")
    } else {
        PathBuf::from("/tmp/ecoPrimals-manifests")
    }
}

fn scan_directory(dir: &Path) -> Vec<PrimalManifest> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        debug!(dir = %dir.display(), "manifest directory not found");
        return Vec::new();
    };

    let mut manifests = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "json") {
            match std::fs::read_to_string(&path) {
                Ok(contents) => match serde_json::from_str::<PrimalManifest>(&contents) {
                    Ok(m) => {
                        if is_alive(&m) {
                            manifests.push(m);
                        } else {
                            debug!(path = %path.display(), "stale manifest (process not alive)");
                        }
                    }
                    Err(e) => {
                        warn!(path = %path.display(), error = %e, "malformed manifest");
                    }
                },
                Err(e) => {
                    warn!(path = %path.display(), error = %e, "unreadable manifest");
                }
            }
        }
    }

    manifests.sort_by(|a, b| a.primal.cmp(&b.primal));
    manifests
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn test_manifest() -> PrimalManifest {
        PrimalManifest {
            primal: "test-primal".into(),
            version: "0.1.0".into(),
            socket: PathBuf::from("/tmp/test-primal.sock"),
            capabilities: vec!["system.health".into(), "ai.query".into()],
            pid: Some(std::process::id()),
            started_at: Some("2026-03-16T12:00:00Z".into()),
            family_id: None,
        }
    }

    #[test]
    fn manifest_serde_roundtrip() {
        let m = test_manifest();
        let json = serde_json::to_string(&m).unwrap();
        let deser: PrimalManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(m, deser);
    }

    #[test]
    fn scan_empty_directory() {
        let dir = TempDir::new().unwrap();
        let manifests = scan_directory(dir.path());
        assert!(manifests.is_empty());
    }

    #[test]
    fn scan_with_valid_manifest() {
        let dir = TempDir::new().unwrap();
        let m = test_manifest();
        let json = serde_json::to_string(&m).unwrap();
        let mut f = std::fs::File::create(dir.path().join("test-primal.json")).unwrap();
        f.write_all(json.as_bytes()).unwrap();

        let manifests = scan_directory(dir.path());
        assert_eq!(manifests.len(), 1);
        assert_eq!(manifests[0].primal, "test-primal");
    }

    #[test]
    fn scan_skips_malformed_files() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("bad.json"), "not json").unwrap();

        let manifests = scan_directory(dir.path());
        assert!(manifests.is_empty());
    }

    #[test]
    fn scan_skips_non_json_files() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("readme.txt"), "hello").unwrap();

        let manifests = scan_directory(dir.path());
        assert!(manifests.is_empty());
    }

    #[test]
    fn write_and_remove_manifest() {
        let dir = TempDir::new().unwrap();
        temp_env::with_var(
            "XDG_RUNTIME_DIR",
            Some(dir.path().to_str().unwrap()),
            || {
                let m = test_manifest();
                write_manifest(&m).unwrap();

                let expected = dir.path().join("ecoPrimals/test-primal.json");
                assert!(expected.exists());

                remove_manifest("test-primal", None).unwrap();
                assert!(!expected.exists());
            },
        );
    }

    #[test]
    fn write_manifest_with_family_id() {
        let dir = TempDir::new().unwrap();
        temp_env::with_var(
            "XDG_RUNTIME_DIR",
            Some(dir.path().to_str().unwrap()),
            || {
                let mut m = test_manifest();
                m.family_id = Some("alpha".into());
                write_manifest(&m).unwrap();

                let expected = dir.path().join("ecoPrimals/test-primal-alpha.json");
                assert!(expected.exists());
            },
        );
    }

    #[test]
    fn is_alive_with_current_pid() {
        let mut m = test_manifest();
        m.pid = Some(std::process::id());
        assert!(is_alive(&m));
    }

    #[test]
    fn is_alive_with_bogus_pid() {
        let mut m = test_manifest();
        m.pid = Some(999_999_999);
        assert!(!is_alive(&m));
    }

    #[test]
    fn discover_by_capability_filters() {
        let dir = TempDir::new().unwrap();

        let m1 = PrimalManifest {
            primal: "alpha".into(),
            version: "1.0".into(),
            socket: PathBuf::from("/tmp/a.sock"),
            capabilities: vec!["ai.query".into()],
            pid: Some(std::process::id()),
            started_at: None,
            family_id: None,
        };
        let m2 = PrimalManifest {
            primal: "beta".into(),
            version: "1.0".into(),
            socket: PathBuf::from("/tmp/b.sock"),
            capabilities: vec!["storage.read".into()],
            pid: Some(std::process::id()),
            started_at: None,
            family_id: None,
        };

        let eco_dir = dir.path().join("ecoPrimals");
        std::fs::create_dir_all(&eco_dir).unwrap();
        std::fs::write(
            eco_dir.join("alpha.json"),
            serde_json::to_string(&m1).unwrap(),
        )
        .unwrap();
        std::fs::write(
            eco_dir.join("beta.json"),
            serde_json::to_string(&m2).unwrap(),
        )
        .unwrap();

        let all = scan_directory(&eco_dir.parent().unwrap().join("ecoPrimals"));
        let ai_primals: Vec<_> = all
            .into_iter()
            .filter(|m| m.capabilities.iter().any(|c| c == "ai.query"))
            .collect();
        assert_eq!(ai_primals.len(), 1);
        assert_eq!(ai_primals[0].primal, "alpha");
    }
}
