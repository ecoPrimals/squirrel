// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use squirrel_config::{NetworkConfig, Config, ConfigError, ConfigValidator};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use serde_json::json;
use std::collections::HashMap;

/// Mock network configuration for benchmarking
#[derive(Clone)]
struct MockNetworkContext {
    configs: Vec<NetworkConfig>,
    endpoints: HashMap<String, String>,
    ports: Vec<u16>,
}

impl MockNetworkContext {
    fn new(config_count: usize) -> Self {
        let mut configs = Vec::new();
        let mut endpoints = HashMap::new();
        let mut ports = Vec::new();
        
        for i in 0..config_count {
            let base_port = 8000 + i as u16;
            
            let config = NetworkConfig {
                host: format!("service-{}.local", i),
                port: base_port,
                protocol: if i % 2 == 0 { "http".to_string() } else { "https".to_string() },
                timeout: Duration::from_secs(30),
                max_connections: 100,
                keep_alive: true,
                compression: i % 3 == 0,
                tls_enabled: i % 2 == 1,
                endpoints: vec![
                    format!("/api/v1/service-{}", i),
                    format!("/health"),
                    format!("/metrics"),
                ],
            };
            
            endpoints.insert(format!("service-{}", i), format!("{}:{}", config.host, config.port));
            ports.push(base_port);
            configs.push(config);
        }
        
        Self { configs, endpoints, ports }
    }
}

/// Benchmark network configuration loading
fn benchmark_config_loading(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("config_loading");
    
    // Test different configuration sizes
    for config_count in [10, 50, 200, 1000].iter() {
        group.throughput(Throughput::Elements(*config_count as u64));
        
        group.bench_with_input(
            BenchmarkId::new("load_network_configs", config_count),
            config_count,
            |b, &config_count| {
                b.to_async(&rt).iter(|| async {
                    let context = MockNetworkContext::new(config_count);
                    
                    // Simulate loading configurations from various sources
                    for config in &context.configs {
                        // Serialize and deserialize to simulate file I/O
                        let serialized = serde_json::to_string(config).unwrap();
                        let _deserialized: NetworkConfig = serde_json::from_str(&serialized).unwrap();
                        
                        // Simulate validation
                        let _is_valid = validate_network_config(config).await;
                        
                        black_box(config);
                    }
                    
                    black_box(context.configs.len());
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark configuration validation
fn benchmark_config_validation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("config_validation");
    
    // Test validation performance with different complexity
    for complexity in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("validate_configs", complexity),
            complexity,
            |b, &complexity| {
                b.to_async(&rt).iter(|| async {
                    let context = MockNetworkContext::new(100);
                    let validator = ConfigValidator::new();
                    
                    for config in &context.configs {
                        // Run multiple validation passes to increase complexity
                        for _ in 0..complexity {
                            let validation_result = validator.validate_network_config(config).await;
                            black_box(validation_result);
                        }
                    }
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark endpoint resolution performance
fn benchmark_endpoint_resolution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("endpoint_resolution");
    
    // Test different endpoint counts
    for endpoint_count in [100, 500, 2000, 10000].iter() {
        group.throughput(Throughput::Elements(*endpoint_count as u64));
        
        group.bench_with_input(
            BenchmarkId::new("resolve_endpoints", endpoint_count),
            endpoint_count,
            |b, &endpoint_count| {
                b.to_async(&rt).iter(|| async {
                    let context = MockNetworkContext::new(endpoint_count);
                    
                    // Simulate endpoint resolution
                    for _ in 0..100 {
                        let service_name = format!("service-{}", rand::random::<usize>() % endpoint_count);
                        let resolved = context.endpoints.get(&service_name);
                        black_box(resolved);
                    }
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark concurrent configuration access
fn benchmark_concurrent_config_access(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("concurrent_config_access");
    
    // Test different concurrency levels
    for concurrency in [10, 50, 100, 200].iter() {
        group.throughput(Throughput::Elements(*concurrency as u64));
        
        group.bench_with_input(
            BenchmarkId::new("concurrent_access", concurrency),
            concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    let context = Arc::new(MockNetworkContext::new(1000));
                    let mut handles = Vec::new();
                    
                    // Create concurrent tasks accessing configuration
                    for i in 0..concurrency {
                        let context_clone = context.clone();
                        
                        handles.push(tokio::spawn(async move {
                            // Mix of read operations
                            for _ in 0..50 {
                                let config_index = rand::random::<usize>() % context_clone.configs.len();
                                let config = &context_clone.configs[config_index];
                                
                                // Simulate various access patterns
                                match rand::random::<u8>() % 4 {
                                    0 => {
                                        // Host lookup
                                        let _host = &config.host;
                                    },
                                    1 => {
                                        // Port lookup
                                        let _port = config.port;
                                    },
                                    2 => {
                                        // Endpoint validation
                                        let _endpoints = &config.endpoints;
                                    },
                                    _ => {
                                        // Full config serialization
                                        let _serialized = serde_json::to_string(config);
                                    }
                                }
                            }
                        }));
                    }
                    
                    // Wait for all tasks
                    for handle in handles {
                        let _ = handle.await.unwrap();
                    }
                    
                    black_box(context.configs.len());
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark configuration hot reloading
fn benchmark_config_hot_reload(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("config_hot_reload");
    
    group.bench_function("reload_configurations", |b| {
        b.to_async(&rt).iter(|| async {
            let mut context = MockNetworkContext::new(100);
            
            // Simulate configuration updates
            for i in 0..10 {
                // Update configurations
                for (idx, config) in context.configs.iter_mut().enumerate() {
                    config.port = 9000 + idx as u16 + i * 100;
                    config.max_connections = 50 + (i * 10);
                    
                    // Update endpoint mapping
                    let service_name = format!("service-{}", idx);
                    let new_endpoint = format!("{}:{}", config.host, config.port);
                    context.endpoints.insert(service_name, new_endpoint);
                }
                
                // Simulate validation of updated configs
                for config in &context.configs {
                    let _is_valid = validate_network_config(config).await;
                }
                
                black_box(&context.configs);
            }
        });
    });
    
    group.finish();
}

/// Benchmark configuration serialization/deserialization
fn benchmark_config_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_serialization");
    
    // Test different configuration sizes
    for config_count in [10, 100, 1000].iter() {
        let context = MockNetworkContext::new(*config_count);
        
        group.bench_with_input(
            BenchmarkId::new("serialize_configs", config_count),
            &context.configs,
            |b, configs| {
                b.iter(|| {
                    let serialized = black_box(serde_json::to_string(configs).unwrap());
                    black_box(serialized.len());
                });
            },
        );
        
        let serialized_configs = serde_json::to_string(&context.configs).unwrap();
        group.bench_with_input(
            BenchmarkId::new("deserialize_configs", config_count),
            &serialized_configs,
            |b, serialized| {
                b.iter(|| {
                    let deserialized: Vec<NetworkConfig> = black_box(
                        serde_json::from_str(serialized).unwrap()
                    );
                    black_box(deserialized.len());
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark environment variable override performance
fn benchmark_env_overrides(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("env_overrides");
    
    group.bench_function("apply_env_overrides", |b| {
        b.to_async(&rt).iter(|| async {
            let mut context = MockNetworkContext::new(100);
            
            // Simulate environment variable overrides
            let env_overrides = vec![
                ("SQUIRREL_HOST", "override.local"),
                ("SQUIRREL_PORT", "9999"),
                ("SQUIRREL_TIMEOUT", "45"),
                ("SQUIRREL_MAX_CONNECTIONS", "200"),
                ("SQUIRREL_TLS_ENABLED", "true"),
            ];
            
            for config in &mut context.configs {
                for (env_key, env_value) in &env_overrides {
                    match *env_key {
                        "SQUIRREL_HOST" => {
                            config.host = env_value.to_string();
                        },
                        "SQUIRREL_PORT" => {
                            if let Ok(port) = env_value.parse::<u16>() {
                                config.port = port;
                            }
                        },
                        "SQUIRREL_TIMEOUT" => {
                            if let Ok(timeout_secs) = env_value.parse::<u64>() {
                                config.timeout = Duration::from_secs(timeout_secs);
                            }
                        },
                        "SQUIRREL_MAX_CONNECTIONS" => {
                            if let Ok(max_conn) = env_value.parse::<usize>() {
                                config.max_connections = max_conn;
                            }
                        },
                        "SQUIRREL_TLS_ENABLED" => {
                            config.tls_enabled = env_value.parse::<bool>().unwrap_or(false);
                        },
                        _ => {}
                    }
                }
                
                // Re-validate after override
                let _is_valid = validate_network_config(config).await;
            }
            
            black_box(context.configs.len());
        });
    });
    
    group.finish();
}

/// Helper function for configuration validation
async fn validate_network_config(config: &NetworkConfig) -> bool {
    // Simulate validation logic
    if config.port < 1024 || config.port > 65535 {
        return false;
    }
    
    if config.host.is_empty() {
        return false;
    }
    
    if config.max_connections == 0 {
        return false;
    }
    
    if config.timeout.as_secs() == 0 {
        return false;
    }
    
    // Simulate async validation (e.g., DNS lookup)
    tokio::time::sleep(Duration::from_micros(10)).await;
    
    true
}

criterion_group!(
    benches,
    benchmark_config_loading,
    benchmark_config_validation,
    benchmark_endpoint_resolution,
    benchmark_concurrent_config_access,
    benchmark_config_hot_reload,
    benchmark_config_serialization,
    benchmark_env_overrides
);
criterion_main!(benches); 