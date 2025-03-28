---
title: MCP-UI Integration Pattern
author: DataScienceBioLab
version: 1.0.0
date: 2024-08-30
status: implemented
---

# MCP-UI Integration Pattern

## Context

This pattern is used when integrating the Machine Context Protocol (MCP) with user interface components, particularly for displaying protocol metrics, connection status, and providing protocol debugging capabilities.

## Pattern Description

The MCP-UI Integration Pattern provides a standardized approach for UI components to consume MCP protocol data, visualize metrics, and monitor connection health. It uses an adapter-based approach with clearly defined interfaces to ensure loose coupling between the MCP implementation and UI components.

## Implementation

### Core Components

#### 1. McpMetricsProvider Interface

This interface defines the contract for obtaining protocol metrics and connection information:

```rust
#[async_trait]
pub trait McpMetricsProvider: Send + Sync + std::fmt::Debug {
    // Get current metrics snapshot
    async fn get_metrics(&self) -> Result<McpMetrics, String>;
    
    // Subscribe to metrics updates with specified interval
    fn subscribe(&self, interval_ms: u64) -> mpsc::Receiver<McpMetrics>;
    
    // Get connection status
    async fn connection_status(&self) -> ConnectionStatus;
    
    // Configure metrics collection
    async fn configure(&self, config: McpMetricsConfig) -> Result<(), String>;
    
    // Get protocol metrics as a HashMap
    fn get_protocol_metrics(&self) -> Result<HashMap<String, f64>, String>;
    
    // Get protocol status
    fn get_protocol_status(&self) -> Result<ProtocolStatus, String>;
    
    // Get connection health information
    fn connection_health(&self) -> Result<ConnectionHealth, String>;
    
    // Attempt to reconnect to the MCP service
    async fn reconnect(&self) -> Result<bool, String>;
    
    // Get connection history
    fn connection_history(&self) -> Result<Vec<ConnectionEvent>, String>;
}
```

#### 2. Data Models

The pattern defines clear data models for protocol information:

```rust
// Connection health status
#[derive(Debug, Clone)]
pub struct ConnectionHealth {
    pub status: ConnectionStatus,
    pub last_successful: Option<DateTime<Utc>>,
    pub failure_count: u32,
    pub latency_ms: Option<u64>,
    pub error_details: Option<String>,
}

// Connection event for history tracking
#[derive(Debug, Clone)]
pub struct ConnectionEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: ConnectionEventType,
    pub details: Option<String>,
}

// Connection event type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionEventType {
    Connected,
    Disconnected,
    Reconnecting,
    ReconnectSuccess,
    ReconnectFailure,
    Error,
}

// Connection status enum
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Connecting, 
    Degraded,
    Failed,
}
```

#### 3. Protocol Widget

The UI component that visualizes protocol data:

```rust
pub struct ProtocolWidget<'a> {
    // Protocol data to display
    protocol: &'a ProtocolData,
    // Widget title
    title: &'a str,
    // Active protocol tab index
    active_tab: usize,
    // Connection health data
    connection_health: Option<&'a ConnectionHealth>,
    // Connection history
    connection_history: Option<&'a Vec<ConnectionEvent>>,
    // Metrics history
    metrics_history: Option<&'a HashMap<String, Vec<(DateTime<Utc>, f64)>>>,
}
```

### Integration Flow

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   MCP Service   │────▶│ McpMetricsProvider│────▶│  ProtocolWidget │
└─────────────────┘     └─────────────────┘     └─────────────────┘
         │                        │                        │
         ▼                        ▼                        ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│Protocol Messages│     │Connection Health │     │    UI Rendering │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

1. The MCP Service collects protocol metrics and connection information.
2. The McpMetricsProvider implementation adapts this data for UI consumption.
3. The ProtocolWidget visualizes the data with tabbed interface showing:
   - Overview of protocol status
   - Detailed metrics
   - Connection health and history
   - Historical metrics charts

### Mock Implementation

For testing, a mock implementation is provided:

```rust
// Mock implementation for testing
pub struct MockMcpMetricsProvider {
    config: McpMetricsConfig,
    should_fail: bool,
    connection_health: ConnectionHealth,
    connection_history: Vec<ConnectionEvent>,
    last_reconnect: Option<DateTime<Utc>>,
}
```

## Benefits

1. **Loose Coupling**: UI components are not directly dependent on MCP implementation.
2. **Testability**: Mock providers enable UI testing without an actual MCP service.
3. **Consistent Interface**: Standardized interface for obtaining protocol data.
4. **Detailed Visualization**: Comprehensive protocol visualization with multiple views.
5. **Connection Management**: Built-in connection health monitoring and history tracking.

## Example Usage

```rust
// Create protocol widget with connection health and history
let protocol_widget = ProtocolWidget::new(&protocol_data, "Protocol")
    .with_connection_health(&connection_health)
    .with_connection_history(&connection_history)
    .with_metrics_history(&metrics_history);

// Render the widget
protocol_widget.render(f, area);
```

## Considerations

- **Performance**: Be mindful of data collection frequency and rendering performance.
- **History Storage**: Implement efficient storage and pruning for historical data.
- **Error Handling**: Ensure graceful handling of connection failures and errors.
- **Threading**: Use proper synchronization for cross-thread communication.

## Related Patterns

- Adapter Pattern: Used for adapting MCP data for UI consumption.
- Observer Pattern: Used for pushing updates from MCP to UI components.
- Repository Pattern: Used for storing and retrieving historical data.

## Status

This pattern is implemented and in use in the Terminal UI for visualizing MCP protocol data. 