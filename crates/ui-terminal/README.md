# Terminal UI for Squirrel Dashboard

This crate provides a terminal-based user interface for the Squirrel dashboard, built with [Ratatui](https://github.com/ratatui-org/ratatui).

## Status: In Progress

This crate has been updated to be compatible with Ratatui 0.24.0+, which introduces breaking changes from previous versions. All core widgets have been updated.

### Implementation Progress

- ✅ Core infrastructure (app.rs, lib.rs, events.rs)
- ✅ UI rendering framework (ui.rs)
- ✅ All key widgets:
  - ✅ MetricsWidget
  - ✅ ProtocolWidget
  - ✅ AlertsWidget
  - ✅ NetworkWidget
  - ✅ ChartWidget
  - ✅ HealthWidget
- 🔄 Testing and optimization
- 🔄 Enhanced features
- 🔄 Documentation and finalization

## Overview

The Terminal UI provides a comprehensive dashboard for monitoring system metrics, network activity, and alerts in a terminal environment. It is designed to be efficient, keyboard-driven, and to work across different terminal types.

## Architecture

The Terminal UI follows a layered architecture:

```
┌───────────────────────────────────┐
│            Application            │
│           (lib.rs, app.rs)        │
├───────────────────────────────────┤
│              UI Layer             │
│              (ui.rs)              │
├───────────────────────────────────┤
│           Widget System           │
│         (widgets/*.rs)            │
├───────────────────────────────────┤
│            Data Adapter           │
│           (adapter.rs)            │
├───────────────────────────────────┤
│          Dashboard Core           │
│        (External Dependency)      │
└───────────────────────────────────┘
```

### Key Components

1. **Application (lib.rs, app.rs)**: Manages application state and event handling.
2. **UI Layer (ui.rs)**: Handles layout and rendering of the UI components.
3. **Widget System (widgets/*.rs)**: Provides specialized widgets for different types of data.
4. **Data Adapter (adapter.rs)**: Transforms dashboard data into UI-compatible format.
5. **Events System (events.rs)**: Handles keyboard and terminal events.

## Features

- System metrics visualization (CPU, memory, disk)
- Network activity monitoring
- Protocol metrics display
- Alerting system with acknowledgement
- Real-time updates
- Keyboard navigation
- Responsive layouts

## Usage

To run the Terminal UI:

```rust
use dashboard_core::DashboardService;
use ui_terminal::TuiDashboard;
use std::sync::Arc;

// Create dashboard service
let dashboard_service = Arc::new(MyDashboardService::new());

// Create and run UI
let mut ui = TuiDashboard::new(dashboard_service);
ui.run().await?;
```

## Widget System

The Terminal UI includes the following widgets:

- **MetricsWidget**: Displays system metrics like CPU and memory usage.
- **ProtocolWidget**: Shows protocol-specific metrics.
- **AlertsWidget**: Displays alerts with severity indicators.
- **NetworkWidget**: Shows network activity and interfaces.
- **ChartWidget**: Renders time-series data as charts.
- **HealthWidget**: Displays system health status.

## Keyboard Shortcuts

- **Tab/Shift+Tab**: Navigate between tabs
- **1-6**: Select specific tabs
- **Up/Down**: Scroll content
- **q/Esc**: Quit
- **?**: Show help
- **r**: Refresh data

## Development

### Building

```bash
cargo build --package ui-terminal
```

### Running Tests

```bash
cargo test --package ui-terminal
```

### Running Example

```bash
cargo run --example dashboard
```

## Dependencies

- **ratatui**: Terminal UI framework (v0.24.0+)
- **crossterm**: Terminal control
- **tokio**: Async runtime
- **dashboard-core**: Core dashboard functionality

## License

Same as the main Squirrel project.