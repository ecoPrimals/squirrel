// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Comprehensive Benchmarking Framework for ecoPrimals Ecosystem
#![allow(dead_code)] // Benchmarking infrastructure awaiting activation
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

mod runners;

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
    /// Unique identifier for this benchmark run
    pub benchmark_id: String,
    /// Name of the operation benchmarked
    pub operation_name: String,
    /// Total duration in milliseconds
    pub duration_ms: f64,
    /// Operations per second achieved
    pub ops_per_second: f64,
    /// Number of operations completed
    pub operations_count: u64,
    /// Concurrency level used
    pub concurrency_level: u32,
    /// Memory usage in MB during benchmark
    pub memory_usage_mb: f64,
    /// CPU usage percentage during benchmark
    pub cpu_usage_percent: f64,
    /// Fraction of operations that succeeded (0.0 to 1.0)
    pub success_rate: f64,
    /// When the benchmark was run
    pub timestamp: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Where this benchmark baseline came from (script, commit, environment).
    pub provenance: universal_patterns::provenance::Provenance,
}

/// Benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Benchmark name
    pub name: String,
    /// Duration of the benchmark phase
    pub duration: Duration,
    /// Concurrency levels to test
    pub concurrency_levels: Vec<u32>,
    /// Maximum operations to run
    pub operation_count: u64,
    /// Warm-up phase duration
    pub warm_up_duration: Duration,
    /// Cool-down phase duration
    pub cool_down_duration: Duration,
    /// Whether to monitor memory usage
    pub memory_monitoring: bool,
    /// Whether to monitor CPU usage
    pub cpu_monitoring: bool,
    /// Whether to collect detailed metrics
    pub collect_detailed_metrics: bool,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            name: "default_benchmark".to_string(),
            duration: Duration::from_millis(100),
            concurrency_levels: vec![1, 4, 8],
            operation_count: 100,
            warm_up_duration: Duration::from_millis(10),
            cool_down_duration: Duration::ZERO,
            memory_monitoring: true,
            cpu_monitoring: true,
            collect_detailed_metrics: true,
        }
    }
}

impl BenchmarkConfig {
    /// Create a production benchmark config with longer durations for real perf measurement.
    pub fn production() -> Self {
        Self {
            name: "production_benchmark".to_string(),
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
    /// Memory usage in MB
    pub memory_usage_mb: f64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Network I/O bytes
    pub network_io_bytes: u64,
    /// Disk I/O bytes
    pub disk_io_bytes: u64,
    /// Thread count
    pub thread_count: u32,
    /// Handle count
    pub handle_count: u32,
    /// When metrics were collected
    pub timestamp: DateTime<Utc>,
}

/// Individual benchmark runner
#[derive(Debug, Clone)]
pub struct BenchmarkRunner {
    /// Benchmark configuration
    pub config: BenchmarkConfig,
    /// When the benchmark started
    pub start_time: Option<Instant>,
    /// When the benchmark ended
    pub end_time: Option<Instant>,
    /// Number of operations completed
    pub operations_completed: u64,
    /// Number of operations that failed
    pub operations_failed: u64,
    /// Total duration of the benchmark
    pub total_duration: Duration,
}

impl BenchmarkSuite {
    /// Create a new benchmark suite
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            suite_id: format!("{}_{}", name, Uuid::new_v4()),
            results: Arc::new(RwLock::new(Vec::new())),
            active_benchmarks: Arc::new(RwLock::new(HashMap::new())),
            system_metrics: Arc::new(RwLock::new(SystemMetrics::default())),
        }
    }

    /// Get the suite identifier (for testing)
    #[must_use]
    pub fn suite_id(&self) -> &str {
        &self.suite_id
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

    /// Core benchmark execution framework
    pub async fn run_benchmark<F, Fut>(
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
            let _ = operation().await; // Ignore warm-up failures
        }

        // Main benchmark phase
        debug!("Starting main benchmark phase for {}", name);
        let benchmark_start = Instant::now();
        let benchmark_end = benchmark_start + config.duration;

        // Run operations (collect_system_metrics deferred to end - it can block)
        let mut operation_count = 0u64;
        while Instant::now() < benchmark_end && operation_count < config.operation_count {
            match operation().await {
                Ok(()) => successful_operations += 1,
                Err(_) => _failed_operations += 1,
            }
            operation_count += 1;
        }

        // Cool-down phase: yield to let other tasks run (no sleep)
        debug!("Cool-down yield for {}", name);
        tokio::task::yield_now().await;

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
            provenance: universal_patterns::provenance::Provenance::auto(),
        };

        // Store result
        self.results.write().await.push(result.clone());

        Ok(result)
    }

    /// Collect system metrics
    ///
    /// Uses /proc/self/statm on Linux for memory (pure Rust, no C deps).
    pub(crate) async fn collect_system_metrics(&self) -> SystemMetrics {
        let (memory_usage_mb, cpu_usage_percent) = Self::measure_process_resources();
        let (network_io_bytes, disk_io_bytes) = (0u64, 0u64); // Would integrate with /proc/net or similar
        let thread_count = std::thread::available_parallelism()
            .map(|p| p.get() as u32)
            .unwrap_or(1);
        SystemMetrics {
            memory_usage_mb,
            cpu_usage_percent,
            network_io_bytes,
            disk_io_bytes,
            thread_count,
            handle_count: 256,
            timestamp: Utc::now(),
        }
    }

    /// Measure process memory (RSS) and CPU usage.
    ///
    /// On Linux: reads /proc/self/statm for RSS (resident set size).
    /// Uses universal_constants::sys_info (pure Rust, no C deps).
    fn measure_process_resources() -> (f64, f64) {
        let mem_mb = universal_constants::sys_info::process_rss_mb().unwrap_or(0.0);
        let cpu = universal_constants::sys_info::system_cpu_usage_percent().unwrap_or(0.0);
        (mem_mb, cpu)
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
    /// Suite identifier
    pub suite_id: String,
    /// Total duration of the suite
    pub total_duration: Duration,
    /// Number of benchmarks run
    pub total_benchmarks: usize,
    /// Individual benchmark results
    pub results: Vec<BenchmarkResult>,
    /// System metrics at report time
    pub system_metrics: SystemMetrics,
    /// When the report was generated
    pub timestamp: DateTime<Utc>,
}

impl BenchmarkSuiteReport {
    /// Generate summary statistics
    #[must_use]
    pub fn generate_summary(&self) -> BenchmarkSummary {
        let total_ops = self.results.iter().map(|r| r.operations_count).sum();
        let avg_ops_per_sec = if self.results.is_empty() {
            0.0
        } else {
            self.results.iter().map(|r| r.ops_per_second).sum::<f64>() / self.results.len() as f64
        };
        let avg_success_rate = if self.results.is_empty() {
            0.0
        } else {
            self.results.iter().map(|r| r.success_rate).sum::<f64>() / self.results.len() as f64
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
    /// Total number of benchmarks run
    pub total_benchmarks: usize,
    /// Total operations across all benchmarks
    pub total_operations: u64,
    /// Total duration of the suite
    pub total_duration: Duration,
    /// Average operations per second
    pub average_ops_per_second: f64,
    /// Average success rate (0.0 to 1.0)
    pub average_success_rate: f64,
    /// Peak memory usage in MB
    pub peak_memory_usage_mb: f64,
    /// Peak CPU usage percentage
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
#[path = "benchmarking_tests.rs"]
mod tests;
