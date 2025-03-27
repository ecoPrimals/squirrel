# Monitoring Crate

## Overview

The Monitoring crate provides a comprehensive system for real-time monitoring, alerting, and metrics collection. It is designed to be flexible, efficient, and scalable, capable of handling both small-scale and large-scale deployment scenarios.

## Features

- **Alerting System**: Configurable alerts based on metric thresholds
- **Component-based Architecture**: Modular design with pluggable components
- **Metric Collection**: Built-in collectors for various system and application metrics
- **Historical Data**: Storage and retrieval of historical metrics data
- **Custom Reporting**: Generate custom reports based on collected metrics

## Architecture

The monitoring system consists of several key components:

1. **Metric Collectors**: Gather metrics from various sources
2. **Alert Manager**: Processes and manages alerts based on metric thresholds
3. **Storage Backend**: Stores historical metrics data
4. **Report Generator**: Creates customized reports from collected data

```
+----------------+      +----------------+
|                |      |                |
|  Collectors    +----->+  Alert         |
|                |      |  Manager       |
+-------+--------+      +-------+--------+
        |                       |
        v                       v
+-------+-----------------------+-------+
|                                       |
|            Storage Backend            |
|                                       |
+----------------+---------------------++
                 |
        +--------v-------+
        |                |
        |  Report        |
        |  Generator     |
        +----------------+
```

## Components

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
        .with_storage("sqlite://metrics.db");
        
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

## API Documentation

For detailed API documentation, run:

```
cargo doc --open
```

## Examples

The `examples` directory contains several examples demonstrating different aspects of the monitoring system:

- `prometheus_component.rs`: Example showing how to use the Prometheus metric collector
- `plugin_example.rs`: Example demonstrating the plugin system for extending monitoring capabilities

## License

This project is licensed under the MIT License. 