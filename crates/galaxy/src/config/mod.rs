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
}

impl Default for CredentialStorageConfig {
    fn default() -> Self {
        Self {
            storage_type: CredentialStorageType::Memory,
            file_storage_path: None,
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
        
        // No credentials found
        Err(SecurityError::MissingCredentials(
            "No credentials provided in configuration or environment".to_string(),
        ).into())
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.api_url.is_empty() {
            return Err(Error::Config("API URL cannot be empty".to_string()));
        }
        
        // Either we have explicit credentials, or we're allowed to use environment variables
        if self.api_key.is_none() && (self.email.is_none() || self.password.is_none()) && !self.allow_env_credentials {
            return Err(Error::Config(
                "Either API key or email/password credentials must be provided, or allow_env_credentials must be true".to_string(),
            ));
        }
        
        Ok(())
    }
    
    /// Create a configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let api_url = std::env::var("GALAXY_API_URL").unwrap_or_else(|_| DEFAULT_GALAXY_URL.to_string());
        
        let mut config = Self::new(api_url)
            .allow_env_credentials(true);
        
        // Try to load explicit API key or email/password
        if let Ok(api_key) = std::env::var("GALAXY_API_KEY") {
            config = config.with_api_key(api_key);
        } else if let Ok(email) = std::env::var("GALAXY_EMAIL") {
            if let Ok(password) = std::env::var("GALAXY_PASSWORD") {
                config = config.with_credentials(email, password);
            }
        }
        
        // Load other configuration from environment if available
        if let Ok(timeout) = std::env::var("GALAXY_TIMEOUT") {
            if let Ok(timeout) = timeout.parse::<u64>() {
                config = config.with_timeout(timeout);
            }
        }
        
        config.validate()?;
        
        Ok(config)
    }
    
    /// Create a configuration for testing
    pub fn for_testing() -> Self {
        Self::new("http://localhost:8080/api")
            .with_api_key("test-api-key")
            .with_debug(true)
    }
}

/// Load configuration from a TOML file
pub fn load_config(path: &std::path::Path) -> Result<GalaxyConfig> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| Error::Config(format!("Failed to read config file: {}", e)))?;
    
    let config: GalaxyConfig = toml::from_str(&content)
        .map_err(|e| Error::Config(format!("Failed to parse config: {}", e)))?;
    
    config.validate()?;
    
    Ok(config)
}

/// Save configuration to a TOML file
pub fn save_config(config: &GalaxyConfig, path: &std::path::Path) -> Result<()> {
    let content = toml::to_string_pretty(config)
        .map_err(|e| Error::Config(format!("Failed to serialize config: {}", e)))?;
    
    std::fs::write(path, content)
        .map_err(|e| Error::Config(format!("Failed to write config file: {}", e)))?;
    
    Ok(())
} 