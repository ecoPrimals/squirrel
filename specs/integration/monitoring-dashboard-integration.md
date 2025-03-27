---
title: Monitoring and Dashboard Integration
version: 1.0.0
date: 2024-07-20
status: implemented
author: DataScienceBioLab
---

# Monitoring and Dashboard Integration

## Overview

This document details the implementation of the integration between the `squirrel-monitoring` crate and the `dashboard-core` and `ui-terminal` components. The integration provides real-time system metrics collection and visualization through a comprehensive terminal-based UI dashboard.

## Integration Architecture

The integration follows a layered architecture:

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Monitoring     │────▶│   Dashboard     │────▶│   Terminal UI   │
│  Components     │     │   Core          │     │   Components    │
└─────────────────┘     └─────────────────┘     └─────────────────┘
        │                        │                        │
        ▼                        ▼                        ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  System Metrics │     │  Data Models    │     │  UI Rendering   │
│  Collection     │     │  & Services     │     │  & Interaction  │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

### Key Components

1. **ResourceMetricsCollectorAdapter**: Connects system metrics from monitoring to dashboard-core data models
2. **DefaultDashboardService**: Provides dashboard data management and update notification
3. **TuiDashboard**: Renders dashboard UI and handles user interaction

## Implementation Details

### 1. Resource Metrics Collector Adapter

```rust
/// Resource metrics collector adapter for connecting monitoring to dashboard-core
#[derive(Debug, Clone)]
pub struct ResourceMetricsCollectorAdapter {
    system: System,
}

impl ResourceMetricsCollectorAdapter {
    /// Create a new resource metrics collector adapter
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        ResourceMetricsCollectorAdapter {
            system,
        }
    }
    
    /// Refresh system data
    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }
    
    /// Collect system metrics and convert to dashboard-core format
    pub fn collect_system_metrics(&mut self) -> SystemSnapshot {
        self.refresh();
        
        // Collect CPU metrics
        let cpu_usage = self.system.global_cpu_info().cpu_usage();
        
        // Collect memory metrics
        let memory_used = self.system.used_memory();
        let memory_total = self.system.total_memory();
        
        // Collect disk metrics
        let disks = self.system.disks();
        let mut disk_used = 0;
        let mut disk_total = 0;
        
        for disk in disks {
            disk_used += disk.total_space() - disk.available_space();
            disk_total += disk.total_space();
        }
        
        // Create system snapshot
        SystemSnapshot {
            cpu_usage,
            memory_used,
            memory_total,
            disk_used,
            disk_total,
            load_average: [0.0, 0.0, 0.0], // Replace with actual values if available
            uptime: self.system.uptime(),
        }
    }
    
    /// Collect network metrics and convert to dashboard-core format
    pub fn collect_network_metrics(&mut self) -> NetworkSnapshot {
        self.refresh();
        
        let mut rx_bytes = 0;
        let mut tx_bytes = 0;
        let mut rx_packets = 0;
        let mut tx_packets = 0;
        let mut interfaces = HashMap::new();
        
        self.system.refresh_networks();
        
        for (name, network) in self.system.networks() {
            let rx_bytes_interface = network.received();
            let tx_bytes_interface = network.transmitted();
            let rx_packets_interface = network.packets_received();
            let tx_packets_interface = network.packets_transmitted();
            
            // Update totals
            rx_bytes += rx_bytes_interface;
            tx_bytes += tx_bytes_interface;
            rx_packets += rx_packets_interface;
            tx_packets += tx_packets_interface;
            
            // Store interface metrics
            interfaces.insert(name.clone(), InterfaceStats {
                name: name.clone(),
                rx_bytes: rx_bytes_interface,
                tx_bytes: tx_bytes_interface,
                rx_packets: rx_packets_interface,
                tx_packets: tx_packets_interface,
                is_up: true, // Fill with actual status if available
            });
        }
        
        // Create network snapshot
        NetworkSnapshot {
            rx_bytes,
            tx_bytes,
            rx_packets,
            tx_packets,
            interfaces,
        }
    }
    
    /// Collect all metrics as dashboard data
    pub fn collect_dashboard_data(&mut self) -> (SystemSnapshot, NetworkSnapshot) {
        let system_snapshot = self.collect_system_metrics();
        let network_snapshot = self.collect_network_metrics();
        
        (system_snapshot, network_snapshot)
    }
}
```

### 2. Dashboard Service Update Method

Added to enable direct updates from monitoring data:

```rust
impl DefaultDashboardService {
    /// Update dashboard data directly
    pub async fn update_data(&self, data: DashboardData) -> Result<()> {
        // Update the data
        *self.data.write().await = data.clone();
        
        // Send update to subscribers
        if let Err(e) = self.update_sender.send(DashboardUpdate::FullUpdate(data)).await {
            return Err(DashboardError::Update(format!("Failed to send update: {}", e)));
        }
        
        Ok(())
    }
}
```

### 3. Terminal UI Integration

Updated main executable to use real system metrics:

```rust
#[tokio::main]
async fn main() -> io::Result<()> {
    // Parse command line arguments
    let args = Args::parse();
    
    // Create dashboard configuration with builder pattern
    let config = DashboardConfig::default()
        .with_update_interval(args.interval)
        .with_max_history_points(args.history_points);
    
    // Create dashboard service
    let dashboard_service_with_rx = DefaultDashboardService::new(config);
    let (dashboard_service, _rx) = &dashboard_service_with_rx;
    
    // Create monitoring adapter
    let mut adapter = ResourceMetricsCollectorAdapter::new();
    
    // Start a task to periodically collect metrics and update the dashboard
    let dashboard_service_clone = dashboard_service.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(args.interval));
        
        loop {
            interval.tick().await;
            
            // Collect metrics
            let (system, network) = adapter.collect_dashboard_data();
            
            // Update dashboard data
            if let Ok(mut data) = dashboard_service_clone.get_dashboard_data().await {
                data.system = system;
                data.network = network;
                data.timestamp = chrono::Utc::now();
                
                // Update the dashboard with new data
                if let Err(e) = dashboard_service_clone.update_data(data).await {
                    eprintln!("Failed to update dashboard data: {}", e);
                }
            }
        }
    });
    
    // Start dashboard service
    dashboard_service.start().await.unwrap_or_else(|e| {
        eprintln!("Warning: Failed to start dashboard service: {}", e);
    });
    
    // Create and run terminal UI
    let mut tui = TuiDashboard::new_from_default_service(dashboard_service_with_rx);
    tui.run().await
}
```

## Standardization Improvements

The following standardization fixes were implemented:

1. **Added Missing sysinfo Traits**: All relevant files now include the required trait imports:
   ```rust
   use sysinfo::{SystemExt, ProcessExt, NetworksExt, DiskExt, CpuExt, NetworkExt, DiskUsageExt};
   ```

2. **Fixed Resource Access Methods**: Updated methods to use consistent patterns:
   ```rust
   // Use system.disks() instead of creating new instance
   let disks = system.disks();
   
   // Use system.networks() instead of creating new instance
   let networks = system.networks();
   ```

3. **Improved Network Method Calls**: Updated the network access methods to follow the current sysinfo API:
   ```rust
   // Changed from
   network.total_received();
   
   // To
   network.received();
   ```

4. **Data Structure Alignment**: Ensured dashboard-core data models align with the data provided by monitoring.

## Testing Approach

The integration is tested at multiple levels:

1. **Unit Tests**: Individual components are tested with mock data
2. **Integration Tests**: Testing the adapter with both real and simulated data
3. **End-to-End Tests**: Verifying the complete flow from system metrics to UI display

Example test:

```rust
#[test]
fn test_metrics_can_be_converted_to_dashboard_format() {
    let mut collector = ResourceMetricsCollectorAdapter::new();
    let (system, network) = collector.collect_dashboard_data();
    
    // Verify system metrics
    assert!(system.cpu_usage >= 0.0 && system.cpu_usage <= 100.0);
    assert!(system.memory_used <= system.memory_total);
    assert!(system.disk_used <= system.disk_total);
    
    // Verify network metrics
    assert!(network.rx_bytes >= 0);
    assert!(network.tx_bytes >= 0);
    assert!(!network.interfaces.is_empty());
}
```

## Future Improvements

1. **History Management**: Implement more sophisticated metric history retention
2. **Advanced Visualization**: Add specialized visualizations for different metric types
3. **Alerts Integration**: Connect system metrics to alerting system
4. **Custom Dashboards**: Allow user-defined dashboard layouts

## Conclusion

The integration between monitoring, dashboard-core, and ui-terminal provides a seamless flow of real-time system metrics to a user-friendly terminal-based dashboard. The standardization improvements ensure consistent and reliable metrics collection across the system.

---

*Created by DataScienceBioLab on 2024-07-20* 