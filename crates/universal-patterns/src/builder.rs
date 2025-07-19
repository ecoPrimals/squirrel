//! Configuration builder for universal patterns
//!
//! This module provides a fluent builder API for creating universal primal configurations.

use crate::config::{LoadBalancingStrategy, PrimalInstanceConfig, UniversalPrimalConfig};
use crate::traits::SecurityLevel;

/// Builder for UniversalPrimalConfig
#[derive(Debug, Default)]
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
