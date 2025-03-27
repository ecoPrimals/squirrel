/*!
 * Security module for the Galaxy adapter.
 * 
 * This module handles secure credential management, authentication,
 * and secure storage of sensitive information.
 */

use crate::error::{Error, Result};
use std::sync::Arc;
use thiserror::Error;
use time;

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
    /// Whether to automatically check for rotation on get
    auto_check_rotation: bool,
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
    /// Whether to update dependent services when rotating
    pub update_dependents: bool,
}

/// Credential history entry
#[derive(Debug, Clone)]
struct CredentialHistoryEntry {
    /// The credentials
    credentials: SecureCredentials,
    /// When the credentials were created
    created_at: time::OffsetDateTime,
    /// When the credentials were retired
    retired_at: time::OffsetDateTime,
}

impl Default for RotationPolicy {
    fn default() -> Self {
        Self {
            frequency_days: 30,
            auto_rotate: false,
            history_size: 3,
            update_dependents: false,
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
            auto_check_rotation: false,
        }
    }
    
    /// Create a new security manager with the provided storage
    pub fn with_storage(storage: Arc<dyn CredentialStorage>) -> Self {
        Self {
            storage,
            allow_env_vars: false,
            rotation_policy: None,
            auto_check_rotation: false,
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
    
    /// Enable or disable automatic rotation checking
    pub fn auto_check_rotation(mut self, enabled: bool) -> Self {
        self.auto_check_rotation = enabled;
        self
    }
    
    /// Get credentials by ID
    pub async fn get_credentials(&self, id: &str) -> Result<SecureCredentials> {
        let credentials = self.storage.get(id).await?;
        
        // Check if rotation is needed
        if self.auto_check_rotation {
            if let Some(policy) = &self.rotation_policy {
                if policy.auto_rotate && self.should_rotate(&credentials) {
                    tracing::info!("Automatic credential rotation triggered for {}", id);
                    return Err(SecurityError::ExpiredCredentials(
                        format!("Credentials for {} need rotation", id)
                    ).into());
                }
            }
        }
        
        Ok(credentials)
    }
    
    /// Check if credentials should be rotated based on policy
    pub fn should_rotate(&self, credentials: &SecureCredentials) -> bool {
        if let Some(policy) = &self.rotation_policy {
            let now = time::OffsetDateTime::now_utc();
            let created = credentials.created_at();
            let age = now - created;
            
            // Check if older than rotation frequency
            if age.whole_days() >= policy.frequency_days as i64 {
                return true;
            }
            
            // Check if expired
            if credentials.is_expired() {
                return true;
            }
        }
        
        false
    }
    
    /// Store credentials
    pub async fn store_credentials(&self, id: &str, credentials: SecureCredentials) -> Result<()> {
        self.storage.store(id, credentials).await
    }
    
    /// Delete credentials
    pub async fn delete_credentials(&self, id: &str) -> Result<()> {
        self.storage.delete(id).await
    }
    
    /// Format history ID
    fn format_history_id(&self, id: &str, timestamp: time::OffsetDateTime) -> String {
        format!("{}.history.{}", id, timestamp.unix_timestamp())
    }
    
    /// Rotate credentials with history tracking
    pub async fn rotate_credentials(&self, id: &str, new_credentials: SecureCredentials) -> Result<()> {
        // Get old credentials if they exist
        let old_credentials = match self.storage.get(id).await {
            Ok(creds) => Some(creds),
            Err(_) => None,
        };
        
        // Store new credentials first to ensure we don't lose access
        self.storage.store(id, new_credentials).await?;
        
        // Store old credentials in history if requested and they exist
        if let (Some(policy), Some(old_creds)) = (&self.rotation_policy, old_credentials) {
            if policy.history_size > 0 {
                let now = time::OffsetDateTime::now_utc();
                let history_id = self.format_history_id(id, now);
                
                // Store the old credentials in history
                self.storage.store(&history_id, old_creds).await?;
                
                // Clean up old history entries if needed
                self.clean_credential_history(id, policy.history_size).await?;
            }
        }
        
        Ok(())
    }
    
    /// Get credential history
    pub async fn get_credential_history(&self, id: &str) -> Result<Vec<SecureCredentials>> {
        // List all credentials
        let all_ids = self.storage.list().await?;
        
        // Filter for history entries related to this ID
        let prefix = format!("{}.history.", id);
        let history_ids: Vec<_> = all_ids.into_iter()
            .filter(|hist_id| hist_id.starts_with(&prefix))
            .collect();
        
        // Get all history entries
        let mut history = Vec::new();
        for hist_id in history_ids {
            if let Ok(creds) = self.storage.get(&hist_id).await {
                history.push(creds);
            }
        }
        
        // Sort by creation time (newest first)
        history.sort_by(|a: &SecureCredentials, b: &SecureCredentials| b.created_at().cmp(&a.created_at()));
        
        Ok(history)
    }
    
    /// Clean up old history entries
    async fn clean_credential_history(&self, id: &str, keep_count: usize) -> Result<()> {
        // Get all credential history
        let history = self.get_credential_history(id).await?;
        
        // If we have more history entries than we should keep, delete the oldest ones
        if history.len() > keep_count {
            let all_ids = self.storage.list().await?;
            let prefix = format!("{}.history.", id);
            
            // Get all history IDs sorted by timestamp (oldest first)
            let mut history_ids: Vec<_> = all_ids.into_iter()
                .filter(|hist_id| hist_id.starts_with(&prefix))
                .collect();
            
            // Sort by timestamp (oldest first)
            history_ids.sort_by(|a, b| {
                let a_ts = a.split('.').last().unwrap_or("0").parse::<i64>().unwrap_or(0);
                let b_ts = b.split('.').last().unwrap_or("0").parse::<i64>().unwrap_or(0);
                a_ts.cmp(&b_ts)
            });
            
            // Delete oldest entries
            let to_delete = history_ids.len() - keep_count;
            for i in 0..to_delete {
                if i < history_ids.len() {
                    let _ = self.storage.delete(&history_ids[i]).await;
                }
            }
        }
        
        Ok(())
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
    
    /// Find credentials by API key
    pub async fn find_by_api_key(&self, api_key: &SecretString) -> Result<Option<(String, SecureCredentials)>> {
        let ids = self.storage.list().await?;
        
        for id in ids {
            if let Ok(creds) = self.storage.get(&id).await {
                if let Some(key) = creds.api_key() {
                    if key == api_key {
                        return Ok(Some((id, creds)));
                    }
                }
            }
        }
        
        Ok(None)
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