---
description: ENFORCE standardized code quality in the squirrel-plugins crate through gradual linting improvements
globs: crates/plugins/**/*.rs
---

# Squirrel Plugins Crate Linting Plan

## Context
- When refactoring the plugins crate for code quality
- When implementing Rust best practices 
- When improving documentation standards
- When preparing for open source contribution
- When ensuring security and safety of plugin code

## Summary
This document outlines a phased approach to address linting issues in the `squirrel-plugins` crate identified through Clippy and other static analysis tools. The plan allows for gradual improvements while the team continues to focus on other core development tasks.

## Priority Levels
- **P0**: Critical issues affecting correctness or security
- **P1**: Issues affecting maintainability or performance
- **P2**: Style and documentation issues
- **P3**: Optional improvements

## Phase 1: Critical Fixes

### Safety and Error Handling (P0)
- [ ] Fix all unwrap() calls that could panic
  - Target files:
    - `crates/plugins/src/galaxy/adapter_plugin.rs`
  - Approach: Replace with proper error handling and propagation

### Missing Documentation for Public APIs (P1)
- [ ] Add missing documentation for public modules:
  - [ ] `simple_test`
  - [ ] `simple_test_utils`
  - [ ] `plugins/context_impl`
- [ ] Add `# Errors` sections to all functions returning `Result`
  - Target files:
    - `galaxy/adapter_plugin.rs`
    - `galaxy/example.rs`
    - `commands/mod.rs`
    - `simple_test_utils.rs`
    - `test_utils/mod.rs` 
- [ ] Add `# Panics` sections to functions that can panic
  - Target files:
    - `galaxy/adapter_plugin.rs`

### Milestone 1 Success Criteria
- All P0 issues resolved
- At least 50% of missing documentation issues addressed
- No unwrap() calls without documented panic conditions

## Phase 2: Performance and Resource Management

### Resource Management Improvements (P1)
- [ ] Fix temporaries with significant `Drop`
  - Target files:
    - `state.rs`
    - `galaxy/adapter_plugin.rs`
  - Example: Replace multi-line operations with chained method calls

### Pattern Matching Optimization (P1)
- [ ] Replace single match statements with if let or map_or_else
  - Target files:
    - `galaxy/adapter_plugin.rs`
    - `plugins/context_impl.rs`

### Unnecessary Async (P1)
- [ ] Remove async for functions with no await points
  - Target files:
    - `discovery.rs`
    - `manager.rs`
    - `security/signature.rs`

### Milestone 2 Success Criteria
- All resource management issues addressed
- Pattern matching optimized
- Unnecessary async usage removed

## Phase 3: Code Style and Structure

### Function Attributes (P2)
- [ ] Add `#[must_use]` to functions returning important values
  - Prioritize builder pattern methods returning `Self`
- [ ] Convert eligible functions to `const fn`
  - Target files:
    - `galaxy/adapter_plugin.rs`
    - `simple_test.rs`
    - `mcp/mod.rs`
    - `commands/mod.rs`

### Type Improvements (P2)
- [ ] Add `Debug` implementations for types missing them
  - Target types:
    - `TestPluginContext`
- [ ] Fix unnecessary `Result` wrapping
  - Target files:
    - `galaxy/adapter_plugin.rs`

### Module Visibility (P2)
- [ ] Fix unreachable pub items by using `pub(super)`
  - Target files:
    - `galaxy/adapter_plugin.rs`
  
### Self Usage (P2)
- [ ] Replace structure name repetition with `Self`
  - Target files:
    - `simple_test_utils.rs`

### String Formatting (P3)
- [ ] Update to use modern string formatting with direct variable interpolation
  - Target files:
    - `galaxy/example.rs`
    - `simple_test_utils.rs`

### Milestone 3 Success Criteria
- All function attributes properly applied
- All types have appropriate debug implementations
- Module visibility properly scoped
- Modern string formatting used throughout the codebase

## Phase 4: Dead Code and Final Cleanup

### Dead Code Elimination (P3)
- [ ] Address unused methods
  - `initialize_plugins_default` in manager.rs
  - `check_dependency_cycles` in manager.rs
- [ ] Address unused fields
  - `state_manager` in manager.rs
  - `base_dir` in state.rs

### Final Linting Pass (P3)
- [ ] Run Clippy with all lints enabled and address any remaining issues
- [ ] Run `cargo fix --edition-idioms` to catch easy-to-fix issues

### Milestone 4 Success Criteria
- No dead code warnings
- Clean Clippy output with no warnings (or all warnings intentionally allowed)

## Implementation Strategy

### Gradual Approach
- Focus on 1-2 files per week
- Prioritize files that are actively being modified for other reasons
- Use fixup commits that can be easily reviewed and merged

### Testing Requirements
- Each lint fix must maintain or improve test coverage
- Run tests after each set of changes to ensure no regressions

### Documentation Updates
- Update API documentation during the process
- Document any intentional deviations from linting rules

## Automation Opportunities
- Create a custom Clippy configuration file to standardize linting rules
- Implement CI checks to prevent new linting issues
- Consider using a tool like `cargo-lints` to track progress

## Team Communication
- Provide weekly updates on linting progress
- Document any patterns discovered that should be applied across other crates
- Share lessons learned with other teams to improve overall code quality

## Success Metrics
- Reduction in Clippy warnings
- Improved documentation coverage percentage
- Cleaner code reviews with fewer style issues

<version>1.0.0</version> 