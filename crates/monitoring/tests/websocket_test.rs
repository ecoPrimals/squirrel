use std::sync::Arc;
use tokio::time::Duration;
use squirrel_monitoring::{
    dashboard::{self, DashboardManager, DashboardConfig, Config, AlertManagerTrait},
    Result,
    health::{HealthChecker, status::HealthStatus, ComponentHealth},
    metrics::{MetricCollector, Metric},
    alerts::{Alert},
};

/// Integration test for the WebSocket dashboard functionality
#[tokio::test]
async fn test_dashboard_websocket() -> Result<()> {
    // Create a dashboard configuration with WebSocket enabled
    let config = DashboardConfig {
        enabled: true,
        refresh_interval: 1, // Fast refresh for testing
        max_metrics: 100,
        websocket_port: 9898, // Use a different port for testing
    };
    
    // Create and start the dashboard manager
    let mut dashboard = DashboardManager::new(config.clone());
    
    // Try to start the dashboard manager
    dashboard.start().await?;
    println!("Dashboard started successfully with WebSocket server on port 9898");
    
    // Wait a moment to make sure the server is running
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Test the server by creating a mock layout
    let layout = dashboard::Layout {
        id: "test_layout".to_string(),
        name: "Test Layout".to_string(),
        description: "Test layout for WebSocket integration test".to_string(),
        components: vec![
            dashboard::Component::PerformanceGraph {
                id: "test_performance_graph".to_string(),
                title: "Test Performance Graph".to_string(),
                description: "Test performance graph for WebSocket".to_string(),
                operation_type: squirrel_monitoring::metrics::performance::OperationType::NetworkRequest,
                time_range: Duration::from_secs(60),
            },
        ],
        grid: serde_json::json!({"layout": "grid"}),
        created_at: time::OffsetDateTime::now_utc(),
        updated_at: time::OffsetDateTime::now_utc(),
    };
    
    // Create a Manager with converted Config
    let dashboard_config = Config::from(config);
    let manager = dashboard::Manager::new(
        dashboard_config,
        Box::new(TestMetricCollector {}),
        Box::new(TestAlertManager {}),
        Box::new(TestHealthChecker {})
    );
    let arc_manager = Arc::new(manager);
    
    // Add the layout
    arc_manager.add_layout(layout).await?;
    println!("Added test layout");
    
    // Add some sample data
    let mut data = std::collections::HashMap::new();
    data.insert(
        "test_performance_graph".to_string(),
        serde_json::json!({
            "value": 42.0,
            "timestamp": time::OffsetDateTime::now_utc(),
        }),
    );
    arc_manager.update_data(data).await?;
    println!("Added test data");
    
    // Allow time for WebSocket to process
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Stop the dashboard manager
    dashboard.stop().await?;
    println!("Dashboard stopped successfully");
    
    Ok(())
}

// Test implementations
#[derive(Debug)]
struct TestMetricCollector {}

#[async_trait::async_trait]
impl MetricCollector for TestMetricCollector {
    async fn collect_metrics(&self) -> Result<Vec<Metric>> {
        Ok(Vec::new())
    }
    
    async fn record_metric(&self, _metric: Metric) -> Result<()> {
        Ok(())
    }
    
    async fn start(&self) -> Result<()> {
        Ok(())
    }
    
    async fn stop(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
struct TestAlertManager {}

#[async_trait::async_trait]
impl AlertManagerTrait for TestAlertManager {
    async fn process_alerts(&self) -> Result<()> {
        Ok(())
    }
    
    async fn add_alert(&self, _alert: Alert) -> Result<()> {
        Ok(())
    }
    
    async fn get_alerts(&self) -> Result<Vec<Alert>> {
        Ok(Vec::new())
    }
    
    async fn acknowledge_alert(&self, _alert_id: &str) -> Result<()> {
        Ok(())
    }
    
    async fn start(&self) -> Result<()> {
        Ok(())
    }
    
    async fn stop(&self) -> Result<()> {
        Ok(())
    }
    
    async fn get_active_alerts(&self) -> Result<Vec<Alert>> {
        Ok(Vec::new())
    }
}

#[derive(Debug)]
struct TestHealthChecker {}

#[async_trait::async_trait]
impl HealthChecker for TestHealthChecker {
    async fn check_health(&self) -> Result<HealthStatus> {
        Ok(HealthStatus::default())
    }
    
    async fn start(&self) -> Result<()> {
        Ok(())
    }
    
    async fn stop(&self) -> Result<()> {
        Ok(())
    }
    
    async fn initialize(&self) -> Result<()> {
        Ok(())
    }
    
    async fn get_component_health<'a>(&'a self, _component: &'a str) -> Result<Option<ComponentHealth>> {
        Ok(None)
    }
} 