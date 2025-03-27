# Terminal UI Dashboard

A terminal-based UI for monitoring system metrics, providing a lightweight and responsive interface for the Squirrel Dashboard.

## Features

- Real-time system metrics monitoring
- Protocol monitoring with message statistics, transaction tracking, and error monitoring
- Network metrics visualization
- Historical data charts
- Responsive layout adapting to terminal size
- Keyboard navigation and shortcuts
- Integration with monitoring system

## Architecture

The terminal UI uses a modular architecture with clear separation of concerns:

```
┌─────────────────┐     ┌─────────────────┐
│   Dashboard     │     │   Terminal UI    │
│     Core        │────▶│    Components    │
└─────────────────┘     └─────────────────┘
        │                        │
        ▼                        ▼
┌─────────────────┐     ┌─────────────────┐
│  Metrics        │     │      UI         │
│  Collection     │     │    Rendering    │
└─────────────────┘     └─────────────────┘
        ▲
        │
┌─────────────────┐
│   Monitoring    │
│    Adapter      │
└─────────────────┘
```

- **Dashboard Core**: Handles metrics collection, history tracking, and configuration
- **Terminal UI**: Renders the UI based on the current state
- **Monitoring Adapter**: Connects to system monitoring for metrics collection

## Components

### Dashboard Tabs

- **Overview**: Summary of all key metrics
- **System**: Detailed system metrics (CPU, memory, disk)
- **Protocol**: Message statistics, transaction tracking, and error monitoring
- **Tools**: Configuration and management tools
- **Alerts**: Alert notifications and management
- **Network**: Network interface statistics and throughput

### Widgets

- **Chart Widget**: Time-series visualization
- **Metrics Widget**: Metrics display and statistics
- **Network Widget**: Network throughput visualization
- **Alerts Widget**: Alert listing and management
- **Protocol Widget**: Protocol metrics visualization with message, transaction, and error statistics

## Integration with Monitoring System

The Terminal UI integrates with the monitoring system through the `MonitoringToDashboardAdapter` which:

1. Collects system metrics from sysinfo and other monitoring tools
2. Converts metrics to the dashboard-core format
3. Provides real-time updates to the UI

## Usage

```
# Run with default configuration
cargo run --bin main

# Run with monitoring integration enabled
cargo run --bin main -- --monitoring

# Customize update interval and history size
cargo run --bin main -- --interval 10 --history-points 500 --monitoring
```

## Keyboard Shortcuts

- `1-6`: Select tab
- `Tab`: Next tab
- `Shift+Tab`: Previous tab
- `j/Down`: Scroll down
- `k/Up`: Scroll up
- `r`: Refresh data
- `?`: Toggle help
- `q/Ctrl+c`: Quit

## Recent Updates

- **Protocol Monitoring**: Added protocol metrics visualization with message, transaction, and error statistics
- **Monitoring Integration**: Implemented adapter pattern for connecting to system monitoring
- **Real-time Updates**: Added asynchronous update mechanism for smooth UI updates
- **UI Improvements**: Enhanced UI layout and responsiveness

## License

MIT