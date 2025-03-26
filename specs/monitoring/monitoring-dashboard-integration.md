---
title: Monitoring and Dashboard Integration Guide
version: 1.0.0
date: 2024-06-22
status: Proposed
---

# Monitoring and Dashboard Integration Guide

## Overview

This document provides guidance on integrating the `squirrel-monitoring` crate with the new `dashboard-core` and UI implementation crates (`ui-terminal`, etc.). After the separation of dashboard functionality from the monitoring crate, this guide explains how to use these components together effectively.

## Architecture

The new architecture separates concerns between monitoring and dashboard visualization:

```
┌───────────────────┐      ┌───────────────────┐      ┌───────────────────┐
│                   │      │                   │      │                   │
│  squirrel-        │      │  dashboard-       │      │  ui-terminal      │
│  monitoring       │─────▶│  core             │─────▶│  (or other UIs)   │
│                   │      │                   │      │                   │
└───────────────────┘      └───────────────────┘      └───────────────────┘
      Metrics               Data Processing           Visualization
      Collection            & Management
```

- **squirrel-monitoring**: Collects system and application metrics
- **dashboard-core**: Processes, stores, and manages dashboard data
- **ui-terminal** (or other UI implementations): Visualizes the dashboard data

## Integration Points

### 1. Monitoring to Dashboard Core

The monitoring crate provides metrics that the dashboard core can consume. This integration can be achieved through:

#### Option A: Direct Integration

```rust
use squirrel_monitoring::metrics::{MetricsCollector, SystemMetrics};
use dashboard_core::{DashboardService, DashboardData};

async fn integrate_monitoring_with_dashboard() {
    // Create a metrics collector
    let metrics_collector = MetricsCollector::new();
    
    // Create a dashboard service
    let dashboard_service = DefaultDashboardService::new();
    
    // Set up periodic collection and updates
    tokio::spawn(async move {
        loop {
            // Collect metrics
            let system_metrics = metrics_collector.collect_system_metrics().await;
            let network_metrics = metrics_collector.collect_network_metrics().await;
            
            // Update dashboard data
            dashboard_service.update_system_metrics(system_metrics).await;
            dashboard_service.update_network_metrics(network_metrics).await;
            
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
}
```

#### Option B: Using a Metrics Adapter

```rust
use squirrel_monitoring::metrics::MetricsCollector;
use dashboard_core::adapters::MonitoringAdapter;

async fn use_metrics_adapter() {
    // Create a metrics collector
    let metrics_collector = MetricsCollector::new();
    
    // Create a dashboard service
    let dashboard_service = DefaultDashboardService::new();
    
    // Create an adapter
    let adapter = MonitoringAdapter::new(metrics_collector, dashboard_service);
    
    // Start the adapter (it will handle the periodic updates)
    adapter.start(Duration::from_secs(5)).await;
}
```

### 2. Dashboard Core to UI Terminal

The UI implementation can consume the dashboard data from the dashboard core:

```rust
use dashboard_core::{DashboardService, DashboardData};
use ui_terminal::TuiDashboard;
use std::sync::Arc;

async fn run_terminal_ui() {
    // Create a dashboard service
    let dashboard_service = Arc::new(DefaultDashboardService::new());
    
    // Create and run the terminal UI
    let mut terminal_ui = TuiDashboard::new(dashboard_service.clone());
    terminal_ui.run().await.expect("Failed to run terminal UI");
}
```

## Complete Integration Example

Here's a complete example showing how to integrate all three components:

```rust
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

use squirrel_monitoring::metrics::MetricsCollector;
use dashboard_core::{
    config::DashboardConfig,
    service::{DashboardService, DefaultDashboardService},
};
use ui_terminal::TuiDashboard;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create tokio runtime
    let runtime = Runtime::new()?;
    
    // Run the integrated application
    runtime.block_on(async {
        // Create a metrics collector
        let metrics_collector = MetricsCollector::new();
        
        // Create a dashboard configuration
        let config = DashboardConfig::default();
        
        // Create a dashboard service
        let dashboard_service = Arc::new(DefaultDashboardService::new(config));
        
        // Spawn a task to collect metrics and update dashboard
        let dashboard_service_clone = dashboard_service.clone();
        tokio::spawn(async move {
            loop {
                // Collect system metrics
                match metrics_collector.collect_system_metrics().await {
                    Ok(metrics) => {
                        if let Err(e) = dashboard_service_clone.update_system_metrics(metrics).await {
                            eprintln!("Failed to update system metrics: {}", e);
                        }
                    },
                    Err(e) => eprintln!("Failed to collect system metrics: {}", e),
                }
                
                // Collect network metrics
                match metrics_collector.collect_network_metrics().await {
                    Ok(metrics) => {
                        if let Err(e) = dashboard_service_clone.update_network_metrics(metrics).await {
                            eprintln!("Failed to update network metrics: {}", e);
                        }
                    },
                    Err(e) => eprintln!("Failed to collect network metrics: {}", e),
                }
                
                // Sleep before next collection
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });
        
        // Create and run the terminal UI
        let mut terminal_ui = TuiDashboard::new(dashboard_service);
        terminal_ui.run().await?;
        
        Ok::<(), Box<dyn std::error::Error>>(())
    })
}
```

## Dashboard Core Configuration

The dashboard core can be configured to adjust various aspects of its behavior:

```rust
use dashboard_core::config::DashboardConfig;
use std::time::Duration;

fn configure_dashboard() {
    let config = DashboardConfig {
        refresh_interval: Duration::from_secs(5),
        data_retention_period: Duration::from_secs(3600), // 1 hour
        enable_compression: true,
        compression_level: 6,
        max_connections: 100,
        debug: false,
    };
    
    // Use this config when creating the dashboard service
    let dashboard_service = DefaultDashboardService::new(config);
}
```

## Using Multiple UI Implementations

The dashboard core can be used with multiple UI implementations simultaneously:

```rust
async fn run_multiple_uis(dashboard_service: Arc<dyn DashboardService>) {
    // Spawn a terminal UI
    let dashboard_service_clone = dashboard_service.clone();
    tokio::spawn(async move {
        let mut terminal_ui = TuiDashboard::new(dashboard_service_clone);
        terminal_ui.run().await.unwrap();
    });
    
    // Spawn a web UI (when available)
    let dashboard_service_clone = dashboard_service.clone();
    tokio::spawn(async move {
        let mut web_ui = WebUIDashboard::new(dashboard_service_clone);
        web_ui.run().await.unwrap();
    });
    
    // Main application can continue running...
}
```

## Best Practices

1. **Shared Services**: Use `Arc<dyn DashboardService>` to share the dashboard service between components.

2. **Error Handling**: Always handle errors when interacting between components, especially during metric collection and updates.

3. **Configuration**: Tailor the dashboard configuration based on your specific needs, such as refresh rates and data retention.

4. **Resource Management**: Be mindful of resource usage, especially when collecting metrics at high frequencies.

5. **Shutdown Handling**: Implement proper shutdown procedures to ensure all components terminate gracefully.

## Troubleshooting

### Common Issues

1. **Metrics Not Appearing in Dashboard**
   - Check that the metrics collector is running and collecting data
   - Verify that the dashboard service is receiving updates
   - Ensure the UI is properly subscribing to updates

2. **High CPU Usage**
   - Consider reducing the frequency of metric collection
   - Check for any infinite loops or excessive polling

3. **Memory Leaks**
   - Ensure proper cleanup of resources
   - Check that subscriptions are being properly closed

## Conclusion

By following this integration guide, you can effectively combine the monitoring, dashboard core, and UI implementation components to create a comprehensive monitoring and visualization system. The separation of concerns allows for greater flexibility and extensibility while maintaining a cohesive user experience. 