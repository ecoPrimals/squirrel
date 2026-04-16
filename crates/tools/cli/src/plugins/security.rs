// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Secure plugin loading and validation
//!
//! This module provides secure plugin loading with proper validation,
//! sandboxing, and error handling to prevent security vulnerabilities.
//!
//! **Native code execution:** After validation, [`SecurePluginLoader::load_plugin_secure`]
//! returns [`SecurePluginStub`], which is the **intentional production implementation** for
//! this CLI tier: native `.so` execution is disabled; integration happens through the command
//! registry. This is not a test double — it is a deny-by-default execution policy until an
//! optional sandboxed runtime (for example WebAssembly) exists.

use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use thiserror::Error;
use tracing::{info, warn};

// Backward compatibility: PluginMetadata during migration to squirrel_interfaces
use crate::plugins::plugin::PluginMetadata;
use crate::plugins::{Plugin, PluginError}; // Use local PluginMetadata for compatibility

/// Constant-time equality for equal-length byte slices (mitigates timing leaks on match).
fn constant_time_eq_bytes(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Secure plugin loader errors
#[derive(Error, Debug)]
pub enum PluginSecurityError {
    #[error("Plugin validation failed: {0}")]
    /// Plugin validation failed with a reason.
    ValidationFailed(String),

    #[error("Plugin signature verification failed: {0}")]
    /// Plugin signature verification failed with a reason.
    SignatureVerificationFailed(String),

    #[error("Plugin sandboxing failed: {0}")]
    /// Plugin sandboxing failed with a reason.
    SandboxingFailed(String),

    #[error("Plugin loading denied: {0}")]
    /// Plugin loading was denied with a reason.
    LoadingDenied(String),

    #[error("Plugin execution timeout")]
    /// Plugin execution exceeded the allowed time limit.
    ExecutionTimeout,
}

/// Plugin validation result
#[derive(Debug)]
pub struct ValidationResult {
    /// Whether the plugin passed all validation checks.
    pub is_valid: bool,
    /// Hex-encoded checksum of the plugin artifact.
    pub checksum: String,
    /// Whether the plugin signature was verified successfully.
    pub signature_valid: bool,
    /// Non-fatal security warnings collected during validation.
    pub warnings: Vec<String>,
}

/// Secure plugin loader that replaces unsafe dynamic loading
pub struct SecurePluginLoader {
    /// Allowed plugin directories
    allowed_directories: Vec<String>,
    /// Enable signature verification
    verify_signatures: bool,
    /// Maximum plugin size in bytes
    max_plugin_size: usize,
}

impl SecurePluginLoader {
    /// Create a new secure plugin loader.
    ///
    /// Plugin directories are read from `SQUIRREL_PLUGIN_DIRS` (colon-separated)
    /// with sensible defaults when the variable is absent.
    pub fn new() -> Self {
        let allowed_directories = std::env::var("SQUIRREL_PLUGIN_DIRS")
            .map(|v| v.split(':').map(String::from).collect())
            .unwrap_or_else(|_| {
                vec![
                    "./plugins".to_string(),
                    "/usr/local/lib/squirrel/plugins".to_string(),
                ]
            });
        Self {
            allowed_directories,
            verify_signatures: true,
            max_plugin_size: 50 * 1024 * 1024,
        }
    }

    /// Validate a plugin before loading
    pub fn validate_plugin(
        &self,
        plugin_path: &Path,
        metadata: &PluginMetadata,
    ) -> Result<ValidationResult, PluginSecurityError> {
        info!("🔒 Validating plugin: {}", metadata.name);

        // Check if path is in allowed directories
        if !self.is_path_allowed(plugin_path)? {
            return Err(PluginSecurityError::LoadingDenied(format!(
                "Plugin path not in allowed directories: {}",
                plugin_path.display()
            )));
        }

        // Check file size
        let file_size = std::fs::metadata(plugin_path)
            .map_err(|e| {
                PluginSecurityError::ValidationFailed(format!("Cannot read plugin metadata: {}", e))
            })?
            .len();

        if file_size > self.max_plugin_size as u64 {
            return Err(PluginSecurityError::ValidationFailed(format!(
                "Plugin file too large: {} bytes (max: {} bytes)",
                file_size, self.max_plugin_size
            )));
        }

        // Calculate checksum
        let checksum = self.calculate_checksum(plugin_path)?;
        info!("📋 Plugin checksum: {}", checksum);

        // Verify signature if enabled
        let signature_valid = if self.verify_signatures {
            self.verify_plugin_signature(plugin_path, &checksum)?
        } else {
            warn!("⚠️ Signature verification disabled");
            true
        };

        // Additional security checks
        let warnings = self.perform_security_checks(plugin_path)?;

        Ok(ValidationResult {
            is_valid: signature_valid && warnings.is_empty(),
            checksum,
            signature_valid,
            warnings,
        })
    }

    /// Securely load a plugin (replaces unsafe loading)
    pub fn load_plugin_secure(
        &self,
        plugin_path: &Path,
        metadata: &PluginMetadata,
    ) -> Result<Arc<dyn Plugin>, PluginSecurityError> {
        // First validate the plugin
        let validation = self.validate_plugin(plugin_path, metadata)?;

        if !validation.is_valid {
            return Err(PluginSecurityError::ValidationFailed(format!(
                "Plugin validation failed with {} warnings",
                validation.warnings.len()
            )));
        }

        // For now, return a secure stub plugin instead of unsafe dynamic loading
        // NOTE(phase2): Proper sandboxed plugin loading requires WebAssembly runtime integration
        info!("🔒 Creating secure plugin stub for: {}", metadata.name);
        Ok(Arc::new(SecurePluginStub::new(metadata.clone())))
    }

    /// Check if plugin path is in allowed directories
    fn is_path_allowed(&self, plugin_path: &Path) -> Result<bool, PluginSecurityError> {
        let canonical_path = plugin_path.canonicalize().map_err(|e| {
            PluginSecurityError::ValidationFailed(format!("Cannot canonicalize path: {}", e))
        })?;

        for allowed_dir in &self.allowed_directories {
            let allowed_canonical = Path::new(allowed_dir)
                .canonicalize()
                .unwrap_or_else(|_| Path::new(allowed_dir).to_path_buf());

            if canonical_path.starts_with(&allowed_canonical) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Calculate secure checksum of plugin file using blake3 (pure Rust)
    fn calculate_checksum(&self, plugin_path: &Path) -> Result<String, PluginSecurityError> {
        let contents = std::fs::read(plugin_path).map_err(|e| {
            PluginSecurityError::ValidationFailed(format!("Cannot read plugin file: {}", e))
        })?;

        Ok(blake3::hash(&contents).to_hex().to_string())
    }

    /// Verify plugin signature by comparing the computed blake3 hex checksum with the hash in the `.sig` file.
    fn verify_plugin_signature(
        &self,
        plugin_path: &Path,
        checksum: &str,
    ) -> Result<bool, PluginSecurityError> {
        let sig_path = plugin_path.with_extension("sig");

        if !sig_path.exists() {
            warn!(
                "⚠️ No signature file found for plugin: {}",
                plugin_path.display()
            );
            return Ok(false);
        }

        let sig_contents = std::fs::read_to_string(&sig_path).map_err(|e| {
            PluginSecurityError::SignatureVerificationFailed(format!(
                "Cannot read signature file {}: {}",
                sig_path.display(),
                e
            ))
        })?;

        let expected = sig_contents.trim();
        if expected.is_empty() {
            warn!("⚠️ Signature file is empty: {}", sig_path.display());
            return Ok(false);
        }

        // `calculate_checksum` uses blake3 hex (lowercase); normalize file contents for comparison.
        let expected_lower = expected.to_ascii_lowercase();
        if expected_lower.len() != checksum.len() {
            warn!(
                "⚠️ Signature hash length mismatch (want {} hex chars, got {}): {}",
                checksum.len(),
                expected_lower.len(),
                sig_path.display()
            );
            return Ok(false);
        }

        let matches = constant_time_eq_bytes(expected_lower.as_bytes(), checksum.as_bytes());
        if matches {
            info!(
                "🔐 Plugin signature verified (blake3 matches {}): {}",
                sig_path.display(),
                plugin_path.display()
            );
        } else {
            warn!(
                "⚠️ Plugin signature mismatch: computed checksum does not match {}",
                sig_path.display()
            );
        }

        Ok(matches)
    }

    /// Perform additional security checks
    fn perform_security_checks(
        &self,
        plugin_path: &Path,
    ) -> Result<Vec<String>, PluginSecurityError> {
        let mut warnings = Vec::new();

        // Check file permissions
        let metadata = std::fs::metadata(plugin_path).map_err(|e| {
            PluginSecurityError::ValidationFailed(format!("Cannot read file metadata: {}", e))
        })?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = metadata.permissions().mode();

            // Check if file is world-writable (security risk)
            if mode & 0o002 != 0 {
                warnings.push("Plugin file is world-writable".to_string());
            }
        }

        Ok(warnings)
    }
}

/// Sandboxed plugin implementation returned after validation (not a test double).
///
/// Native `.so` / arbitrary code execution is intentionally disabled. Plugins integrate through
/// the CLI [`crate::commands::registry::CommandRegistry`] instead. A future WebAssembly runtime
/// could replace this type; until then this is the supported production path after
/// [`SecurePluginLoader::validate_plugin`].
pub struct SecurePluginStub {
    metadata: PluginMetadata,
}

impl SecurePluginStub {
    /// Creates a secure plugin stub with the given metadata.
    pub fn new(metadata: PluginMetadata) -> Self {
        Self { metadata }
    }
}

impl Plugin for SecurePluginStub {
    fn name(&self) -> &str {
        &self.metadata.name
    }

    fn version(&self) -> &str {
        &self.metadata.version
    }

    fn description(&self) -> Option<&str> {
        self.metadata.description.as_deref()
    }

    fn initialize(&self) -> Pin<Box<dyn Future<Output = Result<(), PluginError>> + Send + '_>> {
        let name = self.metadata.name.clone();
        Box::pin(async move {
            info!("🔒 Secure plugin stub initialized: {}", name);
            Ok(())
        })
    }

    fn register_commands(
        &self,
        _registry: &crate::commands::registry::CommandRegistry,
    ) -> Result<(), PluginError> {
        // Intentionally empty: validated plugins do not inject native command handlers here;
        // operators register CLI commands through the shared registry separately.
        Ok(())
    }

    fn commands(&self) -> Vec<std::sync::Arc<dyn squirrel_commands::Command>> {
        // No dynamic handlers — matches deny-native-execution policy (see `execute`).
        Vec::new()
    }

    fn execute(
        &self,
        _args: &[String],
    ) -> Pin<Box<dyn Future<Output = Result<String, PluginError>> + Send + '_>> {
        let name = self.metadata.name.clone();
        Box::pin(async move {
            Err(PluginError::SecurityError(format!(
                "Plugin '{name}' is a security sandbox — native execution is disabled. \
                 Register commands via the CLI command registry instead.",
            )))
        })
    }

    fn cleanup(&self) -> Pin<Box<dyn Future<Output = Result<(), PluginError>> + Send + '_>> {
        let name = self.metadata.name.clone();
        Box::pin(async move {
            info!("🔒 Secure plugin stub cleanup: {}", name);
            Ok(())
        })
    }
}

impl Default for SecurePluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_secure_plugin_loader_new() {
        temp_env::with_var_unset("SQUIRREL_PLUGIN_DIRS", || {
            let loader = SecurePluginLoader::new();
            assert!(loader.verify_signatures);
            assert_eq!(loader.max_plugin_size, 50 * 1024 * 1024);
            assert_eq!(loader.allowed_directories.len(), 2);
        });
    }

    #[test]
    fn test_secure_plugin_loader_default() {
        let loader = SecurePluginLoader::default();
        assert!(loader.verify_signatures);
    }

    #[test]
    fn test_plugin_security_error_display() {
        let cases = vec![
            (
                PluginSecurityError::ValidationFailed("bad".to_string()),
                "Plugin validation failed: bad",
            ),
            (
                PluginSecurityError::SignatureVerificationFailed("sig".to_string()),
                "Plugin signature verification failed: sig",
            ),
            (
                PluginSecurityError::SandboxingFailed("sandbox".to_string()),
                "Plugin sandboxing failed: sandbox",
            ),
            (
                PluginSecurityError::LoadingDenied("denied".to_string()),
                "Plugin loading denied: denied",
            ),
            (
                PluginSecurityError::ExecutionTimeout,
                "Plugin execution timeout",
            ),
        ];
        for (error, expected) in cases {
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn test_validation_result_fields() {
        let result = ValidationResult {
            is_valid: true,
            checksum: "abc123".to_string(),
            signature_valid: true,
            warnings: vec![],
        };
        assert!(result.is_valid);
        assert!(result.signature_valid);
        assert!(result.warnings.is_empty());
        assert_eq!(result.checksum, "abc123");
    }

    #[test]
    fn test_validation_result_with_warnings() {
        let result = ValidationResult {
            is_valid: false,
            checksum: "def456".to_string(),
            signature_valid: false,
            warnings: vec!["world-writable".to_string()],
        };
        assert!(!result.is_valid);
        assert!(!result.signature_valid);
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_is_path_allowed_nonexistent() {
        let loader = SecurePluginLoader::new();
        // Non-existent path should fail to canonicalize
        let result = loader.is_path_allowed(Path::new("/nonexistent/path/plugin.so"));
        assert!(result.is_err());
    }

    #[test]
    fn test_secure_plugin_stub_name() {
        let metadata = PluginMetadata {
            name: "test-stub".to_string(),
            version: "1.0.0".to_string(),
            description: Some("A stub".to_string()),
            author: None,
            homepage: None,
            capabilities: vec![],
        };
        let stub = SecurePluginStub::new(metadata);
        assert_eq!(stub.name(), "test-stub");
        assert_eq!(stub.version(), "1.0.0");
        assert_eq!(stub.description(), Some("A stub"));
    }

    #[test]
    fn test_secure_plugin_stub_no_description() {
        let metadata = PluginMetadata {
            name: "no-desc".to_string(),
            version: "0.1.0".to_string(),
            description: None,
            author: None,
            homepage: None,
            capabilities: vec![],
        };
        let stub = SecurePluginStub::new(metadata);
        assert!(stub.description().is_none());
    }

    #[tokio::test]
    async fn test_secure_plugin_stub_initialize() {
        let metadata = PluginMetadata {
            name: "init-test".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            author: None,
            homepage: None,
            capabilities: vec![],
        };
        let stub = SecurePluginStub::new(metadata);
        let result = stub.initialize().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_secure_plugin_stub_execute_returns_security_error() {
        let metadata = PluginMetadata {
            name: "exec-test".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            author: None,
            homepage: None,
            capabilities: vec![],
        };
        let stub = SecurePluginStub::new(metadata);
        let result = stub.execute(&["arg1".to_string()]).await;
        assert!(
            result.is_err(),
            "Sandbox plugins must reject direct execution"
        );
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("security sandbox"),
            "Error should mention sandbox: {err}"
        );
    }

    #[tokio::test]
    async fn test_secure_plugin_stub_cleanup() {
        let metadata = PluginMetadata {
            name: "cleanup-test".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            author: None,
            homepage: None,
            capabilities: vec![],
        };
        let stub = SecurePluginStub::new(metadata);
        let result = stub.cleanup().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_secure_plugin_stub_commands_empty() {
        let metadata = PluginMetadata {
            name: "cmd-test".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            author: None,
            homepage: None,
            capabilities: vec![],
        };
        let stub = SecurePluginStub::new(metadata);
        assert!(stub.commands().is_empty());
    }

    #[test]
    fn validate_plugin_rejects_path_outside_allowed_dirs() {
        let dir = tempdir().expect("tempdir");
        let outside =
            std::env::temp_dir().join(format!("squirrel-plugin-outside-{}.so", std::process::id()));
        std::fs::write(&outside, b"x").expect("write outside plugin");
        temp_env::with_var(
            "SQUIRREL_PLUGIN_DIRS",
            Some(dir.path().to_str().expect("utf8")),
            || {
                let loader = SecurePluginLoader::new();
                let meta = PluginMetadata {
                    name: "x".to_string(),
                    version: "1".to_string(),
                    description: None,
                    author: None,
                    homepage: None,
                    capabilities: vec![],
                };
                let err = loader
                    .validate_plugin(&outside, &meta)
                    .expect_err("outside allowed dirs");
                assert!(matches!(err, PluginSecurityError::LoadingDenied(_)));
            },
        );
        let _ = std::fs::remove_file(&outside);
    }

    #[test]
    fn validate_plugin_checksum_and_signature_happy_path() {
        let dir = tempdir().expect("tempdir");
        let plugin_path = dir.path().join("p.so");
        std::fs::write(&plugin_path, b"plugin-bytes").expect("write plugin");
        let hash = blake3::hash(b"plugin-bytes").to_hex().to_string();
        let mut sig = std::fs::File::create(plugin_path.with_extension("sig")).expect("sig");
        writeln!(sig, "{}", hash.to_ascii_uppercase()).expect("write sig");

        temp_env::with_var(
            "SQUIRREL_PLUGIN_DIRS",
            Some(dir.path().to_str().expect("utf8")),
            || {
                let loader = SecurePluginLoader::new();
                let meta = PluginMetadata {
                    name: "plug".to_string(),
                    version: "0.1.0".to_string(),
                    description: None,
                    author: None,
                    homepage: None,
                    capabilities: vec![],
                };
                let v = loader.validate_plugin(&plugin_path, &meta).expect("valid");
                assert!(v.signature_valid);
                assert!(v.is_valid);
                assert_eq!(v.checksum, hash);
            },
        );
    }

    #[test]
    fn validate_plugin_missing_signature_marks_invalid() {
        let dir = tempdir().expect("tempdir");
        let plugin_path = dir.path().join("n.so");
        std::fs::write(&plugin_path, b"x").expect("write");

        temp_env::with_var(
            "SQUIRREL_PLUGIN_DIRS",
            Some(dir.path().to_str().expect("utf8")),
            || {
                let loader = SecurePluginLoader::new();
                let meta = PluginMetadata {
                    name: "n".to_string(),
                    version: "1".to_string(),
                    description: None,
                    author: None,
                    homepage: None,
                    capabilities: vec![],
                };
                let v = loader
                    .validate_plugin(&plugin_path, &meta)
                    .expect("validate");
                assert!(!v.signature_valid);
                assert!(!v.is_valid);
            },
        );
    }

    #[test]
    fn validate_plugin_signature_mismatch() {
        let dir = tempdir().expect("tempdir");
        let plugin_path = dir.path().join("m.so");
        std::fs::write(&plugin_path, b"data").expect("write");
        std::fs::write(
            plugin_path.with_extension("sig"),
            "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
        )
        .expect("sig");

        temp_env::with_var(
            "SQUIRREL_PLUGIN_DIRS",
            Some(dir.path().to_str().expect("utf8")),
            || {
                let loader = SecurePluginLoader::new();
                let meta = PluginMetadata {
                    name: "m".to_string(),
                    version: "1".to_string(),
                    description: None,
                    author: None,
                    homepage: None,
                    capabilities: vec![],
                };
                let v = loader
                    .validate_plugin(&plugin_path, &meta)
                    .expect("validate");
                assert!(!v.signature_valid);
            },
        );
    }

    #[test]
    fn validate_plugin_short_signature_file_rejected() {
        let dir = tempdir().expect("tempdir");
        let plugin_path = dir.path().join("short.so");
        std::fs::write(&plugin_path, b"z").expect("write");
        let real = blake3::hash(b"z").to_hex().to_string();
        std::fs::write(
            plugin_path.with_extension("sig"),
            &real[..real.len().saturating_sub(4)],
        )
        .expect("sig");

        temp_env::with_var(
            "SQUIRREL_PLUGIN_DIRS",
            Some(dir.path().to_str().expect("utf8")),
            || {
                let loader = SecurePluginLoader::new();
                let meta = PluginMetadata {
                    name: "s".to_string(),
                    version: "1".to_string(),
                    description: None,
                    author: None,
                    homepage: None,
                    capabilities: vec![],
                };
                let v = loader
                    .validate_plugin(&plugin_path, &meta)
                    .expect("validate");
                assert!(!v.signature_valid);
            },
        );
    }

    #[test]
    fn load_plugin_secure_fails_when_validation_not_valid() {
        let dir = tempdir().expect("tempdir");
        let plugin_path = dir.path().join("bad.so");
        std::fs::write(&plugin_path, b"q").expect("write");

        temp_env::with_var(
            "SQUIRREL_PLUGIN_DIRS",
            Some(dir.path().to_str().expect("utf8")),
            || {
                let loader = SecurePluginLoader::new();
                let meta = PluginMetadata {
                    name: "bad".to_string(),
                    version: "1".to_string(),
                    description: None,
                    author: None,
                    homepage: None,
                    capabilities: vec![],
                };
                let Err(err) = loader.load_plugin_secure(&plugin_path, &meta) else {
                    panic!("expected validation failure without signature");
                };
                assert!(matches!(err, PluginSecurityError::ValidationFailed(_)));
            },
        );
    }

    #[test]
    fn load_plugin_secure_returns_stub_when_valid() {
        let dir = tempdir().expect("tempdir");
        let plugin_path = dir.path().join("ok.so");
        std::fs::write(&plugin_path, b"ok-bytes").expect("write");
        let h = blake3::hash(b"ok-bytes").to_hex().to_string();
        std::fs::write(plugin_path.with_extension("sig"), format!("{h}\n")).expect("sig");

        temp_env::with_var(
            "SQUIRREL_PLUGIN_DIRS",
            Some(dir.path().to_str().expect("utf8")),
            || {
                let loader = SecurePluginLoader::new();
                let meta = PluginMetadata {
                    name: "okp".to_string(),
                    version: "2".to_string(),
                    description: Some("d".to_string()),
                    author: None,
                    homepage: None,
                    capabilities: vec![],
                };
                let plug = loader
                    .load_plugin_secure(&plugin_path, &meta)
                    .expect("stub");
                assert_eq!(plug.name(), "okp");
            },
        );
    }

    #[tokio::test]
    async fn secure_plugin_stub_register_commands_ok() {
        let metadata = PluginMetadata {
            name: "reg".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            author: None,
            homepage: None,
            capabilities: vec![],
        };
        let stub = SecurePluginStub::new(metadata);
        let reg = crate::commands::registry::CommandRegistry::new();
        assert!(stub.register_commands(&reg).is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn validate_plugin_world_writable_adds_warning() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir().expect("tempdir");
        let plugin_path = dir.path().join("ww.so");
        std::fs::write(&plugin_path, b"w").expect("write");
        let h = blake3::hash(b"w").to_hex().to_string();
        std::fs::write(plugin_path.with_extension("sig"), &h).expect("sig");
        let mut perms = std::fs::metadata(&plugin_path).expect("meta").permissions();
        perms.set_mode(0o666);
        std::fs::set_permissions(&plugin_path, perms).expect("chmod");

        temp_env::with_var(
            "SQUIRREL_PLUGIN_DIRS",
            Some(dir.path().to_str().expect("utf8")),
            || {
                let loader = SecurePluginLoader::new();
                let meta = PluginMetadata {
                    name: "ww".to_string(),
                    version: "1".to_string(),
                    description: None,
                    author: None,
                    homepage: None,
                    capabilities: vec![],
                };
                let v = loader
                    .validate_plugin(&plugin_path, &meta)
                    .expect("validate");
                assert!(v.signature_valid);
                assert!(!v.warnings.is_empty());
                assert!(!v.is_valid);
            },
        );
    }
}
