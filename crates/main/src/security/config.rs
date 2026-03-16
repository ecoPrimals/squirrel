// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Security Configuration
//!
//! This module contains all configuration types and authentication methods
//! for the universal security adapter.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use zeroize::{Zeroize, ZeroizeOnDrop};

use super::types::{SecurityCapability, SecurityLevel};

/// Security provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityProviderConfig {
    /// Provider type (capability-based, e.g., "security.authentication", "security.encryption")
    /// Use capability names instead of hardcoded primal names
    pub provider_type: String,
    /// Provider endpoint (discovered at runtime via capability discovery)
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
///
/// Sensitive fields (token, key, client_secret) are automatically zeroized on drop
/// via the ZeroizeOnDrop derive on the enum.
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub enum AuthMethod {
    /// JWT token authentication
    JWT {
        /// The JWT bearer token.
        token: String,
    },
    /// API key authentication
    ApiKey {
        /// The API key.
        key: String,
    },
    /// Mutual TLS authentication
    MTLS {
        /// Path to the client certificate.
        cert_path: String,
        /// Path to the client private key.
        key_path: String,
    },
    /// `OAuth2` authentication
    OAuth2 {
        /// OAuth2 client ID.
        client_id: String,
        /// OAuth2 client secret.
        client_secret: String,
    },
    /// Unix socket authentication (secure by default, no credentials needed)
    UnixSocket,
    /// No authentication (for testing)
    None,
}

impl Default for SecurityProviderConfig {
    fn default() -> Self {
        // TRUE PRIMAL: Use capability-based discovery, not hardcoded primal names
        // Endpoint will be discovered at runtime via SECURITY_SERVICE_ENDPOINT or capability discovery
        let endpoint = std::env::var("SECURITY_SERVICE_ENDPOINT").unwrap_or_else(|_| {
            // Fallback to standard socket path if available
            let uid = nix::unistd::getuid();
            let dir = crate::primal_names::BIOMEOS_SOCKET_DIR;
            let standard_socket = format!("/run/user/{uid}/{dir}/security.sock");
            if std::path::Path::new(&standard_socket).exists() {
                format!("unix://{standard_socket}")
            } else {
                // Last resort: use localhost with standard port
                let port = std::env::var("SECURITY_AUTHENTICATION_PORT")
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or(8443);
                format!("http://localhost:{port}")
            }
        });

        Self {
            provider_type: "security.authentication".to_string(), // Capability-based, not primal name
            endpoint,
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

/// Security configuration helpers
impl SecurityProviderConfig {
    /// Create configuration for security provider with authentication capability
    ///
    /// **TRUE PRIMAL**: Uses capability-based discovery instead of hardcoded primal names.
    /// For runtime discovery, use `discover_capability("security.authentication")` or
    /// `RuntimeDiscoveryEngine::discover_capability("security.authentication")`.
    ///
    /// **DEPRECATED**: Prefer capability-based discovery at runtime. This constructor
    /// is maintained for backward compatibility only.
    #[deprecated(
        since = "0.1.0",
        note = "Use capability-based discovery: discover_capability(\"security.authentication\")"
    )]
    #[must_use]
    pub fn beardog(endpoint: &str, auth_method: AuthMethod) -> Self {
        Self {
            // TRUE PRIMAL: Use capability name, not primal name
            provider_type: "security.authentication".to_string(),
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

/// Simple security configuration for security service coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityServiceConfig {
    /// Security service endpoint (discovered via capability matching at runtime)
    /// Set via SECURITY_SERVICE_ENDPOINT environment variable or capability discovery
    pub security_service_endpoint: String,
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
                format!("http://localhost:{port}")
            });

        Self {
            security_service_endpoint,
            enabled: true,
            timeout_seconds: 30,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.base_delay, Duration::from_millis(100));
        assert_eq!(config.max_delay, Duration::from_secs(30));
        assert!((config.backoff_factor - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_retry_config_serde() {
        let config = RetryConfig::default();
        let json = serde_json::to_string(&config).expect("serialize");
        let deser: RetryConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.max_retries, 3);
    }

    #[test]
    fn test_security_provider_config_default() {
        temp_env::with_vars_unset(
            ["SECURITY_SERVICE_ENDPOINT", "SECURITY_AUTHENTICATION_PORT"],
            || {
                let config = SecurityProviderConfig::default();
                assert_eq!(config.provider_type, "security.authentication");
                assert!(matches!(config.auth_method, AuthMethod::UnixSocket));
                assert_eq!(config.timeout, Duration::from_secs(30));
                assert_eq!(config.security_level, SecurityLevel::Standard);
                assert!(
                    config
                        .capabilities
                        .contains(&SecurityCapability::Authentication)
                );
                assert!(
                    config
                        .capabilities
                        .contains(&SecurityCapability::AccessControl)
                );
            },
        );
    }

    #[test]
    #[allow(deprecated)] // Tests deprecated path for backward compatibility
    fn test_beardog_constructor() {
        let config =
            SecurityProviderConfig::beardog("http://localhost:8443", AuthMethod::UnixSocket);
        assert_eq!(config.provider_type, "security.authentication");
        assert_eq!(config.endpoint, "http://localhost:8443");
        assert_eq!(config.security_level, SecurityLevel::Critical);
        assert_eq!(config.retry_config.max_retries, 5);
        assert!(config.capabilities.len() >= 3);
    }

    #[test]
    fn test_custom_constructor() {
        let config = SecurityProviderConfig::custom("http://test:1234", AuthMethod::None);
        assert_eq!(config.provider_type, "custom");
        assert_eq!(config.endpoint, "http://test:1234");
        assert_eq!(config.timeout, Duration::from_secs(15));
        assert_eq!(config.security_level, SecurityLevel::Standard);
    }

    #[test]
    fn test_auth_method_serde() {
        let methods = vec![
            AuthMethod::JWT {
                token: "tok".to_string(),
            },
            AuthMethod::ApiKey {
                key: "key123".to_string(),
            },
            AuthMethod::MTLS {
                cert_path: "/cert".to_string(),
                key_path: "/key".to_string(),
            },
            AuthMethod::OAuth2 {
                client_id: "id".to_string(),
                client_secret: "secret".to_string(),
            },
            AuthMethod::UnixSocket,
            AuthMethod::None,
        ];
        for method in methods {
            let json = serde_json::to_string(&method).expect("serialize");
            let _deser: AuthMethod = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_security_service_config_default() {
        temp_env::with_vars_unset(
            ["SECURITY_SERVICE_ENDPOINT", "SECURITY_AUTHENTICATION_PORT"],
            || {
                let config = SecurityServiceConfig::default();
                assert_eq!(config.security_service_endpoint, "http://localhost:8443");
                assert!(config.enabled);
                assert_eq!(config.timeout_seconds, 30);
            },
        );
    }

    #[test]
    fn test_security_service_config_env_override() {
        temp_env::with_var(
            "SECURITY_SERVICE_ENDPOINT",
            Some("https://secure.example.com"),
            || {
                let config = SecurityServiceConfig::default();
                assert_eq!(
                    config.security_service_endpoint,
                    "https://secure.example.com"
                );
            },
        );
    }

    #[test]
    fn test_security_service_config_port_override() {
        temp_env::with_vars(
            [
                ("SECURITY_SERVICE_ENDPOINT", None::<&str>),
                ("SECURITY_AUTHENTICATION_PORT", Some("9999")),
            ],
            || {
                let config = SecurityServiceConfig::default();
                assert_eq!(config.security_service_endpoint, "http://localhost:9999");
            },
        );
    }

    #[test]
    fn test_security_service_config_serde() {
        let config = SecurityServiceConfig {
            security_service_endpoint: "http://test:8443".to_string(),
            enabled: false,
            timeout_seconds: 60,
        };
        let json = serde_json::to_string(&config).expect("serialize");
        let deser: SecurityServiceConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.security_service_endpoint, "http://test:8443");
        assert!(!deser.enabled);
        assert_eq!(deser.timeout_seconds, 60);
    }
}
