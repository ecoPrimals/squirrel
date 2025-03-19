use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use crate::error::Result;
use crate::monitoring::{
    MonitoringConfig, MonitoringService, MonitoringIntervals,
    Alert, NetworkStats, Metric, MetricType,
    alerts::{AlertConfig, AlertManager, AlertManagerAdapter, create_manager_adapter, AlertSeverity, AlertManagerFactory, DefaultAlertManager},
    health::{HealthConfig, HealthCheckerAdapter, create_checker_adapter, ComponentHealth, status::Status},
    metrics::{MetricConfig, DefaultMetricCollector},
    network::{NetworkConfig, NetworkMonitor, NetworkMonitorAdapter, create_monitor_adapter},
};
use mockall::predicate::*;
use mockall::mock;
use std::collections::HashMap;
use crate::test_utils::{TestError, TestData};
use crate::monitoring::metrics::MetricCollector;
use crate::monitoring::health::HealthChecker;

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
    // Create a properly initialized health checker adapter
    let health_config = HealthConfig::default();
    let mut health_checker_adapter = HealthCheckerAdapter::new();
    health_checker_adapter.initialize_with_config(health_config).expect("Failed to initialize health checker");
    let health_checker = Arc::new(health_checker_adapter);
    
    // Create a properly initialized metric collector
    let metric_config = MetricConfig::default();
    let mut metric_collector_adapter = DefaultMetricCollector::new();
    metric_collector_adapter.initialize_with_config(metric_config).expect("Failed to initialize metric collector");
    let metric_collector = Arc::new(metric_collector_adapter);
    
    // Create a properly initialized alert manager adapter
    let alert_config = AlertConfig::default();
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
        HashMap::new(),
    )
}

#[tokio::test]
async fn test_service_initialization() {
    // ARRANGE: Create test service
    let (service, health_checker, metric_collector, alert_manager, network_monitor) = 
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
    // ARRANGE: Create service
    let (service, _, metric_collector, alert_manager, _) = create_test_service().await;
    
    // Start the service
    service.start().await.expect("Failed to start service");
    
    // ACT: Register a component
    let component = ComponentHealth::new(
        "test-component".to_string(),
        Status::Healthy,
        "All good".to_string(),
    );
    
    service.health_checker.register_component(component).await
        .expect("Failed to register component");
    
    // ACT: Record a metric
    let test_metric = create_test_metric("test-metric", 42.0);
    metric_collector.record_metric(test_metric.clone()).await
        .expect("Failed to record metric");
    
    // ACT: Create and send an alert
    let mut labels = HashMap::new();
    labels.insert("service".to_string(), "test".to_string());
    
    let test_alert = Alert::new(
        "Test Alert".to_string(),
        "Test alert description".to_string(),
        AlertSeverity::Medium,
        labels,
        "Test alert message".to_string(),
        "test-component".to_string(),
    );
    
    alert_manager.add_alert(test_alert.clone()).await
        .expect("Failed to add alert");
    
    // Get all data
    let health = service.health_status().await.unwrap();
    assert_eq!(health.status, Status::Healthy);
    
    // Check the components from the health checker
    let component = service.health_checker
        .get_component_health("test-component").await.unwrap();
    assert!(component.is_some(), "Health check should contain registered component");
    
    let metrics = service.get_metrics().await
        .expect("Failed to get metrics");
    let alerts = service.get_alerts().await
        .expect("Failed to get alerts");
    
    // ASSERT: Verify all data is present
    assert!(!metrics.is_empty());
    let metric_found = metrics.iter().any(|m| m.name == "test-metric");
    assert!(metric_found, "Test metric should be present");
    
    assert!(!alerts.is_empty());
    let alert_found = alerts.iter().any(|a| a.name == "Test Alert");
    assert!(alert_found, "Test alert should be present");
    
    // Cleanup
    service.stop().await.expect("Failed to stop service");
}

mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::TestCaseResult;

    proptest! {
        #[test]
        fn test_metric_values_are_preserved(value in -1000.0..1000.0) {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            
            rt.block_on(async {
                let config = create_test_config();
                let service = MonitoringService::new(config);
                
                service.start().await.unwrap();
                service.record_metric("test_metric", value).await.unwrap();
                
                let metrics = service.get_metrics().await.unwrap();
                let found = metrics.iter().find(|m| m.name == "test_metric");
                prop_assert!(found.is_some());
                prop_assert_eq!(found.unwrap().value, value);
                
                service.stop().await.unwrap();
                Ok(())
            }).unwrap()
        }

        #[test]
        fn test_alert_fields_are_preserved(
            name in "[a-zA-Z0-9_]{1,20}",
            description in "[a-zA-Z0-9_\\s]{1,50}",
            message in "[a-zA-Z0-9_\\s]{1,50}",
            component in "[a-zA-Z0-9_]{1,20}",
        ) {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            
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
                prop_assert!(!alerts.is_empty());
                
                let found = alerts.iter().find(|a| a.name == name);
                prop_assert!(found.is_some());
                
                let alert = found.unwrap();
                prop_assert_eq!(&alert.description, &description);
                prop_assert_eq!(&alert.message, &message);
                prop_assert_eq!(&alert.component, &component);
                
                service.stop().await.unwrap();
                Ok(())
            }).unwrap()
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

#[tokio::test]
async fn test_metric_collector_basic() {
    // Set up test
    let (service, health_checker, metric_collector, alert_manager, _network_monitor) =
        create_test_service().await;

    // Create a test component
    let component = ComponentHealth::new(
        "test-component".to_string(), 
        Status::Healthy, 
        "All good".to_string()
    );
    
    // Register component with health checker
    health_checker.register_component(component).await
        .expect("Failed to register component");

    // Create a test metric
    let test_metric = Metric::new(
        "test_metric".to_string(),
        42.0,
        MetricType::Gauge,
        HashMap::new(),
    );

    // Record metric with collector
    metric_collector.record_metric(test_metric.clone()).await
        .expect("Failed to record metric");

    // Verify health status
    let health = health_checker.check_health().await.expect("Failed to get health");
    assert_eq!(health.status, Status::Healthy);
    
    // Directly check components from the health checker
    let component = health_checker.get_component_health("test-component").await.unwrap();
    assert!(component.is_some(), "Health check should contain registered component");

    // Create test alert
    let mut labels = HashMap::new();
    labels.insert("test".to_string(), "value".to_string());

    let alert = Alert::new(
        "Test Alert".to_string(),
        "Test description".to_string(),
        AlertSeverity::Warning,
        labels,
        "Test alert message".to_string(),
        "test-component".to_string(),
    );

    // Send alert
    alert_manager.add_alert(alert).await.expect("Failed to send alert");

    // Clean up
    service.stop().await.expect("Failed to stop service");
}

#[tokio::test]
async fn test_metrics_stress() {
    let mut tasks: Vec<tokio::task::JoinHandle<Result<()>>> = Vec::new();
    
    for i in 0..10 {
        let (service, _, metric_collector, _, _) = create_test_service().await;
        
        let handle = tokio::spawn(async move {
            for j in 0..100 {
                // Use the metric collector directly from the service
                let metric = Metric::new(
                    format!("metric_{}_{}", i, j),
                    (i * j) as f64,
                    MetricType::Counter,
                    HashMap::new(),
                );
                
                metric_collector.record_metric(metric).await?;
            }
            Ok(())
        });
        
        tasks.push(handle);
    }
    
    let results = futures::future::join_all(tasks).await;
    
    assert!(results.iter().all(|r| r.as_ref().unwrap().is_ok()));
}

#[tokio::test]
async fn test_alerts_stress() {
    // Create multiple test services and add alerts to them in parallel
    let mut tasks: Vec<tokio::task::JoinHandle<Result<()>>> = Vec::new();
    
    for i in 0..5 {
        let (_, _, _, alert_manager, _) = create_test_service().await;
        
        let handle = tokio::spawn(async move {
            for j in 0..20 {
                let mut labels = HashMap::new();
                labels.insert("test".to_string(), format!("value_{}", j));
                
                let alert = Alert::new(
                    format!("Test Alert {}-{}", i, j),
                    format!("Description for test alert {}-{}", i, j),
                    AlertSeverity::Warning,
                    labels,
                    format!("Message for test alert {}-{}", i, j),
                    format!("test-component-{}", i),
                );
                
                // Use the alert manager directly
                alert_manager.add_alert(alert).await?;
            }
            Ok(())
        });
        
        tasks.push(handle);
    }
    
    let results = futures::future::join_all(tasks).await;
    
    assert!(results.iter().all(|r| r.as_ref().unwrap().is_ok()));
}

#[tokio::test]
async fn test_parallel_metrics() {
    // Create service
    let config = create_test_config();
    let service = MonitoringService::new(config);
    
    // Start the service
    service.start().await.unwrap();
    
    // Record metrics in parallel
    let mut tasks: Vec<tokio::task::JoinHandle<Result<()>>> = Vec::new();
    
    for i in 0..10 {
        let metric_collector = service.metric_collector.clone();
        tasks.push(tokio::spawn(async move {
            let metric = create_test_metric(&format!("metric_{}", i), i as f64);
            metric_collector.record_metric(metric).await?;
            Ok(())
        }));
    }
    
    // Wait for all tasks to complete
    let results = futures::future::join_all(tasks).await;
    assert!(results.iter().all(|r| r.as_ref().unwrap().is_ok()));
    
    // Check if all metrics were recorded
    let metrics = service.get_metrics().await.unwrap();
    assert_eq!(metrics.len(), 10);
    
    // Cleanup
    service.stop().await.unwrap();
}

#[tokio::test]
async fn test_parallel_alerts() {
    // Create service
    let config = create_test_config();
    let service = MonitoringService::new(config);
    
    // Start the service
    service.start().await.unwrap();
    
    // Send alerts in parallel
    let mut tasks: Vec<tokio::task::JoinHandle<Result<()>>> = Vec::new();
    
    for i in 0..10 {
        let alert_manager = service.alert_manager.clone();
        tasks.push(tokio::spawn(async move {
            let alert = Alert::new(
                format!("Alert {}", i),
                format!("Description {}", i),
                AlertSeverity::Warning,
                HashMap::new(),
                format!("Message {}", i),
                format!("Component {}", i),
            );
            alert_manager.add_alert(alert).await?;
            Ok(())
        }));
    }
    
    // Wait for all tasks to complete
    let results = futures::future::join_all(tasks).await;
    assert!(results.iter().all(|r| r.as_ref().unwrap().is_ok()));
    
    // Check if all alerts were recorded
    let alerts = service.get_alerts().await.unwrap();
    assert_eq!(alerts.len(), 10);
    
    // Cleanup
    service.stop().await.unwrap();
}

#[tokio::test]
async fn test_component_registration_and_health_check() {
    // Create health checker with proper initialization
    let health_config = HealthConfig::default();
    let mut health_checker = HealthCheckerAdapter::new();
    health_checker.initialize_with_config(health_config).expect("Failed to initialize health checker");
    
    // Register a component
    let component = ComponentHealth::new(
        "test-component".to_string(),
        Status::Healthy,
        "All systems operational".to_string(),
    );
    
    health_checker.register_component(component).await
        .expect("Failed to register component");
    
    // Check health
    let health = health_checker.check_health().await
        .expect("Failed to get health");
    assert_eq!(health.status, Status::Healthy);
    
    // Check component directly
    let component = health_checker.get_component_health("test-component").await.unwrap();
    assert!(component.is_some(), "Component should be registered");
    assert_eq!(component.unwrap().status, Status::Healthy);
    
    // Register an unhealthy component
    let unhealthy_component = ComponentHealth::new(
        "unhealthy-component".to_string(),
        Status::Unhealthy,
        "Service unavailable".to_string(),
    );
    
    health_checker.register_component(unhealthy_component).await
        .expect("Failed to register unhealthy component");
    
    // Health should now be unhealthy overall
    let health = health_checker.check_health().await
        .expect("Failed to get health");
    assert_eq!(health.status, Status::Unhealthy);
} 