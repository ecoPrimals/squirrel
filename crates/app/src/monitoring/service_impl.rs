//! Monitoring service implementation
//!
//! Implements the monitoring service functionality
use std::collections::HashMap;
use std::fmt;
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::error::{CoreError, Result};
use super::{
    Alert, AppHealthStatus, Metric, MonitoringConfigType, MonitoringServiceTrait,
    MetricCollectorTrait
};
use squirrel_core::error::SquirrelError;
use squirrel_monitoring::metrics::performance::PerformanceCollector;
use squirrel_monitoring::health::HealthChecker;
use squirrel_monitoring::alerts::AlertManager;
use squirrel_monitoring::alerts::manager::AlertManager as SquirrelAlertManager;

/// System status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage percentage
    pub memory_usage: f64,
    /// Disk usage percentage
    pub disk_usage: f64,
    /// Network usage (bytes/sec)
    pub network_usage: f64,
}

impl Default for SystemStatus {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            disk_usage: 0.0,
            network_usage: 0.0,
        }
    }
}

/// `MonitoringService` implementation
#[derive(Debug)]
pub struct MonitoringServiceImpl {
    /// Configuration
    config: MonitoringConfigType,
    /// System status
    status: std::sync::Arc<Mutex<SystemStatus>>,
    /// Health status
    health_status: std::sync::RwLock<AppHealthStatus>,
    /// Started flag
    started: Mutex<bool>,
    /// Stopped flag
    stopped: Mutex<bool>,
    /// Alert manager
    alert_manager: Box<dyn super::AlertManagerTrait>,
    /// Metric collector
    metric_collector: Box<dyn super::MetricCollectorTrait>,
}

impl MonitoringServiceImpl {
    /// Create a new `MonitoringServiceImpl`
    #[must_use]
    pub fn new(config: MonitoringConfigType) -> Self {
        Self {
            config,
            status: std::sync::Arc::new(Mutex::new(SystemStatus::default())),
            health_status: std::sync::RwLock::new(AppHealthStatus::default()),
            started: Mutex::new(false),
            stopped: Mutex::new(false),
            alert_manager: Box::new(super::alert::AlertManagerImpl::new()),
            metric_collector: Box::new(super::metrics::MetricCollectorImpl::new()),
        }
    }
    
    /// Checks whether the monitoring service is initialized (started)
    /// 
    /// # Returns
    /// 
    /// Returns `true` if the service has been started, `false` otherwise
    pub async fn is_initialized(&self) -> bool {
        match self.started.lock().await {
            started => *started
        }
    }
}

#[async_trait]
impl MonitoringServiceTrait for MonitoringServiceImpl {
    /// Starts the monitoring service
    /// 
    /// # Errors
    /// Returns an error if the service fails to start
    async fn start(&self) -> Result<()> {
        let mut service_started = self.started.lock()
            .await;
        
        // Check if already started
        if *service_started {
            return Ok(());
        }
        
        // Start the services
        self.alert_manager.start().await?;
        
        // Set started flag
        *service_started = true;
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        let mut stopped = self.stopped.lock()
            .await;
        
        // Check if already stopped
        if *stopped {
            return Ok(());
        }
        
        // Stop the services
        self.alert_manager.stop().await?;
        
        // Set stopped flag
        *stopped = true;
        Ok(())
    }

    /// Checks the health status of the monitoring service
    /// 
    /// # Errors
    /// Returns an error if unable to check health status
    async fn health_status(&self) -> Result<AppHealthStatus> {
        let status = self.health_status.read()
            .map_err(|e| SquirrelError::generic(format!("Failed to acquire lock: {e}")))
            .map_err(|e| CoreError::Monitoring(e.to_string()))?;
        
        Ok(status.clone())
    }

    /// Gets the health status
    /// 
    /// # Errors
    /// Returns an error if unable to check health status
    async fn get_health(&self) -> Result<AppHealthStatus> {
        // Simply delegate to health_status method for consistency
        self.health_status().await
    }

    /// Gets the current system status
    /// 
    /// # Errors
    /// Returns an error if unable to check system status
    async fn get_system_status(&self) -> Result<HashMap<String, String>> {
        let status = self.status.lock().await;
        
        let mut result = HashMap::new();
        result.insert("cpu_usage".to_string(), format!("{}", status.cpu_usage));
        result.insert("memory_usage".to_string(), format!("{}", status.memory_usage));
        result.insert("disk_usage".to_string(), format!("{}", status.disk_usage));
        result.insert("network_usage".to_string(), format!("{}", status.network_usage));
        Ok(result)
    }

    async fn get_metrics(&self) -> Result<Vec<HashMap<String, Metric>>> {
        // Collect metrics from the metric collector
        let metrics_map = self.metric_collector.collect().await?;
        
        // Convert to the expected format
        let mut metrics = Vec::new();
        metrics.push(metrics_map);
        
        Ok(metrics)
    }

    async fn get_alerts(&self) -> Result<Vec<Alert>> {
        self.alert_manager.get_alerts().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::HealthLevel;
    use crate::monitoring::MonitoringServiceTrait;
    
    #[tokio::test]
    async fn test_monitoring_service() {
        let config = MonitoringConfigType::default();
        let service = MonitoringServiceImpl::new(config);
        
        // Test if service is initialized before starting
        assert!(!service.is_initialized().await);
        
        // Test starting the service
        service.start().await.unwrap();
        
        // Test if service is initialized after starting
        assert!(service.is_initialized().await);
        
        // Test getting health
        let health = service.health_status().await.unwrap();
        assert_eq!(health.level, HealthLevel::Healthy);
        
        // Test getting system status
        let status = service.get_system_status().await.unwrap();
        assert!(status.contains_key("cpu_usage"));
        if let Some(cpu_usage) = status.get("cpu_usage") {
            assert!(!cpu_usage.is_empty(), "CPU usage should not be empty");
        } else {
            panic!("Expected 'cpu_usage' key in system status");
        }
        
        // Test getting metrics
        let metrics = service.get_metrics().await.unwrap();
        assert!(!metrics.is_empty());
        
        // Test getting alerts
        let alerts = service.get_alerts().await.unwrap();
        assert!(alerts.is_empty());
        
        // Test stopping the service
        service.stop().await.unwrap();
    }
} 