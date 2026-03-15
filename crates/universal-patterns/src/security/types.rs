// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Security API types and request/response structures
//!
//! This module defines the data structures used for communication with
//! security providers and external services.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::traits::{Credentials, Principal};

use super::context::SecurityContext;

/// Authentication request structure
///
/// This structure represents a request to authenticate credentials.
///
/// # Examples
///
/// ```ignore
/// use universal_patterns::security::types::AuthRequest;
/// use universal_patterns::traits::Credentials;
/// use chrono::Utc;
///
/// let request = AuthRequest {
///     service_id: "my-service".to_string(),
///     credentials: Credentials::Test { service_id: "test-service".to_string() },
///     timestamp: Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRequest {
    /// Service ID making the request
    pub service_id: String,
    /// Credentials to authenticate
    pub credentials: Credentials,
    /// Request timestamp
    pub timestamp: DateTime<Utc>,
}

impl AuthRequest {
    /// Create a new authentication request
    ///
    /// # Arguments
    ///
    /// * `service_id` - The service ID making the request
    /// * `credentials` - The credentials to authenticate
    ///
    /// # Returns
    ///
    /// Returns a new `AuthRequest` instance.
    pub fn new(service_id: String, credentials: Credentials) -> Self {
        Self {
            service_id,
            credentials,
            timestamp: Utc::now(),
        }
    }
}

/// Authorization request structure
///
/// This structure represents a request to authorize an action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationRequest {
    /// Service ID making the request
    pub service_id: String,
    /// Principal requesting authorization
    pub principal: Principal,
    /// Action to authorize
    pub action: String,
    /// Resource being accessed
    pub resource: String,
    /// Request timestamp
    pub timestamp: DateTime<Utc>,
}

impl AuthorizationRequest {
    /// Create a new authorization request
    ///
    /// # Arguments
    ///
    /// * `service_id` - The service ID making the request
    /// * `principal` - The principal requesting authorization
    /// * `action` - The action to authorize
    /// * `resource` - The resource being accessed
    ///
    /// # Returns
    ///
    /// Returns a new `AuthorizationRequest` instance.
    pub fn new(service_id: String, principal: Principal, action: String, resource: String) -> Self {
        Self {
            service_id,
            principal,
            action,
            resource,
            timestamp: Utc::now(),
        }
    }
}

/// Authorization result structure
///
/// This structure represents the result of an authorization request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationResult {
    /// Whether the action is authorized
    pub authorized: bool,
}

impl AuthorizationResult {
    /// Create a new authorization result
    ///
    /// # Arguments
    ///
    /// * `authorized` - Whether the action is authorized
    ///
    /// # Returns
    ///
    /// Returns a new `AuthorizationResult` instance.
    pub fn new(authorized: bool) -> Self {
        Self { authorized }
    }

    /// Create an authorized result
    ///
    /// # Returns
    ///
    /// Returns an `AuthorizationResult` with authorized set to true.
    pub fn authorized() -> Self {
        Self { authorized: true }
    }

    /// Create an unauthorized result
    ///
    /// # Returns
    ///
    /// Returns an `AuthorizationResult` with authorized set to false.
    pub fn unauthorized() -> Self {
        Self { authorized: false }
    }
}

/// Encryption request structure
///
/// This structure represents a request to encrypt data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionRequest {
    /// Service ID making the request
    pub service_id: String,
    /// Data to encrypt (base64 encoded)
    pub data: String,
}

impl EncryptionRequest {
    /// Create a new encryption request
    ///
    /// # Arguments
    ///
    /// * `service_id` - The service ID making the request
    /// * `data` - The data to encrypt (base64 encoded)
    ///
    /// # Returns
    ///
    /// Returns a new `EncryptionRequest` instance.
    pub fn new(service_id: String, data: String) -> Self {
        Self { service_id, data }
    }
}

/// Encryption result structure
///
/// This structure represents the result of an encryption request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionResult {
    /// Encrypted data (base64 encoded)
    pub encrypted_data: String,
}

impl EncryptionResult {
    /// Create a new encryption result
    ///
    /// # Arguments
    ///
    /// * `encrypted_data` - The encrypted data (base64 encoded)
    ///
    /// # Returns
    ///
    /// Returns a new `EncryptionResult` instance.
    pub fn new(encrypted_data: String) -> Self {
        Self { encrypted_data }
    }
}

/// Decryption request structure
///
/// This structure represents a request to decrypt data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecryptionRequest {
    /// Service ID making the request
    pub service_id: String,
    /// Encrypted data to decrypt (base64 encoded)
    pub encrypted_data: String,
}

impl DecryptionRequest {
    /// Create a new decryption request
    ///
    /// # Arguments
    ///
    /// * `service_id` - The service ID making the request
    /// * `encrypted_data` - The encrypted data to decrypt (base64 encoded)
    ///
    /// # Returns
    ///
    /// Returns a new `DecryptionRequest` instance.
    pub fn new(service_id: String, encrypted_data: String) -> Self {
        Self {
            service_id,
            encrypted_data,
        }
    }
}

/// Decryption result structure
///
/// This structure represents the result of a decryption request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecryptionResult {
    /// Decrypted data (base64 encoded)
    pub decrypted_data: String,
}

impl DecryptionResult {
    /// Create a new decryption result
    ///
    /// # Arguments
    ///
    /// * `decrypted_data` - The decrypted data (base64 encoded)
    ///
    /// # Returns
    ///
    /// Returns a new `DecryptionResult` instance.
    pub fn new(decrypted_data: String) -> Self {
        Self { decrypted_data }
    }
}

/// Signing request structure
///
/// This structure represents a request to sign data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningRequest {
    /// Service ID making the request
    pub service_id: String,
    /// Data to sign (base64 encoded)
    pub data: String,
}

impl SigningRequest {
    /// Create a new signing request
    ///
    /// # Arguments
    ///
    /// * `service_id` - The service ID making the request
    /// * `data` - The data to sign (base64 encoded)
    ///
    /// # Returns
    ///
    /// Returns a new `SigningRequest` instance.
    pub fn new(service_id: String, data: String) -> Self {
        Self { service_id, data }
    }
}

/// Signing result structure
///
/// This structure represents the result of a signing request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningResult {
    /// Signature (base64 encoded)
    pub signature: String,
}

impl SigningResult {
    /// Create a new signing result
    ///
    /// # Arguments
    ///
    /// * `signature` - The signature (base64 encoded)
    ///
    /// # Returns
    ///
    /// Returns a new `SigningResult` instance.
    pub fn new(signature: String) -> Self {
        Self { signature }
    }
}

/// Verification request structure
///
/// This structure represents a request to verify a signature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRequest {
    /// Service ID making the request
    pub service_id: String,
    /// Original data (base64 encoded)
    pub data: String,
    /// Signature to verify (base64 encoded)
    pub signature: String,
}

impl VerificationRequest {
    /// Create a new verification request
    ///
    /// # Arguments
    ///
    /// * `service_id` - The service ID making the request
    /// * `data` - The original data (base64 encoded)
    /// * `signature` - The signature to verify (base64 encoded)
    ///
    /// # Returns
    ///
    /// Returns a new `VerificationRequest` instance.
    pub fn new(service_id: String, data: String, signature: String) -> Self {
        Self {
            service_id,
            data,
            signature,
        }
    }
}

/// Verification result structure
///
/// This structure represents the result of a verification request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Whether the signature is valid
    pub valid: bool,
}

impl VerificationResult {
    /// Create a new verification result
    ///
    /// # Arguments
    ///
    /// * `valid` - Whether the signature is valid
    ///
    /// # Returns
    ///
    /// Returns a new `VerificationResult` instance.
    pub fn new(valid: bool) -> Self {
        Self { valid }
    }

    /// Create a valid result
    ///
    /// # Returns
    ///
    /// Returns a `VerificationResult` with valid set to true.
    pub fn valid() -> Self {
        Self { valid: true }
    }

    /// Create an invalid result
    ///
    /// # Returns
    ///
    /// Returns a `VerificationResult` with valid set to false.
    pub fn invalid() -> Self {
        Self { valid: false }
    }
}

/// Audit log request structure
///
/// This structure represents a request to log an audit event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogRequest {
    /// Service ID making the request
    pub service_id: String,
    /// Operation being logged
    pub operation: String,
    /// Security context for the operation
    pub context: SecurityContext,
    /// Request timestamp
    pub timestamp: DateTime<Utc>,
}

impl AuditLogRequest {
    /// Create a new audit log request
    ///
    /// # Arguments
    ///
    /// * `service_id` - The service ID making the request
    /// * `operation` - The operation being logged
    /// * `context` - The security context for the operation
    ///
    /// # Returns
    ///
    /// Returns a new `AuditLogRequest` instance.
    pub fn new(service_id: String, operation: String, context: SecurityContext) -> Self {
        Self {
            service_id,
            operation,
            context,
            timestamp: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{Credentials, Principal, PrincipalType};
    use std::collections::HashMap;

    #[test]
    fn test_auth_request_creation() {
        let credentials = Credentials::Test {
            service_id: "test-service".to_string(),
        };

        let request = AuthRequest::new("test-service".to_string(), credentials.clone());
        assert_eq!(request.service_id, "test-service");
        assert!(matches!(request.credentials, Credentials::Test { .. }));
    }

    #[test]
    fn test_authorization_request_creation() {
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

    #[test]
    fn test_authorization_result_creation() {
        let authorized = AuthorizationResult::authorized();
        assert!(authorized.authorized);

        let unauthorized = AuthorizationResult::unauthorized();
        assert!(!unauthorized.authorized);
    }

    #[test]
    fn test_encryption_request_creation() {
        let request = EncryptionRequest::new("test-service".to_string(), "data".to_string());
        assert_eq!(request.service_id, "test-service");
        assert_eq!(request.data, "data");
    }

    #[test]
    fn test_verification_result_creation() {
        let valid = VerificationResult::valid();
        assert!(valid.valid);

        let invalid = VerificationResult::invalid();
        assert!(!invalid.valid);
    }

    #[test]
    fn test_audit_log_request_creation() {
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
}
