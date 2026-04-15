// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Configuration loader for ecosystem integration
//!
//! Service mesh settings use `SERVICE_MESH_*` environment keys (prefixed by the loader’s
//! `env_prefix`). `SONGBIRD_*` keys are **deprecated legacy fallbacks** for the same values—prefer
//! the `SERVICE_MESH_*` names so configuration stays capability-oriented rather than primal-named.

use crate::error::ConfigError;
use crate::traits::{
    FeatureFlags, ResourceConfig, RetryConfig, ServiceConfig, ServiceMeshConfig, UniversalConfig,
};
use crate::types::{SecurityConfig, SecurityLevel};
use std::collections::HashMap;
use std::env;
use universal_constants::network::BIND_ALL_INTERFACES;

/// Universal configuration loader
pub struct ConfigLoader {
    env_prefix: String,
    required_vars: Vec<String>,
}

impl ConfigLoader {
    /// Create a new configuration loader
    #[must_use]
    pub fn new(env_prefix: &str) -> Self {
        Self {
            env_prefix: env_prefix.to_string(),
            required_vars: Vec::new(),
        }
    }

    /// Add a required environment variable
    pub fn require_var(&mut self, var_name: &str) -> &mut Self {
        self.required_vars.push(var_name.to_string());
        self
    }

    /// Load universal configuration from environment
    ///
    /// # Errors
    ///
    /// Returns an error when required variables are missing or any section fails to parse.
    pub fn load_universal_config(&self) -> Result<UniversalConfig, ConfigError> {
        self.validate_required_vars()?;
        let service = self.load_service_config()?;
        let service_mesh = self.load_service_mesh_config()?;
        let security = self.load_security_config()?;
        let resources = self.load_resource_config()?;
        let features = self.load_feature_flags()?;
        let primal_specific = self.load_primal_specific_config();

        Ok(UniversalConfig {
            service,
            service_mesh,
            security,
            resources,
            features,
            primal_specific,
        })
    }

    fn load_service_config(&self) -> Result<ServiceConfig, ConfigError> {
        let name = self.get_required_env("SERVICE_NAME")?;
        let version = env!("CARGO_PKG_VERSION").to_string();
        let description = self.get_required_env("SERVICE_DESCRIPTION")?;
        let bind_address = self.get_env_or_default("BIND_ADDRESS", BIND_ALL_INTERFACES);
        let port = self
            .get_env_or_default("PORT", "0")
            .parse::<u16>()
            .map_err(|e| ConfigError::InvalidValue {
                key: "PORT".to_string(),
                value: e.to_string(),
            })?;
        let log_level = self.get_env_or_default("LOG_LEVEL", "info");
        let instance_id = self.get_env_or_default("INSTANCE_ID", &uuid::Uuid::new_v4().to_string());

        Ok(ServiceConfig {
            name,
            version,
            description,
            bind_address,
            port,
            log_level,
            instance_id,
        })
    }

    fn load_service_mesh_config(&self) -> Result<ServiceMeshConfig, ConfigError> {
        // Legacy env var — prefer SERVICE_MESH_DISCOVERY_ENDPOINT
        let discovery_endpoint = self.get_required_env_with_legacy_fallback(
            "SERVICE_MESH_DISCOVERY_ENDPOINT",
            "SONGBIRD_DISCOVERY_ENDPOINT",
        )?;
        // Legacy env var — prefer SERVICE_MESH_REGISTRATION_ENDPOINT
        let registration_endpoint = self.get_required_env_with_legacy_fallback(
            "SERVICE_MESH_REGISTRATION_ENDPOINT",
            "SONGBIRD_REGISTRATION_ENDPOINT",
        )?;
        // Legacy env var — prefer SERVICE_MESH_HEALTH_ENDPOINT
        let health_endpoint = self.get_required_env_with_legacy_fallback(
            "SERVICE_MESH_HEALTH_ENDPOINT",
            "SONGBIRD_HEALTH_ENDPOINT",
        )?;
        // Legacy env var — prefer SERVICE_MESH_AUTH_TOKEN
        let auth_token = self.get_optional_env_with_legacy_fallback(
            "SERVICE_MESH_AUTH_TOKEN",
            "SONGBIRD_AUTH_TOKEN",
        );
        let heartbeat_interval_secs = self
            // Legacy env var — prefer SERVICE_MESH_HEARTBEAT_INTERVAL
            .get_env_or_default_with_legacy_fallback(
                "SERVICE_MESH_HEARTBEAT_INTERVAL",
                "SONGBIRD_HEARTBEAT_INTERVAL",
                "30",
            )
            .parse::<u64>()
            .map_err(|e| ConfigError::InvalidValue {
                key: "SERVICE_MESH_HEARTBEAT_INTERVAL".to_string(),
                value: e.to_string(),
            })?;

        let retry_config = RetryConfig {
            max_retries: self
                // Legacy env var — prefer SERVICE_MESH_MAX_RETRIES
                .get_env_or_default_with_legacy_fallback(
                    "SERVICE_MESH_MAX_RETRIES",
                    "SONGBIRD_MAX_RETRIES",
                    "3",
                )
                .parse::<u32>()
                .map_err(|e| ConfigError::InvalidValue {
                    key: "SERVICE_MESH_MAX_RETRIES".to_string(),
                    value: e.to_string(),
                })?,
            initial_delay_ms: self
                // Legacy env var — prefer SERVICE_MESH_INITIAL_DELAY_MS
                .get_env_or_default_with_legacy_fallback(
                    "SERVICE_MESH_INITIAL_DELAY_MS",
                    "SONGBIRD_INITIAL_DELAY_MS",
                    "1000",
                )
                .parse::<u64>()
                .map_err(|e| ConfigError::InvalidValue {
                    key: "SERVICE_MESH_INITIAL_DELAY_MS".to_string(),
                    value: e.to_string(),
                })?,
            max_delay_ms: self
                // Legacy env var — prefer SERVICE_MESH_MAX_DELAY_MS
                .get_env_or_default_with_legacy_fallback(
                    "SERVICE_MESH_MAX_DELAY_MS",
                    "SONGBIRD_MAX_DELAY_MS",
                    "30000",
                )
                .parse::<u64>()
                .map_err(|e| ConfigError::InvalidValue {
                    key: "SERVICE_MESH_MAX_DELAY_MS".to_string(),
                    value: e.to_string(),
                })?,
            backoff_multiplier: self
                // Legacy env var — prefer SERVICE_MESH_BACKOFF_MULTIPLIER
                .get_env_or_default_with_legacy_fallback(
                    "SERVICE_MESH_BACKOFF_MULTIPLIER",
                    "SONGBIRD_BACKOFF_MULTIPLIER",
                    "2.0",
                )
                .parse::<f64>()
                .map_err(|e| ConfigError::InvalidValue {
                    key: "SERVICE_MESH_BACKOFF_MULTIPLIER".to_string(),
                    value: e.to_string(),
                })?,
        };

        Ok(ServiceMeshConfig {
            discovery_endpoint,
            registration_endpoint,
            health_endpoint,
            auth_token,
            retry_config,
            heartbeat_interval_secs,
        })
    }

    fn load_security_config(&self) -> Result<SecurityConfig, ConfigError> {
        let auth_method = self.get_env_or_default("SECURITY_AUTH_METHOD", "bearer");
        let tls_enabled = self
            .get_env_or_default("SECURITY_TLS_ENABLED", "true")
            .parse::<bool>()
            .map_err(|e| ConfigError::InvalidValue {
                key: "SECURITY_TLS_ENABLED".to_string(),
                value: e.to_string(),
            })?;
        let mtls_required = self
            .get_env_or_default("SECURITY_MTLS_REQUIRED", "false")
            .parse::<bool>()
            .map_err(|e| ConfigError::InvalidValue {
                key: "SECURITY_MTLS_REQUIRED".to_string(),
                value: e.to_string(),
            })?;
        let trust_domain = self.get_env_or_default("SECURITY_TRUST_DOMAIN", "ecosystem.local");
        let security_level = match self
            .get_env_or_default("SECURITY_LEVEL", "internal")
            .as_str()
        {
            "public" => SecurityLevel::Public,
            "internal" => SecurityLevel::Internal,
            "restricted" => SecurityLevel::Restricted,
            "confidential" => SecurityLevel::Confidential,
            level => {
                return Err(ConfigError::InvalidValue {
                    key: "SECURITY_LEVEL".to_string(),
                    value: level.to_string(),
                });
            }
        };
        let crypto_lock_enabled = self
            .get_env_or_default("SECURITY_CRYPTO_LOCK_ENABLED", "false")
            .parse::<bool>()
            .map_err(|e| ConfigError::InvalidValue {
                key: "SECURITY_CRYPTO_LOCK_ENABLED".to_string(),
                value: e.to_string(),
            })?;

        Ok(SecurityConfig {
            auth_method,
            tls_enabled,
            mtls_required,
            trust_domain,
            security_level,
            crypto_lock_enabled,
        })
    }

    fn load_resource_config(&self) -> Result<ResourceConfig, ConfigError> {
        let cpu_cores = self
            .get_optional_env("RESOURCES_CPU_CORES")
            .map(|v| v.parse::<f64>())
            .transpose()
            .map_err(|e| ConfigError::InvalidValue {
                key: "RESOURCES_CPU_CORES".to_string(),
                value: e.to_string(),
            })?;

        let memory_mb = self
            .get_optional_env("RESOURCES_MEMORY_MB")
            .map(|v| v.parse::<u64>())
            .transpose()
            .map_err(|e| ConfigError::InvalidValue {
                key: "RESOURCES_MEMORY_MB".to_string(),
                value: e.to_string(),
            })?;

        let disk_mb = self
            .get_optional_env("RESOURCES_DISK_MB")
            .map(|v| v.parse::<u64>())
            .transpose()
            .map_err(|e| ConfigError::InvalidValue {
                key: "RESOURCES_DISK_MB".to_string(),
                value: e.to_string(),
            })?;

        let network_bandwidth_mbps = self
            .get_optional_env("RESOURCES_NETWORK_BANDWIDTH_MBPS")
            .map(|v| v.parse::<u64>())
            .transpose()
            .map_err(|e| ConfigError::InvalidValue {
                key: "RESOURCES_NETWORK_BANDWIDTH_MBPS".to_string(),
                value: e.to_string(),
            })?;

        let gpu_count = self
            .get_optional_env("RESOURCES_GPU_COUNT")
            .map(|v| v.parse::<u32>())
            .transpose()
            .map_err(|e| ConfigError::InvalidValue {
                key: "RESOURCES_GPU_COUNT".to_string(),
                value: e.to_string(),
            })?;

        Ok(ResourceConfig {
            cpu_cores,
            memory_mb,
            disk_mb,
            network_bandwidth_mbps,
            gpu_count,
        })
    }

    fn load_feature_flags(&self) -> Result<FeatureFlags, ConfigError> {
        let development_mode = self
            .get_env_or_default("FEATURES_DEVELOPMENT_MODE", "false")
            .parse::<bool>()
            .map_err(|e| ConfigError::InvalidValue {
                key: "FEATURES_DEVELOPMENT_MODE".to_string(),
                value: e.to_string(),
            })?;

        let debug_logging = self
            .get_env_or_default("FEATURES_DEBUG_LOGGING", "false")
            .parse::<bool>()
            .map_err(|e| ConfigError::InvalidValue {
                key: "FEATURES_DEBUG_LOGGING".to_string(),
                value: e.to_string(),
            })?;

        let metrics_enabled = self
            .get_env_or_default("FEATURES_METRICS_ENABLED", "true")
            .parse::<bool>()
            .map_err(|e| ConfigError::InvalidValue {
                key: "FEATURES_METRICS_ENABLED".to_string(),
                value: e.to_string(),
            })?;

        let tracing_enabled = self
            .get_env_or_default("FEATURES_TRACING_ENABLED", "true")
            .parse::<bool>()
            .map_err(|e| ConfigError::InvalidValue {
                key: "FEATURES_TRACING_ENABLED".to_string(),
                value: e.to_string(),
            })?;

        let experimental_features = self
            .get_env_or_default("FEATURES_EXPERIMENTAL", "")
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.trim().to_string())
            .collect();

        Ok(FeatureFlags {
            development_mode,
            debug_logging,
            metrics_enabled,
            tracing_enabled,
            experimental_features,
        })
    }

    fn load_primal_specific_config(&self) -> HashMap<String, serde_json::Value> {
        let mut config = HashMap::new();
        for (key, value) in env::vars() {
            if key.starts_with(&format!("{}_", self.env_prefix)) {
                let config_key = match key.strip_prefix(&format!("{}_", self.env_prefix)) {
                    Some(k) => k.to_lowercase(),
                    None => continue,
                };
                let config_value = serde_json::from_str::<serde_json::Value>(&value)
                    .map_or(serde_json::Value::String(value), |json_value| json_value);
                config.insert(config_key, config_value);
            }
        }
        config
    }

    fn get_required_env(&self, var_name: &str) -> Result<String, ConfigError> {
        let full_var_name = format!("{}_{}", self.env_prefix, var_name);
        env::var(&full_var_name).map_err(|_| ConfigError::MissingEnvVar(full_var_name))
    }

    fn get_required_env_with_legacy_fallback(
        &self,
        primary_var: &str,
        legacy_var: &str,
    ) -> Result<String, ConfigError> {
        let primary_full = format!("{}_{}", self.env_prefix, primary_var);
        let legacy_full = format!("{}_{}", self.env_prefix, legacy_var);
        env::var(&primary_full)
            .or_else(|_| {
                // Legacy env var — prefer capability-based name (`primary_var`, prefixed).
                env::var(&legacy_full)
            })
            .map_err(|_| ConfigError::MissingEnvVar(primary_full))
    }

    fn get_optional_env(&self, var_name: &str) -> Option<String> {
        let full_var_name = format!("{}_{}", self.env_prefix, var_name);
        env::var(&full_var_name).ok()
    }

    fn get_optional_env_with_legacy_fallback(
        &self,
        primary_var: &str,
        legacy_var: &str,
    ) -> Option<String> {
        let primary_full = format!("{}_{}", self.env_prefix, primary_var);
        let legacy_full = format!("{}_{}", self.env_prefix, legacy_var);
        env::var(&primary_full).ok().or_else(|| {
            // Legacy env var — prefer capability-based name (`primary_var`, prefixed).
            env::var(&legacy_full).ok()
        })
    }

    pub(crate) fn get_env_or_default(&self, var_name: &str, default: &str) -> String {
        self.get_optional_env(var_name)
            .unwrap_or_else(|| default.to_string())
    }

    fn get_env_or_default_with_legacy_fallback(
        &self,
        primary_var: &str,
        legacy_var: &str,
        default: &str,
    ) -> String {
        self.get_optional_env_with_legacy_fallback(primary_var, legacy_var)
            .unwrap_or_else(|| default.to_string())
    }

    fn validate_required_vars(&self) -> Result<(), ConfigError> {
        for var_name in &self.required_vars {
            let full_var_name = format!("{}_{}", self.env_prefix, var_name);
            if env::var(&full_var_name).is_err() {
                return Err(ConfigError::MissingEnvVar(full_var_name));
            }
        }
        Ok(())
    }
}
