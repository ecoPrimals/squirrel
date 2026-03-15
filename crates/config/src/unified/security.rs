// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Modern Security Configuration Module
//!
//! This module provides a modernized, testable security configuration with:
//! - Type-safe AuthMode enum
//! - Validated secret newtypes (JwtSecret, ApiKey)
//! - Builder pattern for easy construction
//! - Security-focused presets
//! - Comprehensive validation
//!
//! # Example
//!
//! ```rust
//! use squirrel_mcp_config::unified::security::{SecurityConfig, AuthMode};
//!
//! // For testing - security disabled
//! let config = SecurityConfig::testing();
//!
//! // Builder pattern
//! let config = SecurityConfig::builder()
//!     .auth_mode(AuthMode::ApiKey)
//!     .add_api_key("secret-key-123456")
//!     .expect("Failed to add API key")
//!     .build();
//!
//! // Production config
//! let config = SecurityConfig::production();
//! ```

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Authentication mode for the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthMode {
    /// No authentication (testing only!)
    None,
    /// API key authentication
    ApiKey,
    /// JWT token authentication
    Jwt,
    /// Mutual TLS authentication
    MTls,
    /// Combined authentication (API key + JWT)
    Combined,
}

impl AuthMode {
    /// Parse from string
    pub fn from_str(s: &str) -> Result<Self, AuthModeError> {
        match s.to_lowercase().as_str() {
            "none" => Ok(AuthMode::None),
            "apikey" | "api_key" => Ok(AuthMode::ApiKey),
            "jwt" => Ok(AuthMode::Jwt),
            "mtls" | "mutual_tls" => Ok(AuthMode::MTls),
            "combined" => Ok(AuthMode::Combined),
            _ => Err(AuthModeError::Invalid(s.to_string())),
        }
    }

    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            AuthMode::None => "none",
            AuthMode::ApiKey => "apikey",
            AuthMode::Jwt => "jwt",
            AuthMode::MTls => "mtls",
            AuthMode::Combined => "combined",
        }
    }

    /// Check if this mode requires API keys
    pub fn requires_api_keys(&self) -> bool {
        matches!(self, AuthMode::ApiKey | AuthMode::Combined)
    }

    /// Check if this mode requires JWT configuration
    pub fn requires_jwt(&self) -> bool {
        matches!(self, AuthMode::Jwt | AuthMode::Combined)
    }

    /// Check if this mode requires TLS certificates
    pub fn requires_mtls(&self) -> bool {
        matches!(self, AuthMode::MTls | AuthMode::Combined)
    }

    /// Check if authentication is disabled (dangerous in production!)
    pub fn is_disabled(&self) -> bool {
        matches!(self, AuthMode::None)
    }
}

impl Default for AuthMode {
    fn default() -> Self {
        AuthMode::ApiKey // Safe default
    }
}

impl std::fmt::Display for AuthMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Auth mode errors
#[derive(Debug, thiserror::Error)]
pub enum AuthModeError {
    #[error("Invalid auth mode: {0}. Valid values are: none, apikey, jwt, mtls, combined")]
    Invalid(String),
}

/// Validated JWT secret
#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct JwtSecret(#[zeroize(drop)] String);

impl JwtSecret {
    /// Create a new JWT secret with validation
    pub fn new(secret: impl Into<String>) -> Result<Self, JwtSecretError> {
        let secret = secret.into();

        if secret.is_empty() {
            return Err(JwtSecretError::Empty);
        }

        if secret.len() < 32 {
            return Err(JwtSecretError::TooShort {
                length: secret.len(),
                min_length: 32,
            });
        }

        Ok(JwtSecret(secret))
    }

    /// Get the secret value (use carefully!)
    pub fn get(&self) -> &str {
        &self.0
    }
}

// Don't print secrets in debug output!
impl std::fmt::Debug for JwtSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JwtSecret([REDACTED])")
    }
}

/// JWT secret errors
#[derive(Debug, thiserror::Error)]
pub enum JwtSecretError {
    #[error("JWT secret cannot be empty")]
    Empty,

    #[error("JWT secret too short: {length} characters (minimum: {min_length})")]
    TooShort { length: usize, min_length: usize },
}

/// Validated API key
#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct ApiKey(#[zeroize(drop)] String);

impl ApiKey {
    /// Create a new API key with validation
    pub fn new(key: impl Into<String>) -> Result<Self, ApiKeyError> {
        let key = key.into();

        if key.is_empty() {
            return Err(ApiKeyError::Empty);
        }

        if key.len() < 16 {
            return Err(ApiKeyError::TooShort {
                length: key.len(),
                min_length: 16,
            });
        }

        Ok(ApiKey(key))
    }

    /// Get the key value (use carefully!)
    pub fn get(&self) -> &str {
        &self.0
    }
}

// Don't print secrets in debug output!
impl std::fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ApiKey([REDACTED])")
    }
}

/// API key errors
#[derive(Debug, thiserror::Error)]
pub enum ApiKeyError {
    #[error("API key cannot be empty")]
    Empty,

    #[error("API key too short: {length} characters (minimum: {min_length})")]
    TooShort { length: usize, min_length: usize },
}

/// Modern security configuration with builder pattern
///
/// This replaces the old monolithic SecurityConfig with a more testable,
/// type-safe design using validated newtypes and enums.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Security features enabled
    #[serde(default = "default_true")]
    enabled: bool,

    /// Authentication mode
    #[serde(default)]
    auth_mode: AuthMode,

    /// Require authentication for all requests
    #[serde(default = "default_true")]
    require_authentication: bool,

    /// JWT secret (if using JWT auth)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    jwt_secret: Option<JwtSecret>,

    /// API keys (if using API key auth)
    #[serde(default)]
    api_keys: Vec<ApiKey>,

    /// Enable HTTPS/TLS
    #[serde(default)]
    enable_tls: bool,

    /// TLS certificate path
    #[serde(default)]
    tls_cert_path: Option<PathBuf>,

    /// TLS key path
    #[serde(default)]
    tls_key_path: Option<PathBuf>,

    /// Enable mutual TLS
    #[serde(default)]
    enable_mtls: bool,

    /// Rate limiting enabled
    #[serde(default = "default_true")]
    rate_limiting: bool,

    /// CORS enabled
    #[serde(default = "default_true")]
    cors_enabled: bool,
}

fn default_true() -> bool {
    true
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auth_mode: AuthMode::ApiKey,
            require_authentication: true,
            jwt_secret: None,
            api_keys: Vec::new(),
            enable_tls: false,
            tls_cert_path: None,
            tls_key_path: None,
            enable_mtls: false,
            rate_limiting: true,
            cors_enabled: true,
        }
    }
}

impl SecurityConfig {
    /// Create a new builder for SecurityConfig
    pub fn builder() -> SecurityConfigBuilder {
        SecurityConfigBuilder::default()
    }

    /// Create a config for testing (security disabled!)
    ///
    /// **WARNING**: Only use in tests! This disables all security features.
    pub fn testing() -> Self {
        Self {
            enabled: false,
            auth_mode: AuthMode::None,
            require_authentication: false,
            jwt_secret: None,
            api_keys: Vec::new(),
            enable_tls: false,
            tls_cert_path: None,
            tls_key_path: None,
            enable_mtls: false,
            rate_limiting: false,
            cors_enabled: true, // Still useful in tests
        }
    }

    /// Create a development config (relaxed security)
    pub fn development() -> Self {
        Self {
            enabled: true,
            auth_mode: AuthMode::ApiKey,
            require_authentication: true,
            jwt_secret: Some(
                JwtSecret::new("dev-secret-key-at-least-32-chars-long")
                    .expect("BUG: hardcoded dev JWT secret is valid (>32 chars)")
            ),
            api_keys: vec![
                ApiKey::new("dev-api-key-123456")
                    .expect("BUG: hardcoded dev API key is valid (>16 chars)")
            ],
            enable_tls: false, // Not required in dev
            tls_cert_path: None,
            tls_key_path: None,
            enable_mtls: false,
            rate_limiting: false, // Easier development without rate limiting
            cors_enabled: true,
        }
    }

    /// Create a production config (maximum security)
    pub fn production() -> Self {
        Self {
            enabled: true,
            auth_mode: AuthMode::Combined, // Strongest auth
            require_authentication: true,
            jwt_secret: None,     // Must be set explicitly!
            api_keys: Vec::new(), // Must be set explicitly!
            enable_tls: true,
            tls_cert_path: Some(PathBuf::from("/etc/squirrel/tls/cert.pem")),
            tls_key_path: Some(PathBuf::from("/etc/squirrel/tls/key.pem")),
            enable_mtls: false, // Can be enabled if needed
            rate_limiting: true,
            cors_enabled: true,
        }
    }

    // Getters with clear ownership
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn auth_mode(&self) -> AuthMode {
        self.auth_mode
    }

    pub fn requires_authentication(&self) -> bool {
        self.require_authentication
    }

    pub fn jwt_secret(&self) -> Option<&JwtSecret> {
        self.jwt_secret.as_ref()
    }

    pub fn api_keys(&self) -> &[ApiKey] {
        &self.api_keys
    }

    pub fn is_tls_enabled(&self) -> bool {
        self.enable_tls
    }

    pub fn tls_cert_path(&self) -> Option<&PathBuf> {
        self.tls_cert_path.as_ref()
    }

    pub fn tls_key_path(&self) -> Option<&PathBuf> {
        self.tls_key_path.as_ref()
    }

    pub fn is_mtls_enabled(&self) -> bool {
        self.enable_mtls
    }

    pub fn is_rate_limiting_enabled(&self) -> bool {
        self.rate_limiting
    }

    pub fn is_cors_enabled(&self) -> bool {
        self.cors_enabled
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), SecurityConfigError> {
        // If security is disabled, warn but allow
        if !self.enabled {
            // Could add warning system here
            return Ok(());
        }

        // Check auth mode requirements
        if self.auth_mode.requires_jwt() && self.jwt_secret.is_none() {
            return Err(SecurityConfigError::JwtSecretRequired);
        }

        if self.auth_mode.requires_api_keys() && self.api_keys.is_empty() {
            return Err(SecurityConfigError::ApiKeysRequired);
        }

        // Check TLS configuration
        if self.enable_tls {
            if self.tls_cert_path.is_none() {
                return Err(SecurityConfigError::TlsCertMissing);
            }
            if self.tls_key_path.is_none() {
                return Err(SecurityConfigError::TlsKeyMissing);
            }
        }

        // mTLS requires TLS
        if self.enable_mtls && !self.enable_tls {
            return Err(SecurityConfigError::MtlsRequiresTls);
        }

        // Check for auth mode = None in production (dangerous!)
        if self.auth_mode.is_disabled() && self.enabled {
            return Err(SecurityConfigError::AuthDisabledButSecurityEnabled);
        }

        Ok(())
    }
}

/// Builder for SecurityConfig
#[derive(Debug, Default)]
pub struct SecurityConfigBuilder {
    enabled: Option<bool>,
    auth_mode: Option<AuthMode>,
    require_authentication: Option<bool>,
    jwt_secret: Option<JwtSecret>,
    api_keys: Vec<ApiKey>,
    enable_tls: Option<bool>,
    tls_cert_path: Option<PathBuf>,
    tls_key_path: Option<PathBuf>,
    enable_mtls: Option<bool>,
    rate_limiting: Option<bool>,
    cors_enabled: Option<bool>,
}

impl SecurityConfigBuilder {
    /// Enable or disable security
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    /// Set the authentication mode
    pub fn auth_mode(mut self, mode: AuthMode) -> Self {
        self.auth_mode = Some(mode);
        self
    }

    /// Require authentication
    pub fn require_authentication(mut self, require: bool) -> Self {
        self.require_authentication = Some(require);
        self
    }

    /// Set the JWT secret
    pub fn jwt_secret(mut self, secret: impl Into<String>) -> Result<Self, JwtSecretError> {
        self.jwt_secret = Some(JwtSecret::new(secret)?);
        Ok(self)
    }

    /// Add an API key
    pub fn add_api_key(mut self, key: impl Into<String>) -> Result<Self, ApiKeyError> {
        self.api_keys.push(ApiKey::new(key)?);
        Ok(self)
    }

    /// Set API keys (replaces existing)
    pub fn api_keys(mut self, keys: Vec<ApiKey>) -> Self {
        self.api_keys = keys;
        self
    }

    /// Enable or disable TLS
    pub fn enable_tls(mut self, enable: bool) -> Self {
        self.enable_tls = Some(enable);
        self
    }

    /// Set the TLS certificate path
    pub fn tls_cert_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.tls_cert_path = Some(path.into());
        self
    }

    /// Set the TLS key path
    pub fn tls_key_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.tls_key_path = Some(path.into());
        self
    }

    /// Enable or disable mutual TLS
    pub fn enable_mtls(mut self, enable: bool) -> Self {
        self.enable_mtls = Some(enable);
        self
    }

    /// Enable or disable rate limiting
    pub fn rate_limiting(mut self, enable: bool) -> Self {
        self.rate_limiting = Some(enable);
        self
    }

    /// Enable or disable CORS
    pub fn cors_enabled(mut self, enable: bool) -> Self {
        self.cors_enabled = Some(enable);
        self
    }

    /// Build the SecurityConfig
    pub fn build(self) -> SecurityConfig {
        SecurityConfig {
            enabled: self.enabled.unwrap_or(true),
            auth_mode: self.auth_mode.unwrap_or_default(),
            require_authentication: self.require_authentication.unwrap_or(true),
            jwt_secret: self.jwt_secret,
            api_keys: self.api_keys,
            enable_tls: self.enable_tls.unwrap_or(false),
            tls_cert_path: self.tls_cert_path,
            tls_key_path: self.tls_key_path,
            enable_mtls: self.enable_mtls.unwrap_or(false),
            rate_limiting: self.rate_limiting.unwrap_or(true),
            cors_enabled: self.cors_enabled.unwrap_or(true),
        }
    }
}

/// Security configuration errors
#[derive(Debug, thiserror::Error)]
pub enum SecurityConfigError {
    #[error("JWT secret is required when using JWT authentication")]
    JwtSecretRequired,

    #[error("API keys are required when using API key authentication")]
    ApiKeysRequired,

    #[error("TLS is enabled but certificate path is missing")]
    TlsCertMissing,

    #[error("TLS is enabled but key path is missing")]
    TlsKeyMissing,

    #[error("Mutual TLS requires TLS to be enabled")]
    MtlsRequiresTls,

    #[error("Authentication is disabled but security is enabled (inconsistent configuration)")]
    AuthDisabledButSecurityEnabled,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== AuthMode Tests ==========

    #[test]
    fn test_auth_mode_from_str() {
        assert_eq!(AuthMode::from_str("none").expect("test: should succeed"), AuthMode::None);
        assert_eq!(AuthMode::from_str("apikey").expect("test: should succeed"), AuthMode::ApiKey);
        assert_eq!(AuthMode::from_str("api_key").expect("test: should succeed"), AuthMode::ApiKey);
        assert_eq!(AuthMode::from_str("jwt").expect("test: should succeed"), AuthMode::Jwt);
        assert_eq!(AuthMode::from_str("mtls").expect("test: should succeed"), AuthMode::MTls);
        assert_eq!(AuthMode::from_str("combined").expect("test: should succeed"), AuthMode::Combined);

        assert!(AuthMode::from_str("invalid").is_err());
    }

    #[test]
    fn test_auth_mode_requirements() {
        assert!(AuthMode::ApiKey.requires_api_keys());
        assert!(AuthMode::Combined.requires_api_keys());
        assert!(!AuthMode::Jwt.requires_api_keys());

        assert!(AuthMode::Jwt.requires_jwt());
        assert!(AuthMode::Combined.requires_jwt());
        assert!(!AuthMode::ApiKey.requires_jwt());

        assert!(AuthMode::MTls.requires_mtls());
        assert!(!AuthMode::ApiKey.requires_mtls());
    }

    #[test]
    fn test_auth_mode_default() {
        assert_eq!(AuthMode::default(), AuthMode::ApiKey);
    }

    // ========== JwtSecret Tests ==========

    #[test]
    fn test_jwt_secret_validation() {
        // Valid secret
        assert!(JwtSecret::new("a".repeat(32)).is_ok());

        // Empty secret
        assert!(matches!(JwtSecret::new(""), Err(JwtSecretError::Empty)));

        // Too short
        assert!(matches!(
            JwtSecret::new("short"),
            Err(JwtSecretError::TooShort { .. })
        ));
    }

    #[test]
    fn test_jwt_secret_no_debug_leak() {
        let secret = JwtSecret::new("super-secret-key-at-least-32-chars").expect("test: should succeed");
        let debug_output = format!("{:?}", secret);
        assert!(!debug_output.contains("super-secret"));
        assert!(debug_output.contains("[REDACTED]"));
    }

    // ========== ApiKey Tests ==========

    #[test]
    fn test_api_key_validation() {
        // Valid key
        assert!(ApiKey::new("a".repeat(16)).is_ok());

        // Empty key
        assert!(matches!(ApiKey::new(""), Err(ApiKeyError::Empty)));

        // Too short
        assert!(matches!(
            ApiKey::new("short"),
            Err(ApiKeyError::TooShort { .. })
        ));
    }

    #[test]
    fn test_api_key_no_debug_leak() {
        let key = ApiKey::new("secret-key-123456").expect("test: should succeed");
        let debug_output = format!("{:?}", key);
        assert!(!debug_output.contains("secret-key"));
        assert!(debug_output.contains("[REDACTED]"));
    }

    // ========== SecurityConfig Tests ==========

    #[test]
    fn test_default_config() {
        let config = SecurityConfig::default();
        assert!(config.is_enabled());
        assert_eq!(config.auth_mode(), AuthMode::ApiKey);
        assert!(config.requires_authentication());
    }

    #[test]
    fn test_testing_config() {
        let config = SecurityConfig::testing();
        assert!(!config.is_enabled());
        assert_eq!(config.auth_mode(), AuthMode::None);
        assert!(!config.requires_authentication());
    }

    #[test]
    fn test_development_config() {
        let config = SecurityConfig::development();
        assert!(config.is_enabled());
        assert_eq!(config.auth_mode(), AuthMode::ApiKey);
        assert!(config.jwt_secret().is_some());
        assert!(!config.api_keys().is_empty());
    }

    #[test]
    fn test_production_config() {
        let config = SecurityConfig::production();
        assert!(config.is_enabled());
        assert_eq!(config.auth_mode(), AuthMode::Combined);
        assert!(config.is_tls_enabled());
        assert!(config.tls_cert_path().is_some());
    }

    #[test]
    fn test_builder() {
        let config = SecurityConfig::builder()
            .auth_mode(AuthMode::Jwt)
            .jwt_secret("super-secret-key-at-least-32-chars")
            .expect("test: should succeed")
            .enable_tls(true)
            .build();

        assert_eq!(config.auth_mode(), AuthMode::Jwt);
        assert!(config.jwt_secret().is_some());
        assert!(config.is_tls_enabled());
    }

    #[test]
    fn test_builder_with_defaults() {
        let config = SecurityConfig::builder()
            .auth_mode(AuthMode::None)
            .enabled(false)
            .build();

        assert!(!config.is_enabled());
        assert_eq!(config.auth_mode(), AuthMode::None);
    }

    #[test]
    fn test_validation_jwt_secret_required() {
        let config = SecurityConfig {
            auth_mode: AuthMode::Jwt,
            jwt_secret: None,
            ..Default::default()
        };

        let result = config.validate();
        assert!(matches!(
            result,
            Err(SecurityConfigError::JwtSecretRequired)
        ));
    }

    #[test]
    fn test_validation_api_keys_required() {
        let config = SecurityConfig {
            auth_mode: AuthMode::ApiKey,
            api_keys: Vec::new(),
            ..Default::default()
        };

        let result = config.validate();
        assert!(matches!(result, Err(SecurityConfigError::ApiKeysRequired)));
    }

    #[test]
    fn test_validation_tls_cert_missing() {
        let mut config = SecurityConfig::testing(); // Start with valid base
        config.enabled = true;
        config.auth_mode = AuthMode::None; // Skip auth validation
        config.enable_tls = true;
        config.tls_cert_path = None;

        let result = config.validate();
        assert!(matches!(result, Err(SecurityConfigError::TlsCertMissing)));
    }

    #[test]
    fn test_validation_mtls_requires_tls() {
        let mut config = SecurityConfig::testing(); // Start with valid base
        config.enabled = true;
        config.auth_mode = AuthMode::None; // Skip auth validation
        config.enable_mtls = true;
        config.enable_tls = false;

        let result = config.validate();
        assert!(matches!(result, Err(SecurityConfigError::MtlsRequiresTls)));
    }

    #[test]
    fn test_validation_auth_disabled_but_security_enabled() {
        let config = SecurityConfig {
            enabled: true,
            auth_mode: AuthMode::None,
            ..Default::default()
        };

        let result = config.validate();
        assert!(matches!(
            result,
            Err(SecurityConfigError::AuthDisabledButSecurityEnabled)
        ));
    }

    #[test]
    fn test_validation_success() {
        let config = SecurityConfig::testing();
        assert!(config.validate().is_ok());

        let config = SecurityConfig::development();
        assert!(config.validate().is_ok());
    }
}
