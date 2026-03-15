// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Security provider traits
//!
//! This module defines the core traits that all security providers must implement.

use async_trait::async_trait;

use crate::traits::{AuthResult, Credentials, Principal};

use super::context::{SecurityContext, SecurityHealth};
use super::errors::SecurityError;

/// Security provider trait
///
/// This trait defines the core security operations that all providers must implement.
/// It includes authentication, authorization, cryptographic operations, and signing.
///
/// # Examples
///
/// ```ignore
/// use async_trait::async_trait;
/// use universal_patterns::security::{SecurityProvider, SecurityError};
/// use universal_patterns::traits::{AuthResult, Credentials, Principal};
///
/// struct MySecurityProvider;
///
/// #[async_trait]
/// impl SecurityProvider for MySecurityProvider {
///     async fn authenticate(&self, credentials: &Credentials) -> Result<AuthResult, SecurityError> {
///         // Implementation here
///         todo!()
///     }
///     
///     async fn authorize(&self, principal: &Principal, action: &str, resource: &str) -> Result<bool, SecurityError> {
///         // Implementation here
///         todo!()
///     }
///     
///     // ... other methods
/// }
/// ```
#[async_trait]
pub trait SecurityProvider: Send + Sync {
    /// Authenticate credentials
    ///
    /// This method takes user credentials and returns an authentication result
    /// containing the principal, token, and permissions if successful.
    ///
    /// # Arguments
    ///
    /// * `credentials` - The credentials to authenticate
    ///
    /// # Returns
    ///
    /// Returns an `AuthResult` on success or a `SecurityError` on failure.
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthResult, SecurityError>;

    /// Authorize an action
    ///
    /// This method checks if a principal is authorized to perform a specific
    /// action on a resource.
    ///
    /// # Arguments
    ///
    /// * `principal` - The principal requesting authorization
    /// * `action` - The action to authorize (e.g., "read", "write", "delete")
    /// * `resource` - The resource being accessed
    ///
    /// # Returns
    ///
    /// Returns `true` if authorized, `false` if not authorized, or an error.
    async fn authorize(
        &self,
        principal: &Principal,
        action: &str,
        resource: &str,
    ) -> Result<bool, SecurityError>;

    /// Encrypt data
    ///
    /// This method encrypts the provided data using the provider's encryption
    /// algorithm and keys.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to encrypt
    ///
    /// # Returns
    ///
    /// Returns the encrypted data or an error.
    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError>;

    /// Decrypt data
    ///
    /// This method decrypts the provided encrypted data using the provider's
    /// decryption algorithm and keys.
    ///
    /// # Arguments
    ///
    /// * `encrypted_data` - The encrypted data to decrypt
    ///
    /// # Returns
    ///
    /// Returns the decrypted data or an error.
    async fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, SecurityError>;

    /// Sign data
    ///
    /// This method creates a digital signature for the provided data.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to sign
    ///
    /// # Returns
    ///
    /// Returns the signature or an error.
    async fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError>;

    /// Verify signature
    ///
    /// This method verifies that a signature is valid for the provided data.
    ///
    /// # Arguments
    ///
    /// * `data` - The original data
    /// * `signature` - The signature to verify
    ///
    /// # Returns
    ///
    /// Returns `true` if the signature is valid, `false` if not, or an error.
    async fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, SecurityError>;
}

/// Universal security provider trait
///
/// This trait extends the basic SecurityProvider with additional functionality
/// required for universal security operations, including audit logging and
/// health checking.
///
/// # Examples
///
/// ```ignore
/// use async_trait::async_trait;
/// use universal_patterns::security::{UniversalSecurityProvider, SecurityError, SecurityContext, SecurityHealth};
/// use universal_patterns::traits::{AuthResult, Credentials, Principal};
///
/// struct MyUniversalProvider;
///
/// #[async_trait]
/// impl UniversalSecurityProvider for MyUniversalProvider {
///     async fn authenticate(&self, credentials: &Credentials) -> Result<AuthResult, SecurityError> {
///         // Implementation here
///         todo!()
///     }
///     
///     async fn audit_log(&self, operation: &str, context: &SecurityContext) -> Result<(), SecurityError> {
///         // Implementation here
///         todo!()
///     }
///     
///     async fn health_check(&self) -> Result<SecurityHealth, SecurityError> {
///         // Implementation here
///         todo!()
///     }
///     
///     // ... other methods
/// }
/// ```
#[async_trait]
pub trait UniversalSecurityProvider: Send + Sync {
    /// Authenticate credentials
    ///
    /// This method takes user credentials and returns an authentication result
    /// containing the principal, token, and permissions if successful.
    ///
    /// # Arguments
    ///
    /// * `credentials` - The credentials to authenticate
    ///
    /// # Returns
    ///
    /// Returns an `AuthResult` on success or a `SecurityError` on failure.
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthResult, SecurityError>;

    /// Authorize an action
    ///
    /// This method checks if a principal is authorized to perform a specific
    /// action on a resource.
    ///
    /// # Arguments
    ///
    /// * `principal` - The principal requesting authorization
    /// * `action` - The action to authorize (e.g., "read", "write", "delete")
    /// * `resource` - The resource being accessed
    ///
    /// # Returns
    ///
    /// Returns `true` if authorized, `false` if not authorized, or an error.
    async fn authorize(
        &self,
        principal: &Principal,
        action: &str,
        resource: &str,
    ) -> Result<bool, SecurityError>;

    /// Encrypt data
    ///
    /// This method encrypts the provided data using the provider's encryption
    /// algorithm and keys.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to encrypt
    ///
    /// # Returns
    ///
    /// Returns the encrypted data or an error.
    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError>;

    /// Decrypt data
    ///
    /// This method decrypts the provided encrypted data using the provider's
    /// decryption algorithm and keys.
    ///
    /// # Arguments
    ///
    /// * `encrypted_data` - The encrypted data to decrypt
    ///
    /// # Returns
    ///
    /// Returns the decrypted data or an error.
    async fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, SecurityError>;

    /// Sign data
    ///
    /// This method creates a digital signature for the provided data.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to sign
    ///
    /// # Returns
    ///
    /// Returns the signature or an error.
    async fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError>;

    /// Verify signature
    ///
    /// This method verifies that a signature is valid for the provided data.
    ///
    /// # Arguments
    ///
    /// * `data` - The original data
    /// * `signature` - The signature to verify
    ///
    /// # Returns
    ///
    /// Returns `true` if the signature is valid, `false` if not, or an error.
    async fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, SecurityError>;

    /// Audit log operation
    ///
    /// This method logs security operations for audit purposes.
    ///
    /// # Arguments
    ///
    /// * `operation` - The operation being logged
    /// * `context` - The security context for the operation
    ///
    /// # Returns
    ///
    /// Returns `()` on success or an error.
    async fn audit_log(
        &self,
        operation: &str,
        context: &SecurityContext,
    ) -> Result<(), SecurityError>;

    /// Health check
    ///
    /// This method performs a health check on the security provider.
    ///
    /// # Returns
    ///
    /// Returns `SecurityHealth` information or an error.
    async fn health_check(&self) -> Result<SecurityHealth, SecurityError>;
}

/// Blanket implementation to convert UniversalSecurityProvider to SecurityProvider
#[async_trait]
impl<T> SecurityProvider for T
where
    T: UniversalSecurityProvider + ?Sized,
{
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthResult, SecurityError> {
        UniversalSecurityProvider::authenticate(self, credentials).await
    }

    async fn authorize(
        &self,
        principal: &Principal,
        action: &str,
        resource: &str,
    ) -> Result<bool, SecurityError> {
        UniversalSecurityProvider::authorize(self, principal, action, resource).await
    }

    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        UniversalSecurityProvider::encrypt(self, data).await
    }

    async fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        UniversalSecurityProvider::decrypt(self, encrypted_data).await
    }

    async fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        UniversalSecurityProvider::sign(self, data).await
    }

    async fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, SecurityError> {
        UniversalSecurityProvider::verify(self, data, signature).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::time::Duration;

    use crate::traits::PrincipalType;

    /// Test implementation of UniversalSecurityProvider
    struct TestSecurityProvider;

    #[async_trait]
    impl UniversalSecurityProvider for TestSecurityProvider {
        async fn authenticate(
            &self,
            credentials: &Credentials,
        ) -> Result<AuthResult, SecurityError> {
            match credentials {
                Credentials::Password { username, .. } if username == "admin" => Ok(AuthResult {
                    principal: Principal {
                        id: "admin-1".to_string(),
                        name: "admin".to_string(),
                        principal_type: PrincipalType::User,
                        roles: vec!["admin".to_string()],
                        permissions: vec!["read".to_string(), "write".to_string()],
                        metadata: HashMap::new(),
                    },
                    token: "test-token".to_string(),
                    expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
                    permissions: vec!["read".to_string(), "write".to_string()],
                    metadata: HashMap::new(),
                }),
                _ => Err(SecurityError::authentication("Invalid credentials")),
            }
        }

        async fn authorize(
            &self,
            principal: &Principal,
            action: &str,
            _resource: &str,
        ) -> Result<bool, SecurityError> {
            Ok(principal.permissions.contains(&action.to_string()))
        }

        async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
            // Simple XOR "encryption" for testing
            Ok(data.iter().map(|b| b ^ 0x42).collect())
        }

        async fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, SecurityError> {
            // XOR is symmetric
            Ok(encrypted_data.iter().map(|b| b ^ 0x42).collect())
        }

        async fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
            // Simple signature: first 4 bytes of data
            Ok(data.iter().take(4).copied().collect())
        }

        async fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, SecurityError> {
            let expected: Vec<u8> = data.iter().take(4).copied().collect();
            Ok(expected == signature)
        }

        async fn audit_log(
            &self,
            _operation: &str,
            _context: &SecurityContext,
        ) -> Result<(), SecurityError> {
            Ok(())
        }

        async fn health_check(&self) -> Result<SecurityHealth, SecurityError> {
            Ok(SecurityHealth {
                status: super::super::context::HealthStatus::Healthy,
                latency: Duration::from_millis(5),
                last_check: chrono::Utc::now(),
                details: HashMap::new(),
            })
        }
    }

    // Helper to call UniversalSecurityProvider methods without ambiguity
    async fn auth(
        provider: &TestSecurityProvider,
        creds: &Credentials,
    ) -> Result<AuthResult, SecurityError> {
        UniversalSecurityProvider::authenticate(provider, creds).await
    }

    async fn authz(
        provider: &TestSecurityProvider,
        principal: &Principal,
        action: &str,
        resource: &str,
    ) -> Result<bool, SecurityError> {
        UniversalSecurityProvider::authorize(provider, principal, action, resource).await
    }

    async fn enc(provider: &TestSecurityProvider, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        UniversalSecurityProvider::encrypt(provider, data).await
    }

    async fn dec(provider: &TestSecurityProvider, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        UniversalSecurityProvider::decrypt(provider, data).await
    }

    async fn sig(provider: &TestSecurityProvider, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        UniversalSecurityProvider::sign(provider, data).await
    }

    async fn ver(
        provider: &TestSecurityProvider,
        data: &[u8],
        signature: &[u8],
    ) -> Result<bool, SecurityError> {
        UniversalSecurityProvider::verify(provider, data, signature).await
    }

    #[tokio::test]
    async fn test_authenticate_success() {
        let provider = TestSecurityProvider;
        let creds = Credentials::Password {
            username: "admin".to_string(),
            password: "pass".to_string(),
        };
        let result = auth(&provider, &creds).await;
        assert!(result.is_ok());
        let auth_result = result.unwrap();
        assert_eq!(auth_result.principal.name, "admin");
        assert_eq!(auth_result.token, "test-token");
        assert!(auth_result.permissions.contains(&"read".to_string()));
    }

    #[tokio::test]
    async fn test_authenticate_failure() {
        let provider = TestSecurityProvider;
        let creds = Credentials::Password {
            username: "unknown".to_string(),
            password: "pass".to_string(),
        };
        let result = auth(&provider, &creds).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_authorize_allowed() {
        let provider = TestSecurityProvider;
        let principal = Principal {
            id: "1".to_string(),
            name: "user".to_string(),
            principal_type: PrincipalType::User,
            roles: vec![],
            permissions: vec!["read".to_string()],
            metadata: HashMap::new(),
        };
        let result = authz(&provider, &principal, "read", "/data").await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_authorize_denied() {
        let provider = TestSecurityProvider;
        let principal = Principal {
            id: "1".to_string(),
            name: "user".to_string(),
            principal_type: PrincipalType::User,
            roles: vec![],
            permissions: vec!["read".to_string()],
            metadata: HashMap::new(),
        };
        let result = authz(&provider, &principal, "delete", "/data").await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_encrypt_decrypt_roundtrip() {
        let provider = TestSecurityProvider;
        let original = b"Hello, World!";

        let encrypted = enc(&provider, original).await.unwrap();
        assert_ne!(encrypted, original);

        let decrypted = dec(&provider, &encrypted).await.unwrap();
        assert_eq!(decrypted, original);
    }

    #[tokio::test]
    async fn test_sign_verify() {
        let provider = TestSecurityProvider;
        let data = b"Sign this data";

        let signature = sig(&provider, data).await.unwrap();
        let is_valid = ver(&provider, data, &signature).await.unwrap();
        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_verify_invalid_signature() {
        let provider = TestSecurityProvider;
        let data = b"Sign this data";

        let is_valid = ver(&provider, data, b"bad").await.unwrap();
        assert!(!is_valid);
    }

    #[tokio::test]
    async fn test_health_check() {
        let provider = TestSecurityProvider;
        let health = UniversalSecurityProvider::health_check(&provider)
            .await
            .unwrap();
        assert!(matches!(
            health.status,
            super::super::context::HealthStatus::Healthy
        ));
    }

    #[tokio::test]
    async fn test_blanket_impl_security_provider() {
        // Test that UniversalSecurityProvider automatically implements SecurityProvider
        let provider = TestSecurityProvider;
        let provider_ref: &dyn SecurityProvider = &provider;

        let data = b"test data";
        let encrypted = provider_ref.encrypt(data).await.unwrap();
        let decrypted = provider_ref.decrypt(&encrypted).await.unwrap();
        assert_eq!(decrypted, data);
    }

    #[tokio::test]
    async fn test_audit_log() {
        let provider = TestSecurityProvider;
        let context = SecurityContext {
            principal: Principal {
                id: "test-1".to_string(),
                name: "tester".to_string(),
                principal_type: PrincipalType::User,
                roles: vec![],
                permissions: vec![],
                metadata: HashMap::new(),
            },
            token: "test-token".to_string(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
            permissions: vec![],
            metadata: HashMap::new(),
        };
        let result = provider.audit_log("test_operation", &context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_authenticate_various_credentials() {
        let provider = TestSecurityProvider;

        // API key
        let creds = Credentials::ApiKey {
            key: "key".to_string(),
            service_id: "svc".to_string(),
        };
        assert!(auth(&provider, &creds).await.is_err());

        // Bearer
        let creds = Credentials::Bearer {
            token: "token".to_string(),
        };
        assert!(auth(&provider, &creds).await.is_err());

        // Token
        let creds = Credentials::Token {
            token: "jwt".to_string(),
        };
        assert!(auth(&provider, &creds).await.is_err());
    }
}
