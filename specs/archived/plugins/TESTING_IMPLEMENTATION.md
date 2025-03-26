# Plugin System Test Implementation Guide

*Last Updated: 2024-04-20*

## Introduction

This document outlines the implementation approach for the testing framework of the Squirrel Plugin System. It provides practical guidance on implementing the tests described in the `TESTING_SPECIFICATION.md` document.

## Test Implementation Structure

The test implementation follows a modular structure that mirrors the plugin system architecture:

```
src/
  tests/
    mod.rs                   # Main test module
    tool_plugin_test.rs      # Tests for tool plugins
    command_plugin_test.rs   # Tests for command plugins
    context_test.rs          # Tests for plugin context
    dynamic_loading_test.rs  # Tests for dynamic loading
    plugin_registry_test.rs  # Tests for plugin registry
    plugin_manager_test.rs   # Tests for plugin manager
    marketplace_test.rs      # Tests for plugin marketplace
    security_test.rs         # Tests for plugin security
benchmark/
  mod.rs                     # Main benchmark module 
  loading_bench.rs           # Benchmarks for plugin loading
  execution_bench.rs         # Benchmarks for plugin execution
```

## Test Categories Implementation

### 1. Unit Tests

Unit tests for each component should follow this pattern:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;
    
    #[test]
    async fn test_component_specific_functionality() {
        // Arrange: Set up test data
        
        // Act: Call the function being tested
        
        // Assert: Verify the results
    }
    
    #[test]
    async fn test_component_error_conditions() {
        // Test how the component handles errors
    }
}
```

#### Example: Tool Plugin Tests

The tool plugin tests (already implemented in `tool_plugin_test.rs`) demonstrate:
- Testing the listing of available tools
- Testing successful tool execution
- Testing error conditions
- Testing invalid input handling

Similar patterns should be applied to all other components.

### 2. Integration Tests

Integration tests should verify the interaction between different components:

```rust
#[cfg(test)]
mod integration_tests {
    use crate::plugins::registry::PluginRegistry;
    use crate::plugins::manager::PluginManager;
    use crate::plugins::dynamic::DynamicLoader;
    
    #[tokio::test]
    async fn test_manager_registry_integration() {
        // Create a registry
        let registry = PluginRegistryImpl::new();
        
        // Create a manager that uses the registry
        let manager = PluginManagerImpl::new(registry.clone());
        
        // Test interactions between the two
        let plugin_id = uuid::Uuid::new_v4();
        let metadata = create_test_metadata(plugin_id);
        
        // Registry operations
        registry.register_plugin(plugin_id, metadata.clone()).await?;
        
        // Manager operations that use the registry
        let retrieved_metadata = manager.get_plugin_metadata(plugin_id).await?;
        
        // Verify integration
        assert_eq!(retrieved_metadata.id, metadata.id);
    }
}
```

### 3. System Tests

System tests verify complete workflows:

```rust
#[cfg(test)]
mod system_tests {
    use std::path::PathBuf;
    use crate::plugins::Plugin;
    use crate::plugins::manager::PluginManager;
    
    #[tokio::test]
    async fn test_end_to_end_plugin_workflow() {
        // Create all necessary components
        let registry = create_test_registry();
        let loader = create_test_loader();
        let manager = create_test_manager(registry.clone(), loader.clone());
        
        // Test a complete workflow
        // 1. Load a plugin
        let plugin_path = get_test_plugin_path();
        let plugin_id = manager.load_plugin(&plugin_path).await?;
        
        // 2. Get plugin capabilities
        let plugin = manager.get_plugin(plugin_id).await?;
        
        // 3. Execute plugin functionality
        let result = plugin.do_something().await?;
        
        // 4. Unload the plugin
        manager.unload_plugin(plugin_id).await?;
        
        // Verify the complete workflow
        assert!(result.is_success());
        assert!(!registry.contains_plugin(plugin_id).await);
    }
}
```

### 4. Performance Benchmarks

Performance benchmarks should be implemented using the Criterion framework:

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_plugin_loading(c: &mut Criterion) {
    // Setup
    let manager = create_benchmarking_manager();
    let plugin_path = get_benchmark_plugin_path();
    
    c.bench_function("plugin_load", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(|| async {
            manager.load_plugin(&plugin_path).await
        })
    });
}

fn bench_plugin_execution(c: &mut Criterion) {
    // Setup
    let manager = create_benchmarking_manager();
    let plugin_id = load_benchmark_plugin(manager).await;
    
    c.bench_function("plugin_execute", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(|| async {
            let plugin = manager.get_plugin_as::<dyn CommandsPlugin>(plugin_id).await.unwrap();
            plugin.execute_command("benchmark_command", &serde_json::json!({})).await
        })
    });
}

criterion_group!(benches, bench_plugin_loading, bench_plugin_execution);
criterion_main!(benches);
```

## Mock Implementations

For effective testing, create mock implementations of the plugin interfaces:

```rust
use mockall::predicate::*;
use mockall::mock;

mock! {
    pub CommandsPlugin {}
    
    #[async_trait::async_trait]
    impl CommandsPlugin for CommandsPlugin {
        async fn list_commands(&self) -> Result<Vec<Command>>;
        async fn execute_command(&self, name: &str, args: &serde_json::Value) -> Result<CommandResult>;
    }
}

fn create_mock_commands_plugin() -> MockCommandsPlugin {
    let mut mock = MockCommandsPlugin::new();
    
    // Configure the mock
    mock.expect_list_commands()
        .returning(|| Ok(vec![Command::new("test_command")]));
        
    mock.expect_execute_command()
        .with(eq("test_command"), always())
        .returning(|_, _| Ok(CommandResult::Success(serde_json::json!({"status": "success"}))));
        
    mock
}
```

## Test Data Management

Create utilities for managing test data:

```rust
// In a test_utils.rs file
use uuid::Uuid;
use crate::plugins::dynamic::PluginMetadata;

pub fn create_test_metadata(id: Uuid) -> PluginMetadata {
    PluginMetadata {
        id,
        name: format!("Test Plugin {}", id),
        version: "1.0.0".to_string(),
        api_version: "1.0.0".to_string(),
        description: "Test plugin for unit tests".to_string(),
        author: "Test Framework".to_string(),
        dependencies: Vec::new(),
    }
}

pub fn get_test_plugin_path() -> PathBuf {
    // Return path to a test plugin suitable for the current platform
    #[cfg(target_os = "windows")]
    return PathBuf::from("tests/test_plugins/test_plugin.dll");
    
    #[cfg(target_os = "linux")]
    return PathBuf::from("tests/test_plugins/test_plugin.so");
    
    #[cfg(target_os = "macos")]
    return PathBuf::from("tests/test_plugins/test_plugin.dylib");
}
```

## Test Environment Setup

Create a standardized test environment:

```rust
struct TestEnvironment {
    registry: Arc<dyn PluginRegistry>,
    loader: Arc<dyn DynamicLoader>,
    manager: Arc<dyn PluginManager>,
    test_plugins_dir: PathBuf,
}

impl TestEnvironment {
    fn new() -> Self {
        let registry = Arc::new(InMemoryPluginRegistry::new());
        let loader = Arc::new(MockDynamicLoader::new());
        let manager = Arc::new(PluginManagerImpl::new(registry.clone(), loader.clone()));
        let test_plugins_dir = PathBuf::from("tests/test_plugins");
        
        Self {
            registry,
            loader,
            manager,
            test_plugins_dir,
        }
    }
    
    fn get_test_plugin_path(&self, name: &str) -> PathBuf {
        self.test_plugins_dir.join(format!("{}.{}", name, self.get_plugin_extension()))
    }
    
    fn get_plugin_extension(&self) -> &'static str {
        #[cfg(target_os = "windows")]
        return "dll";
        
        #[cfg(target_os = "linux")]
        return "so";
        
        #[cfg(target_os = "macos")]
        return "dylib";
    }
}
```

## Testing Cross-Platform Functionality

For testing cross-platform functionality, use conditional compilation:

```rust
#[cfg(test)]
mod tests {
    #[cfg(target_os = "windows")]
    #[tokio::test]
    async fn test_windows_specific_functionality() {
        // Windows-specific test
    }
    
    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn test_linux_specific_functionality() {
        // Linux-specific test
    }
    
    #[cfg(target_os = "macos")]
    #[tokio::test]
    async fn test_macos_specific_functionality() {
        // macOS-specific test
    }
    
    #[tokio::test]
    async fn test_platform_agnostic_functionality() {
        // Platform-agnostic test that runs on all platforms
    }
}
```

## Error Injection Testing

Create tests that inject errors to verify error handling:

```rust
#[tokio::test]
async fn test_plugin_manager_handles_load_error() {
    // Create a mock loader that returns errors
    let mut loader = MockDynamicLoader::new();
    loader.expect_load_plugin()
        .returning(|_| Err(PluginError::LoadError("Simulated load error".to_string())));
    
    let registry = Arc::new(InMemoryPluginRegistry::new());
    let manager = PluginManagerImpl::new(registry.clone(), Arc::new(loader));
    
    // Attempt to load a plugin
    let result = manager.load_plugin(&PathBuf::from("nonexistent.dll")).await;
    
    // Verify that the error is properly propagated
    assert!(result.is_err());
    if let Err(PluginError::LoadError(msg)) = result {
        assert_eq!(msg, "Simulated load error");
    } else {
        panic!("Expected LoadError");
    }
}
```

## Test Reporting

Implement custom test reporting to track test metrics:

```rust
#[derive(Default)]
struct TestReportData {
    total_tests: usize,
    passed_tests: usize,
    failed_tests: usize,
    skipped_tests: usize,
    test_execution_time: Duration,
}

fn generate_test_report(data: TestReportData) -> String {
    format!(
        "Test Report:\n\
        Total Tests: {}\n\
        Passed: {}\n\
        Failed: {}\n\
        Skipped: {}\n\
        Execution Time: {:?}",
        data.total_tests,
        data.passed_tests,
        data.failed_tests,
        data.skipped_tests,
        data.test_execution_time
    )
}
```

## Continuous Integration Testing

Set up continuous integration testing to run tests on multiple platforms:

```yaml
# In .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    name: Test on ${{ matrix.os }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        
    steps:
    - uses: actions/checkout@v2
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        profile: minimal
        toolchain: stable
        override: true
    - name: Build
      run: cargo build --verbose
    - name: Run tests
      run: cargo test --verbose
```

## Conclusion

This implementation guide provides a comprehensive approach to testing the Squirrel Plugin System. By following these patterns and examples, you can create a robust test suite that ensures the plugin system works correctly across all supported platforms and handles error conditions gracefully.

As new features are added to the plugin system, corresponding tests should be added to maintain high code quality and ensure reliable operation. 