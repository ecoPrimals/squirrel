// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Configuration types for the ecosystem registry manager

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Retry configuration for registry operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
    /// Whether to use jitter
    pub jitter: bool,
}

/// Configuration for ecosystem registry manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemRegistryConfig {
    /// Service mesh endpoint for capability-based service discovery
    /// This is a generic endpoint, not tied to any specific primal
    pub service_mesh_endpoint: String,
    /// Registration retry configuration
    pub retry_config: RetryConfig,
    /// Health check configuration
    pub health_config: HealthConfig,
    /// Discovery configuration
    pub discovery_config: DiscoveryConfig,
    /// Security configuration
    pub security_config: RegistrySecurityConfig,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    /// Interval between health checks
    pub check_interval: Duration,
    /// Timeout for each health check
    pub timeout: Duration,
    /// Failures before marking unhealthy
    pub failure_threshold: u32,
    /// Successes before marking healthy
    pub recovery_threshold: u32,
    /// Grace period after startup before health checks
    pub grace_period: Duration,
}

/// Discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Whether service discovery is enabled
    pub enabled: bool,
    /// Interval between discovery scans
    pub discovery_interval: Duration,
    /// Timeout for service discovery requests
    pub service_timeout: Duration,
    /// Whether to auto-register with the registry
    pub auto_register: bool,
    /// Preferred endpoints by service name
    pub preferred_endpoints: HashMap<String, String>,
}

/// Security configuration for ecosystem communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrySecurityConfig {
    /// Whether TLS is enabled
    pub tls_enabled: bool,
    /// Whether mutual TLS is required
    pub mtls_required: bool,
    /// Optional authentication token
    pub auth_token: Option<String>,
    /// Trust domain for certificate validation
    pub trust_domain: String,
    /// Path to TLS certificate
    pub certificate_path: Option<String>,
    /// Path to private key
    pub key_path: Option<String>,
}

// Default implementations
impl Default for EcosystemRegistryConfig {
    fn default() -> Self {
        use universal_constants::network::get_service_port;

        let service_mesh_endpoint = std::env::var("ECOSYSTEM_SERVICE_MESH_ENDPOINT")
            .or_else(|_| std::env::var("SERVICE_MESH_ENDPOINT"))
            .unwrap_or_else(|_| {
                let port = std::env::var("SERVICE_MESH_PORT")
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or_else(|| get_service_port("service_mesh"));
                format!("http://localhost:{port}")
            });

        Self {
            service_mesh_endpoint,
            retry_config: RetryConfig::default(),
            health_config: HealthConfig::default(),
            discovery_config: DiscoveryConfig::default(),
            security_config: RegistrySecurityConfig::default(),
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            timeout: Duration::from_secs(10),
            failure_threshold: 3,
            recovery_threshold: 2,
            grace_period: Duration::from_secs(30),
        }
    }
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            discovery_interval: Duration::from_secs(60),
            service_timeout: Duration::from_secs(5),
            auto_register: true,
            preferred_endpoints: HashMap::new(),
        }
    }
}

impl Default for RegistrySecurityConfig {
    fn default() -> Self {
        Self {
            tls_enabled: true,
            mtls_required: false,
            auth_token: None,
            trust_domain: "squirrel".to_string(),
            certificate_path: None,
            key_path: None,
        }
    }
}
