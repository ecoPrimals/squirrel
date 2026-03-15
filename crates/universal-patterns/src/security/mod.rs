// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Security integration module for universal patterns
//!
//! This module provides security patterns and integration with Beardog
//! for consistent security across all primals.
//!
//! # Architecture
//!
//! The security module is organized into several focused components:
//!
//! - **Errors**: Comprehensive error types for security operations
//! - **Traits**: Core security provider interfaces
//! - **Context**: Security context and health monitoring types
//! - **Client**: Universal security client with fallback capabilities
//! - **Providers**: Security provider implementations (Beardog, local, etc.)
//! - **Types**: API request/response structures
//! - **Tests**: Comprehensive test suite
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```ignore
//! use universal_patterns::security::UniversalSecurityClient;
//! use universal_patterns::config::SecurityConfig;
//! use universal_patterns::traits::Credentials;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create security configuration
//! let config = SecurityConfig::default();
//!
//! // Create universal security client
//! let client = UniversalSecurityClient::new(config).await?;
//!
//! // Authenticate user
//! let credentials = Credentials::Test {
//!     username: "user".to_string(),
//!     password: "pass".to_string(),
//! };
//!
//! let auth_result = client.authenticate(&credentials).await?;
//! println!("Authenticated user: {}", auth_result.principal.name);
//!
//! // Authorize action
//! let authorized = client.authorize(
//!     &auth_result.principal,
//!     "read",
//!     "resource"
//! ).await?;
//!
//! if authorized {
//!     println!("Access granted!");
//! }
//! # Ok(())
//! # }
//! ```

// Internal modules
mod client;
mod context;
mod errors;
mod hardening;
mod providers;
mod tests;
mod traits;
mod types;
mod zero_copy;

// Public re-exports - Core types and traits
pub use client::UniversalSecurityClient;
pub use context::{HealthStatus, SecurityContext, SecurityHealth};
pub use errors::SecurityError;
pub use traits::{SecurityProvider, UniversalSecurityProvider};

// Public re-exports - Provider implementations
pub use providers::{BeardogIntegration, BeardogSecurityProvider, LocalSecurityProvider};

// Public re-exports - Zero-copy types for high performance
pub use zero_copy::{
    CacheStats, CredentialsBuilder, PrincipalCache, PrincipalType, ZeroCopyAuthRequest,
    ZeroCopyAuthResult, ZeroCopyAuthzRequest, ZeroCopyCredentials, ZeroCopyPrincipal,
    ZeroCopySecurityProvider,
};

// Public re-exports - Security hardening for production
pub use hardening::{
    AuthRateLimitError, Environment, RiskLevel, SecurityHardening, SecurityHardeningConfig,
    SecurityIncident, SecurityMetrics, initialize_production_security,
};

// Public re-exports - API types
pub use types::{
    AuditLogRequest, AuthRequest, AuthorizationRequest, AuthorizationResult, DecryptionRequest,
    DecryptionResult, EncryptionRequest, EncryptionResult, SigningRequest, SigningResult,
    VerificationRequest, VerificationResult,
};

/// Security module version
pub const VERSION: &str = "1.0.0";

/// Create a new universal security client with default configuration
pub async fn create_default_client() -> Result<UniversalSecurityClient, SecurityError> {
    use crate::config::{
        AuthMethod, CredentialStorage, EncryptionAlgorithm, EncryptionConfig, KeyManagement,
        SecurityConfig, SecurityFallback,
    };
    use std::path::PathBuf;

    let config = SecurityConfig {
        beardog_endpoint: None,
        auth_method: AuthMethod::None,
        credential_storage: CredentialStorage::Memory,
        encryption: EncryptionConfig {
            enable_inter_primal: false,
            enable_at_rest: false,
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            key_management: KeyManagement::File {
                path: PathBuf::from("keys/encryption.key"),
            },
        },
        audit_logging: false,
        fallback: SecurityFallback {
            enable_local_fallback: true,
            local_auth_method: AuthMethod::None,
            fallback_timeout: 5,
        },
    };

    UniversalSecurityClient::new(config).await
}

/// Create a new universal security client with Beardog configuration
pub async fn create_beardog_client(
    endpoint: url::Url,
    service_id: String,
    enable_fallback: bool,
) -> Result<UniversalSecurityClient, SecurityError> {
    use crate::config::{
        AuthMethod, CredentialStorage, EncryptionAlgorithm, EncryptionConfig, KeyManagement,
        SecurityConfig, SecurityFallback,
    };

    let config = SecurityConfig {
        auth_method: AuthMethod::Beardog { service_id },
        beardog_endpoint: Some(endpoint),
        credential_storage: CredentialStorage::Beardog,
        encryption: EncryptionConfig {
            enable_inter_primal: true,
            enable_at_rest: true,
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            key_management: KeyManagement::Beardog,
        },
        audit_logging: true,
        fallback: SecurityFallback {
            enable_local_fallback: enable_fallback,
            local_auth_method: AuthMethod::None,
            fallback_timeout: 5,
        },
    };

    UniversalSecurityClient::new(config).await
}

/// Create a new local security provider
pub async fn create_local_provider() -> Result<LocalSecurityProvider, SecurityError> {
    use providers::SecurityServiceConfig;

    let config = SecurityServiceConfig {
        service_id: "local-security".to_string(),
        endpoint: None,
        timeout_seconds: Some(30),
        max_retries: Some(3),
        auth_config: None,
    };

    // Create the provider using the new method
    providers::LocalSecurityProvider::new(config).await
}

/// Check if the security module is properly initialized
pub async fn validate_initialization() -> Result<(), SecurityError> {
    use providers::UniversalSecurityService;

    // Create a local provider to ensure basic functionality works
    let provider = create_local_provider().await?;

    // Perform a basic health check
    let health = UniversalSecurityService::health_check(&provider).await?;

    if !health.is_healthy() {
        return Err(SecurityError::Configuration(
            "Security module health check failed".to_string(),
        ));
    }

    Ok(())
}

/// Get security module information
pub fn get_module_info() -> std::collections::HashMap<String, String> {
    let mut info = std::collections::HashMap::new();

    info.insert("version".to_string(), VERSION.to_string());
    info.insert("name".to_string(), "Universal Security Module".to_string());
    info.insert(
        "description".to_string(),
        "Security integration module for universal patterns".to_string(),
    );
    info.insert("supports_beardog".to_string(), "true".to_string());
    info.insert("supports_local_fallback".to_string(), "true".to_string());
    info.insert("supports_audit_logging".to_string(), "true".to_string());
    info.insert("supports_health_monitoring".to_string(), "true".to_string());
    info.insert("supports_encryption".to_string(), "true".to_string());
    info.insert(
        "supports_digital_signatures".to_string(),
        "true".to_string(),
    );
    info.insert("thread_safe".to_string(), "true".to_string());

    info
}

#[cfg(test)]
mod module_tests {
    use super::*;

    #[test]
    fn test_version_constant() {
        assert_eq!(VERSION, "1.0.0");
    }

    #[test]
    fn test_get_module_info_keys() {
        let info = get_module_info();
        assert!(info.contains_key("version"));
        assert!(info.contains_key("name"));
        assert!(info.contains_key("description"));
        assert!(info.contains_key("supports_beardog"));
        assert!(info.contains_key("supports_local_fallback"));
        assert!(info.contains_key("supports_audit_logging"));
        assert!(info.contains_key("supports_health_monitoring"));
        assert!(info.contains_key("supports_encryption"));
        assert!(info.contains_key("supports_digital_signatures"));
        assert!(info.contains_key("thread_safe"));
    }

    #[test]
    fn test_get_module_info_values() {
        let info = get_module_info();
        assert_eq!(info.get("version").unwrap(), "1.0.0");
        assert_eq!(info.get("name").unwrap(), "Universal Security Module");
        assert_eq!(info.get("supports_beardog").unwrap(), "true");
        assert_eq!(info.get("thread_safe").unwrap(), "true");
    }

    #[test]
    fn test_get_module_info_count() {
        let info = get_module_info();
        assert_eq!(info.len(), 10);
    }

    #[tokio::test]
    async fn test_create_default_client() {
        // Should create a client with default local configuration
        let result = create_default_client().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_local_provider() {
        let result = create_local_provider().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_initialization() {
        let result = validate_initialization().await;
        assert!(result.is_ok());
    }
}
