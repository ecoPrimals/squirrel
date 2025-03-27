# CLI Test Framework

## Overview

The CLI test framework provides comprehensive testing for the CLI module of the Squirrel project. It includes a range of test types, from unit tests to integration tests focusing on adapters, concurrency, resource management, and end-to-end usage.

## Running Tests

To run all tests, use the following command:

```bash
cargo test --features testing
```

The `--features testing` flag is required because some test modules (notably the `test_command` module) are gated behind the `testing` feature flag. Without this flag, import errors will occur.

## Test Structure

The test structure is organized as follows:

1. **Unit Tests**: Located within the source files using the standard `#[cfg(test)]` attribute
2. **Adapter Tests**: Testing the various command adapters in isolation and with mock implementations
3. **Concurrency Tests**: Validating behavior under concurrent execution conditions
4. **Resource Management Tests**: Testing handling of resource limits and cleanup
5. **End-to-End Tests**: Full CLI integration tests with command execution

## Recent Improvements

The test framework has undergone several improvements:

1. **Fixed Trait Safety Issues**: Refactored `TestCommand` trait to properly handle async methods using a combination of base traits and type erasure patterns
2. **Improved Timing Assumptions**: Made concurrency tests more reliable across different systems
3. **Enhanced Resource Testing**: Created proper test commands for memory allocation testing
4. **Better Error Handling**: Improved error reporting and handling throughout tests

## Async Trait Pattern

One of the key innovations is the approach to handling async traits safely. The pattern uses:

- A base trait with non-async methods (object-safe)
- An extended trait with async methods (not used with `dyn`)
- Type erasure wrappers to avoid `dyn AsyncTrait` issues

For detailed documentation on this pattern, see [the async-trait-safety pattern document](../patterns/async-trait-safety.md).

## Configuration

Tests can be configured using environment variables:

- `SQUIRREL_TEST_LOG_LEVEL`: Controls the log level during tests (default: "warn")
- `SQUIRREL_TEST_TIMEOUT`: Sets the default timeout for async operations (default: 5000ms)

## Contributing

When contributing new tests:

1. Follow the existing patterns for similar test types
2. Use the async trait pattern for any new traits with async methods
3. Consider placing test-only code behind the `testing` feature flag
4. Ensure tests clean up any resources they create

## References

- [Test Fix Progress](TEST_FIX_PROGRESS.md) - Detailed record of test framework improvements
- [Async Trait Safety Pattern](../patterns/async-trait-safety.md) - Documentation of the async trait pattern
- [TEAMCHAT Update](TEAMCHAT_UPDATE.md) - Team communication about test framework changes 