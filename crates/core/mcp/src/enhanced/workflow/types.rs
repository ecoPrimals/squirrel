// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Workflow Management Types
//!
//! This module contains all the type definitions for the workflow management system.

use std::collections::HashMap;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

// Import canonical config types
use crate::config::{EncryptionConfig, ScalingConfig};

/// Workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    /// Workflow ID
    pub id: String,
    
    /// Workflow name
    pub name: String,
    
    /// Workflow description
    pub description: String,
    
    /// Workflow version
    pub version: String,
    
    /// Workflow steps
    pub steps: Vec<WorkflowStep>,
    
    /// Workflow configuration
    pub config: WorkflowConfig,
    
    /// Workflow metadata
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Workflow parameters
    pub parameters: Vec<WorkflowParameter>,
    
    /// Workflow outputs
    pub outputs: Vec<WorkflowOutput>,
    
    /// Workflow triggers
    pub triggers: Vec<WorkflowTrigger>,
    
    /// Workflow dependencies
    pub dependencies: Vec<WorkflowDependency>,
    
    /// Workflow constraints
    pub constraints: Vec<WorkflowConstraint>,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last modified timestamp
    pub modified_at: DateTime<Utc>,
}

/// Workflow step definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// Step ID
    pub id: String,
    
    /// Step name
    pub name: String,
    
    /// Step description
    pub description: String,
    
    /// Step type
    pub step_type: WorkflowStepType,
    
    /// Step configuration
    pub config: serde_json::Value,
    
    /// Step dependencies
    pub dependencies: Vec<String>,
    
    /// Step conditions
    pub conditions: Vec<StepCondition>,
    
    /// Step timeout
    pub timeout: Duration,
    
    /// Step retry configuration
    pub retry: RetryConfig,
    
    /// Step resources
    pub resources: ResourceRequirements,
    
    /// Step metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Workflow step types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkflowStepType {
    /// AI inference step
    AIInference,
    
    /// Service composition step
    ServiceComposition,
    
    /// Data processing step
    DataProcessing,
    
    /// Condition evaluation step
    Condition,
    
    /// Loop step
    Loop,
    
    /// Parallel execution step
    Parallel,
    
    /// Wait step
    Wait,
    
    /// Notification step
    Notification,
    
    /// Webhook step
    Webhook,
    
    /// Custom step
    Custom(String),
}

/// Step condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepCondition {
    /// Condition type
    pub condition_type: ConditionType,
    
    /// Condition expression
    pub expression: String,
    
    /// Condition value
    pub value: serde_json::Value,
    
    /// Condition operator
    pub operator: ConditionOperator,
}

/// Condition types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionType {
    /// Input condition
    Input,
    
    /// Output condition
    Output,
    
    /// State condition
    State,
    
    /// Time condition
    Time,
    
    /// Resource condition
    Resource,
    
    /// Custom condition
    Custom(String),
}

/// Condition operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionOperator {
    /// Equals
    Equals,
    
    /// Not equals
    NotEquals,
    
    /// Greater than
    GreaterThan,
    
    /// Less than
    LessThan,
    
    /// Greater than or equal
    GreaterThanOrEqual,
    
    /// Less than or equal
    LessThanOrEqual,
    
    /// Contains
    Contains,
    
    /// In
    In,
    
    /// Not in
    NotIn,
    
    /// Regular expression
    Regex,
    
    /// Custom operator
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
    pub retry: RetryConfig,
    
    /// Resource limits
    pub resources: ResourceLimits,
    
    /// Monitoring configuration
    pub monitoring: MonitoringConfig,
    
    /// Error handling configuration
    pub error_handling: ErrorHandlingConfig,
    
    /// Security configuration
    pub security: SecurityConfig,
    
    /// Scaling configuration
    pub scaling: ScalingConfig,
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
    
    /// Map-Reduce execution
    MapReduce,
    
    /// Event-driven execution
    EventDriven,
    
    /// Custom execution
    Custom(String),
}

/// Workflow parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowParameter {
    /// Parameter name
    pub name: String,
    
    /// Parameter type
    pub parameter_type: ParameterType,
    
    /// Parameter description
    pub description: String,
    
    /// Default value
    pub default_value: Option<serde_json::Value>,
    
    /// Required flag
    pub required: bool,
    
    /// Parameter constraints
    pub constraints: Vec<ParameterConstraint>,
}

/// Parameter types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    /// String parameter
    String,
    
    /// Integer parameter
    Integer,
    
    /// Float parameter
    Float,
    
    /// Boolean parameter
    Boolean,
    
    /// Array parameter
    Array,
    
    /// Object parameter
    Object,
    
    /// Custom parameter
    Custom(String),
}

/// Parameter constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterConstraint {
    /// Constraint type
    pub constraint_type: ConstraintType,
    
    /// Constraint value
    pub value: serde_json::Value,
    
    /// Constraint description
    pub description: String,
}

/// Constraint types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    /// Minimum value
    MinValue,
    
    /// Maximum value
    MaxValue,
    
    /// Minimum length
    MinLength,
    
    /// Maximum length
    MaxLength,
    
    /// Pattern constraint
    Pattern,
    
    /// Enum constraint
    Enum,
    
    /// Custom constraint
    Custom(String),
}

/// Workflow output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowOutput {
    /// Output name
    pub name: String,
    
    /// Output type
    pub output_type: OutputType,
    
    /// Output description
    pub description: String,
    
    /// Output source
    pub source: String,
    
    /// Output transformation
    pub transformation: Option<String>,
}

/// Output types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputType {
    /// Raw output
    Raw,
    
    /// Processed output
    Processed,
    
    /// Aggregated output
    Aggregated,
    
    /// Filtered output
    Filtered,
    
    /// Transformed output
    Transformed,
    
    /// Custom output
    Custom(String),
}

/// Workflow trigger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTrigger {
    /// Trigger type
    pub trigger_type: TriggerType,
    
    /// Trigger configuration
    pub config: serde_json::Value,
    
    /// Trigger condition
    pub condition: Option<String>,
    
    /// Trigger metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Trigger types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerType {
    /// Manual trigger
    Manual,
    
    /// Schedule trigger
    Schedule,
    
    /// Event trigger
    Event,
    
    /// Webhook trigger
    Webhook,
    
    /// File trigger
    File,
    
    /// Database trigger
    Database,
    
    /// Custom trigger
    Custom(String),
}

/// Workflow dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDependency {
    /// Dependency type
    pub dependency_type: DependencyType,
    
    /// Dependency target
    pub target: String,
    
    /// Dependency condition
    pub condition: Option<String>,
    
    /// Dependency metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Dependency types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    /// Workflow dependency
    Workflow,
    
    /// Service dependency
    Service,
    
    /// Resource dependency
    Resource,
    
    /// Data dependency
    Data,
    
    /// Custom dependency
    Custom(String),
}

/// Workflow constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConstraint {
    /// Constraint type
    pub constraint_type: String,
    
    /// Constraint value
    pub value: serde_json::Value,
    
    /// Constraint description
    pub description: String,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum retry attempts
    pub max_attempts: u32,
    
    /// Retry delay
    pub delay: Duration,
    
    /// Backoff strategy
    pub backoff_strategy: BackoffStrategy,
    
    /// Retry conditions
    pub conditions: Vec<RetryCondition>,
}

/// Backoff strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    /// Fixed delay
    Fixed,
    
    /// Exponential backoff
    Exponential,
    
    /// Linear backoff
    Linear,
    
    /// Custom backoff
    Custom(String),
}

/// Retry condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryCondition {
    /// Condition type
    pub condition_type: String,
    
    /// Condition value
    pub value: serde_json::Value,
}

/// Resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// CPU requirements
    pub cpu: f64,
    
    /// Memory requirements
    pub memory: u64,
    
    /// Storage requirements
    pub storage: u64,
    
    /// Network requirements
    pub network: u64,
    
    /// Custom requirements
    pub custom: HashMap<String, serde_json::Value>,
}

/// Resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum CPU
    pub max_cpu: f64,
    
    /// Maximum memory
    pub max_memory: u64,
    
    /// Maximum storage
    pub max_storage: u64,
    
    /// Maximum network
    pub max_network: u64,
    
    /// Custom limits
    pub custom: HashMap<String, serde_json::Value>,
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

/// Error handling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorHandlingConfig {
    /// Error handling strategy
    pub strategy: ErrorHandlingStrategy,
    
    /// Error recovery actions
    pub recovery_actions: Vec<String>,
    
    /// Error notification settings
    pub notifications: Vec<String>,
}

/// Error handling strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorHandlingStrategy {
    /// Fail fast
    FailFast,
    
    /// Retry
    Retry,
    
    /// Fallback
    Fallback,
    
    /// Ignore
    Ignore,
    
    /// Custom strategy
    Custom(String),
}

// SecurityConfig and AccessControlConfig now imported from crate::config::ServiceSecurityConfig
pub use crate::config::{ServiceSecurityConfig as SecurityConfig, AccessControlConfig};

// EncryptionConfig and ScalingConfig are now imported from crate::config

/// Workflow instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInstance {
    /// Instance ID
    pub id: String,
    
    /// Workflow ID
    pub workflow_id: String,
    
    /// Instance state
    pub state: WorkflowState,
    
    /// Instance parameters
    pub parameters: HashMap<String, serde_json::Value>,
    
    /// Instance outputs
    pub outputs: HashMap<String, serde_json::Value>,
    
    /// Instance metadata
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    
    /// Started timestamp
    pub started_at: Option<DateTime<Utc>>,
    
    /// Completed timestamp
    pub completed_at: Option<DateTime<Utc>>,
    
    /// Step states
    pub step_states: HashMap<String, StepState>,
}

/// Workflow state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkflowState {
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
    
    /// Paused
    Paused,
}

/// Step state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepState {
    /// Pending
    Pending,
    
    /// Running
    Running,
    
    /// Completed
    Completed,
    
    /// Failed
    Failed,
    
    /// Skipped
    Skipped,
}

/// Workflow metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetrics {
    /// Total workflows
    pub total_workflows: u64,
    
    /// Active workflows
    pub active_workflows: u64,
    
    /// Completed workflows
    pub completed_workflows: u64,
    
    /// Failed workflows
    pub failed_workflows: u64,
    
    /// Average execution time
    pub avg_execution_time: Duration,
    
    /// Success rate
    pub success_rate: f64,
}

/// Workflow management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowManagementConfig {
    /// Maximum concurrent workflows
    pub max_concurrent_workflows: u32,
    
    /// Default timeout
    pub default_timeout: Duration,
    
    /// Metrics collection interval
    pub metrics_interval: Duration,
    
    /// Cleanup interval
    pub cleanup_interval: Duration,
    
    /// Storage configuration
    pub storage: StorageConfig,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Storage type
    pub storage_type: String,
    
    /// Storage connection string
    pub connection_string: String,
    
    /// Storage configuration
    pub config: HashMap<String, serde_json::Value>,
} 