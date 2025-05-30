# Terminal UI Implementation Task Checklist (Post-Rollback)

## Overview
This document provides a detailed checklist of **current and upcoming tasks** for the Terminal UI implementation, reflecting the state **after a significant rollback**. It uses Ratatui 0.24.0+ and integrates with `dashboard-core` data structures.

## Current State Summary
- **Implemented**: Basic Ratatui app structure, Overview tab, Health widget, Metrics widget, Chart widget (CPU/Mem).
- **Placeholders**: Network tab/widget, Alerts tab/widget.
- **Missing**: System tab, Protocol tab/widget, advanced MCP integration, performance optimizations.

## High Priority Tasks

### Core Functionality & Missing Widgets
- [ ] **Network Widget Implementation** (`widgets/network.rs`)
  - [ ] Fetch and display network interface data (rx/tx bytes, rates) from `DashboardData`.
  - [ ] Implement rendering logic (e.g., table format).
- [ ] **Alerts Widget Implementation** (`widgets/alerts.rs`)
  - [ ] Fetch and display alerts from `DashboardData`.
  - [ ] Implement rendering logic (e.g., list format, severity colors).
  - [ ] Consider basic acknowledgment/clearing mechanism (if feasible).
- [ ] **Protocol Widget Implementation** (New file `widgets/protocol.rs`)
  - [ ] Fetch and display basic protocol status/metrics from `DashboardData`.
  - [ ] Implement rendering logic.
- [ ] **Tab Implementation** (`ui.rs`)
  - [ ] Implement `render_network_tab` using `NetworkWidget`.
  - [ ] Implement `render_alerts_tab` using `AlertsWidget`.
  - [ ] Implement `render_protocol_tab` using `ProtocolWidget`.
  - [ ] Implement `render_system_tab` (needs corresponding widgets/data).
- [ ] **Refine Connection Status** (`app.rs`)
  - [ ] Improve logic for inferring `AppState.connection_status` from `DashboardData.protocol`.

### Testing (Foundation)
- [ ] **Health Widget Tests** (`widgets/health.rs`)
  - [ ] Add unit tests for `HealthCheck` logic and rendering.
- [ ] **Metrics Widget Tests** (`widgets/metrics.rs`)
  - [ ] Add unit tests for metrics display and formatting.
- [ ] **Chart Widget Tests** (`widgets/chart.rs`)
  - [ ] Add unit tests for chart rendering with different data scenarios.
- [ ] **App State Tests** (`app.rs`)
  - [ ] Add unit tests for state updates and health check generation.

## Medium Priority Tasks

### Testing (Expansion)
- [ ] **Network Widget Tests** (`widgets/network.rs`)
  - [ ] Add unit tests once widget is implemented.
- [ ] **Alerts Widget Tests** (`widgets/alerts.rs`)
  - [ ] Add unit tests once widget is implemented.
- [ ] **Protocol Widget Tests** (`widgets/protocol.rs`)
  - [ ] Add unit tests once widget is implemented.
- [ ] **End-to-End Tests**
  - [ ] Implement basic app flow tests (startup, tab switching, data update).

### Code Quality & Documentation
- [ ] **Code Comments**
  - [ ] Add documentation for existing widgets (`Health`, `Metrics`, `Chart`).
  - [ ] Document `app.rs` state and logic.
  - [ ] Document `ui.rs` rendering functions.
  - [ ] Add documentation for newly implemented widgets.
- [ ] **Warnings Cleanup**
  - [ ] Evaluate and fix or suppress remaining compiler warnings.
  - [ ] Document reasons for any intentionally unused code/variables.
- [ ] **Error Handling**
  - [ ] Review and refine error handling in `app.rs` update logic.
  - [ ] Add more specific error types in `error.rs` if needed.

### Feature Refinement
- [ ] **Chart Widget Enhancements** (`widgets/chart.rs`)
  - [ ] Improve axis scaling and labeling.
  - [ ] Add dynamic time window adjustment (if desired).
- [ ] **System Tab Implementation** (`ui.rs`, new widgets)
  - [ ] Define required widgets (e.g., process list, system info).
  - [ ] Implement widgets and tab rendering.

## Low Priority / Future Considerations

### Performance Optimization
- [ ] **Profiling**
  - [ ] Profile current application under load to identify bottlenecks.
- [ ] **Memory Optimization** (Consider if needed after profiling)
  - [ ] Re-evaluate `CompressedTimeSeries` or similar structures for history.
  - [ ] Investigate memory usage for large datasets.
- [ ] **Rendering Optimization** (Consider if needed after profiling)
  - [ ] Investigate `CachedWidget` or selective rendering.
  - [ ] Investigate viewport clipping.
- [ ] **CPU Usage Optimization** (Consider if needed after profiling)
  - [ ] Investigate adaptive polling/throttling.

### Advanced MCP Integration (Future Phase)
- [ ] Implement direct MCP client interaction (separate from `DashboardService` if needed).
- [ ] Implement detailed `ConnectionHealth` struct and monitoring.
- [ ] Implement UI-initiated reconnection logic.
- [ ] Implement caching strategies.
- [ ] Implement `ConnectionHealthWidget`.
- [ ] Implement MCP error console / advanced views.

### Testing (Advanced)
- [ ] Create test utilities and helper functions.
- [ ] Implement mock dashboard service / MCP client.
- [ ] Add adapter transformation tests (if adapters are used).
- [ ] Implement performance benchmark tests.
- [ ] Set up CI/CD pipeline for automated testing.

### Documentation (Advanced)
- [ ] Update architecture documentation.
- [ ] Create test pattern documentation / developer guides.
- [ ] Document performance best practices.

### Accessibility & Localization
- [ ] Implement screen reader support.
- [ ] Implement internationalization.

## Implementation Strategy (Revised)

1.  Implement missing core widgets and tabs (`Network`, `Alerts`, `Protocol`, `System`).
2.  Establish foundational unit test coverage for existing and new widgets.
3.  Improve code quality (warnings, comments, basic error handling).
4.  Refine existing features (connection status, charts).
5.  Expand test coverage (E2E, more unit tests).
6.  (Future) Profile and implement performance optimizations *as needed*.
7.  (Future) Plan and implement advanced MCP integration features.

---

*Last updated: 2024-03-29* 