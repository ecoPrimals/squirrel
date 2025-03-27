---
description: DEFINE specific requirements for terminal UI integration with the MCP crate
---

# MCP Integration Specification for Terminal UI

## Context

- When integrating the Terminal UI dashboard with the MCP protocol
- When collecting protocol metrics from MCP processes
- When visualizing MCP protocol statistics in the dashboard
- When monitoring MCP communication in real-time

## Requirements

### Metrics Collection Interface

The MCP crate should expose a metrics API that provides:

1. **Synchronous Access**:
   - Point-in-time metrics snapshots
   - Thread-safe retrieval

2. **Asynchronous Streaming**:
   - Real-time metrics updates
   - Configurable update frequency

#### API Structure

```rust
/// MCP metrics collection API
pub trait McpMetricsProvider: Send + Sync {
    /// Get current metrics snapshot
    fn get_metrics(&self) -> Result<McpMetrics, McpError>;
    
    /// Subscribe to metrics updates with specified interval
    fn subscribe(&self, interval_ms: u64) -> mpsc::Receiver<McpMetrics>;
}

/// Implementation for the MCP client
impl McpMetricsProvider for McpClient {
    fn get_metrics(&self) -> Result<McpMetrics, McpError> {
        // Implementation
    }
    
    fn subscribe(&self, interval_ms: u64) -> mpsc::Receiver<McpMetrics> {
        // Implementation
    }
}
```

### Metrics Data Structure

MCP metrics should be structured to provide all necessary information for the Protocol tab:

```rust
/// MCP metrics data structure
pub struct McpMetrics {
    /// Message statistics
    pub message_stats: MessageStats,
    
    /// Transaction statistics
    pub transaction_stats: TransactionStats,
    
    /// Error statistics
    pub error_stats: ErrorStats,
    
    /// Latency measurements
    pub latency_stats: LatencyStats,
    
    /// Timestamp when metrics were collected
    pub timestamp: DateTime<Utc>,
}

pub struct MessageStats {
    pub total_requests: u64,
    pub total_responses: u64,
    pub request_rate: f64,  // requests per second
    pub response_rate: f64, // responses per second
    pub request_types: HashMap<String, u64>, // count by request type
}

pub struct TransactionStats {
    pub total_transactions: u64,
    pub successful_transactions: u64,
    pub failed_transactions: u64,
    pub transaction_rate: f64, // transactions per second
    pub success_rate: f64,     // percentage
}

pub struct ErrorStats {
    pub total_errors: u64,
    pub connection_errors: u64,
    pub protocol_errors: u64,
    pub timeout_errors: u64,
    pub error_rate: f64,       // percentage
    pub error_types: HashMap<String, u64>, // count by error type
}

pub struct LatencyStats {
    pub average_latency_ms: f64,
    pub median_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub min_latency_ms: f64,
    pub max_latency_ms: f64,
    pub latency_histogram: Vec<f64>, // latency distribution
}
```

### Integration Implementation

#### Terminal UI Side

The Terminal UI should implement an adapter for MCP metrics:

```rust
/// Adapter for MCP metrics
pub struct McpMetricsAdapter {
    /// MCP client reference
    client: Option<Arc<dyn McpMetricsProvider>>,
    
    /// Cached metrics for fallback
    cached_metrics: Option<McpMetrics>,
    
    /// Last update timestamp
    last_update: DateTime<Utc>,
    
    /// Update channel receiver
    metrics_rx: Option<mpsc::Receiver<McpMetrics>>,
}

impl McpMetricsAdapter {
    /// Create a new adapter with an MCP client
    pub fn new(client: Option<Arc<dyn McpMetricsProvider>>) -> Self {
        let metrics_rx = client.as_ref().map(|c| c.subscribe(1000)); // 1-second updates
        
        Self {
            client,
            cached_metrics: None,
            last_update: Utc::now(),
            metrics_rx,
        }
    }
    
    /// Collect metrics from MCP
    pub async fn collect_metrics(&mut self) -> MetricsSnapshot {
        // Try to get metrics from the update channel first
        if let Some(rx) = &mut self.metrics_rx {
            if let Ok(Some(mcp_metrics)) = rx.try_recv().map_or(Ok(None), |m| Ok(Some(m))) {
                self.cached_metrics = Some(mcp_metrics.clone());
                self.last_update = Utc::now();
                return self.convert_to_dashboard_metrics(mcp_metrics);
            }
        }
        
        // If no updates from channel, try direct fetch
        if let Some(client) = &self.client {
            match client.get_metrics() {
                Ok(mcp_metrics) => {
                    self.cached_metrics = Some(mcp_metrics.clone());
                    self.last_update = Utc::now();
                    return self.convert_to_dashboard_metrics(mcp_metrics);
                },
                Err(e) => {
                    log::warn!("Failed to get MCP metrics: {}", e);
                    // Fall back to cached metrics
                    if let Some(cached) = &self.cached_metrics {
                        return self.convert_to_dashboard_metrics(cached.clone());
                    }
                }
            }
        }
        
        // If all else fails, return empty metrics
        self.create_empty_metrics()
    }
    
    /// Convert MCP metrics to dashboard metrics format
    fn convert_to_dashboard_metrics(&self, mcp_metrics: McpMetrics) -> MetricsSnapshot {
        let mut counters = HashMap::new();
        let mut gauges = HashMap::new();
        let mut histograms = HashMap::new();
        
        // Convert message stats
        counters.insert("protocol.messages".to_string(), 
                       mcp_metrics.message_stats.total_requests + mcp_metrics.message_stats.total_responses);
        counters.insert("mcp.requests".to_string(), mcp_metrics.message_stats.total_requests);
        counters.insert("mcp.responses".to_string(), mcp_metrics.message_stats.total_responses);
        
        gauges.insert("protocol.message_rate".to_string(), 
                     mcp_metrics.message_stats.request_rate + mcp_metrics.message_stats.response_rate);
        
        // Convert transaction stats
        counters.insert("protocol.transactions".to_string(), mcp_metrics.transaction_stats.total_transactions);
        counters.insert("mcp.transactions".to_string(), mcp_metrics.transaction_stats.total_transactions);
        gauges.insert("protocol.transaction_rate".to_string(), mcp_metrics.transaction_stats.transaction_rate);
        gauges.insert("mcp.success_rate".to_string(), mcp_metrics.transaction_stats.success_rate);
        
        // Convert error stats
        counters.insert("protocol.errors".to_string(), mcp_metrics.error_stats.total_errors);
        counters.insert("mcp.connection_errors".to_string(), mcp_metrics.error_stats.connection_errors);
        counters.insert("mcp.protocol_errors".to_string(), mcp_metrics.error_stats.protocol_errors);
        counters.insert("mcp.timeout_errors".to_string(), mcp_metrics.error_stats.timeout_errors);
        gauges.insert("protocol.error_rate".to_string(), mcp_metrics.error_stats.error_rate);
        
        // Convert latency stats
        histograms.insert("protocol.latency".to_string(), mcp_metrics.latency_stats.latency_histogram);
        gauges.insert("mcp.average_latency".to_string(), mcp_metrics.latency_stats.average_latency_ms);
        gauges.insert("mcp.p95_latency".to_string(), mcp_metrics.latency_stats.p95_latency_ms);
        
        MetricsSnapshot {
            values: HashMap::new(),
            counters,
            gauges,
            histograms,
        }
    }
    
    /// Create empty metrics when no data is available
    fn create_empty_metrics(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            values: HashMap::new(),
            counters: HashMap::new(),
            gauges: HashMap::new(),
            histograms: HashMap::new(),
        }
    }
}
```

### Dashboard Configuration

The dashboard should support MCP-specific configuration:

```rust
/// MCP dashboard configuration
pub struct McpDashboardConfig {
    /// Whether to show MCP metrics
    pub show_mcp_metrics: bool,
    
    /// Update interval in milliseconds
    pub update_interval_ms: u64,
    
    /// Connection settings
    pub connection: McpConnectionConfig,
    
    /// Display settings
    pub display: McpDisplayConfig,
}

pub struct McpConnectionConfig {
    /// MCP endpoint URL
    pub endpoint: String,
    
    /// Authentication method
    pub auth_method: McpAuthMethod,
    
    /// Connection timeout in milliseconds
    pub timeout_ms: u64,
    
    /// Retry settings
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
}

pub enum McpAuthMethod {
    None,
    ApiKey(String),
    OAuth(OAuthConfig),
}

pub struct McpDisplayConfig {
    /// Which metrics to display
    pub show_message_stats: bool,
    pub show_transaction_stats: bool,
    pub show_error_stats: bool,
    pub show_latency_stats: bool,
    
    /// Display thresholds for coloring
    pub error_warning_threshold: f64,
    pub error_critical_threshold: f64,
    pub latency_warning_threshold: f64,
    pub latency_critical_threshold: f64,
}
```

## Integration Flow

The integration between Terminal UI and MCP should follow this flow:

```
┌────────────────┐     ┌─────────────────┐     ┌───────────────┐     ┌──────────────┐
│  MCP Metrics   │────▶│ McpMetricsAdapter│────▶│ DashboardData │────▶│ Protocol Tab │
│   Provider     │     │                  │     │               │     │              │
└────────────────┘     └─────────────────┘     └───────────────┘     └──────────────┘
```

1. The MCP crate implements the `McpMetricsProvider` trait
2. The Terminal UI creates an `McpMetricsAdapter` with the MCP client
3. The adapter collects and converts metrics to the dashboard format
4. The dashboard displays the metrics in the Protocol tab

## Error Handling

1. **Connection Errors**:
   - Display "Not Connected" status when MCP client is unavailable
   - Show reconnection countdown
   - Provide manual reconnect option

2. **Data Collection Errors**:
   - Use cached data with "Stale Data" indicator
   - Show last successful update time
   - Log detailed error information

3. **Format Conversion Errors**:
   - Handle unexpected data formats gracefully
   - Show partial data if possible
   - Log format mismatches for debugging

## Testing Requirements

1. **Unit Tests**:
   - Test metrics conversion with sample MCP data
   - Verify error handling with various failure scenarios
   - Test configuration parsing and validation

2. **Integration Tests**:
   - Test with simulated MCP metrics provider
   - Verify real-time updates flow correctly
   - Test error recovery mechanisms

3. **UI Tests**:
   - Verify Protocol tab correctly displays MCP metrics
   - Test UI responsiveness with high-frequency updates
   - Verify error state displays correctly

## Implementation Priorities

1. **MVP Requirements** (Must Have):
   - Basic MCP metrics collection (messages, transactions, errors)
   - Protocol tab visualization
   - Error handling for connection failures
   - Cached metrics for temporary disconnections

2. **V1 Requirements** (Should Have):
   - Real-time updates via subscription
   - Detailed metrics breakdown by message/error type
   - Historical metrics tracking with charts
   - Configuration options for display and collection

3. **Future Enhancements** (Could Have):
   - Advanced filtering of protocol metrics
   - Drill-down capability for transaction details
   - Export capability for metrics data
   - Custom alert thresholds for MCP metrics 