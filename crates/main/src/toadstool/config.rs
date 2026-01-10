//! Configuration types for `ToadStool` integration
//!
//! This module defines configuration structures for connecting to and
//! managing the `ToadStool` compute primal.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for `ToadStool` integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToadStoolConfig {
    pub toadstool_endpoint: String,
    pub heartbeat_interval: Duration,
    pub compute_timeout: Duration,
    pub max_retries: u32,
    pub auth_token: Option<String>,
    pub compute_pool_size: u32,
    pub resource_limits: ResourceLimits,
}

impl Default for ToadStoolConfig {
    fn default() -> Self {
        use universal_constants::{builders, network};

        // Use environment-aware endpoint configuration
        let port = network::get_port_from_env("TOADSTOOL_PORT", 9030);
        let endpoint =
            std::env::var("TOADSTOOL_ENDPOINT").unwrap_or_else(|_| builders::localhost_http(port));

        Self {
            toadstool_endpoint: endpoint,
            heartbeat_interval: Duration::from_secs(30),
            compute_timeout: Duration::from_secs(300),
            max_retries: 3,
            auth_token: std::env::var("TOADSTOOL_AUTH_TOKEN").ok(),
            compute_pool_size: 10,
            resource_limits: ResourceLimits::default(),
        }
    }
}

/// Resource limits for compute operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_cpu_cores: u32,
    pub max_memory_gb: u32,
    pub max_gpu_units: u32,
    pub max_concurrent_jobs: u32,
    pub max_job_duration: Duration,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu_cores: 16,
            max_memory_gb: 64,
            max_gpu_units: 4,
            max_concurrent_jobs: 20,
            max_job_duration: Duration::from_secs(3600),
        }
    }
}
