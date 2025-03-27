# Squirrel CLI Testing Guide

This directory contains tests for the Squirrel CLI crate, organized into different categories for comprehensive coverage.

## Test Categories

### 1. Adapter Tests

- **isolated_adapter_tests.rs**: Tests adapter implementations in isolation without dependencies on project modules.
- **standalone_adapter_tests.rs**: Tests standalone adapter implementations with comprehensive auth and plugin testing.
- **adapter_tests.rs**: Tests the actual CLI adapter implementations with project dependencies.

### 2. End-to-End Tests

- **cli_end_to_end_tests.rs**: Tests the CLI commands from a user's perspective, verifying the entire command execution flow.

### 3. Concurrency Tests

- **concurrency_tests.rs**: Tests the CLI's behavior under high concurrency, ensuring thread safety and proper lock handling.

### 4. Resource Tests

- **resource_limit_tests.rs**: Tests the CLI's behavior when dealing with resource constraints, such as memory limits and high connection counts.

## Running the Tests

To run all tests:

```bash
cargo test -p squirrel-cli
```

To run a specific test category:

```bash
cargo test -p squirrel-cli -- concurrency
```

To run a specific test:

```bash
cargo test -p squirrel-cli -- test_concurrent_command_execution
```

## Benchmarks

Performance benchmarks are located in the `benches` directory at the workspace root. These benchmarks measure command execution performance, lock contention, and concurrent operations.

To run the benchmarks:

```bash
cargo bench -p squirrel-cli
```

## Test Design Principles

1. **Isolation**: Tests are designed to isolate components for focused testing
2. **Mock Implementations**: Tests use mock objects and test doubles where appropriate
3. **Async Testing**: Tests properly use async/await with the tokio test runtime
4. **Concurrency**: Tests verify behavior under concurrent access and high load
5. **Resource Usage**: Tests verify proper resource management and cleanup
6. **Error Handling**: Tests verify proper error handling and propagation

## Adding New Tests

When adding new tests, follow these guidelines:

1. Place the test in the appropriate category file
2. Use descriptive test names that clearly indicate what is being tested
3. Include assertions that verify the expected behavior
4. For performance-sensitive tests, measure and compare execution times
5. For concurrency tests, ensure the test creates actual concurrent conditions
6. For resource tests, verify both success cases and graceful failure cases

## Test Fixtures

The test module provides several helper functions and types to make writing tests easier:

- `create_test_command`: Creates a test command with the specified name, description, and output
- `create_test_registry`: Creates a command registry with test commands pre-registered
- `TestCommand` trait: Defines the interface for test commands

Use these fixtures to create consistent and reusable test components. 