// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Security configuration types

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use url::Url;

/// Security configuration for Beardog integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Beardog service endpoint
    pub beardog_endpoint: Option<Url>,

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
    /// Enable local fallback when Beardog unavailable
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
    /// Beardog-managed authentication
    Beardog {
        /// Service ID for Beardog authentication
        service_id: String,
    },
}

/// Credential storage options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CredentialStorage {
    /// In-memory storage (not persistent)
    Memory,
    /// File-based storage
    File {
        /// Path to the credential storage file
        path: PathBuf,
    },
    /// Beardog-managed storage
    Beardog,
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
    /// Beardog-managed keys
    Beardog,
    /// Environment variable
    Environment {
        /// Name of the environment variable containing the key
        var_name: String,
    },
}
