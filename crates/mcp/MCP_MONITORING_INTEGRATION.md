# MCP-Monitoring Integration

## Overview

This document explains the integration between the MCP resilience framework's health monitoring and the global monitoring system. The integration follows an adapter pattern to establish bidirectional communication between these two systems.

## Core Components

### 1. HealthMonitoringBridge

The `HealthMonitoringBridge` is the primary integration component, serving as a mediator between the MCP resilience health monitor and the monitoring system.

```rust
/// Bridge that forwards MCP resilience health data to the global monitoring system
pub struct HealthMonitoringBridge {
    /// Reference to the MCP resilience health monitor
    resilience_monitor: Arc<HealthMonitor>,
    
    /// Reference to the monitoring system's metrics collector
    metrics_collector: Arc<MetricsCollector>,
    
    /// Reference to the monitoring system's alert manager
    alert_manager: Arc<AlertManager>,
    
    /// Configuration for the bridge
    config: HealthMonitoringBridgeConfig,
    
    // ... other fields ...
}
```

The bridge can be configured using the `HealthMonitoringBridgeConfig`:

```rust
pub struct HealthMonitoringBridgeConfig {
    /// How often to forward health data (in seconds)
    pub forward_interval: u64,
    
    /// Whether to forward all components or only unhealthy ones
    pub forward_all_components: bool,
    
    /// Whether to enable bidirectional integration
    pub bidirectional: bool,
}
```

### 2. ResilienceHealthCheckAdapter

The `ResilienceHealthCheckAdapter` adapts resilience health checks to the monitoring system's health check interface, making them compatible with the global monitoring dashboard.

```rust
/// Adapter for integrating resilience health checks with the monitoring system
pub struct ResilienceHealthCheckAdapter<T> where T: HealthCheck {
    /// The inner resilience health check
    inner: T,
    /// Whether to forward metrics to the monitoring system
    forward_metrics: bool,
}
```

### 3. AlertToRecoveryAdapter

The `AlertToRecoveryAdapter` converts monitoring alerts to resilience recovery actions, enabling alerts from the monitoring system to trigger recovery mechanisms.

```rust
/// Adapter for converting monitoring alerts to resilience recovery actions
pub struct AlertToRecoveryAdapter {
    /// Reference to the recovery strategy
    recovery_strategy: Arc<Mutex<RecoveryStrategy>>,
    /// Whether to log recovery actions
    log_recovery: bool,
}
```

## Integration Setup

### Basic Setup

```rust
// Create components
let resilience_monitor = Arc::new(HealthMonitor::default());
let recovery_strategy = Arc::new(Mutex::new(RecoveryStrategy::new()));
let metrics_collector = Arc::new(MetricsCollector::new());
let alert_manager = Arc::new(AlertManager::new());

// Create bridge configuration
let config = HealthMonitoringBridgeConfig {
    forward_interval: 10,
    forward_all_components: true,
    bidirectional: true,
};

// Create and start the bridge
let bridge = HealthMonitoringBridge::new(
    resilience_monitor.clone(),
    metrics_collector.clone(),
    alert_manager.clone(),
    config,
).with_recovery_strategy(recovery_strategy.clone());

bridge.start().await?;
```

### Advanced Setup with Alert Handler

```rust
// Create alert adapter
let alert_adapter = AlertToRecoveryAdapter::new(recovery_strategy.clone());

// Register the alert handler
alert_manager.register_handler("resilience_recovery", Box::new(alert_adapter)).await?;
```

## Integration Capabilities

### 1. Forward Health Data to Monitoring

The bridge periodically forwards health data from the resilience health monitor to the monitoring system:

- Forwards component health status
- Forwards metrics from health checks
- Generates appropriate alerts for unhealthy components
- Can be configured to forward all components or only unhealthy ones

### 2. Trigger Recovery Actions from Alerts

When the monitoring system generates alerts, they can trigger recovery actions:

- Converts alert severity to failure severity
- Creates failure information from alerts
- Triggers appropriate recovery strategies
- Logs recovery actions and results

### 3. Resilience Health Checks in Monitoring Dashboard

Resilience health checks can be displayed in the monitoring dashboard:

- Adapts resilience health checks to monitoring system format
- Shows component health status
- Displays metrics from health checks
- Allows central visualization of all system components

## Detailed Usage Example

A full example is available in `examples/monitoring_integration.rs`. Here's how to run it:

```bash
cargo run --example monitoring_integration
```

The example demonstrates:

1. Setting up resilience and monitoring components
2. Creating a health monitoring bridge
3. Registering health checks
4. Simulating health status changes
5. Triggering alerts from the monitoring system
6. Handling bidirectional integration

## Guidelines for Custom Components

### Custom Health Checks

When creating custom health checks that integrate with both systems:

```rust
#[derive(Debug)]
struct MyComponentHealthCheck {
    config: HealthCheckConfig,
    // ... component-specific fields ...
}

#[async_trait]
impl HealthCheck for MyComponentHealthCheck {
    fn id(&self) -> &str {
        "my_component"
    }
    
    async fn check(&self) -> HealthCheckResult {
        // Perform actual health check
        // ...
        
        // Return result with metrics
        let mut result = HealthCheckResult::new(
            "my_component".to_string(),
            HealthStatus::Healthy,
            "Component is healthy".to_string(),
        );
        
        // Add relevant metrics
        result = result.with_metric("response_time_ms", 150.0);
        result = result.with_metric("error_rate", 0.05);
        
        result
    }
    
    // ... other required methods ...
}
```

### Custom Recovery Strategies

To create custom recovery strategies that work with the alert adapter:

```rust
#[derive(Debug)]
struct MyRecoveryStrategy {
    // ... strategy-specific fields ...
}

impl MyRecoveryStrategy {
    async fn handle_failure<F>(&mut self, failure_info: FailureInfo, recovery_action: F) -> Result<()>
    where
        F: FnOnce() -> Result<()>,
    {
        // Implement recovery logic
        // ...
        
        // Call the provided recovery action
        recovery_action()
    }
}
```

## Testing

The integration components include comprehensive test coverage:

1. **Unit Tests**: For individual components
2. **Integration Tests**: For interaction between components
3. **Mocks**: For isolating components during testing

Example of testing a custom health check adapter:

```rust
#[tokio::test]
async fn test_custom_health_check() {
    let health_check = MyComponentHealthCheck::new();
    let adapter = ResilienceHealthCheckAdapter::new(health_check);
    
    // Test the adapter
    let result = adapter.check().await.unwrap();
    assert!(result.is_healthy);
    
    // ... additional assertions ...
}
```

## Performance Considerations

When using the integration, consider these performance implications:

1. **Forwarding Interval**: Set an appropriate interval based on your needs
2. **Forward All Components**: Consider forwarding only unhealthy components in high-load systems
3. **Alert Handling**: Be mindful of potential cascading alerts in complex systems

## Security Considerations

When integrating with sensitive systems:

1. **Authentication**: Ensure proper authentication between systems
2. **Authorization**: Restrict recovery actions to authorized handlers
3. **Data Isolation**: Maintain proper isolation between components
4. **Audit Logging**: Log all recovery actions for audit purposes

## Future Enhancements

Planned enhancements for the integration:

1. **Enhanced Metrics Aggregation**: Better aggregation of metrics from multiple sources
2. **Advanced Recovery Policies**: More sophisticated recovery policies based on alert patterns
3. **Dashboard Integration**: Deeper integration with visualization dashboards
4. **Historical Analysis**: Tracking historical health data for trend analysis

---

*Created by DataScienceBioLab* | *Version 1.0.0* | *Last Updated: July 18, 2024* 