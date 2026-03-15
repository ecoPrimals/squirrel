// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use squirrel_core::routing::{RoutingEngine, RoutingConfig, Route};
use squirrel_core::service::{ServiceMesh, ServiceRegistry, ServiceHealthCheck};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use uuid::Uuid;

/// Benchmark routing engine performance
fn benchmark_routing_engine(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("routing_engine");
    
    // Test different route counts
    for route_count in [10, 100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*route_count as u64));
        
        group.bench_with_input(
            BenchmarkId::new("route_lookup", route_count),
            route_count,
            |b, &route_count| {
                let engine = rt.block_on(async {
                    let config = RoutingConfig::default();
                    let mut engine = RoutingEngine::new(config).await.unwrap();
                    
                    // Pre-populate routes
                    for i in 0..route_count {
                        let route = Route {
                            id: format!("route-{}", i),
                            path: format!("/api/v1/test/{}", i),
                            destination: format!("http://service-{}.local:8080", i),
                            methods: vec!["GET".to_string(), "POST".to_string()],
                            priority: 1.0,
                            timeout: Duration::from_secs(30),
                            retries: 3,
                            circuit_breaker: None,
                        };
                        engine.add_route(route).await.unwrap();
                    }
                    
                    engine
                });
                
                b.to_async(&rt).iter(|| async {
                    let path = format!("/api/v1/test/{}", rand::random::<usize>() % route_count);
                    let _ = black_box(engine.find_route(&path, "GET").await);
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark service mesh performance
fn benchmark_service_mesh(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("service_mesh");
    
    // Test different service counts
    for service_count in [10, 50, 200, 1000].iter() {
        group.throughput(Throughput::Elements(*service_count as u64));
        
        group.bench_with_input(
            BenchmarkId::new("service_discovery", service_count),
            service_count,
            |b, &service_count| {
                let mesh = rt.block_on(async {
                    let mesh = ServiceMesh::new().await.unwrap();
                    
                    // Pre-populate services
                    for i in 0..service_count {
                        let service_id = format!("service-{}", i);
                        let endpoint = format!("http://service-{}.local:8080", i);
                        mesh.register_service(
                            service_id.clone(),
                            endpoint,
                            ServiceHealthCheck {
                                path: "/health".to_string(),
                                interval: Duration::from_secs(30),
                                timeout: Duration::from_secs(5),
                                retries: 3,
                            }
                        ).await.unwrap();
                    }
                    
                    mesh
                });
                
                b.to_async(&rt).iter(|| async {
                    let service_id = format!("service-{}", rand::random::<usize>() % service_count);
                    let _ = black_box(mesh.discover_service(&service_id).await);
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark concurrent operations
fn benchmark_concurrent_core_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("concurrent_core_operations");
    
    // Test different concurrency levels
    for concurrency in [1, 10, 50, 100].iter() {
        group.throughput(Throughput::Elements(*concurrency as u64));
        
        group.bench_with_input(
            BenchmarkId::new("concurrent_routing", concurrency),
            concurrency,
            |b, &concurrency| {
                let engine = rt.block_on(async {
                    let config = RoutingConfig::default();
                    let mut engine = RoutingEngine::new(config).await.unwrap();
                    
                    // Pre-populate routes
                    for i in 0..1000 {
                        let route = Route {
                            id: format!("route-{}", i),
                            path: format!("/api/v1/test/{}", i),
                            destination: format!("http://service-{}.local:8080", i),
                            methods: vec!["GET".to_string()],
                            priority: 1.0,
                            timeout: Duration::from_secs(30),
                            retries: 3,
                            circuit_breaker: None,
                        };
                        engine.add_route(route).await.unwrap();
                    }
                    
                    Arc::new(engine)
                });
                
                b.to_async(&rt).iter(|| async {
                    let mut handles = Vec::new();
                    
                    for _ in 0..concurrency {
                        let engine_clone = engine.clone();
                        handles.push(tokio::spawn(async move {
                            let path = format!("/api/v1/test/{}", rand::random::<usize>() % 1000);
                            engine_clone.find_route(&path, "GET").await
                        }));
                    }
                    
                    for handle in handles {
                        let _ = black_box(handle.await.unwrap());
                    }
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark memory allocation patterns
fn benchmark_memory_allocations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("memory_allocations");
    
    // Test route creation and cleanup
    group.bench_function("route_creation", |b| {
        b.to_async(&rt).iter(|| async {
            let config = RoutingConfig::default();
            let mut engine = RoutingEngine::new(config).await.unwrap();
            
            // Create and destroy routes rapidly
            for i in 0..100 {
                let route = Route {
                    id: format!("temp-route-{}", i),
                    path: format!("/temp/{}", i),
                    destination: format!("http://temp-{}.local:8080", i),
                    methods: vec!["GET".to_string()],
                    priority: 1.0,
                    timeout: Duration::from_secs(30),
                    retries: 3,
                    circuit_breaker: None,
                };
                engine.add_route(route).await.unwrap();
            }
            
            // Clean up
            for i in 0..100 {
                let route_id = format!("temp-route-{}", i);
                let _ = engine.remove_route(&route_id).await;
            }
            
            black_box(engine)
        });
    });
    
    group.finish();
}

/// Benchmark error handling performance
fn benchmark_error_handling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("error_handling");
    
    group.bench_function("routing_errors", |b| {
        b.to_async(&rt).iter(|| async {
            let config = RoutingConfig::default();
            let engine = RoutingEngine::new(config).await.unwrap();
            
            // Test non-existent route lookups (should return errors gracefully)
            for i in 0..100 {
                let path = format!("/nonexistent/{}", i);
                let result = engine.find_route(&path, "GET").await;
                black_box(result);
            }
        });
    });
    
    group.bench_function("service_mesh_errors", |b| {
        b.to_async(&rt).iter(|| async {
            let mesh = ServiceMesh::new().await.unwrap();
            
            // Test non-existent service lookups
            for i in 0..100 {
                let service_id = format!("nonexistent-service-{}", i);
                let result = mesh.discover_service(&service_id).await;
                black_box(result);
            }
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_routing_engine,
    benchmark_service_mesh,
    benchmark_concurrent_core_operations,
    benchmark_memory_allocations,
    benchmark_error_handling
);
criterion_main!(benches); 