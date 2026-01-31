//! Security Configuration
//!
//! This module contains all configuration types and authentication methods
//! for the universal security adapter.

use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::types::{SecurityCapability, SecurityLevel};

/// Security provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityProviderConfig {
    /// Provider type (e.g., "beardog", "custom", "integrated")
    pub provider_type: String,
    /// Provider endpoint
    pub endpoint: String,
    /// Authentication method
    pub auth_method: AuthMethod,
    /// Connection timeout
    pub timeout: Duration,
    /// Retry configuration
    pub retry_config: RetryConfig,
    /// Security level
    pub security_level: SecurityLevel,
    /// Capabilities
    pub capabilities: Vec<SecurityCapability>,
}

/// Universal authentication methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    /// JWT token authentication
    JWT { token: String },
    /// API key authentication
    ApiKey { key: String },
    /// Mutual TLS authentication
    MTLS { cert_path: String, key_path: String },
    /// `OAuth2` authentication
    OAuth2 {
        client_id: String,
        client_secret: String,
    },
    /// Unix socket authentication (secure by default, no credentials needed)
    UnixSocket,
    /// No authentication (for testing)
    None,
}

impl Default for SecurityProviderConfig {
    fn default() -> Self {
        // Default to BearDog provider with capability-based discovery
        Self {
            provider_type: "beardog".to_string(),
            // Unix socket endpoint (discovered at runtime via family_id)
            endpoint: "unix:///tmp/beardog-nat0.sock".to_string(),
            auth_method: AuthMethod::UnixSocket, // Secure by default
            timeout: Duration::from_secs(30),
            retry_config: RetryConfig::default(),
            security_level: SecurityLevel::Standard,
            capabilities: vec![
                SecurityCapability::Authentication,
                SecurityCapability::AccessControl,
            ],
        }
    }
}

/// Retry configuration for security operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial delay between retries
    pub base_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Backoff factor for exponential backoff
    pub backoff_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_factor: 2.0,
        }
    }
}

/// `BearDog` security configuration
impl SecurityProviderConfig {
    /// Create configuration for `BearDog` security provider
    ///
    /// **NOTE**: This is a convenience constructor for BearDog-specific config.
    /// For capability-based discovery, use `UniversalAdapterV2::connect_capability("security.authentication")`.
    #[must_use]
    pub fn beardog(endpoint: &str, auth_method: AuthMethod) -> Self {
        Self {
            // Note: "beardog" here is just a label in config, not a hardcoded dependency
            provider_type: "beardog".to_string(),
            endpoint: endpoint.to_string(),
            auth_method,
            timeout: Duration::from_secs(30),
            retry_config: RetryConfig {
                max_retries: 5,
                base_delay: Duration::from_millis(200),
                max_delay: Duration::from_secs(60),
                backoff_factor: 2.0,
            },
            security_level: SecurityLevel::Critical,
            capabilities: vec![
                SecurityCapability::Authentication,
                SecurityCapability::Encryption,
                SecurityCapability::AccessControl,
                SecurityCapability::ThreatDetection,
                SecurityCapability::ComplianceMonitoring,
            ],
        }
    }

    /// Create configuration for custom security provider
    #[must_use]
    pub fn custom(endpoint: &str, auth_method: AuthMethod) -> Self {
        Self {
            provider_type: "custom".to_string(),
            endpoint: endpoint.to_string(),
            auth_method,
            timeout: Duration::from_secs(15),
            retry_config: RetryConfig::default(),
            security_level: SecurityLevel::Standard,
            capabilities: vec![
                SecurityCapability::Authentication,
                SecurityCapability::AccessControl,
            ],
        }
    }
}

/// Simple security configuration for `BearDog` coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityServiceConfig {
    /// `BearDog` endpoint
    pub security_service_endpoint: String, // Discovered via capability matching
    /// Enable security coordination
    pub enabled: bool,
    /// Timeout for security operations
    pub timeout_seconds: u64,
}

impl Default for SecurityServiceConfig {
    fn default() -> Self {
        // Multi-tier security endpoint resolution
        // 1. SECURITY_SERVICE_ENDPOINT (full endpoint)
        // 2. SECURITY_AUTHENTICATION_PORT (port override)
        // 3. Default: http://localhost:8443
        let security_service_endpoint =
            std::env::var("SECURITY_SERVICE_ENDPOINT").unwrap_or_else(|_| {
                let port = std::env::var("SECURITY_AUTHENTICATION_PORT")
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or(8443); // Default security auth port
                format!("http://localhost:{}", port)
            });

        Self {
            security_service_endpoint,
            enabled: true,
            timeout_seconds: 30,
        }
    }
}
