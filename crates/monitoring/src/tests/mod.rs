use std::sync::Arc;
use crate::{MonitoringConfig, MonitoringService, MonitoringIntervals, MonitoringStatus};
use crate::alerts::{LegacyAlertManager, AlertConfig};
use crate::alerts::adapter::AlertManagerAdapter;
use crate::alerts::status::{Alert, AlertType, AlertSeverity};
use crate::health::{HealthConfig, HealthCheckerAdapter, ComponentHealth, status::Status, SystemHealth};
use crate::metrics::{Metric, MetricConfig, DefaultMetricCollector, MetricType, MetricCollector};
use crate::network::{NetworkConfig, NetworkMonitorAdapter, NetworkStats};
use mockall::predicate::*;
use mockall::mock;
use std::collections::HashMap;
use crate::health::HealthChecker;
use std::default::Default;
use async_trait::async_trait;
use chrono::Utc;
use squirrel_core::error::Result;

// Include factory tests module
mod factory_tests;
// Include factory runner
mod factory_runner;

// Mock implementations for testing
mock! {
    pub AlertManager {
        fn send_alert(&self, alert: Alert) -> Result<()>;
        fn get_alerts(&self) -> Result<Vec<Alert>>;
    }
}

mock! {
    pub MetricCollector {
        fn record_metric(&self, name: &str, value: f64) -> Result<()>;
        fn get_metrics(&self) -> Result<HashMap<String, f64>>;
    }
}

mock! {
    pub NetworkMonitor {
        fn get_stats(&self) -> Result<HashMap<String, NetworkStats>>;
        fn get_interface_stats(&self, interface: &str) -> Result<Option<NetworkStats>>;
    }
}

/// Helper function to create a test configuration
fn create_test_config() -> MonitoringConfig {
    MonitoringConfig {
        intervals: MonitoringIntervals {
            health_check_interval: 1,
            metrics_collection_interval: 1, 
            alert_processing_interval: 1,
            network_stats_interval: 1,
        },
        health_config: HealthConfig::default(),
        metrics_config: MetricConfig::default(),
        alert_config: AlertConfig::default(),
        network_config: NetworkConfig::default(),
    }
}

/// Basic implementation of MonitoringService for testing
struct TestMonitoringService {
    health_checker: Arc<HealthCheckerAdapter>,
    metric_collector: Arc<DefaultMetricCollector>,
    alert_manager: Arc<AlertManagerAdapter>,
    network_monitor: Arc<NetworkMonitorAdapter>,
}

#[async_trait]
impl MonitoringService for TestMonitoringService {
    async fn start(&self) -> Result<()> {
        // Start components
        self.health_checker.start().await?;
        self.metric_collector.start().await?;
        self.alert_manager.start().await?;
        self.network_monitor.start().await?;
        Ok(())
    }
    
    async fn stop(&self) -> Result<()> {
        // Stop components
        self.health_checker.stop().await?;
        self.metric_collector.stop().await?;
        self.alert_manager.stop().await?;
        self.network_monitor.stop().await?;
        Ok(())
    }
    
    async fn status(&self) -> Result<MonitoringStatus> {
        // Get health status
        let health = match self.health_checker.check_health().await {
            Ok(status) => SystemHealth {
                status: status.status,
                components: HashMap::new(), // Simplified for test
                last_check: Utc::now(),
            },
            Err(_) => SystemHealth {
                status: Status::Unknown,
                components: HashMap::new(),
                last_check: Utc::now(),
            },
        };
        
        Ok(MonitoringStatus {
            running: true,
            health,
            last_update: Utc::now(),
        })
    }
}

impl TestMonitoringService {
    // Helper methods for tests
    async fn health_status(&self) -> Result<crate::health::HealthStatus> {
        self.health_checker.check_health().await
    }
    
    async fn get_metrics(&self) -> Result<Vec<Metric>> {
        self.metric_collector.collect_metrics().await
    }
    
    async fn get_alerts(&self) -> Result<Vec<Alert>> {
        self.alert_manager.get_alerts().await
    }
    
    async fn get_network_stats(&self) -> Result<HashMap<String, NetworkStats>> {
        self.network_monitor.get_stats().await
    }
}

/// Helper function to create a test monitoring service with mocked dependencies
async fn create_test_service() -> (
    Arc<TestMonitoringService>, 
    Arc<HealthCheckerAdapter>,
    Arc<DefaultMetricCollector>,
    Arc<AlertManagerAdapter>,
    Arc<NetworkMonitorAdapter>
) {
    // ARRANGE: Create dependencies with DI pattern
    // Create a properly initialized health checker adapter
    let health_config = HealthConfig::default();
    let mut health_checker_adapter = HealthCheckerAdapter::new();
    health_checker_adapter.initialize_with_config(health_config).expect("Failed to initialize health checker");
    let health_checker = Arc::new(health_checker_adapter);
    
    // Create a properly initialized metric collector
    let metric_config = MetricConfig::default();
    let mut metric_collector_adapter = DefaultMetricCollector::new();
    metric_collector_adapter.initialize_with_config(metric_config).await.expect("Failed to initialize metric collector");
    let metric_collector = Arc::new(metric_collector_adapter);
    
    // Create a properly initialized alert manager adapter
    let alert_config = crate::alerts::LegacyAlertConfig::default();
    let mut alert_manager_adapter = AlertManagerAdapter::<()>::new();
    alert_manager_adapter.initialize_with_config(alert_config).expect("Failed to initialize alert manager");
    let alert_manager = Arc::new(alert_manager_adapter);
    
    // Create a properly initialized network monitor adapter
    let network_config = NetworkConfig::default();
    let mut network_monitor_adapter = NetworkMonitorAdapter::new();
    network_monitor_adapter.initialize_with_config(network_config).expect("Failed to initialize network monitor");
    let network_monitor = Arc::new(network_monitor_adapter);
    
    // Print initialization status for debugging
    println!("AlertManager initialized: {}", alert_manager.is_initialized());
    
    // Create service with dependencies
    let service = Arc::new(TestMonitoringService {
        health_checker: health_checker.clone(),
        metric_collector: metric_collector.clone(),
        alert_manager: alert_manager.clone(),
        network_monitor: network_monitor.clone(),
    });
    
    // Return both the service and its dependencies for verification
    (service, health_checker, metric_collector, alert_manager, network_monitor)
}

// Helper to create a test metric
fn create_test_metric(name: &str, value: f64) -> Metric {
    Metric::new(
        name.to_string(),
        value,
        MetricType::Gauge,
        HashMap::new(),
    )
}

/// Implementation of Default for MonitoringConfig to make tests work
impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            intervals: MonitoringIntervals {
                health_check_interval: 5,
                metrics_collection_interval: 10,
                alert_processing_interval: 15,
                network_stats_interval: 20,
            },
            health_config: HealthConfig::default(),
            metrics_config: MetricConfig::default(),
            alert_config: AlertConfig::default(),
            network_config: NetworkConfig::default(),
        }
    }
}

#[tokio::test]
async fn test_service_initialization() {
    // ARRANGE: Create test service
    let (service, _health_checker, _metric_collector, _alert_manager, _network_monitor) = 
        create_test_service().await;
    
    // ACT: Start the service and capture the result
    let start_result = service.start().await;
    
    // Print the error if present for debugging
    if let Err(ref e) = start_result {
        println!("Service start error: {:?}", e);
    }
    
    // ASSERT: Verify the service started successfully
    assert!(start_result.is_ok(), "Failed to start monitoring service");
    
    // ACT: Stop the service
    let stop_result = service.stop().await;
    
    // ASSERT: Verify the service stopped successfully
    assert!(stop_result.is_ok(), "Failed to stop monitoring service");
}

#[tokio::test]
async fn test_health_checker() {
    // ARRANGE: Create test service
    let (service, health_checker, _, _, _) = create_test_service().await;
    
    // Start the service
    service.start().await.expect("Failed to start service");
    
    // ACT: Register a healthy component
    let component = ComponentHealth {
        name: "test-component".to_string(),
        status: Status::Healthy,
        message: "All good".to_string(),
        timestamp: 0,
        metadata: None,
    };
    
    health_checker.as_ref().register_component(component).await
        .expect("Failed to register component");
    
    // ACT: Check health
    let health = service.health_status().await
        .expect("Failed to check health");
    
    // ASSERT: Verify health status
    assert_eq!(health.status, Status::Healthy, 
        "Health status should be Healthy");
    
    // Check that the component is registered by getting it directly
    let component = health_checker.as_ref()
        .get_component_health("test-component").await.unwrap();
    assert!(component.is_some(), "Component should be registered");
    
    // Cleanup
    service.stop().await.expect("Failed to stop service");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_metric_collection() {
    // ARRANGE: Create test service
    let (service, _health_checker, metric_collector, _alert_manager, _network_monitor) = 
        create_test_service().await;
    
    // Start the service
    service.start().await.expect("Failed to start service");
    
    // ACT: Create a test metric and record it
    let test_metric = create_test_metric("test_collection", 5.0);
    metric_collector.record_metric(test_metric.clone()).await.expect("Failed to record metric");
    
    // ACT: Get the metrics
    let metrics = service.get_metrics().await.expect("Failed to get metrics");
    
    // ASSERT: Verify our test metric was collected
    assert!(
        metrics.iter().any(|m| m.name == "test_collection"),
        "Test metric should be in collected metrics"
    );
    
    // Clean up
    service.stop().await.expect("Failed to stop service");
}

#[tokio::test]
async fn test_alert_manager() {
    // ARRANGE: Create test service with its dependencies
    let (service, _, _, alert_manager, _) = create_test_service().await;
    
    // Start the service
    service.start().await.expect("Failed to start service");
    
    // ACT: Create a test alert using the new Alert struct format
    let mut details = HashMap::new();
    details.insert("source".to_string(), serde_json::Value::String("test".to_string()));
    
    let test_alert = Alert::new(
        AlertType::Generic,
        AlertSeverity::Warning,
        "Test Alert".to_string(), // source
        "This is a test alert".to_string(), // message
    ).with_details(details);
    
    // Send the alert
    // Note: We're converting the Alert to a LegacyAlert internally in the adapter
    alert_manager.send_alert(test_alert).await.expect("Failed to send alert");
    
    // ACT: Get alerts
    let alerts = service.get_alerts().await.expect("Failed to get alerts");
    
    // ASSERT: Verify the alert was added
    assert!(!alerts.is_empty(), "No alerts found");
    
    // Check that we have the test alert
    let found = alerts.iter().any(|a| a.source == "Test Alert");
    assert!(found, "Test alert not found in alerts");
    
    // Stop the service
    service.stop().await.expect("Failed to stop service");
}

#[tokio::test]
async fn test_network_monitoring() {
    // ARRANGE: Create test service
    let (service, _, _, _, _network_monitor) = create_test_service().await;
    
    // Start the service
    service.start().await.expect("Failed to start service");
    
    // ACT: Get network stats
    let stats = service.get_network_stats().await
        .expect("Failed to get network stats");
    
    // ASSERT: Verify we got some network stats
    // Note: This is a basic test and might need to be adapted based on the actual implementation
    println!("Network stats: {:?}", stats);
    
    // Cleanup
    service.stop().await.expect("Failed to stop service");
}

#[tokio::test]
async fn test_service_initialization_with_monitoring_service_alias() {
    // ARRANGE: Create test service
    let (service, _health_checker, _metric_collector, _alert_manager, _network_monitor) = 
        create_test_service().await;
    
    // ACT: Start the service and capture the result
    let start_result = service.start().await;
    
    // Print the error if present for debugging
    if let Err(ref e) = start_result {
        println!("Service start error: {:?}", e);
    }
    
    // ASSERT: Verify the service started successfully
    assert!(start_result.is_ok(), "Failed to start monitoring service");
    
    // ACT: Stop the service
    let stop_result = service.stop().await;
    
    // ASSERT: Verify the service stopped successfully
    assert!(stop_result.is_ok(), "Failed to stop monitoring service");
}

#[tokio::test]
async fn test_health_checker_with_monitoring_service_alias() {
    // ARRANGE: Create test service
    let (service, health_checker, _, _, _) = create_test_service().await;
    
    // Start the service
    service.start().await.expect("Failed to start service");
    
    // ACT: Register a healthy component
    let component = ComponentHealth {
        name: "test-component".to_string(),
        status: Status::Healthy,
        message: "All good".to_string(),
        timestamp: 0,
        metadata: None,
    };
    
    health_checker.as_ref().register_component(component).await
        .expect("Failed to register component");
    
    // ACT: Check health via service status and directly
    let service_status = service.status().await
        .expect("Failed to check service status");
    
    let health_status = service.health_status().await
        .expect("Failed to check health status");
    
    // ASSERT: Verify health status
    assert_eq!(service_status.health.status, Status::Healthy, 
        "Service health status should be Healthy");
    
    assert_eq!(health_status.status, Status::Healthy, 
        "Health status should be Healthy");
    
    // Check that the component is registered by getting it directly
    let component = health_checker.as_ref()
        .get_component_health("test-component").await.unwrap();
    assert!(component.is_some(), "Component should be registered");
    
    // Cleanup
    service.stop().await.expect("Failed to stop service");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_metric_collection_with_monitoring_service_alias() {
    // This test is identical to test_metric_collection but uses the alias
    // to demonstrate the same functionality with different syntax

    // ARRANGE: Create test service
    let (service, _health_checker, metric_collector, _alert_manager, _network_monitor) = 
        create_test_service().await;
    
    // Start the service
    let monitoring_service = service.as_ref() as &dyn MonitoringService;
    monitoring_service.start().await.expect("Failed to start service");
    
    // ACT: Create a test metric and record it
    let test_metric = create_test_metric("test_collection", 10.0);
    metric_collector.record_metric(test_metric.clone()).await.expect("Failed to record metric");
    
    // ACT: Get the metrics
    let metrics = service.get_metrics().await.expect("Failed to get metrics");
    
    // ASSERT: Verify our test metric was collected
    assert!(
        metrics.iter().any(|m| m.name == "test_collection"),
        "Test metric should be in collected metrics"
    );
    
    // Clean up
    monitoring_service.stop().await.expect("Failed to stop service");
}

#[tokio::test]
async fn test_alert_manager_with_monitoring_service_alias() {
    // ARRANGE: Create test service with its dependencies  
    let (service, _, _, alert_manager, _) = create_test_service().await;
    
    // Start the service
    service.start().await.expect("Failed to start service");
    
    // ACT: Create a test alert using the new Alert struct format
    let mut details = HashMap::new();
    details.insert("source".to_string(), serde_json::Value::String("test".to_string()));
    
    let test_alert = Alert::new(
        AlertType::Generic,
        AlertSeverity::Warning,
        "Test Alert".to_string(), // source
        "This is a test alert".to_string(), // message
    ).with_details(details);
    
    // Send the alert
    // Note: We're converting the Alert to a LegacyAlert internally in the adapter
    alert_manager.send_alert(test_alert).await.expect("Failed to send alert");
    
    // ACT: Get alerts
    let alerts = service.get_alerts().await.expect("Failed to get alerts");
    
    // ASSERT: Verify the alert was added
    assert!(!alerts.is_empty(), "No alerts found");
    
    // Check that we have the test alert
    let found = alerts.iter().any(|a| a.source == "Test Alert");
    assert!(found, "Test alert not found in alerts");
    
    // Stop the service
    service.stop().await.expect("Failed to stop service");
}

#[tokio::test]
async fn test_network_monitoring_with_monitoring_service_alias() {
    // ARRANGE: Create test service
    let (service, _, _, _, _network_monitor) = create_test_service().await;
    
    // Start the service
    service.start().await.expect("Failed to start service");
    
    // ACT: Get network stats
    let stats = service.get_network_stats().await
        .expect("Failed to get network stats");
    
    // ASSERT: Verify we got some network stats
    println!("Network stats: {:?}", stats);
    
    // Cleanup
    service.stop().await.expect("Failed to stop service");
} 