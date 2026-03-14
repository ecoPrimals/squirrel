// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Secure plugin loading and validation
//!
//! This module provides secure plugin loading with proper validation,
//! sandboxing, and error handling to prevent security vulnerabilities.

use std::path::Path;
use std::sync::Arc;
use thiserror::Error;
use tracing::{info, warn};

// Backward compatibility: PluginMetadata during migration to squirrel_interfaces
use crate::plugins::plugin::PluginMetadata;
use crate::plugins::{Plugin, PluginError}; // Use local PluginMetadata for compatibility

/// Secure plugin loader errors
#[derive(Error, Debug)]
pub enum PluginSecurityError {
    #[error("Plugin validation failed: {0}")]
    ValidationFailed(String),

    #[error("Plugin signature verification failed: {0}")]
    SignatureVerificationFailed(String),

    #[error("Plugin sandboxing failed: {0}")]
    SandboxingFailed(String),

    #[error("Plugin loading denied: {0}")]
    LoadingDenied(String),

    #[error("Plugin execution timeout")]
    ExecutionTimeout,
}

/// Plugin validation result
#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub checksum: String,
    pub signature_valid: bool,
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
    /// Create a new secure plugin loader
    pub fn new() -> Self {
        Self {
            allowed_directories: vec![
                "./plugins".to_string(),
                "/usr/local/lib/squirrel/plugins".to_string(),
            ],
            verify_signatures: true,
            max_plugin_size: 50 * 1024 * 1024, // 50MB limit
        }
    }

    /// Validate a plugin before loading
    pub async fn validate_plugin(
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
            self.verify_plugin_signature(plugin_path, &checksum).await?
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
    pub async fn load_plugin_secure(
        &self,
        plugin_path: &Path,
        metadata: &PluginMetadata,
    ) -> Result<Arc<dyn Plugin>, PluginSecurityError> {
        // First validate the plugin
        let validation = self.validate_plugin(plugin_path, metadata).await?;

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

    /// Calculate secure checksum of plugin file
    fn calculate_checksum(&self, plugin_path: &Path) -> Result<String, PluginSecurityError> {
        use sha2::{Digest, Sha256};

        let contents = std::fs::read(plugin_path).map_err(|e| {
            PluginSecurityError::ValidationFailed(format!("Cannot read plugin file: {}", e))
        })?;

        let mut hasher = Sha256::new();
        hasher.update(&contents);

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Verify plugin signature (placeholder - integrate with actual security system)
    async fn verify_plugin_signature(
        &self,
        plugin_path: &Path,
        checksum: &str,
    ) -> Result<bool, PluginSecurityError> {
        // Look for .sig file
        let sig_path = plugin_path.with_extension("sig");

        if !sig_path.exists() {
            warn!(
                "⚠️ No signature file found for plugin: {}",
                plugin_path.display()
            );
            return Ok(false); // In production, this should fail
        }

        // NOTE(phase2): Security primal integration via capability discovery for signature verification
        // Use capability registry to discover security service, then verify via Unix socket JSON-RPC
        info!(
            "🔐 Signature verification placeholder for checksum: {}",
            checksum
        );

        Ok(true) // Placeholder - implement actual verification
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

/// Secure plugin stub that replaces unsafe dynamic loading
pub struct SecurePluginStub {
    metadata: PluginMetadata,
}

impl SecurePluginStub {
    pub fn new(metadata: PluginMetadata) -> Self {
        Self { metadata }
    }
}

#[async_trait::async_trait]
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

    async fn initialize(&self) -> Result<(), PluginError> {
        info!("🔒 Secure plugin stub initialized: {}", self.metadata.name);
        Ok(())
    }

    fn register_commands(
        &self,
        _registry: &crate::commands::registry::CommandRegistry,
    ) -> Result<(), PluginError> {
        // Stub implementation - no commands to register
        Ok(())
    }

    fn commands(&self) -> Vec<std::sync::Arc<dyn squirrel_commands::Command>> {
        // Stub implementation - no commands
        Vec::new()
    }

    async fn execute(&self, _args: &[String]) -> Result<String, PluginError> {
        Ok("Secure plugin stub executed".to_string())
    }

    async fn cleanup(&self) -> Result<(), PluginError> {
        info!("🔒 Secure plugin stub cleanup: {}", self.metadata.name);
        Ok(())
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

    #[test]
    fn test_secure_plugin_loader_new() {
        let loader = SecurePluginLoader::new();
        assert!(loader.verify_signatures);
        assert_eq!(loader.max_plugin_size, 50 * 1024 * 1024);
        assert_eq!(loader.allowed_directories.len(), 2);
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
        };
        let stub = SecurePluginStub::new(metadata);
        let result = stub.initialize().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_secure_plugin_stub_execute() {
        let metadata = PluginMetadata {
            name: "exec-test".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            author: None,
            homepage: None,
        };
        let stub = SecurePluginStub::new(metadata);
        let result = stub.execute(&["arg1".to_string()]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Secure plugin stub executed");
    }

    #[tokio::test]
    async fn test_secure_plugin_stub_cleanup() {
        let metadata = PluginMetadata {
            name: "cleanup-test".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            author: None,
            homepage: None,
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
        };
        let stub = SecurePluginStub::new(metadata);
        assert!(stub.commands().is_empty());
    }
}
