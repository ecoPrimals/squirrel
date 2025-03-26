# Monitoring Crate

## Overview

The Monitoring crate provides a comprehensive system for real-time monitoring, alerting, and visualization of metrics. It is designed to be flexible, efficient, and scalable, capable of handling both small-scale and large-scale deployment scenarios.

## Features

- **Real-time Dashboards**: Web-based dashboards for visualizing metrics in real-time
- **WebSocket Communication**: Efficient bi-directional communication between clients and servers
- **Message Compression**: Automatic compression for large messages to reduce network traffic
- **Alerting System**: Configurable alerts based on metric thresholds
- **Component-based Architecture**: Modular design with pluggable components
- **Metric Collection**: Built-in collectors for various system and application metrics
- **Historical Data**: Storage and retrieval of historical metrics data
- **Custom Reporting**: Generate custom reports based on collected metrics

## Architecture

The monitoring system consists of several key components:

1. **Metric Collectors**: Gather metrics from various sources
2. **Dashboard Server**: WebSocket server providing real-time updates to clients
3. **Alert Manager**: Processes and manages alerts based on metric thresholds
4. **Storage Backend**: Stores historical metrics data
5. **Report Generator**: Creates customized reports from collected data
6. **Web Frontend**: User interface for interacting with the monitoring system

```
+----------------+      +----------------+      +----------------+
|                |      |                |      |                |
|  Collectors    +----->+  Dashboard     +<---->+  Web Clients   |
|                |      |  Server        |      |                |
+-------+--------+      +--------+-------+      +----------------+
        |                        |
        v                        v
+-------+------------------------+-------+
|                                        |
|            Storage Backend             |
|                                        |
+----------------+---------------------+-+
                 |                     |
        +--------v-------+    +--------v-------+
        |                |    |                |
        |  Alert Manager |    |  Report        |
        |                |    |  Generator     |
        +----------------+    +----------------+
```

## WebSocket Protocol

The monitoring system uses a WebSocket-based protocol for real-time communication. This enables:

- Low-latency updates
- Bi-directional communication
- Efficient bandwidth usage through message batching and compression
- Support for multiple clients with different subscription needs

For detailed WebSocket protocol documentation, see [websocket_protocol.md](./docs/websocket_protocol.md).

## Components

### Dashboard

The dashboard component provides real-time visualization of system metrics and status. Key features include:

- Component-based subscription model
- Real-time updates via WebSocket
- Customizable views and layouts
- Support for various visualization types (charts, gauges, tables, etc.)
- Filtering and sorting capabilities
- Alert notifications and acknowledgment
- Historical data viewing

### Alerts

The alerting system monitors metrics and triggers notifications when thresholds are exceeded. It provides:

- Configurable alert thresholds
- Multiple severity levels
- Notification channels (email, webhook, etc.)
- Alert acknowledgment and resolution tracking
- Alert history and reporting
- Silencing and muting capabilities

### Metrics

The metrics system collects, processes, and stores various types of measurements:

- System metrics (CPU, memory, disk, network)
- Application metrics (request rates, errors, latency)
- Custom metrics
- Aggregation and statistical processing
- Tagging and metadata

### Network Monitoring

The network monitoring component focuses on tracking network-related metrics:

- Bandwidth usage
- Connection counts
- Protocol-specific metrics
- Latency and packet loss
- Network topology mapping

### Reports

The reporting module generates insights from collected metrics:

- Scheduled reports
- Custom report templates
- Multiple output formats (PDF, HTML, JSON)
- Data aggregation and analysis
- Historical comparisons

## Usage Examples

### Basic Setup

```rust
use monitoring::{Monitor, Config};

fn main() {
    // Create a new monitor with default configuration
    let config = Config::default()
        .with_dashboard(true)
        .with_storage("sqlite://metrics.db")
        .with_port(8765);
        
    let monitor = Monitor::new(config);
    
    // Start the monitoring system
    monitor.start();
    
    // Register custom metrics
    monitor.register_metric("app.requests", "count");
    monitor.register_metric("app.latency", "milliseconds");
    
    // Record metrics in your application
    monitor.record("app.requests", 1.0);
    monitor.record("app.latency", 42.5);
    
    // Run your application...
    
    // Shutdown monitoring when done
    monitor.shutdown();
}
```

### Creating a Custom Collector

```rust
use monitoring::{Collector, MetricValue};
use std::collections::HashMap;

struct CustomCollector;

impl Collector for CustomCollector {
    fn name(&self) -> &str {
        "custom_collector"
    }
    
    fn collect(&self) -> HashMap<String, MetricValue> {
        let mut metrics = HashMap::new();
        
        // Add your custom metrics collection logic here
        metrics.insert("custom.metric1".to_string(), MetricValue::Gauge(42.0));
        metrics.insert("custom.metric2".to_string(), MetricValue::Counter(100));
        
        metrics
    }
    
    fn interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(60) // Collect every minute
    }
}

// Register your collector
fn main() {
    let monitor = monitoring::Monitor::new(monitoring::Config::default());
    monitor.register_collector(Box::new(CustomCollector));
    monitor.start();
    
    // Your application code...
}
```

### Setting Up Alerts

```rust
use monitoring::{Monitor, AlertRule, AlertSeverity};

fn main() {
    let monitor = monitoring::Monitor::new(monitoring::Config::default());
    
    // Create an alert rule
    let rule = AlertRule::new("high_cpu_usage")
        .with_metric("system.cpu.usage")
        .with_threshold(90.0)
        .with_comparison(">")
        .with_severity(AlertSeverity::Warning)
        .with_message("CPU usage is high")
        .with_duration(std::time::Duration::from_mins(5)); // Alert only if condition persists
    
    // Register the alert rule
    monitor.register_alert_rule(rule);
    
    // Your application code...
}
```

### Connecting a WebSocket Client

```javascript
// JavaScript example
const ws = new WebSocket('ws://localhost:8765/ws');

ws.onopen = () => {
  // Subscribe to components
  ws.send(JSON.stringify({
    type: 'subscribe',
    componentId: 'system_cpu'
  }));
};

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Received update:', data);
  
  // Handle different message types
  if (data.type === 'update') {
    updateDashboard(data.componentId, data.data);
  } else if (data.type === 'alert') {
    showAlertNotification(data);
  }
};
```

## API Documentation

For detailed API documentation, run:

```
cargo doc --open
```

## Examples

The `examples` directory contains several examples demonstrating different aspects of the monitoring system:

- `basic_monitor.rs`: Simple monitoring setup with default configuration
- `custom_metrics.rs`: Registering and recording custom metrics
- `websocket_client.rs`: Client for connecting to the WebSocket server
- `websocket_load_test.rs`: Load testing the WebSocket server
- `alert_demo.rs`: Demonstrating alert configuration and handling
- `report_generation.rs`: Generating custom reports from metrics data

To run an example:

```
cargo run --example websocket_client
```

## Configuration

The monitoring system can be configured through the `Config` struct, environment variables, or a configuration file. Key configuration options include:

- WebSocket server host and port
- Storage backend settings
- Collector intervals
- Dashboard settings
- Logging configuration
- Alert notification settings

For a complete list of configuration options, see [configuration.md](./docs/configuration.md).

## Building and Testing

The monitoring crate is in active development. Here's how to build and test it:

### Building

To build the crate:

```bash
# From the crate root directory
cargo build
```

### Testing

Not all tests are currently stable. We provide scripts to run only the stable tests:

```bash
# Linux/macOS
./scripts/build_and_test.sh

# Windows
.\scripts\build_and_test.ps1
```

### Test Status

The current status of tests is documented in [tests/TEST_STATUS.md](tests/TEST_STATUS.md). Some tests are currently failing due to API mismatches and are being updated.

Stable tests include:
- WebSocket compression tests
- WebSocket integration tests

### Examples

To build and verify the examples:

```bash
cargo build --examples
```

To run a specific example:

```bash
cargo run --example websocket_client
```

## Contributing

When contributing to this crate, please:

1. Make sure the code builds with `cargo build`
2. Run the stable tests with the provided scripts
3. If you modify tests, update the TEST_STATUS.md file accordingly
4. Update documentation when changing public APIs

## License

This project is licensed under the [MIT License](../LICENSE). 