//! Service Composition Types
//!
//! This module contains all the type definitions for the service composition system.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tokio::sync::RwLock;

use super::super::providers::UniversalAIProvider;

/// AI Service representation
#[derive(Debug, Clone)]
pub struct AIService {
    /// Service ID
    pub id: String,
    
    /// Service name
    pub name: String,
    
    /// Service description
    pub description: String,
    
    /// Service configuration
    pub config: ServiceConfig,
    
    /// Service capabilities
    pub capabilities: Vec<ServiceCapability>,
    
    /// Service dependencies
    pub dependencies: Vec<ServiceDependency>,
    
    /// Service health status
    pub health: Arc<RwLock<ServiceHealth>>,
    
    /// Service metadata
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Service provider
    pub provider: Arc<dyn UniversalAIProvider>,
}

/// Service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Service type
    pub service_type: ServiceType,
    
    /// Service endpoint
    pub endpoint: String,
    
    /// Service authentication
    pub auth: Option<ServiceAuth>,
    
    /// Service timeout
    pub timeout: Duration,
    
    /// Service retry settings
    pub retry: RetryConfig,
    
    /// Service resource limits
    pub resources: ResourceLimits,
    
    /// Service scaling configuration
    pub scaling: ScalingConfig,
}

/// Service types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceType {
    /// AI inference service
    Inference,
    
    /// AI training service
    Training,
    
    /// AI preprocessing service
    Preprocessing,
    
    /// AI postprocessing service
    Postprocessing,
    
    /// AI aggregation service
    Aggregation,
    
    /// AI validation service
    Validation,
    
    /// AI monitoring service
    Monitoring,
    
    /// Custom service type
    Custom(String),
}

/// Service authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAuth {
    /// Authentication type
    pub auth_type: AuthType,
    
    /// Authentication credentials
    pub credentials: HashMap<String, String>,
    
    /// Authentication metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Authentication types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthType {
    /// No authentication
    None,
    
    /// API key authentication
    ApiKey,
    
    /// Bearer token authentication
    Bearer,
    
    /// OAuth2 authentication
    OAuth2,
    
    /// Basic authentication
    Basic,
    
    /// Custom authentication
    Custom(String),
}

/// Service capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCapability {
    /// Capability name
    pub name: String,
    
    /// Capability description
    pub description: String,
    
    /// Capability parameters
    pub parameters: serde_json::Value,
    
    /// Capability constraints
    pub constraints: Vec<CapabilityConstraint>,
    
    /// Capability performance metrics
    pub performance: Option<CapabilityPerformance>,
}

/// Capability constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityConstraint {
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
    /// Maximum input size
    MaxInputSize,
    
    /// Maximum output size
    MaxOutputSize,
    
    /// Maximum processing time
    MaxProcessingTime,
    
    /// Required input format
    RequiredInputFormat,
    
    /// Required output format
    RequiredOutputFormat,
    
    /// Minimum quality score
    MinQualityScore,
    
    /// Resource requirements
    ResourceRequirements,
    
    /// Custom constraint
    Custom(String),
}

/// Capability performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityPerformance {
    /// Average latency
    pub avg_latency: Duration,
    
    /// Throughput (requests per second)
    pub throughput: f64,
    
    /// Success rate
    pub success_rate: f64,
    
    /// Quality score
    pub quality_score: f64,
    
    /// Cost per request
    pub cost_per_request: f64,
}

/// Service dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependency {
    /// Dependency ID
    pub id: String,
    
    /// Dependency type
    pub dependency_type: DependencyType,
    
    /// Required service
    pub required_service: String,
    
    /// Required capabilities
    pub required_capabilities: Vec<String>,
    
    /// Dependency constraints
    pub constraints: Vec<DependencyConstraint>,
    
    /// Dependency metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Dependency types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DependencyType {
    /// Hard dependency (must be available)
    Hard,
    
    /// Soft dependency (optional)
    Soft,
    
    /// Sequential dependency (must execute in order)
    Sequential,
    
    /// Parallel dependency (can execute simultaneously)
    Parallel,
    
    /// Conditional dependency (depends on conditions)
    Conditional,
    
    /// Circular dependency (mutual dependency)
    Circular,
}

/// Dependency constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyConstraint {
    /// Constraint name
    pub name: String,
    
    /// Constraint condition
    pub condition: String,
    
    /// Constraint value
    pub value: serde_json::Value,
}

/// Service health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    /// Health status
    pub status: HealthStatus,
    
    /// Health score (0.0 to 1.0)
    pub score: f64,
    
    /// Last health check
    pub last_check: DateTime<Utc>,
    
    /// Health metrics
    pub metrics: HealthMetrics,
    
    /// Health issues
    pub issues: Vec<HealthIssue>,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    /// Healthy
    Healthy,
    
    /// Warning
    Warning,
    
    /// Critical
    Critical,
    
    /// Unknown
    Unknown,
    
    /// Maintenance
    Maintenance,
}

/// Health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    /// CPU usage
    pub cpu_usage: f64,
    
    /// Memory usage
    pub memory_usage: f64,
    
    /// Disk usage
    pub disk_usage: f64,
    
    /// Network latency
    pub network_latency: Duration,
    
    /// Request count
    pub request_count: u64,
    
    /// Error count
    pub error_count: u64,
    
    /// Success rate
    pub success_rate: f64,
}

/// Health issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIssue {
    /// Issue type
    pub issue_type: IssueType,
    
    /// Issue description
    pub description: String,
    
    /// Issue severity
    pub severity: IssueSeverity,
    
    /// Issue timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Issue metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Issue types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueType {
    /// Performance issue
    Performance,
    
    /// Connectivity issue
    Connectivity,
    
    /// Resource issue
    Resource,
    
    /// Configuration issue
    Configuration,
    
    /// Security issue
    Security,
    
    /// Custom issue
    Custom(String),
}

/// Issue severity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueSeverity {
    /// Low severity
    Low,
    
    /// Medium severity
    Medium,
    
    /// High severity
    High,
    
    /// Critical severity
    Critical,
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

/// Scaling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingConfig {
    /// Auto-scaling enabled
    pub auto_scaling: bool,
    
    /// Minimum instances
    pub min_instances: u32,
    
    /// Maximum instances
    pub max_instances: u32,
    
    /// Scaling metrics
    pub metrics: Vec<String>,
}

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
    pub condition_type: String,
    
    /// Condition expression
    pub expression: String,
    
    /// Condition value
    pub value: serde_json::Value,
}

/// Workflow dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDependency {
    /// Dependency ID
    pub id: String,
    
    /// Dependency type
    pub dependency_type: String,
    
    /// Dependency target
    pub target: String,
    
    /// Dependency condition
    pub condition: Option<String>,
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
    
    /// Error handling
    pub error_handling: ErrorHandlingConfig,
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
    
    /// Custom execution
    Custom(String),
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

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Authentication required
    pub auth_required: bool,
    
    /// Authorization rules
    pub authorization: Vec<String>,
    
    /// Encryption settings
    pub encryption: EncryptionConfig,
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Encryption enabled
    pub enabled: bool,
    
    /// Encryption algorithm
    pub algorithm: String,
    
    /// Key management
    pub key_management: String,
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

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Result ID
    pub id: String,
    
    /// Result status
    pub status: ExecutionStatus,
    
    /// Result data
    pub data: serde_json::Value,
    
    /// Result metadata
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Execution time
    pub execution_time: Duration,
    
    /// Error information
    pub error: Option<ExecutionError>,
}

/// Execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionStatus {
    /// Success
    Success,
    
    /// Failure
    Failure,
    
    /// Partial success
    PartialSuccess,
    
    /// Timeout
    Timeout,
    
    /// Cancelled
    Cancelled,
}

/// Execution error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionError {
    /// Error type
    pub error_type: String,
    
    /// Error message
    pub message: String,
    
    /// Error details
    pub details: serde_json::Value,
    
    /// Error timestamp
    pub timestamp: DateTime<Utc>,
} 