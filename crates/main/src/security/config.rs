//! Security Configuration
//!
//! This module contains all configuration types and authentication methods
//! for the universal security adapter.

use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::types::*;

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
    /// OAuth2 authentication
    OAuth2 {
        client_id: String,
        client_secret: String,
    },
    /// No authentication (for testing)
    None,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: u32,
    /// Base delay between retries
    pub base_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Exponential backoff factor
    pub backoff_factor: f64,
}

impl Default for SecurityProviderConfig {
    fn default() -> Self {
        Self {
            provider_type: "mock".to_string(),
            endpoint: "http://localhost:8080".to_string(),
            auth_method: AuthMethod::None,
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

/// BearDog security configuration
impl SecurityProviderConfig {
    /// Create configuration for BearDog security provider
    pub fn beardog(endpoint: &str, auth_method: AuthMethod) -> Self {
        Self {
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
