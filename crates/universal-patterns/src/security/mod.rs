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
//! ```no_run
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
mod providers;
mod tests;
mod traits;
mod types;

// Public re-exports - Core types and traits
pub use client::UniversalSecurityClient;
pub use context::{HealthStatus, SecurityContext, SecurityHealth};
pub use errors::SecurityError;
pub use traits::{SecurityProvider, UniversalSecurityProvider};

// Public re-exports - Provider implementations
pub use providers::{BeardogIntegration, BeardogSecurityProvider, LocalSecurityProvider};

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

    LocalSecurityProvider::new(config).await
}

/// Check if the security module is properly initialized
pub async fn validate_initialization() -> Result<(), SecurityError> {
    // Create a local provider to ensure basic functionality works
    let provider = create_local_provider().await?;

    // Perform a basic health check
    let health = provider.health_check().await?;

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
