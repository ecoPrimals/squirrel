# Squirrel Plugin System - Cross-Platform Testing

This directory contains test plugins and build scripts for the Squirrel Plugin System cross-platform testing framework.

## Quick Start

### Building Test Plugins

To quickly build test plugins for your platform, use the provided scripts:

**Windows (PowerShell):**
```powershell
# Navigate to the test_plugins directory
cd test_plugins

# Run the PowerShell build script
.\build_plugins.ps1
```

**Linux/macOS (Bash):**
```bash
# Navigate to the test_plugins directory
cd test_plugins

# Make sure the script is executable
chmod +x build_plugins.sh

# Run the Bash build script
./build_plugins.sh
```

### Running Tests

Once you've built the test plugin for your platform:

```bash
# Run all tests
cargo test

# Run just the dynamic loading tests
cargo test --test dynamic_loading_test

# Run benchmarks
cargo bench --bench plugin_bench
```

## What the Scripts Do

The build scripts automate the entire process of:

1. Creating a temporary directory for building
2. Setting up a new Rust crate with the correct dependencies
3. Copying the test plugin implementation from `src/plugins/examples/test_dynamic_plugin.rs`
4. Building the plugin as a dynamic library
5. Copying the resulting library to the correct location with the platform-appropriate name:
   - Windows: `test_plugin.dll`
   - Linux: `test_plugin.so`
   - macOS: `test_plugin.dylib`
6. Cleaning up temporary files

## Cross-Platform Testing Framework

The Squirrel Plugin System includes a comprehensive cross-platform testing framework that ensures plugins work consistently across all supported platforms:

### Framework Components

1. **Dynamic Loading Test Suite** (`src/tests/dynamic_loading_test.rs`)
   - Platform-specific tests for plugin loading
   - Metadata validation
   - Plugin registration
   - Command execution
   - Resource monitoring

2. **Test Plugin Template** (`src/plugins/examples/test_dynamic_plugin.rs`)
   - Reference implementation for dynamic plugins
   - Platform-specific functionality
   - Command handling

3. **Performance Benchmarks** (`benches/plugin_bench.rs`)
   - Plugin loading benchmarks
   - Command execution benchmarks
   - Resource usage benchmarks
   - Concurrent loading tests

## Manual Build Instructions

If you prefer to build the test plugin manually, follow these steps:

1. Create a new Rust library crate for the test plugin:

```bash
# Create a new library crate
cargo new --lib test-dynamic-plugin
cd test-dynamic-plugin
```

2. Configure the crate for building as a dynamic library by editing `Cargo.toml`:

```toml
[package]
name = "test-dynamic-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
async-trait = "0.1.68"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.28", features = ["full"] }
uuid = { version = "1.3", features = ["v4", "serde"] }
squirrel-mcp = { path = "../path/to/squirrel-mcp" }
squirrel-plugins = { path = "../path/to/squirrel-plugins" }
```

3. Copy the test plugin implementation from `src/plugins/examples/test_dynamic_plugin.rs` to your crate's `src/lib.rs`.

4. Build the dynamic library:

```bash
# Release build (recommended for testing)
cargo build --release
```

5. Copy the compiled library to this directory with the appropriate name:

**Windows:**
```bash
copy target\release\test_dynamic_plugin.dll ..\path\to\test_plugins\test_plugin.dll
```

**Linux:**
```bash
cp target/release/libtest_dynamic_plugin.so ../path/to/test_plugins/test_plugin.so
```

**macOS:**
```bash
cp target/release/libtest_dynamic_plugin.dylib ../path/to/test_plugins/test_plugin.dylib
```

## Running the Tests

Once you have built the test plugin for your platform, you can run the dynamic loading tests:

```bash
# Run all tests
cargo test

# Run dynamic loading tests specifically
cargo test --test dynamic_loading_test

# Run a specific test
cargo test test_plugin_loading
```

## Cross-Platform Testing Best Practices

For thorough testing, follow these best practices:

1. **Test on All Platforms**
   - Build and test on Windows, Linux, and macOS
   - Verify all tests pass on each platform
   - Check for platform-specific issues

2. **Resource Testing**
   - Test with various resource limits
   - Verify plugins respect resource constraints
   - Test under high load conditions

3. **Error Handling**
   - Test error conditions (missing plugins, invalid plugins)
   - Verify proper error recovery
   - Test graceful handling of plugin failures

4. **Performance Benchmarking**
   - Run benchmarks on all platforms
   - Compare performance across environments
   - Track performance changes over time

## Troubleshooting

If you encounter issues:

1. **Library not found**: 
   - Ensure the test plugin exists at the expected location
   - Verify file permissions
   - Check file name matches platform convention

2. **Symbol resolution errors**: 
   - Verify plugin was built with compatible dependencies
   - Check the plugin exports all required symbols
   - Ensure FFI compatibility

3. **Version mismatches**: 
   - Check API version compatibility
   - Ensure dependency versions match

4. **Resource limit failures**:
   - Check system resource availability
   - Verify resource monitor configuration

## Performance Benchmarking

For performance benchmarking of plugin loading and execution:

```bash
# Run all benchmarks
cargo bench --bench plugin_bench

# Run specific benchmark group
cargo bench --bench plugin_bench -- plugin_loading

# Run specific benchmark
cargo bench --bench plugin_bench -- plugin_loading/load_time
```

Results are saved in the `target/criterion` directory and can be viewed as HTML reports.

## Memory Usage Monitoring

To monitor memory usage during plugin operations:

```bash
# Run tests with memory profiling
RUST_LOG=debug cargo test --features memory_profiling test_resource_limits
```

## CI/CD Integration

For integrating cross-platform testing in CI/CD pipelines, see the example configurations in the [Cross-Platform Testing Guide](../docs/plugins/cross_platform_testing.md#integration-with-cicd). 