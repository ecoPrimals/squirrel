# Squirrel Terminal UI

A terminal-based dashboard interface for monitoring system resources and the Squirrel environment.

## Overview

The Squirrel Terminal UI provides a lightweight, responsive interface for monitoring system metrics, protocol operations, and managing the Squirrel environment directly from any terminal. It's built using the Ratatui library and offers real-time visualization of system metrics with historical data tracking.

## Features

- **Dashboard View**: A comprehensive dashboard with multiple tabs for different metrics
- **Real-time Updates**: Continuous updates of system metrics with configurable refresh intervals
- **Historical Data**: Tracking and visualization of historical metrics data
- **Keyboard Navigation**: Intuitive keyboard shortcuts for navigating the dashboard
- **Resource Monitoring**: Real-time monitoring of CPU, memory, disk, and network usage
- **Chart Visualization**: Time-series charts for visualizing metric trends
- **Configurable**: Command-line options to customize refresh rate and history retention

## Screenshots

(Coming soon)

## Installation

The Terminal UI is included as part of the Squirrel project. To build it:

```bash
# From the project root
cargo build --package ui-terminal

# For optimized release build
cargo build --package ui-terminal --release
```

## Usage

Run the Terminal UI with default settings:

```bash
cargo run --bin ui-terminal
```

### Command Line Options

- `--interval <SECONDS>`: Set the update interval in seconds (default: 5)
- `--history-points <COUNT>`: Set the number of history points to retain (default: 1000)

Example with custom settings:

```bash
cargo run --bin ui-terminal -- --interval 2 --history-points 500
```

## Keyboard Controls

- `Tab`: Switch between tabs
- `Arrow Up/Down`: Navigate within a tab
- `Arrow Left/Right`: Adjust time range in charts
- `q`: Quit the application
- `?`: Show help overlay

## Architecture

The Terminal UI consists of several components:

- **Dashboard Core**: Provides metrics collection, configuration, and history tracking
- **Terminal UI**: Handles rendering, layout, and event processing
- **Chart Widget**: Visualizes time-series data with customizable options
- **Metrics Collection**: Real-time system metrics gathering using sysinfo
- **Event Handling**: Non-blocking input handling with async support

## Development

To contribute to the Terminal UI development:

1. Clone the repository
2. Navigate to the ui-terminal crate
3. Make your changes
4. Run tests with `cargo test`
5. Build and try your changes with `cargo run`

### Code Structure

- `src/app.rs`: Core application state management
- `src/ui/`: UI components and layouts
- `src/widgets/`: Custom widgets for the Terminal UI
- `src/event.rs`: Event handling system
- `src/bin/main.rs`: Entry point for the Terminal UI application

## Current Status

The Terminal UI is currently in active development. The core dashboard functionality is implemented with system metrics monitoring. Future enhancements will include:

- Protocol-specific monitoring
- Alert management
- Network monitoring
- Custom dashboard layouts
- Theme customization

## License

This project is licensed under the same license as the Squirrel project.