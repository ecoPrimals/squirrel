use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use crate::error::Result;
use crate::monitoring::{
    MonitoringConfig, MonitoringService,
    alerts::{AlertConfig, AlertManager, AlertManagerAdapter, create_manager_adapter},
    health::{HealthConfig, HealthCheckerAdapter, create_checker_adapter},
    metrics::{MetricConfig, DefaultMetricCollector},
    network::{NetworkConfig, NetworkMonitor, NetworkMonitorAdapter, create_monitor_adapter},
};
use mockall::predicate::*;
use mockall::mock;
use std::collections::HashMap;
use crate::test_utils::{TestError, TestData};

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
            health_check: 1,
            metric_collection: 1,
            network_monitoring: 1,
        },
        health: HealthConfig::default(),
        metrics: MetricConfig::default(),
        alerts: AlertConfig::default(),
        network: NetworkConfig::default(),
    }
}

/// Helper function to create a test monitoring service with mocked dependencies
async fn create_test_service() -> (
    MonitoringService, 
    Arc<HealthCheckerAdapter>,
    Arc<DefaultMetricCollector>,
    Arc<AlertManagerAdapter>,
    Arc<NetworkMonitorAdapter>
) {
    // ARRANGE: Create dependencies with DI pattern
    let health_checker = Arc::new(HealthCheckerAdapter::new());
    let metric_collector = Arc::new(DefaultMetricCollector::new());
    let alert_manager = Arc::new(AlertManagerAdapter::new());
    let network_monitor = Arc::new(NetworkMonitorAdapter::new());
    
    // Create service with dependencies
    let config = MonitoringConfig::default();
    let service = MonitoringService::with_dependencies(
        config,
        health_checker.clone(),
        metric_collector.clone(),
        alert_manager.clone(),
        network_monitor.clone(),
    );
    
    // Return both the service and its dependencies for verification
    (service, health_checker, metric_collector, alert_manager, network_monitor)
}

// Helper to create a test metric
fn create_test_metric(name: &str, value: f64) -> Metric {
    Metric::new(
        name.to_string(),
        value,
        MetricType::Gauge,
        None
    )
}

#[tokio::test]
async fn test_service_initialization() {
    // ARRANGE: Create test service
    let (service, health_checker, metric_collector, alert_manager, network_monitor) = 
        create_test_service().await;
    
    // ACT: Start the service
    let start_result = service.start().await;
    
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
    
    health_checker.register_component(component).await
        .expect("Failed to register component");
    
    // ACT: Check health
    let health = service.health_status().await
        .expect("Failed to check health");
    
    // ASSERT: Verify health status
    assert_eq!(health.status, Status::Healthy, 
        "Health status should be Healthy");
    assert!(health.components.contains_key("test-component"),
        "Component should be registered");
    
    // Cleanup
    service.stop().await.expect("Failed to stop service");
}

#[tokio::test]
async fn test_metric_collection() {
    // ARRANGE: Create test service
    let (service, _, metric_collector, _, _) = create_test_service().await;
    
    // Start the service
    service.start().await.expect("Failed to start service");
    
    // ACT: Record a metric
    let test_metric = create_test_metric("test-metric", 42.0);
    metric_collector.record_metric(test_metric.clone()).await
        .expect("Failed to record metric");
    
    // ACT: Get metrics
    let metrics = service.get_metrics().await
        .expect("Failed to get metrics");
    
    // ASSERT: Verify metrics
    assert!(!metrics.is_empty(), "Metrics should not be empty");
    let found = metrics.iter().any(|m| m.name == "test-metric" && m.value == 42.0);
    assert!(found, "Test metric should be present");
    
    // Cleanup
    service.stop().await.expect("Failed to stop service");
}

#[tokio::test]
async fn test_alert_manager() {
    // ARRANGE: Create test service
    let (service, _, _, alert_manager, _) = create_test_service().await;
    
    // Start the service
    service.start().await.expect("Failed to start service");
    
    // ACT: Create and add an alert
    let test_alert = Alert::new(
        "Test Alert".to_string(),
        "This is a test alert".to_string(),
        AlertSeverity::Warning,
        HashMap::new(),
        "Test alert message".to_string(),
        "test-component".to_string(),
    );
    
    alert_manager.add_alert(test_alert.clone()).await
        .expect("Failed to add alert");
    
    // ACT: Get alerts
    let alerts = service.get_alerts().await
        .expect("Failed to get alerts");
    
    // ASSERT: Verify alert was added
    assert!(!alerts.is_empty(), "Alerts should not be empty");
    let found = alerts.iter().any(|a| a.name == "Test Alert");
    assert!(found, "Test alert should be present");
    
    // Cleanup
    service.stop().await.expect("Failed to stop service");
}

#[tokio::test]
async fn test_network_monitoring() {
    // ARRANGE: Create test service
    let (service, _, _, _, network_monitor) = create_test_service().await;
    
    // Start the service
    service.start().await.expect("Failed to start service");
    
    // ACT: Get network stats
    let stats = service.get_network_stats().await;
    
    // ASSERT: Verify we can get network stats without error
    assert!(stats.is_ok(), "Should be able to get network stats");
    
    // Cleanup
    service.stop().await.expect("Failed to stop service");
}

#[tokio::test]
async fn test_full_monitoring_flow() {
    // ARRANGE: Create test service
    let (service, health_checker, metric_collector, alert_manager, _) = 
        create_test_service().await;
    
    // Start the service
    service.start().await.expect("Failed to start service");
    
    // ACT: Register component
    let component = ComponentHealth {
        name: "test-component".to_string(),
        status: Status::Healthy,
        message: "All good".to_string(),
        timestamp: 0,
        metadata: None,
    };
    
    health_checker.register_component(component).await
        .expect("Failed to register component");
    
    // Record a metric
    let test_metric = create_test_metric("test-metric", 42.0);
    metric_collector.record_metric(test_metric.clone()).await
        .expect("Failed to record metric");
    
    // Add an alert
    let test_alert = Alert::new(
        "Test Alert".to_string(),
        "This is a test alert".to_string(),
        AlertSeverity::Warning,
        HashMap::new(),
        "Test alert message".to_string(),
        "test-component".to_string(),
    );
    
    alert_manager.add_alert(test_alert.clone()).await
        .expect("Failed to add alert");
    
    // Get all data
    let health = service.health_status().await
        .expect("Failed to check health");
    let metrics = service.get_metrics().await
        .expect("Failed to get metrics");
    let alerts = service.get_alerts().await
        .expect("Failed to get alerts");
    
    // ASSERT: Verify all data is present
    assert_eq!(health.status, Status::Healthy);
    assert!(health.components.contains_key("test-component"));
    
    assert!(!metrics.is_empty());
    let metric_found = metrics.iter().any(|m| m.name == "test-metric");
    assert!(metric_found, "Test metric should be present");
    
    assert!(!alerts.is_empty());
    let alert_found = alerts.iter().any(|a| a.name == "Test Alert");
    assert!(alert_found, "Test alert should be present");
    
    // Cleanup
    service.stop().await.expect("Failed to stop service");
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_metric_values_are_preserved(value in -1000.0..1000.0) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let config = create_test_config();
                let service = MonitoringService::new(config);
                
                service.start().await.unwrap();
                service.record_metric("test_metric", value).await.unwrap();
                
                let metrics = service.get_metrics().await.unwrap();
                prop_assert_eq!(metrics.get("test_metric").unwrap(), &value);
                
                service.stop().await.unwrap();
            });
        }

        #[test]
        fn test_alert_fields_are_preserved(
            name in "[a-zA-Z0-9_]{1,20}",
            description in "[a-zA-Z0-9_\\s]{1,50}",
            message in "[a-zA-Z0-9_\\s]{1,50}",
            component in "[a-zA-Z0-9_]{1,20}",
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let config = create_test_config();
                let service = MonitoringService::new(config);
                
                service.start().await.unwrap();
                
                let alert = Alert::new(
                    name.clone(),
                    description.clone(),
                    AlertSeverity::Warning,
                    HashMap::new(),
                    message.clone(),
                    component.clone(),
                );
                
                service.send_alert(alert).await.unwrap();
                
                let alerts = service.get_alerts().await.unwrap();
                let sent_alert = alerts.iter().find(|a| a.name == name).unwrap();
                
                prop_assert_eq!(sent_alert.description, description);
                prop_assert_eq!(sent_alert.message, message);
                prop_assert_eq!(sent_alert.component, component);
                
                service.stop().await.unwrap();
            });
        }
    }
}

#[cfg(test)]
mod stress_tests {
    use super::*;
    use futures::future::join_all;
    use std::time::Instant;

    #[tokio::test]
    async fn test_concurrent_metric_recording() {
        let config = create_test_config();
        let service = Arc::new(MonitoringService::new(config));
        
        let start = Instant::now();
        let mut tasks = Vec::new();
        
        for i in 0..1000 {
            let service = service.clone();
            tasks.push(tokio::spawn(async move {
                service.record_metric(&format!("metric_{}", i), i as f64).await
            }));
        }
        
        let results = join_all(tasks).await;
        let duration = start.elapsed();
        
        // All operations should succeed
        assert!(results.iter().all(|r| r.as_ref().unwrap().is_ok()));
        
        // Should complete within reasonable time (adjust as needed)
        assert!(duration < Duration::from_secs(5));
        
        let metrics = service.get_metrics().await.unwrap();
        assert_eq!(metrics.len(), 1000);
    }

    #[tokio::test]
    async fn test_concurrent_alert_processing() {
        let config = create_test_config();
        let service = Arc::new(MonitoringService::new(config));
        
        let start = Instant::now();
        let mut tasks = Vec::new();
        
        for i in 0..100 {
            let service = service.clone();
            let alert = Alert::new(
                format!("alert_{}", i),
                "Test alert".to_string(),
                AlertSeverity::Warning,
                HashMap::new(),
                "Test message".to_string(),
                "test_component".to_string(),
            );
            
            tasks.push(tokio::spawn(async move {
                service.send_alert(alert).await
            }));
        }
        
        let results = join_all(tasks).await;
        let duration = start.elapsed();
        
        // All operations should succeed
        assert!(results.iter().all(|r| r.as_ref().unwrap().is_ok()));
        
        // Should complete within reasonable time (adjust as needed)
        assert!(duration < Duration::from_secs(5));
        
        let alerts = service.get_alerts().await.unwrap();
        assert_eq!(alerts.len(), 100);
    }
} 