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
- ✅ MCP Integration:
  - ✅ ConnectionHealth monitoring
  - ✅ Metrics caching
  - ✅ Performance tracking
  - ✅ Error handling
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
│      (adapter.rs, mcp_adapter.rs) │
├───────────────────────────────────┤
│          Dashboard Core           │
│        (External Dependency)      │
└───────────────────────────────────┘
```

### Key Components

1. **Application (lib.rs, app.rs)**: Manages application state and event handling.
2. **UI Layer (ui.rs)**: Handles layout and rendering of the UI components.
3. **Widget System (widgets/*.rs)**: Provides specialized widgets for different types of data.
4. **Data Adapter (adapter.rs, mcp_adapter.rs)**: Transforms dashboard data and MCP metrics into UI-compatible format.
5. **Events System (events.rs)**: Handles keyboard and terminal events.

## Features

- System metrics visualization (CPU, memory, disk)
- Network activity monitoring
- Protocol metrics display
- Alerting system with acknowledgement
- Real-time updates
- Keyboard navigation
- Responsive layouts
- MCP protocol integration with connection health monitoring

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
- **r**: Refresh data / Reconnect MCP (when on Protocol tab)

## MCP Integration

This crate provides extensive integration with the Machine Context Protocol (MCP):

### Connection Health Monitoring

The `ConnectionHealth` structure provides detailed metrics about the quality of the MCP connection:

```rust
pub struct ConnectionHealth {
    pub latency_ms: f64,         // Latency in milliseconds
    pub packet_loss: f64,        // Packet loss percentage (0-100)
    pub stability: f64,          // Connection stability percentage (0-100)
    pub signal_strength: f64,    // Signal strength percentage (0-100)
    pub last_checked: DateTime<Utc>, // Last checked timestamp
}
```

These metrics help monitor and diagnose connection issues in real-time.

### Examples

The crate includes example programs demonstrating MCP integration:

#### MCP Monitor

A command-line tool that displays real-time MCP protocol metrics and connection health:

```bash
cargo run --example mcp_monitor -- [OPTIONS]
```

Options:
- `--mcp-server <ADDRESS>`: Specify the MCP server address
- `--mcp-interval <MS>`: Set update interval in milliseconds
- `--simulate-issues`: Enable simulation of connection issues

#### Custom Dashboard

A full dashboard application that integrates MCP metrics:

```bash
cargo run --example custom_dashboard -- --mcp [OPTIONS]
```

Options:
- `--mcp-server <ADDRESS>`: Specify the MCP server address
- `--mcp-interval <MS>`: Set update interval in milliseconds
- `--simulate-issues`: Enable simulation of connection issues

For detailed information on the MCP examples, see the [MCP_EXAMPLES.md](../../specs/ui/MCP_EXAMPLES.md) document.

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
- **mcp**: Machine Context Protocol (feature-gated)

## License

Same as the main Squirrel project.