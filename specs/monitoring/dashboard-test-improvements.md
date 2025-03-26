---
version: 1.0.0
last_updated: 2024-05-28
status: proposed
priority: medium
---

# Dashboard Testing Improvements Specification

## Overview

This document outlines the work required to resolve the issues with the dashboard integration tests in the Squirrel monitoring system. Currently, these tests have been marked as `#[ignore]` due to connection failures with the WebSocket server during testing.

## Current Issues

The dashboard tests are failing with the following errors:

1. **WebSocket Connection Failures**: All tests fail with a "500 Internal Server Error" when trying to connect to the dashboard's WebSocket endpoints.
   ```
   Error: Generic("Failed to connect: HTTP error: 500 Internal Server Error")
   ```

2. **Test Isolation**: Tests are using hardcoded ports (9902, 9903, 9904) which can cause conflicts if multiple tests run in parallel or if those ports are already in use.

3. **WebSocket Server Lifecycle Management**: The dashboard server is started correctly, but there appears to be an issue with how it handles WebSocket connections during testing.

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
    dashboard: DashboardManager,
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

### 3. Dashboard Manager Testing Mode

Extend the DashboardManager to support a testing mode:

```rust
impl DashboardManager {
    pub fn new_for_testing(config: DashboardConfig) -> Self {
        let mut dashboard = Self::new(config);
        dashboard.testing_mode = true;
        dashboard
    }
    
    fn initialize_testing_server(&self) -> Result<()> {
        // Initialize a simplified server for testing
    }
}
```

### 4. Test-Specific WebSocket Handler

Create a simplified WebSocket handler for testing:

```rust
pub struct TestWebSocketHandler {
    dashboard_data: Arc<RwLock<DashboardData>>,
    clients: Arc<RwLock<HashMap<String, mpsc::Sender<Message>>>>,
}

impl TestWebSocketHandler {
    pub fn new() -> Self {
        // Initialize with test data
    }
    
    pub async fn handle_connection(&self, socket: WebSocket) {
        // Handle WebSocket connection for testing
    }
    
    pub async fn update_data(&self, data: DashboardData) {
        // Update test data and notify clients
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

### 4. Component Listing Test

```rust
#[tokio::test]
async fn test_dashboard_components() -> Result<()> {
    let mut fixture = DashboardTestFixture::new().await?;
    
    // Connect client
    fixture.connect_client().await?;
    
    // Request component list
    fixture.request_components_list().await?;
    
    // Verify response
    let components_list = fixture.receive_components_list().await?;
    assert!(!components_list.is_empty());
    assert!(components_list.iter().all(|c| c.id.is_some() && c.name.is_some()));
    
    // Cleanup
    fixture.cleanup().await?;
    
    Ok(())
}
```

## Implementation Plan

### Phase 1: Test Infrastructure

1. Create the `MockDashboardServer` implementation
2. Implement port discovery utility functions
3. Create the `DashboardTestFixture` helper

### Phase 2: Dashboard Modifications

1. Add testing mode to `DashboardManager`
2. Implement simplified WebSocket handlers for testing
3. Create test-specific data generators

### Phase 3: Test Implementation

1. Convert existing tests to use the new test infrastructure
2. Fix WebSocket connection handling
3. Improve test isolation with dynamic ports
4. Add proper cleanup for all tests

### Phase 4: Additional Test Coverage

1. Add more comprehensive tests for dashboard functionality
2. Implement stress tests for multiple connections
3. Test error handling and recovery scenarios

## Success Criteria

- All dashboard tests run successfully without being marked as ignored
- Tests can run in parallel without port conflicts
- Testing infrastructure is reusable for future dashboard tests
- Test coverage for dashboard components reaches at least 80%

## Dependencies

- tokio = "1.0"
- tokio-tungstenite = "0.19"
- futures-util = "0.3"
- serde_json = "1.0"
- rand = "0.8" (for port selection)

## Estimated Effort

- **Phase 1**: 1-2 days
- **Phase 2**: 2-3 days  
- **Phase 3**: 1-2 days
- **Phase 4**: 2-3 days

Total: 6-10 days of development time 