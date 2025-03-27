---
description: DEFINE patterns and standards for UI integration with other crates
---

# UI Integration Patterns

## Context

- When integrating UI components with other crates
- When collecting metrics from system services
- When designing communication protocols between UI and services
- When implementing consistent data flows

## Integration Patterns

### 1. Adapter Pattern for Metrics Collection

The primary pattern for integrating UI dashboards with other crates is the **Adapter Pattern**.

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Source Crate  │────▶│     Adapter     │────▶│  UI Dashboard   │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

#### Implementation Requirements:

- Adapters should convert source-specific data models to UI-compatible models
- Adapters should handle errors from source crates gracefully
- Adapters should provide fallback mechanisms for temporarily unavailable sources
- Metrics collection should be non-blocking and asynchronous

#### Example:

```rust
// Adapter for collecting MCP protocol metrics
pub struct McpMetricsAdapter {
    // MCP client or connection
    client: Option<mcp::Client>,
    // Cached metrics for fallback
    cached_metrics: HashMap<String, MetricValue>,
    // Last successful update
    last_update: DateTime<Utc>,
}

impl McpMetricsAdapter {
    // Collect metrics from MCP crate
    pub async fn collect_metrics(&mut self) -> Result<MetricsSnapshot, AdapterError> {
        // Try to get metrics from MCP client
        if let Some(client) = &self.client {
            match client.get_metrics().await {
                Ok(mcp_metrics) => {
                    let metrics = self.convert_metrics(mcp_metrics);
                    self.cached_metrics = metrics.clone();
                    self.last_update = Utc::now();
                    Ok(metrics)
                },
                Err(e) => {
                    // Fall back to cached metrics if available
                    if !self.cached_metrics.is_empty() {
                        log::warn!("Using cached MCP metrics due to error: {}", e);
                        Ok(self.cached_metrics.clone())
                    } else {
                        Err(AdapterError::SourceError(e.to_string()))
                    }
                }
            }
        } else {
            // No client available
            Err(AdapterError::NotConnected)
        }
    }
}
```

### 2. Event-Based Integration

For real-time updates, use an event-based approach with channels:

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Source Crate  │────▶│ Message Channel │────▶│  UI Components  │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

#### Implementation Requirements:

- Use `tokio::sync::mpsc` or similar for message passing
- Implement a unified event type for UI updates
- Use bounded channels with appropriate capacity
- Handle channel closure and errors gracefully

#### Example:

```rust
// UI update event type
pub enum UiEvent {
    MetricsUpdate(MetricsSnapshot),
    AlertTriggered(Alert),
    ConnectionStatusChanged(ConnectionStatus),
    ConfigurationChanged(Configuration),
}

// Producer (in source crate)
let (tx, _) = mpsc::channel(100);
// Send metrics updates
tx.send(UiEvent::MetricsUpdate(metrics)).await?;

// Consumer (in UI crate)
while let Some(event) = rx.recv().await {
    match event {
        UiEvent::MetricsUpdate(metrics) => update_dashboard(metrics),
        UiEvent::AlertTriggered(alert) => display_alert(alert),
        // Handle other events
        _ => {}
    }
}
```

### 3. Service Interface Pattern

Define clear service interfaces for core functionality:

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Source Crate  │────▶│ Service Trait   │────▶│  UI Components  │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

#### Implementation Requirements:

- Define traits for each service interface
- Use async functions for non-blocking operations
- Provide clear error types and handling
- Use Arc<dyn Trait> for shared service instances

#### Example:

```rust
#[async_trait]
pub trait MetricsService: Send + Sync {
    // Get current metrics
    async fn get_metrics(&self) -> Result<MetricsSnapshot, ServiceError>;
    
    // Subscribe to metrics updates
    async fn subscribe(&self) -> mpsc::Receiver<MetricsSnapshot>;
    
    // Configure metrics collection
    async fn configure(&self, config: MetricsConfig) -> Result<(), ServiceError>;
}

// Implementation for MCP metrics
pub struct McpMetricsService {
    client: Arc<mcp::Client>,
    // Other fields
}

#[async_trait]
impl MetricsService for McpMetricsService {
    async fn get_metrics(&self) -> Result<MetricsSnapshot, ServiceError> {
        // Implementation
    }
    
    async fn subscribe(&self) -> mpsc::Receiver<MetricsSnapshot> {
        // Implementation
    }
    
    async fn configure(&self, config: MetricsConfig) -> Result<(), ServiceError> {
        // Implementation
    }
}
```

## Integration Standards for Specific Crates

### MCP Crate Integration

The MCP crate should expose a metrics collection API that provides:

1. **Protocol Metrics**:
   - Message counts: requests, responses
   - Transaction statistics: rates, success rates
   - Error counts and types: connection errors, protocol errors
   - Latency measurements: request-response times

2. **API Requirements**:
   - Asynchronous metrics collection API
   - Clear error handling
   - Optional real-time metrics subscription
   - Thread-safe implementation

#### Example API:

```rust
// MCP metrics collection API
impl McpClient {
    // Get current metrics snapshot
    pub async fn get_metrics(&self) -> Result<McpMetrics, McpError> {
        // Implementation
    }
    
    // Subscribe to metrics updates
    pub fn subscribe_to_metrics(&self) -> mpsc::Receiver<McpMetrics> {
        // Implementation
    }
}

// MCP metrics data structure
pub struct McpMetrics {
    // Message statistics
    pub requests: u64,
    pub responses: u64,
    
    // Transaction statistics
    pub transactions: u64,
    pub success_rate: f64,
    
    // Error statistics
    pub connection_errors: u64,
    pub protocol_errors: u64,
    
    // Latency statistics
    pub average_latency_ms: f64,
    pub latency_histogram: Vec<f64>,
    
    // Timestamp
    pub timestamp: DateTime<Utc>,
}
```

### Dashboard-Core Integration

The Dashboard-Core crate should provide:

1. **Data Model**:
   - Clear data structures for metrics, alerts, and system information
   - Serialization/deserialization support
   - Conversion methods for external metrics formats

2. **Service Interface**:
   - Dashboard service for central data management
   - Update mechanisms for real-time data
   - History tracking for metrics

#### Integration Points:

1. **MetricsSnapshot** - Core data structure for metrics data
2. **DashboardService** - Service interface for dashboard operations
3. **DashboardUpdate** - Event type for dashboard updates

## UI Component Requirements

UI components that integrate with external crates should:

1. **Accept Generic Data Sources**:
   - Use trait objects or generic parameters for data sources
   - Support multiple backend implementations

2. **Handle Error States Gracefully**:
   - Display appropriate error messages
   - Provide fallback UI when data is unavailable
   - Retry mechanisms for temporary failures

3. **Support Asynchronous Updates**:
   - Non-blocking data fetching
   - Smooth state transitions
   - Loading indicators for pending operations

## Configuration Standards

Configuration for integrated components should:

1. **Be Centrally Managed**:
   - Single configuration system
   - Hierarchical configuration
   - Environment variable support

2. **Support Runtime Updates**:
   - Hot-reload capability
   - Apply changes without restart
   - Validate configuration changes

3. **Provide Sensible Defaults**:
   - Work out-of-the-box with minimal setup
   - Document default values
   - Explain configuration options

## Error Handling Standards

Error handling across integration points should:

1. **Use Domain-Specific Error Types**:
   - Clear error categories
   - Contextual information
   - Helpful error messages

2. **Provide Recovery Mechanisms**:
   - Retry policies
   - Fallback options
   - Graceful degradation

3. **Log Detailed Diagnostics**:
   - Structured logging
   - Error context
   - Correlation IDs

## Testing Standards

Integration tests should:

1. **Test Realistic Scenarios**:
   - End-to-end workflows
   - Error conditions
   - Performance characteristics

2. **Mock External Dependencies**:
   - Simulate various response patterns
   - Test error handling
   - Control timing and latency

3. **Verify Integration Points**:
   - Data conversion correctness
   - API contract adherence
   - Backward compatibility 