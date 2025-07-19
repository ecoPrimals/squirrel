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

// pub mod experiments;
pub mod fault_injection;
// pub mod resilience_patterns;
pub mod monitoring;
pub mod recovery;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

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
    recovery: Arc<recovery::RecoveryOrchestrator>,
    /// Experiment results storage
    results: Arc<Mutex<Vec<ExperimentResult>>>,
}

/// Types of faults that can be injected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FaultType {
    /// Network-related failures
    NetworkFailure {
        /// Failure rate (0.0 to 1.0)
        rate: f64,
        /// Latency injection in milliseconds
        latency_ms: Option<u64>,
        /// Specific error type to inject
        error_type: NetworkErrorType,
    },
    /// Resource exhaustion scenarios
    ResourceExhaustion {
        /// Resource type to exhaust
        resource: ResourceType,
        /// Exhaustion level (0.0 to 1.0)
        level: f64,
        /// Duration of exhaustion
        duration: Duration,
    },
    /// Service unavailability
    ServiceUnavailable {
        /// Service name to make unavailable
        service_name: String,
        /// Unavailability duration
        duration: Duration,
        /// Error response to return
        error_response: Option<String>,
    },
    /// Memory pressure simulation
    MemoryPressure {
        /// Memory to allocate (in MB)
        allocation_mb: u64,
        /// Duration to hold memory
        duration: Duration,
        /// Gradual pressure increase
        gradual: bool,
    },
    /// CPU starvation
    CpuStarvation {
        /// CPU usage percentage to consume
        cpu_percentage: f64,
        /// Duration of starvation
        duration: Duration,
        /// Number of threads to use
        threads: usize,
    },
    /// Disk I/O failures
    DiskIoFailure {
        /// Failure rate for I/O operations
        failure_rate: f64,
        /// Latency injection for successful operations
        latency_ms: Option<u64>,
        /// Target paths to affect
        target_paths: Vec<String>,
    },
}

/// Network error types for fault injection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkErrorType {
    /// Connection timeout
    Timeout,
    /// Connection refused
    ConnectionRefused,
    /// DNS resolution failure
    DnsFailure,
    /// HTTP 500 errors
    ServerError,
    /// HTTP 503 service unavailable
    ServiceUnavailable,
    /// Partial response (connection dropped mid-transfer)
    PartialResponse,
    /// SSL/TLS handshake failure
    TlsFailure,
}

/// System resources that can be exhausted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    /// Memory exhaustion
    Memory,
    /// CPU saturation
    Cpu,
    /// Network bandwidth
    NetworkBandwidth,
    /// File descriptors
    FileDescriptors,
    /// Database connections
    DatabaseConnections,
    /// Thread pool
    ThreadPool,
}

/// Configuration for a chaos engineering experiment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    /// Unique experiment identifier
    pub id: String,
    /// Experiment name and description
    pub name: String,
    pub description: String,
    /// Faults to inject during the experiment
    pub faults: Vec<FaultType>,
    /// Experiment duration
    pub duration: Duration,
    /// Target services/components to affect
    pub targets: Vec<String>,
    /// Expected outcomes and success criteria
    pub success_criteria: Vec<SuccessCriterion>,
    /// Monitoring configuration
    pub monitoring: MonitoringConfig,
    /// Recovery settings
    pub recovery: RecoveryConfig,
}

/// Success criteria for experiment validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuccessCriterion {
    /// System should remain responsive
    SystemResponsive {
        /// Maximum acceptable response time
        max_response_time_ms: u64,
        /// Success rate threshold (0.0 to 1.0)
        success_rate_threshold: f64,
    },
    /// Specific metrics should stay within bounds
    MetricBounds {
        /// Metric name to monitor
        metric_name: String,
        /// Minimum acceptable value
        min_value: f64,
        /// Maximum acceptable value
        max_value: f64,
    },
    /// Error rates should remain acceptable
    ErrorRate {
        /// Maximum acceptable error rate
        max_error_rate: f64,
        /// Time window for measurement
        window_duration: Duration,
    },
    /// Recovery should complete within time limit
    RecoveryTime {
        /// Maximum time for recovery
        max_recovery_time: Duration,
    },
}



/// Active experiment tracking
#[derive(Debug)]
pub struct ActiveExperiment {
    /// Experiment configuration
    pub config: ExperimentConfig,
    /// Start time
    pub start_time: Instant,
    /// Current status
    pub status: ExperimentStatus,
    /// Active fault injections
    pub active_faults: Vec<String>,
    /// Collected metrics
    pub metrics: Vec<MetricPoint>,
    /// Events during experiment
    pub events: Vec<ExperimentEvent>,
}

/// Experiment execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperimentStatus {
    /// Starting up the experiment
    Starting,
    /// Actively running
    Running,
    /// Injecting faults
    InjectingFaults,
    /// Monitoring recovery
    Recovering,
    /// Successfully completed
    Completed,
    /// Failed with error
    Failed { error: String },
    /// Manually stopped
    Stopped,
}

/// Metric data point collected during experiment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    /// Metric name
    pub name: String,
    /// Metric value
    pub value: f64,
    /// Timestamp of collection
    pub timestamp: std::time::SystemTime,
    /// Additional labels/tags
    pub labels: HashMap<String, String>,
}

/// Events that occur during experiments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentEvent {
    /// Event timestamp
    pub timestamp: std::time::SystemTime,
    /// Event type
    pub event_type: EventType,
    /// Event description
    pub description: String,
    /// Additional event data
    pub data: serde_json::Value,
}

/// Types of experiment events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    /// Experiment started
    ExperimentStarted,
    /// Fault injection began
    FaultInjectionStarted,
    /// Fault injection stopped
    FaultInjectionStopped,
    /// System degradation detected
    SystemDegradation,
    /// Recovery initiated
    RecoveryInitiated,
    /// Recovery completed
    RecoveryCompleted,
    /// Experiment completed
    ExperimentCompleted,
    /// Error occurred
    Error,
    /// Alert triggered
    AlertTriggered,
}

/// Complete experiment results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResult {
    /// Experiment configuration that was run
    pub config: ExperimentConfig,
    /// Experiment execution status
    pub status: ExperimentStatus,
    /// Start and end times
    pub start_time: std::time::SystemTime,
    pub end_time: std::time::SystemTime,
    /// Duration of execution
    pub duration: Duration,
    /// Whether success criteria were met
    pub success_criteria_met: Vec<(SuccessCriterion, bool)>,
    /// All collected metrics
    pub metrics: Vec<MetricPoint>,
    /// All events during execution
    pub events: Vec<ExperimentEvent>,
    /// Summary statistics
    pub summary: ExperimentSummary,
    /// Lessons learned and recommendations
    pub recommendations: Vec<String>,
}

/// Experiment execution summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentSummary {
    /// Total requests processed
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Average response time
    pub avg_response_time_ms: f64,
    /// 95th percentile response time
    pub p95_response_time_ms: f64,
    /// Maximum response time
    pub max_response_time_ms: f64,
    /// Error rate during experiment
    pub error_rate: f64,
    /// Recovery time if applicable
    pub recovery_time: Option<Duration>,
    /// System health score (0.0 to 1.0)
    pub health_score: f64,
}

impl ChaosEngineer {
    /// Create a new chaos engineering orchestrator
    pub fn new() -> Self {
        Self {
            active_experiments: Arc::new(RwLock::new(HashMap::new())),
            fault_injector: Arc::new(fault_injection::FaultInjector::new()),
            monitor: Arc::new(monitoring::ChaosMonitor::new()),
            recovery: Arc::new(recovery::RecoveryOrchestrator::new()),
            results: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Run a chaos engineering experiment
    pub async fn run_experiment(
        &self, 
        config: ExperimentConfig
    ) -> Result<ExperimentResult, ChaosError> {
        let experiment_id = config.id.clone();
        let start_time = Instant::now();

        // Create active experiment tracking
        let active = ActiveExperiment {
            config: config.clone(),
            start_time,
            status: ExperimentStatus::Starting,
            active_faults: Vec::new(),
            metrics: Vec::new(),
            events: Vec::new(),
        };

        // Register active experiment
        {
            let mut experiments = self.active_experiments.write().await;
            experiments.insert(experiment_id.clone(), active);
        }

        // Execute experiment phases
        let result = self.execute_experiment(&experiment_id, &config).await;

        // Remove from active experiments
        {
            let mut experiments = self.active_experiments.write().await;
            experiments.remove(&experiment_id);
        }

        match result {
            Ok(experiment_result) => {
                // Store results
                {
                    let mut results = self.results.lock().await;
                    results.push(experiment_result.clone());
                }
                Ok(experiment_result)
            }
            Err(e) => Err(e),
        }
    }

    /// Execute the main experiment phases
    async fn execute_experiment(
        &self,
        experiment_id: &str,
        config: &ExperimentConfig,
    ) -> Result<ExperimentResult, ChaosError> {
        let start_time = std::time::SystemTime::now();

        // Phase 1: Setup and baseline
        self.update_experiment_status(experiment_id, ExperimentStatus::Starting).await;
        self.record_event(experiment_id, EventType::ExperimentStarted, 
                         "Chaos experiment started", serde_json::Value::Null).await;

        // Start monitoring
        let monitoring_handle = self.monitor.start_monitoring(&config.monitoring).await?;

        // Phase 2: Fault injection
        self.update_experiment_status(experiment_id, ExperimentStatus::InjectingFaults).await;
        let fault_handles = self.inject_faults(experiment_id, &config.faults).await?;

        // Phase 3: Observation period
        self.update_experiment_status(experiment_id, ExperimentStatus::Running).await;
        tokio::time::sleep(config.duration).await;

        // Phase 4: Recovery
        self.update_experiment_status(experiment_id, ExperimentStatus::Recovering).await;
        self.stop_faults(experiment_id, fault_handles).await?;
        
        if config.recovery.auto_recovery {
            self.recovery.initiate_recovery(experiment_id, &config.recovery).await?;
        }

        // Phase 5: Validation and results
        let metrics = self.monitor.stop_monitoring(monitoring_handle).await?;
        let events = self.get_experiment_events(experiment_id).await;
        
        let success_criteria_met = self.validate_success_criteria(
            &config.success_criteria, 
            &metrics
        ).await?;

        let end_time = std::time::SystemTime::now();
        let duration = end_time.duration_since(start_time)?;

        let summary = self.calculate_summary(&metrics).await;
        let recommendations = self.generate_recommendations(&config, &metrics, &success_criteria_met).await;

        self.update_experiment_status(experiment_id, ExperimentStatus::Completed).await;
        self.record_event(experiment_id, EventType::ExperimentCompleted, 
                         "Chaos experiment completed", serde_json::Value::Null).await;

        Ok(ExperimentResult {
            config: config.clone(),
            status: ExperimentStatus::Completed,
            start_time,
            end_time,
            duration,
            success_criteria_met,
            metrics,
            events,
            summary,
            recommendations,
        })
    }

    /// Inject configured faults into the system
    async fn inject_faults(
        &self,
        experiment_id: &str,
        faults: &[FaultType],
    ) -> Result<Vec<String>, ChaosError> {
        let mut handles = Vec::new();

        for fault in faults {
            let handle = self.fault_injector.inject_fault(fault.clone()).await?;
            handles.push(handle.clone());

            self.record_event(
                experiment_id,
                EventType::FaultInjectionStarted,
                &format!("Fault injection started: {:?}", fault),
                serde_json::to_value(fault)?
            ).await;
        }

        Ok(handles)
    }

    /// Stop all active fault injections
    async fn stop_faults(
        &self,
        experiment_id: &str,
        fault_handles: Vec<String>,
    ) -> Result<(), ChaosError> {
        for handle in fault_handles {
            self.fault_injector.stop_fault(&handle).await?;
            
            self.record_event(
                experiment_id,
                EventType::FaultInjectionStopped,
                &format!("Fault injection stopped: {}", handle),
                serde_json::Value::String(handle)
            ).await;
        }

        Ok(())
    }

    /// Update experiment status
    async fn update_experiment_status(&self, experiment_id: &str, status: ExperimentStatus) {
        let mut experiments = self.active_experiments.write().await;
        if let Some(experiment) = experiments.get_mut(experiment_id) {
            experiment.status = status;
        }
    }

    /// Record an experiment event
    async fn record_event(
        &self,
        experiment_id: &str,
        event_type: EventType,
        description: &str,
        data: serde_json::Value,
    ) {
        let event = ExperimentEvent {
            timestamp: std::time::SystemTime::now(),
            event_type,
            description: description.to_string(),
            data,
        };

        let mut experiments = self.active_experiments.write().await;
        if let Some(experiment) = experiments.get_mut(experiment_id) {
            experiment.events.push(event);
        }
    }

    /// Get all events for an experiment
    async fn get_experiment_events(&self, experiment_id: &str) -> Vec<ExperimentEvent> {
        let experiments = self.active_experiments.read().await;
        if let Some(experiment) = experiments.get(experiment_id) {
            return experiment.events.clone();
        }
        Vec::new()
    }

    /// Validate success criteria against collected metrics
    async fn validate_success_criteria(
        &self,
        criteria: &[SuccessCriterion],
        metrics: &[MetricPoint],
    ) -> Result<Vec<(SuccessCriterion, bool)>, ChaosError> {
        let mut results = Vec::new();

        for criterion in criteria {
            let met = match criterion {
                SuccessCriterion::SystemResponsive { max_response_time_ms, success_rate_threshold } => {
                    self.validate_system_responsiveness(metrics, *max_response_time_ms, *success_rate_threshold).await
                }
                SuccessCriterion::MetricBounds { metric_name, min_value, max_value } => {
                    self.validate_metric_bounds(metrics, metric_name, *min_value, *max_value).await
                }
                SuccessCriterion::ErrorRate { max_error_rate, window_duration } => {
                    self.validate_error_rate(metrics, *max_error_rate, *window_duration).await
                }
                SuccessCriterion::RecoveryTime { max_recovery_time } => {
                    self.validate_recovery_time(metrics, *max_recovery_time).await
                }
            };

            results.push((criterion.clone(), met));
        }

        Ok(results)
    }

    /// Calculate experiment summary statistics
    async fn calculate_summary(&self, metrics: &[MetricPoint]) -> ExperimentSummary {
        let response_times: Vec<f64> = metrics
            .iter()
            .filter(|m| m.name == "response_time_ms")
            .map(|m| m.value)
            .collect();

        let total_requests = metrics
            .iter()
            .find(|m| m.name == "total_requests")
            .map(|m| m.value as u64)
            .unwrap_or(0);

        let successful_requests = metrics
            .iter()
            .find(|m| m.name == "successful_requests")
            .map(|m| m.value as u64)
            .unwrap_or(0);

        let failed_requests = total_requests.saturating_sub(successful_requests);
        let error_rate = if total_requests > 0 {
            failed_requests as f64 / total_requests as f64
        } else {
            0.0
        };

        let avg_response_time_ms = if !response_times.is_empty() {
            response_times.iter().sum::<f64>() / response_times.len() as f64
        } else {
            0.0
        };

        let mut sorted_times = response_times.clone();
        sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let p95_response_time_ms = if !sorted_times.is_empty() {
            let index = (sorted_times.len() as f64 * 0.95) as usize;
            sorted_times.get(index).copied().unwrap_or(0.0)
        } else {
            0.0
        };

        let max_response_time_ms = sorted_times.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).copied().unwrap_or(0.0);

        let health_score = 1.0 - error_rate;

        ExperimentSummary {
            total_requests,
            successful_requests,
            failed_requests,
            avg_response_time_ms,
            p95_response_time_ms,
            max_response_time_ms,
            error_rate,
            recovery_time: None, // TODO: Calculate from recovery events
            health_score,
        }
    }

    /// Generate recommendations based on experiment results
    async fn generate_recommendations(
        &self,
        config: &ExperimentConfig,
        metrics: &[MetricPoint],
        success_criteria_met: &[(SuccessCriterion, bool)],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Analyze failed criteria
        for (criterion, met) in success_criteria_met {
            if !met {
                match criterion {
                    SuccessCriterion::SystemResponsive { .. } => {
                        recommendations.push("Consider implementing circuit breaker patterns to prevent cascade failures".to_string());
                        recommendations.push("Add request timeout and retry mechanisms".to_string());
                    }
                    SuccessCriterion::ErrorRate { .. } => {
                        recommendations.push("Implement graceful degradation for non-critical features".to_string());
                        recommendations.push("Add error rate monitoring and alerting".to_string());
                    }
                    SuccessCriterion::MetricBounds { metric_name, .. } => {
                        recommendations.push(format!("Monitor {} metric more closely and set up alerts", metric_name));
                    }
                    SuccessCriterion::RecoveryTime { .. } => {
                        recommendations.push("Optimize recovery procedures and reduce MTTR".to_string());
                        recommendations.push("Consider implementing automated recovery mechanisms".to_string());
                    }
                }
            }
        }

        // Add general recommendations based on fault types
        for fault in &config.faults {
            match fault {
                FaultType::NetworkFailure { .. } => {
                    recommendations.push("Consider implementing multiple AI provider fallbacks".to_string());
                }
                FaultType::ResourceExhaustion { .. } => {
                    recommendations.push("Implement resource quotas and limits".to_string());
                }
                FaultType::MemoryPressure { .. } => {
                    recommendations.push("Monitor memory usage and implement garbage collection tuning".to_string());
                }
                _ => {}
            }
        }

        recommendations
    }

    /// Validation helper methods
    async fn validate_system_responsiveness(
        &self,
        metrics: &[MetricPoint],
        max_response_time_ms: u64,
        success_rate_threshold: f64,
    ) -> bool {
        let response_times: Vec<f64> = metrics
            .iter()
            .filter(|m| m.name == "response_time_ms")
            .map(|m| m.value)
            .collect();

        if response_times.is_empty() {
            return false;
        }

        let successful_responses = response_times
            .iter()
            .filter(|&&time| time <= max_response_time_ms as f64)
            .count();

        let success_rate = successful_responses as f64 / response_times.len() as f64;
        success_rate >= success_rate_threshold
    }

    async fn validate_metric_bounds(
        &self,
        metrics: &[MetricPoint],
        metric_name: &str,
        min_value: f64,
        max_value: f64,
    ) -> bool {
        metrics
            .iter()
            .filter(|m| m.name == metric_name)
            .all(|m| m.value >= min_value && m.value <= max_value)
    }

    async fn validate_error_rate(
        &self,
        metrics: &[MetricPoint],
        max_error_rate: f64,
        _window_duration: Duration,
    ) -> bool {
        // Get the latest error rate measurement
        metrics
            .iter()
            .filter(|m| m.name == "error_rate")
            .last()
            .map(|m| m.value <= max_error_rate)
            .unwrap_or(true)
    }

    async fn validate_recovery_time(&self, metrics: &[MetricPoint], max_recovery_time: Duration) -> bool {
        metrics
            .iter()
            .filter(|m| m.name == "recovery_time_ms")
            .last()
            .map(|m| Duration::from_millis(m.value as u64) <= max_recovery_time)
            .unwrap_or(true)
    }

    /// List all experiment results
    pub async fn get_experiment_results(&self) -> Vec<ExperimentResult> {
        let results = self.results.lock().await;
        results.clone()
    }

    /// Get specific experiment result
    pub async fn get_experiment_result(&self, experiment_id: &str) -> Option<ExperimentResult> {
        let results = self.results.lock().await;
        results.iter().find(|r| r.config.id == experiment_id).cloned()
    }

    /// Get currently active experiments
    pub async fn get_active_experiments(&self) -> HashMap<String, ExperimentStatus> {
        let experiments = self.active_experiments.read().await;
        experiments
            .iter()
            .map(|(id, exp)| (id.clone(), exp.status.clone()))
            .collect()
    }
}

/// Chaos engineering errors
#[derive(Debug, thiserror::Error)]
pub enum ChaosError {
    #[error("Fault injection failed: {0}")]
    FaultInjectionError(String),

    #[error("Monitoring error: {0}")]
    MonitoringError(String),

    #[error("Recovery error: {0}")]
    RecoveryError(String),

    #[error("Experiment validation failed: {0}")]
    ValidationError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("System time error: {0}")]
    SystemTimeError(#[from] std::time::SystemTimeError),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl ExperimentConfig {
    /// Create a new experiment configuration
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: "Untitled Experiment".to_string(),
            description: String::new(),
            faults: Vec::new(),
            duration: Duration::from_secs(60),
            targets: Vec::new(),
            success_criteria: Vec::new(),
            monitoring: MonitoringConfig::default(),
            recovery: RecoveryConfig::default(),
        }
    }

    /// Set experiment name and description
    pub fn with_name(mut self, name: String, description: String) -> Self {
        self.name = name;
        self.description = description;
        self
    }

    /// Add a fault to inject during the experiment
    pub fn with_fault(mut self, fault: FaultType) -> Self {
        self.faults.push(fault);
        self
    }

    /// Set experiment duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Add target component/service
    pub fn with_target(mut self, target: String) -> Self {
        self.targets.push(target);
        self
    }

    /// Add success criterion
    pub fn with_success_criterion(mut self, criterion: SuccessCriterion) -> Self {
        self.success_criteria.push(criterion);
        self
    }

    /// Set monitoring configuration
    pub fn with_monitoring(mut self, monitoring: MonitoringConfig) -> Self {
        self.monitoring = monitoring;
        self
    }

    /// Set recovery configuration
    pub fn with_recovery(mut self, recovery: RecoveryConfig) -> Self {
        self.recovery = recovery;
        self
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            metrics: vec![
                "response_time_ms".to_string(),
                "error_rate".to_string(),
                "cpu_usage".to_string(),
                "memory_usage".to_string(),
                "request_count".to_string(),
            ],
            collection_interval: Duration::from_secs(1),
            detailed_logging: true,
            alert_thresholds: HashMap::new(),
        }
    }
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            auto_recovery: true,
            recovery_timeout: Duration::from_secs(300),
            strategies: vec![recovery::RecoveryStrategy::RestartServices],
            validation_steps: Vec::new(),
        }
    }
}

/// Monitoring configuration for experiments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Metrics to collect during experiment
    pub metrics: Vec<String>,
    /// Collection interval
    pub collection_interval: Duration,
    /// Enable detailed logging
    pub detailed_logging: bool,
    /// Alert thresholds
    pub alert_thresholds: HashMap<String, f64>,
}

/// Recovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryConfig {
    /// Automatic recovery enabled
    pub auto_recovery: bool,
    /// Recovery timeout
    pub recovery_timeout: Duration,
    /// Recovery strategies to attempt
    pub strategies: Vec<recovery::RecoveryStrategy>,
    /// Validation steps after recovery
    pub validation_steps: Vec<recovery::ValidationStep>,
}

 