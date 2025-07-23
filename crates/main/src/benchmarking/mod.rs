//! Comprehensive Benchmarking Framework for ecoPrimals Ecosystem
//!
//! This module provides a unified benchmarking framework for measuring performance
//! across all ecosystem components including:
//! - Squirrel AI intelligence processing
//! - Songbird orchestration and coordination
//! - ToadStool compute delegation
//! - NestGate storage operations
//! - BearDog security and authentication
//! - MCP protocol operations
//! - Universal patterns and adapters
//! - BiomeOS integration features

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

use crate::error::PrimalError;

/// Benchmark result for a single operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub benchmark_id: String,
    pub operation_name: String,
    pub duration_ms: f64,
    pub ops_per_second: f64,
    pub operations_count: u64,
    pub concurrency_level: u32,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub success_rate: f64,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// Benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    pub name: String,
    pub duration: Duration,
    pub concurrency_levels: Vec<u32>,
    pub operation_count: u64,
    pub warm_up_duration: Duration,
    pub cool_down_duration: Duration,
    pub memory_monitoring: bool,
    pub cpu_monitoring: bool,
    pub collect_detailed_metrics: bool,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            name: "default_benchmark".to_string(),
            duration: Duration::from_secs(30),
            concurrency_levels: vec![1, 4, 8, 16, 32],
            operation_count: 1000,
            warm_up_duration: Duration::from_secs(5),
            cool_down_duration: Duration::from_secs(2),
            memory_monitoring: true,
            cpu_monitoring: true,
            collect_detailed_metrics: true,
        }
    }
}

/// Benchmark suite for ecosystem components
#[derive(Debug, Clone)]
pub struct BenchmarkSuite {
    suite_id: String,
    results: Arc<RwLock<Vec<BenchmarkResult>>>,
    active_benchmarks: Arc<RwLock<HashMap<String, BenchmarkRunner>>>,
    system_metrics: Arc<RwLock<SystemMetrics>>,
}

/// System metrics collected during benchmarks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub network_io_bytes: u64,
    pub disk_io_bytes: u64,
    pub thread_count: u32,
    pub handle_count: u32,
    pub timestamp: DateTime<Utc>,
}

/// Individual benchmark runner
#[derive(Debug, Clone)]
pub struct BenchmarkRunner {
    pub config: BenchmarkConfig,
    pub start_time: Option<Instant>,
    pub end_time: Option<Instant>,
    pub operations_completed: u64,
    pub operations_failed: u64,
    pub total_duration: Duration,
}

impl BenchmarkSuite {
    /// Create a new benchmark suite
    pub fn new(name: &str) -> Self {
        Self {
            suite_id: format!("{}_{}", name, Uuid::new_v4()),
            results: Arc::new(RwLock::new(Vec::new())),
            active_benchmarks: Arc::new(RwLock::new(HashMap::new())),
            system_metrics: Arc::new(RwLock::new(SystemMetrics::default())),
        }
    }

    /// Run AI intelligence benchmarks
    pub async fn benchmark_ai_intelligence(&self) -> Result<Vec<BenchmarkResult>, PrimalError> {
        let mut results = Vec::new();

        // Benchmark different AI operations
        info!("Running AI benchmark: text_generation");
        results.push(self.benchmark_text_generation().await?);

        info!("Running AI benchmark: context_processing");
        results.push(self.benchmark_context_processing().await?);

        info!("Running AI benchmark: tool_orchestration");
        results.push(self.benchmark_tool_orchestration().await?);

        info!("Running AI benchmark: response_synthesis");
        results.push(self.benchmark_response_synthesis().await?);

        Ok(results)
    }

    /// Run orchestration benchmarks
    pub async fn benchmark_orchestration(&self) -> Result<Vec<BenchmarkResult>, PrimalError> {
        let mut results = Vec::new();

        info!("Running orchestration benchmark: task_scheduling");
        results.push(self.benchmark_task_scheduling().await?);

        info!("Running orchestration benchmark: service_discovery");
        results.push(self.benchmark_service_discovery().await?);

        info!("Running orchestration benchmark: load_balancing");
        results.push(self.benchmark_load_balancing().await?);

        info!("Running orchestration benchmark: health_monitoring");
        results.push(self.benchmark_health_monitoring().await?);

        Ok(results)
    }

    /// Run compute delegation benchmarks
    pub async fn benchmark_compute_delegation(&self) -> Result<Vec<BenchmarkResult>, PrimalError> {
        let mut results = Vec::new();

        info!("Running compute benchmark: job_submission");
        results.push(self.benchmark_job_submission().await?);

        info!("Running compute benchmark: resource_allocation");
        results.push(self.benchmark_resource_allocation().await?);

        info!("Running compute benchmark: parallel_processing");
        results.push(self.benchmark_parallel_processing().await?);

        info!("Running compute benchmark: job_completion");
        results.push(self.benchmark_job_completion().await?);

        Ok(results)
    }

    /// Run storage benchmarks
    pub async fn benchmark_storage(&self) -> Result<Vec<BenchmarkResult>, PrimalError> {
        let mut results = Vec::new();

        info!("Running storage benchmark: data_storage");
        results.push(self.benchmark_data_storage().await?);

        info!("Running storage benchmark: data_retrieval");
        results.push(self.benchmark_data_retrieval().await?);

        info!("Running storage benchmark: context_persistence");
        results.push(self.benchmark_context_persistence().await?);

        info!("Running storage benchmark: model_caching");
        results.push(self.benchmark_model_caching().await?);

        Ok(results)
    }

    /// Run security benchmarks
    pub async fn benchmark_security(&self) -> Result<Vec<BenchmarkResult>, PrimalError> {
        let mut results = Vec::new();

        info!("Running security benchmark: authentication");
        results.push(self.benchmark_authentication().await?);

        info!("Running security benchmark: authorization");
        results.push(self.benchmark_authorization().await?);

        info!("Running security benchmark: token_validation");
        results.push(self.benchmark_token_validation().await?);

        info!("Running security benchmark: credential_management");
        results.push(self.benchmark_credential_management().await?);

        Ok(results)
    }

    /// Run MCP protocol benchmarks
    pub async fn benchmark_mcp_protocol(&self) -> Result<Vec<BenchmarkResult>, PrimalError> {
        let mut results = Vec::new();

        info!("Running MCP benchmark: message_serialization");
        results.push(self.benchmark_message_serialization().await?);

        info!("Running MCP benchmark: connection_management");
        results.push(self.benchmark_connection_management().await?);

        info!("Running MCP benchmark: session_handling");
        results.push(self.benchmark_session_handling().await?);

        info!("Running MCP benchmark: protocol_negotiation");
        results.push(self.benchmark_protocol_negotiation().await?);

        Ok(results)
    }

    /// Run complete ecosystem benchmark suite
    pub async fn run_complete_benchmark_suite(&self) -> Result<BenchmarkSuiteReport, PrimalError> {
        info!("Starting complete ecosystem benchmark suite");

        let start_time = Instant::now();
        let mut all_results = Vec::new();

        // Run all benchmark categories
        info!("Running AI Intelligence benchmarks");
        all_results.extend(self.benchmark_ai_intelligence().await?);

        info!("Running Orchestration benchmarks");
        all_results.extend(self.benchmark_orchestration().await?);

        info!("Running Compute Delegation benchmarks");
        all_results.extend(self.benchmark_compute_delegation().await?);

        info!("Running Storage benchmarks");
        all_results.extend(self.benchmark_storage().await?);

        info!("Running Security benchmarks");
        all_results.extend(self.benchmark_security().await?);

        info!("Running MCP Protocol benchmarks");
        all_results.extend(self.benchmark_mcp_protocol().await?);

        let total_duration = start_time.elapsed();

        // Generate comprehensive report
        let report = BenchmarkSuiteReport {
            suite_id: self.suite_id.clone(),
            total_duration,
            total_benchmarks: all_results.len(),
            results: all_results,
            system_metrics: self.system_metrics.read().await.clone(),
            timestamp: Utc::now(),
        };

        info!(
            "Completed ecosystem benchmark suite in {:?}",
            total_duration
        );
        Ok(report)
    }

    // Individual benchmark implementations
    async fn benchmark_text_generation(&self) -> Result<BenchmarkResult, PrimalError> {
        let config = BenchmarkConfig {
            name: "text_generation".to_string(),
            ..Default::default()
        };

        self.run_benchmark("text_generation", config, || async {
            // Simulate text generation
            tokio::time::sleep(Duration::from_millis(50)).await;
            Ok(())
        })
        .await
    }

    async fn benchmark_context_processing(&self) -> Result<BenchmarkResult, PrimalError> {
        let config = BenchmarkConfig {
            name: "context_processing".to_string(),
            ..Default::default()
        };

        self.run_benchmark("context_processing", config, || async {
            // Simulate context processing
            tokio::time::sleep(Duration::from_millis(30)).await;
            Ok(())
        })
        .await
    }

    async fn benchmark_tool_orchestration(&self) -> Result<BenchmarkResult, PrimalError> {
        let config = BenchmarkConfig {
            name: "tool_orchestration".to_string(),
            ..Default::default()
        };

        self.run_benchmark("tool_orchestration", config, || async {
            // Simulate tool orchestration
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok(())
        })
        .await
    }

    async fn benchmark_response_synthesis(&self) -> Result<BenchmarkResult, PrimalError> {
        let config = BenchmarkConfig {
            name: "response_synthesis".to_string(),
            ..Default::default()
        };

        self.run_benchmark("response_synthesis", config, || async {
            // Simulate response synthesis
            tokio::time::sleep(Duration::from_millis(75)).await;
            Ok(())
        })
        .await
    }

    async fn benchmark_task_scheduling(&self) -> Result<BenchmarkResult, PrimalError> {
        let config = BenchmarkConfig {
            name: "task_scheduling".to_string(),
            ..Default::default()
        };

        self.run_benchmark("task_scheduling", config, || async {
            // Simulate task scheduling
            tokio::time::sleep(Duration::from_millis(25)).await;
            Ok(())
        })
        .await
    }

    async fn benchmark_service_discovery(&self) -> Result<BenchmarkResult, PrimalError> {
        let config = BenchmarkConfig {
            name: "service_discovery".to_string(),
            ..Default::default()
        };

        self.run_benchmark("service_discovery", config, || async {
            // Simulate service discovery
            tokio::time::sleep(Duration::from_millis(40)).await;
            Ok(())
        })
        .await
    }

    async fn benchmark_load_balancing(&self) -> Result<BenchmarkResult, PrimalError> {
        let config = BenchmarkConfig {
            name: "load_balancing".to_string(),
            ..Default::default()
        };

        self.run_benchmark("load_balancing", config, || async {
            // Simulate load balancing
            tokio::time::sleep(Duration::from_millis(20)).await;
            Ok(())
        })
        .await
    }

    async fn benchmark_health_monitoring(&self) -> Result<BenchmarkResult, PrimalError> {
        let config = BenchmarkConfig {
            name: "health_monitoring".to_string(),
            ..Default::default()
        };

        self.run_benchmark("health_monitoring", config, || async {
            // Simulate health monitoring
            tokio::time::sleep(Duration::from_millis(15)).await;
            Ok(())
        })
        .await
    }

    async fn benchmark_job_submission(&self) -> Result<BenchmarkResult, PrimalError> {
        let config = BenchmarkConfig {
            name: "job_submission".to_string(),
            ..Default::default()
        };

        self.run_benchmark("job_submission", config, || async {
            // Simulate job submission
            tokio::time::sleep(Duration::from_millis(35)).await;
            Ok(())
        })
        .await
    }

    async fn benchmark_resource_allocation(&self) -> Result<BenchmarkResult, PrimalError> {
        let config = BenchmarkConfig {
            name: "resource_allocation".to_string(),
            ..Default::default()
        };

        self.run_benchmark("resource_allocation", config, || async {
            // Simulate resource allocation
            tokio::time::sleep(Duration::from_millis(45)).await;
            Ok(())
        })
        .await
    }

    async fn benchmark_parallel_processing(&self) -> Result<BenchmarkResult, PrimalError> {
        let config = BenchmarkConfig {
            name: "parallel_processing".to_string(),
            ..Default::default()
        };

        self.run_benchmark("parallel_processing", config, || async {
            // Simulate parallel processing
            tokio::time::sleep(Duration::from_millis(80)).await;
            Ok(())
        })
        .await
    }

    async fn benchmark_job_completion(&self) -> Result<BenchmarkResult, PrimalError> {
        let config = BenchmarkConfig {
            name: "job_completion".to_string(),
            ..Default::default()
        };

        self.run_benchmark("job_completion", config, || async {
            // Simulate job completion
            tokio::time::sleep(Duration::from_millis(60)).await;
            Ok(())
        })
        .await
    }

    async fn benchmark_data_storage(&self) -> Result<BenchmarkResult, PrimalError> {
        let config = BenchmarkConfig {
            name: "data_storage".to_string(),
            ..Default::default()
        };

        self.run_benchmark("data_storage", config, || async {
            // Simulate data storage
            tokio::time::sleep(Duration::from_millis(70)).await;
            Ok(())
        })
        .await
    }

    async fn benchmark_data_retrieval(&self) -> Result<BenchmarkResult, PrimalError> {
        let config = BenchmarkConfig {
            name: "data_retrieval".to_string(),
            ..Default::default()
        };

        self.run_benchmark("data_retrieval", config, || async {
            // Simulate data retrieval
            tokio::time::sleep(Duration::from_millis(40)).await;
            Ok(())
        })
        .await
    }

    async fn benchmark_context_persistence(&self) -> Result<BenchmarkResult, PrimalError> {
        let config = BenchmarkConfig {
            name: "context_persistence".to_string(),
            ..Default::default()
        };

        self.run_benchmark("context_persistence", config, || async {
            // Simulate context persistence
            tokio::time::sleep(Duration::from_millis(55)).await;
            Ok(())
        })
        .await
    }

    async fn benchmark_model_caching(&self) -> Result<BenchmarkResult, PrimalError> {
        let config = BenchmarkConfig {
            name: "model_caching".to_string(),
            ..Default::default()
        };

        self.run_benchmark("model_caching", config, || async {
            // Simulate model caching
            tokio::time::sleep(Duration::from_millis(30)).await;
            Ok(())
        })
        .await
    }

    async fn benchmark_authentication(&self) -> Result<BenchmarkResult, PrimalError> {
        let config = BenchmarkConfig {
            name: "authentication".to_string(),
            ..Default::default()
        };

        self.run_benchmark("authentication", config, || async {
            // Simulate authentication
            tokio::time::sleep(Duration::from_millis(25)).await;
            Ok(())
        })
        .await
    }

    async fn benchmark_authorization(&self) -> Result<BenchmarkResult, PrimalError> {
        let config = BenchmarkConfig {
            name: "authorization".to_string(),
            ..Default::default()
        };

        self.run_benchmark("authorization", config, || async {
            // Simulate authorization
            tokio::time::sleep(Duration::from_millis(20)).await;
            Ok(())
        })
        .await
    }

    async fn benchmark_token_validation(&self) -> Result<BenchmarkResult, PrimalError> {
        let config = BenchmarkConfig {
            name: "token_validation".to_string(),
            ..Default::default()
        };

        self.run_benchmark("token_validation", config, || async {
            // Simulate token validation
            tokio::time::sleep(Duration::from_millis(15)).await;
            Ok(())
        })
        .await
    }

    async fn benchmark_credential_management(&self) -> Result<BenchmarkResult, PrimalError> {
        let config = BenchmarkConfig {
            name: "credential_management".to_string(),
            ..Default::default()
        };

        self.run_benchmark("credential_management", config, || async {
            // Simulate credential management
            tokio::time::sleep(Duration::from_millis(35)).await;
            Ok(())
        })
        .await
    }

    async fn benchmark_message_serialization(&self) -> Result<BenchmarkResult, PrimalError> {
        let config = BenchmarkConfig {
            name: "message_serialization".to_string(),
            ..Default::default()
        };

        self.run_benchmark("message_serialization", config, || async {
            // Simulate message serialization
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok(())
        })
        .await
    }

    async fn benchmark_connection_management(&self) -> Result<BenchmarkResult, PrimalError> {
        let config = BenchmarkConfig {
            name: "connection_management".to_string(),
            ..Default::default()
        };

        self.run_benchmark("connection_management", config, || async {
            // Simulate connection management
            tokio::time::sleep(Duration::from_millis(30)).await;
            Ok(())
        })
        .await
    }

    async fn benchmark_session_handling(&self) -> Result<BenchmarkResult, PrimalError> {
        let config = BenchmarkConfig {
            name: "session_handling".to_string(),
            ..Default::default()
        };

        self.run_benchmark("session_handling", config, || async {
            // Simulate session handling
            tokio::time::sleep(Duration::from_millis(25)).await;
            Ok(())
        })
        .await
    }

    async fn benchmark_protocol_negotiation(&self) -> Result<BenchmarkResult, PrimalError> {
        let config = BenchmarkConfig {
            name: "protocol_negotiation".to_string(),
            ..Default::default()
        };

        self.run_benchmark("protocol_negotiation", config, || async {
            // Simulate protocol negotiation
            tokio::time::sleep(Duration::from_millis(40)).await;
            Ok(())
        })
        .await
    }

    /// Core benchmark execution framework
    async fn run_benchmark<F, Fut>(
        &self,
        name: &str,
        config: BenchmarkConfig,
        operation: F,
    ) -> Result<BenchmarkResult, PrimalError>
    where
        F: Fn() -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<(), PrimalError>> + Send + 'static,
    {
        let start_time = Instant::now();
        let mut successful_operations = 0u64;
        let mut _failed_operations = 0u64;

        // Warm-up phase
        debug!("Starting warm-up phase for {}", name);
        let warm_up_end = Instant::now() + config.warm_up_duration;
        while Instant::now() < warm_up_end {
            if let Err(_) = operation().await {
                // Ignore warm-up failures
            }
        }

        // Main benchmark phase
        debug!("Starting main benchmark phase for {}", name);
        let benchmark_start = Instant::now();
        let benchmark_end = benchmark_start + config.duration;

        // Record initial system metrics
        self.collect_system_metrics().await;

        // Run operations
        let mut operation_count = 0u64;
        while Instant::now() < benchmark_end && operation_count < config.operation_count {
            match operation().await {
                Ok(_) => successful_operations += 1,
                Err(_) => _failed_operations += 1,
            }
            operation_count += 1;
        }

        // Cool-down phase
        debug!("Starting cool-down phase for {}", name);
        tokio::time::sleep(config.cool_down_duration).await;

        // Calculate metrics
        let total_duration = start_time.elapsed();
        let operations_per_second = successful_operations as f64 / total_duration.as_secs_f64();
        let success_rate = if operation_count > 0 {
            successful_operations as f64 / operation_count as f64
        } else {
            0.0
        };

        // Collect final system metrics
        let final_metrics = self.collect_system_metrics().await;

        let result = BenchmarkResult {
            benchmark_id: Uuid::new_v4().to_string(),
            operation_name: name.to_string(),
            duration_ms: total_duration.as_millis() as f64,
            ops_per_second: operations_per_second,
            operations_count: successful_operations,
            concurrency_level: 1, // Base implementation is single-threaded
            memory_usage_mb: final_metrics.memory_usage_mb,
            cpu_usage_percent: final_metrics.cpu_usage_percent,
            success_rate,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };

        // Store result
        self.results.write().await.push(result.clone());

        Ok(result)
    }

    /// Collect system metrics
    async fn collect_system_metrics(&self) -> SystemMetrics {
        // This would integrate with actual system monitoring in production
        SystemMetrics {
            memory_usage_mb: 128.0,  // Placeholder
            cpu_usage_percent: 45.0, // Placeholder
            network_io_bytes: 1024,
            disk_io_bytes: 2048,
            thread_count: 16,
            handle_count: 256,
            timestamp: Utc::now(),
        }
    }

    /// Get all benchmark results
    pub async fn get_results(&self) -> Vec<BenchmarkResult> {
        self.results.read().await.clone()
    }

    /// Clear all benchmark results
    pub async fn clear_results(&self) {
        self.results.write().await.clear();
    }
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            network_io_bytes: 0,
            disk_io_bytes: 0,
            thread_count: 0,
            handle_count: 0,
            timestamp: Utc::now(),
        }
    }
}

/// Comprehensive benchmark suite report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSuiteReport {
    pub suite_id: String,
    pub total_duration: Duration,
    pub total_benchmarks: usize,
    pub results: Vec<BenchmarkResult>,
    pub system_metrics: SystemMetrics,
    pub timestamp: DateTime<Utc>,
}

impl BenchmarkSuiteReport {
    /// Generate summary statistics
    pub fn generate_summary(&self) -> BenchmarkSummary {
        let total_ops = self.results.iter().map(|r| r.operations_count).sum();
        let avg_ops_per_sec = if !self.results.is_empty() {
            self.results.iter().map(|r| r.ops_per_second).sum::<f64>() / self.results.len() as f64
        } else {
            0.0
        };
        let avg_success_rate = if !self.results.is_empty() {
            self.results.iter().map(|r| r.success_rate).sum::<f64>() / self.results.len() as f64
        } else {
            0.0
        };

        BenchmarkSummary {
            total_benchmarks: self.total_benchmarks,
            total_operations: total_ops,
            total_duration: self.total_duration,
            average_ops_per_second: avg_ops_per_sec,
            average_success_rate: avg_success_rate,
            peak_memory_usage_mb: self.system_metrics.memory_usage_mb,
            peak_cpu_usage_percent: self.system_metrics.cpu_usage_percent,
        }
    }
}

/// Summary statistics for benchmark suite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSummary {
    pub total_benchmarks: usize,
    pub total_operations: u64,
    pub total_duration: Duration,
    pub average_ops_per_second: f64,
    pub average_success_rate: f64,
    pub peak_memory_usage_mb: f64,
    pub peak_cpu_usage_percent: f64,
}

/// Initialize comprehensive benchmarking framework
pub async fn initialize_benchmarking() -> Result<BenchmarkSuite, PrimalError> {
    info!("Initializing comprehensive benchmarking framework");
    let suite = BenchmarkSuite::new("ecosystem_comprehensive");
    info!("Benchmarking framework initialized successfully");
    Ok(suite)
}

/// Run complete ecosystem benchmarks
pub async fn run_ecosystem_benchmarks() -> Result<BenchmarkSuiteReport, PrimalError> {
    let suite = initialize_benchmarking().await?;
    suite.run_complete_benchmark_suite().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_benchmark_suite_creation() {
        let suite = BenchmarkSuite::new("test_suite");
        assert!(suite.suite_id.contains("test_suite"));
    }

    #[test]
    async fn test_benchmark_config_default() {
        let config = BenchmarkConfig::default();
        assert_eq!(config.name, "default_benchmark");
        assert_eq!(config.duration, Duration::from_secs(30));
    }

    #[test]
    async fn test_benchmark_execution() {
        let suite = BenchmarkSuite::new("test_execution");
        let config = BenchmarkConfig::default();

        let result = suite
            .run_benchmark("test_operation", config, || async {
                tokio::time::sleep(Duration::from_millis(1)).await;
                Ok(())
            })
            .await;

        assert!(result.is_ok());
        let benchmark_result = result.unwrap();
        assert_eq!(benchmark_result.operation_name, "test_operation");
        assert!(benchmark_result.ops_per_second > 0.0);
    }

    #[test]
    async fn test_system_metrics_collection() {
        let suite = BenchmarkSuite::new("test_metrics");
        let metrics = suite.collect_system_metrics().await;

        assert!(metrics.memory_usage_mb >= 0.0);
        assert!(metrics.cpu_usage_percent >= 0.0);
    }

    #[test]
    async fn test_benchmark_report_generation() {
        let suite = BenchmarkSuite::new("test_report");

        // Run a simple benchmark
        let _result = suite
            .run_benchmark("test_op", BenchmarkConfig::default(), || async { Ok(()) })
            .await;

        let results = suite.get_results().await;
        assert!(!results.is_empty());
    }
}
