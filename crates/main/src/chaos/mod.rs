//! # Chaos Engineering Framework
//!
//! Comprehensive fault injection and resilience testing framework for the Squirrel AI Primal.
//! This module provides tools to systematically test system behavior under various failure conditions.
//!
//! ## Features
//!
//! - **Fault Injection**: Network, resource, and service failures
//! - **Resilience Testing**: Circuit breaker, retry, and timeout patterns
//! - **Load Testing**: High-throughput scenario validation
//! - **Failure Recovery**: Automatic recovery and degradation testing
//! - **Observability**: Real-time monitoring during chaos experiments
//!
//! ## Usage
//!
//! ```rust
//! use squirrel::chaos::{ChaosEngineer, FaultType, ExperimentConfig};
//!
//! let chaos = ChaosEngineer::new();
//!
//! // Test network failures
//! let experiment = ExperimentConfig::new()
//!     .with_fault(FaultType::NetworkFailure { rate: 0.1 })
//!     .with_duration(Duration::from_secs(30))
//!     .with_target("ai_provider");
//!
//! let results = chaos.run_experiment(experiment).await?;
//! ```

pub mod fault_injection;
pub mod monitoring;
pub mod recovery; // Now a module instead of a single file

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

// Re-export from recovery module
pub use recovery::{
    ActiveRecovery, CompletedRecoveryMetrics, RecoveryConfig, RecoveryMetrics,
    RecoveryOrchestrator, RecoveryStatus, RecoveryStep, RecoveryStrategy, StepStatus,
    ValidationResult, ValidationStep,
};

/// Comprehensive chaos engineering orchestrator
#[derive(Debug)]
pub struct ChaosEngineer {
    /// Active experiments being executed
    active_experiments: Arc<RwLock<HashMap<String, ActiveExperiment>>>,
    /// Fault injection mechanisms
    fault_injector: Arc<fault_injection::FaultInjector>,
    /// System monitoring during experiments
    monitor: Arc<monitoring::ChaosMonitor>,
    /// Recovery orchestrator
    recovery: Arc<RecoveryOrchestrator>,
    /// Experiment results storage
    results: Arc<Mutex<Vec<ExperimentResult>>>,
}

/// Types of faults that can be injected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FaultType {
    /// Network-related failures
    NetworkFailure {
        /// Type of network error to inject
        error_type: NetworkErrorType,
        /// Failure rate (0.0 to 1.0)
        rate: f64,
    },
    /// Resource exhaustion
    ResourceExhaustion {
        /// Type of resource to exhaust
        resource: ResourceType,
        /// Level of exhaustion (0.0 to 1.0)
        level: f64,
    },
    /// Service unavailability
    ServiceUnavailable {
        /// Service name to make unavailable
        service_name: String,
        /// Custom error response
        error_response: Option<String>,
    },
    /// Memory pressure
    MemoryPressure {
        /// Amount of memory to consume (in MB)
        memory_mb: u64,
        /// Duration to maintain pressure
        duration: Duration,
    },
    /// CPU starvation
    CpuStarvation {
        /// CPU percentage to consume (0.0 to 1.0)
        cpu_percentage: f64,
        /// Number of threads to spawn
        threads: usize,
        /// Duration to maintain load
        duration: Duration,
    },
    /// Disk I/O failures
    DiskIoFailure {
        /// Path to affect
        path: String,
        /// Failure rate (0.0 to 1.0)
        failure_rate: f64,
    },
}

/// Network error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkErrorType {
    /// Connection timeout
    Timeout,
    /// Connection refused
    ConnectionRefused,
    /// DNS resolution failure
    DnsFailure,
    /// SSL/TLS handshake failure
    SslFailure,
    /// HTTP 500 error
    Http500,
    /// HTTP 503 service unavailable
    Http503,
    /// Network packet loss
    PacketLoss { rate: f64 },
    /// Network latency injection
    Latency { delay_ms: u64 },
}

/// Resource types for exhaustion testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    /// Memory exhaustion
    Memory,
    /// CPU exhaustion
    Cpu,
    /// Disk space exhaustion
    DiskSpace,
    /// File descriptor exhaustion
    FileDescriptors,
    /// Network connection exhaustion
    NetworkConnections,
}

/// Experiment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    /// Unique experiment identifier
    pub id: String,
    /// Human-readable experiment name
    pub name: String,
    /// Description of what the experiment tests
    pub description: String,
    /// Target system or component
    pub target: String,
    /// Fault to inject
    pub fault: FaultType,
    /// Duration of the experiment
    pub duration: Duration,
    /// Recovery configuration
    pub recovery_config: Option<RecoveryConfig>,
    /// Whether to automatically recover after the experiment
    pub auto_recovery: bool,
    /// Tags for categorization
    pub tags: Vec<String>,
}

/// Active experiment tracking
#[derive(Debug, Clone)]
pub struct ActiveExperiment {
    /// Experiment configuration
    pub config: ExperimentConfig,
    /// Start time
    pub start_time: Instant,
    /// Current status
    pub status: ExperimentStatus,
    /// Fault injection handle
    pub fault_handle: Option<String>,
    /// Recovery operation handle
    pub recovery_handle: Option<String>,
    /// Collected metrics
    pub metrics: ExperimentMetrics,
}

/// Experiment execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperimentStatus {
    /// Experiment is being prepared
    Preparing,
    /// Fault is being injected
    InjectingFault,
    /// Experiment is running
    Running,
    /// Recovery is in progress
    Recovering,
    /// Experiment completed successfully
    Completed,
    /// Experiment failed
    Failed { reason: String },
    /// Experiment was stopped
    Stopped,
}

/// Experiment metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentMetrics {
    /// System performance metrics during experiment
    pub system_metrics: Vec<SystemSnapshot>,
    /// Application-specific metrics
    pub application_metrics: HashMap<String, Vec<MetricValue>>,
    /// Error counts and rates
    pub error_metrics: ErrorMetrics,
    /// Recovery effectiveness metrics
    pub recovery_metrics: Option<RecoveryMetrics>,
}

/// System performance snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    /// Timestamp of the snapshot
    pub timestamp: std::time::SystemTime,
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Network I/O statistics
    pub network_io: NetworkIoStats,
    /// Disk I/O statistics
    pub disk_io: DiskIoStats,
}

/// Network I/O statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIoStats {
    /// Bytes sent
    pub bytes_sent: u64,
    /// Bytes received
    pub bytes_received: u64,
    /// Packets sent
    pub packets_sent: u64,
    /// Packets received
    pub packets_received: u64,
    /// Connection count
    pub connections: u32,
}

/// Disk I/O statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskIoStats {
    /// Bytes read
    pub bytes_read: u64,
    /// Bytes written
    pub bytes_written: u64,
    /// Read operations
    pub read_ops: u64,
    /// Write operations
    pub write_ops: u64,
    /// I/O wait time
    pub io_wait_ms: u64,
}

/// Application metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    /// Timestamp of the metric
    pub timestamp: std::time::SystemTime,
    /// Metric value
    pub value: f64,
    /// Optional labels/tags
    pub labels: HashMap<String, String>,
}

/// Error metrics tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    /// Total error count
    pub total_errors: u64,
    /// Errors by type
    pub errors_by_type: HashMap<String, u64>,
    /// Error rate (errors per second)
    pub error_rate: f64,
    /// Time to first error
    pub time_to_first_error: Option<Duration>,
}

/// Experiment execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResult {
    /// Experiment that was executed
    pub experiment: ExperimentConfig,
    /// Final status
    pub final_status: ExperimentStatus,
    /// Total execution time
    pub execution_time: Duration,
    /// Collected metrics
    pub metrics: ExperimentMetrics,
    /// Recovery results if recovery was performed
    pub recovery_results: Option<Vec<CompletedRecoveryMetrics>>,
    /// Lessons learned or observations
    pub observations: Vec<String>,
    /// Completion timestamp
    pub completed_at: std::time::SystemTime,
}

/// Chaos experiment error types
#[derive(Debug, thiserror::Error)]
pub enum ChaosError {
    /// Fault injection failed
    #[error("Fault injection failed: {0}")]
    FaultInjectionError(String),
    /// Recovery operation failed
    #[error("Recovery failed: {0}")]
    RecoveryError(String),
    /// Experiment validation failed
    #[error("Experiment validation failed: {0}")]
    ValidationError(String),
    /// Monitoring system error
    #[error("Monitoring error: {0}")]
    MonitoringError(String),
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),
    /// Resource not available
    #[error("Resource not available: {0}")]
    ResourceUnavailable(String),
    /// Operation timeout
    #[error("Operation timed out: {0}")]
    Timeout(String),
    /// Generic chaos engineering error
    #[error("Chaos engineering error: {0}")]
    Other(String),
}

impl ChaosEngineer {
    /// Create a new chaos engineer
    pub fn new() -> Self {
        Self {
            active_experiments: Arc::new(RwLock::new(HashMap::new())),
            fault_injector: Arc::new(fault_injection::FaultInjector::new()),
            monitor: Arc::new(monitoring::ChaosMonitor::new()),
            recovery: Arc::new(RecoveryOrchestrator::new()),
            results: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Run a chaos experiment
    pub async fn run_experiment(
        &self,
        config: ExperimentConfig,
    ) -> Result<ExperimentResult, ChaosError> {
        let experiment_id = config.id.clone();

        // Create active experiment
        let active_experiment = ActiveExperiment {
            config: config.clone(),
            start_time: Instant::now(),
            status: ExperimentStatus::Preparing,
            fault_handle: None,
            recovery_handle: None,
            metrics: ExperimentMetrics {
                system_metrics: Vec::new(),
                application_metrics: HashMap::new(),
                error_metrics: ErrorMetrics {
                    total_errors: 0,
                    errors_by_type: HashMap::new(),
                    error_rate: 0.0,
                    time_to_first_error: None,
                },
                recovery_metrics: None,
            },
        };

        // Store active experiment
        {
            let mut experiments = self.active_experiments.write().await;
            experiments.insert(experiment_id.clone(), active_experiment);
        }

        // Execute experiment
        let result = self.execute_experiment(&experiment_id).await;

        // Clean up active experiment
        {
            let mut experiments = self.active_experiments.write().await;
            experiments.remove(&experiment_id);
        }

        result
    }

    /// Execute the experiment
    async fn execute_experiment(
        &self,
        experiment_id: &str,
    ) -> Result<ExperimentResult, ChaosError> {
        // TODO: Implement complete experiment execution logic
        // This is a simplified implementation for now

        let config = {
            let experiments = self.active_experiments.read().await;
            experiments.get(experiment_id).map(|e| e.config.clone())
        };

        let config = config
            .ok_or_else(|| ChaosError::Other(format!("Experiment not found: {}", experiment_id)))?;

        // Create a basic result for now
        Ok(ExperimentResult {
            experiment: config,
            final_status: ExperimentStatus::Completed,
            execution_time: Duration::from_secs(1),
            metrics: ExperimentMetrics {
                system_metrics: Vec::new(),
                application_metrics: HashMap::new(),
                error_metrics: ErrorMetrics {
                    total_errors: 0,
                    errors_by_type: HashMap::new(),
                    error_rate: 0.0,
                    time_to_first_error: None,
                },
                recovery_metrics: None,
            },
            recovery_results: None,
            observations: Vec::new(),
            completed_at: std::time::SystemTime::now(),
        })
    }

    /// Get active experiments
    pub async fn get_active_experiments(&self) -> Vec<ActiveExperiment> {
        let experiments = self.active_experiments.read().await;
        experiments.values().cloned().collect()
    }

    /// Stop an active experiment
    pub async fn stop_experiment(&self, experiment_id: &str) -> Result<(), ChaosError> {
        let mut experiments = self.active_experiments.write().await;
        if let Some(experiment) = experiments.get_mut(experiment_id) {
            experiment.status = ExperimentStatus::Stopped;
            Ok(())
        } else {
            Err(ChaosError::Other(format!(
                "Experiment not found: {}",
                experiment_id
            )))
        }
    }

    /// Get experiment results
    pub async fn get_results(&self) -> Vec<ExperimentResult> {
        self.results.lock().await.clone()
    }
}

impl Default for ChaosEngineer {
    fn default() -> Self {
        Self::new()
    }
}
