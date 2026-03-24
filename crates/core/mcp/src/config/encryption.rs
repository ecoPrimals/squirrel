// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Encryption configuration for MCP services, workflows, and compositions.
//!
//! This module provides the canonical `EncryptionConfig` used across the MCP subsystem
//! for encryption settings, algorithm selection, and key management.

use serde::{Deserialize, Serialize};

/// Encryption configuration
///
/// Controls encryption behavior for MCP services, workflows, and service compositions.
/// Supports various encryption algorithms and key management strategies.
///
/// # Examples
///
/// ```rust
/// use squirrel_mcp::config::EncryptionConfig;
///
/// let config = EncryptionConfig {
///     enabled: true,
///     algorithm: "AES-256-GCM".to_string(),
///     key_management: "vault".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EncryptionConfig {
    /// Encryption enabled
    ///
    /// When true, data is encrypted using the specified algorithm.
    /// When false, data is transmitted and stored in plain text.
    pub enabled: bool,
    
    /// Encryption algorithm
    ///
    /// The cryptographic algorithm to use for encryption/decryption.
    /// Common values:
    /// - "AES-256-GCM" (default, recommended)
    /// - "AES-128-GCM"
    /// - "ChaCha20-Poly1305"
    pub algorithm: String,
    
    /// Key management strategy
    ///
    /// How encryption keys are managed and stored.
    /// Common values:
    /// - "vault" - Use external key vault (e.g., HashiCorp Vault)
    /// - "local" - Use local key storage
    /// - "kms" - Use cloud KMS (AWS KMS, GCP KMS, Azure Key Vault)
    /// - "env" - Load from environment variables
    pub key_management: String,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            algorithm: "AES-256-GCM".to_string(),
            key_management: "local".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let config = EncryptionConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.algorithm, "AES-256-GCM");
        assert_eq!(config.key_management, "local");
    }

    #[test]
    fn test_serde() {
        let config = EncryptionConfig {
            enabled: true,
            algorithm: "ChaCha20-Poly1305".to_string(),
            key_management: "vault".to_string(),
        };

        let json = serde_json::to_string(&config).expect("should succeed");
        let deserialized: EncryptionConfig = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(config, deserialized);
    }
}

