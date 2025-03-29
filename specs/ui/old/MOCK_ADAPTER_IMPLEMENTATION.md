---
title: "Mock Adapter Implementation for Terminal UI"
date: "2025-03-28"
status: "Complete"
version: "1.0.0"
---

# Mock Adapter Implementation for Terminal UI

## Overview

This document describes the implementation of the mock adapter for the terminal UI in the Squirrel monitoring system. The mock adapter provides realistic test data for development and testing of the terminal UI, allowing for the display of dashboard metrics without requiring a connection to the actual monitoring system.

## Implementation Details

### Core Components

The mock adapter implementation includes the following key components:

1. **`MockAdapter` Struct**: A simplified adapter that implements the `McpMetricsProvider` trait.
2. **Data Generation Methods**: Methods for generating mock metrics, protocol data, and performance statistics.
3. **Concurrency Management**: Proper use of `Mutex` for thread-safe access to shared state.
4. **Error Handling**: Consistent error handling throughout the adapter.

### Key Features

- **Realistic Test Data**: Generates realistic system metrics including CPU, memory, disk, and network statistics.
- **Configurable Status**: Allows setting the connection status for testing different states.
- **Simulated Failures**: Supports a `should_fail` flag to test error handling.
- **Protocol Status**: Simulates connection status and protocol information.
- **Performance Metrics**: Provides mock performance data for the monitoring system.

### Implementation Highlights

#### MockAdapter Structure

```rust
pub struct MockAdapter {
    connection_status: Mutex<ConnectionStatus>,
    connection_health: Mutex<ConnectionHealth>,
    connection_history: Mutex<Vec<ConnectionEvent>>,
    error_log: Mutex<Vec<String>>,
    message_log: Mutex<Vec<String>>,
    performance_metrics: Mutex<PerformanceMetrics>,
    performance_options: Mutex<PerformanceOptions>,
    protocol_metrics: Mutex<HashMap<String, f64>>,
    should_fail: Mutex<bool>,
}
```

#### Mock Metrics Generation

The `generate_mock_metrics` method produces realistic system metrics:

- **CPU Metrics**: Usage percentages, core utilization, temperature, and load averages
- **Memory Metrics**: Total, used, available, and free memory, plus swap utilization
- **Network Metrics**: Interface statistics, packet counts, and bandwidth utilization
- **Disk Metrics**: Storage capacity, usage, and I/O operations

#### Thread Safety

All shared state is protected by `Mutex`, and locks are released as quickly as possible to prevent deadlocks:

```rust
async fn generate_mock_protocol_data(&self) -> ProtocolData {
    let connection_status = self.connection_status.lock().await;
    
    let connected = matches!(connection_status.clone(), ConnectionStatus::Connected);
    let status_string = connection_status.to_string();
    
    // Release the lock before other operations
    drop(connection_status);
    
    let metrics = self.protocol_metrics.lock().await.clone();
    
    ProtocolData {
        name: "MCP".to_string(),
        protocol_type: "TCP".to_string(),
        version: "1.0".to_string(),
        connected,
        last_connected: Some(Utc::now()),
        status: status_string,
        error: None,
        retry_count: 0,
        metrics,
    }
}
```

## Integration with Terminal UI

The mock adapter is integrated into the terminal UI through the `run` function:

```rust
pub async fn run(dashboard_service: Arc<dyn DashboardService>, demo_mode: bool) -> Result<(), Box<dyn Error>> {
    println!("Starting terminal dashboard in simplified mode...");
    
    if demo_mode {
        println!("Demo mode activated. Using mock adapter for dashboard metrics.");
        
        // Create and initialize mock adapter
        let mock_adapter = Arc::new(MockAdapter::new());
        
        // Display some basic information
        println!("Getting connection status...");
        match mock_adapter.get_connection_status().await {
            Ok(status) => println!("Connection status: {:?}", status),
            Err(err) => println!("Error getting connection status: {}", err),
        }
        
        println!("Getting dashboard data...");
        match mock_adapter.get_dashboard_data().await {
            Ok(data) => println!("Dashboard data retrieved with timestamp: {}", data.timestamp),
            Err(err) => println!("Error getting dashboard data: {}", err),
        }
        
        // Display performance metrics if available
        println!("Getting performance metrics...");
        match mock_adapter.get_performance_metrics().await {
            Ok(metrics) => println!("Performance metrics retrieved: CPU: {}%, Memory: {}MB", 
                             metrics.cpu_usage.unwrap_or(0.0),
                             metrics.memory_usage.unwrap_or(0.0)),
            Err(err) => println!("Error getting performance metrics: {}", err),
        }
        
        println!("\nPress Enter to exit...");
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer)?;
    } else {
        println!("Non-demo mode is currently not supported in simplified mode.");
        println!("Please restart with --demo flag or use the web UI instead.");
        std::thread::sleep(Duration::from_secs(5));
    }
    
    Ok(())
}
```

## Testing

The mock adapter has been successfully tested and verified to work with the terminal UI. It provides realistic data for all required metrics and properly handles error conditions.

## Future Enhancements

1. **Randomized Data**: Add options for generating randomized data that changes over time to better simulate a live system.
2. **Configurable Scenarios**: Support predefined scenarios (high load, error conditions, etc.) that can be selected for testing.
3. **Record/Replay**: Add the ability to record real monitoring data and replay it through the mock adapter.
4. **Integration with TUI**: Enhance the mock adapter to work with the TUI for more interactive testing.

## Conclusion

The mock adapter implementation provides a robust way to test and develop the terminal UI without requiring a connection to the actual monitoring system. It generates realistic data for all the required metrics and properly implements thread safety and error handling.

This implementation serves as a foundation for further development of the terminal UI and can be extended to support more advanced testing scenarios in the future. 