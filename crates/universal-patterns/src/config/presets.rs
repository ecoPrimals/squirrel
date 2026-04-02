// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Configuration presets and default implementations
//!
//! This module provides sensible default configurations for various environments
//! and use cases, making it easy to get started with the universal patterns framework.
//!
//! TRUE PRIMAL: Endpoints use capability-based discovery via `universal_constants::deployment`.
//! Set env vars (SERVICE_MESH_HOST, SECURITY_SERVICE_HOST, etc.) for staging/production.

#![allow(clippy::wildcard_imports)] // Parent `config` re-exports; preset tables stay readable

use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;
use universal_constants::deployment::endpoints;
use url::Url;
use uuid::Uuid;

use super::*;

/// Parse URLs from [`endpoints`] builders. They are always valid `http://` URLs; if parsing ever
/// fails (e.g. corrupted env), fall back to a minimal localhost URL so presets stay constructible.
fn parse_deployment_endpoint(url: &str) -> Url {
    #[expect(clippy::expect_used, reason = "static literal is always valid")]
    let fallback = Url::parse("http://127.0.0.1:8444/").expect("static fallback URL");
    Url::parse(url).unwrap_or(fallback)
}

impl Default for PrimalConfig {
    fn default() -> Self {
        Self {
            info: PrimalInfo {
                name: "unnamed-primal".to_string(),
                version: "0.1.0".to_string(),
                instance_id: Uuid::new_v4(),
                primal_type: PrimalType::Custom("generic".to_string()),
                description: "A generic primal instance".to_string(),
                created_at: Utc::now(),
            },
            network: NetworkConfig {
                bind_address: "127.0.0.1".to_string(),
                port: 8080,
                public_address: None,
                tls: None,
                timeouts: TimeoutConfig::default(),
                limits: ConnectionLimits::default(),
            },
            security: SecurityConfig {
                beardog_endpoint: None,
                auth_method: AuthMethod::None,
                credential_storage: CredentialStorage::Memory,
                encryption: EncryptionConfig {
                    enable_inter_primal: false,
                    enable_at_rest: false,
                    algorithm: EncryptionAlgorithm::Aes256Gcm,
                    key_management: KeyManagement::Environment {
                        var_name: "PRIMAL_ENCRYPTION_KEY".to_string(),
                    },
                },
                audit_logging: false,
                fallback: SecurityFallback {
                    enable_local_fallback: true,
                    local_auth_method: AuthMethod::None,
                    fallback_timeout: 30,
                },
            },
            orchestration: OrchestrationConfig {
                songbird_endpoint: None,
                enabled: false,
                mode: OrchestrationMode::Standalone,
                health_check: HealthCheckConfig::default(),
                service_discovery: ServiceDiscoveryConfig {
                    enabled: false,
                    method: ServiceDiscoveryMethod::File {
                        path: PathBuf::from("./services.json"),
                    },
                    ttl: 300,
                },
            },
            logging: LoggingConfig {
                level: LogLevel::Info,
                format: LogFormat::Human,
                outputs: vec![LogOutput::Stdout],
                structured: false,
                tracing: false,
            },
            environment: EnvironmentConfig {
                name: "development".to_string(),
                variables: HashMap::new(),
                features: HashMap::new(),
                resources: ResourceLimits::default(),
            },
            custom: HashMap::new(),
        }
    }
}

impl Default for UniversalPrimalConfig {
    fn default() -> Self {
        Self {
            auto_discovery_enabled: true,
            primal_instances: HashMap::new(),
            multi_instance: MultiInstanceConfig::default(),
            lifecycle: InstanceLifecycleConfig::default(),
            port_management: PortManagementConfig::default(),
            timeouts: TimeoutConfig::default(),
            monitoring: MonitoringConfig::default(),
            squirrel: None,
            beardog: None,
            nestgate: None,
            toadstool: None,
        }
    }
}

impl Default for MultiInstanceConfig {
    fn default() -> Self {
        Self {
            max_instances_per_type: 10,
            max_instances_per_user: 5,
            load_balancing_strategy: LoadBalancingStrategy::RoundRobin,
            failover: FailoverConfig::default(),
            scaling: ScalingConfig::default(),
        }
    }
}

impl Default for InstanceLifecycleConfig {
    fn default() -> Self {
        Self {
            auto_start: true,
            auto_stop: false,
            idle_timeout_minutes: 60,
            health_monitoring: HealthMonitoringConfig::default(),
        }
    }
}

impl Default for PortManagementConfig {
    fn default() -> Self {
        Self {
            port_range: PortRange {
                start: 8000,
                end: 9000,
            },
            lease_duration_minutes: 60,
            allocation_strategy: PortAllocationStrategy::Sequential,
            reserved_ports: vec![8080, 8443, 9090],
        }
    }
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 100,
            min_connections: 5,
            connection_timeout_seconds: 30,
            idle_timeout_seconds: 300,
        }
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: 30,
            timeout: 10,
            endpoint: "/health".to_string(),
        }
    }
}

impl Default for FailoverConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_retries: 3,
            retry_delay_seconds: 5,
            circuit_breaker: CircuitBreakerConfig::default(),
        }
    }
}

impl Default for ScalingConfig {
    fn default() -> Self {
        Self {
            auto_scaling_enabled: false,
            min_instances: 1,
            max_instances: 10,
            scale_up_cpu_threshold: 80.0,
            scale_down_cpu_threshold: 20.0,
            scale_up_memory_threshold: 80.0,
            scale_down_memory_threshold: 20.0,
        }
    }
}

impl Default for HealthMonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_seconds: 30,
            check_timeout_seconds: 5,
            failure_threshold: 3,
            recovery_threshold: 2,
        }
    }
}

// CircuitBreakerConfig Default impl is now in squirrel-mcp-config

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            connect: 10,
            request: 30,
            keep_alive: 300,
            idle: 600,
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            metrics_enabled: true,
            metrics_endpoint: "/metrics".to_string(),
            metrics_port: 9090,
            tracing: TracingConfig::default(),
        }
    }
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            level: "info".to_string(),
            format: "pretty".to_string(),
            include_location: false,
        }
    }
}

impl Default for ConnectionLimits {
    fn default() -> Self {
        Self {
            max_connections: 1000,
            max_requests_per_connection: 100,
            rate_limit: Some(100.0),
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: Some(1024),
            max_cpu_percent: Some(80.0),
            max_disk_mb: Some(10240),
            max_file_descriptors: Some(1024),
        }
    }
}

/// Configuration presets for common environments
pub struct ConfigPresets;

impl ConfigPresets {
    /// Development environment configuration
    pub fn development() -> PrimalConfig {
        let mut config = PrimalConfig::default();
        config.environment.name = "development".to_string();
        config.logging.level = LogLevel::Debug;
        config.logging.tracing = true;
        config.security.audit_logging = false;
        config.orchestration.enabled = false;
        config
    }

    /// Staging environment configuration
    pub fn staging() -> PrimalConfig {
        let mut config = PrimalConfig::default();
        config.environment.name = "staging".to_string();
        config.logging.level = LogLevel::Info;
        config.logging.format = LogFormat::Json;
        config.logging.structured = true;
        config.security.audit_logging = true;
        config.orchestration.enabled = true;
        config.orchestration.mode = OrchestrationMode::Managed;
        config.orchestration.songbird_endpoint =
            Some(parse_deployment_endpoint(&endpoints::service_mesh()));
        config
    }

    /// Production environment configuration
    pub fn production() -> PrimalConfig {
        let mut config = PrimalConfig::default();
        config.environment.name = "production".to_string();
        config.logging.level = LogLevel::Warn;
        config.logging.format = LogFormat::Json;
        config.logging.structured = true;
        config.logging.outputs = vec![
            LogOutput::File {
                path: PathBuf::from("/var/log/primal.log"),
            },
            LogOutput::Syslog,
        ];
        config.security.auth_method = AuthMethod::Beardog {
            service_id: "primal-production".to_string(),
        };
        config.security.beardog_endpoint =
            Some(parse_deployment_endpoint(&endpoints::security_service()));
        config.security.audit_logging = true;
        config.security.encryption.enable_inter_primal = true;
        config.security.encryption.enable_at_rest = true;
        config.orchestration.enabled = true;
        config.orchestration.mode = OrchestrationMode::Managed;
        config.orchestration.songbird_endpoint =
            Some(parse_deployment_endpoint(&endpoints::service_mesh()));
        config.network.tls = Some(TlsConfig {
            cert_file: PathBuf::from("/etc/ssl/certs/primal.crt"),
            key_file: PathBuf::from("/etc/ssl/private/primal.key"),
            ca_file: Some(PathBuf::from("/etc/ssl/certs/ca.crt")),
            require_client_cert: true,
        });
        config
    }

    /// Testing environment configuration
    pub fn testing() -> PrimalConfig {
        let mut config = PrimalConfig::default();
        config.environment.name = "testing".to_string();
        config.logging.level = LogLevel::Error;
        config.logging.outputs = vec![LogOutput::Stderr];
        config.security.audit_logging = false;
        config.orchestration.enabled = false;
        config.network.port = 0; // Random port allocation
        config
    }

    /// High-performance configuration
    pub fn high_performance() -> PrimalConfig {
        let mut config = Self::production();
        config.network.limits.max_connections = 10000;
        config.network.limits.max_requests_per_connection = 1000;
        config.network.limits.rate_limit = Some(1000.0);
        config.environment.resources.max_memory_mb = Some(8192);
        config.environment.resources.max_cpu_percent = Some(95.0);
        config
    }

    /// Security-focused configuration
    pub fn secure() -> PrimalConfig {
        let mut config = Self::production();
        config.security.auth_method = AuthMethod::Certificate {
            cert_file: PathBuf::from("/etc/ssl/certs/client.crt"),
            key_file: PathBuf::from("/etc/ssl/private/client.key"),
        };
        config.security.credential_storage = CredentialStorage::Beardog;
        config.security.encryption.enable_inter_primal = true;
        config.security.encryption.enable_at_rest = true;
        config.security.fallback.enable_local_fallback = false;
        if let Some(tls) = config.network.tls.as_mut() {
            tls.require_client_cert = true;
        }
        config
    }

    /// Minimal configuration for embedded systems
    pub fn minimal() -> PrimalConfig {
        let mut config = PrimalConfig::default();
        config.logging.level = LogLevel::Error;
        config.logging.outputs = vec![LogOutput::Stderr];
        config.network.limits.max_connections = 10;
        config.network.limits.max_requests_per_connection = 10;
        config.environment.resources.max_memory_mb = Some(64);
        config.environment.resources.max_file_descriptors = Some(64);
        config.orchestration.enabled = false;
        config.security.audit_logging = false;
        config
    }

    /// Docker container configuration
    pub fn docker() -> PrimalConfig {
        let mut config = PrimalConfig::default();
        config.network.bind_address = "0.0.0.0".to_string();
        config.logging.outputs = vec![LogOutput::Stdout];
        config.logging.format = LogFormat::Json;
        config.orchestration.enabled = true;
        config.orchestration.songbird_endpoint =
            Some(parse_deployment_endpoint(&endpoints::service_mesh()));
        config.orchestration.service_discovery.enabled = true;
        config.orchestration.service_discovery.method = ServiceDiscoveryMethod::Dns {
            domain: "cluster.local".to_string(),
        };
        config
    }

    /// Kubernetes configuration
    pub fn kubernetes() -> PrimalConfig {
        let mut config = Self::docker();
        config.environment.name = "kubernetes".to_string();
        config.logging.structured = true;
        config.orchestration.health_check.endpoint = "/ready".to_string();
        config.security.credential_storage = CredentialStorage::File {
            path: PathBuf::from("/var/secrets/credentials"),
        };
        config
    }
}

/// Universal configuration presets
pub struct UniversalConfigPresets;

impl UniversalConfigPresets {
    /// Single-user development configuration
    pub fn single_user_dev() -> UniversalPrimalConfig {
        let mut config = UniversalPrimalConfig::default();
        config.multi_instance.max_instances_per_user = 1;
        config.multi_instance.max_instances_per_type = 1;
        config.lifecycle.auto_start = true;
        config.lifecycle.auto_stop = false;
        config.port_management.port_range = PortRange {
            start: 8000,
            end: 8010,
        };
        config
    }

    /// Multi-user production configuration
    pub fn multi_user_prod() -> UniversalPrimalConfig {
        let mut config = UniversalPrimalConfig::default();
        config.multi_instance.max_instances_per_user = 10;
        config.multi_instance.max_instances_per_type = 100;
        config.multi_instance.load_balancing_strategy = LoadBalancingStrategy::HealthBased;
        config.multi_instance.scaling.auto_scaling_enabled = true;
        config.lifecycle.auto_start = true;
        config.lifecycle.auto_stop = true;
        config.lifecycle.idle_timeout_minutes = 30;
        config.port_management.port_range = PortRange {
            start: 8000,
            end: 65535,
        };
        config
    }

    /// High-availability configuration
    pub fn high_availability() -> UniversalPrimalConfig {
        let mut config = Self::multi_user_prod();
        config.multi_instance.failover.enabled = true;
        config.multi_instance.failover.max_retries = 5;
        config.multi_instance.failover.circuit_breaker.enabled = true;
        config.lifecycle.health_monitoring.enabled = true;
        config.lifecycle.health_monitoring.check_interval_seconds = 10;
        config.lifecycle.health_monitoring.failure_threshold = 2;
        config
    }

    /// Testing configuration
    pub fn testing() -> UniversalPrimalConfig {
        let mut config = UniversalPrimalConfig::default();
        config.multi_instance.max_instances_per_user = 2;
        config.multi_instance.max_instances_per_type = 2;
        config.lifecycle.auto_start = false;
        config.lifecycle.auto_stop = true;
        config.lifecycle.idle_timeout_minutes = 5;
        config.port_management.port_range = PortRange {
            start: 9000,
            end: 9100,
        };
        config.monitoring.metrics_enabled = false;
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primal_config_defaults() {
        let config = PrimalConfig::default();
        assert_eq!(config.info.name, "unnamed-primal");
        assert_eq!(config.network.port, 8080);
        assert_eq!(config.environment.name, "development");
    }

    #[test]
    fn test_universal_config_defaults() {
        let config = UniversalPrimalConfig::default();
        assert!(config.auto_discovery_enabled);
        assert_eq!(config.multi_instance.max_instances_per_type, 10);
        assert_eq!(config.port_management.port_range.start, 8000);
    }

    #[test]
    fn test_development_preset() {
        let config = ConfigPresets::development();
        assert_eq!(config.environment.name, "development");
        assert_eq!(config.logging.level, LogLevel::Debug);
        assert!(config.logging.tracing);
    }

    #[test]
    fn test_production_preset() {
        let config = ConfigPresets::production();
        assert_eq!(config.environment.name, "production");
        assert_eq!(config.logging.level, LogLevel::Warn);
        assert_eq!(config.logging.format, LogFormat::Json);
        assert!(config.security.encryption.enable_inter_primal);
        assert!(config.network.tls.is_some());
    }

    #[test]
    fn test_secure_preset() {
        let config = ConfigPresets::secure();
        assert!(matches!(
            config.security.auth_method,
            AuthMethod::Certificate { .. }
        ));
        assert!(matches!(
            config.security.credential_storage,
            CredentialStorage::Beardog
        ));
        assert!(!config.security.fallback.enable_local_fallback);
    }

    #[test]
    fn test_minimal_preset() {
        let config = ConfigPresets::minimal();
        assert_eq!(config.logging.level, LogLevel::Error);
        assert_eq!(config.network.limits.max_connections, 10);
        assert_eq!(config.environment.resources.max_memory_mb, Some(64));
    }

    #[test]
    fn test_universal_single_user_dev() {
        let config = UniversalConfigPresets::single_user_dev();
        assert_eq!(config.multi_instance.max_instances_per_user, 1);
        assert_eq!(config.port_management.port_range.end, 8010);
    }

    #[test]
    fn test_universal_high_availability() {
        let config = UniversalConfigPresets::high_availability();
        assert!(config.multi_instance.failover.enabled);
        assert!(config.multi_instance.failover.circuit_breaker.enabled);
        assert_eq!(
            config.lifecycle.health_monitoring.check_interval_seconds,
            10
        );
    }
}
