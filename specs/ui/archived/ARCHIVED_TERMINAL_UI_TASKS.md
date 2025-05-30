# Terminal UI Implementation Task Checklist (Phased Approach)

## Overview
This document provides a detailed checklist of **current and upcoming tasks** for the Terminal UI implementation, reflecting the state **after a significant rollback** and organized according to the phased development plan outlined in `IMPLEMENTATION_PROGRESS.md`.

It uses Ratatui 0.24.0+ and integrates with `dashboard-core` data structures.

## Current State Summary
- **Implemented**: Basic Ratatui app structure, Overview tab, Health widget, Metrics widget, Chart widget (CPU/Mem).
- **Placeholders**: Network tab/widget, Alerts tab/widget.
- **Missing**: System tab, Protocol tab/widget, advanced MCP integration, performance optimizations.

---

## Phase 1: Core Functionality & Stabilization (Current Focus)
**Goal:** Implement all basic tabs and widgets, establish foundational testing.

### Core Widgets & Tabs
- [ ] **Network Widget Implementation** (`widgets/network.rs`)
  - [ ] Fetch and display network interface data (rx/tx bytes, rates) from `DashboardData`.
  - [ ] Implement rendering logic (e.g., table format).
- [ ] **Alerts Widget Implementation** (`widgets/alerts.rs`)
  - [ ] Fetch and display alerts from `DashboardData`.
  - [ ] Implement rendering logic (e.g., list format, severity colors).
- [ ] **Protocol Widget Implementation** (New file `widgets/protocol.rs`)
  - [ ] Fetch and display basic protocol status/metrics from `DashboardData`.
  - [ ] Implement rendering logic.
- [ ] **System Tab & Widgets** (`ui.rs`, new widgets)
  - [ ] Define required widgets (e.g., process list, system info).
  - [ ] Implement widgets and tab rendering.
- [ ] **Tab Implementation** (`ui.rs`)
  - [ ] Implement `render_network_tab` using `NetworkWidget`.
  - [ ] Implement `render_alerts_tab` using `AlertsWidget`.
  - [ ] Implement `render_protocol_tab` using `ProtocolWidget`.
  - [ ] Implement `render_system_tab` using new system widgets.
- [ ] **Refine Connection Status** (`app.rs`)
  - [ ] Improve logic for inferring `AppState.connection_status` from `DashboardData.protocol`.

### Foundational Testing
- [ ] **Health Widget Tests** (`widgets/health.rs`)
  - [ ] Add unit tests for `HealthCheck` logic and rendering.
- [ ] **Metrics Widget Tests** (`widgets/metrics.rs`)
  - [ ] Add unit tests for metrics display and formatting.
- [ ] **Chart Widget Tests** (`widgets/chart.rs`)
  - [ ] Add unit tests for chart rendering with different data scenarios.
- [ ] **Network Widget Tests** (`widgets/network.rs`)
  - [ ] Add unit tests once widget is implemented.
- [ ] **Alerts Widget Tests** (`widgets/alerts.rs`)
  - [ ] Add unit tests once widget is implemented.
- [ ] **Protocol Widget Tests** (`widgets/protocol.rs`)
  - [ ] Add unit tests once widget is implemented.
- [ ] **System Widget Tests** (New widget files)
  - [ ] Add unit tests once widgets are implemented.
- [ ] **App State Tests** (`app.rs`)
  - [ ] Add unit tests for state updates and health check generation.
- [ ] **Basic End-to-End Tests**
  - [ ] Implement basic app flow tests (startup, tab switching, data update).

### Basic Code Quality & Docs
- [ ] **Warnings Cleanup (Critical)**
  - [ ] Evaluate and fix or suppress critical compiler warnings.
- [ ] **Code Comments (Essential)**
  - [ ] Add documentation for essential core logic in `app.rs` and `ui.rs`.
  - [ ] Add basic documentation for new Phase 1 widgets.
- [ ] **Error Handling (Basic)**
  - [ ] Review and refine error handling in `app.rs` update logic.
  - [ ] Add more specific error types in `error.rs` if needed for Phase 1 features.

---

## Phase 2: Enhancements & Refinement
**Goal:** Improve usability, code quality, and test coverage of the core features.

### Feature Refinement
- [ ] **Chart Widget Enhancements** (`widgets/chart.rs`)
  - [ ] Improve axis scaling and labeling.
  - [ ] Add dynamic time window adjustment (if desired).
- [ ] **Layout & Display Refinement** (`ui.rs`, `widgets/*.rs`)
  - [ ] Review and improve layout consistency across tabs.
  - [ ] Enhance data display clarity and formatting.
- [ ] **User Input Handling** (`app.rs`, `event.rs`)
  - [ ] Review and refine keybinding logic.
  - [ ] Consider adding basic help screen (`?` key toggle).

### Comprehensive Code Quality & Docs
- [ ] **Warnings Cleanup (All)**
  - [ ] Address all remaining compiler warnings.
  - [ ] Document reasons for any intentionally unused code/variables.
- [ ] **Code Comments & Docs (`rustdoc`)**
  - [ ] Add comprehensive `rustdoc` for all public functions, structs, and modules.
  - [ ] Improve inline comments for complex logic.
- [ ] **Error Handling (Robustness)**
  - [ ] Enhance error reporting in the UI (e.g., status bar messages).
  - [ ] Ensure graceful handling of unexpected data provider errors.

### Testing Expansion
- [ ] **Unit Test Coverage**
  - [ ] Increase coverage for existing and Phase 1 components.
  - [ ] Add tests for edge cases and error conditions.
- [ ] **End-to-End Test Expansion**
  - [ ] Cover more user interactions (e.g., specific keybindings).
  - [ ] Test UI behavior with various `DashboardData` scenarios (e.g., empty data, errors).
- [ ] **Mocking Implementation**
  - [ ] Implement a basic mock `DashboardService` for more controlled testing.

---

## Phase 3+: Future Considerations (Optimization, Advanced MCP, Polish)
**Goal:** Re-introduce performance optimizations, implement deeper MCP integration, and add advanced features.

### Performance Optimization (Phase 3)
- [ ] **Profiling**
  - [ ] Profile Phase 1/2 application under load to identify bottlenecks.
- [ ] **Memory Optimization**
  - [ ] Re-evaluate `CompressedTimeSeries` or similar structures for history.
  - [ ] Investigate memory usage for large datasets.
- [ ] **Rendering Optimization**
  - [ ] Investigate `CachedWidget` or selective rendering.
  - [ ] Investigate viewport clipping.
- [ ] **CPU Usage Optimization**
  - [ ] Investigate adaptive polling/throttling.

### Advanced MCP Integration (Phase 4)
- [ ] Implement detailed `ConnectionHealth` struct and monitoring.
- [ ] Implement `ConnectionHealthWidget`.
- [ ] Implement UI-initiated reconnection logic (if needed).
- [ ] Implement MCP error console / advanced views.
- [ ] Investigate direct MCP client interaction (if needed).

### Advanced Features & Polish (Phase 5)
- [ ] Theming and Customization.
- [ ] User Preferences Persistence.
- [ ] Advanced Filtering/Sorting.
- [ ] Accessibility Improvements (Screen reader, etc.).
- [ ] Advanced Testing (Benchmarks, CI/CD Integration).
- [ ] Documentation (Architecture, Developer Guides).

--- 