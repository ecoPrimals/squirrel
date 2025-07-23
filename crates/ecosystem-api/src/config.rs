//! Configuration management for ecosystem integration
//!
//! This module provides environment-driven configuration management
//! that eliminates hardcoded values and supports dynamic configuration
//! updates through the ecosystem.

use crate::error::*;
use crate::traits::{
    FeatureFlags, ResourceConfig, RetryConfig, ServiceConfig, SongbirdConfig, UniversalConfig,
};
use crate::types::*;
use std::collections::HashMap;
use std::env;
// Removed unused Duration import

/// Universal configuration loader
pub struct ConfigLoader {
    env_prefix: String,
    required_vars: Vec<String>,
}

impl ConfigLoader {
    /// Create a new configuration loader
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
    pub fn load_universal_config(&self) -> Result<UniversalConfig, ConfigError> {
        // Validate required environment variables
        self.validate_required_vars()?;

        // Load service configuration
        let service = self.load_service_config()?;

        // Load Songbird configuration
        let songbird = self.load_songbird_config()?;

        // Load security configuration
        let security = self.load_security_config()?;

        // Load resource configuration
        let resources = self.load_resource_config()?;

        // Load feature flags
        let features = self.load_feature_flags()?;

        // Load primal-specific configuration
        let primal_specific = self.load_primal_specific_config()?;

        Ok(UniversalConfig {
            service,
            songbird,
            security,
            resources,
            features,
            primal_specific,
        })
    }

    /// Load service configuration
    fn load_service_config(&self) -> Result<ServiceConfig, ConfigError> {
        let name = self.get_required_env("SERVICE_NAME")?;
        let version = env!("CARGO_PKG_VERSION").to_string();
        let description = self.get_required_env("SERVICE_DESCRIPTION")?;
        let bind_address = self.get_env_or_default("BIND_ADDRESS", "0.0.0.0");
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

    /// Load Songbird configuration
    fn load_songbird_config(&self) -> Result<SongbirdConfig, ConfigError> {
        let discovery_endpoint = self.get_required_env("SONGBIRD_DISCOVERY_ENDPOINT")?;
        let registration_endpoint = self.get_required_env("SONGBIRD_REGISTRATION_ENDPOINT")?;
        let health_endpoint = self.get_required_env("SONGBIRD_HEALTH_ENDPOINT")?;
        let auth_token = self.get_optional_env("SONGBIRD_AUTH_TOKEN");
        let heartbeat_interval_secs = self
            .get_env_or_default("SONGBIRD_HEARTBEAT_INTERVAL", "30")
            .parse::<u64>()
            .map_err(|e| ConfigError::InvalidValue {
                key: "SONGBIRD_HEARTBEAT_INTERVAL".to_string(),
                value: e.to_string(),
            })?;

        let retry_config = RetryConfig {
            max_retries: self
                .get_env_or_default("SONGBIRD_MAX_RETRIES", "3")
                .parse::<u32>()
                .map_err(|e| ConfigError::InvalidValue {
                    key: "SONGBIRD_MAX_RETRIES".to_string(),
                    value: e.to_string(),
                })?,
            initial_delay_ms: self
                .get_env_or_default("SONGBIRD_INITIAL_DELAY_MS", "1000")
                .parse::<u64>()
                .map_err(|e| ConfigError::InvalidValue {
                    key: "SONGBIRD_INITIAL_DELAY_MS".to_string(),
                    value: e.to_string(),
                })?,
            max_delay_ms: self
                .get_env_or_default("SONGBIRD_MAX_DELAY_MS", "30000")
                .parse::<u64>()
                .map_err(|e| ConfigError::InvalidValue {
                    key: "SONGBIRD_MAX_DELAY_MS".to_string(),
                    value: e.to_string(),
                })?,
            backoff_multiplier: self
                .get_env_or_default("SONGBIRD_BACKOFF_MULTIPLIER", "2.0")
                .parse::<f64>()
                .map_err(|e| ConfigError::InvalidValue {
                    key: "SONGBIRD_BACKOFF_MULTIPLIER".to_string(),
                    value: e.to_string(),
                })?,
        };

        Ok(SongbirdConfig {
            discovery_endpoint,
            registration_endpoint,
            health_endpoint,
            auth_token,
            retry_config,
            heartbeat_interval_secs,
        })
    }

    /// Load security configuration
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
                })
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

    /// Load resource configuration
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

    /// Load feature flags
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

    /// Load primal-specific configuration
    fn load_primal_specific_config(
        &self,
    ) -> Result<HashMap<String, serde_json::Value>, ConfigError> {
        let mut config = HashMap::new();

        // Scan for primal-specific environment variables
        for (key, value) in env::vars() {
            if key.starts_with(&format!("{}_", self.env_prefix)) {
                let config_key = match key.strip_prefix(&format!("{}_", self.env_prefix)) {
                    Some(key) => key.to_lowercase(),
                    None => continue, // Skip if prefix stripping fails
                };

                // Try to parse as JSON first, then fall back to string
                let config_value = match serde_json::from_str::<serde_json::Value>(&value) {
                    Ok(json_value) => json_value,
                    Err(_) => serde_json::Value::String(value),
                };

                config.insert(config_key, config_value);
            }
        }

        Ok(config)
    }

    /// Get required environment variable
    fn get_required_env(&self, var_name: &str) -> Result<String, ConfigError> {
        let full_var_name = format!("{}_{}", self.env_prefix, var_name);
        env::var(&full_var_name).map_err(|_| ConfigError::MissingEnvVar(full_var_name))
    }

    /// Get optional environment variable
    fn get_optional_env(&self, var_name: &str) -> Option<String> {
        let full_var_name = format!("{}_{}", self.env_prefix, var_name);
        env::var(&full_var_name).ok()
    }

    /// Get environment variable or default
    fn get_env_or_default(&self, var_name: &str, default: &str) -> String {
        self.get_optional_env(var_name)
            .unwrap_or_else(|| default.to_string())
    }

    /// Validate required environment variables
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

/// Configuration validation utilities
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate universal configuration
    pub fn validate_universal_config(config: &UniversalConfig) -> Result<(), ConfigError> {
        // Validate service configuration
        Self::validate_service_config(&config.service)?;

        // Validate Songbird configuration
        Self::validate_songbird_config(&config.songbird)?;

        // Validate security configuration
        Self::validate_security_config(&config.security)?;

        // Validate resource configuration
        Self::validate_resource_config(&config.resources)?;

        Ok(())
    }

    /// Validate service configuration
    fn validate_service_config(config: &ServiceConfig) -> Result<(), ConfigError> {
        if config.name.is_empty() {
            return Err(ConfigError::ValidationFailed(
                "Service name cannot be empty".to_string(),
            ));
        }

        if config.version.is_empty() {
            return Err(ConfigError::ValidationFailed(
                "Service version cannot be empty".to_string(),
            ));
        }

        if config.bind_address.is_empty() {
            return Err(ConfigError::ValidationFailed(
                "Bind address cannot be empty".to_string(),
            ));
        }

        if config.instance_id.is_empty() {
            return Err(ConfigError::ValidationFailed(
                "Instance ID cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate Songbird configuration
    fn validate_songbird_config(config: &SongbirdConfig) -> Result<(), ConfigError> {
        if config.discovery_endpoint.is_empty() {
            return Err(ConfigError::ValidationFailed(
                "Discovery endpoint cannot be empty".to_string(),
            ));
        }

        if config.registration_endpoint.is_empty() {
            return Err(ConfigError::ValidationFailed(
                "Registration endpoint cannot be empty".to_string(),
            ));
        }

        if config.health_endpoint.is_empty() {
            return Err(ConfigError::ValidationFailed(
                "Health endpoint cannot be empty".to_string(),
            ));
        }

        // Validate URLs
        url::Url::parse(&config.discovery_endpoint).map_err(|_| {
            ConfigError::ValidationFailed("Invalid discovery endpoint URL".to_string())
        })?;

        url::Url::parse(&config.registration_endpoint).map_err(|_| {
            ConfigError::ValidationFailed("Invalid registration endpoint URL".to_string())
        })?;

        url::Url::parse(&config.health_endpoint).map_err(|_| {
            ConfigError::ValidationFailed("Invalid health endpoint URL".to_string())
        })?;

        // Validate retry configuration
        if config.retry_config.max_retries == 0 {
            return Err(ConfigError::ValidationFailed(
                "Max retries must be greater than 0".to_string(),
            ));
        }

        if config.retry_config.initial_delay_ms == 0 {
            return Err(ConfigError::ValidationFailed(
                "Initial delay must be greater than 0".to_string(),
            ));
        }

        if config.retry_config.max_delay_ms < config.retry_config.initial_delay_ms {
            return Err(ConfigError::ValidationFailed(
                "Max delay must be greater than initial delay".to_string(),
            ));
        }

        if config.retry_config.backoff_multiplier <= 1.0 {
            return Err(ConfigError::ValidationFailed(
                "Backoff multiplier must be greater than 1.0".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate security configuration
    fn validate_security_config(config: &SecurityConfig) -> Result<(), ConfigError> {
        if config.auth_method.is_empty() {
            return Err(ConfigError::ValidationFailed(
                "Auth method cannot be empty".to_string(),
            ));
        }

        if config.trust_domain.is_empty() {
            return Err(ConfigError::ValidationFailed(
                "Trust domain cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate resource configuration
    fn validate_resource_config(config: &ResourceConfig) -> Result<(), ConfigError> {
        if let Some(cpu_cores) = config.cpu_cores {
            if cpu_cores <= 0.0 {
                return Err(ConfigError::ValidationFailed(
                    "CPU cores must be greater than 0".to_string(),
                ));
            }
        }

        if let Some(memory_mb) = config.memory_mb {
            if memory_mb == 0 {
                return Err(ConfigError::ValidationFailed(
                    "Memory must be greater than 0".to_string(),
                ));
            }
        }

        if let Some(disk_mb) = config.disk_mb {
            if disk_mb == 0 {
                return Err(ConfigError::ValidationFailed(
                    "Disk space must be greater than 0".to_string(),
                ));
            }
        }

        if let Some(network_bandwidth_mbps) = config.network_bandwidth_mbps {
            if network_bandwidth_mbps == 0 {
                return Err(ConfigError::ValidationFailed(
                    "Network bandwidth must be greater than 0".to_string(),
                ));
            }
        }

        if let Some(gpu_count) = config.gpu_count {
            if gpu_count == 0 {
                return Err(ConfigError::ValidationFailed(
                    "GPU count must be greater than 0".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// Configuration defaults for common scenarios
pub struct ConfigDefaults;

impl ConfigDefaults {
    /// Get default configuration for development
    pub fn development() -> UniversalConfig {
        UniversalConfig {
            service: ServiceConfig {
                name: "squirrel-dev".to_string(),
                version: "0.1.0".to_string(),
                description: "Development Squirrel instance".to_string(),
                bind_address: crate::defaults::DefaultEndpoints::dev_bind_address(),
                port: 8080,
                log_level: "debug".to_string(),
                instance_id: uuid::Uuid::new_v4().to_string(),
            },
            songbird: SongbirdConfig {
                discovery_endpoint: crate::defaults::DefaultEndpoints::discovery_endpoint(),
                registration_endpoint: crate::defaults::DefaultEndpoints::registration_endpoint(),
                health_endpoint: format!(
                    "{}/api/v1/health",
                    crate::defaults::DefaultEndpoints::songbird_endpoint()
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
    pub fn production() -> UniversalConfig {
        UniversalConfig {
            service: ServiceConfig {
                name: "squirrel".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                description: "Production Squirrel AI primal".to_string(),
                bind_address: "0.0.0.0".to_string(),
                port: 0, // Dynamic port from Songbird
                log_level: "info".to_string(),
                instance_id: uuid::Uuid::new_v4().to_string(),
            },
            songbird: SongbirdConfig {
                discovery_endpoint: "".to_string(), // Must be provided via environment
                registration_endpoint: "".to_string(), // Must be provided via environment
                health_endpoint: "".to_string(),    // Must be provided via environment
                auth_token: None,                   // Must be provided via environment
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
                cpu_cores: None,              // Will be provided by container orchestrator
                memory_mb: None,              // Will be provided by container orchestrator
                disk_mb: None,                // Will be provided by container orchestrator
                network_bandwidth_mbps: None, // Will be provided by container orchestrator
                gpu_count: None,              // Will be provided by container orchestrator
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
