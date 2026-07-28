// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Security configuration types

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use url::Url;

/// Deprecated serde alias — use `"security_provider"` in config.
#[deprecated(since = "0.2.0", note = "use \"security_provider\" in config")]
pub const AUTH_METHOD_SERDE_BEARDOG: &str = "beardog";

/// Deprecated serde alias — use `"security_provider"` in config.
#[deprecated(since = "0.2.0", note = "use \"security_provider\" in config")]
pub const AUTH_METHOD_SERDE_BEARDOG_PASCAL: &str = "Beardog";

/// Security provider configuration (capability-based)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Security primal endpoint (discovered via capability)
    #[serde(alias = "beardog_endpoint")]
    pub security_endpoint: Option<Url>,

    /// Authentication method
    pub auth_method: AuthMethod,

    /// Token/credential storage
    pub credential_storage: CredentialStorage,

    /// Encryption settings
    pub encryption: EncryptionConfig,

    /// Enable audit logging
    pub audit_logging: bool,

    /// Security fallback settings
    pub fallback: SecurityFallback,
}

/// Security fallback configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFallback {
    /// Enable local fallback when security primal unavailable
    pub enable_local_fallback: bool,

    /// Local authentication method for fallback
    pub local_auth_method: AuthMethod,

    /// Fallback timeout (seconds)
    pub fallback_timeout: u64,
}

/// Authentication methods
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum AuthMethod {
    /// No authentication (development only)
    None,
    /// Token-based authentication
    Token {
        /// Path to the token file
        token_file: PathBuf,
    },
    /// Certificate-based authentication
    Certificate {
        /// Path to the certificate file
        cert_file: PathBuf,
        /// Path to the private key file
        key_file: PathBuf,
    },
    /// Security-provider-managed authentication (capability-based).
    ///
    /// Serializes as `"security_provider"`. Legacy aliases `"Beardog"` and `"beardog"`
    /// are deprecated — see [`AUTH_METHOD_SERDE_BEARDOG`].
    #[serde(rename = "security_provider", alias = "Beardog", alias = "beardog")]
    SecurityProvider {
        /// Service ID for security-provider authentication
        service_id: String,
    },
}

/// Credential storage options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CredentialStorage {
    /// In-memory storage (not persistent)
    Memory,
    /// File-based storage at an explicit path
    File {
        /// Path to the credential storage file
        path: PathBuf,
    },
    /// Security-provider-managed storage.
    ///
    /// Serializes as `"security_provider"`. Legacy aliases `"Beardog"` and `"beardog"`
    /// are deprecated — see [`AUTH_METHOD_SERDE_BEARDOG`].
    #[serde(rename = "security_provider", alias = "Beardog", alias = "beardog")]
    SecurityProvider,
    /// Platform credential cache — file-based fallback for offline/bootstrap.
    ///
    /// Auto-detects the OS-appropriate file path:
    /// - **Linux/Unix**: `$XDG_DATA_HOME/squirrel/secrets.json` (`0o600`)
    /// - **Windows**: `%APPDATA%\squirrel\secrets.json`
    /// - **macOS**: `~/Library/Application Support/squirrel/secrets.json`
    /// - **Android**: App-private file store (via data dir)
    ///
    /// This is a **local cache** for bootstrap/offline use.  For
    /// production-grade HSM-backed credentials, use [`SecurityProvider`]
    /// which delegates to the security provider via IPC.
    Platform,
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Enable encryption for inter-primal communication
    pub enable_inter_primal: bool,

    /// Enable encryption for data at rest
    pub enable_at_rest: bool,

    /// Encryption algorithm
    pub algorithm: EncryptionAlgorithm,

    /// Key management
    pub key_management: KeyManagement,
}

/// Encryption algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    /// AES-256-GCM
    Aes256Gcm,
    /// ChaCha20-Poly1305
    ChaCha20Poly1305,
}

/// Key management options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyManagement {
    /// File-based key storage
    File {
        /// Path to the key file
        path: PathBuf,
    },
    /// Security-provider-managed keys.
    ///
    /// Serializes as `"security_provider"`. Legacy aliases `"Beardog"` and `"beardog"`
    /// are deprecated — see [`AUTH_METHOD_SERDE_BEARDOG`].
    #[serde(rename = "security_provider", alias = "Beardog", alias = "beardog")]
    SecurityProvider,
    /// Environment variable
    Environment {
        /// Name of the environment variable containing the key
        var_name: String,
    },
}
