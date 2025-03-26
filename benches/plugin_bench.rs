// Plugin System Performance Benchmarks
//
// This module contains benchmarks for the plugin system,
// particularly focusing on plugin loading and execution performance.

use std::path::PathBuf;
use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, black_box};
use tokio::runtime::Runtime;
use uuid::Uuid;
use serde_json::json;

use squirrel_plugins::plugins::{
    dynamic::create_library_loader,
    management::{PluginRegistry, PluginRegistryImpl},
    interfaces::CommandsPlugin,
};

// Get test plugin path for the current platform
fn get_test_plugin_path() -> PathBuf {
    let base_path = PathBuf::from("test_plugins");
    
    #[cfg(target_os = "windows")]
    {
        base_path.join("test_plugin.dll")
    }
    
    #[cfg(target_os = "linux")]
    {
        base_path.join("test_plugin.so")
    }
    
    #[cfg(target_os = "macos")]
    {
        base_path.join("test_plugin.dylib")
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        panic!("Unsupported operating system");
    }
}

// Benchmark the plugin loading time
fn bench_plugin_loading(c: &mut Criterion) {
    let plugin_path = get_test_plugin_path();
    
    // Skip the benchmark if the test plugin doesn't exist
    if !plugin_path.exists() {
        println!("Skipping benchmark: Test plugin not found at {:?}", plugin_path);
        return;
    }
    
    let rt = Runtime::new().unwrap();
    let loader = create_library_loader();
    
    let mut group = c.benchmark_group("Plugin Loading");
    group.bench_function(BenchmarkId::new("load_time", 1), |b| {
        b.to_async(&rt).iter(|| async {
            black_box(loader.load_plugin(&plugin_path).await.unwrap())
        });
    });
    group.finish();
}

// Benchmark the plugin command execution time
fn bench_plugin_command_execution(c: &mut Criterion) {
    let plugin_path = get_test_plugin_path();
    
    // Skip the benchmark if the test plugin doesn't exist
    if !plugin_path.exists() {
        println!("Skipping benchmark: Test plugin not found at {:?}", plugin_path);
        return;
    }
    
    let rt = Runtime::new().unwrap();
    let loader = create_library_loader();
    let registry = Arc::new(PluginRegistryImpl::new());
    
    // Load the plugin
    let plugin = rt.block_on(async {
        let plugin = loader.load_plugin(&plugin_path).await.unwrap();
        let plugin_id = plugin.metadata().id;
        registry.register_plugin(plugin_id, plugin).await.unwrap();
        plugin_id
    });
    
    // Get the CommandsPlugin interface
    let commands_plugin = rt.block_on(async {
        registry.get_plugin_as::<dyn CommandsPlugin>(&plugin).await
    });
    
    // Skip if the plugin doesn't implement CommandsPlugin
    if commands_plugin.is_err() {
        println!("Skipping benchmark: Plugin doesn't implement CommandsPlugin");
        return;
    }
    
    let commands_plugin = commands_plugin.unwrap();
    
    // Get the available commands
    let commands = commands_plugin.list_commands();
    
    // Skip if no commands are available
    if commands.is_empty() {
        println!("Skipping benchmark: No commands available");
        return;
    }
    
    // Use the first command for benchmarking
    let command_name = commands[0].name.clone();
    let args = json!({});
    
    let mut group = c.benchmark_group("Plugin Command Execution");
    
    group.bench_function(BenchmarkId::new("execute_time", command_name), |b| {
        b.to_async(&rt).iter(|| async {
            black_box(commands_plugin.execute_command(&command_name, args.clone()).await.unwrap())
        });
    });
    
    group.finish();
    
    // Clean up
    rt.block_on(async {
        registry.unregister_plugin(&plugin).await.unwrap();
    });
}

// Benchmark plugin loading with various resource conditions
fn bench_plugin_loading_with_resources(c: &mut Criterion) {
    let plugin_path = get_test_plugin_path();
    
    // Skip the benchmark if the test plugin doesn't exist
    if !plugin_path.exists() {
        println!("Skipping benchmark: Test plugin not found at {:?}", plugin_path);
        return;
    }
    
    let rt = Runtime::new().unwrap();
    let loader = create_library_loader();
    
    let mut group = c.benchmark_group("Plugin Loading With Resources");
    
    // Benchmark loading under memory pressure
    group.bench_function(BenchmarkId::new("memory_pressure", 1), |b| {
        // Allocate a large amount of memory to create pressure
        let memory_pressure = vec![0u8; 100 * 1024 * 1024]; // 100 MB
        
        b.to_async(&rt).iter(|| async {
            // Keep the memory pressure alive during the benchmark
            black_box(&memory_pressure);
            black_box(loader.load_plugin(&plugin_path).await.unwrap())
        });
    });
    
    // Benchmark loading under CPU pressure
    group.bench_function(BenchmarkId::new("cpu_pressure", 1), |b| {
        b.to_async(&rt).iter(|| async {
            // Perform some CPU-intensive work before loading
            let mut sum = 0;
            for i in 0..1000000 {
                sum += i;
            }
            black_box(sum);
            
            black_box(loader.load_plugin(&plugin_path).await.unwrap())
        });
    });
    
    group.finish();
}

// Benchmark concurrent plugin loading
fn bench_concurrent_plugin_loading(c: &mut Criterion) {
    let plugin_path = get_test_plugin_path();
    
    // Skip the benchmark if the test plugin doesn't exist
    if !plugin_path.exists() {
        println!("Skipping benchmark: Test plugin not found at {:?}", plugin_path);
        return;
    }
    
    let rt = Runtime::new().unwrap();
    let loader = create_library_loader();
    
    let mut group = c.benchmark_group("Concurrent Plugin Loading");
    
    // Benchmark loading 5 plugins concurrently
    group.bench_function(BenchmarkId::new("concurrent", 5), |b| {
        b.to_async(&rt).iter(|| async {
            // Create 5 concurrent loading tasks
            let mut tasks = Vec::new();
            for _ in 0..5 {
                let loader_clone = loader.clone();
                let path_clone = plugin_path.clone();
                tasks.push(tokio::spawn(async move {
                    loader_clone.load_plugin(&path_clone).await
                }));
            }
            
            // Wait for all tasks to complete
            for task in tasks {
                black_box(task.await.unwrap().unwrap());
            }
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_plugin_loading,
    bench_plugin_command_execution,
    bench_plugin_loading_with_resources,
    bench_concurrent_plugin_loading
);
criterion_main!(benches); 