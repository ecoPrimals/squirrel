// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Universal Executor - Cross-Platform Execution Environment
//!
//! This module provides a unified execution environment that can run code
//! across different platforms while maintaining security and isolation.

use super::{FederationError, FederationResult, Platform};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Universal executor trait
#[async_trait]
pub trait UniversalExecutor: Send + Sync {
    /// Execute code on the specified platform
    async fn execute(&self, request: ExecutionRequest) -> FederationResult<ExecutionResult>;

    /// Check if platform is supported
    fn supports_platform(&self, platform: &Platform) -> bool;

    /// Get supported languages for a platform
    fn supported_languages(&self, platform: &Platform) -> Vec<String>;

    /// Validate execution request
    fn validate_request(&self, request: &ExecutionRequest) -> FederationResult<()>;

    /// Cancel execution
    async fn cancel_execution(&self, execution_id: Uuid) -> FederationResult<()>;

    /// Get execution status
    async fn get_execution_status(&self, execution_id: Uuid) -> FederationResult<ExecutionStatus>;
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

/// Default universal executor implementation
pub struct DefaultUniversalExecutor {
    /// Platform-specific executors
    executors: Arc<RwLock<HashMap<Platform, Box<dyn PlatformExecutor>>>>,
    /// Active executions
    active_executions: Arc<RwLock<HashMap<Uuid, ExecutionStatus>>>,
}

impl DefaultUniversalExecutor {
    /// Create a new universal executor
    pub fn new() -> Self {
        Self {
            executors: Arc::new(RwLock::new(HashMap::new())),
            active_executions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a platform executor
    pub async fn register_executor(&self, platform: Platform, executor: Box<dyn PlatformExecutor>) {
        let mut executors = self.executors.write().await;
        executors.insert(platform, executor);
    }

    /// Get platform executor
    async fn get_executor(&self, platform: &Platform) -> Option<Box<dyn PlatformExecutor>> {
        let executors = self.executors.read().await;
        executors.get(platform).cloned()
    }
}

#[async_trait]
impl UniversalExecutor for DefaultUniversalExecutor {
    async fn execute(&self, request: ExecutionRequest) -> FederationResult<ExecutionResult> {
        // Validate request
        self.validate_request(&request)?;

        // Get platform executor
        let executor = self.get_executor(&request.platform).await.ok_or_else(|| {
            FederationError::UnsupportedPlatform(format!("{:?}", request.platform))
        })?;

        // Update execution status
        {
            let mut active = self.active_executions.write().await;
            active.insert(request.id, ExecutionStatus::Running);
        }

        // Execute the request
        let result = executor.execute(request.clone()).await;

        // Update final status
        {
            let mut active = self.active_executions.write().await;
            match &result {
                Ok(execution_result) => {
                    active.insert(
                        request.id,
                        ExecutionStatus::Completed(execution_result.clone()),
                    );
                }
                Err(e) => {
                    active.insert(request.id, ExecutionStatus::Failed(e.to_string()));
                }
            }
        }

        result
    }

    fn supports_platform(&self, platform: &Platform) -> bool {
        // This would check against registered executors
        matches!(
            platform,
            Platform::Linux(_)
                | Platform::Windows(_)
                | Platform::MacOS(_)
                | Platform::WebAssembly
                | Platform::Container(_)
        )
    }

    fn supported_languages(&self, platform: &Platform) -> Vec<String> {
        match platform {
            Platform::Linux(_) => vec![
                "rust".to_string(),
                "python".to_string(),
                "javascript".to_string(),
                "bash".to_string(),
            ],
            Platform::Windows(_) => vec![
                "rust".to_string(),
                "python".to_string(),
                "javascript".to_string(),
                "powershell".to_string(),
            ],
            Platform::WebAssembly => vec![
                "rust".to_string(),
                "javascript".to_string(),
                "assemblyscript".to_string(),
            ],
            Platform::Container(_) => vec![
                "rust".to_string(),
                "python".to_string(),
                "javascript".to_string(),
                "bash".to_string(),
            ],
            _ => vec![],
        }
    }

    fn validate_request(&self, request: &ExecutionRequest) -> FederationResult<()> {
        // Check if platform is supported
        if !self.supports_platform(&request.platform) {
            return Err(FederationError::UnsupportedPlatform(format!(
                "{:?}",
                request.platform
            )));
        }

        // Check if language is supported
        let supported_languages = self.supported_languages(&request.platform);
        if !supported_languages.contains(&request.language) {
            return Err(FederationError::UnsupportedLanguage(
                request.language.clone(),
            ));
        }

        // Validate resource limits
        if request.resource_limits.max_memory_bytes > 16 * 1024 * 1024 * 1024 {
            return Err(FederationError::ResourceLimitExceeded(
                "Memory limit too high".to_string(),
            ));
        }

        if request.timeout_seconds > 3600 {
            return Err(FederationError::ResourceLimitExceeded(
                "Timeout too long".to_string(),
            ));
        }

        Ok(())
    }

    async fn cancel_execution(&self, execution_id: Uuid) -> FederationResult<()> {
        let mut active = self.active_executions.write().await;
        if let Some(status) = active.get_mut(&execution_id) {
            *status = ExecutionStatus::Cancelled;
            Ok(())
        } else {
            Err(FederationError::ExecutionNotFound(execution_id))
        }
    }

    async fn get_execution_status(&self, execution_id: Uuid) -> FederationResult<ExecutionStatus> {
        let active = self.active_executions.read().await;
        active
            .get(&execution_id)
            .cloned()
            .ok_or(FederationError::ExecutionNotFound(execution_id))
    }
}

/// Platform-specific executor trait
#[async_trait]
pub trait PlatformExecutor: Send + Sync {
    /// Execute code on this platform
    async fn execute(&self, request: ExecutionRequest) -> FederationResult<ExecutionResult>;

    /// Get platform information
    fn platform_info(&self) -> Platform;

    /// Check if language is supported
    fn supports_language(&self, language: &str) -> bool;
}

/// Linux executor implementation
pub struct LinuxExecutor {
    /// Sandbox configuration
    sandbox_enabled: bool,
}

impl LinuxExecutor {
    /// Create a new Linux executor
    pub fn new(sandbox_enabled: bool) -> Self {
        Self { sandbox_enabled }
    }
}

#[async_trait]
impl PlatformExecutor for LinuxExecutor {
    async fn execute(&self, request: ExecutionRequest) -> FederationResult<ExecutionResult> {
        let start_time = std::time::Instant::now();

        // Create command based on language
        let mut command = match request.language.as_str() {
            "rust" => {
                // For Rust, we'd need to compile and run
                return Err(FederationError::NotImplemented(
                    "Rust execution not implemented".to_string(),
                ));
            }
            "python" => {
                let mut cmd = tokio::process::Command::new("python3");
                cmd.arg("-c").arg(&request.code);
                cmd
            }
            "javascript" => {
                let mut cmd = tokio::process::Command::new("node");
                cmd.arg("-e").arg(&request.code);
                cmd
            }
            "bash" => {
                let mut cmd = tokio::process::Command::new("bash");
                cmd.arg("-c").arg(&request.code);
                cmd
            }
            _ => return Err(FederationError::UnsupportedLanguage(request.language)),
        };

        // Apply sandbox restrictions if enabled
        if self.sandbox_enabled {
            // This would apply sandboxing restrictions
            // For now, we'll just set some basic limits
            command.kill_on_drop(true);
        }

        // Execute with timeout
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(request.timeout_seconds),
            command.output(),
        )
        .await;

        let duration_ms = start_time.elapsed().as_millis() as u64;

        match output {
            Ok(Ok(output)) => {
                Ok(ExecutionResult {
                    id: request.id,
                    success: output.status.success(),
                    stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    exit_code: output.status.code(),
                    resource_usage: ResourceUsage {
                        memory_bytes: 0, // Would need to measure actual usage
                        cpu_seconds: 0.0,
                        processes_created: 1,
                        file_descriptors_used: 0,
                    },
                    duration_ms,
                    error: None,
                })
            }
            Ok(Err(e)) => Ok(ExecutionResult {
                id: request.id,
                success: false,
                stdout: String::new(),
                stderr: e.to_string(),
                exit_code: None,
                resource_usage: ResourceUsage {
                    memory_bytes: 0,
                    cpu_seconds: 0.0,
                    processes_created: 0,
                    file_descriptors_used: 0,
                },
                duration_ms,
                error: Some(e.to_string()),
            }),
            Err(_) => Ok(ExecutionResult {
                id: request.id,
                success: false,
                stdout: String::new(),
                stderr: "Execution timed out".to_string(),
                exit_code: None,
                resource_usage: ResourceUsage {
                    memory_bytes: 0,
                    cpu_seconds: 0.0,
                    processes_created: 0,
                    file_descriptors_used: 0,
                },
                duration_ms,
                error: Some("Execution timed out".to_string()),
            }),
        }
    }

    fn platform_info(&self) -> Platform {
        Platform::Linux(super::LinuxVariant::Ubuntu)
    }

    fn supports_language(&self, language: &str) -> bool {
        matches!(language, "python" | "javascript" | "bash")
    }
}

impl Clone for Box<dyn PlatformExecutor> {
    fn clone(&self) -> Self {
        // This is a simplified clone - in a real implementation,
        // we'd need to implement proper cloning for each executor type
        Box::new(LinuxExecutor::new(true))
    }
}

impl Default for DefaultUniversalExecutor {
    fn default() -> Self {
        Self::new()
    }
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
            max_memory_bytes: 512 * 1024 * 1024, // 512MB
            max_cpu_seconds: 30,
            max_execution_seconds: 60,
            max_processes: 10,
            max_file_descriptors: 100,
        }
    }
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            memory_bytes: 0,
            cpu_seconds: 0.0,
            processes_created: 0,
            file_descriptors_used: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::federation::LinuxVariant;

    #[tokio::test]
    async fn test_universal_executor_creation() {
        let executor = DefaultUniversalExecutor::new();
        assert!(executor.supports_platform(&Platform::Linux(LinuxVariant::Ubuntu)));
    }

    #[tokio::test]
    async fn test_platform_support() {
        let executor = DefaultUniversalExecutor::new();
        assert!(executor.supports_platform(&Platform::Linux(LinuxVariant::Ubuntu)));
        assert!(executor.supports_platform(&Platform::WebAssembly));
    }

    #[tokio::test]
    async fn test_language_support() {
        let executor = DefaultUniversalExecutor::new();
        let languages = executor.supported_languages(&Platform::Linux(LinuxVariant::Ubuntu));
        assert!(languages.contains(&"rust".to_string()));
        assert!(languages.contains(&"python".to_string()));
    }

    #[tokio::test]
    async fn test_request_validation() {
        let executor = DefaultUniversalExecutor::new();

        let request = ExecutionRequest {
            id: Uuid::new_v4(),
            platform: Platform::Linux(LinuxVariant::Ubuntu),
            code: "print('hello')".to_string(),
            language: "python".to_string(),
            parameters: HashMap::new(),
            security_context: SecurityContext {
                user_id: "test_user".to_string(),
                permission_level: PermissionLevel::Sandbox,
                allowed_operations: vec![],
                sandbox_config: SandboxConfig::default(),
            },
            resource_limits: ResourceLimits::default(),
            timeout_seconds: 30,
        };

        assert!(executor.validate_request(&request).is_ok());
    }
}
