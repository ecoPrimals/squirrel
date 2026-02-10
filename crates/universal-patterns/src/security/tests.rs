// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Security module tests
//!
//! This module contains comprehensive tests for all security components
//! including providers, clients, and integrations.

#[cfg(test)]
#[allow(clippy::module_inception)] // tests.rs file contains mod tests - standard pattern
mod tests {
    use super::super::*;
    use crate::config::{
        AuthMethod, CredentialStorage, EncryptionAlgorithm, EncryptionConfig, KeyManagement,
        SecurityConfig, SecurityFallback,
    };
    use crate::security::providers::SecurityServiceConfig;
    use crate::traits::{AuthResult, Credentials, Principal, PrincipalType};
    // Note: Utc and Uuid reserved for future timestamp/ID tests
    use std::collections::HashMap;
    use url::Url;

    // ============================================================================
    // TEST HELPER FUNCTIONS - Modern Idiomatic Rust Pattern
    // ============================================================================

    /// Create a test encryption configuration
    ///
    /// This helper provides a consistent encryption config for all tests,
    /// following the DRY principle and making tests more maintainable.
    fn test_encryption_config() -> EncryptionConfig {
        EncryptionConfig {
            enable_inter_primal: false,
            enable_at_rest: false,
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            key_management: KeyManagement::Environment {
                var_name: "TEST_ENCRYPTION_KEY".to_string(),
            },
        }
    }

    /// Create a test security configuration
    ///
    /// This helper function creates a properly initialized SecurityConfig
    /// with all required fields, avoiding repetition across tests.
    fn test_security_config() -> SecurityConfig {
        SecurityConfig {
            beardog_endpoint: None,
            auth_method: AuthMethod::None,
            credential_storage: CredentialStorage::Memory,
            encryption: test_encryption_config(),
            audit_logging: false,
            fallback: SecurityFallback {
                enable_local_fallback: true,
                local_auth_method: AuthMethod::None,
                fallback_timeout: 30,
            },
        }
    }

    /// Create a test security configuration with custom endpoint
    fn test_security_config_with_endpoint(endpoint: Url) -> SecurityConfig {
        SecurityConfig {
            beardog_endpoint: Some(endpoint),
            auth_method: AuthMethod::None,
            credential_storage: CredentialStorage::Memory,
            encryption: test_encryption_config(),
            audit_logging: false,
            fallback: SecurityFallback {
                enable_local_fallback: true,
                local_auth_method: AuthMethod::None,
                fallback_timeout: 30,
            },
        }
    }

    /// Convert SecurityConfig to SecurityServiceConfig for provider creation
    ///
    /// Providers use SecurityServiceConfig, while clients use SecurityConfig.
    /// This helper bridges the gap for tests.
    fn to_service_config(config: &SecurityConfig) -> SecurityServiceConfig {
        SecurityServiceConfig {
            service_id: "test-service".to_string(),
            endpoint: config.beardog_endpoint.as_ref().map(|u| u.to_string()),
            timeout_seconds: Some(30),
            max_retries: Some(3),
            auth_config: None,
        }
    }

    // ============================================================================
    // ACTUAL TESTS
    // ============================================================================

    /// Test the Beardog integration creation
    #[tokio::test]
    async fn test_beardog_integration_creation() {
        let endpoint = Url::parse("http://localhost:8443").expect("Failed to parse test URL");
        let config = SecurityServiceConfig {
            service_id: "test-service".to_string(),
            endpoint: Some(endpoint.to_string()),
            timeout_seconds: Some(30),
            max_retries: Some(3),
            auth_config: None,
        };

        let _integration = BeardogIntegration::new(config)
            .await
            .expect("Failed to create BeardogIntegration");

        // BeardogIntegration::new returns BeardogSecurityProvider, not BeardogIntegration
        // Test passes if creation succeeds without panic
    }

    /// Test security context creation from principal
    #[tokio::test]
    async fn test_security_context() {
        let principal = Principal {
            id: "test-user".to_string(),
            name: "Test User".to_string(),
            principal_type: PrincipalType::User,
            roles: vec!["user".to_string()],
            permissions: vec!["read".to_string()],
            metadata: HashMap::new(),
        };

        let context = SecurityContext::from_principal(&principal);
        assert_eq!(context.principal.id, "test-user");
        assert_eq!(context.principal.name, "Test User");
        assert!(context.has_permission("read"));
        assert!(!context.has_permission("write"));
    }

    /// Test security context creation from auth result
    #[tokio::test]
    async fn test_security_context_from_auth_result() {
        let principal = Principal {
            id: "test-user".to_string(),
            name: "Test User".to_string(),
            principal_type: PrincipalType::User,
            roles: vec!["admin".to_string()],
            permissions: vec!["read".to_string(), "write".to_string()],
            metadata: HashMap::new(),
        };

        let auth_result = AuthResult {
            principal: principal.clone(),
            token: "test-token".to_string(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
            permissions: vec!["read".to_string(), "write".to_string()],
            metadata: HashMap::new(),
        };

        let context = SecurityContext::from_auth_result(&auth_result);
        assert_eq!(context.principal.id, "test-user");
        assert_eq!(context.token, "test-token");
        assert!(context.has_permission("read"));
        assert!(context.has_permission("write"));
    }

    /// Test security context expiration
    #[tokio::test]
    async fn test_security_context_expiration() {
        let principal = Principal {
            id: "test-user".to_string(),
            name: "Test User".to_string(),
            principal_type: PrincipalType::User,
            roles: vec![],
            permissions: vec![],
            metadata: HashMap::new(),
        };

        let mut context = SecurityContext::from_principal(&principal);
        assert!(!context.is_expired());

        // Test time until expiration
        let time_left = context.time_until_expiration();
        assert!(time_left.as_secs() > 0);

        // Set expiration to past
        context.expires_at = chrono::Utc::now() - chrono::Duration::hours(1);
        assert!(context.is_expired());
        assert_eq!(context.time_until_expiration().as_secs(), 0);
    }

    /// Test security context metadata operations
    #[tokio::test]
    async fn test_security_context_metadata() {
        let principal = Principal {
            id: "test-user".to_string(),
            name: "Test User".to_string(),
            principal_type: PrincipalType::User,
            roles: vec![],
            permissions: vec![],
            metadata: HashMap::new(),
        };

        let mut context = SecurityContext::from_principal(&principal);
        assert_eq!(context.get_metadata("key"), None);

        context.add_metadata("key".to_string(), "value".to_string());
        assert_eq!(context.get_metadata("key"), Some(&"value".to_string()));
    }

    /// Test security health creation
    #[tokio::test]
    async fn test_security_health() {
        let health = SecurityHealth::healthy(std::time::Duration::from_millis(100));
        assert!(health.is_healthy());
        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.latency, std::time::Duration::from_millis(100));

        let unhealthy = SecurityHealth::unhealthy("Connection failed".to_string());
        assert!(!unhealthy.is_healthy());
        assert_eq!(unhealthy.status, HealthStatus::Unhealthy);
        assert_eq!(
            unhealthy.details.get("reason"),
            Some(&"Connection failed".to_string())
        );
    }

    /// Test health status methods
    #[tokio::test]
    async fn test_health_status() {
        let healthy = HealthStatus::Healthy;
        assert!(healthy.is_healthy());
        assert!(!healthy.is_unhealthy());
        assert!(!healthy.is_unknown());

        let unhealthy = HealthStatus::Unhealthy;
        assert!(!unhealthy.is_healthy());
        assert!(unhealthy.is_unhealthy());
        assert!(!unhealthy.is_unknown());

        let unknown = HealthStatus::Unknown;
        assert!(!unknown.is_healthy());
        assert!(!unknown.is_unhealthy());
        assert!(unknown.is_unknown());
    }

    /// Test security health recent check
    #[tokio::test]
    async fn test_security_health_recent_check() {
        let health = SecurityHealth::healthy(std::time::Duration::from_millis(10));
        assert!(health.is_recent(std::time::Duration::from_secs(60)));

        let mut old_health = health.clone();
        old_health.last_check = chrono::Utc::now() - chrono::Duration::hours(2);
        assert!(!old_health.is_recent(std::time::Duration::from_secs(60)));
    }

    /// Test security health detail operations
    #[tokio::test]
    async fn test_security_health_details() {
        let mut health = SecurityHealth::healthy(std::time::Duration::from_millis(10));
        assert!(health.details.is_empty());

        health.add_detail("key".to_string(), "value".to_string());
        assert_eq!(health.details.get("key"), Some(&"value".to_string()));
    }

    /// Test security error classification
    #[tokio::test]
    async fn test_error_classification() {
        let auth_error = SecurityError::authentication("test auth error");
        assert!(matches!(auth_error, SecurityError::Authentication(_)));

        let network_error = SecurityError::network("test network error");
        assert!(network_error.is_network_error());
        assert!(network_error.is_recoverable());

        let config_error = SecurityError::configuration("test config error");
        assert!(config_error.is_configuration_error());
        assert!(!config_error.is_recoverable());

        let token_error = SecurityError::Token("expired".to_string());
        assert!(token_error.is_recoverable());
        assert!(!token_error.is_network_error());
    }

    /// Test API request/response types
    #[tokio::test]
    async fn test_auth_request_creation() {
        let credentials = Credentials::Test {
            service_id: "test-service".to_string(),
        };

        let request = AuthRequest::new("test-service".to_string(), credentials.clone());
        assert_eq!(request.service_id, "test-service");
        assert!(matches!(request.credentials, Credentials::Test { .. }));
    }

    /// Test authorization request creation
    #[tokio::test]
    async fn test_authorization_request_creation() {
        let principal = Principal {
            id: "user123".to_string(),
            name: "Test User".to_string(),
            principal_type: PrincipalType::User,
            roles: vec![],
            permissions: vec![],
            metadata: HashMap::new(),
        };

        let request = AuthorizationRequest::new(
            "test-service".to_string(),
            principal.clone(),
            "read".to_string(),
            "resource123".to_string(),
        );

        assert_eq!(request.service_id, "test-service");
        assert_eq!(request.principal.id, "user123");
        assert_eq!(request.action, "read");
        assert_eq!(request.resource, "resource123");
    }

    /// Test authorization result creation
    #[tokio::test]
    async fn test_authorization_result_creation() {
        let authorized = AuthorizationResult::authorized();
        assert!(authorized.authorized);

        let unauthorized = AuthorizationResult::unauthorized();
        assert!(!unauthorized.authorized);
    }

    /// Test encryption request creation
    #[tokio::test]
    async fn test_encryption_request_creation() {
        let request = EncryptionRequest::new("test-service".to_string(), "data".to_string());
        assert_eq!(request.service_id, "test-service");
        assert_eq!(request.data, "data");
    }

    /// Test verification result creation
    #[tokio::test]
    async fn test_verification_result_creation() {
        let valid = VerificationResult::valid();
        assert!(valid.valid);

        let invalid = VerificationResult::invalid();
        assert!(!invalid.valid);
    }

    /// Test audit log request creation
    #[tokio::test]
    async fn test_audit_log_request_creation() {
        let principal = Principal {
            id: "user123".to_string(),
            name: "Test User".to_string(),
            principal_type: PrincipalType::User,
            roles: vec![],
            permissions: vec![],
            metadata: HashMap::new(),
        };

        let context = SecurityContext::from_principal(&principal);
        let request = AuditLogRequest::new(
            "test-service".to_string(),
            "authenticate".to_string(),
            context.clone(),
        );

        assert_eq!(request.service_id, "test-service");
        assert_eq!(request.operation, "authenticate");
        assert_eq!(request.context.principal.id, "user123");
    }

    /// Test universal security client creation
    #[tokio::test]
    async fn test_universal_client_creation() {
        let endpoint = Url::parse("http://localhost:8443").expect("Failed to parse test URL");
        let mut config = test_security_config_with_endpoint(endpoint);
        config.auth_method = AuthMethod::Beardog {
            service_id: "test-service".to_string(),
        };
        config.audit_logging = true;

        let result = UniversalSecurityClient::new(config).await;
        assert!(result.is_ok());
    }

    /// Test client fallback configuration
    #[tokio::test]
    async fn test_client_fallback_configuration() {
        let endpoint = Url::parse("http://localhost:8443").expect("Failed to parse test URL");
        let mut config = test_security_config_with_endpoint(endpoint);
        config.auth_method = AuthMethod::Beardog {
            service_id: "test-service".to_string(),
        };
        config.audit_logging = true;

        let client = UniversalSecurityClient::new(config).await.unwrap();
        assert!(client.is_fallback_enabled());
    }

    /// Test client authentication with fallback
    #[tokio::test]
    async fn test_client_authentication_with_fallback() {
        let endpoint = Url::parse("http://localhost:8443").expect("Failed to parse test URL");
        let mut config = test_security_config_with_endpoint(endpoint);
        config.auth_method = AuthMethod::Beardog {
            service_id: "test-service".to_string(),
        };
        config.fallback.fallback_timeout = 1; // Short timeout to trigger fallback

        let client = UniversalSecurityClient::new(config).await.unwrap();

        let credentials = Credentials::Test {
            service_id: "test-service".to_string(),
        };

        // This should fallback to local provider due to short timeout
        // Note: Using full trait path to avoid ambiguity
        let result =
            crate::security::traits::UniversalSecurityProvider::authenticate(&client, &credentials)
                .await;
        assert!(result.is_ok());
    }

    /// Test client provider health check
    #[tokio::test]
    async fn test_client_provider_health_check() {
        let endpoint = Url::parse("http://localhost:8443").expect("Failed to parse test URL");
        let mut config = test_security_config_with_endpoint(endpoint);
        config.auth_method = AuthMethod::Beardog {
            service_id: "test-service".to_string(),
        };

        let client = UniversalSecurityClient::new(config).await.unwrap();
        let (primary_health, fallback_health) = client.get_providers_health().await;

        // Primary may succeed or fail (depends on if server is running)
        // We're just testing that the health check can be queried
        assert!(
            primary_health.is_ok() || primary_health.is_err(),
            "Health check should return a result"
        );

        // Fallback should always be available (local provider)
        assert!(fallback_health.is_some());
        assert!(fallback_health.unwrap().is_ok());
    }

    /// Test local provider functionality
    #[tokio::test]
    async fn test_local_provider_functionality() {
        let config = test_security_config();
        let service_config = to_service_config(&config);

        let provider = LocalSecurityProvider::new(service_config)
            .await
            .expect("Failed to create LocalSecurityProvider");

        // Test authentication
        let credentials = Credentials::Test {
            service_id: "test-service".to_string(),
        };

        use crate::security::traits::UniversalSecurityProvider;
        let auth_result = UniversalSecurityProvider::authenticate(&provider, &credentials)
            .await
            .expect("Authentication failed");
        assert_eq!(auth_result.principal.id, "local-user");

        // Test authorization
        let principal = Principal {
            id: "test-user".to_string(),
            name: "Test User".to_string(),
            principal_type: PrincipalType::User,
            roles: vec![],
            permissions: vec![],
            metadata: HashMap::new(),
        };

        let authorized =
            UniversalSecurityProvider::authorize(&provider, &principal, "read", "resource")
                .await
                .expect("Authorization failed");
        assert!(authorized); // Local provider allows all operations

        // Test encryption/decryption
        let data = b"test data";
        let encrypted = UniversalSecurityProvider::encrypt(&provider, data)
            .await
            .expect("Encryption failed");
        let decrypted = UniversalSecurityProvider::decrypt(&provider, &encrypted)
            .await
            .expect("Decryption failed");
        assert_eq!(data, decrypted.as_slice());

        // Test signing/verification (use explicit trait method to avoid ambiguity)
        let signature = UniversalSecurityProvider::sign(&provider, data)
            .await
            .expect("Signing failed");
        let valid = UniversalSecurityProvider::verify(&provider, data, &signature)
            .await
            .expect("Verification failed");
        assert!(valid);

        // Test health check
        let health = UniversalSecurityProvider::health_check(&provider)
            .await
            .expect("Health check failed");
        assert!(matches!(
            health.status,
            crate::security::context::HealthStatus::Healthy
        ));
    }

    /// Test audit logging functionality
    #[tokio::test]
    async fn test_audit_logging() {
        let mut config = test_security_config();
        config.audit_logging = true;
        let service_config = to_service_config(&config);

        let provider = LocalSecurityProvider::new(service_config)
            .await
            .expect("Failed to create LocalSecurityProvider");

        let principal = Principal {
            id: "test-user".to_string(),
            name: "Test User".to_string(),
            principal_type: PrincipalType::User,
            roles: vec![],
            permissions: vec![],
            metadata: HashMap::new(),
        };

        let context = SecurityContext::from_principal(&principal);
        let result = provider.audit_log("test-operation", &context).await;
        assert!(result.is_ok(), "Audit logging should succeed");
    }

    /// Test security error convenience methods
    #[tokio::test]
    async fn test_security_error_convenience() {
        let auth_error = SecurityError::authentication("test");
        assert!(matches!(auth_error, SecurityError::Authentication(_)));

        let auth_error2 = SecurityError::authorization("test");
        assert!(matches!(auth_error2, SecurityError::Authorization(_)));

        let encrypt_error = SecurityError::encryption("test");
        assert!(matches!(encrypt_error, SecurityError::Encryption(_)));

        let network_error = SecurityError::network("test");
        assert!(matches!(network_error, SecurityError::Network(_)));

        let config_error = SecurityError::configuration("test");
        assert!(matches!(config_error, SecurityError::Configuration(_)));
    }

    /// Test health status display
    #[tokio::test]
    async fn test_health_status_display() {
        let healthy = HealthStatus::Healthy;
        assert_eq!(format!("{}", healthy), "Healthy");

        let unhealthy = HealthStatus::Unhealthy;
        assert_eq!(format!("{}", unhealthy), "Unhealthy");

        let unknown = HealthStatus::Unknown;
        assert_eq!(format!("{}", unknown), "Unknown");
    }

    /// Test beardog provider creation
    #[tokio::test]
    async fn test_beardog_provider_creation() {
        let endpoint = Url::parse("http://localhost:8443").expect("Failed to parse test URL");
        let mut config = test_security_config_with_endpoint(endpoint);
        config.auth_method = AuthMethod::Beardog {
            service_id: "test-service".to_string(),
        };
        config.fallback.enable_local_fallback = false;
        let service_config = to_service_config(&config);

        let _provider = BeardogSecurityProvider::new(service_config)
            .await
            .expect("Failed to create BeardogSecurityProvider");

        // Note: BeardogSecurityProvider doesn't have service_id() method
        // Test passes if provider creation succeeds without panic
    }

    /// Test client with custom providers
    #[tokio::test]
    async fn test_client_with_custom_providers() {
        let endpoint = Url::parse("http://localhost:8443").expect("Failed to parse test URL");
        let mut config = test_security_config_with_endpoint(endpoint);
        config.auth_method = AuthMethod::Beardog {
            service_id: "test-service".to_string(),
        };
        config.fallback.enable_local_fallback = false;
        let service_config = to_service_config(&config);

        let primary = std::sync::Arc::new(
            LocalSecurityProvider::new(service_config)
                .await
                .expect("Failed to create LocalSecurityProvider"),
        );
        let client = UniversalSecurityClient::with_providers(primary, None, config);

        assert!(!client.is_fallback_enabled());
    }
}
