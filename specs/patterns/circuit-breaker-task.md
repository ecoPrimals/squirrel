# Circuit Breaker Refactoring Task

## Summary

The circuit breaker pattern is a critical resilience mechanism for the Squirrel system. The current implementation has Rust-specific issues that prevent it from being used effectively. This task outlines the work needed to refactor the circuit breaker implementation to make it safe, idiomatic, and maintainable.

## Priority

**HIGH** - The circuit breaker is a foundational resilience pattern needed across the system.

## Background

The circuit breaker pattern prevents cascading failures by temporarily blocking operations when failures exceed thresholds. Our implementation needs to work reliably with MCP and other integration points, following Rust best practices.

A detailed specification for the circuit breaker implementation exists in `specs/patterns/circuit-breaker-implementation.md`, but the current code has several issues that need to be addressed.

## Technical Issues

1. **Trait Object Safety** - The `CircuitBreaker` trait is not object-safe due to generic methods
2. **Error Type Issues** - Type bounds problems with `anyhow::Error` and `BreakerError<E>`
3. **Trait vs Type Confusion** - Improper use of traits as types
4. **String Type Mismatches** - Inconsistent string handling

See `specs/patterns/circuit-breaker-refactoring.md` for a comprehensive analysis and proposed solutions.

## Tasks

### 1. Trait Redesign (2 days)

- [ ] Split `CircuitBreaker` trait into object-safe and generic parts
- [ ] Create concrete error types without generic parameters
- [ ] Update trait bounds and type constraints
- [ ] Document new trait hierarchy

### 2. Implementation Refactoring (3 days)

- [ ] Refactor `StandardCircuitBreaker` implementation
- [ ] Update `MonitoringCircuitBreaker` integration
- [ ] Fix string handling and type conversions
- [ ] Ensure thread safety with proper mutex usage

### 3. API Integration (2 days)

- [ ] Update all client code to use new API
- [ ] Fix function signatures that use circuit breakers
- [ ] Update example code
- [ ] Update MCP resilience framework integration

### 4. Testing & Verification (2 days)

- [ ] Update unit tests for new implementation
- [ ] Add tests for edge cases and failure scenarios
- [ ] Verify monitoring integration
- [ ] Create integration tests with other resilience patterns

## Tasks

### 1. Trait Redesign (2 days) - COMPLETED

- [x] Split `CircuitBreaker` trait into object-safe and generic parts
- [x] Create concrete error types without generic parameters
- [x] Update trait bounds and type constraints
- [x] Document new trait hierarchy

### 2. Implementation Refactoring (3 days) - COMPLETED

- [x] Refactor `StandardCircuitBreaker` implementation
- [x] Update `MonitoringCircuitBreaker` integration
- [x] Fix string handling and type conversions
- [x] Ensure thread safety with proper mutex usage

### 3. API Integration (2 days) - COMPLETED

- [x] Update all client code to use new API
- [x] Fix function signatures that use circuit breakers
- [x] Update example code
- [x] Update MCP resilience framework integration

### 4. Testing & Verification (2 days) - COMPLETED

- [x] Update unit tests for new implementation
- [x] Add tests for edge cases and failure scenarios
- [x] Verify monitoring integration
- [x] Create integration tests with other resilience patterns
- [x] Fix test flakiness issues with more robust assertion patterns

## Deliverables

1. Refactored circuit breaker implementation
2. Updated documentation and examples
3. Comprehensive test suite
4. Integration with MCP and monitoring system

## Acceptance Criteria

- [x] Code builds without errors or warnings
- [x] All tests pass
- [x] Circuit breaker can be used as a trait object where needed
- [x] Generic operations can be executed through the circuit breaker
- [x] Monitoring integration works correctly
- [x] Error handling is consistent and type-safe
- [x] Documentation is complete and accurate

## Related Documents

- [Circuit Breaker Implementation](specs/patterns/circuit-breaker-implementation.md)
- [Circuit Breaker Refactoring](specs/patterns/circuit-breaker-refactoring.md)
- [Resilience Framework](specs/patterns/resilience-framework.md)
- [MCP Integration](specs/integration/mcp-monitoring-integration.md)

## Notes

This refactoring should prioritize correctness and maintainability over performance optimizations. The circuit breaker is a critical resilience mechanism, so robust error handling and clear API design are essential. Once the refactoring is complete, we can evaluate performance and make targeted optimizations if needed. 