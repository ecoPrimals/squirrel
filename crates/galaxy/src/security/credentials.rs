/*!
 * Secure credential management for the Galaxy adapter.
 * 
 * This module provides secure types for handling sensitive information
 * like API keys and passwords.
 */

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Deref;
use zeroize::{Zeroize, ZeroizeOnDrop};
use super::SecurityError;

/// A string that is zeroed when dropped and never displayed in debug output
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecretString {
    value: String,
}

impl SecretString {
    /// Create a new secret string
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
    
    /// Create a new empty secret string
    pub fn empty() -> Self {
        Self {
            value: String::new(),
        }
    }
    
    /// Create a secret string from an environment variable
    pub fn from_env(var_name: &str) -> Result<Self> {
        match std::env::var(var_name) {
            Ok(value) => Ok(Self::new(value)),
            Err(_) => Err(SecurityError::EnvironmentError(
                format!("Environment variable {} not found", var_name),
            ).into()),
        }
    }
    
    /// Check if the secret string is empty
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
    
    /// Get the length of the secret string
    pub fn len(&self) -> usize {
        self.value.len()
    }
    
    /// Expose the secret value
    /// 
    /// This should be used only when absolutely necessary, such as
    /// when sending the value to an external API.
    pub fn expose(&self) -> &str {
        &self.value
    }
    
    /// Convert to owned String
    /// 
    /// This should be used only when absolutely necessary, such as
    /// when you need to pass ownership of the value.
    pub fn to_owned_string(&self) -> String {
        self.value.clone()
    }
}

impl Deref for SecretString {
    type Target = str;
    
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl PartialEq for SecretString {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for SecretString {}

// Custom serialization that keeps the value secure
impl Serialize for SecretString {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // This is a secure serialization that doesn't expose the actual value
        // In a real implementation, we might want to encrypt this
        serializer.serialize_str("[ENCRYPTED]")
    }
}

// Custom deserialization
impl<'de> Deserialize<'de> for SecretString {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        
        // In a real implementation, this would decrypt the value
        // For now, we just pass it through
        Ok(Self::new(s))
    }
}

/// Secure credentials for Galaxy authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureCredentials {
    /// API key for authentication
    api_key: Option<SecretString>,
    
    /// Email for authentication
    email: Option<String>,
    
    /// Password for authentication
    password: Option<SecretString>,
    
    /// Creation time of these credentials
    #[serde(with = "time::serde::timestamp")]
    created_at: time::OffsetDateTime,
    
    /// Expiration time of these credentials (if any)
    #[serde(with = "time::serde::timestamp::option")]
    expires_at: Option<time::OffsetDateTime>,
}

impl SecureCredentials {
    /// Create new credentials with an API key
    pub fn with_api_key(api_key: SecretString) -> Self {
        Self {
            api_key: Some(api_key),
            email: None,
            password: None,
            created_at: time::OffsetDateTime::now_utc(),
            expires_at: None,
        }
    }
    
    /// Create new credentials with email and password
    pub fn with_email_password(email: String, password: SecretString) -> Self {
        Self {
            api_key: None,
            email: Some(email),
            password: Some(password),
            created_at: time::OffsetDateTime::now_utc(),
            expires_at: None,
        }
    }
    
    /// Create empty credentials
    pub fn empty() -> Self {
        Self {
            api_key: None,
            email: None,
            password: None,
            created_at: time::OffsetDateTime::now_utc(),
            expires_at: None,
        }
    }
    
    /// Set expiration time
    pub fn with_expiration(mut self, expires_at: time::OffsetDateTime) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
    
    /// Check if credentials are empty
    pub fn is_empty(&self) -> bool {
        self.api_key.is_none() && (self.email.is_none() || self.password.is_none())
    }
    
    /// Check if credentials are expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            expires_at < time::OffsetDateTime::now_utc()
        } else {
            false
        }
    }
    
    /// Get API key if present
    pub fn api_key(&self) -> Option<&SecretString> {
        self.api_key.as_ref()
    }
    
    /// Get email if present
    pub fn email(&self) -> Option<&str> {
        self.email.as_deref()
    }
    
    /// Get password if present
    pub fn password(&self) -> Option<&SecretString> {
        self.password.as_ref()
    }
    
    /// Get the creation time
    pub fn created_at(&self) -> time::OffsetDateTime {
        self.created_at
    }
    
    /// Get the expiration time
    pub fn expires_at(&self) -> Option<time::OffsetDateTime> {
        self.expires_at
    }
    
    /// Clone and update the API key
    pub fn with_updated_api_key(&self, api_key: SecretString) -> Self {
        let mut new = self.clone();
        new.api_key = Some(api_key);
        new.created_at = time::OffsetDateTime::now_utc();
        new
    }
    
    /// Clone and update the email/password
    pub fn with_updated_email_password(&self, email: String, password: SecretString) -> Self {
        let mut new = self.clone();
        new.email = Some(email);
        new.password = Some(password);
        new.created_at = time::OffsetDateTime::now_utc();
        new
    }
}

impl ZeroizeOnDrop for SecureCredentials {}

impl Zeroize for SecureCredentials {
    fn zeroize(&mut self) {
        if let Some(api_key) = &mut self.api_key {
            api_key.zeroize();
        }
        if let Some(email) = &mut self.email {
            email.zeroize();
        }
        if let Some(password) = &mut self.password {
            password.zeroize();
        }
    }
}

/// Convert from config to secure credentials
pub fn credentials_from_config(config: &crate::config::GalaxyConfig) -> Result<SecureCredentials> {
    if let Some(api_key) = &config.api_key {
        Ok(SecureCredentials::with_api_key(SecretString::new(api_key)))
    } else if let Some(email) = &config.email {
        if let Some(password) = &config.password {
            Ok(SecureCredentials::with_email_password(
                email.clone(),
                SecretString::new(password),
            ))
        } else {
            Err(SecurityError::MissingCredentials("Password is required when using email authentication".to_string()).into())
        }
    } else {
        Err(SecurityError::MissingCredentials("No credentials provided in configuration".to_string()).into())
    }
} 