---
title: Monitoring and Dashboard Integration Guide
version: 1.1.0
date: 2024-07-08
status: In Progress
---

# Monitoring and Dashboard Integration Guide

## Overview

This document provides guidance on integrating the `squirrel-monitoring` crate with the new `dashboard-core` and UI implementation crates (`ui-terminal`, etc.). After the separation of dashboard functionality from the monitoring crate, this guide explains how to use these components together effectively and outlines the steps needed to complete the migration to 100%.

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
      Collection            & Management              & User Interaction
         │                        │                         │
         │                        │                         │
         └────────────────────────┴─────────────────────────┘
                       WebSocket Communication
```

- **squirrel-monitoring**: Collects system and application metrics
- **dashboard-core**: Processes, stores, and manages dashboard data
- **ui-terminal** (or other UI implementations): Visualizes the dashboard data
- **WebSocket API**: Provides real-time communication between components

## Current Integration Status

The integration between these components is currently **85% complete**:

- ✅ **Monitoring to WebSocket**: Fully implemented and tested
- ✅ **WebSocket API**: Fully implemented with subscription management
- ✅ **Dashboard Core**: Fully implemented with proper data models
- 🔄 **UI Terminal**: Partially implemented (base structure complete, widgets in progress)
- 🔄 **Integration Testing**: In progress with key scenarios identified

## Integration Points

### 1. Monitoring to WebSocket API

The monitoring crate now exposes its data through a WebSocket API:

```rust
use squirrel_monitoring::websocket::{WebSocketServer, WebSocketConfig};
use std::net::SocketAddr;

async fn start_monitoring_websocket() -> Result<()> {
    // Create a WebSocket configuration
    let config = WebSocketConfig {
        host: "127.0.0.1".to_string(),
        port: 8765,
        update_interval: 1000,
        max_connections: 100,
        enable_compression: false,
        auth_required: false,
    };
    
    // Create and start the WebSocket server
    let server = WebSocketServer::new(config);
    server.start().await?;
    
    Ok(())
}
```

### 2. Dashboard Core to WebSocket API

The dashboard core connects to the WebSocket API to receive monitoring data:

```rust
use dashboard_core::websocket::{WebSocketClient, WebSocketClientConfig};
use dashboard_core::service::DefaultDashboardService;
use std::sync::Arc;

async fn connect_dashboard_to_websocket() -> Result<()> {
    // Create a WebSocket client configuration
    let config = WebSocketClientConfig {
        url: "ws://127.0.0.1:8765".to_string(),
        reconnect_interval: 5000,
        auth_token: None,
    };
    
    // Create a dashboard service
    let dashboard_service = Arc::new(DefaultDashboardService::new(
        dashboard_core::config::DashboardConfig::default()
    ));
    
    // Create and connect a WebSocket client
    let client = WebSocketClient::new(config, dashboard_service.clone());
    client.connect().await?;
    
    // Subscribe to metrics and alerts
    client.subscribe("system.metrics").await?;
    client.subscribe("system.alerts").await?;
    
    Ok(())
}
```

### 3. UI Terminal to Dashboard Core

The UI terminal connects to the dashboard core to visualize the data:

```rust
use dashboard_core::service::DashboardService;
use ui_terminal::TuiDashboard;
use std::sync::Arc;

async fn run_terminal_ui(dashboard_service: Arc<dyn DashboardService>) -> Result<()> {
    // Create and run the terminal UI
    let mut terminal_ui = TuiDashboard::new(dashboard_service);
    terminal_ui.run().await?;
    
    Ok(())
}
```

## Steps to Complete Integration (100%)

To complete the integration to 100%, the following steps must be taken:

### 1. Complete UI Terminal Implementation

#### 1.1 Update Widget Implementation

```rust
// From ui-terminal/src/widgets/metrics_widget.rs
use dashboard_core::models::Metrics;
use ratatui::widgets::{Widget, Block, Borders};
use ratatui::layout::Rect;
use ratatui::Frame;

// Update widget implementation to work with new data models
pub struct MetricsWidget<'a> {
    metrics: &'a Metrics,
    title: &'a str,
}

impl<'a> MetricsWidget<'a> {
    pub fn new(metrics: &'a Metrics, title: &'a str) -> Self {
        Self { metrics, title }
    }
}

impl<'a> Widget for MetricsWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create a block for the widget
        let block = Block::default()
            .title(self.title)
            .borders(Borders::ALL);
        
        // Render metrics data using new data model
        // ...
    }
}
```

#### 1.2 Fix Ratatui Compatibility Issues

```rust
// From ui-terminal/src/widgets/text_widget.rs
use ratatui::text::{Line, Span}; // Note: upgraded from old Spans to new Line

pub fn create_text_line(content: &str, style: Style) -> Line<'static> {
    // Create a line with spans (new ratatui API)
    Line::from(vec![Span::styled(content.to_string(), style)])
}
```

#### 1.3 Complete Event Handling with New Models

```rust
// From ui-terminal/src/app.rs
use dashboard_core::models::{Metrics, Alerts, SystemStatus};
use dashboard_core::service::DashboardService;

pub struct App {
    dashboard_service: Arc<dyn DashboardService>,
    // Other fields...
}

impl App {
    pub async fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Key(key) => {
                // Handle key events
                self.handle_key_event(key).await?;
            },
            Event::Tick => {
                // Update dashboard data
                self.update_dashboard_data().await?;
            },
            // Other events...
        }
        
        Ok(())
    }
    
    async fn update_dashboard_data(&mut self) -> Result<()> {
        // Get updated data from dashboard service using new data models
        self.metrics = self.dashboard_service.get_metrics().await?;
        self.alerts = self.dashboard_service.get_alerts().await?;
        self.system_status = self.dashboard_service.get_system_status().await?;
        
        Ok(())
    }
}
```

### 2. Create End-to-End Integration Example

Create a complete example that demonstrates the end-to-end integration:

```rust
// examples/dashboard_integration.rs
use std::sync::Arc;
use tokio::runtime::Runtime;

use squirrel_monitoring::metrics::MetricsCollector;
use squirrel_monitoring::websocket::{WebSocketServer, WebSocketConfig};
use dashboard_core::{
    config::DashboardConfig,
    service::DefaultDashboardService,
    websocket::WebSocketClient,
};
use ui_terminal::TuiDashboard;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Start the monitoring WebSocket server
    let ws_config = WebSocketConfig {
        host: "127.0.0.1".to_string(),
        port: 8765,
        update_interval: 1000,
        max_connections: 100,
        enable_compression: false,
        auth_required: false,
    };
    
    let server = WebSocketServer::new(ws_config);
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.start().await {
            eprintln!("WebSocket server error: {:?}", e);
        }
    });
    
    // 2. Create a dashboard service and connect to WebSocket
    let dashboard_config = DashboardConfig::default();
    let dashboard_service = Arc::new(DefaultDashboardService::new(dashboard_config));
    
    let ws_client_config = WebSocketClientConfig {
        url: "ws://127.0.0.1:8765".to_string(),
        reconnect_interval: 5000,
        auth_token: None,
    };
    
    let client = WebSocketClient::new(ws_client_config, dashboard_service.clone());
    client.connect().await?;
    
    // Subscribe to metrics and alerts
    client.subscribe("system.metrics").await?;
    client.subscribe("system.alerts").await?;
    
    // 3. Create and run the terminal UI
    let mut terminal_ui = TuiDashboard::new(dashboard_service.clone());
    terminal_ui.run().await?;
    
    // Clean up
    server_handle.abort();
    
    Ok(())
}
```

### 3. Complete Cross-Crate Integration Tests

Create comprehensive integration tests that verify the communication between all components:

```rust
// tests/integration_tests.rs
use std::sync::Arc;
use tokio::runtime::Runtime;

use squirrel_monitoring::metrics::MetricsCollector;
use squirrel_monitoring::websocket::{WebSocketServer, WebSocketConfig};
use dashboard_core::{
    config::DashboardConfig,
    service::DefaultDashboardService,
    websocket::WebSocketClient,
};
use ui_terminal::TuiDashboard;

#[tokio::test]
async fn test_monitoring_to_dashboard_core_integration() {
    // 1. Start a WebSocket server
    let ws_config = WebSocketConfig {
        host: "127.0.0.1".to_string(),
        port: 9000, // Use a different port for tests
        update_interval: 100,
        max_connections: 10,
        enable_compression: false,
        auth_required: false,
    };
    
    let server = WebSocketServer::new(ws_config);
    let server_handle = tokio::spawn(async move {
        server.start().await.unwrap();
    });
    
    // Wait for server to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // 2. Connect dashboard core to WebSocket
    let dashboard_config = DashboardConfig::default();
    let dashboard_service = Arc::new(DefaultDashboardService::new(dashboard_config));
    
    let ws_client_config = WebSocketClientConfig {
        url: "ws://127.0.0.1:9000".to_string(),
        reconnect_interval: 1000,
        auth_token: None,
    };
    
    let client = WebSocketClient::new(ws_client_config, dashboard_service.clone());
    client.connect().await.unwrap();
    
    // 3. Subscribe to test topics
    client.subscribe("test.metrics").await.unwrap();
    
    // 4. Publish test data
    let test_data = serde_json::json!({
        "cpu": 50.0,
        "memory": 1024,
        "timestamp": chrono::Utc::now().timestamp_millis(),
    });
    
    // Use a separate client to send data
    // ...
    
    // 5. Verify data is received by dashboard service
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    let metrics = dashboard_service.get_metrics().await.unwrap();
    assert!(metrics.contains_key("cpu"));
    assert_eq!(metrics.get("cpu").unwrap().value, 50.0);
    
    // Clean up
    server_handle.abort();
}
```

### 4. Fix Public API Documentation

Ensure all public APIs are properly documented:

```rust
/// Dashboard service interface for interacting with monitoring data.
///
/// # Examples
///
/// ```rust
/// use dashboard_core::{
///     config::DashboardConfig,
///     service::{DashboardService, DefaultDashboardService},
/// };
/// use std::sync::Arc;
///
/// # async fn example() {
/// let config = DashboardConfig::default();
/// let service = Arc::new(DefaultDashboardService::new(config));
///
/// // Get metrics
/// let metrics = service.get_metrics().await.unwrap();
///
/// // Get alerts
/// let alerts = service.get_alerts().await.unwrap();
/// # }
/// ```
#[async_trait]
pub trait DashboardService: Send + Sync + Debug {
    /// Get current system metrics
    async fn get_metrics(&self) -> Result<Metrics>;
    
    /// Get active alerts
    async fn get_alerts(&self) -> Result<Alerts>;
    
    /// Get system status
    async fn get_system_status(&self) -> Result<SystemStatus>;
    
    /// Update system metrics
    async fn update_metrics(&self, metrics: Metrics) -> Result<()>;
    
    /// Update alerts
    async fn update_alerts(&self, alerts: Alerts) -> Result<()>;
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

## WebSocket API Protocol

The WebSocket API follows a standard protocol for communication:

### Client Messages

```json
{
  "action": "subscribe",
  "topic": "system.metrics"
}
```

```json
{
  "action": "unsubscribe",
  "topic": "system.metrics"
}
```

```json
{
  "action": "query",
  "topic": "system.metrics",
  "parameters": {
    "timeRange": "1h"
  }
}
```

### Server Messages

```json
{
  "event": "data",
  "topic": "system.metrics",
  "data": {
    "cpu": 45.2,
    "memory": 2048,
    "timestamp": 1625097600000
  }
}
```

```json
{
  "event": "alert",
  "topic": "system.alerts",
  "data": {
    "id": "alert-123",
    "severity": "critical",
    "message": "CPU usage above threshold",
    "timestamp": 1625097600000
  }
}
```

## Integration Timeline

The integration is expected to be 100% complete by July 15, 2024, with the following milestones:

| Milestone | Target Date | Status |
|-----------|-------------|--------|
| UI Terminal Widget Implementation | July 10, 2024 | In Progress |
| UI Terminal Event Handling | July 12, 2024 | Not Started |
| Integration Tests | July 13, 2024 | In Progress |
| Example Applications | July 14, 2024 | Not Started |
| Final Documentation | July 15, 2024 | In Progress |

## Conclusion

The integration between the monitoring system and dashboard components is progressing well, with the core functionality already implemented. The focus now is on completing the UI terminal implementation, finalizing integration testing, and creating comprehensive documentation. With the steps outlined in this guide, the integration should reach 100% completion by the target date. 