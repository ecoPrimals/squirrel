// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Core configuration types and structures
//!
//! This module defines all the configuration types used throughout the universal
//! patterns framework, organized into submodules for maintainability.

mod endpoints;
mod security;
mod transport;

pub use endpoints::*;
pub use security::*;
pub use transport::*;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use url::Url;
use uuid::Uuid;

/// Core configuration structure for all primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalConfig {
    /// Primal identification
    pub info: PrimalInfo,

    /// Network configuration
    pub network: NetworkConfig,

    /// Security configuration
    pub security: SecurityConfig,

    /// Orchestration configuration
    pub orchestration: OrchestrationConfig,

    /// Logging configuration
    pub logging: LoggingConfig,

    /// Environment-specific settings
    pub environment: EnvironmentConfig,

    /// Custom primal-specific configuration
    pub custom: HashMap<String, serde_json::Value>,
}

/// Primal identification information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    /// Primal name (e.g., "squirrel", "beardog", "songbird")
    pub name: String,

    /// Primal version
    pub version: String,

    /// Unique instance identifier
    pub instance_id: Uuid,

    /// Primal type/category
    pub primal_type: PrimalType,

    /// Human-readable description
    pub description: String,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Types of primals in the ecosystem
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalType {
    /// AI coordination and MCP protocol management
    Coordinator,
    /// Security and authentication management
    Security,
    /// Orchestration and task management
    Orchestration,
    /// Data storage and retrieval
    Storage,
    /// Compute and processing
    Compute,
    /// Custom/Other primal types
    Custom(String),
}

/// Orchestration configuration for Songbird integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationConfig {
    /// Songbird service endpoint
    pub songbird_endpoint: Option<Url>,

    /// Enable orchestration
    pub enabled: bool,

    /// Orchestration mode
    pub mode: OrchestrationMode,

    /// Health check configuration
    pub health_check: HealthCheckConfig,

    /// Service discovery configuration
    pub service_discovery: ServiceDiscoveryConfig,
}

/// Orchestration modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrchestrationMode {
    /// Standalone mode (no orchestration)
    Standalone,
    /// Managed mode (orchestrated by Songbird)
    Managed,
    /// Hybrid mode (partial orchestration)
    Hybrid,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Enable health checks
    pub enabled: bool,

    /// Health check interval (seconds)
    pub interval: u64,

    /// Health check timeout (seconds)
    pub timeout: u64,

    /// Health check endpoint path
    pub endpoint: String,
}

/// Service discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscoveryConfig {
    /// Enable service discovery
    pub enabled: bool,

    /// Service discovery method
    pub method: ServiceDiscoveryMethod,

    /// Service registration TTL (seconds)
    pub ttl: u64,
}

/// Service discovery methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceDiscoveryMethod {
    /// DNS-based discovery
    Dns {
        /// Domain name for DNS-based discovery
        domain: String,
    },
    /// File-based discovery
    File {
        /// Path to the service discovery file
        path: PathBuf,
    },
    /// Songbird-managed discovery
    Songbird,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: LogLevel,

    /// Log format
    pub format: LogFormat,

    /// Log output destinations
    pub outputs: Vec<LogOutput>,

    /// Enable structured logging
    pub structured: bool,

    /// Enable tracing
    pub tracing: bool,
}

/// Log levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    /// Trace level logging
    Trace,
    /// Debug level logging
    Debug,
    /// Info level logging
    Info,
    /// Warning level logging
    Warn,
    /// Error level logging
    Error,
}

/// Log formats
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogFormat {
    /// Human-readable format
    Human,
    /// JSON format
    Json,
    /// Compact format
    Compact,
}

/// Log output destinations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogOutput {
    /// Standard output
    Stdout,
    /// Standard error
    Stderr,
    /// File output
    File {
        /// Path to the log file
        path: PathBuf,
    },
    /// Syslog output
    Syslog,
}

/// Environment-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Environment name (development, staging, production)
    pub name: String,

    /// Environment-specific variables
    pub variables: HashMap<String, String>,

    /// Feature flags
    pub features: HashMap<String, bool>,

    /// Resource limits
    pub resources: ResourceLimits,
}

/// Resource limits for the primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage (MB)
    pub max_memory_mb: Option<u64>,

    /// Maximum CPU usage (percentage)
    pub max_cpu_percent: Option<f64>,

    /// Maximum disk usage (MB)
    pub max_disk_mb: Option<u64>,

    /// Maximum file descriptors
    pub max_file_descriptors: Option<u64>,
}

/// Universal configuration for managing multiple primal instances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalPrimalConfig {
    /// Whether to enable auto-discovery of primals
    pub auto_discovery_enabled: bool,

    /// Individual primal instance configurations
    pub primal_instances: HashMap<String, PrimalInstanceConfig>,

    /// Multi-instance management settings
    pub multi_instance: MultiInstanceConfig,

    /// Instance lifecycle management
    pub lifecycle: InstanceLifecycleConfig,

    /// Port management configuration
    pub port_management: PortManagementConfig,

    /// Global timeout settings
    pub timeouts: TimeoutConfig,

    /// Logging and monitoring configuration
    pub monitoring: MonitoringConfig,

    /// Legacy configuration for backward compatibility
    pub squirrel: Option<PrimalConfig>,
    /// Legacy Beardog configuration
    pub beardog: Option<PrimalConfig>,
    /// Legacy Nestgate configuration
    pub nestgate: Option<PrimalConfig>,
    /// Legacy Toadstool configuration
    pub toadstool: Option<PrimalConfig>,
}

/// Multi-instance management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiInstanceConfig {
    /// Maximum number of instances per primal type
    pub max_instances_per_type: usize,

    /// Maximum number of instances per user
    pub max_instances_per_user: usize,

    /// Load balancing strategy
    pub load_balancing_strategy: LoadBalancingStrategy,

    /// Instance failover configuration
    pub failover: FailoverConfig,

    /// Instance scaling configuration
    pub scaling: ScalingConfig,
}

/// Instance lifecycle management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceLifecycleConfig {
    /// Whether to automatically start instances
    pub auto_start: bool,

    /// Whether to automatically stop unused instances
    pub auto_stop: bool,

    /// Time before stopping unused instances
    pub idle_timeout_minutes: u64,

    /// Health check configuration
    pub health_monitoring: HealthMonitoringConfig,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Whether to enable metrics collection
    pub metrics_enabled: bool,

    /// Metrics endpoint
    pub metrics_endpoint: String,

    /// Metrics port
    pub metrics_port: u16,

    /// Tracing configuration
    pub tracing: TracingConfig,
}

/// Tracing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// Whether to enable tracing
    pub enabled: bool,

    /// Tracing level
    pub level: String,

    /// Tracing format
    pub format: String,

    /// Whether to include file and line information
    pub include_location: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_primal_instance_config_builder() {
        let config = PrimalInstanceConfig::new(
            "http://localhost:8080".to_string(),
            "instance-1".to_string(),
            "user-123".to_string(),
            "device-456".to_string(),
        )
        .with_api_key("test-key".to_string())
        .with_security_level("high".to_string())
        .with_header("X-Custom".to_string(), "value".to_string());

        assert_eq!(config.base_url, "http://localhost:8080");
        assert_eq!(config.instance_id, "instance-1");
        assert_eq!(config.user_id, "user-123");
        assert_eq!(config.device_id, "device-456");
        assert_eq!(config.api_key, Some("test-key".to_string()));
        assert_eq!(config.security_level, "high");
        assert_eq!(config.headers.get("X-Custom"), Some(&"value".to_string()));
    }

    #[test]
    fn test_port_range() {
        let range = PortRange::new(8000, 8010);

        assert!(range.contains(8005));
        assert!(!range.contains(7999));
        assert!(!range.contains(8011));
        assert_eq!(range.size(), 11);

        let ports: Vec<u16> = range.ports().collect();
        assert_eq!(ports.len(), 11);
        assert_eq!(ports[0], 8000);
        assert_eq!(ports[10], 8010);
    }

    #[test]
    fn test_primal_type_serialization() {
        let coordinator = PrimalType::Coordinator;
        let custom = PrimalType::Custom("my-primal".to_string());

        let coordinator_json = serde_json::to_string(&coordinator).unwrap();
        let custom_json = serde_json::to_string(&custom).unwrap();

        assert_eq!(coordinator_json, "\"Coordinator\"");
        assert!(custom_json.contains("my-primal"));
    }

    #[test]
    fn test_auth_method_variants() {
        let _none = AuthMethod::None;
        let token = AuthMethod::Token {
            token_file: PathBuf::from("/path/to/token"),
        };
        let cert = AuthMethod::Certificate {
            cert_file: PathBuf::from("/cert"),
            key_file: PathBuf::from("/key"),
        };
        let beardog = AuthMethod::Beardog {
            service_id: "service-123".to_string(),
        };

        match token {
            AuthMethod::Token { .. } => {}
            _ => panic!("Expected Token variant"),
        }

        match cert {
            AuthMethod::Certificate { .. } => {}
            _ => panic!("Expected Certificate variant"),
        }

        match beardog {
            AuthMethod::Beardog { .. } => {}
            _ => panic!("Expected Beardog variant"),
        }
    }

    #[test]
    fn test_primal_type_equality() {
        assert_eq!(PrimalType::Coordinator, PrimalType::Coordinator);
        assert_ne!(PrimalType::Coordinator, PrimalType::Security);
        assert_eq!(
            PrimalType::Custom("a".to_string()),
            PrimalType::Custom("a".to_string())
        );
        assert_ne!(
            PrimalType::Custom("a".to_string()),
            PrimalType::Custom("b".to_string())
        );
    }

    #[test]
    fn test_credential_storage_variants() {
        let memory = CredentialStorage::Memory;
        let file = CredentialStorage::File {
            path: PathBuf::from("/tmp/creds"),
        };
        let _beardog = CredentialStorage::Beardog;

        let memory_json = serde_json::to_string(&memory).unwrap();
        let file_json = serde_json::to_string(&file).unwrap();

        let _: CredentialStorage = serde_json::from_str(&memory_json).unwrap();
        let _: CredentialStorage = serde_json::from_str(&file_json).unwrap();
    }

    #[test]
    fn test_encryption_algorithm_serialization() {
        let aes = EncryptionAlgorithm::Aes256Gcm;
        let chacha = EncryptionAlgorithm::ChaCha20Poly1305;

        let aes_json = serde_json::to_string(&aes).unwrap();
        let chacha_json = serde_json::to_string(&chacha).unwrap();

        let _: EncryptionAlgorithm = serde_json::from_str(&aes_json).unwrap();
        let _: EncryptionAlgorithm = serde_json::from_str(&chacha_json).unwrap();
    }

    #[test]
    fn test_key_management_variants() {
        let file = KeyManagement::File {
            path: PathBuf::from("/tmp/key"),
        };
        let beardog = KeyManagement::Beardog;
        let env = KeyManagement::Environment {
            var_name: "MY_KEY".to_string(),
        };

        for variant in [file, beardog, env] {
            let json = serde_json::to_string(&variant).unwrap();
            let _: KeyManagement = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_orchestration_mode_serialization() {
        for mode in [
            OrchestrationMode::Standalone,
            OrchestrationMode::Managed,
            OrchestrationMode::Hybrid,
        ] {
            let json = serde_json::to_string(&mode).unwrap();
            let _: OrchestrationMode = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_log_level_equality() {
        assert_eq!(LogLevel::Info, LogLevel::Info);
        assert_ne!(LogLevel::Debug, LogLevel::Error);
    }

    #[test]
    fn test_log_format_equality() {
        assert_eq!(LogFormat::Json, LogFormat::Json);
        assert_ne!(LogFormat::Human, LogFormat::Compact);
    }

    #[test]
    fn test_log_output_serialization() {
        for output in [
            LogOutput::Stdout,
            LogOutput::Stderr,
            LogOutput::File {
                path: PathBuf::from("/var/log/squirrel.log"),
            },
            LogOutput::Syslog,
        ] {
            let json = serde_json::to_string(&output).unwrap();
            let _: LogOutput = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_load_balancing_strategy_serialization() {
        for strategy in [
            LoadBalancingStrategy::RoundRobin,
            LoadBalancingStrategy::LeastConnections,
            LoadBalancingStrategy::Random,
            LoadBalancingStrategy::Weighted,
            LoadBalancingStrategy::HealthBased,
        ] {
            let json = serde_json::to_string(&strategy).unwrap();
            let deserialized: LoadBalancingStrategy = serde_json::from_str(&json).unwrap();
            assert_eq!(strategy, deserialized);
        }
    }

    #[test]
    fn test_port_allocation_strategy_serialization() {
        for strategy in [
            PortAllocationStrategy::Sequential,
            PortAllocationStrategy::Random,
            PortAllocationStrategy::LeastRecentlyUsed,
        ] {
            let json = serde_json::to_string(&strategy).unwrap();
            let _: PortAllocationStrategy = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_service_discovery_method_serialization() {
        for method in [
            ServiceDiscoveryMethod::Dns {
                domain: "primals.local".to_string(),
            },
            ServiceDiscoveryMethod::File {
                path: PathBuf::from("/etc/primals/services"),
            },
            ServiceDiscoveryMethod::Songbird,
        ] {
            let json = serde_json::to_string(&method).unwrap();
            let _: ServiceDiscoveryMethod = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_port_range_boundary_cases() {
        let range = PortRange::new(8000, 8000);
        assert!(range.contains(8000));
        assert_eq!(range.size(), 1);
        assert_eq!(range.ports().count(), 1);
    }

    #[test]
    fn test_primal_instance_config_defaults() {
        let config = PrimalInstanceConfig::new(
            "http://localhost:8080".to_string(),
            "inst-1".to_string(),
            "user-1".to_string(),
            "dev-1".to_string(),
        );

        assert_eq!(config.security_level, "standard");
        assert!(config.api_key.is_none());
        assert!(config.headers.is_empty());
        assert_eq!(config.timeout_seconds, 30);
        assert!(config.health_check.enabled);
    }
}
