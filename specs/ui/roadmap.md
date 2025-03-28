# Terminal UI Implementation Roadmap

## Current Status

We've successfully implemented the `MockMonitoringAdapter` to correctly match the `MonitoringAdapter` trait and use proper dashboard-core data structures. However, there are still several issues that need to be addressed for the UI terminal to compile and function correctly.

## Remaining Issues

### 1. Import Issues
- Duplicate imports (`Duration`, `Event`)
- Unused imports (clean up throughout the codebase)
- Missing imports (add where needed)
- Incorrect imports (replace/update)

### 2. Type Mismatches
- `Arc<dyn DashboardService>` vs `&dyn DashboardService`
- `Option<Metrics>` vs `Metrics` in the DashboardData structure
- `HelpSystem` vs `Arc<HelpSystem>`

### 3. `WidgetManager` Trait Implementation
- Missing methods: `tick`, `handle_key`, `handle_mouse`
- Need to implement these methods on all types that implement `WidgetManager`

### 4. Async/Sync Issues
- The `run` method is not async but contains async operations
- Tokio channel usage without proper await
- Future handling in synchronous context 

### 5. DefaultDashboardService Issues
- Missing `default()` implementation
- May need to implement manually or ensure trait is derived

### 6. Other Components
- Fix `HelpSection` type in the `HelpSystem`
- Fix `McpClientAdapter` to implement `Debug`
- Fix or remove `MockMcpClient` usage

## Implementation Plan

### Phase 1: Fix Import and Basic Type Issues
1. Resolve duplicate imports by removing or renaming (use `as`)
2. Fix missing imports
3. Clean up unused imports
4. Fix type mismatches with proper conversion or wrapping

### Phase 2: Fix WidgetManager Trait
1. Update the `WidgetManager` trait to include the missing methods
2. Ensure all implementations provide these methods
3. Fix the `App` struct to use these methods correctly

### Phase 3: Fix Async/Sync Issues
1. Convert `run` to an async function OR
2. Use a different approach to handle async code in a synchronous context
3. Fix Tokio channel usage with proper buffer size and await

### Phase 4: Fix Service Integration
1. Implement `default()` for `DefaultDashboardService` 
2. Fix the conversion from `Arc<dyn DashboardService>` to `&dyn DashboardService`
3. Ensure proper handling of `Metrics` vs `Option<Metrics>`

### Phase 5: Fix Remaining Component Issues
1. Define and implement the `HelpSection` type
2. Fix `McpClientAdapter` debug implementation
3. Fix or remove `MockMcpClient` usage

## Prioritized Tasks

1. ✅ Fix `MockMonitoringAdapter` implementation
2. Fix import issues - especially duplicates and missing imports
3. Update the `WidgetManager` trait and implementations
4. Fix async/sync issues in the `run` method
5. Fix `DefaultDashboardService` issues
6. Address remaining component issues

## Expected Outcome

After addressing these issues, the ui-terminal crate should compile successfully and integrate properly with the dashboard-core crate, providing a functional terminal UI for the monitoring dashboard. 