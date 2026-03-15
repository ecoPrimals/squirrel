// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Composition-related types for the service composition system

use std::collections::HashMap;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use super::config::{ResourceLimits, MonitoringConfig, SecurityConfig};

/// Service composition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Composition {
    /// Composition ID
    pub id: String,
    
    /// Composition name
    pub name: String,
    
    /// Composition description
    pub description: String,
    
    /// Composition services
    pub services: Vec<String>,
    
    /// Composition workflow
    pub workflow: CompositionWorkflow,
    
    /// Composition configuration
    pub config: CompositionConfig,
    
    /// Composition state
    pub state: CompositionState,
    
    /// Composition metadata
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    
    /// Started timestamp
    pub started_at: Option<DateTime<Utc>>,
    
    /// Completed timestamp
    pub completed_at: Option<DateTime<Utc>>,
}

/// Composition workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionWorkflow {
    /// Workflow steps
    pub steps: Vec<CompositionStep>,
    
    /// Workflow dependencies
    pub dependencies: Vec<WorkflowDependency>,
    
    /// Workflow configuration
    pub config: WorkflowConfig,
}

/// Composition step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionStep {
    /// Step ID
    pub id: String,
    
    /// Step name
    pub name: String,
    
    /// Step service
    pub service: String,
    
    /// Step configuration
    pub config: serde_json::Value,
    
    /// Step dependencies
    pub dependencies: Vec<String>,
    
    /// Step conditions
    pub conditions: Vec<StepCondition>,
}

/// Step condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepCondition {
    /// Condition type
    pub condition_type: ConditionType,
    
    /// Condition expression
    pub expression: String,
    
    /// Condition parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Condition types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionType {
    /// Simple boolean condition
    Boolean,
    
    /// Value comparison
    Comparison,
    
    /// Regular expression match
    Regex,
    
    /// JSON path query
    JsonPath,
    
    /// Custom condition
    Custom(String),
}

/// Workflow dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDependency {
    /// Dependency ID
    pub id: String,
    
    /// Source step
    pub source_step: String,
    
    /// Target step
    pub target_step: String,
    
    /// Dependency type
    pub dependency_type: WorkflowDependencyType,
    
    /// Dependency metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Workflow dependency types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowDependencyType {
    /// Sequential dependency
    Sequential,
    
    /// Data dependency
    Data,
    
    /// Resource dependency
    Resource,
    
    /// Custom dependency
    Custom(String),
}

/// Workflow configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    /// Execution strategy
    pub execution_strategy: ExecutionStrategy,
    
    /// Timeout configuration
    pub timeout: Duration,
    
    /// Retry configuration
    pub retry: WorkflowRetryConfig,
    
    /// Error handling strategy
    pub error_handling: ErrorHandlingStrategy,
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        // Load unified config for environment-aware timeout values
        let config = squirrel_mcp_config::unified::ConfigLoader::load()
            .ok()
            .and_then(|loaded| loaded.try_into_config().ok());
        
        let timeout = if let Some(cfg) = config {
            cfg.timeouts.get_custom_timeout("workflow_timeout")
                .unwrap_or_else(|| Duration::from_secs(3600))
        } else {
            Duration::from_secs(3600) // 1 hour
        };
        
        Self {
            execution_strategy: ExecutionStrategy::Sequential,
            timeout,
            retry: WorkflowRetryConfig::default(),
            error_handling: ErrorHandlingStrategy::StopOnError,
        }
    }
}

/// Execution strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStrategy {
    /// Sequential execution
    Sequential,
    
    /// Parallel execution
    Parallel,
    
    /// Conditional execution
    Conditional,
    
    /// Pipeline execution
    Pipeline,
    
    /// Custom execution strategy
    Custom(String),
}

/// Workflow retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRetryConfig {
    /// Maximum retry attempts
    pub max_attempts: u32,
    
    /// Retry delay
    pub delay: Duration,
    
    /// Exponential backoff
    pub exponential_backoff: bool,
    
    /// Maximum delay
    pub max_delay: Duration,
    
    /// Retryable errors
    pub retryable_errors: Vec<String>,
}

impl Default for WorkflowRetryConfig {
    fn default() -> Self {
        // Load unified config for environment-aware timeout values
        let config = squirrel_mcp_config::unified::ConfigLoader::load()
            .ok()
            .and_then(|loaded| loaded.try_into_config().ok());
        
        let (delay, max_delay) = if let Some(cfg) = config {
            let d = cfg.timeouts.get_custom_timeout("workflow_retry_delay")
                .unwrap_or_else(|| Duration::from_secs(5));
            let m = cfg.timeouts.get_custom_timeout("workflow_retry_max")
                .unwrap_or_else(|| Duration::from_secs(300));
            (d, m)
        } else {
            (Duration::from_secs(5), Duration::from_secs(300))
        };
        
        Self {
            max_attempts: 3,
            delay,
            exponential_backoff: true,
            max_delay,
            retryable_errors: vec![
                "timeout".to_string(),
                "connection_error".to_string(),
                "service_unavailable".to_string(),
            ],
        }
    }
}

/// Error handling strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorHandlingStrategy {
    /// Stop execution on first error
    StopOnError,
    
    /// Continue execution despite errors
    ContinueOnError,
    
    /// Skip failed steps and continue
    SkipOnError,
    
    /// Rollback on error
    RollbackOnError,
    
    /// Custom error handling
    Custom(String),
}

/// Composition configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionConfig {
    /// Maximum execution time
    pub max_execution_time: Duration,
    
    /// Resource limits
    pub resources: ResourceLimits,
    
    /// Monitoring configuration
    pub monitoring: MonitoringConfig,
    
    /// Security configuration
    pub security: SecurityConfig,
}

impl Default for CompositionConfig {
    fn default() -> Self {
        // Load unified config for environment-aware timeout values
        let config = squirrel_mcp_config::unified::ConfigLoader::load()
            .ok()
            .and_then(|loaded| loaded.try_into_config().ok());
        
        let max_execution_time = if let Some(cfg) = config {
            cfg.timeouts.get_custom_timeout("comp_max_execution")
                .unwrap_or_else(|| Duration::from_secs(600))
        } else {
            Duration::from_secs(600) // 10 minutes
        };
        
        Self {
            max_execution_time,
            resources: ResourceLimits::default(),
            monitoring: MonitoringConfig::default(),
            security: SecurityConfig::default(),
        }
    }
}

/// Composition state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompositionState {
    /// Pending
    Pending,
    
    /// Running
    Running,
    
    /// Completed
    Completed,
    
    /// Failed
    Failed,
    
    /// Cancelled
    Cancelled,
}

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Execution ID
    pub execution_id: String,
    
    /// Composition ID
    pub composition_id: String,
    
    /// Execution status
    pub status: ExecutionStatus,
    
    /// Result data
    pub data: serde_json::Value,
    
    /// Execution time
    pub execution_time: Duration,
    
    /// Error message if any
    pub error: Option<String>,
    
    /// Execution metadata
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Started timestamp
    pub started_at: DateTime<Utc>,
    
    /// Completed timestamp
    pub completed_at: Option<DateTime<Utc>>,
}

/// Execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionStatus {
    /// Execution successful
    Success,
    
    /// Execution failed
    Failed,
    
    /// Execution cancelled
    Cancelled,
    
    /// Execution timeout
    Timeout,
}

/// Composition type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompositionType {
    /// Sequential execution
    Sequential,
    
    /// Parallel execution
    Parallel,
    
    /// Conditional execution
    Conditional,
    
    /// Pipeline execution
    Pipeline,
    
    /// Custom composition type
    Custom(String),
} 