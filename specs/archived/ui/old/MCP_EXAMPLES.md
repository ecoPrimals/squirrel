# MCP Integration Examples

## Overview

This document provides detailed guidance on using the example programs that showcase the MCP (Machine Context Protocol) integration with the UI components. These examples demonstrate how to use the `ConnectionHealth` monitoring features and other capabilities of the MCP integration.

**Version**: 1.0.0  
**Date**: 2025-03-28  
**Status**: Complete  

## Available Examples

The following example programs are available in the `crates/ui-terminal/examples` directory:

1. **mcp_monitor**: A command-line tool that displays real-time MCP protocol metrics and connection health statistics.
2. **custom_dashboard**: A full dashboard application that incorporates MCP metrics into a user-friendly interface.

## Running the Examples

### Prerequisites

Before running the examples, ensure you have:

1. Rust toolchain installed (version 1.70.0 or later)
2. Clone of the Squirrel UI repository
3. Built the project with `cargo build`

### MCP Monitor Example

The MCP Monitor example demonstrates the real-time monitoring of MCP protocol metrics and connection health.

#### Running the Example

```bash
cargo run --example mcp_monitor -- [OPTIONS]
```

#### Available Options

- `--mcp-server <ADDRESS>`: Specify the MCP server address (default: "127.0.0.1:8778")
- `--mcp-interval <MS>`: Set the update interval in milliseconds (default: 1000)
- `--simulate-issues`: Enable simulation of connection issues for testing
- `--verbose`: Enable verbose output with additional metrics

#### Example Output

```
Connection Health:
  - Latency: 15.20 ms
  - Stability: 100.0%
  - Signal Strength: 98.5%
  - Packet Loss: 0.1%
  - Last Checked: 14:32:45.123

Protocol Metrics:
  - Total Requests: 152
  - Total Responses: 152
  - Request Rate: 10.5/s
  - Response Rate: 10.5/s
```

During connection issues (when simulated):

```
Failed to get connection health: Connection error: Simulated failure
Attempting reconnection...
```

#### Testing Connection Health Features

1. **Normal Operation**: By default, the example shows stable connection metrics.
2. **Simulated Issues**: Use the `--simulate-issues` flag to create periodic connection failures:
   ```bash
   cargo run --example mcp_monitor -- --simulate-issues
   ```
3. **Reconnection**: When connection issues occur (simulated or real), the example will attempt to reconnect.
4. **Performance Monitoring**: With `--verbose`, you can see cache hit rates and request timing.

### Custom Dashboard Example

The Custom Dashboard example demonstrates the integration of MCP metrics with a full dashboard UI.

#### Running the Example

```bash
cargo run --example custom_dashboard -- --mcp [OPTIONS]
```

#### Available Options

- `--mcp`: Enable MCP integration
- `--mcp-server <ADDRESS>`: Specify the MCP server address (default: "127.0.0.1:8778")
- `--mcp-interval <MS>`: Set the update interval in milliseconds (default: 1000)
- `--simulate-issues`: Enable simulation of connection issues
- `--theme <THEME>`: Select a UI theme (default: "dark")

#### User Interface

The dashboard UI includes:

1. **Protocol Tab**: Shows MCP connection status and metrics
2. **Connection Health Panel**: Displays:
   - Latency gauge
   - Stability percentage
   - Signal strength indicator
   - Packet loss statistics
3. **Reconnect Button**: For manually triggering reconnection
4. **Alert Panel**: Shows notifications about connection issues

#### Keyboard Shortcuts

- `Tab`: Switch between tabs
- `r`: Trigger reconnection (when on Protocol tab)
- `q`: Quit the application

## ConnectionHealth Features

Both examples showcase the `ConnectionHealth` structure, which provides the following metrics:

### Latency

Measures the round-trip time for messages in milliseconds. Lower values indicate better performance.

```rust
// Access latency
let latency = connection_health.latency_ms;
println!("Latency: {:.2} ms", latency);
```

### Stability

Indicates the overall connection reliability as a percentage (0-100%). Values closer to 100% represent more stable connections.

```rust
// Access stability
let stability = connection_health.stability;
println!("Stability: {:.1}%", stability);
```

### Signal Strength

Represents the connection quality as a percentage (0-100%). Higher values indicate stronger connections.

```rust
// Access signal strength
let signal = connection_health.signal_strength;
println!("Signal Strength: {:.1}%", signal);
```

### Packet Loss

Shows the percentage of lost packets (0-100%). Lower values indicate better connection quality.

```rust
// Access packet loss
let packet_loss = connection_health.packet_loss;
println!("Packet Loss: {:.1}%", packet_loss);
```

### Last Checked Timestamp

Provides the time when the metrics were last updated.

```rust
// Access last checked timestamp
let timestamp = connection_health.last_checked;
println!("Last Updated: {}", timestamp.format("%H:%M:%S%.3f"));
```

## Implementing Your Own MCP-Enabled Application

To implement MCP integration in your own application, follow these steps:

### 1. Initialize the MCP Metrics Provider

```rust
// Create configuration
let mcp_config = McpMetricsConfig {
    update_interval_ms: 1000,
    server_address: "127.0.0.1:8778".to_string(),
    ..Default::default()
};

// Create provider
let provider = Arc::new(RealMcpMetricsProvider::with_config(mcp_config));
```

### 2. Create Background Task for Updates

```rust
let provider_clone = provider.clone();
tokio::spawn(async move {
    let mut interval = time::interval(Duration::from_millis(1000));
    
    loop {
        interval.tick().await;
        
        // Get connection health
        if let Ok(health) = provider_clone.get_connection_health().await {
            // Use health metrics
            println!("Latency: {:.2} ms", health.latency_ms);
        }
        
        // Get other metrics as needed
        if let Ok(metrics) = provider_clone.get_metrics().await {
            // Use metrics
        }
    }
});
```

### 3. Handle Reconnection

```rust
// Attempt reconnection when needed
match provider.reconnect().await {
    Ok(true) => println!("Successfully reconnected"),
    Ok(false) => println!("Reconnection failed but no error returned"),
    Err(e) => println!("Reconnection error: {}", e),
}
```

### 4. Display Connection Health

```rust
// Get and display connection health
match provider.get_connection_health().await {
    Ok(health) => {
        // Display health metrics
        println!("Connection Health:");
        println!("  - Latency: {:.2} ms", health.latency_ms);
        println!("  - Stability: {:.1}%", health.stability);
        println!("  - Signal Strength: {:.1}%", health.signal_strength);
        println!("  - Packet Loss: {:.1}%", health.packet_loss);
    },
    Err(e) => {
        println!("Failed to get connection health: {}", e);
    }
}
```

## Testing with MockAdapter

For testing without a real MCP server, you can use the `MockAdapter`:

```rust
// Create a mock adapter
let mock = MockAdapter::new();

// Set up simulation patterns
let mock_clone = mock.clone();
tokio::spawn(async move {
    let mut interval = time::interval(Duration::from_millis(5000));
    let mut connected = true;
    
    loop {
        interval.tick().await;
        connected = !connected;
        
        if connected {
            mock_clone.set_connection_status(ConnectionStatus::Connected).await;
        } else {
            mock_clone.set_connection_status(ConnectionStatus::Disconnected).await;
        }
    }
});

// Use mock adapter like a real provider
if let Ok(health) = mock.get_connection_health().await {
    println!("Mock Connection Health: Latency = {:.2} ms", health.latency_ms);
}
```

## Conclusion

The example programs provide practical demonstrations of the MCP integration capabilities, particularly focusing on the `ConnectionHealth` monitoring features. By examining these examples, you can understand how to implement similar functionality in your own applications and leverage the comprehensive connection monitoring capabilities provided by the MCP integration.

For more detailed information on the MCP integration implementation, refer to the `MCP_IMPLEMENTATION_SUMMARY.md` document.

---

Last Updated: March 28, 2025 