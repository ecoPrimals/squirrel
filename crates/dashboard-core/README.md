# Dashboard Core Crate

## Overview

The Dashboard Core crate provides the foundational models, traits, and services required for implementing monitoring dashboards across different UI interfaces. It acts as the backbone for dashboard functionality, handling data collection, processing, and providing a consistent API for UI implementations.

## Components

### Data Models

Core data structures that represent dashboard information:

- `DashboardData`: Complete representation of all dashboard data
- `SystemSnapshot`: System resource metrics (CPU, memory, disk)
- `NetworkSnapshot`: Network traffic and statistics
- `AlertsSnapshot`: Alert status and history
- `MetricsSnapshot`: Custom application metrics

### Dashboard Service

The `DashboardService` trait defines the contract for dashboard implementations:

- Data retrieval methods
- Historical metrics access
- Alert management
- Configuration updates
- Real-time update subscription

### Default Implementation

The `DefaultDashboardService` provides a standard implementation that can be used out-of-the-box or extended for custom needs.

## Integration

The dashboard core is designed to be UI-agnostic, allowing for multiple frontends:

- Terminal UI via the `ui-terminal` crate
- Web UI implementations
- Native desktop applications
- Mobile applications

## Usage

Here's a simple example of using the dashboard core:

```rust
use dashboard_core::{
    DashboardService, 
    DefaultDashboardService,
    DashboardConfig
};

async fn example() {
    // Create a dashboard service with default configuration
    let config = DashboardConfig::default();
    let (service, _) = DefaultDashboardService::new(config);
    
    // Start the service
    service.start().await.expect("Failed to start dashboard service");
    
    // Get current dashboard data
    let data = service.get_dashboard_data().await.expect("Failed to get dashboard data");
    println!("Current CPU usage: {}%", data.system.cpu_usage);
    
    // Subscribe to updates
    let mut updates = service.subscribe().await;
    
    // Process updates
    tokio::spawn(async move {
        while let Some(update) = updates.recv().await {
            println!("Received dashboard update: {:?}", update);
        }
    });
}
```

## UI Implementation Support

The core provides support for implementing various UI interfaces through:

1. Data structures optimized for display
2. Real-time update mechanisms
3. Configuration options for display preferences
4. History retrieval for trend visualization

Check the `ui-terminal` crate for an example of a complete UI implementation using this core.

## License

This project is licensed under the MIT License. 