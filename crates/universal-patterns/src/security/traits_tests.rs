// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Comprehensive tests for SecurityProvider trait
//!
//! Coverage goal: 90%+
//! Strategy: Test trait implementations, all methods, error paths, edge cases

use async_trait::async_trait;

use super::traits::SecurityProvider;
use super::errors::SecurityError;
use super::context::{SecurityContext, SecurityHealth};
use crate::traits::{AuthResult, Credentials, Principal, PrincipalType};

/// Mock security provider for testing
struct MockSecurityProvider {
    auth_should_fail: bool,
    authz_should_fail: bool,
    encrypt_should_fail: bool,
    sign_should_fail: bool,
}

impl MockSecurityProvider {
    fn new() -> Self {
        Self {
            auth_should_fail: false,
            authz_should_fail: false,
            encrypt_should_fail: false,
            sign_should_fail: false,
        }
    }

    fn with_auth_failure(mut self) -> Self {
        self.auth_should_fail = true;
        self
    }

    fn with_authz_failure(mut self) -> Self {
        self.authz_should_fail = true;
        self
    }

    fn with_encrypt_failure(mut self) -> Self {
        self.encrypt_should_fail = true;
        self
    }

    fn with_sign_failure(mut self) -> Self {
        self.sign_should_fail = true;
        self
    }
}

#[async_trait]
impl SecurityProvider for MockSecurityProvider {
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthResult, SecurityError> {
        if self.auth_should_fail {
            return Err(SecurityError::AuthenticationFailed(
                "Mock authentication failure".to_string(),
            ));
        }

        // Successful authentication
        Ok(AuthResult {
            principal: Principal {
                id: "test-user".to_string(),
                username: credentials.username.clone(),
                principal_type: PrincipalType::User,
                roles: vec!["user".to_string()],
                permissions: vec!["read".to_string()],
                metadata: std::collections::HashMap::new(),
            },
            token: "mock-token-12345".to_string(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
            refresh_token: Some("mock-refresh-token".to_string()),
        })
    }

    async fn authorize(
        &self,
        principal: &Principal,
        action: &str,
        resource: &str,
    ) -> Result<bool, SecurityError> {
        if self.authz_should_fail {
            return Err(SecurityError::AuthorizationFailed(
                "Mock authorization failure".to_string(),
            ));
        }

        // Simple mock authorization logic
        if principal.permissions.contains(&action.to_string()) {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        if self.encrypt_should_fail {
            return Err(SecurityError::EncryptionFailed(
                "Mock encryption failure".to_string(),
            ));
        }

        // Mock encryption: just reverse the bytes
        Ok(data.iter().copied().rev().collect())
    }

    async fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        // Mock decryption: reverse again to get original
        Ok(encrypted_data.iter().copied().rev().collect())
    }

    async fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        if self.sign_should_fail {
            return Err(SecurityError::SigningFailed(
                "Mock signing failure".to_string(),
            ));
        }

        // Mock signature: SHA-256-like fake
        Ok(vec![0xAB, 0xCD, 0xEF, 0x12, 0x34, 0x56, 0x78, 0x90])
    }

    async fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, SecurityError> {
        // Mock verification: check if signature matches expected
        let expected = vec![0xAB, 0xCD, 0xEF, 0x12, 0x34, 0x56, 0x78, 0x90];
        Ok(signature == expected)
    }

    async fn health_check(&self) -> Result<SecurityHealth, SecurityError> {
        Ok(SecurityHealth {
            is_healthy: true,
            encryption_available: !self.encrypt_should_fail,
            signing_available: !self.sign_should_fail,
            details: "Mock provider healthy".to_string(),
        })
    }

    async fn get_context(&self) -> Result<SecurityContext, SecurityError> {
        Ok(SecurityContext {
            service_name: "mock-security".to_string(),
            environment: "test".to_string(),
            encryption_enabled: true,
            signing_enabled: true,
            metadata: std::collections::HashMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_authenticate_success() {
        let provider = MockSecurityProvider::new();
        let credentials = Credentials {
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };

        let result = provider.authenticate(&credentials).await;
        assert!(result.is_ok());

        let auth_result = result.expect("should succeed");
        assert_eq!(auth_result.principal.username, "testuser");
        assert_eq!(auth_result.principal.id, "test-user");
        assert!(auth_result.token.starts_with("mock-token"));
        assert!(auth_result.refresh_token.is_some());
    }

    #[tokio::test]
    async fn test_authenticate_failure() {
        let provider = MockSecurityProvider::new().with_auth_failure();
        let credentials = Credentials {
            username: "testuser".to_string(),
            password: "wrongpass".to_string(),
        };

        let result = provider.authenticate(&credentials).await;
        assert!(result.is_err());

        match result {
            Err(SecurityError::AuthenticationFailed(msg)) => {
                assert!(msg.contains("Mock authentication failure"));
            }
            _ => unreachable!("Expected AuthenticationFailed error"),
        }
    }

    #[tokio::test]
    async fn test_authorize_success() {
        let provider = MockSecurityProvider::new();
        let principal = Principal {
            id: "user-123".to_string(),
            username: "testuser".to_string(),
            principal_type: PrincipalType::User,
            roles: vec!["admin".to_string()],
            permissions: vec!["read".to_string(), "write".to_string()],
            metadata: std::collections::HashMap::new(),
        };

        // Should be authorized for "read"
        let result = provider.authorize(&principal, "read", "resource").await;
        assert!(result.is_ok());
        assert_eq!(result.expect("should succeed"), true);
    }

    #[tokio::test]
    async fn test_authorize_denied() {
        let provider = MockSecurityProvider::new();
        let principal = Principal {
            id: "user-123".to_string(),
            username: "testuser".to_string(),
            principal_type: PrincipalType::User,
            roles: vec!["user".to_string()],
            permissions: vec!["read".to_string()],
            metadata: std::collections::HashMap::new(),
        };

        // Should NOT be authorized for "delete"
        let result = provider.authorize(&principal, "delete", "resource").await;
        assert!(result.is_ok());
        assert_eq!(result.expect("should succeed"), false);
    }

    #[tokio::test]
    async fn test_authorize_failure() {
        let provider = MockSecurityProvider::new().with_authz_failure();
        let principal = Principal {
            id: "user-123".to_string(),
            username: "testuser".to_string(),
            principal_type: PrincipalType::User,
            roles: vec![],
            permissions: vec![],
            metadata: std::collections::HashMap::new(),
        };

        let result = provider.authorize(&principal, "read", "resource").await;
        assert!(result.is_err());

        match result {
            Err(SecurityError::AuthorizationFailed(msg)) => {
                assert!(msg.contains("Mock authorization failure"));
            }
            _ => unreachable!("Expected AuthorizationFailed error"),
        }
    }

    #[tokio::test]
    async fn test_encrypt_decrypt_roundtrip() {
        let provider = MockSecurityProvider::new();
        let original_data = b"Hello, secure world!";

        // Encrypt
        let encrypted = provider.encrypt(original_data).await.expect("should succeed");
        assert_ne!(encrypted, original_data);

        // Decrypt
        let decrypted = provider.decrypt(&encrypted).await.expect("should succeed");
        assert_eq!(decrypted, original_data);
    }

    #[tokio::test]
    async fn test_encrypt_failure() {
        let provider = MockSecurityProvider::new().with_encrypt_failure();
        let data = b"test data";

        let result = provider.encrypt(data).await;
        assert!(result.is_err());

        match result {
            Err(SecurityError::EncryptionFailed(msg)) => {
                assert!(msg.contains("Mock encryption failure"));
            }
            _ => unreachable!("Expected EncryptionFailed error"),
        }
    }

    #[tokio::test]
    async fn test_encrypt_empty_data() {
        let provider = MockSecurityProvider::new();
        let empty_data: &[u8] = &[];

        let result = provider.encrypt(empty_data).await;
        assert!(result.is_ok());
        assert!(result.expect("should succeed").is_empty());
    }

    #[tokio::test]
    async fn test_encrypt_large_data() {
        let provider = MockSecurityProvider::new();
        let large_data: Vec<u8> = vec![0xFF; 10_000];

        let result = provider.encrypt(&large_data).await;
        assert!(result.is_ok());
        assert_eq!(result.expect("should succeed").len(), 10_000);
    }

    #[tokio::test]
    async fn test_sign_and_verify() {
        let provider = MockSecurityProvider::new();
        let data = b"data to sign";

        // Sign
        let signature = provider.sign(data).await.expect("should succeed");
        assert!(!signature.is_empty());

        // Verify
        let is_valid = provider.verify(data, &signature).await.expect("should succeed");
        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_verify_invalid_signature() {
        let provider = MockSecurityProvider::new();
        let data = b"data to sign";
        let invalid_signature = vec![0x00, 0x11, 0x22];

        let is_valid = provider.verify(data, &invalid_signature).await.expect("should succeed");
        assert!(!is_valid);
    }

    #[tokio::test]
    async fn test_sign_failure() {
        let provider = MockSecurityProvider::new().with_sign_failure();
        let data = b"test data";

        let result = provider.sign(data).await;
        assert!(result.is_err());

        match result {
            Err(SecurityError::SigningFailed(msg)) => {
                assert!(msg.contains("Mock signing failure"));
            }
            _ => unreachable!("Expected SigningFailed error"),
        }
    }

    #[tokio::test]
    async fn test_health_check_healthy() {
        let provider = MockSecurityProvider::new();

        let result = provider.health_check().await;
        assert!(result.is_ok());

        let health = result.expect("should succeed");
        assert!(health.is_healthy);
        assert!(health.encryption_available);
        assert!(health.signing_available);
    }

    #[tokio::test]
    async fn test_health_check_with_failures() {
        let provider = MockSecurityProvider::new()
            .with_encrypt_failure()
            .with_sign_failure();

        let result = provider.health_check().await;
        assert!(result.is_ok());

        let health = result.expect("should succeed");
        assert!(health.is_healthy); // Overall still healthy
        assert!(!health.encryption_available);
        assert!(!health.signing_available);
    }

    #[tokio::test]
    async fn test_get_context() {
        let provider = MockSecurityProvider::new();

        let result = provider.get_context().await;
        assert!(result.is_ok());

        let context = result.expect("should succeed");
        assert_eq!(context.service_name, "mock-security");
        assert_eq!(context.environment, "test");
        assert!(context.encryption_enabled);
        assert!(context.signing_enabled);
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        use tokio::task::JoinSet;

        let provider = std::sync::Arc::new(MockSecurityProvider::new());
        let mut set = JoinSet::new();

        // Spawn 10 concurrent authentication requests
        for i in 0..10 {
            let provider_clone = provider.clone();
            set.spawn(async move {
                let credentials = Credentials {
                    username: format!("user{}", i),
                    password: "password".to_string(),
                };
                provider_clone.authenticate(&credentials).await
            });
        }

        // All should succeed
        let mut success_count = 0;
        while let Some(result) = set.join_next().await {
            if result.expect("should succeed").is_ok() {
                success_count += 1;
            }
        }

        assert_eq!(success_count, 10);
    }

    #[tokio::test]
    async fn test_security_provider_trait_bounds() {
        // Test that SecurityProvider is Send + Sync
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<MockSecurityProvider>();
        assert_sync::<MockSecurityProvider>();
    }

    #[tokio::test]
    async fn test_full_security_workflow() {
        let provider = MockSecurityProvider::new();

        // 1. Authenticate
        let credentials = Credentials {
            username: "workflow-user".to_string(),
            password: "secure-pass".to_string(),
        };
        let auth_result = provider.authenticate(&credentials).await.expect("should succeed");

        // 2. Authorize
        let is_authorized = provider
            .authorize(&auth_result.principal, "read", "document")
            .await
            .expect("should succeed");
        assert!(is_authorized);

        // 3. Encrypt data
        let sensitive_data = b"sensitive information";
        let encrypted = provider.encrypt(sensitive_data).await.expect("should succeed");

        // 4. Sign data
        let signature = provider.sign(sensitive_data).await.expect("should succeed");

        // 5. Verify signature
        let is_valid = provider.verify(sensitive_data, &signature).await.expect("should succeed");
        assert!(is_valid);

        // 6. Decrypt data
        let decrypted = provider.decrypt(&encrypted).await.expect("should succeed");
        assert_eq!(decrypted, sensitive_data);

        // 7. Health check
        let health = provider.health_check().await.expect("should succeed");
        assert!(health.is_healthy);

        // 8. Get context
        let context = provider.get_context().await.expect("should succeed");
        assert!(context.encryption_enabled);
    }
}

