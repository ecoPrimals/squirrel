//! Tests for the monitoring plugins

#[cfg(test)]
mod tests {
    use super::super::*;
    use serde_json::Value;
    use std::sync::Arc;
    use anyhow::Result;
    use tokio::sync::Mutex;
    use std::collections::HashMap;
    use async_trait::async_trait;
    use chrono::Utc;
    use crate::health::{ComponentHealth, status::Status};
    use crate::health::component::HealthCheck;
    
    /// Simple in-memory plugin registry for testing
    struct InMemoryPluginRegistry {
        plugins: Mutex<Vec<Arc<dyn MonitoringPlugin>>>,
    }
    
    impl InMemoryPluginRegistry {
        fn new() -> Self {
            Self {
                plugins: Mutex::new(Vec::new()),
            }
        }
        
        async fn get_plugins(&self) -> Result<Vec<Arc<dyn MonitoringPlugin>>> {
            let plugins = self.plugins.lock().await;
            Ok(plugins.clone())
        }
    }
    
    #[async_trait::async_trait]
    impl MonitoringPluginRegistry for InMemoryPluginRegistry {
        async fn register_monitoring_plugin<T>(&self, plugin: Arc<T>) -> anyhow::Result<()>
        where
            T: MonitoringPlugin + Send + Sync + 'static
        {
            let mut plugins = self.plugins.lock().await;
            plugins.push(plugin);
            Ok(())
        }
    }
    
    #[tokio::test]
    async fn test_system_metrics_plugin() -> Result<()> {
        // Create the plugin
        let plugin = SystemMetricsPlugin::new();
        
        // Initialize
        plugin.initialize().await?;
        
        // Verify metadata
        assert_eq!(plugin.metadata().name, "System Metrics Plugin");
        
        // Collect metrics
        let metrics = plugin.collect_metrics().await?;
        assert!(metrics.is_object());
        
        // Verify monitoring targets
        let targets = plugin.get_monitoring_targets();
        assert!(targets.contains(&"system".to_string()));
        assert!(targets.contains(&"cpu".to_string()));
        
        // Shutdown
        plugin.shutdown().await?;
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_health_reporter_plugin() -> Result<()> {
        // Create the plugin
        let plugin = HealthReporterPlugin::new();
        
        // Initialize
        plugin.initialize().await?;
        
        // Verify metadata
        assert_eq!(plugin.metadata().name, "Health Reporter Plugin");
        
        // Collect metrics (should be empty at first)
        let metrics = plugin.collect_metrics().await?;
        assert!(metrics.is_array());
        assert_eq!(metrics.as_array().unwrap().len(), 0);
        
        // Register a test health check
        plugin.register_health_check(Box::new(TestHealthCheck::new("test-component"))).await;
        
        // Collect metrics again (should have one health check)
        let metrics = plugin.collect_metrics().await?;
        assert!(metrics.is_array());
        assert_eq!(metrics.as_array().unwrap().len(), 1);
        
        // Verify monitoring targets
        let targets = plugin.get_monitoring_targets();
        assert!(targets.contains(&"health".to_string()));
        
        // Shutdown
        plugin.shutdown().await?;
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_alert_handler_plugin() -> Result<()> {
        // Create the plugin
        let plugin = AlertHandlerPlugin::new();
        
        // Initialize
        plugin.initialize().await?;
        
        // Verify metadata
        assert_eq!(plugin.metadata().name, "Alert Handler Plugin");
        
        // Collect metrics (should be empty at first)
        let metrics = plugin.collect_metrics().await?;
        assert!(metrics.is_object());
        
        // Register a test alert handler
        plugin.register_handler(Box::new(TestAlertHandler::new())).await;
        
        // Create a test alert
        let alert = AlertHandlerPlugin::create_alert(
            "test-alert",
            "test-source",
            "Test alert message",
            crate::alerts::AlertLevel::Warning,
            HashMap::new(),
        );
        
        // Handle the alert
        let alert_json = serde_json::to_value(alert)?;
        plugin.handle_alert(alert_json).await?;
        
        // Verify active alerts
        let active_alerts = plugin.get_active_alerts().await;
        assert_eq!(active_alerts.len(), 1);
        
        // Verify monitoring targets
        let targets = plugin.get_monitoring_targets();
        assert!(targets.contains(&"alerts".to_string()));
        
        // Shutdown
        plugin.shutdown().await?;
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_plugin_registration() -> Result<()> {
        // Create registry
        let registry = Arc::new(InMemoryPluginRegistry::new());
        
        // Register plugins
        register_plugins(registry.clone()).await?;
        
        // Verify plugins are registered
        let plugins = registry.get_plugins().await?;
        assert_eq!(plugins.len(), 3);
        
        // Verify plugin types
        let mut found_system_metrics = false;
        let mut found_health_reporter = false;
        let mut found_alert_handler = false;
        
        for plugin in plugins {
            match plugin.metadata().name.as_str() {
                "System Metrics Plugin" => found_system_metrics = true,
                "Health Reporter Plugin" => found_health_reporter = true,
                "Alert Handler Plugin" => found_alert_handler = true,
                _ => (),
            }
        }
        
        assert!(found_system_metrics);
        assert!(found_health_reporter);
        assert!(found_alert_handler);
        
        Ok(())
    }
    
    /// Test health check implementation
    #[derive(Debug)]
    struct TestHealthCheck {
        name: String,
        status: Arc<Mutex<Status>>,
    }
    
    impl TestHealthCheck {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                status: Arc::new(Mutex::new(Status::Healthy)),
            }
        }
    }
    
    #[async_trait::async_trait]
    impl HealthCheck for TestHealthCheck {
        fn name(&self) -> &str {
            &self.name
        }
        
        async fn check(&self) -> squirrel_core::error::Result<ComponentHealth> {
            let status = *self.status.lock().await;
            Ok(ComponentHealth {
                name: self.name.clone(),
                status,
                message: Some("Test health check".to_string()),
                last_check: Utc::now(),
                details: std::collections::HashMap::new(),
            })
        }
    }
    
    /// Test alert handler implementation
    #[derive(Debug)]
    struct TestAlertHandler;
    
    impl TestAlertHandler {
        fn new() -> Self {
            Self
        }
    }
    
    #[async_trait::async_trait]
    impl crate::plugins::alert_handler::AlertHandler for TestAlertHandler {
        fn name(&self) -> &str {
            "test-handler"
        }
        
        fn supported_types(&self) -> &[&str] {
            &["test-alert"]
        }
        
        async fn handle_alert(&self, alert: &crate::alerts::Alert) -> Result<()> {
            tracing::info!("Test handler processing alert: {}", alert.message);
            Ok(())
        }
    }
} 