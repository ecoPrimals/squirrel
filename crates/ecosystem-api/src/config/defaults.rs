// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Configuration defaults for common scenarios

use crate::traits::{
    FeatureFlags, ResourceConfig, RetryConfig, ServiceConfig, ServiceMeshConfig, UniversalConfig,
};
use crate::types::{SecurityConfig, SecurityLevel};
use std::collections::HashMap;
use universal_constants::network::{BIND_ALL_INTERFACES, get_service_port};

/// Configuration defaults for common scenarios
pub struct ConfigDefaults;

impl ConfigDefaults {
    /// Get default configuration for development
    #[must_use]
    pub fn development() -> UniversalConfig {
        UniversalConfig {
            service: ServiceConfig {
                name: "squirrel-dev".to_string(),
                version: "0.1.0".to_string(),
                description: "Development Squirrel instance".to_string(),
                bind_address: crate::defaults::DefaultEndpoints::dev_bind_address(),
                port: get_service_port("websocket"),
                log_level: "debug".to_string(),
                instance_id: uuid::Uuid::new_v4().to_string(),
            },
            service_mesh: ServiceMeshConfig {
                discovery_endpoint: crate::defaults::DefaultEndpoints::discovery_endpoint(),
                registration_endpoint: crate::defaults::DefaultEndpoints::registration_endpoint(),
                health_endpoint: format!(
                    "{}/api/v1/health",
                    crate::defaults::DefaultEndpoints::service_mesh_endpoint()
                ),
                auth_token: None,
                retry_config: RetryConfig {
                    max_retries: 3,
                    initial_delay_ms: 1000,
                    max_delay_ms: 10000,
                    backoff_multiplier: 2.0,
                },
                heartbeat_interval_secs: 30,
            },
            security: SecurityConfig {
                auth_method: "bearer".to_string(),
                tls_enabled: false,
                mtls_required: false,
                trust_domain: "dev.local".to_string(),
                security_level: SecurityLevel::Internal,
                crypto_lock_enabled: false,
            },
            resources: ResourceConfig {
                cpu_cores: Some(2.0),
                memory_mb: Some(1024),
                disk_mb: Some(10240),
                network_bandwidth_mbps: Some(100),
                gpu_count: None,
            },
            features: FeatureFlags {
                development_mode: true,
                debug_logging: true,
                metrics_enabled: true,
                tracing_enabled: true,
                experimental_features: vec!["dev_mode".to_string()],
            },
            primal_specific: HashMap::new(),
        }
    }

    /// Get default configuration for production
    #[must_use]
    pub fn production() -> UniversalConfig {
        UniversalConfig {
            service: ServiceConfig {
                name: "squirrel".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                description: "Production Squirrel AI primal".to_string(),
                bind_address: BIND_ALL_INTERFACES.to_string(),
                port: 0,
                log_level: "info".to_string(),
                instance_id: uuid::Uuid::new_v4().to_string(),
            },
            service_mesh: ServiceMeshConfig {
                discovery_endpoint: String::new(),
                registration_endpoint: String::new(),
                health_endpoint: String::new(),
                auth_token: None,
                retry_config: RetryConfig {
                    max_retries: 5,
                    initial_delay_ms: 1000,
                    max_delay_ms: 30000,
                    backoff_multiplier: 2.0,
                },
                heartbeat_interval_secs: 30,
            },
            security: SecurityConfig {
                auth_method: "bearer".to_string(),
                tls_enabled: true,
                mtls_required: true,
                trust_domain: "ecosystem.local".to_string(),
                security_level: SecurityLevel::Internal,
                crypto_lock_enabled: true,
            },
            resources: ResourceConfig {
                cpu_cores: None,
                memory_mb: None,
                disk_mb: None,
                network_bandwidth_mbps: None,
                gpu_count: None,
            },
            features: FeatureFlags {
                development_mode: false,
                debug_logging: false,
                metrics_enabled: true,
                tracing_enabled: true,
                experimental_features: vec![],
            },
            primal_specific: HashMap::new(),
        }
    }
}
