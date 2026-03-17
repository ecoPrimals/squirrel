// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use tokio::test;

#[test]
async fn test_benchmark_suite_creation() {
    let suite = BenchmarkSuite::new("test_suite");
    assert!(suite.suite_id().contains("test_suite"));
}

#[test]
async fn test_benchmark_config_default() {
    let config = BenchmarkConfig::default();
    assert_eq!(config.name, "default_benchmark");
    assert_eq!(config.duration, Duration::from_millis(100));
}

#[test]
async fn test_benchmark_config_production() {
    let config = BenchmarkConfig::production();
    assert_eq!(config.name, "production_benchmark");
    assert_eq!(config.duration, Duration::from_secs(30));
    assert_eq!(config.operation_count, 1000);
}

#[tokio::test]
async fn test_benchmark_execution() {
    let suite = BenchmarkSuite::new("test_execution");
    let config = BenchmarkConfig::default();

    let result = suite
        .run_benchmark("test_operation", config, || async {
            // Actual work instead of sleep
            let _ = serde_json::json!({"op": "test"});
            Ok(())
        })
        .await;

    assert!(result.is_ok());
    let benchmark_result = result.unwrap();
    assert_eq!(benchmark_result.operation_name, "test_operation");
    assert!(benchmark_result.ops_per_second > 0.0);
}

#[tokio::test]
async fn test_system_metrics_collection() {
    let suite = BenchmarkSuite::new("test_metrics");
    let metrics = suite.collect_system_metrics().await;

    assert!(metrics.memory_usage_mb >= 0.0);
    assert!(metrics.cpu_usage_percent >= 0.0);
}

#[tokio::test]
async fn test_benchmark_report_generation() {
    let suite = BenchmarkSuite::new("test_report");

    // Run a simple benchmark
    let _result = suite
        .run_benchmark("test_op", BenchmarkConfig::default(), || async { Ok(()) })
        .await;

    let results = suite.get_results().await;
    assert!(!results.is_empty());
}

#[tokio::test]
async fn test_benchmark_ai_intelligence() {
    let suite = BenchmarkSuite::new("test_ai");
    let results = suite.benchmark_ai_intelligence().await.unwrap();
    assert_eq!(results.len(), 4);
    assert!(results.iter().all(|r| r.success_rate >= 0.0));
}

#[tokio::test]
async fn test_benchmark_orchestration() {
    let suite = BenchmarkSuite::new("test_orch");
    let results = suite.benchmark_orchestration().await.unwrap();
    assert_eq!(results.len(), 4);
}

#[tokio::test]
async fn test_benchmark_mcp_protocol() {
    let suite = BenchmarkSuite::new("test_mcp");
    let results = suite.benchmark_mcp_protocol().await.unwrap();
    assert_eq!(results.len(), 4);
}

#[tokio::test]
async fn test_run_complete_benchmark_suite() {
    let suite = BenchmarkSuite::new("test_complete");
    let report = suite.run_complete_benchmark_suite().await.unwrap();
    assert!(report.total_benchmarks >= 20);
    assert!(!report.results.is_empty());
}

#[tokio::test]
async fn test_benchmark_suite_report_summary() {
    let suite = BenchmarkSuite::new("test_summary");
    let _ = suite.benchmark_ai_intelligence().await.unwrap();
    let results = suite.get_results().await;
    let report = BenchmarkSuiteReport {
        suite_id: "test_summary_123".to_string(),
        total_duration: Duration::from_secs(1),
        total_benchmarks: results.len(),
        results,
        system_metrics: SystemMetrics::default(),
        timestamp: Utc::now(),
    };
    let summary = report.generate_summary();
    assert!(summary.total_benchmarks > 0);
    assert!(summary.average_ops_per_second >= 0.0);
}

#[tokio::test]
async fn test_initialize_benchmarking() {
    let suite = initialize_benchmarking().await.unwrap();
    let _ = suite.benchmark_ai_intelligence().await.unwrap();
    assert!(!suite.get_results().await.is_empty());
}

#[tokio::test]
async fn test_run_ecosystem_benchmarks() {
    let report = run_ecosystem_benchmarks().await.unwrap();
    assert!(!report.results.is_empty());
    assert!(report.total_benchmarks >= 20);
}

#[tokio::test]
async fn test_clear_results() {
    let suite = BenchmarkSuite::new("test_clear");
    let _ = suite.benchmark_ai_intelligence().await.unwrap();
    assert!(!suite.get_results().await.is_empty());
    suite.clear_results().await;
    assert!(suite.get_results().await.is_empty());
}

#[tokio::test]
async fn test_benchmark_result_serde() {
    let result = BenchmarkResult {
        benchmark_id: "id-1".to_string(),
        operation_name: "test".to_string(),
        duration_ms: 100.0,
        ops_per_second: 10.0,
        operations_count: 100,
        concurrency_level: 1,
        memory_usage_mb: 64.0,
        cpu_usage_percent: 50.0,
        success_rate: 1.0,
        timestamp: Utc::now(),
        metadata: HashMap::new(),
        provenance: universal_patterns::provenance::Provenance::auto(),
    };
    let json = serde_json::to_string(&result).unwrap();
    let _: BenchmarkResult = serde_json::from_str(&json).unwrap();
}

#[tokio::test]
async fn test_system_metrics_default() {
    let m = SystemMetrics::default();
    assert_eq!(m.memory_usage_mb, 0.0);
    assert_eq!(m.cpu_usage_percent, 0.0);
}
