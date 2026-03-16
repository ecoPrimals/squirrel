// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Configuration builder for universal patterns
//!
//! This module provides a fluent builder API for creating universal primal configurations.

use crate::config::{LoadBalancingStrategy, PrimalInstanceConfig, UniversalPrimalConfig};
use crate::traits::SecurityLevel;

/// Builder for UniversalPrimalConfig
#[derive(Debug, Default)]
#[must_use = "call `.build()` to construct the configuration"]
pub struct UniversalConfigBuilder {
    config: UniversalPrimalConfig,
}

impl UniversalConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            config: UniversalPrimalConfig::default(),
        }
    }

    /// Enable or disable auto-discovery
    pub fn auto_discovery(mut self, enabled: bool) -> Self {
        self.config.auto_discovery_enabled = enabled;
        self
    }

    /// Set maximum instances per primal type
    pub fn max_instances_per_type(mut self, max: usize) -> Self {
        self.config.multi_instance.max_instances_per_type = max;
        self
    }

    /// Set maximum instances per user
    pub fn max_instances_per_user(mut self, max: usize) -> Self {
        self.config.multi_instance.max_instances_per_user = max;
        self
    }

    /// Set load balancing strategy
    pub fn load_balancing_strategy(mut self, strategy: LoadBalancingStrategy) -> Self {
        self.config.multi_instance.load_balancing_strategy = strategy;
        self
    }

    /// Enable metrics collection
    pub fn metrics_enabled(mut self, enabled: bool) -> Self {
        self.config.monitoring.metrics_enabled = enabled;
        self
    }

    /// Set tracing level
    pub fn tracing_level(mut self, level: String) -> Self {
        self.config.monitoring.tracing.level = level;
        self
    }

    /// Set port range for dynamic allocation
    pub fn port_range(mut self, start: u16, end: u16) -> Self {
        self.config.port_management.port_range.start = start;
        self.config.port_management.port_range.end = end;
        self
    }

    /// Add a primal instance configuration
    pub fn add_primal_instance(
        mut self,
        instance_id: String,
        instance_config: PrimalInstanceConfig,
    ) -> Self {
        self.config
            .primal_instances
            .insert(instance_id, instance_config);
        self
    }

    /// Build the final configuration
    #[must_use = "call this to finish building the configuration"]
    #[inline]
    pub fn build(self) -> UniversalPrimalConfig {
        self.config
    }
}

/// Builder for PrimalInstanceConfig
#[derive(Debug)]
pub struct PrimalInstanceConfigBuilder {
    base_url: String,
    instance_id: String,
    user_id: String,
    device_id: String,
    security_level: SecurityLevel,
    api_key: Option<String>,
    timeout_seconds: u64,
}

impl PrimalInstanceConfigBuilder {
    /// Create a new primal instance configuration builder
    pub fn new(base_url: String, instance_id: String, user_id: String, device_id: String) -> Self {
        Self {
            base_url,
            instance_id,
            user_id,
            device_id,
            security_level: SecurityLevel::Standard,
            api_key: None,
            timeout_seconds: 30,
        }
    }

    /// Set security level
    pub fn security_level(mut self, level: SecurityLevel) -> Self {
        self.security_level = level;
        self
    }

    /// Set API key
    pub fn api_key(mut self, key: String) -> Self {
        self.api_key = Some(key);
        self
    }

    /// Set timeout
    pub fn timeout_seconds(mut self, timeout: u64) -> Self {
        self.timeout_seconds = timeout;
        self
    }

    /// Build the final primal instance configuration
    pub fn build(self) -> PrimalInstanceConfig {
        PrimalInstanceConfig::new(
            self.base_url,
            self.instance_id,
            self.user_id,
            self.device_id,
        )
        .with_security_level(match self.security_level {
            SecurityLevel::Basic => "basic".to_string(),
            SecurityLevel::Standard => "standard".to_string(),
            SecurityLevel::High => "high".to_string(),
            SecurityLevel::Maximum => "maximum".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_universal_config_builder_new() {
        let builder = UniversalConfigBuilder::new();
        let config = builder.build();

        // Verify defaults are set
        assert!(config.auto_discovery_enabled);
        assert!(config.monitoring.metrics_enabled);
    }

    #[test]
    fn test_universal_config_builder_auto_discovery() {
        let config = UniversalConfigBuilder::new().auto_discovery(false).build();

        assert!(!config.auto_discovery_enabled);
    }

    #[test]
    fn test_universal_config_builder_max_instances() {
        let config = UniversalConfigBuilder::new()
            .max_instances_per_type(10)
            .max_instances_per_user(5)
            .build();

        assert_eq!(config.multi_instance.max_instances_per_type, 10);
        assert_eq!(config.multi_instance.max_instances_per_user, 5);
    }

    #[test]
    fn test_universal_config_builder_load_balancing() {
        let config = UniversalConfigBuilder::new()
            .load_balancing_strategy(LoadBalancingStrategy::LeastConnections)
            .build();

        assert_eq!(
            config.multi_instance.load_balancing_strategy,
            LoadBalancingStrategy::LeastConnections
        );
    }

    #[test]
    fn test_universal_config_builder_metrics() {
        let config = UniversalConfigBuilder::new().metrics_enabled(false).build();

        assert!(!config.monitoring.metrics_enabled);
    }

    #[test]
    fn test_universal_config_builder_tracing_level() {
        let config = UniversalConfigBuilder::new()
            .tracing_level("debug".to_string())
            .build();

        assert_eq!(config.monitoring.tracing.level, "debug");
    }

    #[test]
    fn test_universal_config_builder_port_range() {
        let config = UniversalConfigBuilder::new().port_range(9000, 9100).build();

        assert_eq!(config.port_management.port_range.start, 9000);
        assert_eq!(config.port_management.port_range.end, 9100);
    }

    #[test]
    fn test_universal_config_builder_add_primal_instance() {
        let instance_config = PrimalInstanceConfig::new(
            "http://localhost:8080".to_string(),
            "test-instance".to_string(),
            "user123".to_string(),
            "device456".to_string(),
        );

        let config = UniversalConfigBuilder::new()
            .add_primal_instance("test-instance".to_string(), instance_config)
            .build();

        assert_eq!(config.primal_instances.len(), 1);
        assert!(config.primal_instances.contains_key("test-instance"));
    }

    #[test]
    fn test_universal_config_builder_chaining() {
        let config = UniversalConfigBuilder::new()
            .auto_discovery(true)
            .max_instances_per_type(20)
            .max_instances_per_user(10)
            .load_balancing_strategy(LoadBalancingStrategy::RoundRobin)
            .metrics_enabled(true)
            .tracing_level("info".to_string())
            .port_range(8000, 8100)
            .build();

        assert!(config.auto_discovery_enabled);
        assert_eq!(config.multi_instance.max_instances_per_type, 20);
        assert_eq!(config.multi_instance.max_instances_per_user, 10);
        assert_eq!(
            config.multi_instance.load_balancing_strategy,
            LoadBalancingStrategy::RoundRobin
        );
        assert!(config.monitoring.metrics_enabled);
        assert_eq!(config.monitoring.tracing.level, "info");
        assert_eq!(config.port_management.port_range.start, 8000);
        assert_eq!(config.port_management.port_range.end, 8100);
    }

    #[test]
    fn test_primal_instance_config_builder_new() {
        let builder = PrimalInstanceConfigBuilder::new(
            "http://localhost:8080".to_string(),
            "instance-1".to_string(),
            "user-1".to_string(),
            "device-1".to_string(),
        );
        let config = builder.build();

        assert_eq!(config.base_url, "http://localhost:8080");
        assert_eq!(config.instance_id, "instance-1");
    }

    #[test]
    fn test_primal_instance_config_builder_security_levels() {
        // Test Basic
        let config = PrimalInstanceConfigBuilder::new(
            "http://localhost:8080".to_string(),
            "instance-1".to_string(),
            "user-1".to_string(),
            "device-1".to_string(),
        )
        .security_level(SecurityLevel::Basic)
        .build();
        assert_eq!(config.security_level, "basic");

        // Test Standard (default)
        let config = PrimalInstanceConfigBuilder::new(
            "http://localhost:8080".to_string(),
            "instance-2".to_string(),
            "user-1".to_string(),
            "device-1".to_string(),
        )
        .security_level(SecurityLevel::Standard)
        .build();
        assert_eq!(config.security_level, "standard");

        // Test High
        let config = PrimalInstanceConfigBuilder::new(
            "http://localhost:8080".to_string(),
            "instance-3".to_string(),
            "user-1".to_string(),
            "device-1".to_string(),
        )
        .security_level(SecurityLevel::High)
        .build();
        assert_eq!(config.security_level, "high");

        // Test Maximum
        let config = PrimalInstanceConfigBuilder::new(
            "http://localhost:8080".to_string(),
            "instance-4".to_string(),
            "user-1".to_string(),
            "device-1".to_string(),
        )
        .security_level(SecurityLevel::Maximum)
        .build();
        assert_eq!(config.security_level, "maximum");
    }

    #[test]
    fn test_primal_instance_config_builder_api_key() {
        let builder = PrimalInstanceConfigBuilder::new(
            "http://localhost:8080".to_string(),
            "instance-1".to_string(),
            "user-1".to_string(),
            "device-1".to_string(),
        )
        .api_key("test-api-key-12345".to_string());

        let config = builder.build();
        assert_eq!(config.base_url, "http://localhost:8080");
    }

    #[test]
    fn test_primal_instance_config_builder_timeout() {
        let builder = PrimalInstanceConfigBuilder::new(
            "http://localhost:8080".to_string(),
            "instance-1".to_string(),
            "user-1".to_string(),
            "device-1".to_string(),
        )
        .timeout_seconds(60);

        let config = builder.build();
        assert_eq!(config.base_url, "http://localhost:8080");
    }

    #[test]
    fn test_primal_instance_config_builder_chaining() {
        let config = PrimalInstanceConfigBuilder::new(
            "http://api.example.com".to_string(),
            "prod-instance-1".to_string(),
            "user-123".to_string(),
            "device-456".to_string(),
        )
        .security_level(SecurityLevel::Maximum)
        .api_key("secure-key-789".to_string())
        .timeout_seconds(120)
        .build();

        assert_eq!(config.base_url, "http://api.example.com");
        assert_eq!(config.instance_id, "prod-instance-1");
        assert_eq!(config.security_level, "maximum");
    }

    #[test]
    fn test_builder_default_impl() {
        let builder: UniversalConfigBuilder = Default::default();
        let config = builder.build();

        // Should behave the same as new()
        assert!(config.auto_discovery_enabled);
    }

    #[test]
    fn test_multiple_primal_instances() {
        let instance1 = PrimalInstanceConfig::new(
            "http://localhost:8080".to_string(),
            "instance-1".to_string(),
            "user-1".to_string(),
            "device-1".to_string(),
        );

        let instance2 = PrimalInstanceConfig::new(
            "http://localhost:8081".to_string(),
            "instance-2".to_string(),
            "user-2".to_string(),
            "device-2".to_string(),
        );

        let config = UniversalConfigBuilder::new()
            .add_primal_instance("instance-1".to_string(), instance1)
            .add_primal_instance("instance-2".to_string(), instance2)
            .build();

        assert_eq!(config.primal_instances.len(), 2);
        assert!(config.primal_instances.contains_key("instance-1"));
        assert!(config.primal_instances.contains_key("instance-2"));
    }

    #[test]
    fn test_comprehensive_builder_workflow() {
        // Simulate a realistic configuration workflow
        let instance = PrimalInstanceConfig::new(
            "https://secure.api.example.com".to_string(),
            "prod-squirrel-1".to_string(),
            "user-enterprise-001".to_string(),
            "device-workstation-042".to_string(),
        );

        let config = UniversalConfigBuilder::new()
            .auto_discovery(true)
            .max_instances_per_type(50)
            .max_instances_per_user(25)
            .load_balancing_strategy(LoadBalancingStrategy::Random)
            .metrics_enabled(true)
            .tracing_level("warn".to_string())
            .port_range(10000, 11000)
            .add_primal_instance("prod-squirrel-1".to_string(), instance)
            .build();

        // Verify all settings
        assert!(config.auto_discovery_enabled);
        assert_eq!(config.multi_instance.max_instances_per_type, 50);
        assert_eq!(config.multi_instance.max_instances_per_user, 25);
        assert_eq!(
            config.multi_instance.load_balancing_strategy,
            LoadBalancingStrategy::Random
        );
        assert!(config.monitoring.metrics_enabled);
        assert_eq!(config.monitoring.tracing.level, "warn");
        assert_eq!(config.port_management.port_range.start, 10000);
        assert_eq!(config.port_management.port_range.end, 11000);
        assert_eq!(config.primal_instances.len(), 1);
    }
}
