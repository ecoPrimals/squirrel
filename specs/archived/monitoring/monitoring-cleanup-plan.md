---
title: Monitoring Crate Cleanup Plan
version: 1.1.0
date: 2024-06-24
status: In Progress
priority: High
---

# Monitoring Crate Cleanup Plan

## Overview

This document outlines the plan for cleaning up the `squirrel-monitoring` crate by removing dashboard functionality that has been migrated to the dedicated `dashboard-core` and `ui-terminal` crates. The cleanup aims to maintain the monitoring crate's core functionality while removing all dashboard-related code that's now maintained in separate crates.

## Current State

The `squirrel-monitoring` crate currently contains both monitoring functionality and dashboard components. The dashboard functionality has been migrated to:

- `dashboard-core` - Core dashboard functionality, data models, and services
- `ui-terminal` - Terminal UI implementation of the dashboard

According to the migration status document, the dashboard core crate and initial monitoring crate cleanup have been completed, but there are still some integration issues to resolve.

## Cleanup Tasks

### 1. Remove Dashboard Module

The entire `src/dashboard/` directory should be removed from the monitoring crate, including:

- `src/dashboard/mod.rs`
- `src/dashboard/manager.rs`
- `src/dashboard/server.rs`
- `src/dashboard/component.rs`
- `src/dashboard/config.rs`
- `src/dashboard/error.rs`
- `src/dashboard/stats.rs`
- `src/dashboard/security.rs`
- `src/dashboard/secure_server.rs`
- `src/dashboard/components/` (directory)
- `src/dashboard/plugins/` (directory)
- `src/dashboard/adapter.rs`
- `src/dashboard/tests.rs`
- `src/dashboard/websocket_protocol.md`
- `src/dashboard/api_documentation.md`

### 2. Update Exports in lib.rs

Modify `src/lib.rs` to remove dashboard module exports, for example:

```rust
// Remove lines like:
pub mod dashboard;
pub use dashboard::{DashboardManager, DashboardComponent};
```

### 3. Update Cargo.toml

Remove dashboard-specific dependencies that are no longer needed:

- Review dependencies that were only used for the dashboard functionality
- Remove or make optional any dashboard-specific features
- Update the crate description to reflect its focused purpose

Specific dependencies to review include:
- `ratatui` (if only used for dashboard)
- `websocket` or `tokio-tungstenite` (if only used for dashboard)
- `crossterm` (if only used for dashboard UI)
- Any other UI-specific libraries

### 4. Update Examples

Remove or update examples that showcase dashboard functionality:

- Remove `examples/secure_dashboard.rs`
- Remove `examples/dashboard_plugin_example.rs`
- Remove `examples/analytics_dashboard_integration.rs`
- Update any remaining examples that reference dashboard functionality

### 5. Update Documentation

Update documentation to reflect the changes:

- Update `README.md` to remove dashboard-related sections
- Update any API documentation that references dashboard components
- Add migration guidance for users to switch to new dashboard crates

### 6. Implement Backward Compatibility 

For backward compatibility:

1. Optionally create an adapter crate `squirrel-monitoring-dashboard-adapter` that depends on both `squirrel-monitoring` and `dashboard-core` and provides compatibility APIs
2. Document migration paths with examples for users
3. Create comprehensive examples showing how to use the monitoring crate with the new dashboard crates

### 7. Clean Up Test Files

Remove dashboard-specific tests and update test infrastructure:

- Remove dashboard test files
- Update test fixtures that may include dashboard components
- Add integration tests between monitoring and new dashboard crates

## Testing Plan

After the cleanup, ensure that the monitoring crate still works as expected:

1. Run the existing test suite (excluding dashboard tests)
2. Verify that monitoring functionality is unaffected
3. Ensure compilation succeeds without dashboard components
4. Test integration with the new dashboard-core and ui-terminal crates
5. Create new end-to-end tests that integrate monitoring and dashboard

## Backward Compatibility Strategy

To maintain backward compatibility:

1. Create the adapter crate mentioned above
2. Provide clear migration documentation with code examples
3. Version the monitoring crate appropriately (major version bump)
4. Provide a transition period before removing adapter support

## Timeline

1. Complete dashboard-core and ui-terminal crate implementations (1-2 days)
2. Implement the monitoring crate cleanup (1 day)
3. Create and test backward compatibility adapter (1 day)
4. Update documentation and examples (1 day)
5. Create integration tests (1-2 days)
6. Release updated versions with appropriate semver bumps (1 day)

Total estimated time: 6-8 days

## Risks and Mitigation

- **Risk**: Breaking changes for existing users
  - **Mitigation**: Provide compatibility adapter and clear migration documentation

- **Risk**: Functionality gaps between old and new implementation
  - **Mitigation**: Comprehensive testing of both implementations before final release

- **Risk**: Test coverage may be reduced
  - **Mitigation**: Maintain and adapt existing tests, add new integration tests

- **Risk**: Integration complexity between monitoring and dashboard
  - **Mitigation**: Create well-defined interfaces and comprehensive examples

## Next Steps

1. Complete the UI Terminal crate implementation
2. Begin monitoring crate cleanup process
3. Develop backward compatibility adapter
4. Update documentation and testing infrastructure
5. Create integration examples 