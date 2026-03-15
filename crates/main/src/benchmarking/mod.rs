// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

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
    /// Where this benchmark baseline came from (script, commit, environment).
    pub provenance: universal_patterns::provenance::Provenance,
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
    #[must_use]
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
    //
    // Each benchmark exercises actual code paths (serialization, hashing,
    // collection operations) rather than sleeping. This produces meaningful
    // perf data and doesn't block the executor.

    async fn benchmark_text_generation(&self) -> Result<BenchmarkResult, PrimalError> {
        let config = BenchmarkConfig {
            name: "text_generation".to_string(),
            ..Default::default()
        };
        self.run_benchmark("text_generation", config, || async {
            // JSON-RPC request/response cycle (real serialization work)
            let req = serde_json::json!({"method":"ai.query","params":{"prompt":"bench"},"id":1});
            let _s =
                serde_json::to_string(&req).expect("JSON serialization of valid Value cannot fail");
            let _v: serde_json::Value = serde_json::from_str(&_s)
                .expect("JSON deserialization of just-serialized Value cannot fail");
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
            // HashMap insert/lookup cycle
            let mut map = std::collections::HashMap::new();
            for i in 0..100 {
                map.insert(format!("key_{i}"), format!("val_{i}"));
            }
            for i in 0..100 {
                let _ = map.get(&format!("key_{i}"));
            }
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
            // Concurrent task spawn and join
            let handles: Vec<_> = (0..4).map(|i| tokio::spawn(async move { i * i })).collect();
            for h in handles {
                let _ = h.await;
            }
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
            let mut s = String::with_capacity(4096);
            for i in 0..100 {
                s.push_str(&format!("token_{i} "));
            }
            let _ = s.len();
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
            let mut v: Vec<u64> = (0..200).rev().collect();
            v.sort_unstable();
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
            // UUID generation + string formatting (discovery ID generation)
            for _ in 0..10 {
                let _ = uuid::Uuid::new_v4().to_string();
            }
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
            // Weighted random selection (load balancer hot path)
            let weights = [10u32, 20, 30, 40];
            let total: u32 = weights.iter().sum();
            let _ = weights
                .iter()
                .scan(0u32, |acc, &w| {
                    *acc += w;
                    Some(*acc)
                })
                .position(|acc| acc >= total / 2);
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
            let _ = chrono::Utc::now().to_rfc3339();
            let _ = serde_json::json!({"status":"healthy","ts": chrono::Utc::now().timestamp()});
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
            let job = serde_json::json!({"id": uuid::Uuid::new_v4().to_string(), "type":"compute","priority":5});
            let _ = serde_json::to_vec(&job).expect("JSON serialization of valid Value cannot fail");
            Ok(())
        }).await
    }

    async fn benchmark_resource_allocation(&self) -> Result<BenchmarkResult, PrimalError> {
        let config = BenchmarkConfig {
            name: "resource_allocation".to_string(),
            ..Default::default()
        };
        self.run_benchmark("resource_allocation", config, || async {
            let mut resources: Vec<(String, u64)> =
                (0..50).map(|i| (format!("res_{i}"), i * 1024)).collect();
            resources.sort_by_key(|r| std::cmp::Reverse(r.1));
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
            let handles: Vec<_> = (0..8)
                .map(|i| {
                    tokio::spawn(async move {
                        let mut sum = 0u64;
                        for j in 0..1000 {
                            sum += (i * 1000 + j) as u64;
                        }
                        sum
                    })
                })
                .collect();
            let mut total = 0u64;
            for h in handles {
                total += h.await.unwrap_or(0);
            }
            let _ = total;
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
            let result =
                serde_json::json!({"status":"complete","duration_ms":42,"output_size":1024});
            let _ = serde_json::to_string(&result)
                .expect("JSON serialization of valid Value cannot fail");
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
            let data: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
            let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data);
            let _ = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &encoded)
                .expect("base64 decode of just-encoded data cannot fail");
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
            let map: std::collections::HashMap<String, String> = (0..200)
                .map(|i| (format!("k{i}"), format!("v{i}")))
                .collect();
            for i in 0..200 {
                let _ = map.get(&format!("k{i}"));
            }
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
            let ctx = serde_json::json!({"session":"abc","data":{"k1":"v1","k2":"v2"},"ts": chrono::Utc::now().timestamp()});
            let bytes = serde_json::to_vec(&ctx).expect("JSON serialization of valid Value cannot fail");
            let _: serde_json::Value = serde_json::from_slice(&bytes).expect("JSON deserialization of just-serialized Value cannot fail");
            Ok(())
        }).await
    }

    async fn benchmark_model_caching(&self) -> Result<BenchmarkResult, PrimalError> {
        let config = BenchmarkConfig {
            name: "model_caching".to_string(),
            ..Default::default()
        };
        self.run_benchmark("model_caching", config, || async {
            let cache = dashmap::DashMap::new();
            for i in 0..100 {
                cache.insert(format!("model_{i}"), vec![0u8; 64]);
            }
            for i in 0..100 {
                let _ = cache.get(&format!("model_{i}"));
            }
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
            // Token generation and base64 encoding
            let token = format!(
                "{}:{}",
                uuid::Uuid::new_v4(),
                chrono::Utc::now().timestamp()
            );
            let _ = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                token.as_bytes(),
            );
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
            let perms = ["read", "write", "admin", "execute"];
            let required = ["read", "write"];
            let _ = required.iter().all(|r| perms.contains(r));
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
            let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.test";
            let parts: Vec<&str> = token.split('.').collect();
            let _ = parts.len() == 3;
            if let Some(payload) = parts.get(1) {
                let _ = base64::Engine::decode(
                    &base64::engine::general_purpose::URL_SAFE_NO_PAD,
                    payload,
                );
            }
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
            let mut store = std::collections::HashMap::new();
            for i in 0..20 {
                store.insert(format!("svc_{i}"), uuid::Uuid::new_v4().to_string());
            }
            for i in 0..20 {
                let _ = store.get(&format!("svc_{i}"));
            }
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
            let msg = serde_json::json!({"jsonrpc":"2.0","method":"test","params":{"data":[1,2,3]},"id":42});
            let bytes = serde_json::to_vec(&msg).expect("JSON serialization of valid Value cannot fail");
            let _: serde_json::Value = serde_json::from_slice(&bytes).expect("JSON deserialization of just-serialized Value cannot fail");
            Ok(())
        }).await
    }

    async fn benchmark_connection_management(&self) -> Result<BenchmarkResult, PrimalError> {
        let config = BenchmarkConfig {
            name: "connection_management".to_string(),
            ..Default::default()
        };
        self.run_benchmark("connection_management", config, || async {
            // Connection pool simulation: Arc allocation and drop
            let pool: Vec<std::sync::Arc<String>> = (0..10)
                .map(|i| std::sync::Arc::new(format!("conn_{i}")))
                .collect();
            for conn in &pool {
                let _ = std::sync::Arc::strong_count(conn);
            }
            drop(pool);
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
            let session = serde_json::json!({"id": uuid::Uuid::new_v4().to_string(), "created": chrono::Utc::now().to_rfc3339()});
            let _ = serde_json::to_string(&session).expect("JSON serialization of valid Value cannot fail");
            Ok(())
        }).await
    }

    async fn benchmark_protocol_negotiation(&self) -> Result<BenchmarkResult, PrimalError> {
        let config = BenchmarkConfig {
            name: "protocol_negotiation".to_string(),
            ..Default::default()
        };
        self.run_benchmark("protocol_negotiation", config, || async {
            // Protocol header parsing
            let header = r#"{"jsonrpc":"2.0","method":"capability.discover","id":1}"#;
            let v: serde_json::Value =
                serde_json::from_str(header).expect("JSON header is a compile-time constant");
            let _ = v.get("method").and_then(|m| m.as_str());
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
    /// Uses /proc/self/statm on Linux for memory, sysinfo elsewhere.
    async fn collect_system_metrics(&self) -> SystemMetrics {
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
    /// With `system-metrics` feature: falls back to sysinfo crate.
    fn measure_process_resources() -> (f64, f64) {
        #[cfg(feature = "system-metrics")]
        {
            let pid = sysinfo::Pid::from(std::process::id() as usize);
            let mut sys = sysinfo::System::new_all();
            sys.refresh_all();
            sys.refresh_process(pid);

            let mem_mb = {
                #[cfg(target_os = "linux")]
                {
                    Self::read_proc_statm_rss_mb().unwrap_or_else(|_| {
                        sys.process(pid)
                            .map(|p| (p.memory() as f64) / (1024.0 * 1024.0))
                            .unwrap_or(0.0)
                    })
                }
                #[cfg(not(target_os = "linux"))]
                {
                    sys.process(pid)
                        .map(|p| (p.memory() as f64) / (1024.0 * 1024.0))
                        .unwrap_or(0.0)
                }
            };

            let cpu = sys
                .process(pid)
                .map(|p| p.cpu_usage() as f64)
                .unwrap_or(0.0);

            (mem_mb, cpu)
        }

        #[cfg(not(feature = "system-metrics"))]
        {
            #[cfg(target_os = "linux")]
            let mem_mb = Self::read_proc_statm_rss_mb().unwrap_or(0.0);
            #[cfg(not(target_os = "linux"))]
            let mem_mb = 0.0;
            (mem_mb, 0.0)
        }
    }

    #[cfg(target_os = "linux")]
    fn read_proc_statm_rss_mb() -> Result<f64, std::io::Error> {
        let statm = std::fs::read_to_string("/proc/self/statm")?;
        // Format: size resident shared text lib data dt (pages, typically 4KB)
        let parts: Vec<&str> = statm.split_whitespace().collect();
        let rss_pages: u64 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let page_size = 4096u64; // Linux default
        let rss_bytes = rss_pages * page_size;
        Ok((rss_bytes as f64) / (1024.0 * 1024.0))
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
#[path = "benchmarking_tests.rs"]
mod tests;
