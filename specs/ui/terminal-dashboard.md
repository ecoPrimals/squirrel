# Terminal Dashboard UI Specification

## Overview

This document outlines the current state, known issues, and roadmap for the terminal-based dashboard UI implementation. The terminal UI is implemented using the Ratatui library and provides monitoring and visualization capabilities in a terminal interface.

## Current State

The terminal dashboard UI implementation is currently transitioning from a legacy structure to a new architecture based on the `dashboard-core` crate. This transition involves several data structure mismatches and interface inconsistencies that need to be resolved.

### Components

1. **TuiDashboard**: The main application controller that initializes the terminal UI and event handling.
2. **App**: Manages application state and delegates events to WidgetManagers.
3. **WidgetManager**: A trait implemented by components that can render to the terminal UI.
4. **MonitoringAdapter**: Trait for components that provide metrics, health checks, alerts, and protocol status.
5. **MockMonitoringAdapter**: Test implementation of MonitoringAdapter providing simulated data.
6. **AlertManager**: Handles alert creation, tracking, and status.
7. **HelpSystem**: Provides contextual help information.

## Known Issues

### 1. Data Structure Mismatches

There's a significant mismatch between how `dashboard-core` defines data structures and how they're being used in the `ui-terminal` crate:

- **Protocol and ProtocolStatus**: Defined as enums in `dashboard-core` but treated as structs in implementation
- **Field name differences**: Field names and types in various structures like `Metrics`, `CpuMetrics`, etc. don't match between implementation and usage
- **AlertSeverity discrepancies**: Different definitions between `ui-terminal` and `dashboard-core`

### 2. Trait Implementation Issues

- The `MonitoringAdapter` trait methods in `mock.rs` don't match the trait definition in `mod.rs`
- Many of the async methods in the mock implementation aren't part of the trait
- The `WidgetManager` trait is missing required methods in some implementations

### 3. Type Conversions and Integration Issues

- `Arc<dyn DashboardService>` vs `dyn DashboardService` parameter inconsistencies
- `HelpSystem` is not properly wrapped in `Arc` where required
- Improper event handling integration between `TuiDashboard` and `App` structs

## Implementation Roadmap

### Phase 1: Fix Data Structure Alignment

1. Update `MockMonitoringAdapter` to correctly implement the `MonitoringAdapter` trait
2. Align types and field names with those defined in `dashboard-core`
3. Fix enum vs struct usage discrepancies for `Protocol` and `ProtocolStatus`

### Phase 2: Improve Trait Implementations

1. Update trait implementations to match trait definitions
2. Remove async methods that aren't part of the required traits
3. Complete implementation of required methods in all WidgetManager implementations

### Phase 3: Fix UI Event Handling

1. Properly integrate tokio/async operations with the terminal event loop
2. Ensure proper event delegation from `TuiDashboard` to `App`
3. Fix mouse, keyboard, and resize event handling

### Phase 4: Enhance UI Components

1. Implement proper metrics visualization
2. Add alert display and management
3. Improve protocol status display
4. Add help system integration

## Conclusion

The terminal UI dashboard implementation requires significant refactoring to align with the core crate's data structures and interfaces. Once these issues are resolved, the terminal UI will provide a robust monitoring interface compatible with the broader application architecture. 