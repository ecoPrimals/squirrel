// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Type definitions for the Universal Executor
//!
//! Execution requests, results, security contexts, resource limits,
//! and sandbox configuration used by [`super::universal_executor`].

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::Platform;

/// Universal execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    /// Unique execution identifier
    pub id: Uuid,
    /// Target platform for execution
    pub platform: Platform,
    /// Code or command to execute
    pub code: String,
    /// Programming language or execution type
    pub language: String,
    /// Input parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Security context
    pub security_context: SecurityContext,
    /// Resource limits
    pub resource_limits: ResourceLimits,
    /// Timeout in seconds
    pub timeout_seconds: u64,
}

/// Security context for execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// User identifier
    pub user_id: String,
    /// Permission level
    pub permission_level: PermissionLevel,
    /// Allowed system operations
    pub allowed_operations: Vec<String>,
    /// Sandbox configuration
    pub sandbox_config: SandboxConfig,
}

/// Permission levels for execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionLevel {
    /// Full system access
    Administrator,
    /// Standard user permissions
    User,
    /// Restricted sandbox environment
    Sandbox,
    /// Read-only access
    ReadOnly,
}

/// Sandbox configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Enable network access
    pub network_access: bool,
    /// Enable file system access
    pub filesystem_access: bool,
    /// Allowed file paths
    pub allowed_paths: Vec<String>,
    /// Maximum memory usage in MB
    pub max_memory_mb: u64,
    /// Maximum CPU usage percentage
    pub max_cpu_percent: f64,
}

/// Resource limits for execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in bytes
    pub max_memory_bytes: u64,
    /// Maximum CPU time in seconds
    pub max_cpu_seconds: u64,
    /// Maximum execution time in seconds
    pub max_execution_seconds: u64,
    /// Maximum number of processes
    pub max_processes: u32,
    /// Maximum file descriptors
    pub max_file_descriptors: u32,
}

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Execution identifier
    pub id: Uuid,
    /// Success status
    pub success: bool,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Exit code
    pub exit_code: Option<i32>,
    /// Resource usage statistics
    pub resource_usage: ResourceUsage,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Error message if execution failed
    pub error: Option<String>,
}

/// Resource usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Memory usage in bytes
    pub memory_bytes: u64,
    /// CPU time in seconds
    pub cpu_seconds: f64,
    /// Number of processes created
    pub processes_created: u32,
    /// File descriptors used
    pub file_descriptors_used: u32,
}

/// Execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    /// Execution is queued
    Queued,
    /// Execution is running
    Running,
    /// Execution completed successfully
    Completed(ExecutionResult),
    /// Execution failed
    Failed(String),
    /// Execution was cancelled
    Cancelled,
    /// Execution timed out
    TimedOut,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            network_access: false,
            filesystem_access: false,
            allowed_paths: vec![],
            max_memory_mb: 512,
            max_cpu_percent: 50.0,
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 512 * 1024 * 1024,
            max_cpu_seconds: 30,
            max_execution_seconds: 60,
            max_processes: 10,
            max_file_descriptors: 100,
        }
    }
}

