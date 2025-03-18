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
fn create_test_service() -> (MonitoringService, MockAlertManager, MockMetricCollector, MockNetworkMonitor) {
    let mock_alert_manager = MockAlertManager::new();
    let mock_metric_collector = MockMetricCollector::new();
    let mock_network_monitor = MockNetworkMonitor::new();
    
    let config = create_test_config();
    let health_checker = create_checker_adapter();
    
    let service = MonitoringService::with_dependencies(
        config,
        health_checker,
        Arc::new(mock_metric_collector.clone()),
        Arc::new(mock_alert_manager.clone()),
        Arc::new(mock_network_monitor.clone()),
    );
    
    (service, mock_alert_manager, mock_metric_collector, mock_network_monitor)
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[tokio::test]
    async fn test_service_initialization() {
        let (service, _, _, _) = create_test_service();
        assert!(service.start().await.is_ok());
        assert!(service.stop().await.is_ok());
    }

    #[tokio::test]
    async fn test_alert_manager_integration() {
        let (service, mut mock_alert, _, _) = create_test_service();
        
        mock_alert
            .expect_send_alert()
            .with(predicate::function(|alert: &Alert| alert.name == "test_alert"))
            .times(1)
            .returning(|_| Ok(()));
            
        let alert = Alert::new(
            "test_alert".to_string(),
            "Test alert".to_string(),
            AlertSeverity::Warning,
            HashMap::new(),
            "Test message".to_string(),
            "test_component".to_string(),
        );
        
        assert!(service.send_alert(alert).await.is_ok());
    }

    #[tokio::test]
    async fn test_metric_collection() {
        let (service, _, mut mock_metrics, _) = create_test_service();
        
        mock_metrics
            .expect_record_metric()
            .with(eq("test_metric"), eq(42.0))
            .times(1)
            .returning(|_, _| Ok(()));
            
        assert!(service.record_metric("test_metric", 42.0).await.is_ok());
    }

    #[tokio::test]
    async fn test_network_monitoring() {
        let (service, _, _, mut mock_network) = create_test_service();
        
        let mut stats = HashMap::new();
        stats.insert(
            "eth0".to_string(),
            NetworkStats {
                interface: "eth0".to_string(),
                received_bytes: 1000,
                transmitted_bytes: 2000,
                receive_rate: 100.0,
                transmit_rate: 200.0,
                packets_received: 10,
                packets_transmitted: 20,
                errors_on_received: 0,
                errors_on_transmitted: 0,
            },
        );
        
        mock_network
            .expect_get_stats()
            .times(1)
            .returning(move || Ok(stats.clone()));
            
        let result = service.get_network_stats().await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result["eth0"].received_bytes, 1000);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_monitoring_flow() {
        // Create a real service with real components
        let config = create_test_config();
        let service = MonitoringService::new(config);
        
        // Start the service
        service.start().await.unwrap();
        
        // Record some metrics
        service.record_metric("test_metric", 42.0).await.unwrap();
        
        // Send an alert
        let alert = Alert::new(
            "test_alert".to_string(),
            "Test alert".to_string(),
            AlertSeverity::Warning,
            HashMap::new(),
            "Test message".to_string(),
            "test_component".to_string(),
        );
        service.send_alert(alert).await.unwrap();
        
        // Get network stats
        let network_stats = service.get_network_stats().await.unwrap();
        assert!(!network_stats.is_empty());
        
        // Stop the service
        service.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_component_interaction() {
        let config = create_test_config();
        let service = MonitoringService::new(config);
        
        service.start().await.unwrap();
        
        // Register a component for health checking
        service.register_component("test_component", HealthStatus::healthy()).await.unwrap();
        
        // Record metrics for the component
        service.record_metric("test_component.latency", 10.0).await.unwrap();
        
        // Verify health status
        let health = service.get_health().await.unwrap();
        assert!(health.components.contains_key("test_component"));
        
        // Verify metrics
        let metrics = service.get_metrics().await.unwrap();
        assert!(metrics.contains_key("test_component.latency"));
        
        service.stop().await.unwrap();
    }
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