// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Configuration-related types for the service composition system

use std::collections::HashMap;
use std::time::Duration;
use serde::{Serialize, Deserialize};

// Import canonical config types
use crate::config::EncryptionConfig;

/// Service composition configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCompositionConfig {
    /// Maximum concurrent compositions
    pub max_concurrent_compositions: u32,
    
    /// Default timeout
    pub default_timeout: Duration,
    
    /// Health check interval
    pub health_check_interval: Duration,
    
    /// Metrics collection interval
    pub metrics_interval: Duration,
    
    /// Service discovery configuration
    pub service_discovery: ServiceDiscoveryConfig,
}

impl Default for ServiceCompositionConfig {
    fn default() -> Self {
        Self {
            max_concurrent_compositions: 50,
            default_timeout: Duration::from_secs(300), // 5 minutes
            health_check_interval: Duration::from_secs(30), // 30 seconds
            metrics_interval: Duration::from_secs(60), // 1 minute
            service_discovery: ServiceDiscoveryConfig::default(),
        }
    }
}

/// Service discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscoveryConfig {
    /// Discovery strategy
    pub strategy: DiscoveryStrategy,
    
    /// Discovery interval
    pub interval: Duration,
    
    /// Discovery timeout
    pub timeout: Duration,
    
    /// Discovery endpoints
    pub endpoints: Vec<String>,
}

impl Default for ServiceDiscoveryConfig {
    fn default() -> Self {
        Self {
            strategy: DiscoveryStrategy::Static,
            interval: Duration::from_secs(60), // 1 minute
            timeout: Duration::from_secs(10), // 10 seconds
            endpoints: vec![],
        }
    }
}

/// Discovery strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryStrategy {
    /// Static configuration
    Static,
    
    /// Dynamic discovery
    Dynamic,
    
    /// Hybrid approach
    Hybrid,
    
    /// Custom strategy
    Custom(String),
}

/// Resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage (bytes)
    pub max_memory: u64,
    
    /// Maximum CPU usage (cores)
    pub max_cpu: f64,
    
    /// Maximum execution time
    pub max_execution_time: Duration,
    
    /// Maximum concurrent requests
    pub max_concurrent_requests: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory: 1024 * 1024 * 1024, // 1GB
            max_cpu: 1.0,
            max_execution_time: Duration::from_secs(300), // 5 minutes
            max_concurrent_requests: 10,
        }
    }
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Metrics collection enabled
    pub metrics_enabled: bool,
    
    /// Logging enabled
    pub logging_enabled: bool,
    
    /// Tracing enabled
    pub tracing_enabled: bool,
    
    /// Alert configuration
    pub alerts: Vec<AlertConfig>,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            metrics_enabled: true,
            logging_enabled: true,
            tracing_enabled: true,
            alerts: vec![],
        }
    }
}

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Alert name
    pub name: String,
    
    /// Alert condition
    pub condition: String,
    
    /// Alert threshold
    pub threshold: f64,
    
    /// Alert actions
    pub actions: Vec<String>,
}

// SecurityConfig now imported from crate::config::ServiceSecurityConfig
pub use crate::config::ServiceSecurityConfig as SecurityConfig;

/// Service composition metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCompositionMetrics {
    /// Total compositions
    pub total_compositions: u64,
    
    /// Active compositions
    pub active_compositions: u64,
    
    /// Completed compositions
    pub completed_compositions: u64,
    
    /// Failed compositions
    pub failed_compositions: u64,
    
    /// Average execution time
    pub avg_execution_time: Duration,
    
    /// Success rate
    pub success_rate: f64,
    
    /// Service availability
    pub service_availability: HashMap<String, f64>,
}

impl Default for ServiceCompositionMetrics {
    fn default() -> Self {
        Self {
            total_compositions: 0,
            active_compositions: 0,
            completed_compositions: 0,
            failed_compositions: 0,
            avg_execution_time: Duration::from_secs(0),
            success_rate: 0.0,
            service_availability: HashMap::new(),
        }
    }
} 