# MCP Test Fixes Plan

## Overview

This document outlines the plan to address test failures in the Machine Context Protocol (MCP) implementation. The test failures are primarily related to:

1. Async handling in resilience module tests
2. Type mismatches in function calls
3. Method visibility issues
4. Updating tests to match API changes

## Priority Areas

### 1. Resilience Module Tests

The resilience module shows significant test failures caused by recent API changes, particularly around:

- Updates to accept owned `RetryMechanism` instead of references
- Need to await futures in test assertions
- Visibility issues with internal methods like `calculate_delay` in RetryMechanism
- Type mismatches with error handling

#### Action Items

| ID  | Issue                                      | File(s)                               | Priority |
|-----|--------------------------------------------|------------------------------------|----------|
| R-1 | Make `calculate_delay` public for testing  | `retry.rs`                          | High     |
| R-2 | Fix async tests to use `.await` properly   | `resilience/tests/*.rs`            | High     |
| R-3 | Update tests to pass owned RetryMechanism  | `resilience/tests/*.rs`            | High     |
| R-4 | Fix type annotations in recovery tests     | `resilience/tests/recovery_tests.rs`| Medium   |
| R-5 | Update error type coercions in integration tests | `resilience/tests/integration_tests.rs` | Medium |

### 2. Circuit Breaker Tests

These tests have issues with method name changes and missing awaits:

#### Action Items

| ID  | Issue                                      | File(s)                               | Priority |
|-----|--------------------------------------------|------------------------------------|----------|
| C-1 | Update `get_state()` to `state()` in tests | `resilience/tests/integration_tests.rs` | High |
| C-2 | Update error handling in circuit breaker tests | `resilience/tests/circuit_breaker_tests.rs` | Medium |

### 3. State Sync Tests

State sync tests need to be updated to handle the async nature of the API:

#### Action Items

| ID  | Issue                                      | File(s)                               | Priority |
|-----|--------------------------------------------|------------------------------------|----------|
| S-1 | Add `.await` to `is_ok()` and `is_err()` calls | `resilience/tests/state_sync_tests.rs` | High |
| S-2 | Update error handling in state sync tests | `resilience/tests/state_sync_tests.rs` | Medium |

### 4. General Fixes

Overall improvements to the test suite:

#### Action Items

| ID  | Issue                                      | File(s)                               | Priority |
|-----|--------------------------------------------|------------------------------------|----------|
| G-1 | Fix transport frame mutable variable warning | `transport/frame.rs:162` | Low |
| G-2 | Remove unused variable warning in recovery.rs | `resilience/recovery.rs:279` | Low |

## Implementation Steps

1. **Analysis Phase**
   - Categorize each test failure by root cause
   - Determine common patterns in failing tests
   - Create test file dependency graph to identify fix order

2. **Fix Core Issues**
   - Start with R-1 through R-3 (highest priority resilience module fixes)
   - Address state sync test issues (S-1)
   - Fix circuit breaker test method naming (C-1)

3. **Fix Derived Issues**
   - Address type annotations and coercions (R-4, R-5)
   - Complete state sync error handling (S-2)
   - Fix circuit breaker error handling (C-2)

4. **Cleanup**
   - Fix low-priority warnings (G-1, G-2)
   - Run comprehensive tests and fix any remaining issues
   - Document any API changes or patterns for future reference

## Testing Strategy

1. Fix tests incrementally, starting with the most fundamental modules
2. After each fix, run targeted tests with `cargo test --package squirrel-mcp module_name`
3. Once a module's tests pass, run full suite to check for regressions

## Integration Considerations

- Updates must maintain backward compatibility where possible
- Document any public API changes in the module documentation
- Consider any impacts on dependent modules/crates

## Timeline

- **Phase 1** (High Priority): Address R-1, R-2, R-3, C-1, S-1 (Estimated: 1-2 days)
- **Phase 2** (Medium Priority): Address R-4, R-5, C-2, S-2 (Estimated: 1-2 days)
- **Phase 3** (Low Priority): Address G-1, G-2 and complete documentation (Estimated: 1 day)

## Team Responsibilities

- **Developer**: Implement fixes according to plan
- **Reviewer**: Verify fixes maintain intended behavior
- **Test Coordinator**: Ensure comprehensive test coverage

## Success Criteria

1. All tests pass when running `cargo test --package squirrel-mcp`
2. No new warnings are introduced
3. The public API remains stable or has documented changes
4. Test coverage remains at or above previous levels 