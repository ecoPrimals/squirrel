# MCP Clippy Cleanup Plan

## Overview

This document outlines a systematic approach to address the large number of Clippy warnings in the MCP crate. The goal is to improve code quality, maintain API stability, and ensure proper documentation while addressing warnings in a prioritized, incremental manner.

## Current Status

The MCP crate currently has approximately 2,600+ Clippy warnings. Major categories include:

1. Missing `Debug` trait implementations
2. Missing documentation on public APIs
3. Unused `async` functions (functions marked as `async` but containing no `await` expressions)
4. Unnecessary mutability in parameters
5. Public trait issues (e.g., using `async fn` in traits without proper bounds)
6. Various pedantic warnings

## Phased Cleanup Strategy

We'll adopt a phased approach to clean up these warnings, temporarily relaxing linting requirements while we make progress.

### Phase 1: Disable Warnings (Completed)

Add temporary allowances to `lib.rs` to disable the most noisy warnings while we work on addressing them:

```rust
#![allow(clippy::module_inception)]
#![allow(clippy::needless_doctest_main)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::manual_async_fn)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(missing_docs)]
```

This will let us focus on specific categories of warnings while keeping the code in a compilable state.

### Phase 2: Debug Traits (In Progress)

- Created a `debug_impl.rs` file to implement Debug for types with non-Debug fields
- Implemented Debug for `ToolManagerBuilder` that has function pointers and trait objects
- Implemented a basic Debug for `MessageBuilder` since its fields are private
- Added conditional Debug implementation for `TcpTransport` when the feature is enabled
- Added progress tracking for Debug implementations in different modules
- Current progress by module:
  - Tool module: 10%
  - Transport module: 5% 
  - Message module: 25%

Tasks remaining:
- Implement Debug for more router and protocol types 
- Address remaining complex types that lack Debug
- Update feature flags as needed for conditional implementations

### Phase 3: Async/Await (Not Started)

- Fix `manual_async_fn` warnings
- Convert uses of `.then()` and other futures combinators to async/await
- Review and update futures error handling

### Phase 4: Mutability (Not Started)

- Address `unused_mut` warnings
- Fix `needless_borrow` and `needless_deref` warnings
- Convert unnecessary mutable variables to immutable

### Phase 5: Traits (Not Started)

- Address `derivable_impls` warnings
- Clean up custom implementations that could be derived
- Fix trait implementations that could be simplified

### Phase 6: Documentation (Not Started)

- Add missing module-level documentation
- Add missing struct/enum documentation
- Ensure all public items are documented

### Phase 7: Pedantic (Not Started)

- Enable and address pedantic lints
- Fix any remaining minor style issues
- Final cleanup pass

## Implementation Plan

### Week 1: Setup and Debug Traits
- Disable all warnings temporarily
- Implement Debug for core types (Phase 2, first half)
- Update the MCP_REFACTORING_SUMMARY.md file

### Week 2: Complete Debug Traits
- Complete Debug implementations for all types
- Begin addressing async/await issues
- Re-enable missing_debug_implementations warning

### Week 3: Async/Await and Mutability
- Complete async/await fixes
- Address mutability issues
- Re-enable related warnings

### Week 4: Traits and Documentation Start
- Fix trait issues
- Begin documentation of core modules
- Re-enable trait-related warnings

### Week 5-6: Documentation Completion
- Complete documentation for all modules
- Address any remaining warnings
- Re-enable all warnings

## Testing Strategy

For each phase:
1. Run unit tests after each significant set of changes
2. Run integration tests after completing each phase
3. Verify that no regressions have been introduced
4. Update documentation to reflect changes

## Temporary Configuration

Add to `lib.rs` at the crate level:

```rust
// Temporarily allow warnings during cleanup
#![allow(clippy::all)]
#![allow(missing_docs)]
#![allow(missing_debug_implementations)]
```

This will be gradually removed as we address each category.

## Tracking Progress

We'll maintain a table in this document to track progress:

| Phase | Description | Status | Completion Date |
|-------|-------------|--------|----------------|
| 1 | Disable Warnings | Completed | March 29, 2024 |
| 2 | Debug Traits | In Progress | - |
| 3 | Async/Await | Not Started | - |
| 4 | Mutability | Not Started | - |
| 5 | Traits | Not Started | - |
| 6 | Documentation | Not Started | - |
| 7 | Pedantic | Not Started | - |

## Conclusion

This phased approach allows us to systematically address Clippy warnings while maintaining a functioning codebase. It prioritizes the most critical issues first and leaves documentation for later phases when the code structure is more stable. 