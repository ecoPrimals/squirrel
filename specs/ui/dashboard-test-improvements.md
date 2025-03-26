---
version: 1.1.0
last_updated: 2024-06-24
status: in_progress
priority: high
---

# Dashboard Testing Improvements Specification

## Overview

This document outlines the test strategy for the dashboard components after the migration from the `squirrel-monitoring` crate to the dedicated `dashboard-core` and `ui-terminal` crates. Currently, several tests have been marked as `#[ignore]` due to connection failures with the WebSocket server during testing, and these need to be fixed in the new architecture.

## Current Issues

The dashboard tests are failing with the following errors:

1. **WebSocket Connection Failures**: All tests fail with a "500 Internal Server Error" when trying to connect to the dashboard's WebSocket endpoints.
   ```
   Error: Generic("Failed to connect: HTTP error: 500 Internal Server Error")
   ```

2. **Test Isolation**: Tests are using hardcoded ports (9902, 9903, 9904) which can cause conflicts if multiple tests run in parallel or if those ports are already in use.

3. **WebSocket Server Lifecycle Management**: The dashboard server is started correctly, but there appears to be an issue with how it handles WebSocket connections during testing.

4. **Cross-Crate Testing**: After the migration, tests need to verify correct integration between `dashboard-core`, `ui-terminal`, and `squirrel-monitoring` crates.

## Testing Architecture

Given the new architecture with separate crates, we need to implement a structured testing approach:

1. **Unit Tests**: Test individual components in each crate
2. **Integration Tests**: Test interactions between components within each crate 
3. **Cross-Crate Tests**: Test interactions between the `dashboard-core`, `ui-terminal`, and `squirrel-monitoring` crates
4. **End-to-End Tests**: Test complete workflows across all crates

## Required Improvements

### 1. Mock WebSocket Server

Create a proper mock WebSocket server implementation for testing:

```rust
pub struct MockDashboardServer {
    port: u16,
    server_handle: Option<JoinHandle<()>>,
    message_tx: mpsc::Sender<Message>,
    message_rx: mpsc::Receiver<Message>,
}

impl MockDashboardServer {
    pub async fn new() -> Self {
        // Find an available port instead of hardcoding
        let port = find_available_port().await;
        let (message_tx, message_rx) = mpsc::channel(100);
        
        Self {
            port,
            server_handle: None,
            message_tx,
            message_rx,
        }
    }
    
    pub async fn start(&mut self) -> Result<SocketAddr> {
        // Implementation of a test-specific WebSocket server
        // that doesn't depend on the entire dashboard implementation
    }
    
    pub async fn stop(&mut self) -> Result<()> {
        // Properly shutdown the server
    }
    
    pub async fn send_message(&self, message: Message) -> Result<()> {
        // Send a message to connected clients
    }
    
    pub async fn receive_message(&mut self) -> Option<Message> {
        // Receive a message from clients
    }
}
```

### 2. Test Fixtures and Utilities

Create test fixtures to simplify test setup and teardown:

```rust
pub struct DashboardTestFixture {
    dashboard_service: Arc<dyn DashboardService>,
    mock_server: MockDashboardServer,
    client: Option<WebSocketClient>,
    address: SocketAddr,
}

impl DashboardTestFixture {
    pub async fn new() -> Result<Self> {
        // Create and configure test components
    }
    
    pub async fn connect_client(&mut self) -> Result<()> {
        // Connect a test client to the dashboard
    }
    
    pub async fn cleanup(&mut self) -> Result<()> {
        // Clean up all resources
    }
}

// Helper function to find an available port
async fn find_available_port() -> u16 {
    // Implementation to dynamically find an open port
}
```

### 3. Testing Dashboard Core

Implement comprehensive tests for the `dashboard-core` crate:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    
    #[tokio::test]
    async fn test_dashboard_service_provides_metrics() {
        // Test that the dashboard service correctly provides metrics
        let service = create_test_dashboard_service();
        let data = service.get_dashboard_data().await.unwrap();
        
        assert!(!data.metrics.is_empty());
        // Additional assertions
    }
    
    #[tokio::test]
    async fn test_dashboard_service_provides_alerts() {
        // Test that the dashboard service correctly provides alerts
        let service = create_test_dashboard_service();
        let data = service.get_dashboard_data().await.unwrap();
        
        // Assertions about alerts
    }
    
    // Additional tests for other dashboard core functionality
}
```

### 4. Testing UI Terminal

Implement tests for the `ui-terminal` crate:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use dashboard_core::DashboardService;
    use mockall::predicate::*;
    
    mock! {
        DashboardServiceMock {}
        impl DashboardService for DashboardServiceMock {
            async fn get_dashboard_data(&self) -> Result<DashboardData>;
            async fn get_metric_history(...) -> Result<Vec<MetricDataPoint>>;
            async fn acknowledge_alert(&self, alert_id: &str) -> Result<()>;
            async fn configure_dashboard(&self, config: DashboardConfig) -> Result<()>;
            async fn subscribe(&self) -> Result<mpsc::Receiver<DashboardUpdate>>;
        }
    }
    
    #[test]
    fn test_ui_terminal_renders_metrics() {
        // Test that the UI correctly renders metrics
        let mut mock = MockDashboardServiceMock::new();
        
        // Configure mock
        mock.expect_get_dashboard_data()
            .returning(|| Ok(create_test_dashboard_data()));
        
        // Test UI rendering
        let ui = create_test_ui(mock);
        
        // Assertions about UI rendering
    }
    
    // Additional tests for UI components
}
```

### 5. Cross-Crate Integration Tests

Create integration tests between `dashboard-core`, `ui-terminal`, and `squirrel-monitoring`:

```rust
#[cfg(test)]
mod integration_tests {
    use dashboard_core::DashboardService;
    use ui_terminal::TerminalDashboard;
    use squirrel_monitoring::metrics::MetricsCollector;
    
    #[tokio::test]
    async fn test_monitoring_to_dashboard_integration() {
        // Test that metrics from monitoring flow correctly to dashboard
        let metrics_collector = MetricsCollector::new();
        let dashboard_service = create_dashboard_service_with_metrics(metrics_collector.clone());
        
        // Record some metrics
        metrics_collector.record_value("test.metric", 42.0);
        
        // Verify dashboard receives the metrics
        let dashboard_data = dashboard_service.get_dashboard_data().await.unwrap();
        
        assert!(dashboard_data.metrics.iter().any(|m| m.name == "test.metric" && m.value == 42.0));
    }
    
    #[tokio::test]
    async fn test_end_to_end_ui_display() {
        // Test end-to-end flow from metrics collection to UI display
        // This might use a headless UI testing approach
    }
}
```

## Test Cases to Implement

### 1. Basic Connectivity Test

```rust
#[tokio::test]
async fn test_dashboard_connectivity() -> Result<()> {
    let mut fixture = DashboardTestFixture::new().await?;
    
    // Connect client
    fixture.connect_client().await?;
    
    // Verify connection is established
    assert!(fixture.client_is_connected());
    
    // Cleanup
    fixture.cleanup().await?;
    
    Ok(())
}
```

### 2. Metrics Subscription Test

```rust
#[tokio::test]
async fn test_dashboard_metrics_subscription() -> Result<()> {
    let mut fixture = DashboardTestFixture::new().await?;
    
    // Connect client and subscribe to metrics
    fixture.connect_client().await?;
    fixture.subscribe_to_metrics().await?;
    
    // Generate test metrics
    let test_metrics = fixture.generate_test_metrics(5);
    fixture.publish_metrics(test_metrics.clone()).await?;
    
    // Verify client receives metrics
    let received_metrics = fixture.collect_received_metrics(Duration::from_secs(5)).await?;
    assert_metrics_match(test_metrics, received_metrics);
    
    // Cleanup
    fixture.cleanup().await?;
    
    Ok(())
}
```

### 3. Alert Notification Test

```rust
#[tokio::test]
async fn test_dashboard_alerts() -> Result<()> {
    let mut fixture = DashboardTestFixture::new().await?;
    
    // Connect client and subscribe to alerts
    fixture.connect_client().await?;
    fixture.subscribe_to_alerts().await?;
    
    // Generate test alerts
    let test_alerts = fixture.generate_test_alerts(3);
    fixture.publish_alerts(test_alerts.clone()).await?;
    
    // Verify client receives alerts
    let received_alerts = fixture.collect_received_alerts(Duration::from_secs(5)).await?;
    assert_alerts_match(test_alerts, received_alerts);
    
    // Cleanup
    fixture.cleanup().await?;
    
    Ok(())
}
```

### 4. UI Terminal Rendering Test

```rust
#[test]
fn test_terminal_dashboard_render() {
    // Create mock dashboard service
    let mock_service = create_mock_dashboard_service();
    
    // Create terminal dashboard with mock service
    let terminal_dashboard = TerminalDashboard::new(mock_service);
    
    // Simulate UI rendering
    let buffer = terminal_dashboard.render_to_buffer();
    
    // Verify UI elements are rendered correctly
    assert!(buffer.contains_widget("Metrics"));
    assert!(buffer.contains_widget("Alerts"));
    // More assertions
}
```

## Implementation Plan

### Phase 1: Core Testing Infrastructure
1. Create mock WebSocket server implementation
2. Implement test fixtures and utilities
3. Create test helper functions

### Phase 2: Dashboard Core Tests
1. Implement unit tests for dashboard models
2. Implement unit tests for dashboard services
3. Implement integration tests within dashboard-core

### Phase 3: UI Terminal Tests
1. Implement unit tests for UI components
2. Implement rendering tests
3. Implement event handling tests

### Phase 4: Cross-Crate Integration Tests
1. Implement dashboard-core to ui-terminal integration tests
2. Implement monitoring to dashboard-core integration tests
3. Implement end-to-end tests

### Phase 5: Continuous Integration
1. Add test suites to CI pipeline
2. Set up test coverage reporting
3. Create test documentation

## Testing Tools and Libraries

1. **mockall**: For creating mock implementations of traits
2. **tokio-test**: For testing async code
3. **test-context**: For managing test fixtures
4. **insta**: For snapshot testing of UI components
5. **criterion**: For benchmarking performance
6. **coverage-tools**: For measuring test coverage

## Success Criteria

1. All tests pass consistently
2. No more `#[ignore]` tags on tests due to connection issues
3. Test coverage of at least 80% for each crate
4. End-to-end tests verify complete functionality
5. Performance tests show acceptable latency for updates

## Next Steps

1. Implement mock WebSocket server and test fixtures
2. Convert existing tests to use new test infrastructure
3. Add missing tests for UI components
4. Implement cross-crate integration tests
5. Set up continuous integration for all test suites 