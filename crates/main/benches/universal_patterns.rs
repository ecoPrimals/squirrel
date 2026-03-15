// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Universal Patterns Framework Performance Benchmarks
//!
//! This benchmark suite tests the performance of Universal Patterns components:
//! - Configuration building and validation
//! - Security provider operations
//! - Orchestration provider operations
//! - Trait implementations and type conversions
//! - Memory usage and resource management

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::time::Duration;
use tokio::runtime::Runtime;
use uuid::Uuid;

use universal_patterns::config::ConfigBuilder;
use universal_patterns::orchestration::MockOrchestrationProvider;
use universal_patterns::security::BeardogIntegration;
use universal_patterns::traits::{
    AuthResult, Credentials, HealthReport, HealthStatus, PrimalInfo, PrimalState, Principal,
    TaskInfo, TaskStatus,
};

/// Benchmark configuration building operations
fn bench_configuration_building(c: &mut Criterion) {
    let mut group = c.benchmark_group("configuration_building");

    // Benchmark basic configuration creation
    group.bench_function("create_basic_config", |b| {
        b.iter(|| {
            let config = ConfigBuilder::new().development().build().unwrap();
            black_box(config)
        })
    });

    // Benchmark squirrel configuration
    group.bench_function("create_squirrel_config", |b| {
        b.iter(|| {
            let config = ConfigBuilder::new()
                .squirrel()
                .with_endpoint("http://localhost:8080")
                .build()
                .unwrap();
            black_box(config)
        })
    });

    // Benchmark beardog configuration
    group.bench_function("create_beardog_config", |b| {
        b.iter(|| {
            let config = ConfigBuilder::new()
                .beardog()
                .with_endpoint("http://localhost:8443")
                .build()
                .unwrap();
            black_box(config)
        })
    });

    // Benchmark songbird configuration
    group.bench_function("create_songbird_config", |b| {
        b.iter(|| {
            let config = ConfigBuilder::new()
                .songbird()
                .with_orchestration_enabled(true)
                .with_endpoint("http://localhost:8082")
                .build()
                .unwrap();
            black_box(config)
        })
    });

    // Benchmark production configuration
    group.bench_function("create_production_config", |b| {
        b.iter(|| {
            let config = ConfigBuilder::new()
                .production()
                .with_environment("production")
                .build()
                .unwrap();
            black_box(config)
        })
    });

    group.finish();
}

/// Benchmark security provider operations
fn bench_security_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("security_operations");
    let rt = Runtime::new().unwrap();

    // Benchmark authentication
    group.bench_function("authenticate_user", |b| {
        let config = ConfigBuilder::new()
            .beardog()
            .with_endpoint("http://localhost:8443")
            .build()
            .unwrap();

        let security = BeardogIntegration::new(config);

        let credentials = Credentials {
            username: "benchmark_user".to_string(),
            password: "benchmark_pass".to_string(),
            token: Some("benchmark_token".to_string()),
            metadata: std::collections::HashMap::new(),
        };

        b.to_async(&rt)
            .iter(|| async { black_box(security.authenticate(credentials.clone()).await.unwrap()) })
    });

    // Benchmark authorization
    group.bench_function("authorize_user", |b| {
        let config = ConfigBuilder::new()
            .beardog()
            .with_endpoint("http://localhost:8443")
            .build()
            .unwrap();

        let security = BeardogIntegration::new(config);

        let principal = Principal {
            id: "benchmark_user".to_string(),
            name: "Benchmark User".to_string(),
            roles: vec!["user".to_string()],
            permissions: vec!["read".to_string()],
            metadata: std::collections::HashMap::new(),
        };

        b.to_async(&rt).iter(|| async {
            black_box(security.authorize(principal.clone(), "read").await.unwrap())
        })
    });

    // Benchmark token validation
    group.bench_function("validate_token", |b| {
        let config = ConfigBuilder::new()
            .beardog()
            .with_endpoint("http://localhost:8443")
            .build()
            .unwrap();

        let security = BeardogIntegration::new(config);

        b.to_async(&rt)
            .iter(|| async { black_box(security.validate_token("benchmark_token").await.unwrap()) })
    });

    group.finish();
}

/// Benchmark orchestration provider operations
fn bench_orchestration_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("orchestration_operations");
    let rt = Runtime::new().unwrap();

    // Benchmark task scheduling
    group.bench_function("schedule_task", |b| {
        let provider = MockOrchestrationProvider::new();

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

            black_box(provider.schedule_task(task).await.unwrap())
        })
    });

    // Benchmark service discovery
    group.bench_function("discover_services", |b| {
        let provider = MockOrchestrationProvider::new();

        b.to_async(&rt)
            .iter(|| async { black_box(provider.discover_services().await.unwrap()) })
    });

    // Benchmark health reporting
    group.bench_function("report_health", |b| {
        let provider = MockOrchestrationProvider::new();

        b.to_async(&rt)
            .iter(|| async { black_box(provider.report_health().await.unwrap()) })
    });

    group.finish();
}

/// Benchmark trait implementations and type conversions
fn bench_trait_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("trait_operations");

    // Benchmark PrimalInfo creation
    group.bench_function("create_primal_info", |b| {
        b.iter(|| {
            let info = PrimalInfo {
                name: "benchmark_primal".to_string(),
                version: "1.0.0".to_string(),
                endpoint: "http://localhost:8999".to_string(),
                capabilities: vec!["benchmark".to_string()],
                metadata: std::collections::HashMap::new(),
            };
            black_box(info)
        })
    });

    // Benchmark PrimalState creation
    group.bench_function("create_primal_state", |b| {
        b.iter(|| {
            let state = PrimalState {
                name: "benchmark_primal".to_string(),
                status: "active".to_string(),
                last_heartbeat: chrono::Utc::now(),
                metadata: std::collections::HashMap::new(),
            };
            black_box(state)
        })
    });

    // Benchmark TaskInfo creation
    group.bench_function("create_task_info", |b| {
        b.iter(|| {
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
            black_box(task)
        })
    });

    // Benchmark HealthReport creation
    group.bench_function("create_health_report", |b| {
        b.iter(|| {
            let report = HealthReport {
                service_name: "benchmark_service".to_string(),
                status: HealthStatus::Healthy,
                timestamp: chrono::Utc::now(),
                details: std::collections::HashMap::new(),
                dependencies: vec![],
            };
            black_box(report)
        })
    });

    group.finish();
}

/// Benchmark concurrent operations
fn bench_concurrent_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_patterns");
    let rt = Runtime::new().unwrap();

    // Benchmark concurrent configuration building
    group.bench_function("concurrent_config_building", |b| {
        b.to_async(&rt).iter(|| async {
            let mut handles = Vec::new();

            for i in 0..10 {
                handles.push(tokio::spawn(async move {
                    let config = ConfigBuilder::new()
                        .development()
                        .with_environment(&format!("test_{}", i))
                        .build()
                        .unwrap();
                    black_box(config)
                }));
            }

            for handle in handles {
                black_box(handle.await.unwrap());
            }
        })
    });

    // Benchmark concurrent security operations
    group.bench_function("concurrent_security_operations", |b| {
        let config = ConfigBuilder::new()
            .beardog()
            .with_endpoint("http://localhost:8443")
            .build()
            .unwrap();

        let security = BeardogIntegration::new(config);

        b.to_async(&rt).iter(|| async {
            let mut handles = Vec::new();

            for i in 0..5 {
                let security_clone = security.clone();
                let credentials = Credentials {
                    username: format!("user_{}", i),
                    password: "password".to_string(),
                    token: Some(format!("token_{}", i)),
                    metadata: std::collections::HashMap::new(),
                };

                handles.push(tokio::spawn(async move {
                    security_clone.authenticate(credentials).await
                }));
            }

            for handle in handles {
                black_box(handle.await.unwrap().unwrap());
            }
        })
    });

    // Benchmark concurrent orchestration operations
    group.bench_function("concurrent_orchestration_operations", |b| {
        let provider = MockOrchestrationProvider::new();

        b.to_async(&rt).iter(|| async {
            let mut handles = Vec::new();

            for i in 0..8 {
                let provider_clone = provider.clone();

                if i % 2 == 0 {
                    handles.push(tokio::spawn(async move {
                        provider_clone.discover_services().await.map(|_| ())
                    }));
                } else {
                    handles.push(tokio::spawn(async move {
                        provider_clone.report_health().await.map(|_| ())
                    }));
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
fn bench_memory_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_operations");

    // Benchmark large configuration creation
    group.bench_function("large_config_creation", |b| {
        b.iter(|| {
            let mut config = ConfigBuilder::new()
                .development()
                .with_environment("benchmark");

            // Add many configuration options
            for i in 0..100 {
                config = config.with_property(&format!("key_{}", i), &format!("value_{}", i));
            }

            black_box(config.build().unwrap())
        })
    });

    // Benchmark large metadata creation
    group.bench_function("large_metadata_creation", |b| {
        b.iter(|| {
            let mut metadata = std::collections::HashMap::new();
            for i in 0..500 {
                metadata.insert(format!("key_{}", i), format!("value_{}", i));
            }
            black_box(metadata)
        })
    });

    // Benchmark UUID generation at scale
    group.bench_function("uuid_generation_scale", |b| {
        b.iter(|| {
            let mut ids = Vec::new();
            for _ in 0..1000 {
                ids.push(Uuid::new_v4().to_string());
            }
            black_box(ids)
        })
    });

    group.finish();
}

/// Benchmark configuration validation
fn bench_configuration_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("configuration_validation");

    // Benchmark valid configuration
    group.bench_function("validate_valid_config", |b| {
        b.iter(|| {
            let config = ConfigBuilder::new()
                .production()
                .with_endpoint("https://api.example.com")
                .with_environment("production")
                .build()
                .unwrap();
            black_box(config)
        })
    });

    // Benchmark configuration with many properties
    group.bench_function("validate_complex_config", |b| {
        b.iter(|| {
            let mut config = ConfigBuilder::new()
                .production()
                .with_endpoint("https://api.example.com")
                .with_environment("production");

            // Add many properties to test validation performance
            for i in 0..50 {
                config = config.with_property(&format!("config_{}", i), &format!("value_{}", i));
            }

            black_box(config.build().unwrap())
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_configuration_building,
    bench_security_operations,
    bench_orchestration_operations,
    bench_trait_operations,
    bench_concurrent_patterns,
    bench_memory_operations,
    bench_configuration_validation
);

criterion_main!(benches);
