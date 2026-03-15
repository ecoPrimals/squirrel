// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Endpoint and instance configuration types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export canonical CircuitBreakerConfig from squirrel-config unified system
pub use squirrel_mcp_config::CircuitBreakerConfig;

use super::HealthCheckConfig;

/// Configuration for a specific primal instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInstanceConfig {
    /// Base URL for the primal service
    pub base_url: String,

    /// Instance identifier
    pub instance_id: String,

    /// User ID this instance serves
    pub user_id: String,

    /// Device ID this instance serves
    pub device_id: String,

    /// Security level for this instance
    pub security_level: String,

    /// API key for authentication
    pub api_key: Option<String>,

    /// Custom headers for requests
    pub headers: HashMap<String, String>,

    /// Maximum request timeout
    pub timeout_seconds: u64,

    /// Connection pool settings
    pub connection_pool: ConnectionPoolConfig,

    /// Health check configuration
    pub health_check: HealthCheckConfig,
}

impl PrimalInstanceConfig {
    /// Create a new primal instance configuration
    pub fn new(base_url: String, instance_id: String, user_id: String, device_id: String) -> Self {
        Self {
            base_url,
            instance_id,
            user_id,
            device_id,
            security_level: "standard".to_string(),
            api_key: None,
            headers: HashMap::new(),
            timeout_seconds: 30,
            connection_pool: ConnectionPoolConfig {
                max_connections: 10,
                min_connections: 1,
                connection_timeout_seconds: 30,
                idle_timeout_seconds: 300,
            },
            health_check: HealthCheckConfig {
                enabled: true,
                interval: 30,
                timeout: 10,
                endpoint: "/health".to_string(),
            },
        }
    }

    /// Set the API key for this instance
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// Set the security level for this instance
    pub fn with_security_level(mut self, level: String) -> Self {
        self.security_level = level;
        self
    }

    /// Add a custom header
    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }
}

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    /// Maximum number of connections
    pub max_connections: usize,

    /// Minimum number of connections
    pub min_connections: usize,

    /// Connection timeout
    pub connection_timeout_seconds: u64,

    /// Idle timeout
    pub idle_timeout_seconds: u64,
}

/// Port range specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortRange {
    /// Start port
    pub start: u16,

    /// End port
    pub end: u16,
}

impl PortRange {
    /// Create a new port range
    pub fn new(start: u16, end: u16) -> Self {
        Self { start, end }
    }

    /// Check if a port is within this range
    pub fn contains(&self, port: u16) -> bool {
        port >= self.start && port <= self.end
    }

    /// Get the size of this port range
    pub fn size(&self) -> usize {
        (self.end - self.start + 1) as usize
    }

    /// Get an iterator over all ports in this range
    pub fn ports(&self) -> impl Iterator<Item = u16> {
        self.start..=self.end
    }
}

/// Port management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortManagementConfig {
    /// Port range for dynamic allocation
    pub port_range: PortRange,

    /// Port lease duration
    pub lease_duration_minutes: u64,

    /// Port allocation strategy
    pub allocation_strategy: PortAllocationStrategy,

    /// Reserved ports that should not be allocated
    pub reserved_ports: Vec<u16>,
}

/// Port allocation strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortAllocationStrategy {
    /// Sequential allocation
    Sequential,

    /// Random allocation
    Random,

    /// Least recently used allocation
    LeastRecentlyUsed,
}

/// Load balancing strategies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LoadBalancingStrategy {
    /// Round-robin load balancing
    RoundRobin,

    /// Least connections load balancing
    LeastConnections,

    /// Random load balancing
    Random,

    /// Weighted load balancing
    Weighted,

    /// Health-based load balancing
    HealthBased,
}

/// Failover configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverConfig {
    /// Whether to enable failover
    pub enabled: bool,

    /// Maximum number of retries
    pub max_retries: u32,

    /// Retry delay
    pub retry_delay_seconds: u64,

    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,
}

/// Scaling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingConfig {
    /// Whether to enable auto-scaling
    pub auto_scaling_enabled: bool,

    /// Minimum number of instances
    pub min_instances: usize,

    /// Maximum number of instances
    pub max_instances: usize,

    /// CPU usage threshold for scaling up
    pub scale_up_cpu_threshold: f64,

    /// CPU usage threshold for scaling down
    pub scale_down_cpu_threshold: f64,

    /// Memory usage threshold for scaling up
    pub scale_up_memory_threshold: f64,

    /// Memory usage threshold for scaling down
    pub scale_down_memory_threshold: f64,
}

/// Health monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMonitoringConfig {
    /// Whether to enable health monitoring
    pub enabled: bool,

    /// Health check interval
    pub check_interval_seconds: u64,

    /// Health check timeout
    pub check_timeout_seconds: u64,

    /// Number of consecutive failures before marking unhealthy
    pub failure_threshold: u32,

    /// Number of consecutive successes before marking healthy
    pub recovery_threshold: u32,
}
