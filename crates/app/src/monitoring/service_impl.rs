use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Mutex;

use crate::app::monitoring::{
    HealthStatus, Metric, MonitoringConfig
};
use crate::app::monitoring::alert::{Alert, AlertManagerImpl};
use crate::error::{Result, SquirrelError};

/// System status information
#[derive(Debug, Default)]
#[allow(clippy::struct_field_names)]
struct SystemStatus {
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Disk usage in bytes
    pub disk_usage: u64,
    /// Network usage in bytes
    pub network_usage: u64,
}

/// `MonitoringService` implementation
#[derive(Debug)]
pub struct MonitoringServiceImpl {
    /// Configuration
    #[allow(dead_code)]
    config: MonitoringConfig,
    /// System status
    status: std::sync::Arc<Mutex<SystemStatus>>,
    /// Health status
    health_status: std::sync::RwLock<HealthStatus>,
    /// Started flag
    started: Mutex<bool>,
    /// Stopped flag
    stopped: Mutex<bool>,
    /// Alert manager
    alert_manager: Box<dyn crate::app::monitoring::AlertManagerTrait + Send + Sync>,
}

impl MonitoringServiceImpl {
    /// Create a new `MonitoringServiceImpl`
    #[must_use]
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            config,
            status: std::sync::Arc::new(Mutex::new(SystemStatus::default())),
            health_status: std::sync::RwLock::new(HealthStatus::default()),
            started: Mutex::new(false),
            stopped: Mutex::new(false),
            alert_manager: Box::new(AlertManagerImpl::new()),
        }
    }
    
    /// Checks whether the monitoring service is initialized (started)
    /// 
    /// # Returns
    /// 
    /// Returns `true` if the service has been started, `false` otherwise
    #[must_use]
    pub fn is_initialized(&self) -> bool {
        match self.started.lock() {
            Ok(started) => *started,
            Err(_) => false, // If we can't acquire the lock, assume we're not started
        }
    }
}

#[async_trait]
impl crate::app::monitoring::MonitoringServiceTrait for MonitoringServiceImpl {
    async fn start(&self) -> Result<()> {
        let mut started = self.started.lock()
            .map_err(|e| SquirrelError::Monitoring(format!("Failed to acquire started lock: {e}")))?;
        *started = true;
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        let mut stopped = self.stopped.lock()
            .map_err(|e| SquirrelError::Monitoring(format!("Failed to acquire stopped lock: {e}")))?;
        *stopped = true;
        Ok(())
    }

    async fn get_health(&self) -> Result<HealthStatus> {
        let health = self.health_status.read()
            .map_err(|e| SquirrelError::Monitoring(format!("Failed to acquire health_status read lock: {e}")))?;
        Ok(health.clone())
    }

    async fn get_system_status(&self) -> Result<HashMap<String, String>> {
        let status = self.status.lock()
            .map_err(|e| SquirrelError::Monitoring(format!("Failed to acquire status lock: {e}")))?;
        let mut result = HashMap::new();
        
        result.insert("cpu_usage".to_string(), format!("{:.2}", status.cpu_usage));
        result.insert("memory_usage".to_string(), format!("{}", status.memory_usage));
        result.insert("disk_usage".to_string(), format!("{}", status.disk_usage));
        result.insert("network_usage".to_string(), format!("{}", status.network_usage));
        
        Ok(result)
    }

    async fn get_metrics(&self) -> Result<Vec<HashMap<String, Metric>>> {
        // Create some dummy metrics for now
        let mut metrics = Vec::new();
        
        // CPU metrics
        let mut cpu_metrics = HashMap::new();
        cpu_metrics.insert("cpu_usage".to_string(), 0.5);
        cpu_metrics.insert("cpu_temperature".to_string(), 45.0);
        metrics.push(cpu_metrics);
        
        // Memory metrics
        let mut mem_metrics = HashMap::new();
        mem_metrics.insert("memory_usage".to_string(), 1024.0 * 1024.0 * 500.0); // 500 MB
        mem_metrics.insert("memory_available".to_string(), 1024.0 * 1024.0 * 1500.0); // 1.5 GB
        metrics.push(mem_metrics);
        
        Ok(metrics)
    }

    async fn get_alerts(&self) -> Result<Vec<Alert>> {
        self.alert_manager.get_alerts().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::monitoring::HealthLevel;
    use crate::app::monitoring::MonitoringServiceTrait;
    
    #[tokio::test]
    async fn test_monitoring_service() {
        let config = MonitoringConfig::default();
        let service = MonitoringServiceImpl::new(config);
        
        // Test if service is initialized before starting
        assert!(!service.is_initialized());
        
        // Test starting the service
        service.start().await.unwrap();
        
        // Test if service is initialized after starting
        assert!(service.is_initialized());
        
        // Test getting health
        let health = service.get_health().await.unwrap();
        assert_eq!(health.level, HealthLevel::Healthy);
        
        // Test getting system status
        let status = service.get_system_status().await.unwrap();
        assert!(status.contains_key("cpu_usage"));
        
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