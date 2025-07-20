//! Recovery system types and data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Active recovery operation tracking
#[derive(Debug, Clone)]
pub struct ActiveRecovery {
    /// Recovery operation ID
    pub recovery_id: String,
    /// Experiment ID being recovered
    pub experiment_id: String,
    /// Recovery configuration
    pub config: RecoveryConfig,
    /// Current recovery status
    pub status: RecoveryStatus,
    /// Start time of recovery
    pub start_time: Instant,
    /// Recovery steps executed
    pub executed_steps: Vec<RecoveryStep>,
    /// Validation results
    pub validation_results: Vec<ValidationResult>,
    /// Recovery metrics
    pub metrics: RecoveryMetrics,
}

/// Recovery operation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryStatus {
    /// Recovery is starting
    Starting,
    /// Executing recovery strategies
    ExecutingStrategies,
    /// Validating system health
    Validating,
    /// Recovery completed successfully
    Completed,
    /// Recovery failed
    Failed { reason: String },
    /// Recovery timed out
    TimedOut,
}

/// Recovery strategies that can be executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    /// Restart affected services
    RestartServices,
    /// Clear caches to reset state
    ClearCaches,
    /// Reset network connections
    ResetConnections,
    /// Garbage collection and memory cleanup
    MemoryCleanup,
    /// Restart database connections
    RestartDbConnections,
    /// Reset thread pools
    ResetThreadPools,
    /// Rollback configuration changes
    RollbackConfig,
    /// Custom recovery script
    CustomScript {
        script_path: String,
        args: Vec<String>,
    },
    /// Wait for natural recovery
    WaitForRecovery { duration: Duration },
    /// Scale out resources
    ScaleOut { resource: String, factor: f64 },
    /// Apply circuit breaker reset
    ResetCircuitBreakers,
}

/// Validation steps to verify recovery success
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationStep {
    /// Check system health endpoints
    HealthCheck {
        endpoint: String,
        expected_status: u16,
    },
    /// Verify database connectivity
    DatabaseConnectivity { connection_string: String },
    /// Test API endpoints
    ApiConnectivity { endpoints: Vec<String> },
    /// Check memory usage is within bounds
    MemoryUsage { max_usage_percent: f64 },
    /// Verify CPU usage is normal
    CpuUsage { max_usage_percent: f64 },
    /// Check error rates are acceptable
    ErrorRateCheck {
        max_error_rate: f64,
        duration: Duration,
    },
    /// Validate response times
    ResponseTimeCheck { max_response_time_ms: u64 },
    /// Custom validation script
    CustomValidation {
        script_path: String,
        expected_exit_code: i32,
    },
    /// Check service availability
    ServiceAvailability { service_names: Vec<String> },
    /// Validate data consistency
    DataConsistency { queries: Vec<String> },
}

/// Recovery step execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStep {
    /// Step identifier
    pub step_id: String,
    /// Strategy being executed
    pub strategy: RecoveryStrategy,
    /// Step execution status
    pub status: StepStatus,
    /// Start time
    pub start_time: std::time::SystemTime,
    /// End time (if completed)
    pub end_time: Option<std::time::SystemTime>,
    /// Duration of execution
    pub duration: Option<Duration>,
    /// Step output/logs
    pub output: String,
    /// Error message if failed
    pub error: Option<String>,
}

/// Step execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepStatus {
    /// Step is pending execution
    Pending,
    /// Step is currently executing
    Running,
    /// Step completed successfully
    Completed,
    /// Step failed
    Failed,
    /// Step was skipped
    Skipped,
    /// Step timed out
    TimedOut,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Validation step that was performed
    pub step: ValidationStep,
    /// Whether the validation passed
    pub passed: bool,
    /// Result message
    pub message: String,
    /// Duration of validation
    pub duration: Duration,
    /// Timestamp of validation
    pub timestamp: std::time::SystemTime,
    /// Detailed metrics if available
    pub metrics: Option<HashMap<String, serde_json::Value>>,
}

/// Recovery metrics for tracking performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryMetrics {
    /// Total time taken for recovery
    pub total_recovery_time: Option<Duration>,
    /// Number of recovery strategies attempted
    pub strategies_attempted: u32,
    /// Number of strategies that succeeded
    pub strategies_succeeded: u32,
    /// Number of validations performed
    pub validations_performed: u32,
    /// Number of validations that passed
    pub validations_passed: u32,
    /// Health score after recovery (0.0 to 1.0)
    pub health_score_after: Option<f64>,
    /// Custom metrics collected during recovery
    pub custom_metrics: HashMap<String, serde_json::Value>,
}

/// Configuration for recovery operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryConfig {
    /// Maximum time to spend on recovery
    pub timeout: Duration,
    /// Recovery strategies to attempt
    pub strategies: Vec<RecoveryStrategy>,
    /// Validation steps to perform
    pub validation_steps: Vec<ValidationStep>,
    /// Whether to stop on first validation failure
    pub fail_fast: bool,
    /// Number of retry attempts for failed strategies
    pub retry_attempts: u32,
    /// Delay between retry attempts
    pub retry_delay: Duration,
    /// Whether to run strategies in parallel
    pub parallel_execution: bool,
    /// Custom recovery parameters
    pub custom_params: HashMap<String, serde_json::Value>,
}

/// Completed recovery metrics for historical analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedRecoveryMetrics {
    /// Recovery operation ID
    pub recovery_id: String,
    /// Experiment ID that was recovered
    pub experiment_id: String,
    /// Total recovery duration
    pub recovery_duration: Duration,
    /// Strategies that were used
    pub strategies_used: Vec<RecoveryStrategy>,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Health improvement score
    pub health_improvement: f64,
    /// Timestamp of recovery completion
    pub timestamp: std::time::SystemTime,
}

/// HTTP response for validation checks
#[derive(Debug)]
pub struct HttpResponse {
    /// HTTP status code
    pub status: u16,
    /// Response body
    pub body: String,
}

impl RecoveryMetrics {
    /// Create new recovery metrics instance
    pub fn new() -> Self {
        Self {
            total_recovery_time: None,
            strategies_attempted: 0,
            strategies_succeeded: 0,
            validations_performed: 0,
            validations_passed: 0,
            health_score_after: None,
            custom_metrics: HashMap::new(),
        }
    }
}

impl Default for RecoveryMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(300), // 5 minutes default
            strategies: vec![],
            validation_steps: vec![],
            fail_fast: false,
            retry_attempts: 3,
            retry_delay: Duration::from_secs(10),
            parallel_execution: false,
            custom_params: HashMap::new(),
        }
    }
}
