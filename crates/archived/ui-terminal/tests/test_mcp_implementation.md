# MCP Integration Implementation Tests

## Overview

This document describes the test approach for verifying the MCP integration in the UI Terminal crate. Due to the current dependency structure, we outline the tests conceptually rather than implementing them directly.

## Test Scenarios

### 1. McpMetricsProviderTrait Implementation

A fully functional test would verify:

```rust
#[tokio::test]
async fn test_metrics_provider_trait() {
    // Create a test provider implementing McpMetricsProviderTrait
    let provider = create_test_provider();
    
    // Test get_metrics
    let metrics = provider.get_metrics().await.unwrap();
    assert_eq!(metrics.message_stats.total_requests, 100);
    assert_eq!(metrics.message_stats.total_responses, 95);
    assert_eq!(metrics.error_stats.total_errors, 5);
    assert_eq!(metrics.active_connections, 2);
    
    // Test get_connection_status
    let status = provider.get_connection_status().await.unwrap();
    assert!(matches!(status, ConnectionStatus::Connected));
    
    // Test get_connection_health
    let health = provider.get_connection_health().await.unwrap();
    assert!(matches!(health.status, ConnectionStatus::Connected));
    assert_eq!(health.failure_count, 0);
    assert!(health.latency_ms.is_some());
    
    // Test get_protocol_metrics
    let protocol_metrics = provider.get_protocol_metrics().await.unwrap();
    assert!(protocol_metrics.contains_key("throughput"));
    
    // Test get_connection_history
    let history = provider.get_connection_history().await.unwrap();
    assert_eq!(history.len(), 4);
    
    // Test get_performance_metrics
    let perf = provider.get_performance_metrics().await.unwrap();
    assert_eq!(perf.metrics_requests, 100);
    assert_eq!(perf.cache_hits, 80);
}
```

### 2. RealMcpMetricsProvider Caching

The caching mechanism should be tested to ensure it properly caches metrics and respects TTL values:

```rust
#[tokio::test]
async fn test_real_provider_caching() {
    // Create a provider with caching enabled
    let provider = Arc::new(RealMcpMetricsProvider::new(
        "test_cache".to_string(),
        None,
        Duration::from_secs(1),
    ));
    
    // Manually update the cache
    let metrics = create_test_metrics(200);
    provider.update_metrics_cache(metrics.clone()).await;
    
    // Calling get_metrics should return cached value
    let cached_metrics = provider.get_metrics().await.unwrap();
    assert_eq!(cached_metrics.message_stats.total_requests, 200);
    
    // Wait for cache to expire
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Cache should be expired
    let result = provider.get_metrics().await;
    assert!(result.is_err());
}
```

### 3. Performance Metrics Tracking

Performance metrics tracking should be tested to ensure it works correctly:

```rust
#[tokio::test]
async fn test_performance_metrics() {
    // Create a provider to test performance metrics
    let provider = Arc::new(RealMcpMetricsProvider::new(
        "test_perf".to_string(),
        None,
        Duration::from_millis(500),
    ));
    
    // Simulate multiple calls to test performance tracking
    for _ in 0..5 {
        let _ = provider.get_metrics().await;
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    // Update the cache to test cache hits
    let metrics = create_test_metrics(300);
    provider.update_metrics_cache(metrics).await;
    
    // Make more calls to test cache hits
    for _ in 0..5 {
        let _ = provider.get_metrics().await;
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    // Verify performance tracking
    let perf = provider.get_performance_metrics().await.unwrap();
    assert!(perf.metrics_requests >= 10);
    assert!(perf.cache_hits >= 5);
    assert!(perf.cache_misses >= 5);
    assert!(perf.average_request_time_ms > 0.0);
}
```

### 4. Connection Management

Connection management should be tested to ensure reconnection works correctly:

```rust
#[tokio::test]
async fn test_connection_management() {
    // Create a provider with a mock client
    let provider = create_provider_with_mock_client();
    
    // Test reconnect
    let result = provider.reconnect().await;
    assert!(result.is_ok());
    assert!(result.unwrap());
    
    // Verify connection history was updated
    let history = provider.get_connection_history().await.unwrap();
    assert!(history.iter().any(|e| matches!(e.event_type, ConnectionEventType::Reconnecting)));
    assert!(history.iter().any(|e| matches!(e.event_type, ConnectionEventType::ReconnectSuccess)));
}
```

### 5. Protocol Metrics Integration

Protocol metrics integration should be tested to ensure it correctly aggregates metrics from the MCP client:

```rust
#[tokio::test]
async fn test_protocol_metrics_integration() {
    // Create a provider with a mock client that returns specific protocol metrics
    let provider = create_provider_with_metrics_client();
    
    // Get protocol metrics
    let metrics = provider.get_protocol_metrics().await.unwrap();
    
    // Verify metrics were correctly aggregated
    assert_eq!(metrics.get("request_count").unwrap(), &100.0);
    assert_eq!(metrics.get("response_count").unwrap(), &95.0);
    assert_eq!(metrics.get("error_count").unwrap(), &5.0);
    assert_eq!(metrics.get("average_latency").unwrap(), &45.6);
}
```

## Simulating MCP Client in Tests

For testing the MCP integration without relying on the actual MCP crate, we could create a mock MCP client:

```rust
struct MockMcpClient {
    metrics: HashMap<String, f64>,
    connection_status: String,
    should_fail: bool,
}

impl MockMcpClient {
    fn new() -> Self {
        let mut metrics = HashMap::new();
        metrics.insert("request_count".to_string(), 100.0);
        metrics.insert("response_count".to_string(), 95.0);
        metrics.insert("error_count".to_string(), 5.0);
        metrics.insert("average_latency".to_string(), 45.6);
        
        Self {
            metrics,
            connection_status: "connected".to_string(),
            should_fail: false,
        }
    }
    
    fn get_metrics(&self) -> Result<HashMap<String, f64>, String> {
        if self.should_fail {
            return Err("Failed to get metrics".to_string());
        }
        Ok(self.metrics.clone())
    }
    
    fn get_connection_status(&self) -> Result<String, String> {
        if self.should_fail {
            return Err("Failed to get connection status".to_string());
        }
        Ok(self.connection_status.clone())
    }
    
    fn reconnect(&self) -> Result<bool, String> {
        if self.should_fail {
            return Err("Failed to reconnect".to_string());
        }
        Ok(true)
    }
}
```

## Manual Testing Steps

Since automated tests face dependency challenges, here's a manual testing protocol:

1. Run the Terminal UI with MCP integration enabled:
   ```
   cargo run --bin ui-terminal -- --mcp --mcp-server 127.0.0.1:8778
   ```

2. Verify the Protocol tab shows MCP metrics

3. Check connection status is displayed correctly

4. Test reconnection by pressing 'r' when on the Protocol tab

5. Verify metrics update at the configured interval

6. Check history tab shows connection events

7. Validate error handling by using an invalid server address

## Implementation Validation

To validate our implementation, we should check:

1. The MCP metrics provider correctly implements the McpMetricsProviderTrait interface
2. Caching works as expected with configured TTL values
3. Performance metrics are tracked correctly
4. Connection management works properly, including reconnection
5. Protocol metrics are correctly aggregated and displayed
6. Error handling is robust and provides useful information
7. The UI correctly displays MCP metrics in the Protocol tab

## Next Steps

Once dependency issues are resolved, we can implement these tests properly to validate our MCP integration.

In the meantime, we've verified our implementation through code review and manual testing. 