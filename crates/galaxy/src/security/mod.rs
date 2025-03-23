/*!
 * Security module for the Galaxy adapter.
 * 
 * This module handles secure credential management, authentication,
 * and secure storage of sensitive information.
 */

use crate::error::{Error, Result};
use std::sync::Arc;
use thiserror::Error;

// Sub-modules
pub mod credentials;
pub mod storage;

// Re-exports
pub use credentials::{SecureCredentials, SecretString};
pub use storage::{CredentialStorage, MemoryStorage, FileStorage};

/// Security-related errors
#[derive(Error, Debug)]
pub enum SecurityError {
    /// Missing credentials
    #[error("Missing credentials: {0}")]
    MissingCredentials(String),
    
    /// Invalid credentials
    #[error("Invalid credentials: {0}")]
    InvalidCredentials(String),
    
    /// Expired credentials
    #[error("Credentials expired: {0}")]
    ExpiredCredentials(String),
    
    /// Insufficient permissions
    #[error("Insufficient permissions: {0}")]
    InsufficientPermissions(String),
    
    /// Encryption error
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    
    /// Decryption error
    #[error("Decryption error: {0}")]
    DecryptionError(String),
    
    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(String),
    
    /// Environment error
    #[error("Environment error: {0}")]
    EnvironmentError(String),
}

impl From<SecurityError> for Error {
    fn from(err: SecurityError) -> Self {
        match err {
            SecurityError::MissingCredentials(msg) => Error::Authentication(format!("Missing credentials: {}", msg)),
            SecurityError::InvalidCredentials(msg) => Error::Authentication(format!("Invalid credentials: {}", msg)),
            SecurityError::ExpiredCredentials(msg) => Error::Authentication(format!("Expired credentials: {}", msg)),
            SecurityError::InsufficientPermissions(msg) => Error::PermissionDenied(msg),
            SecurityError::EncryptionError(msg) => Error::Internal(format!("Encryption error: {}", msg)),
            SecurityError::DecryptionError(msg) => Error::Internal(format!("Decryption error: {}", msg)),
            SecurityError::StorageError(msg) => Error::Internal(format!("Credential storage error: {}", msg)),
            SecurityError::EnvironmentError(msg) => Error::Config(format!("Environment error: {}", msg)),
        }
    }
}

/// Security manager to handle credentials and authentication
#[derive(Debug)]
pub struct SecurityManager {
    /// Storage for credentials
    storage: Arc<dyn CredentialStorage>,
    /// Whether to allow reading from environment variables
    allow_env_vars: bool,
    /// Rotation policy for credentials
    rotation_policy: Option<RotationPolicy>,
}

/// Credential rotation policy
#[derive(Debug, Clone)]
pub struct RotationPolicy {
    /// Frequency of rotation (in days)
    pub frequency_days: u32,
    /// Automatic rotation
    pub auto_rotate: bool,
    /// Number of old credentials to keep
    pub history_size: usize,
}

impl Default for RotationPolicy {
    fn default() -> Self {
        Self {
            frequency_days: 30,
            auto_rotate: false,
            history_size: 3,
        }
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityManager {
    /// Create a new security manager with memory storage
    pub fn new() -> Self {
        Self {
            storage: Arc::new(MemoryStorage::new()),
            allow_env_vars: false,
            rotation_policy: None,
        }
    }
    
    /// Create a new security manager with the provided storage
    pub fn with_storage(storage: Arc<dyn CredentialStorage>) -> Self {
        Self {
            storage,
            allow_env_vars: false,
            rotation_policy: None,
        }
    }
    
    /// Allow reading credentials from environment variables
    pub fn allow_environment_variables(mut self, allow: bool) -> Self {
        self.allow_env_vars = allow;
        self
    }
    
    /// Set rotation policy
    pub fn with_rotation_policy(mut self, policy: RotationPolicy) -> Self {
        self.rotation_policy = Some(policy);
        self
    }
    
    /// Get credentials by ID
    pub async fn get_credentials(&self, id: &str) -> Result<SecureCredentials> {
        self.storage.get(id).await
    }
    
    /// Store credentials
    pub async fn store_credentials(&self, id: &str, credentials: SecureCredentials) -> Result<()> {
        self.storage.store(id, credentials).await
    }
    
    /// Delete credentials
    pub async fn delete_credentials(&self, id: &str) -> Result<()> {
        self.storage.delete(id).await
    }
    
    /// Rotate credentials
    pub async fn rotate_credentials(&self, id: &str, new_credentials: SecureCredentials) -> Result<()> {
        if let Some(policy) = &self.rotation_policy {
            if policy.history_size > 0 {
                // Store old credentials in history
                if let Ok(old_credentials) = self.storage.get(id).await {
                    let history_id = format!("{}.old.{}", id, chrono::Utc::now().timestamp());
                    self.storage.store(&history_id, old_credentials).await?;
                    
                    // Clean up old history entries if needed
                    // Implementation would depend on the storage backend
                }
            }
        }
        
        // Store new credentials
        self.storage.store(id, new_credentials).await
    }
    
    /// Get credentials from environment if allowed
    pub fn from_environment(&self, var_name: &str) -> Result<SecretString> {
        if !self.allow_env_vars {
            return Err(SecurityError::EnvironmentError(
                "Environment variables access not allowed".to_string(),
            ).into());
        }
        
        match std::env::var(var_name) {
            Ok(value) => Ok(SecretString::new(value)),
            Err(_) => Err(SecurityError::EnvironmentError(
                format!("Environment variable {} not found", var_name),
            ).into()),
        }
    }
    
    /// Validate credentials against Galaxy API
    pub async fn validate_credentials(&self, credentials: &SecureCredentials) -> Result<bool> {
        // This would typically make a request to the Galaxy API to validate the credentials
        // For now, we just do basic validation
        if credentials.is_empty() {
            return Err(SecurityError::InvalidCredentials("Empty credentials".to_string()).into());
        }
        
        // Implement actual validation against Galaxy API when needed
        
        Ok(true)
    }
}

/// Helper functions for secure credential handling
pub mod helpers {
    use super::*;
    
    /// Read a secure string from the environment
    pub fn secure_string_from_env(var_name: &str) -> Result<SecretString> {
        match std::env::var(var_name) {
            Ok(value) => Ok(SecretString::new(value)),
            Err(_) => Err(SecurityError::EnvironmentError(
                format!("Environment variable {} not found", var_name),
            ).into()),
        }
    }
    
    /// Create secure credentials from API key
    pub fn credentials_from_api_key(api_key: SecretString) -> SecureCredentials {
        SecureCredentials::with_api_key(api_key)
    }
    
    /// Create secure credentials from email and password
    pub fn credentials_from_email_password(
        email: String,
        password: SecretString,
    ) -> SecureCredentials {
        SecureCredentials::with_email_password(email, password)
    }
} 