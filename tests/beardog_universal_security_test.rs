//! Universal Security Integration Tests
//!
//! This test suite validates the universal security patterns and Beardog integration
//! implemented as part of the standardized security architecture.

use std::collections::HashMap;
use universal_patterns::prelude::*;
use universal_patterns::config::{SecurityConfig, SecurityFallback, AuthMethod, CredentialStorage, EncryptionConfig, EncryptionAlgorithm, KeyManagement};
use universal_patterns::security::{UniversalSecurityClient, UniversalSecurityProvider, SecurityContext, HealthStatus};
use universal_patterns::traits::{Credentials, Principal, PrincipalType};
use url::Url;

/// Test universal security client creation and configuration
#[tokio::test]
async fn test_universal_security_client_creation() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Create test security configuration
    let config = SecurityConfig {
        beardog_endpoint: Some(Url::parse("http://localhost:8443").unwrap()),
        auth_method: AuthMethod::Beardog {
            service_id: "test-service".to_string(),
        },
        credential_storage: CredentialStorage::Memory,
        encryption: EncryptionConfig {
            enable_inter_primal: true,
            enable_at_rest: true,
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            key_management: KeyManagement::Environment {
                var_name: "TEST_ENCRYPTION_KEY".to_string(),
            },
        },
        audit_logging: true,
        fallback: SecurityFallback {
            enable_local_fallback: true,
            local_auth_method: AuthMethod::None,
            fallback_timeout: 30,
        },
    };

    // Set test environment variable
    std::env::set_var("TEST_ENCRYPTION_KEY", "test-key-32-chars-long-for-aes256");

    // Create universal security client
    let security_client = UniversalSecurityClient::new(config).await?;

    // Verify client was created successfully
    assert!(security_client.health_check().await.is_ok());

    // Clean up
    std::env::remove_var("TEST_ENCRYPTION_KEY");

    Ok(())
}

/// Test fallback mechanism when Beardog is unavailable
#[tokio::test]
async fn test_fallback_mechanism() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Create configuration with fallback enabled
    let config = SecurityConfig {
        beardog_endpoint: Some(Url::parse("http://invalid-endpoint:9999").unwrap()), // Invalid endpoint
        auth_method: AuthMethod::Beardog {
            service_id: "test-service".to_string(),
        },
        credential_storage: CredentialStorage::Memory,
        encryption: EncryptionConfig {
            enable_inter_primal: false,
            enable_at_rest: false,
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            key_management: KeyManagement::Environment {
                var_name: "TEST_ENCRYPTION_KEY".to_string(),
            },
        },
        audit_logging: false,
        fallback: SecurityFallback {
            enable_local_fallback: true,
            local_auth_method: AuthMethod::None,
            fallback_timeout: 5,
        },
    };

    // Set test environment variable
    std::env::set_var("TEST_ENCRYPTION_KEY", "test-key-32-chars-long-for-aes256");

    // Create universal security client
    let security_client = UniversalSecurityClient::new(config).await?;

    // Test authentication with fallback
    let test_credentials = Credentials::Test {
        service_id: "test-service".to_string(),
    };

    let auth_result = security_client.authenticate(&test_credentials).await?;
    assert_eq!(auth_result.principal.name, "Test User");
    assert_eq!(auth_result.token, "local-fallback-token");
    assert!(auth_result.permissions.contains(&"read".to_string()));

    // Test authorization with fallback (should allow all)
    let authorized = security_client.authorize(&auth_result.principal, "read", "test-resource").await?;
    assert!(authorized);

    // Test encryption/decryption with fallback
    let test_data = b"test encryption data";
    let encrypted = security_client.encrypt(test_data).await?;
    let decrypted = security_client.decrypt(&encrypted).await?;
    assert_eq!(decrypted, test_data);

    // Test audit logging with fallback
    let context = SecurityContext::from_principal(&auth_result.principal);
    security_client.audit_log("test-operation", &context).await?;

    // Clean up
    std::env::remove_var("TEST_ENCRYPTION_KEY");

    Ok(())
}

/// Test security context creation and management
#[tokio::test]
async fn test_security_context_management() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Create test principal
    let principal = Principal {
        id: "test-user-123".to_string(),
        name: "Test User".to_string(),
        principal_type: PrincipalType::User,
        roles: vec!["user".to_string(), "admin".to_string()],
        permissions: vec!["read".to_string(), "write".to_string()],
        metadata: HashMap::new(),
    };

    // Create security context from principal
    let context = SecurityContext::from_principal(&principal);
    
    // Verify context properties
    assert_eq!(context.principal.id, "test-user-123");
    assert_eq!(context.principal.name, "Test User");
    assert_eq!(context.principal.principal_type, PrincipalType::User);
    assert!(context.principal.roles.contains(&"user".to_string()));
    assert!(context.principal.permissions.contains(&"read".to_string()));
    assert!(!context.token.is_empty());

    Ok(())
}

/// Test security health monitoring
#[tokio::test]
async fn test_security_health_monitoring() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Create test configuration with local fallback
    let config = SecurityConfig {
        beardog_endpoint: None, // No Beardog endpoint configured
        auth_method: AuthMethod::None,
        credential_storage: CredentialStorage::Memory,
        encryption: EncryptionConfig {
            enable_inter_primal: false,
            enable_at_rest: false,
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            key_management: KeyManagement::Environment {
                var_name: "TEST_ENCRYPTION_KEY".to_string(),
            },
        },
        audit_logging: false,
        fallback: SecurityFallback {
            enable_local_fallback: true,
            local_auth_method: AuthMethod::None,
            fallback_timeout: 30,
        },
    };

    // Set test environment variable
    std::env::set_var("TEST_ENCRYPTION_KEY", "test-key-32-chars-long-for-aes256");

    // Create universal security client
    let security_client = UniversalSecurityClient::new(config).await?;

    // Test health check
    let health = security_client.health_check().await?;
    assert_eq!(health.status, HealthStatus::Healthy);
    assert!(health.latency.as_millis() < 100); // Should be very fast for local fallback

    // Clean up
    std::env::remove_var("TEST_ENCRYPTION_KEY");

    Ok(())
}

/// Test configuration builder patterns
#[tokio::test]
async fn test_configuration_builder_patterns() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Test development security configuration
    let dev_config = ConfigBuilder::development()
        .enable_local_fallback()
        .fallback_timeout(10)
        .build()?;

    assert!(dev_config.security.fallback.enable_local_fallback);
    assert_eq!(dev_config.security.fallback.fallback_timeout, 10);

    // Test production security configuration 
    let prod_config = ConfigBuilder::production()
        .beardog_endpoint("https://prod-beardog.domain.com:8443")?
        .beardog_auth("prod-service")
        .disable_fallback()
        .build()?;

    assert!(!prod_config.security.fallback.enable_local_fallback);
    assert!(prod_config.security.beardog_endpoint.is_some());
    assert!(matches!(prod_config.security.auth_method, AuthMethod::Beardog { .. }));

    // Test Squirrel-specific configuration
    let squirrel_config = ConfigBuilder::squirrel()
        .beardog_endpoint("https://squirrel-beardog.domain.com:8443")?
        .beardog_auth("squirrel-production")
        .enable_audit_logging()
        .enable_inter_primal_encryption()
        .build()?;

    assert_eq!(squirrel_config.info.name, "squirrel");
    assert!(squirrel_config.security.audit_logging);
    assert!(squirrel_config.security.encryption.enable_inter_primal);

    Ok(())
}

/// Test credential variants
#[tokio::test]
async fn test_credential_variants() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Test different credential types
    let api_key_creds = Credentials::ApiKey {
        key: "test-api-key".to_string(),
        service_id: "test-service".to_string(),
    };

    let bearer_creds = Credentials::Bearer {
        token: "test-bearer-token".to_string(),
    };

    let service_creds = Credentials::ServiceAccount {
        service_id: "test-service".to_string(),
        api_key: "test-service-key".to_string(),
    };

    let bootstrap_creds = Credentials::Bootstrap {
        service_id: "test-service".to_string(),
    };

    let test_creds = Credentials::Test {
        service_id: "test-service".to_string(),
    };

    // Create fallback-only security client
    let config = SecurityConfig {
        beardog_endpoint: None,
        auth_method: AuthMethod::None,
        credential_storage: CredentialStorage::Memory,
        encryption: EncryptionConfig {
            enable_inter_primal: false,
            enable_at_rest: false,
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            key_management: KeyManagement::Environment {
                var_name: "TEST_ENCRYPTION_KEY".to_string(),
            },
        },
        audit_logging: false,
        fallback: SecurityFallback {
            enable_local_fallback: true,
            local_auth_method: AuthMethod::None,
            fallback_timeout: 30,
        },
    };

    std::env::set_var("TEST_ENCRYPTION_KEY", "test-key-32-chars-long-for-aes256");
    let security_client = UniversalSecurityClient::new(config).await?;

    // Test authentication with test credentials (should work with fallback)
    let auth_result = security_client.authenticate(&test_creds).await?;
    assert_eq!(auth_result.principal.name, "Test User");

    // Other credential types should fail with local fallback
    assert!(security_client.authenticate(&api_key_creds).await.is_err());
    assert!(security_client.authenticate(&bearer_creds).await.is_err());
    assert!(security_client.authenticate(&service_creds).await.is_err());
    assert!(security_client.authenticate(&bootstrap_creds).await.is_err());

    std::env::remove_var("TEST_ENCRYPTION_KEY");

    Ok(())
}

/// Test encryption and signing operations
#[tokio::test]
async fn test_encryption_and_signing() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Create fallback-only security client
    let config = SecurityConfig {
        beardog_endpoint: None,
        auth_method: AuthMethod::None,
        credential_storage: CredentialStorage::Memory,
        encryption: EncryptionConfig {
            enable_inter_primal: true,
            enable_at_rest: true,
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            key_management: KeyManagement::Environment {
                var_name: "TEST_ENCRYPTION_KEY".to_string(),
            },
        },
        audit_logging: false,
        fallback: SecurityFallback {
            enable_local_fallback: true,
            local_auth_method: AuthMethod::None,
            fallback_timeout: 30,
        },
    };

    std::env::set_var("TEST_ENCRYPTION_KEY", "test-key-32-chars-long-for-aes256");
    let security_client = UniversalSecurityClient::new(config).await?;

    // Test encryption/decryption
    let test_data = b"sensitive data to encrypt";
    let encrypted = security_client.encrypt(test_data).await?;
    assert_ne!(encrypted, test_data); // Should be different when encrypted
    
    let decrypted = security_client.decrypt(&encrypted).await?;
    assert_eq!(decrypted, test_data); // Should match original

    // Test signing/verification
    let test_data_to_sign = b"data to sign";
    let signature = security_client.sign(test_data_to_sign).await?;
    assert!(!signature.is_empty());

    let is_valid = security_client.verify(test_data_to_sign, &signature).await?;
    assert!(is_valid);

    // Test verification with different data (should fail)
    let different_data = b"different data";
    let is_valid_different = security_client.verify(different_data, &signature).await?;
    assert!(!is_valid_different);

    std::env::remove_var("TEST_ENCRYPTION_KEY");

    Ok(())
}

/// Test security client with different authentication methods
#[tokio::test]
async fn test_different_auth_methods() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Test with Bearer token auth method
    let bearer_config = SecurityConfig {
        beardog_endpoint: None,
        auth_method: AuthMethod::Token {
            token_file: std::path::PathBuf::from("test-token.txt"),
        },
        credential_storage: CredentialStorage::Memory,
        encryption: EncryptionConfig {
            enable_inter_primal: false,
            enable_at_rest: false,
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            key_management: KeyManagement::Environment {
                var_name: "TEST_ENCRYPTION_KEY".to_string(),
            },
        },
        audit_logging: false,
        fallback: SecurityFallback {
            enable_local_fallback: true,
            local_auth_method: AuthMethod::None,
            fallback_timeout: 30,
        },
    };

    std::env::set_var("TEST_ENCRYPTION_KEY", "test-key-32-chars-long-for-aes256");
    let security_client = UniversalSecurityClient::new(bearer_config).await?;

    // Should still work with fallback
    let health = security_client.health_check().await?;
    assert_eq!(health.status, HealthStatus::Healthy);

    std::env::remove_var("TEST_ENCRYPTION_KEY");

    Ok(())
}

/// Helper module for test utilities
mod test_utils {
    use super::*;
    
    /// Create a test security configuration
    pub fn create_test_config() -> SecurityConfig {
        SecurityConfig {
            beardog_endpoint: None,
            auth_method: AuthMethod::None,
            credential_storage: CredentialStorage::Memory,
            encryption: EncryptionConfig {
                enable_inter_primal: false,
                enable_at_rest: false,
                algorithm: EncryptionAlgorithm::Aes256Gcm,
                key_management: KeyManagement::Environment {
                    var_name: "TEST_ENCRYPTION_KEY".to_string(),
                },
            },
            audit_logging: false,
            fallback: SecurityFallback {
                enable_local_fallback: true,
                local_auth_method: AuthMethod::None,
                fallback_timeout: 30,
            },
        }
    }
    
    /// Create a test principal
    pub fn create_test_principal() -> Principal {
        Principal {
            id: "test-user-123".to_string(),
            name: "Test User".to_string(),
            principal_type: PrincipalType::User,
            roles: vec!["user".to_string()],
            permissions: vec!["read".to_string()],
            metadata: HashMap::new(),
        }
    }
    
    /// Create test credentials
    pub fn create_test_credentials() -> Credentials {
        Credentials::Test {
            service_id: "test-service".to_string(),
        }
    }
} 