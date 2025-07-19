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
/// ```no_run
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
/// ```no_run
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
