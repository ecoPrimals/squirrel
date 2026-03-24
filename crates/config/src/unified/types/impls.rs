// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! `Default` implementations and `SquirrelUnifiedConfig::validate`.

use std::collections::HashMap;

use crate::unified::timeouts::TimeoutConfig;

use super::defaults::{
    default_bind_address, default_buffer_size, default_circuit_timeout, default_compression_level,
    default_data_dir, default_database_url, default_db_timeout, default_encryption_format,
    default_failure_threshold, default_grpc_port, default_half_open_requests,
    default_health_check_interval, default_heartbeat_interval, default_http_port,
    default_instance_id, default_log_level, default_max_concurrent_ai_requests,
    default_max_connections, default_max_db_connections, default_max_message_size,
    default_max_retries, default_max_services, default_mcp_version, default_metrics_endpoint,
    default_plugin_dir, default_pool_size, default_prometheus_port, default_registry_type,
    default_service_expiration, default_session_timeout, default_success_threshold,
    default_token_expiration, default_token_expiry_minutes, default_websocket_port,
    default_work_dir,
};
use super::definitions::{
    AiProvidersConfig, CircuitBreakerConfig, DatabaseBackend, DatabaseConfig, FeatureFlags,
    LoadBalancingConfig, LoadBalancingStrategy, McpConfig, MonitoringConfig, NetworkConfig,
    SecurityConfig, ServiceMeshConfig, SquirrelUnifiedConfig, SystemConfig,
};

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            require_authentication: true,
            enable_authorization: true,
            jwt_secret: std::env::var("JWT_SECRET").ok(),
            token_expiration_secs: default_token_expiration(),
            api_keys: vec![],
            allowed_origins: vec!["*".to_string()],
            tls_enabled: false,
            tls_cert_path: std::env::var("TLS_CERT_PATH").ok(),
            tls_key_path: std::env::var("TLS_KEY_PATH").ok(),
            ca_cert_path: std::env::var("CA_CERT_PATH").ok(),
            mtls_enabled: false,
            encryption_default_format: default_encryption_format(),
            enable_audit: true,
            enable_encryption: true,
            enable_rbac: true,
            token_expiry_minutes: default_token_expiry_minutes(),
        }
    }
}
impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            failure_threshold: default_failure_threshold(),
            success_threshold: default_success_threshold(),
            timeout_secs: default_circuit_timeout(),
            half_open_max_requests: default_half_open_requests(),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            connection_string: default_database_url(),
            max_connections: default_max_db_connections(),
            timeout_seconds: default_db_timeout(),
            backend: DatabaseBackend::default(),
            enable_pooling: true,
            pool_size: default_pool_size(),
        }
    }
}

impl Default for LoadBalancingConfig {
    fn default() -> Self {
        Self {
            strategy: LoadBalancingStrategy::default(),
            sticky_sessions: false,
            session_timeout_secs: default_session_timeout(),
            circuit_breaker: CircuitBreakerConfig::default(),
            health_based_routing: true,
            retry_failed: true,
            max_retries: default_max_retries(),
        }
    }
}
impl Default for SquirrelUnifiedConfig {
    fn default() -> Self {
        Self {
            system: SystemConfig {
                instance_id: default_instance_id(),
                environment: "development".to_string(),
                log_level: default_log_level(),
                work_dir: default_work_dir(),
                data_dir: default_data_dir(),
                plugin_dir: default_plugin_dir(),
            },
            network: NetworkConfig {
                bind_address: default_bind_address(),
                http_port: default_http_port(),
                websocket_port: default_websocket_port(),
                grpc_port: default_grpc_port(),
                max_connections: default_max_connections(),
                enable_tls: false,
                tls_cert_path: None,
                tls_key_path: None,
            },
            security: SecurityConfig {
                enabled: true,
                require_authentication: true,
                enable_authorization: true,
                jwt_secret: std::env::var("JWT_SECRET").ok(),
                token_expiration_secs: default_token_expiration(),
                api_keys: vec![],
                allowed_origins: vec!["*".to_string()],
                tls_enabled: false,
                tls_cert_path: std::env::var("TLS_CERT_PATH").ok(),
                tls_key_path: std::env::var("TLS_KEY_PATH").ok(),
                ca_cert_path: std::env::var("CA_CERT_PATH").ok(),
                mtls_enabled: false,
                // Consolidated fields (Nov 9, 2025)
                encryption_default_format: default_encryption_format(),
                enable_audit: true,
                enable_encryption: true,
                enable_rbac: true,
                token_expiry_minutes: default_token_expiry_minutes(),
            },
            mcp: McpConfig {
                version: default_mcp_version(),
                max_message_size: default_max_message_size(),
                buffer_size: default_buffer_size(),
                enable_compression: false,
                compression_level: default_compression_level(),
            },
            ai: AiProvidersConfig {
                default_endpoint: String::new(),
                providers: HashMap::new(),
                enable_local: true,
                enable_cloud: true,
                max_concurrent_requests: default_max_concurrent_ai_requests(),
            },
            service_mesh: ServiceMeshConfig {
                enabled: true,
                discovery_endpoints: vec![],
                registry_type: default_registry_type(),
                max_services: default_max_services(),
                health_check_interval_secs: default_health_check_interval(),
                heartbeat_interval_secs: default_heartbeat_interval(),
                service_expiration_secs: default_service_expiration(),
                enable_failover: true,
                metrics_enabled: true,
                namespace: None,
            },
            timeouts: TimeoutConfig::default(),
            monitoring: MonitoringConfig {
                enabled: true,
                metrics_endpoint: default_metrics_endpoint(),
                tracing_endpoint: None,
                enable_prometheus: true,
                prometheus_port: default_prometheus_port(),
            },
            database: DatabaseConfig::default(),
            load_balancing: LoadBalancingConfig::default(),
            features: FeatureFlags {
                experimental: false,
                enable_plugins: true,
                enable_federation: false,
                enable_advanced_routing: true,
                custom: HashMap::new(),
            },
            custom: HashMap::new(),
        }
    }
}

impl SquirrelUnifiedConfig {
    /// Validate the entire configuration
    ///
    /// Performs comprehensive validation across all configuration domains.
    /// Now uses the unified validation module for consistent error messages.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        use crate::unified::validation::Validator;

        let mut errors = Vec::new();

        // Validate timeouts
        if let Err(e) = self.timeouts.validate() {
            errors.push(format!("Timeout validation failed: {}", e));
        }

        // Validate network ports using unified validators
        if let Err(e) = Validator::validate_port(self.network.http_port) {
            errors.push(format!("HTTP port: {}", e));
        }
        if let Err(e) = Validator::validate_port(self.network.websocket_port) {
            errors.push(format!("WebSocket port: {}", e));
        }
        if let Err(e) = Validator::validate_ports_differ(
            self.network.http_port,
            self.network.websocket_port,
            "HTTP",
            "WebSocket",
        ) {
            errors.push(e.to_string());
        }

        // Validate security
        if self.security.enabled && self.security.require_authentication {
            if self.security.jwt_secret.is_none() && self.security.api_keys.is_empty() {
                errors.push(
                    "Authentication required but no JWT secret or API keys configured".to_string(),
                );
            }

            // Validate JWT secret length if provided
            if let Some(ref secret) = self.security.jwt_secret
                && let Err(e) = Validator::validate_jwt_secret(secret)
            {
                errors.push(format!("JWT secret: {}", e));
            }
        }

        // Validate monitoring ports using unified validators
        if self.monitoring.enabled && self.monitoring.enable_prometheus {
            if let Err(e) = Validator::validate_port(self.monitoring.prometheus_port) {
                errors.push(format!("Prometheus port: {}", e));
            }
            if let Err(e) = Validator::validate_ports_differ(
                self.monitoring.prometheus_port,
                self.network.http_port,
                "Prometheus",
                "HTTP",
            ) {
                errors.push(e.to_string());
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
