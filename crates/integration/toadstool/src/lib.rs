//! Squirrel-Toadstool Integration
//!
//! This crate provides integration between Squirrel's MCP platform and Toadstool's compute infrastructure.
//! It replaces direct sandbox execution with calls to the Toadstool compute platform.

pub mod client;
pub mod errors;
pub mod execution;
pub mod sandbox;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub use client::ToadstoolClient;
pub use errors::ToadstoolError;
pub use execution::ExecutionRequest;
pub use sandbox::SandboxPolicy;

// Re-export universal error Result type
pub use universal_error::Result as ToadstoolResult;

/// Configuration for Toadstool integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToadstoolConfig {
    /// Toadstool service endpoint
    pub endpoint: String,
    /// Authentication token
    pub auth_token: Option<String>,
    /// Default timeout for requests
    pub timeout: u64,
    /// Enable debug logging
    pub debug: bool,
}

impl Default for ToadstoolConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:9000".to_string(),
            auth_token: None,
            timeout: 30_000, // 30 seconds
            debug: false,
        }
    }
}

/// Execution environment specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEnvironment {
    /// Environment type (wasm, native, container)
    pub environment_type: String,
    /// Resource limits
    pub resource_limits: ResourceLimits,
    /// Security constraints
    pub security_policy: SandboxPolicy,
    /// Environment variables
    pub env_vars: HashMap<String, String>,
}

/// Resource limits for execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory in bytes
    pub max_memory: Option<u64>,
    /// Maximum CPU time in milliseconds
    pub max_cpu_time: Option<u64>,
    /// Maximum disk space in bytes
    pub max_disk_space: Option<u64>,
    /// Maximum network bandwidth in bytes/sec
    pub max_network_bandwidth: Option<u64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory: Some(128 * 1024 * 1024),      // 128MB
            max_cpu_time: Some(30 * 1000),            // 30 seconds
            max_disk_space: Some(100 * 1024 * 1024),  // 100MB
            max_network_bandwidth: Some(1024 * 1024), // 1MB/s
        }
    }
}

/// Plugin execution trait that can be implemented by different compute backends
/// (native async - Phase 4 migration)
pub trait PluginExecutor {
    /// Execute a plugin with the given environment
    fn execute_plugin(
        &self,
        plugin_id: &str,
        code: &[u8],
        environment: ExecutionEnvironment,
    ) -> impl std::future::Future<Output = ToadstoolResult<ExecutionResult>> + Send;

    /// Check execution status
    fn get_execution_status(&self, execution_id: &Uuid) -> impl std::future::Future<Output = ToadstoolResult<ExecutionStatus>> + Send;

    /// Cancel execution
    fn cancel_execution(&self, execution_id: &Uuid) -> impl std::future::Future<Output = ToadstoolResult<()>> + Send;

    /// List active executions
    fn list_executions(&self) -> impl std::future::Future<Output = ToadstoolResult<Vec<ExecutionInfo>>> + Send;
}

/// Result of plugin execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Unique execution ID
    pub execution_id: Uuid,
    /// Exit code
    pub exit_code: i32,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Execution duration in milliseconds
    pub duration: u64,
    /// Resource usage statistics
    pub resource_usage: ResourceUsage,
}

/// Execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Queued,
    Running,
    Completed(ExecutionResult),
    Failed(String),
    Cancelled,
}

/// Execution information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionInfo {
    /// Execution ID
    pub execution_id: Uuid,
    /// Plugin ID
    pub plugin_id: String,
    /// Current status
    pub status: ExecutionStatus,
    /// Start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// End time (if completed)
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// Resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Peak memory usage in bytes
    pub peak_memory: u64,
    /// Total CPU time in milliseconds
    pub cpu_time: u64,
    /// Disk I/O operations
    pub disk_io: u64,
    /// Network bytes transferred
    pub network_bytes: u64,
}

/// Initialize toadstool integration with configuration
pub async fn init_toadstool(config: ToadstoolConfig) -> ToadstoolResult<ToadstoolClient> {
    ToadstoolClient::new(config).await
}
