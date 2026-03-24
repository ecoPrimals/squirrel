// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::semicolon_if_nothing_returned,
    clippy::explicit_iter_loop,
    clippy::cast_sign_loss,
    missing_docs,
    reason = "Criterion benchmarks; unwrap and style lints deferred for harness code"
)]

//! Comprehensive Ecosystem Benchmarks
//!
//! This benchmark suite provides comprehensive performance testing for the entire
//! ecoPrimals ecosystem, including all components and their interactions.

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use squirrel::benchmarking::{BenchmarkConfig, initialize_benchmarking};
use std::time::Duration;
use tokio::runtime::Runtime;

/// Benchmark the comprehensive ecosystem performance
fn ecosystem_performance_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().expect("should succeed");

    let mut group = c.benchmark_group("ecosystem_performance");
    group.measurement_time(Duration::from_secs(60));
    group.sample_size(10);

    // Initialize benchmarking framework
    let benchmark_suite =
        rt.block_on(async { initialize_benchmarking().await.expect("should succeed") });

    // AI Intelligence benchmarks
    group.bench_function("ai_intelligence_suite", |b| {
        b.to_async(&rt).iter(|| async {
            let results = benchmark_suite.benchmark_ai_intelligence().await;
            black_box(results)
        })
    });

    // Orchestration benchmarks
    group.bench_function("orchestration_suite", |b| {
        b.to_async(&rt).iter(|| async {
            let results = benchmark_suite.benchmark_orchestration().await;
            black_box(results)
        })
    });

    // Compute delegation benchmarks
    group.bench_function("compute_delegation_suite", |b| {
        b.to_async(&rt).iter(|| async {
            let results = benchmark_suite.benchmark_compute_delegation().await;
            black_box(results)
        })
    });

    // Storage benchmarks
    group.bench_function("storage_suite", |b| {
        b.to_async(&rt).iter(|| async {
            let results = benchmark_suite.benchmark_storage().await;
            black_box(results)
        })
    });

    // Security benchmarks
    group.bench_function("security_suite", |b| {
        b.to_async(&rt).iter(|| async {
            let results = benchmark_suite.benchmark_security().await;
            black_box(results)
        })
    });

    // MCP Protocol benchmarks
    group.bench_function("mcp_protocol_suite", |b| {
        b.to_async(&rt).iter(|| async {
            let results = benchmark_suite.benchmark_mcp_protocol().await;
            black_box(results)
        })
    });

    // Complete ecosystem benchmark
    group.bench_function("complete_ecosystem", |b| {
        b.to_async(&rt).iter(|| async {
            let report = benchmark_suite.run_complete_benchmark_suite().await;
            black_box(report)
        })
    });

    group.finish();
}

/// Benchmark individual operations with varying load
fn operation_scaling_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().expect("should succeed");

    let mut group = c.benchmark_group("operation_scaling");
    group.measurement_time(Duration::from_secs(30));

    let benchmark_suite =
        rt.block_on(async { initialize_benchmarking().await.expect("should succeed") });

    // Test with different operation counts
    for operation_count in [100, 500, 1000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::new("ai_processing", operation_count),
            operation_count,
            |b, &op_count| {
                b.to_async(&rt).iter(|| async {
                    let config = BenchmarkConfig {
                        name: "ai_processing_scaling".to_string(),
                        operation_count: op_count as u64,
                        ..Default::default()
                    };

                    let result = benchmark_suite
                        .run_benchmark("ai_processing", config, || async {
                            tokio::time::sleep(Duration::from_millis(10)).await;
                            Ok(())
                        })
                        .await;

                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark system under stress conditions
fn stress_testing_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().expect("should succeed");

    let mut group = c.benchmark_group("stress_testing");
    group.measurement_time(Duration::from_secs(120));
    group.sample_size(10);

    let benchmark_suite =
        rt.block_on(async { initialize_benchmarking().await.expect("should succeed") });

    // High-load stress test
    group.bench_function("high_load_stress", |b| {
        b.to_async(&rt).iter(|| async {
            let config = BenchmarkConfig {
                name: "high_load_stress".to_string(),
                operation_count: 10000,
                duration: Duration::from_secs(60),
                concurrency_levels: vec![1, 8, 16, 32, 64],
                ..Default::default()
            };

            let result = benchmark_suite
                .run_benchmark("stress_test", config, || async {
                    // Simulate high-load operation
                    tokio::time::sleep(Duration::from_millis(5)).await;
                    Ok(())
                })
                .await;

            black_box(result)
        })
    });

    // Memory stress test
    group.bench_function("memory_stress", |b| {
        b.to_async(&rt).iter(|| async {
            let config = BenchmarkConfig {
                name: "memory_stress".to_string(),
                operation_count: 5000,
                memory_monitoring: true,
                ..Default::default()
            };

            let result = benchmark_suite
                .run_benchmark("memory_stress", config, || async {
                    // Simulate memory-intensive operation
                    let _data: Vec<u8> = vec![0; 1024 * 1024]; // 1MB allocation
                    tokio::time::sleep(Duration::from_millis(1)).await;
                    Ok(())
                })
                .await;

            black_box(result)
        })
    });

    group.finish();
}

/// Benchmark concurrent operations
fn concurrency_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().expect("should succeed");

    let mut group = c.benchmark_group("concurrency");
    group.measurement_time(Duration::from_secs(45));

    let benchmark_suite =
        rt.block_on(async { initialize_benchmarking().await.expect("should succeed") });

    // Test with different concurrency levels
    for concurrency in [1, 2, 4, 8, 16, 32].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_operations", concurrency),
            concurrency,
            |b, &concurrency_level| {
                b.to_async(&rt).iter(|| async {
                    let config = BenchmarkConfig {
                        name: "concurrent_operations".to_string(),
                        concurrency_levels: vec![concurrency_level],
                        operation_count: 1000,
                        ..Default::default()
                    };

                    let result = benchmark_suite
                        .run_benchmark("concurrent_ops", config, || async {
                            // Simulate concurrent operation
                            tokio::time::sleep(Duration::from_millis(10)).await;
                            Ok(())
                        })
                        .await;

                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark resource utilization
fn resource_utilization_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().expect("should succeed");

    let mut group = c.benchmark_group("resource_utilization");
    group.measurement_time(Duration::from_secs(30));

    let benchmark_suite =
        rt.block_on(async { initialize_benchmarking().await.expect("should succeed") });

    // CPU-intensive benchmark
    group.bench_function("cpu_intensive", |b| {
        b.to_async(&rt).iter(|| async {
            let config = BenchmarkConfig {
                name: "cpu_intensive".to_string(),
                cpu_monitoring: true,
                operation_count: 1000,
                ..Default::default()
            };

            let result = benchmark_suite
                .run_benchmark("cpu_intensive", config, || async {
                    // Simulate CPU-intensive operation
                    let mut sum = 0;
                    for i in 0..1000 {
                        sum += i * i;
                    }
                    black_box(sum);
                    Ok(())
                })
                .await;

            black_box(result)
        })
    });

    // I/O-intensive benchmark
    group.bench_function("io_intensive", |b| {
        b.to_async(&rt).iter(|| async {
            let config = BenchmarkConfig {
                name: "io_intensive".to_string(),
                operation_count: 500,
                ..Default::default()
            };

            let result = benchmark_suite
                .run_benchmark("io_intensive", config, || async {
                    // Simulate I/O-intensive operation
                    tokio::time::sleep(Duration::from_millis(20)).await;
                    Ok(())
                })
                .await;

            black_box(result)
        })
    });

    group.finish();
}

/// Benchmark end-to-end workflows
fn end_to_end_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().expect("should succeed");

    let mut group = c.benchmark_group("end_to_end");
    group.measurement_time(Duration::from_secs(90));
    group.sample_size(10);

    let benchmark_suite =
        rt.block_on(async { initialize_benchmarking().await.expect("should succeed") });

    // Complete AI processing workflow
    group.bench_function("ai_processing_workflow", |b| {
        b.to_async(&rt).iter(|| async {
            let config = BenchmarkConfig {
                name: "ai_processing_workflow".to_string(),
                operation_count: 100,
                collect_detailed_metrics: true,
                ..Default::default()
            };

            let result = benchmark_suite
                .run_benchmark("ai_workflow", config, || async {
                    // Simulate complete AI processing workflow
                    tokio::time::sleep(Duration::from_millis(50)).await; // Text generation
                    tokio::time::sleep(Duration::from_millis(30)).await; // Context processing
                    tokio::time::sleep(Duration::from_millis(20)).await; // Response synthesis
                    Ok(())
                })
                .await;

            black_box(result)
        })
    });

    // Service coordination workflow
    group.bench_function("service_coordination_workflow", |b| {
        b.to_async(&rt).iter(|| async {
            let config = BenchmarkConfig {
                name: "service_coordination_workflow".to_string(),
                operation_count: 200,
                ..Default::default()
            };

            let result = benchmark_suite
                .run_benchmark("coordination_workflow", config, || async {
                    // Simulate service coordination workflow
                    tokio::time::sleep(Duration::from_millis(25)).await; // Service discovery
                    tokio::time::sleep(Duration::from_millis(15)).await; // Load balancing
                    tokio::time::sleep(Duration::from_millis(10)).await; // Health monitoring
                    Ok(())
                })
                .await;

            black_box(result)
        })
    });

    group.finish();
}

/// Benchmark system initialization and teardown
fn lifecycle_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().expect("should succeed");

    let mut group = c.benchmark_group("lifecycle");
    group.measurement_time(Duration::from_secs(30));

    // System initialization benchmark
    group.bench_function("system_initialization", |b| {
        b.to_async(&rt).iter(|| async {
            let benchmark_suite = initialize_benchmarking().await.expect("should succeed");
            black_box(benchmark_suite)
        })
    });

    // System teardown benchmark
    group.bench_function("system_teardown", |b| {
        b.to_async(&rt).iter(|| async {
            let benchmark_suite = initialize_benchmarking().await.expect("should succeed");
            // Simulate system teardown
            tokio::time::sleep(Duration::from_millis(100)).await;
            black_box(benchmark_suite)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    ecosystem_performance_benchmark,
    operation_scaling_benchmark,
    stress_testing_benchmark,
    concurrency_benchmark,
    resource_utilization_benchmark,
    end_to_end_benchmark,
    lifecycle_benchmark
);

criterion_main!(benches);
