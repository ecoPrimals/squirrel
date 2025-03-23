# Dashboard Component Registry

## Overview

The component registry provides a catalog of available visualization components for the monitoring dashboard. Each component is designed to display specific types of metrics and can be customized for different use cases.

## Core Components

### Charts

#### Line Chart

Displays time series data as a continuous line.

```json
{
  "id": "system_cpu_chart",
  "name": "CPU Usage Over Time",
  "component_type": "Chart",
  "config": {
    "chart_type": "line",
    "y_axis_label": "Usage %",
    "x_axis_label": "Time",
    "show_grid": true,
    "line_color": "#1E88E5",
    "fill_area": true,
    "fill_opacity": 0.2
  }
}
```

#### Bar Chart

Displays categorical data with rectangular bars.

```json
{
  "id": "memory_by_process",
  "name": "Memory Usage by Process",
  "component_type": "Chart",
  "config": {
    "chart_type": "bar",
    "y_axis_label": "Memory (MB)",
    "x_axis_label": "Process",
    "bar_colors": ["#1E88E5", "#42A5F5", "#90CAF9"]
  }
}
```

#### Pie Chart

Displays data as a circular graph divided into slices.

```json
{
  "id": "disk_usage_pie",
  "name": "Disk Usage Distribution",
  "component_type": "Chart",
  "config": {
    "chart_type": "pie",
    "show_legend": true,
    "show_labels": true,
    "label_format": "percent",
    "color_scheme": "pastel"
  }
}
```

### Gauges

#### Circular Gauge

Displays a single value within a range using a circular meter.

```json
{
  "id": "cpu_gauge",
  "name": "Current CPU Usage",
  "component_type": "Gauge",
  "config": {
    "gauge_type": "circular",
    "min_value": 0,
    "max_value": 100,
    "units": "%",
    "thresholds": [
      {"value": 60, "color": "#FFC107", "label": "Warning"},
      {"value": 85, "color": "#F44336", "label": "Critical"}
    ]
  }
}
```

#### Linear Gauge

Displays a single value within a range using a linear meter.

```json
{
  "id": "memory_gauge",
  "name": "Memory Usage",
  "component_type": "Gauge",
  "config": {
    "gauge_type": "linear",
    "orientation": "horizontal",
    "min_value": 0,
    "max_value": 32,
    "units": "GB",
    "show_ticks": true
  }
}
```

### Tables

#### Data Table

Displays structured data in rows and columns.

```json
{
  "id": "process_table",
  "name": "Running Processes",
  "component_type": "Table",
  "config": {
    "columns": [
      {"key": "pid", "name": "PID", "sortable": true},
      {"key": "name", "name": "Process Name", "sortable": true},
      {"key": "cpu", "name": "CPU %", "sortable": true},
      {"key": "memory", "name": "Memory", "sortable": true},
      {"key": "status", "name": "Status", "sortable": false}
    ],
    "pagination": true,
    "items_per_page": 10,
    "enable_search": true
  }
}
```

### Stat Cards

#### Metric Card

Displays a single metric with optional trend indicator.

```json
{
  "id": "requests_per_second",
  "name": "Requests Per Second",
  "component_type": "StatCard",
  "config": {
    "primary_metric": "rps",
    "secondary_metric": "change_percent",
    "icon": "network",
    "trend_period": "1h",
    "show_sparkline": true
  }
}
```

### Timeline

Displays events along a time axis.

```json
{
  "id": "system_events",
  "name": "System Events Timeline",
  "component_type": "Timeline",
  "config": {
    "show_time": true,
    "group_by": "category",
    "color_by": "severity",
    "max_items": 100,
    "auto_scroll": true
  }
}
```

### Heatmap

Displays data intensity using color variations.

```json
{
  "id": "request_latency_heatmap",
  "name": "Request Latency Heatmap",
  "component_type": "Heatmap",
  "config": {
    "x_axis": "time",
    "y_axis": "endpoint",
    "color_scheme": "viridis",
    "cell_size": {"width": 10, "height": 10},
    "legend": true
  }
}
```

### Network Graph

Displays network topology and relationships.

```json
{
  "id": "service_dependencies",
  "name": "Service Dependencies",
  "component_type": "NetworkGraph",
  "config": {
    "node_size_by": "traffic",
    "edge_width_by": "requests",
    "layout": "force_directed",
    "show_labels": true,
    "enable_zoom": true,
    "enable_drag": true
  }
}
```

### Log Viewer

Displays log entries with filtering and highlighting.

```json
{
  "id": "application_logs",
  "name": "Application Logs",
  "component_type": "LogViewer",
  "config": {
    "log_levels": ["error", "warn", "info", "debug"],
    "default_level": "info",
    "enable_search": true,
    "enable_filtering": true,
    "max_entries": 1000,
    "auto_scroll": true,
    "timestamp_format": "YYYY-MM-DD HH:mm:ss.SSS"
  }
}
```

### Alert List

Displays active and historical alerts.

```json
{
  "id": "active_alerts",
  "name": "Active Alerts",
  "component_type": "Alert",
  "config": {
    "show_severity": true,
    "show_timestamp": true,
    "show_acknowledgement": true,
    "filter_by": {"status": "active"},
    "group_by": "source",
    "auto_refresh": true,
    "refresh_interval": 30
  }
}
```

## Custom Components

You can create custom components by extending the core components or implementing completely new visualizations.

### Custom Component Registration

```rust
// Register a custom component type
dashboard_manager.register_component_type("3d_surface_plot", |data, config| {
    // Render implementation for the custom component
    // ...
});

// Create a component using the custom type
let custom_component = Component {
    id: "temperature_surface",
    name: "Temperature Surface Plot",
    component_type: ComponentType::Custom("3d_surface_plot".to_string()),
    config: ComponentConfig::default()
        .with_title("Temperature Distribution")
        .with_refresh_interval(Duration::from_secs(10)),
    data: // ...
};

dashboard_manager.register_component(custom_component)?;
```

## Component Data Structure

Each component requires data in a specific format:

```rust
pub struct ComponentData {
    pub timestamp: u64,
    pub values: serde_json::Value, // Component-specific data structure
    pub metadata: Option<serde_json::Value>, // Optional metadata
}
```

## Component Layout

Components can be arranged in various layouts:

```json
{
  "layout": {
    "type": "grid",
    "columns": 4,
    "rows": 3,
    "components": [
      {"id": "cpu_gauge", "x": 0, "y": 0, "width": 1, "height": 1},
      {"id": "memory_gauge", "x": 1, "y": 0, "width": 1, "height": 1},
      {"id": "system_cpu_chart", "x": 0, "y": 1, "width": 2, "height": 1},
      {"id": "process_table", "x": 2, "y": 0, "width": 2, "height": 2},
      {"id": "active_alerts", "x": 0, "y": 2, "width": 4, "height": 1}
    ]
  }
}
```

## Component Theming

Components can be themed consistently:

```json
{
  "theme": {
    "base": "dark",
    "colors": {
      "primary": "#1976D2",
      "secondary": "#424242",
      "success": "#4CAF50",
      "warning": "#FFC107",
      "error": "#F44336",
      "background": "#121212",
      "text": "#FFFFFF"
    },
    "typography": {
      "fontFamily": "Roboto, sans-serif",
      "fontSize": 14
    },
    "components": {
      "Chart": {
        "gridColor": "#424242",
        "tickColor": "#757575"
      },
      "Gauge": {
        "trackColor": "#424242"
      }
    }
  }
}
```

## Dashboard Templates

Pre-configured dashboard templates combining multiple components:

```json
{
  "template": "system_monitoring",
  "name": "System Monitoring Dashboard",
  "description": "Comprehensive system monitoring dashboard",
  "components": [
    {"type": "Gauge", "id": "cpu_gauge", "position": {"x": 0, "y": 0}},
    {"type": "Gauge", "id": "memory_gauge", "position": {"x": 1, "y": 0}},
    {"type": "Chart", "id": "system_cpu_chart", "position": {"x": 0, "y": 1}},
    {"type": "Table", "id": "process_table", "position": {"x": 2, "y": 0}},
    {"type": "Alert", "id": "active_alerts", "position": {"x": 0, "y": 2}}
  ]
}
```

## Best Practices

1. **Component Selection**: Choose the most appropriate component type for your data.
2. **Data Refresh**: Set appropriate refresh intervals for each component.
3. **Thresholds**: Configure thresholds to highlight important changes in data.
4. **Layout**: Arrange components logically, with related metrics near each other.
5. **Responsive Design**: Consider how components will appear on different screen sizes.

## Further Resources

- [Dashboard API Documentation](./dashboard_api.md)
- [WebSocket Protocol](./websocket_protocol.md)
- [Component Customization Guide](./component_customization.md) 