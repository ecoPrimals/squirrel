# UI Terminal Crate

## Overview

The UI Terminal crate provides a terminal-based user interface for the Squirrel monitoring dashboard. It utilizes the `ratatui` library to create an interactive and visually appealing terminal UI experience.

## Features

- **Tab-based Navigation**: Easily switch between different dashboard views
- **Real-time Updates**: Live updates of system metrics, alerts, and network statistics
- **Interactive Controls**: Keyboard shortcuts for navigation and actions
- **Customizable Layout**: Configurable dashboard layouts and components
- **Responsive Design**: Adapts to different terminal sizes
- **Low Resource Usage**: Efficient implementation suitable for remote servers

## Components

### TUI Dashboard

The main dashboard component that handles terminal setup, event processing, and UI rendering:

```rust
use dashboard_core::{DashboardService, DefaultDashboardService};
use ui_terminal::TuiDashboard;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create dashboard service
    let config = dashboard_core::DashboardConfig::default();
    let (service, _) = DefaultDashboardService::new(config);
    service.start().await?;
    
    // Create and run TUI dashboard
    let dashboard_service = Arc::new(service);
    let mut tui = TuiDashboard::new(dashboard_service);
    tui.run().await?;
    
    Ok(())
}
```

### Application State

The `App` struct manages the application state, including:
- Dashboard data
- UI navigation state
- Event handling
- Update processing

### UI Rendering

The UI module handles rendering the dashboard with various components:
- System metrics panels
- Alert lists and details
- Network statistics
- Custom metrics charts

### Widgets

Custom widgets for visualizing different types of data:
- Spark lines for numeric trends
- Gauge widgets for utilization metrics
- Tables for detailed data
- Charts for historical data

### Event Handling

Cross-platform terminal event handling supporting:
- Keyboard navigation
- Mouse events (when supported)
- Terminal resize events

## Keyboard Controls

- **Tab/Shift+Tab**: Navigate between tabs
- **1-6**: Jump to specific tabs
- **↑/↓ or j/k**: Scroll content
- **r**: Refresh dashboard data
- **?**: Show/hide help
- **q or Ctrl+C**: Quit

## Installation

Add both the ui-terminal and dashboard-core crates to your project:

```toml
[dependencies]
dashboard-core = { path = "../dashboard-core" }
ui-terminal = { path = "../ui-terminal" }
```

## Usage

See the `/examples` directory for complete usage examples.

## License

This project is licensed under the MIT License.