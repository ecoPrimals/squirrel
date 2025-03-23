# Dashboard API Documentation

## Overview

The Dashboard API provides programmatic access to the monitoring system's dashboard functionality. It allows you to create, configure, and manage dashboards, as well as interact with the data visualization components.

## Dashboard Manager

The `DashboardManager` is the primary interface for working with dashboards. It provides methods for creating and managing dashboard instances, handling component subscriptions, and managing the WebSocket server.

### Initialization

```rust
use monitoring::dashboard::{DashboardManager, DashboardConfig};

// Create with default configuration
let dashboard = DashboardManager::new();

// Create with custom configuration
let config = DashboardConfig::default()
    .with_port(9000)
    .with_host("0.0.0.0")
    .with_max_connections(200)
    .with_history_size(100)
    .with_compression_threshold(8192);

let dashboard = DashboardManager::new_with_config(config);

// Start the dashboard server
dashboard.start().await?;
```

### Methods

#### Server Management

```rust
// Start the WebSocket server
async fn start(&self) -> Result<(), DashboardError>;

// Stop the WebSocket server
async fn stop(&self) -> Result<(), DashboardError>;

// Check if the server is running
fn is_running(&self) -> bool;

// Get server statistics
fn get_stats(&self) -> DashboardStats;
```

#### Component Management

```rust
// Register a new component
fn register_component(&self, component: Component) -> Result<(), DashboardError>;

// Update a component's data
fn update_component(&self, component_id: &str, data: ComponentData) -> Result<(), DashboardError>;

// Remove a component
fn remove_component(&self, component_id: &str) -> Result<(), DashboardError>;

// Get a list of all registered components
fn get_components(&self) -> Vec<Component>;

// Get a specific component by ID
fn get_component(&self, component_id: &str) -> Option<Component>;
```

#### Client Management

```rust
// Get the number of connected clients
fn get_client_count(&self) -> usize;

// Get a list of connected clients
fn get_clients(&self) -> Vec<ClientInfo>;

// Disconnect a specific client
fn disconnect_client(&self, client_id: &str) -> Result<(), DashboardError>;

// Send a message to a specific client
fn send_to_client(&self, client_id: &str, message: Message) -> Result<(), DashboardError>;

// Broadcast a message to all clients
fn broadcast(&self, message: Message) -> Result<(), DashboardError>;
```

#### Subscription Management

```rust
// Get subscriptions for a component
fn get_subscriptions(&self, component_id: &str) -> Vec<String>; // Returns client IDs

// Get all component subscriptions for a client
fn get_client_subscriptions(&self, client_id: &str) -> Vec<String>; // Returns component IDs
```

### Events

The `DashboardManager` emits events that you can subscribe to:

```rust
use monitoring::dashboard::{DashboardManager, DashboardEvent};

let dashboard = DashboardManager::new();

// Subscribe to events
let mut event_receiver = dashboard.subscribe_to_events();

// Handle events in a separate task
tokio::spawn(async move {
    while let Some(event) = event_receiver.recv().await {
        match event {
            DashboardEvent::ClientConnected(client_info) => {
                println!("Client connected: {}", client_info.client_id);
            },
            DashboardEvent::ClientDisconnected(client_id) => {
                println!("Client disconnected: {}", client_id);
            },
            DashboardEvent::Subscription(client_id, component_id) => {
                println!("Client {} subscribed to {}", client_id, component_id);
            },
            DashboardEvent::Unsubscription(client_id, component_id) => {
                println!("Client {} unsubscribed from {}", client_id, component_id);
            },
            DashboardEvent::MessageSent(client_id, message_type) => {
                println!("Message of type {} sent to {}", message_type, client_id);
            },
            DashboardEvent::Error(error) => {
                eprintln!("Dashboard error: {}", error);
            },
        }
    }
});
```

## Components

Components are the building blocks of the dashboard. Each component represents a specific visualization or data element.

### Component Structure

```rust
pub struct Component {
    pub id: String,
    pub name: String,
    pub component_type: ComponentType,
    pub config: ComponentConfig,
    pub data: ComponentData,
}
```

### Component Types

The following component types are supported:

```rust
pub enum ComponentType {
    Chart,
    Gauge,
    Table,
    StatCard,
    Timeline,
    Heatmap,
    NetworkGraph,
    LogViewer,
    Alert,
    Custom(String),
}
```

### Component Configuration

Each component has a configuration that defines its behavior and appearance:

```rust
pub struct ComponentConfig {
    pub title: String,
    pub description: Option<String>,
    pub refresh_interval: Duration,
    pub show_timestamp: bool,
    pub data_retention: DataRetention,
    pub visualization: VisualizationConfig,
    pub thresholds: Option<Vec<Threshold>>,
}
```

### Component Data

Component data represents the actual data displayed by the component:

```rust
pub struct ComponentData {
    pub timestamp: u64,
    pub values: serde_json::Value, // Flexible JSON structure for different data types
    pub metadata: Option<serde_json::Value>,
}
```

### Example: Creating and Updating a Component

```rust
use monitoring::dashboard::{
    Component, ComponentType, ComponentConfig, ComponentData, 
    VisualizationConfig, DataRetention, DashboardManager
};
use std::time::Duration;
use serde_json::json;

// Create a new component
let config = ComponentConfig {
    title: "CPU Usage".to_string(),
    description: Some("System CPU usage percentage".to_string()),
    refresh_interval: Duration::from_secs(5),
    show_timestamp: true,
    data_retention: DataRetention::LastNPoints(100),
    visualization: VisualizationConfig::LineChart {
        y_axis_label: "Usage %".to_string(),
        x_axis_label: "Time".to_string(),
        show_grid: true,
        line_color: "#1E88E5".to_string(),
    },
    thresholds: Some(vec![
        Threshold {
            value: 80.0,
            color: "#FF9800".to_string(),
            label: "Warning".to_string(),
        },
        Threshold {
            value: 90.0,
            color: "#F44336".to_string(),
            label: "Critical".to_string(),
        },
    ]),
};

let cpu_component = Component {
    id: "system_cpu".to_string(),
    name: "CPU Usage".to_string(),
    component_type: ComponentType::Chart,
    config,
    data: ComponentData {
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
        values: json!({
            "current": 35.2,
            "history": [
                { "time": 1650123450000, "value": 32.5 },
                { "time": 1650123455000, "value": 33.8 },
                { "time": 1650123460000, "value": 35.2 }
            ]
        }),
        metadata: None,
    },
};

// Register the component with the dashboard
let dashboard = DashboardManager::new();
dashboard.register_component(cpu_component)?;

// Update the component data later
let updated_data = ComponentData {
    timestamp: std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64,
    values: json!({
        "current": 42.7,
        "history": [
            { "time": 1650123465000, "value": 37.1 },
            { "time": 1650123470000, "value": 40.3 },
            { "time": 1650123475000, "value": 42.7 }
        ]
    }),
    metadata: None,
};

dashboard.update_component("system_cpu", updated_data)?;
```

## Dashboard WebSocket Server

The `DashboardServer` manages the WebSocket connections and handles client communication.

### Configuration

```rust
pub struct WebSocketConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub ping_interval: Duration,
    pub connection_timeout: Duration,
    pub max_message_size: usize,
    pub compression_threshold: usize,
    pub enable_compression: bool,
}
```

### Initialization

```rust
use monitoring::dashboard::{DashboardServer, WebSocketConfig};
use std::time::Duration;

let config = WebSocketConfig {
    host: "127.0.0.1".to_string(),
    port: 8765,
    max_connections: 100,
    ping_interval: Duration::from_secs(30),
    connection_timeout: Duration::from_secs(60),
    max_message_size: 1024 * 1024, // 1MB
    compression_threshold: 8192,    // 8KB
    enable_compression: true,
};

let server = DashboardServer::new(config);
server.start().await?;
```

### Methods

```rust
// Start the WebSocket server
async fn start(&self) -> Result<(), WebSocketError>;

// Stop the WebSocket server
async fn stop(&self) -> Result<(), WebSocketError>;

// Send a message to a specific client
async fn send_message(&self, client_id: &str, message: Message) -> Result<(), WebSocketError>;

// Broadcast a message to all clients
async fn broadcast(&self, message: Message) -> Result<(), WebSocketError>;

// Broadcast a message to clients subscribed to a specific component
async fn broadcast_to_subscribers(&self, component_id: &str, message: Message) 
    -> Result<(), WebSocketError>;

// Get server statistics
fn get_stats(&self) -> WebSocketStats;
```

## Message Types

The dashboard uses a set of predefined message types for communication:

### Client to Server Messages

```rust
// Subscribe to a component
{
    "type": "subscribe",
    "componentId": "system_cpu"
}

// Unsubscribe from a component
{
    "type": "unsubscribe",
    "componentId": "system_cpu"
}

// Request a batch update for multiple components
{
    "type": "request_batch",
    "components": ["system_cpu", "system_memory", "network_traffic"],
    "includeHistory": true,
    "historyPoints": 10
}

// Ping message (keepalive)
{
    "type": "ping",
    "timestamp": 1650123456789
}
```

### Server to Client Messages

```rust
// Component update
{
    "type": "update",
    "timestamp": 1650123456789,
    "componentId": "system_cpu",
    "data": {
        "current": 45.2,
        "history": [
            { "time": 1650123450000, "value": 42.5 },
            { "time": 1650123455000, "value": 43.8 },
            { "time": 1650123460000, "value": 45.2 }
        ]
    }
}

// Batch update
{
    "type": "batch",
    "timestamp": 1650123456789,
    "updates": [
        {
            "componentId": "system_cpu",
            "data": { "current": 45.2 }
        },
        {
            "componentId": "system_memory",
            "data": { "used": 8192, "total": 16384 }
        }
    ]
}

// Compressed update
{
    "type": "compressed",
    "timestamp": 1650123456789,
    "compressed": true,
    "compression": "gzip",
    "encoding": "base64",
    "compressed_data": "H4sIAAAAAAAA/6tWSs/PT89J9cjPTM/MKwYA6F..."
}

// Error message
{
    "type": "error",
    "code": "invalid_component",
    "message": "Component 'unknown_component' does not exist"
}

// Pong response
{
    "type": "pong",
    "timestamp": 1650123456789,
    "server_time": 1650123456790
}
```

## Dashboard Adapter

The `DashboardManagerAdapter` provides a simpler interface for integrating the dashboard functionality into an application:

```rust
use monitoring::dashboard::DashboardManagerAdapter;

// Create the adapter
let dashboard = DashboardManagerAdapter::new();

// Initialize and start the server
dashboard.initialize()?;
dashboard.start().await?;

// Update a component with new data
dashboard.update("system_cpu", serde_json::json!({
    "usage": 45.2,
    "temperature": 72.1
}))?;

// Shutdown the server when done
dashboard.shutdown().await?;
```

## Error Handling

The dashboard module provides a comprehensive error handling system:

```rust
pub enum DashboardError {
    // Server errors
    ServerAlreadyRunning,
    ServerStartFailed(String),
    ServerNotRunning,
    
    // Component errors
    ComponentAlreadyExists(String),
    ComponentNotFound(String),
    InvalidComponentData(String),
    
    // Client errors
    ClientNotFound(String),
    TooManyConnections,
    ConnectionFailed(String),
    MessageSendFailed(String),
    
    // Subscription errors
    SubscriptionLimitExceeded,
    
    // Other errors
    SerializationError(String),
    CompressionError(String),
    InternalError(String),
}
```

## Performance Considerations

To optimize dashboard performance, consider the following:

1. **Component Refresh Intervals**: Set appropriate refresh intervals for components based on their importance and how frequently the data changes.

2. **Data Retention**: Configure the `DataRetention` policy to limit the amount of historical data stored for each component.

3. **Message Compression**: Enable compression for large messages to reduce bandwidth usage. Adjust the `compression_threshold` to control when compression is applied.

4. **Batch Updates**: Use batch updates to send multiple component updates in a single message, especially for related components that are likely to be updated together.

5. **Client Subscriptions**: Encourage clients to subscribe only to the components they need to minimize unnecessary message processing and bandwidth usage.

## Advanced Usage

### Custom Component Types

You can create custom component types for specialized visualizations:

```rust
use monitoring::dashboard::{
    Component, ComponentType, ComponentConfig, ComponentData, DashboardManager
};
use serde_json::json;

// Create a custom component
let custom_component = Component {
    id: "custom_visualization".to_string(),
    name: "Custom Visualization".to_string(),
    component_type: ComponentType::Custom("3d_surface_plot".to_string()),
    config: ComponentConfig::default()
        .with_title("3D Surface Plot")
        .with_refresh_interval(Duration::from_secs(10)),
    data: ComponentData {
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
        values: json!({
            "x_values": [1, 2, 3, 4, 5],
            "y_values": [1, 2, 3, 4, 5],
            "z_values": [
                [10, 20, 30, 40, 50],
                [15, 25, 35, 45, 55],
                [20, 30, 40, 50, 60],
                [25, 35, 45, 55, 65],
                [30, 40, 50, 60, 70]
            ]
        }),
        metadata: Some(json!({
            "colormap": "viridis",
            "wireframe": true,
            "rotation": {
                "x": 30,
                "y": 45,
                "z": 0
            }
        })),
    },
};

let dashboard = DashboardManager::new();
dashboard.register_component(custom_component)?;
```

### Integration with Metrics System

The dashboard can be integrated with the metrics system for automatic updates:

```rust
use monitoring::{
    metrics::{MetricsCollector, Metric},
    dashboard::DashboardManagerAdapter
};

// Create the dashboard adapter
let dashboard = DashboardManagerAdapter::new();
dashboard.initialize()?;
dashboard.start().await?;

// Create a metrics collector
let metrics = MetricsCollector::new();

// Register a callback to update the dashboard when metrics are collected
metrics.on_collect(move |metric_batch| {
    for (name, value) in metric_batch {
        match name.as_str() {
            "system.cpu.usage" => {
                dashboard.update("system_cpu", serde_json::json!({
                    "usage": value.as_f64().unwrap_or(0.0)
                })).unwrap_or_else(|e| eprintln!("Failed to update dashboard: {}", e));
            },
            "system.memory.used" => {
                dashboard.update("system_memory", serde_json::json!({
                    "used": value.as_i64().unwrap_or(0)
                })).unwrap_or_else(|e| eprintln!("Failed to update dashboard: {}", e));
            },
            // Handle other metrics...
            _ => {}
        }
    }
});

// Start the metrics collector
metrics.start();
```

## Security

### TLS Configuration

To enable secure WebSocket connections (WSS), configure TLS:

```rust
use monitoring::dashboard::{DashboardManager, DashboardConfig, TlsConfig};

let tls_config = TlsConfig {
    cert_path: "/path/to/cert.pem".to_string(),
    key_path: "/path/to/key.pem".to_string(),
};

let config = DashboardConfig::default()
    .with_tls(Some(tls_config));

let dashboard = DashboardManager::new_with_config(config);
dashboard.start().await?;
```

### Authentication

To enable WebSocket authentication:

```rust
use monitoring::dashboard::{
    DashboardManager, DashboardConfig, AuthConfig, AuthType
};

let auth_config = AuthConfig {
    auth_type: AuthType::Bearer,
    token_validator: Box::new(|token| {
        // Validate the token (e.g., check against a database or JWT validation)
        token == "valid-token"
    }),
};

let config = DashboardConfig::default()
    .with_auth(Some(auth_config));

let dashboard = DashboardManager::new_with_config(config);
dashboard.start().await?;
```

## Testing

The dashboard module includes comprehensive testing tools:

```rust
use monitoring::dashboard::testing::{
    DashboardTester, TestClient, TestConfig
};

#[tokio::test]
async fn test_dashboard() {
    // Create a test configuration
    let config = TestConfig {
        num_clients: 10,
        test_duration: Duration::from_secs(30),
        components_per_client: 5,
        update_interval: Duration::from_millis(500),
    };
    
    // Create a dashboard tester
    let tester = DashboardTester::new(config);
    
    // Run the test
    let results = tester.run().await;
    
    // Verify the results
    assert!(results.success);
    assert!(results.avg_message_latency < Duration::from_millis(100));
    assert!(results.error_count == 0);
    assert!(results.total_messages > 0);
}
```

## Further Resources

- [WebSocket Protocol Documentation](./websocket_protocol.md)
- [Component Registry](./component_registry.md)
- [Dashboard Frontend Integration](./dashboard_frontend.md) 