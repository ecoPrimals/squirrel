// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Configuration implementation methods
//!
//! This module provides implementation methods for configuration management
//! including loading, validation, saving, and custom value handling.

use super::builder::ConfigBuilder;
use super::loader::ConfigLoader;
use super::validation::ConfigValidator;
use super::{ConfigError, PrimalConfig, UniversalPrimalConfig};

impl PrimalConfig {
    /// Create a new configuration builder
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }

    /// Load configuration from a file
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, ConfigError> {
        ConfigLoader::from_file(path)
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        ConfigLoader::from_env()
    }

    /// Load configuration using the default strategy
    /// (environment variables first, then default config file)
    pub fn load() -> Result<Self, ConfigError> {
        ConfigLoader::load()
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        ConfigValidator::validate(self)
    }

    /// Save configuration to a file
    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), ConfigError> {
        let content = serde_yml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get a custom configuration value
    pub fn get_custom<T>(&self, key: &str) -> Result<Option<T>, ConfigError>
    where
        T: serde::de::DeserializeOwned,
    {
        match self.custom.get(key) {
            Some(value) => {
                let parsed = serde_json::from_value(value.clone())?;
                Ok(Some(parsed))
            }
            None => Ok(None),
        }
    }

    /// Set a custom configuration value
    pub fn set_custom<T>(&mut self, key: &str, value: T) -> Result<(), ConfigError>
    where
        T: serde::Serialize,
    {
        let json_value = serde_json::to_value(value)?;
        self.custom.insert(key.to_string(), json_value);
        Ok(())
    }

    /// Get a feature flag value
    pub fn get_feature(&self, feature: &str) -> bool {
        self.environment
            .features
            .get(feature)
            .copied()
            .unwrap_or(false)
    }

    /// Set a feature flag
    pub fn set_feature(&mut self, feature: &str, enabled: bool) {
        self.environment
            .features
            .insert(feature.to_string(), enabled);
    }

    /// Get an environment variable
    pub fn get_env_var(&self, var: &str) -> Option<&String> {
        self.environment.variables.get(var)
    }

    /// Set an environment variable
    pub fn set_env_var(&mut self, var: &str, value: &str) {
        self.environment
            .variables
            .insert(var.to_string(), value.to_string());
    }

    /// Get the full bind address (address:port)
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.network.bind_address, self.network.port)
    }

    /// Get the public address or fallback to bind address
    pub fn public_address(&self) -> String {
        self.network
            .public_address
            .clone()
            .unwrap_or_else(|| self.bind_address())
    }

    /// Check if TLS is enabled
    pub fn is_tls_enabled(&self) -> bool {
        self.network.tls.is_some()
    }

    /// Check if orchestration is enabled
    pub fn is_orchestration_enabled(&self) -> bool {
        self.orchestration.enabled
    }

    /// Check if security audit logging is enabled
    pub fn is_audit_logging_enabled(&self) -> bool {
        self.security.audit_logging
    }

    /// Get the effective log level as a string
    pub fn log_level_str(&self) -> &'static str {
        match self.logging.level {
            super::types::LogLevel::Trace => "trace",
            super::types::LogLevel::Debug => "debug",
            super::types::LogLevel::Info => "info",
            super::types::LogLevel::Warn => "warn",
            super::types::LogLevel::Error => "error",
        }
    }

    /// Check if this is a development environment
    pub fn is_development(&self) -> bool {
        self.environment.name == "development"
    }

    /// Check if this is a production environment
    pub fn is_production(&self) -> bool {
        self.environment.name == "production"
    }

    /// Merge another configuration into this one
    pub fn merge(&mut self, other: PrimalConfig) -> Result<(), ConfigError> {
        // Merge custom fields
        for (key, value) in other.custom {
            self.custom.insert(key, value);
        }

        // Merge environment variables
        for (key, value) in other.environment.variables {
            self.environment.variables.insert(key, value);
        }

        // Merge feature flags
        for (key, value) in other.environment.features {
            self.environment.features.insert(key, value);
        }

        // Update other fields if they're not defaults
        if other.network.port != 8080 {
            self.network.port = other.network.port;
        }

        if other.network.bind_address != "127.0.0.1" {
            self.network.bind_address = other.network.bind_address;
        }

        if other.network.tls.is_some() {
            self.network.tls = other.network.tls;
        }

        Ok(())
    }

    /// Clone configuration with environment overrides
    pub fn with_environment(&self, env_name: &str) -> Self {
        let mut config = self.clone();
        config.environment.name = env_name.to_string();

        // Apply environment-specific defaults
        match env_name {
            "development" => {
                config.logging.level = super::types::LogLevel::Debug;
                config.logging.tracing = true;
                config.security.audit_logging = false;
            }
            "production" => {
                config.logging.level = super::types::LogLevel::Warn;
                config.logging.format = super::types::LogFormat::Json;
                config.security.audit_logging = true;
                config.security.encryption.enable_inter_primal = true;
            }
            "testing" => {
                config.logging.level = super::types::LogLevel::Error;
                config.network.port = 0; // Random port
                config.orchestration.enabled = false;
            }
            _ => {}
        }

        config
    }
}

impl UniversalPrimalConfig {
    /// Load universal configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(discovery) = std::env::var("PRIMAL_AUTO_DISCOVERY") {
            config.auto_discovery_enabled = discovery.parse().unwrap_or(true);
        }

        if let Ok(max_per_type) = std::env::var("PRIMAL_MAX_INSTANCES_PER_TYPE") {
            if let Ok(num) = max_per_type.parse::<usize>() {
                config.multi_instance.max_instances_per_type = num;
            }
        }

        if let Ok(max_per_user) = std::env::var("PRIMAL_MAX_INSTANCES_PER_USER") {
            if let Ok(num) = max_per_user.parse::<usize>() {
                config.multi_instance.max_instances_per_user = num;
            }
        }

        if let Ok(port_start) = std::env::var("PRIMAL_PORT_RANGE_START") {
            if let Ok(start) = port_start.parse::<u16>() {
                config.port_management.port_range.start = start;
            }
        }

        if let Ok(port_end) = std::env::var("PRIMAL_PORT_RANGE_END") {
            if let Ok(end) = port_end.parse::<u16>() {
                config.port_management.port_range.end = end;
            }
        }

        config
    }

    /// Validate the universal configuration
    pub fn validate(&self) -> Result<(), String> {
        // Check port range validity
        if self.port_management.port_range.start >= self.port_management.port_range.end {
            return Err("Port range start must be less than end".to_string());
        }

        // Check instance limits
        if self.multi_instance.max_instances_per_type == 0 {
            return Err("Max instances per type must be greater than 0".to_string());
        }

        if self.multi_instance.max_instances_per_user == 0 {
            return Err("Max instances per user must be greater than 0".to_string());
        }

        // Check timeout values
        if self.timeouts.connect == 0 {
            return Err("Connect timeout must be greater than 0".to_string());
        }

        if self.timeouts.request == 0 {
            return Err("Request timeout must be greater than 0".to_string());
        }

        // Validate scaling configuration
        if self.multi_instance.scaling.auto_scaling_enabled {
            if self.multi_instance.scaling.min_instances
                >= self.multi_instance.scaling.max_instances
            {
                return Err("Min instances must be less than max instances".to_string());
            }

            if self.multi_instance.scaling.scale_up_cpu_threshold
                <= self.multi_instance.scaling.scale_down_cpu_threshold
            {
                return Err(
                    "Scale up CPU threshold must be greater than scale down threshold".to_string(),
                );
            }
        }

        // Validate port management
        let port_range_size = (self.port_management.port_range.end
            - self.port_management.port_range.start
            + 1) as usize;
        let max_total_instances = self.multi_instance.max_instances_per_type * 10; // Assume max 10 primal types

        if port_range_size < max_total_instances {
            return Err(format!(
                "Port range size ({port_range_size}) is too small for max instances ({max_total_instances})"
            ));
        }

        Ok(())
    }

    /// Add a primal instance configuration
    pub fn add_instance(&mut self, name: String, config: super::types::PrimalInstanceConfig) {
        self.primal_instances.insert(name, config);
    }

    /// Remove a primal instance configuration
    pub fn remove_instance(&mut self, name: &str) -> Option<super::types::PrimalInstanceConfig> {
        self.primal_instances.remove(name)
    }

    /// Get a primal instance configuration
    pub fn get_instance(&self, name: &str) -> Option<&super::types::PrimalInstanceConfig> {
        self.primal_instances.get(name)
    }

    /// Get all instance names
    pub fn instance_names(&self) -> Vec<&String> {
        self.primal_instances.keys().collect()
    }

    /// Get the number of configured instances
    pub fn instance_count(&self) -> usize {
        self.primal_instances.len()
    }

    /// Check if an instance exists
    pub fn has_instance(&self, name: &str) -> bool {
        self.primal_instances.contains_key(name)
    }

    /// Enable auto-scaling for the configuration
    pub fn enable_auto_scaling(&mut self, min_instances: usize, max_instances: usize) {
        self.multi_instance.scaling.auto_scaling_enabled = true;
        self.multi_instance.scaling.min_instances = min_instances;
        self.multi_instance.scaling.max_instances = max_instances;
    }

    /// Disable auto-scaling
    pub fn disable_auto_scaling(&mut self) {
        self.multi_instance.scaling.auto_scaling_enabled = false;
    }

    /// Set load balancing strategy
    pub fn set_load_balancing(&mut self, strategy: super::types::LoadBalancingStrategy) {
        self.multi_instance.load_balancing_strategy = strategy;
    }

    /// Enable failover with specified settings
    pub fn enable_failover(&mut self, max_retries: u32, retry_delay: u64) {
        self.multi_instance.failover.enabled = true;
        self.multi_instance.failover.max_retries = max_retries;
        self.multi_instance.failover.retry_delay_seconds = retry_delay;
    }

    /// Disable failover
    pub fn disable_failover(&mut self) {
        self.multi_instance.failover.enabled = false;
    }

    /// Set port range for dynamic allocation
    pub fn set_port_range(&mut self, start: u16, end: u16) -> Result<(), String> {
        if start >= end {
            return Err("Start port must be less than end port".to_string());
        }

        self.port_management.port_range.start = start;
        self.port_management.port_range.end = end;
        Ok(())
    }

    /// Add reserved ports
    pub fn add_reserved_ports(&mut self, ports: &[u16]) {
        for &port in ports {
            if !self.port_management.reserved_ports.contains(&port) {
                self.port_management.reserved_ports.push(port);
            }
        }
    }

    /// Remove reserved ports
    pub fn remove_reserved_ports(&mut self, ports: &[u16]) {
        for &port in ports {
            self.port_management.reserved_ports.retain(|&p| p != port);
        }
    }

    /// Check if a port is reserved
    pub fn is_port_reserved(&self, port: u16) -> bool {
        self.port_management.reserved_ports.contains(&port)
    }

    /// Get available port count
    pub fn available_port_count(&self) -> usize {
        let total_ports = (self.port_management.port_range.end
            - self.port_management.port_range.start
            + 1) as usize;
        let reserved_in_range = self
            .port_management
            .reserved_ports
            .iter()
            .filter(|&&port| {
                port >= self.port_management.port_range.start
                    && port <= self.port_management.port_range.end
            })
            .count();

        total_ports.saturating_sub(reserved_in_range)
    }

    /// Create a development configuration
    pub fn for_development() -> Self {
        super::presets::UniversalConfigPresets::single_user_dev()
    }

    /// Create a production configuration
    pub fn for_production() -> Self {
        super::presets::UniversalConfigPresets::multi_user_prod()
    }

    /// Create a testing configuration
    pub fn for_testing() -> Self {
        super::presets::UniversalConfigPresets::testing()
    }

    /// Create a high-availability configuration
    pub fn for_high_availability() -> Self {
        super::presets::UniversalConfigPresets::high_availability()
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::*;

    #[test]
    fn test_primal_config_custom_values() {
        let mut config = PrimalConfig::default();

        // Test setting and getting custom values
        config.set_custom("test_string", "hello").unwrap();
        config.set_custom("test_number", 42).unwrap();
        config.set_custom("test_bool", true).unwrap();

        let string_val: Option<String> = config.get_custom("test_string").unwrap();
        let number_val: Option<i32> = config.get_custom("test_number").unwrap();
        let bool_val: Option<bool> = config.get_custom("test_bool").unwrap();

        assert_eq!(string_val, Some("hello".to_string()));
        assert_eq!(number_val, Some(42));
        assert_eq!(bool_val, Some(true));

        let missing_val: Option<String> = config.get_custom("missing").unwrap();
        assert_eq!(missing_val, None);
    }

    #[test]
    fn test_primal_config_features() {
        let mut config = PrimalConfig::default();

        // Test feature flags
        assert!(!config.get_feature("test_feature"));

        config.set_feature("test_feature", true);
        assert!(config.get_feature("test_feature"));

        config.set_feature("test_feature", false);
        assert!(!config.get_feature("test_feature"));
    }

    #[test]
    fn test_primal_config_environment_detection() {
        let config = PrimalConfig::default();
        assert!(config.is_development());
        assert!(!config.is_production());

        let prod_config = config.with_environment("production");
        assert!(!prod_config.is_development());
        assert!(prod_config.is_production());
    }

    #[test]
    fn test_universal_config_validation() {
        let mut config = UniversalPrimalConfig::default();

        // Valid configuration should pass
        assert!(config.validate().is_ok());

        // Invalid port range should fail
        config.port_management.port_range.start = 9000;
        config.port_management.port_range.end = 8000;
        assert!(config.validate().is_err());

        // Fix port range
        config.port_management.port_range.start = 8000;
        config.port_management.port_range.end = 9000;
        assert!(config.validate().is_ok());

        // Invalid instance limits should fail
        config.multi_instance.max_instances_per_type = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_universal_config_instance_management() {
        let mut config = UniversalPrimalConfig::default();

        let instance_config = PrimalInstanceConfig::new(
            "http://localhost:8080".to_string(),
            "test-instance".to_string(),
            "user-123".to_string(),
            "device-456".to_string(),
        );

        // Add instance
        config.add_instance("test".to_string(), instance_config);
        assert_eq!(config.instance_count(), 1);
        assert!(config.has_instance("test"));

        // Get instance
        let retrieved = config.get_instance("test");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().instance_id, "test-instance");

        // Remove instance
        let removed = config.remove_instance("test");
        assert!(removed.is_some());
        assert_eq!(config.instance_count(), 0);
        assert!(!config.has_instance("test"));
    }

    #[test]
    fn test_universal_config_port_management() {
        let mut config = UniversalPrimalConfig::default();

        // Test setting port range
        assert!(config.set_port_range(8000, 9000).is_ok());
        assert!(config.set_port_range(9000, 8000).is_err()); // Invalid range

        // Test reserved ports
        config.add_reserved_ports(&[8080, 8443]);
        assert!(config.is_port_reserved(8080));
        assert!(config.is_port_reserved(8443));
        assert!(!config.is_port_reserved(8081));

        config.remove_reserved_ports(&[8080]);
        assert!(!config.is_port_reserved(8080));
        assert!(config.is_port_reserved(8443));

        // Test available port count
        let available = config.available_port_count();
        assert!(available > 0);
    }

    #[test]
    fn test_universal_config_scaling() {
        let mut config = UniversalPrimalConfig::default();

        // Enable auto-scaling
        config.enable_auto_scaling(2, 10);
        assert!(config.multi_instance.scaling.auto_scaling_enabled);
        assert_eq!(config.multi_instance.scaling.min_instances, 2);
        assert_eq!(config.multi_instance.scaling.max_instances, 10);

        // Disable auto-scaling
        config.disable_auto_scaling();
        assert!(!config.multi_instance.scaling.auto_scaling_enabled);
    }

    #[test]
    fn test_universal_config_failover() {
        let mut config = UniversalPrimalConfig::default();

        // Enable failover
        config.enable_failover(5, 10);
        assert!(config.multi_instance.failover.enabled);
        assert_eq!(config.multi_instance.failover.max_retries, 5);
        assert_eq!(config.multi_instance.failover.retry_delay_seconds, 10);

        // Disable failover
        config.disable_failover();
        assert!(!config.multi_instance.failover.enabled);
    }
}
