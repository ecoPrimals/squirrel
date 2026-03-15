// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use squirrel_plugins::{
    PluginManager, DefaultPluginManager, Plugin, PluginMetadata, PluginError,
    PluginRegistry, PluginStatus, PluginLoader, DynamicPluginLoader
};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use uuid::Uuid;
use tempfile::TempDir;
use std::path::PathBuf;

/// Mock plugin for benchmarking
#[derive(Debug, Clone)]
struct MockPlugin {
    id: String,
    metadata: PluginMetadata,
    execution_time: Duration,
}

impl MockPlugin {
    fn new(id: String, execution_time: Duration) -> Self {
        Self {
            metadata: PluginMetadata {
                id: id.clone(),
                name: format!("Mock Plugin {}", id),
                version: "1.0.0".to_string(),
                description: "Benchmark mock plugin".to_string(),
                author: "Benchmark Suite".to_string(),
                dependencies: Vec::new(),
                capabilities: Vec::new(),
            },
            id,
            execution_time,
        }
    }
}

#[async_trait::async_trait]
impl Plugin for MockPlugin {
    fn id(&self) -> &str {
        &self.id
    }

    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn initialize(&mut self) -> Result<(), PluginError> {
        tokio::time::sleep(self.execution_time).await;
        Ok(())
    }

    async fn execute(&self, _input: serde_json::Value) -> Result<serde_json::Value, PluginError> {
        tokio::time::sleep(self.execution_time).await;
        Ok(serde_json::json!({
            "plugin_id": self.id,
            "result": "success",
            "timestamp": chrono::Utc::now()
        }))
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        tokio::time::sleep(self.execution_time / 2).await;
        Ok(())
    }
}

/// Benchmark plugin loading performance
fn benchmark_plugin_loading(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("plugin_loading");
    
    // Test different plugin counts
    for plugin_count in [10, 50, 100, 500].iter() {
        group.throughput(Throughput::Elements(*plugin_count as u64));
        
        group.bench_with_input(
            BenchmarkId::new("load_plugins", plugin_count),
            plugin_count,
            |b, &plugin_count| {
                b.to_async(&rt).iter(|| async {
                    let manager = DefaultPluginManager::new().await.unwrap();
                    
                    // Create mock plugins with minimal execution time
                    for i in 0..plugin_count {
                        let plugin_id = format!("benchmark-plugin-{}", i);
                        let plugin = MockPlugin::new(
                            plugin_id.clone(),
                            Duration::from_micros(1) // Minimal execution time
                        );
                        
                        let _ = black_box(
                            manager.register_plugin(plugin_id, Arc::new(plugin)).await
                        );
                    }
                    
                    black_box(manager);
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark plugin execution performance
fn benchmark_plugin_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("plugin_execution");
    
    // Test different execution speeds
    let execution_times = [
        Duration::from_micros(1),
        Duration::from_micros(10),
        Duration::from_micros(100),
        Duration::from_millis(1),
    ];
    
    for execution_time in execution_times.iter() {
        group.bench_with_input(
            BenchmarkId::new("execute_plugin", format!("{:?}", execution_time)),
            execution_time,
            |b, &execution_time| {
                b.to_async(&rt).iter(|| async {
                    let plugin = MockPlugin::new(
                        "benchmark-plugin".to_string(),
                        execution_time
                    );
                    
                    let input = serde_json::json!({
                        "benchmark": true,
                        "timestamp": chrono::Utc::now()
                    });
                    
                    let result = black_box(plugin.execute(input).await.unwrap());
                    black_box(result);
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark concurrent plugin execution
fn benchmark_concurrent_plugin_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("concurrent_plugin_execution");
    
    // Test different concurrency levels
    for concurrency in [1, 10, 50, 100].iter() {
        group.throughput(Throughput::Elements(*concurrency as u64));
        
        group.bench_with_input(
            BenchmarkId::new("concurrent_execution", concurrency),
            concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    let manager = DefaultPluginManager::new().await.unwrap();
                    
                    // Register plugins
                    for i in 0..concurrency {
                        let plugin_id = format!("concurrent-plugin-{}", i);
                        let plugin = MockPlugin::new(
                            plugin_id.clone(),
                            Duration::from_micros(10)
                        );
                        manager.register_plugin(plugin_id, Arc::new(plugin)).await.unwrap();
                    }
                    
                    // Execute plugins concurrently
                    let mut handles = Vec::new();
                    for i in 0..concurrency {
                        let plugin_id = format!("concurrent-plugin-{}", i);
                        let manager_clone = manager.clone();
                        
                        handles.push(tokio::spawn(async move {
                            let input = serde_json::json!({
                                "concurrent_test": true,
                                "plugin_index": i
                            });
                            manager_clone.execute_plugin(&plugin_id, input).await
                        }));
                    }
                    
                    // Wait for all executions to complete
                    for handle in handles {
                        let _ = black_box(handle.await.unwrap());
                    }
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark plugin lifecycle management
fn benchmark_plugin_lifecycle(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("plugin_lifecycle");
    
    group.bench_function("full_lifecycle", |b| {
        b.to_async(&rt).iter(|| async {
            let manager = DefaultPluginManager::new().await.unwrap();
            let plugin_id = "lifecycle-test-plugin";
            
            // Create plugin
            let plugin = MockPlugin::new(
                plugin_id.to_string(),
                Duration::from_micros(10)
            );
            
            // Full lifecycle: Register -> Initialize -> Execute -> Shutdown -> Unregister
            manager.register_plugin(plugin_id.to_string(), Arc::new(plugin)).await.unwrap();
            
            let _ = manager.initialize_plugin(plugin_id).await.unwrap();
            
            let input = serde_json::json!({"lifecycle_test": true});
            let _ = manager.execute_plugin(plugin_id, input).await.unwrap();
            
            let _ = manager.shutdown_plugin(plugin_id).await.unwrap();
            
            let _ = manager.unregister_plugin(plugin_id).await.unwrap();
            
            black_box(manager);
        });
    });
    
    group.bench_function("rapid_register_unregister", |b| {
        b.to_async(&rt).iter(|| async {
            let manager = DefaultPluginManager::new().await.unwrap();
            
            // Rapidly register and unregister plugins
            for i in 0..100 {
                let plugin_id = format!("rapid-plugin-{}", i);
                let plugin = MockPlugin::new(
                    plugin_id.clone(),
                    Duration::from_micros(1)
                );
                
                manager.register_plugin(plugin_id.clone(), Arc::new(plugin)).await.unwrap();
                manager.unregister_plugin(&plugin_id).await.unwrap();
            }
            
            black_box(manager);
        });
    });
    
    group.finish();
}

/// Benchmark plugin discovery and loading from filesystem
fn benchmark_plugin_discovery(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("plugin_discovery");
    
    // Test different plugin directory sizes
    for plugin_count in [10, 50, 100].iter() {
        group.throughput(Throughput::Elements(*plugin_count as u64));
        
        group.bench_with_input(
            BenchmarkId::new("discover_plugins", plugin_count),
            plugin_count,
            |b, &plugin_count| {
                b.to_async(&rt).iter(|| async {
                    // Create temporary directory with mock plugin files
                    let temp_dir = TempDir::new().unwrap();
                    let plugin_dir = temp_dir.path();
                    
                    // Create mock plugin files
                    for i in 0..plugin_count {
                        let plugin_path = plugin_dir.join(format!("plugin-{}.so", i));
                        std::fs::write(&plugin_path, "mock plugin binary").unwrap();
                        
                        // Create metadata file
                        let metadata_path = plugin_dir.join(format!("plugin-{}.json", i));
                        let metadata = serde_json::json!({
                            "id": format!("plugin-{}", i),
                            "name": format!("Plugin {}", i),
                            "version": "1.0.0",
                            "description": "Mock plugin for benchmarking",
                            "author": "Benchmark Suite"
                        });
                        std::fs::write(&metadata_path, metadata.to_string()).unwrap();
                    }
                    
                    // Benchmark plugin discovery
                    let loader = DynamicPluginLoader::new();
                    let discovered = loader.discover_plugins(plugin_dir).await.unwrap();
                    
                    black_box(discovered);
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark plugin registry operations
fn benchmark_plugin_registry(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("plugin_registry");
    
    // Test registry performance with different plugin counts
    for plugin_count in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*plugin_count as u64));
        
        group.bench_with_input(
            BenchmarkId::new("registry_lookup", plugin_count),
            plugin_count,
            |b, &plugin_count| {
                b.to_async(&rt).iter(|| async {
                    let registry = PluginRegistry::new();
                    
                    // Pre-populate registry
                    for i in 0..plugin_count {
                        let plugin_id = format!("registry-plugin-{}", i);
                        let plugin = MockPlugin::new(
                            plugin_id.clone(),
                            Duration::from_micros(1)
                        );
                        registry.register(plugin_id, Arc::new(plugin)).await.unwrap();
                    }
                    
                    // Benchmark lookups
                    for _ in 0..100 {
                        let plugin_id = format!("registry-plugin-{}", 
                            rand::random::<usize>() % plugin_count);
                        let _ = black_box(registry.get(&plugin_id).await);
                    }
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark plugin error handling
fn benchmark_plugin_error_handling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("plugin_error_handling");
    
    // Mock plugin that always fails
    #[derive(Debug, Clone)]
    struct FailingPlugin {
        id: String,
    }
    
    #[async_trait::async_trait]
    impl Plugin for FailingPlugin {
        fn id(&self) -> &str {
            &self.id
        }

        fn metadata(&self) -> &PluginMetadata {
            // Return minimal metadata
            &PluginMetadata {
                id: self.id.clone(),
                name: "Failing Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "Always fails for benchmarking".to_string(),
                author: "Benchmark Suite".to_string(),
                dependencies: Vec::new(),
                capabilities: Vec::new(),
            }
        }

        async fn initialize(&mut self) -> Result<(), PluginError> {
            Err(PluginError::InitializationFailed("Benchmark failure".to_string()))
        }

        async fn execute(&self, _input: serde_json::Value) -> Result<serde_json::Value, PluginError> {
            Err(PluginError::ExecutionFailed("Benchmark failure".to_string()))
        }

        async fn shutdown(&mut self) -> Result<(), PluginError> {
            Err(PluginError::ShutdownFailed("Benchmark failure".to_string()))
        }
    }
    
    group.bench_function("handle_plugin_failures", |b| {
        b.to_async(&rt).iter(|| async {
            let manager = DefaultPluginManager::new().await.unwrap();
            
            // Register failing plugins and handle errors
            for i in 0..50 {
                let plugin_id = format!("failing-plugin-{}", i);
                let plugin = FailingPlugin { id: plugin_id.clone() };
                
                let _ = manager.register_plugin(plugin_id.clone(), Arc::new(plugin)).await;
                
                // Try to initialize (will fail)
                let init_result = manager.initialize_plugin(&plugin_id).await;
                black_box(init_result);
                
                // Try to execute (will fail)
                let exec_result = manager.execute_plugin(
                    &plugin_id, 
                    serde_json::json!({"test": true})
                ).await;
                black_box(exec_result);
            }
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_plugin_loading,
    benchmark_plugin_execution,
    benchmark_concurrent_plugin_execution,
    benchmark_plugin_lifecycle,
    benchmark_plugin_discovery,
    benchmark_plugin_registry,
    benchmark_plugin_error_handling
);
criterion_main!(benches); 