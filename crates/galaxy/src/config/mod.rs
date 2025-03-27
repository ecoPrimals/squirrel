/*!
 * Configuration management for the Galaxy adapter.
 * 
 * This module defines the configuration options for connecting to and interacting
 * with a Galaxy instance.
 */

use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::error::{Error, Result};
use crate::security::{SecretString, SecureCredentials, SecurityError};

/// Default Galaxy API URL
pub const DEFAULT_GALAXY_URL: &str = "https://usegalaxy.org/api";

/// Default timeout for API requests in seconds
pub const DEFAULT_API_TIMEOUT_SECONDS: u64 = 30;

/// Default buffer size for data uploads/downloads (4MB)
pub const DEFAULT_BUFFER_SIZE: usize = 4 * 1024 * 1024;

/// Default maximum number of retries for API requests
pub const DEFAULT_MAX_RETRIES: u32 = 3;

/// Default credential rotation period in days
pub const DEFAULT_CREDENTIAL_ROTATION_DAYS: u32 = 90;

/// Default credential history size
pub const DEFAULT_CREDENTIAL_HISTORY_SIZE: usize = 3;

/// Galaxy adapter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalaxyConfig {
    /// Galaxy API URL
    pub api_url: String,
    
    /// API key for Galaxy authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    
    /// Email for Galaxy account (alternative to API key)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    
    /// Password for Galaxy account (alternative to API key)
    #[serde(skip_serializing, skip_deserializing)]
    pub password: Option<String>,
    
    /// Request timeout for API calls
    pub timeout: Duration,
    
    /// Maximum number of concurrent requests
    pub max_concurrent_requests: usize,
    
    /// Maximum retries for failed requests
    pub max_retries: u32,
    
    /// Whether to verify SSL certificates
    pub verify_ssl: bool,
    
    /// Buffer size for data transfers
    pub buffer_size: usize,
    
    /// User agent to use for API requests
    pub user_agent: String,
    
    /// Maximum history size to track
    pub max_history_size: usize,
    
    /// Debug mode
    pub debug: bool,
    
    /// Allow reading credentials from environment variables
    pub allow_env_credentials: bool,
    
    /// Environment variable names for credentials
    pub env_credential_names: EnvCredentialNames,
    
    /// Credential storage configuration
    pub credential_storage: CredentialStorageConfig,
    
    /// Allow reading environment variables for configuration
    #[serde(default)]
    pub allow_env_vars: Option<bool>,
    
    /// Path for credential storage (overrides file_storage_path in credential_storage)
    #[serde(default)]
    pub storage_path: Option<String>,
    
    /// Encryption key for secure storage (hex-encoded)
    #[serde(skip_serializing, default)]
    pub encryption_key: Option<String>,
    
    /// Credential rotation period in days
    #[serde(default)]
    pub key_rotation_days: Option<u32>,
    
    /// Whether to automatically rotate keys
    #[serde(default)]
    pub auto_rotate_keys: Option<bool>,
    
    /// Size of credential history to keep
    #[serde(default)]
    pub credential_history_size: Option<usize>,
    
    /// Unique ID for this credential set
    #[serde(default)]
    pub credential_id: Option<String>,
}

/// Environment variable names for credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvCredentialNames {
    /// Environment variable name for API key
    pub api_key: String,
    
    /// Environment variable name for email
    pub email: String,
    
    /// Environment variable name for password
    pub password: String,
}

impl Default for EnvCredentialNames {
    fn default() -> Self {
        Self {
            api_key: "GALAXY_API_KEY".to_string(),
            email: "GALAXY_EMAIL".to_string(),
            password: "GALAXY_PASSWORD".to_string(),
        }
    }
}

/// Credential storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialStorageConfig {
    /// Storage type
    pub storage_type: CredentialStorageType,
    
    /// Base path for file storage
    pub file_storage_path: Option<String>,
    
    /// Whether to encrypt stored credentials
    #[serde(default = "default_true")]
    pub encrypt: bool,
}

/// Helper function to provide a default value of true
fn default_true() -> bool {
    true
}

impl Default for CredentialStorageConfig {
    fn default() -> Self {
        Self {
            storage_type: CredentialStorageType::Memory,
            file_storage_path: None,
            encrypt: true,
        }
    }
}

/// Credential storage type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CredentialStorageType {
    /// In-memory storage (not persistent)
    Memory,
    
    /// File-based storage
    File,
}

impl Default for GalaxyConfig {
    fn default() -> Self {
        Self {
            api_url: DEFAULT_GALAXY_URL.to_string(),
            api_key: None,
            email: None,
            password: None,
            timeout: Duration::from_secs(DEFAULT_API_TIMEOUT_SECONDS),
            max_concurrent_requests: 5,
            max_retries: DEFAULT_MAX_RETRIES,
            verify_ssl: true,
            buffer_size: DEFAULT_BUFFER_SIZE,
            user_agent: format!("galaxy-mcp-adapter/{}", env!("CARGO_PKG_VERSION")),
            max_history_size: 100,
            debug: false,
            allow_env_credentials: false,
            env_credential_names: EnvCredentialNames::default(),
            credential_storage: CredentialStorageConfig::default(),
            allow_env_vars: None,
            storage_path: None,
            encryption_key: None,
            key_rotation_days: Some(DEFAULT_CREDENTIAL_ROTATION_DAYS),
            auto_rotate_keys: Some(false),
            credential_history_size: Some(DEFAULT_CREDENTIAL_HISTORY_SIZE),
            credential_id: None,
        }
    }
}

impl GalaxyConfig {
    /// Create a new configuration with the specified API URL
    pub fn new(api_url: impl Into<String>) -> Self {
        Self {
            api_url: api_url.into(),
            ..Default::default()
        }
    }
    
    /// Set the API key for authentication
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }
    
    /// Set the API key using a SecretString
    pub fn with_secure_api_key(mut self, api_key: SecretString) -> Self {
        self.api_key = Some(api_key.expose().to_string());
        self
    }
    
    /// Set email and password credentials
    pub fn with_credentials(
        mut self,
        email: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        self.email = Some(email.into());
        self.password = Some(password.into());
        self
    }
    
    /// Set email and password with a secure password
    pub fn with_secure_credentials(
        mut self,
        email: impl Into<String>,
        password: SecretString,
    ) -> Self {
        self.email = Some(email.into());
        self.password = Some(password.expose().to_string());
        self
    }
    
    /// Set the request timeout
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout = Duration::from_secs(seconds);
        self
    }
    
    /// Set debug mode
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }
    
    /// Allow reading credentials from environment variables
    pub fn allow_env_credentials(mut self, allow: bool) -> Self {
        self.allow_env_credentials = allow;
        self
    }
    
    /// Set environment variable names for credentials
    pub fn with_env_credential_names(mut self, names: EnvCredentialNames) -> Self {
        self.env_credential_names = names;
        self
    }
    
    /// Set credential storage configuration
    pub fn with_credential_storage(mut self, storage_config: CredentialStorageConfig) -> Self {
        self.credential_storage = storage_config;
        self
    }
    
    /// Set storage path for credentials
    pub fn with_storage_path(mut self, path: impl Into<String>) -> Self {
        self.storage_path = Some(path.into());
        self
    }
    
    /// Set encryption key for secure storage (hex-encoded)
    pub fn with_encryption_key(mut self, key: impl Into<String>) -> Self {
        self.encryption_key = Some(key.into());
        self
    }
    
    /// Set credential rotation period in days
    pub fn with_key_rotation_days(mut self, days: u32) -> Self {
        self.key_rotation_days = Some(days);
        self
    }
    
    /// Enable or disable automatic key rotation
    pub fn with_auto_rotate_keys(mut self, auto_rotate: bool) -> Self {
        self.auto_rotate_keys = Some(auto_rotate);
        self
    }
    
    /// Set credential history size
    pub fn with_credential_history_size(mut self, size: usize) -> Self {
        self.credential_history_size = Some(size);
        self
    }
    
    /// Set credential ID
    pub fn with_credential_id(mut self, id: impl Into<String>) -> Self {
        self.credential_id = Some(id.into());
        self
    }
    
    /// Allow reading environment variables for configuration
    pub fn allow_env_vars(mut self, allow: bool) -> Self {
        self.allow_env_vars = Some(allow);
        self
    }
    
    /// Get secure credentials from this configuration
    pub fn get_secure_credentials(&self) -> Result<SecureCredentials> {
        // Try explicit credentials first
        if let Some(api_key) = &self.api_key {
            return Ok(SecureCredentials::with_api_key(SecretString::new(api_key)));
        }
        
        if let Some(email) = &self.email {
            if let Some(password) = &self.password {
                return Ok(SecureCredentials::with_email_password(
                    email.clone(),
                    SecretString::new(password),
                ));
            }
        }
        
        // Try environment variables if allowed
        if self.allow_env_credentials {
            // Try API key from environment
            if let Ok(api_key) = std::env::var(&self.env_credential_names.api_key) {
                return Ok(SecureCredentials::with_api_key(SecretString::new(api_key)));
            }
            
            // Try email/password from environment
            if let (Ok(email), Ok(password)) = (
                std::env::var(&self.env_credential_names.email),
                std::env::var(&self.env_credential_names.password),
            ) {
                return Ok(SecureCredentials::with_email_password(
                    email,
                    SecretString::new(password),
                ));
            }
        }
        
        Err(SecurityError::MissingCredentials(
            "No Galaxy credentials provided in configuration or environment".to_string(),
        ).into())
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate API URL
        if self.api_url.is_empty() {
            return Err(Error::Config("API URL cannot be empty".into()));
        }
        
        // Validate that we have some form of authentication
        if self.api_key.is_none() && (self.email.is_none() || self.password.is_none()) && !self.allow_env_credentials {
            return Err(Error::Config(
                "No authentication credentials provided (API key or email/password) and environment variables not allowed".into(),
            ));
        }
        
        // Validate storage path if using file storage
        if self.credential_storage.storage_type == CredentialStorageType::File && 
           self.credential_storage.file_storage_path.is_none() && 
           self.storage_path.is_none() {
            return Err(Error::Config(
                "File storage path must be provided when using file storage".into(),
            ));
        }
        
        Ok(())
    }
    
    /// Create a configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let mut config = Self::default();
        
        // Check for environment variables
        if let Ok(url) = std::env::var("GALAXY_API_URL") {
            config.api_url = url;
        }
        
        if let Ok(api_key) = std::env::var("GALAXY_API_KEY") {
            config.api_key = Some(api_key);
        }
        
        if let Ok(email) = std::env::var("GALAXY_EMAIL") {
            config.email = Some(email);
        }
        
        if let Ok(password) = std::env::var("GALAXY_PASSWORD") {
            config.password = Some(password);
        }
        
        if let Ok(timeout) = std::env::var("GALAXY_TIMEOUT")
            .map(|t| t.parse::<u64>().unwrap_or(DEFAULT_API_TIMEOUT_SECONDS)) {
            config.timeout = Duration::from_secs(timeout);
        }
        
        // Advanced security settings
        if let Ok(path) = std::env::var("GALAXY_STORAGE_PATH") {
            config.storage_path = Some(path);
        }
        
        if let Ok(key) = std::env::var("GALAXY_ENCRYPTION_KEY") {
            config.encryption_key = Some(key);
        }
        
        if let Ok(days) = std::env::var("GALAXY_KEY_ROTATION_DAYS")
            .map(|d| d.parse::<u32>().unwrap_or(DEFAULT_CREDENTIAL_ROTATION_DAYS)) {
            config.key_rotation_days = Some(days);
        }
        
        if let Ok(auto_rotate) = std::env::var("GALAXY_AUTO_ROTATE_KEYS")
            .map(|a| a.parse::<bool>().unwrap_or(false)) {
            config.auto_rotate_keys = Some(auto_rotate);
        }
        
        config.allow_env_credentials = true;
        config.allow_env_vars = Some(true);
        
        Ok(config)
    }
    
    /// Create a configuration for testing
    pub fn for_testing() -> Self {
        Self {
            api_url: "http://localhost:8000/api".to_string(),
            api_key: Some("test-api-key".to_string()),
            verify_ssl: false,
            debug: true,
            credential_storage: CredentialStorageConfig {
                storage_type: CredentialStorageType::Memory,
                file_storage_path: None,
                encrypt: false,
            },
            ..Default::default()
        }
    }
}

/// Load configuration from a file
pub fn load_config(path: &std::path::Path) -> Result<GalaxyConfig> {
    let config_str = std::fs::read_to_string(path)
        .map_err(|e| Error::Config(format!("Failed to read config file: {}", e)))?;
    
    let config: GalaxyConfig = toml::from_str(&config_str)
        .map_err(|e| Error::Config(format!("Failed to parse config file: {}", e)))?;
    
    Ok(config)
}

/// Save configuration to a file
pub fn save_config(config: &GalaxyConfig, path: &std::path::Path) -> Result<()> {
    let config_str = toml::to_string_pretty(config)
        .map_err(|e| Error::Config(format!("Failed to serialize config: {}", e)))?;
    
    std::fs::write(path, config_str)
        .map_err(|e| Error::Config(format!("Failed to write config file: {}", e)))?;
    
    Ok(())
}