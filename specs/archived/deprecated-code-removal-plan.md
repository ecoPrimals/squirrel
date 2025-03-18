# Deprecated Code Removal Plan

## Overview
This document outlines the plan for removing all deprecated code from the monitoring system after the successful migration to dependency injection patterns.

## Timeline
- **Start Date**: 2024-03-19
- **End Date**: 2024-03-20
- **Release Target**: v0.3.0

## Phase 1: Tool Metrics Completion (Day 1 Morning)

### Tasks
1. Add deprecation annotations to Tool Metrics Collector:
   ```rust
   #[deprecated(
       since = "0.2.0",
       note = "Use DI pattern with ToolMetricsCollectorFactory::create_collector() instead"
   )]
   pub fn get_collector() -> Option<Arc<ToolMetricsCollector>> {
       COLLECTOR.get().cloned()
   }
   ```

2. Update documentation:
   - Add migration guide
   - Update API docs
   - Add DI examples

3. Run tests:
   ```bash
   cargo test --package squirrel-core --lib monitoring::metrics::tool
   ```

4. Verify Clippy:
   ```bash
   cargo clippy --package squirrel-core -- -D warnings
   ```

5. Update migration tracker:
   - Mark Tool Metrics as complete
   - Update progress section
   - Add completion date

## Phase 2: Usage Analysis (Day 1 Afternoon)

### Static Analysis
```bash
# Find all deprecated function usage
cargo check --all-targets 2>&1 | grep -i "warning: use of deprecated"

# Find specific function calls
rg "get_manager\(\)" --type rust
rg "ensure_factory\(\)" --type rust
```

### Usage Report Template
```markdown
## Deprecated Function Usage Report

### Alert Manager
- Functions: initialize_factory(), get_factory(), etc.
- Usage Count: X
- Locations: [file:line]
- Migration Complexity: [Low/Medium/High]

[Repeat for each component]
```

### Impact Analysis
- List affected components
- Identify critical paths
- Note potential breaking changes

## Phase 3: Migration Preparation (Day 1 Evening)

### Migration Scripts
Create automated scripts for common patterns:

```rust
// Pattern 1: Global Manager Access
// Before:
let manager = get_manager().unwrap();

// After:
let factory = ManagerFactory::new();
let manager = factory.create_manager();

// Pattern 2: Factory Initialization
// Before:
initialize_factory(config)?;
let factory = get_factory().unwrap();

// After:
let factory = ManagerFactory::with_config(config);
```

### Test Templates
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_di_pattern_usage() {
        let factory = ComponentFactory::new();
        let component = factory.create_component();
        assert!(component.is_valid());
    }

    #[test]
    fn test_adapter_pattern() {
        let adapter = create_component_adapter();
        assert!(adapter.is_valid());
    }
}
```

## Phase 4: Code Removal (Day 2 Morning)

### Removal Order
1. Alert Manager
2. Dashboard Manager
3. Network Monitor
4. Notification Manager
5. Metric Exporter
6. Protocol Metrics
7. Monitoring Service
8. Tool Metrics Collector

### Removal Process
For each component:
1. Remove global state:
   ```rust
   // Remove static variables
   static MANAGER: OnceLock<Arc<Manager>> = OnceLock::new();
   ```

2. Remove deprecated functions:
   ```rust
   // Remove these functions
   #[deprecated]
   pub fn get_manager() -> Option<Arc<Manager>>

   #[deprecated]
   pub fn initialize_factory(config: Config) -> Result<()>
   ```

3. Update tests:
   ```rust
   // Remove tests that use deprecated functions
   // Add tests for DI patterns
   ```

4. Update documentation:
   - Remove references to global access
   - Update examples to use DI
   - Add migration notes

## Phase 5: Testing (Day 2 Afternoon)

### Test Suite
```bash
# Run all tests
cargo test --all-features
cargo test --all-targets

# Run Clippy
cargo clippy --all-targets -- -D warnings

# Run specific component tests
cargo test --package squirrel-core --lib monitoring::alerts
cargo test --package squirrel-core --lib monitoring::dashboard
# ... etc
```

### Integration Testing
1. Component Independence:
   - Verify each component works in isolation
   - Test factory creation
   - Test adapter pattern

2. Component Interaction:
   - Test dependency injection
   - Verify event propagation
   - Check error handling

3. Performance Testing:
   - Measure initialization time
   - Compare memory usage
   - Check resource utilization

## Phase 6: Documentation (Day 2 Evening)

### CHANGELOG.md Update
```markdown
# 0.3.0 (Breaking Changes)

## Removed
- All deprecated global access functions from monitoring components:
  - Alert Manager global functions
  - Dashboard Manager global functions
  - Network Monitor global functions
  - Notification Manager global functions
  - Metric Exporter global functions
  - Protocol Metrics global functions
  - Monitoring Service global functions
  - Tool Metrics Collector global functions

## Added
- Comprehensive migration guide (docs/migration/di-pattern.md)
- New examples for dependency injection patterns
- Additional test coverage for DI patterns

## Migration Guide
See docs/migration/di-pattern.md for detailed migration instructions.
```

### Documentation Updates
1. API Documentation:
   - Remove deprecated function docs
   - Add DI pattern examples
   - Update method signatures

2. Migration Guide:
   - Step-by-step migration instructions
   - Code examples
   - Best practices

3. Architecture Documentation:
   - Update component diagrams
   - Document DI patterns
   - Add dependency graphs

## Phase 7: Release Preparation

### Version Update
1. Update Cargo.toml:
   ```toml
   [package]
   name = "squirrel-core"
   version = "0.3.0"
   ```

2. Create git tag:
   ```bash
   git tag -a v0.3.0 -m "Release 0.3.0 - Remove deprecated code"
   ```

### Release Notes
```markdown
# Release 0.3.0

## Breaking Changes
- Removed all deprecated global access functions
- Completed migration to dependency injection pattern

## Migration
- See docs/migration/di-pattern.md for migration guide
- Use provided adapters for gradual migration
- Contact DataScienceBioLab team for support

## New Features
- Improved testing capabilities
- Better error handling
- Reduced coupling between components
```

## Rollback Plan

### Triggers
- Critical bugs found
- Major performance issues
- Integration failures

### Process
1. Revert commit:
   ```bash
   git revert v0.3.0
   ```

2. Release patch:
   ```bash
   # Create emergency patch
   git checkout -b hotfix/0.2.x
   # Fix critical issues
   git tag -a v0.2.x -m "Emergency patch"
   ```

3. Notify users:
   - Send notification
   - Update documentation
   - Provide workarounds

## Success Criteria
- All tests passing
- No Clippy warnings
- Documentation updated
- No global state remaining
- All components using DI pattern
- Performance metrics within acceptable range

## Support Plan
- Monitor issue tracker
- Provide migration assistance
- Update documentation based on feedback
- Regular check-ins with teams
- Performance monitoring 