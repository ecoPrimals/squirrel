//! Songbird Orchestration Performance Benchmarks
//!
//! This benchmark suite tests the performance of Songbird orchestration components:
//! - Task management (scheduling, execution, cancellation)
//! - Service discovery and health monitoring
//! - Cluster coordination and state management
//! - Performance under various load conditions

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;
use tokio::runtime::Runtime;
use uuid::Uuid;

use squirrel::songbird::SongbirdIntegration;
use universal_patterns::traits::{PrimalInfo, PrimalState, TaskInfo, TaskStatus};

/// Setup test orchestration service with mock configuration
fn setup_test_service() -> SongbirdIntegration {
    SongbirdIntegration::new()
}

/// Benchmark task management operations
fn bench_task_management(c: &mut Criterion) {
    let mut group = c.benchmark_group("task_management");
    let rt = Runtime::new().unwrap();

    // Benchmark task scheduling
    group.bench_function("schedule_task", |b| {
        let service = setup_test_service();

        b.to_async(&rt).iter(|| async {
            let task = TaskInfo {
                id: Uuid::new_v4().to_string(),
                name: "benchmark_task".to_string(),
                priority: 1,
                estimated_duration: Duration::from_secs(30),
                dependencies: vec![],
                resource_requirements: std::collections::HashMap::new(),
                metadata: std::collections::HashMap::new(),
                status: TaskStatus::Pending,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };

            black_box(service.schedule_task(task).await.unwrap())
        })
    });

    // Benchmark task status checking
    group.bench_function("check_task_status", |b| {
        let service = setup_test_service();
        let task_id = "benchmark_task_status".to_string();

        b.to_async(&rt)
            .iter(|| async { black_box(service.get_task_status(&task_id).await.unwrap()) })
    });

    // Benchmark task cancellation
    group.bench_function("cancel_task", |b| {
        let service = setup_test_service();
        let task_id = "benchmark_task_cancel".to_string();

        b.to_async(&rt)
            .iter(|| async { black_box(service.cancel_task(&task_id).await.unwrap()) })
    });

    group.finish();
}

/// Benchmark service discovery operations
fn bench_service_discovery(c: &mut Criterion) {
    let mut group = c.benchmark_group("service_discovery");
    let rt = Runtime::new().unwrap();

    // Benchmark service discovery
    group.bench_function("discover_services", |b| {
        let service = setup_test_service();

        b.to_async(&rt)
            .iter(|| async { black_box(service.discover_services().await.unwrap()) })
    });

    // Benchmark service registration
    group.bench_function("register_service", |b| {
        let service = setup_test_service();

        b.to_async(&rt).iter(|| async {
            let primal_info = PrimalInfo {
                name: "benchmark_service".to_string(),
                version: "1.0.0".to_string(),
                endpoint: "http://localhost:8999".to_string(),
                capabilities: vec!["benchmark".to_string()],
                metadata: std::collections::HashMap::new(),
            };

            black_box(service.register_service(primal_info).await.unwrap())
        })
    });

    group.finish();
}

/// Benchmark health monitoring operations
fn bench_health_monitoring(c: &mut Criterion) {
    let mut group = c.benchmark_group("health_monitoring");
    let rt = Runtime::new().unwrap();

    // Benchmark health report generation
    group.bench_function("generate_health_report", |b| {
        let service = setup_test_service();

        b.to_async(&rt)
            .iter(|| async { black_box(service.report_health().await.unwrap()) })
    });

    // Benchmark health check for specific service
    group.bench_function("check_service_health", |b| {
        let service = setup_test_service();
        let service_name = "benchmark_service".to_string();

        b.to_async(&rt).iter(|| async {
            black_box(service.check_service_health(&service_name).await.unwrap())
        })
    });

    group.finish();
}

/// Benchmark cluster coordination operations
fn bench_cluster_coordination(c: &mut Criterion) {
    let mut group = c.benchmark_group("cluster_coordination");
    let rt = Runtime::new().unwrap();

    // Benchmark cluster status retrieval
    group.bench_function("get_cluster_status", |b| {
        let service = setup_test_service();

        b.to_async(&rt)
            .iter(|| async { black_box(service.get_cluster_status().await.unwrap()) })
    });

    // Benchmark state management
    group.bench_function("manage_primal_state", |b| {
        let service = setup_test_service();

        b.to_async(&rt).iter(|| async {
            let new_state = PrimalState {
                name: "benchmark_primal".to_string(),
                status: "active".to_string(),
                last_heartbeat: chrono::Utc::now(),
                metadata: std::collections::HashMap::new(),
            };

            black_box(service.manage_primal_state(new_state).await.unwrap())
        })
    });

    group.finish();
}

/// Benchmark concurrent operations
fn bench_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");
    let rt = Runtime::new().unwrap();

    // Benchmark concurrent task scheduling
    group.bench_function("concurrent_task_scheduling", |b| {
        let service = setup_test_service();

        b.to_async(&rt).iter(|| async {
            let mut handles = Vec::new();

            for i in 0..10 {
                let service_clone = service.clone();
                let task = TaskInfo {
                    id: format!("concurrent_task_{}", i),
                    name: format!("benchmark_task_{}", i),
                    priority: 1,
                    estimated_duration: Duration::from_secs(30),
                    dependencies: vec![],
                    resource_requirements: std::collections::HashMap::new(),
                    metadata: std::collections::HashMap::new(),
                    status: TaskStatus::Pending,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                };

                handles.push(tokio::spawn(async move {
                    service_clone.schedule_task(task).await
                }));
            }

            for handle in handles {
                black_box(handle.await.unwrap().unwrap());
            }
        })
    });

    // Benchmark concurrent service discovery
    group.bench_function("concurrent_service_discovery", |b| {
        let service = setup_test_service();

        b.to_async(&rt).iter(|| async {
            let mut handles = Vec::new();

            for _ in 0..5 {
                let service_clone = service.clone();

                handles.push(tokio::spawn(async move {
                    service_clone.discover_services().await
                }));
            }

            for handle in handles {
                black_box(handle.await.unwrap().unwrap());
            }
        })
    });

    // Benchmark concurrent health monitoring
    group.bench_function("concurrent_health_monitoring", |b| {
        let service = setup_test_service();

        b.to_async(&rt).iter(|| async {
            let mut handles = Vec::new();

            for _ in 0..8 {
                let service_clone = service.clone();

                handles.push(tokio::spawn(
                    async move { service_clone.report_health().await },
                ));
            }

            for handle in handles {
                black_box(handle.await.unwrap().unwrap());
            }
        })
    });

    group.finish();
}

/// Benchmark load testing scenarios
fn bench_load_testing(c: &mut Criterion) {
    let mut group = c.benchmark_group("load_testing");
    let rt = Runtime::new().unwrap();

    // Benchmark high-volume task scheduling
    group.bench_function("high_volume_task_scheduling", |b| {
        let service = setup_test_service();

        b.to_async(&rt).iter(|| async {
            let mut handles = Vec::new();

            for i in 0..100 {
                let service_clone = service.clone();
                let task = TaskInfo {
                    id: format!("load_test_task_{}", i),
                    name: format!("load_test_{}", i),
                    priority: i % 5 + 1,
                    estimated_duration: Duration::from_millis(100 + (i * 10) as u64),
                    dependencies: vec![],
                    resource_requirements: std::collections::HashMap::new(),
                    metadata: std::collections::HashMap::new(),
                    status: TaskStatus::Pending,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                };

                handles.push(tokio::spawn(async move {
                    service_clone.schedule_task(task).await
                }));
            }

            for handle in handles {
                black_box(handle.await.unwrap().unwrap());
            }
        })
    });

    // Benchmark stress testing with mixed operations
    group.bench_function("mixed_operations_stress", |b| {
        let service = setup_test_service();

        b.to_async(&rt).iter(|| async {
            let mut handles = Vec::new();

            // Mix of different operations
            for i in 0..50 {
                let service_clone = service.clone();

                match i % 4 {
                    0 => {
                        // Schedule task
                        let task = TaskInfo {
                            id: format!("stress_task_{}", i),
                            name: format!("stress_test_{}", i),
                            priority: 1,
                            estimated_duration: Duration::from_secs(30),
                            dependencies: vec![],
                            resource_requirements: std::collections::HashMap::new(),
                            metadata: std::collections::HashMap::new(),
                            status: TaskStatus::Pending,
                            created_at: chrono::Utc::now(),
                            updated_at: chrono::Utc::now(),
                        };

                        handles.push(tokio::spawn(async move {
                            service_clone.schedule_task(task).await.map(|_| ())
                        }));
                    }
                    1 => {
                        // Discover services
                        handles.push(tokio::spawn(async move {
                            service_clone.discover_services().await.map(|_| ())
                        }));
                    }
                    2 => {
                        // Report health
                        handles.push(tokio::spawn(async move {
                            service_clone.report_health().await.map(|_| ())
                        }));
                    }
                    3 => {
                        // Get cluster status
                        handles.push(tokio::spawn(async move {
                            service_clone.get_cluster_status().await.map(|_| ())
                        }));
                    }
                    _ => unreachable!(),
                }
            }

            for handle in handles {
                black_box(handle.await.unwrap().unwrap());
            }
        })
    });

    group.finish();
}

/// Benchmark memory and resource usage
fn bench_resource_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("resource_usage");
    let rt = Runtime::new().unwrap();

    // Benchmark memory usage with many tasks
    group.bench_function("memory_usage_many_tasks", |b| {
        let service = setup_test_service();

        b.to_async(&rt).iter(|| async {
            let mut tasks = Vec::new();

            for i in 0..1000 {
                let task = TaskInfo {
                    id: format!("memory_task_{}", i),
                    name: format!("memory_test_{}", i),
                    priority: 1,
                    estimated_duration: Duration::from_secs(30),
                    dependencies: vec![],
                    resource_requirements: std::collections::HashMap::new(),
                    metadata: std::collections::HashMap::new(),
                    status: TaskStatus::Pending,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                };

                tasks.push(task);
            }

            black_box(tasks)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_task_management,
    bench_service_discovery,
    bench_health_monitoring,
    bench_cluster_coordination,
    bench_concurrent_operations,
    bench_load_testing,
    bench_resource_usage
);

criterion_main!(benches);
