use std::fmt::Debug;
use std::sync::RwLock;
use async_trait::async_trait;

use crate::error::{Result, SquirrelError};

/// Alert information
#[derive(Debug, Clone)]
pub struct Alert {
    /// Alert name
    pub name: String,
    /// Alert message
    pub message: String,
    /// Alert severity
    pub severity: String,
    /// Timestamp of the alert
    pub timestamp: u64,
}

/// Alert configuration
#[derive(Debug, Clone)]
pub struct AlertConfig {
    /// Threshold for critical alerts
    pub critical_threshold: f64,
    /// Threshold for warning alerts
    pub warning_threshold: f64,
    /// Whether to enable notifications
    pub enable_notifications: bool,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            critical_threshold: 90.0,
            warning_threshold: 70.0,
            enable_notifications: true,
        }
    }
}

/// Alert manager implementation
#[derive(Debug)]
pub struct AlertManagerImpl {
    /// Active alerts 
    alerts: RwLock<Vec<Alert>>,
    /// Alert configuration
    #[allow(dead_code)]
    config: AlertConfig,
}

impl AlertManagerImpl {
    /// Create a new `AlertManagerImpl`
    #[must_use]
    pub fn new() -> Self {
        Self {
            alerts: RwLock::new(Vec::new()),
            config: AlertConfig::default(),
        }
    }
}

#[async_trait]
impl super::AlertManagerTrait for AlertManagerImpl {
    async fn send_alert(&self, alert: Alert) -> Result<()> {
        let mut alerts = self.alerts.write()
            .map_err(|e| SquirrelError::Monitoring(format!("Failed to acquire write lock: {e}")))?;
        alerts.push(alert);
        Ok(())
    }
    
    async fn get_alerts(&self) -> Result<Vec<Alert>> {
        let alerts = self.alerts.read()
            .map_err(|e| SquirrelError::Monitoring(format!("Failed to acquire read lock: {e}")))?;
        Ok(alerts.clone())
    }
    
    async fn start(&self) -> Result<()> {
        // In a real implementation, this might start a background task
        // that monitors metrics and generates alerts
        Ok(())
    }
    
    async fn stop(&self) -> Result<()> {
        // In a real implementation, this would stop the background task
        Ok(())
    }
}

impl Default for AlertManagerImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::monitoring::AlertManagerTrait;
    
    #[tokio::test]
    async fn test_alert_manager() {
        let alert_manager = AlertManagerImpl::new();
        
        // Start the alert manager
        alert_manager.start().await.unwrap();
        
        // Create and send an alert
        let alert = Alert {
            name: "high_cpu".to_string(),
            message: "CPU usage is high".to_string(),
            severity: "critical".to_string(),
            timestamp: 12345,
        };
        
        alert_manager.send_alert(alert).await.unwrap();
        
        // Get alerts
        let alerts = alert_manager.get_alerts().await.unwrap();
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].name, "high_cpu");
        
        // Stop the alert manager
        alert_manager.stop().await.unwrap();
    }
} 