//! Alert manager implementation for app monitoring.

use std::fmt::Debug;
use std::sync::RwLock;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::error::{Result, CoreError};
use squirrel_core::error::SquirrelError;
use squirrel_monitoring::alerts::Alert;

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
            .map_err(|e| SquirrelError::generic(format!("Failed to acquire write lock: {e}")))
            .map_err(|e| CoreError::Monitoring(e.to_string()))?;
        
        alerts.push(alert);
        Ok(())
    }
    
    /// Gets all alerts
    /// 
    /// # Errors
    /// 
    /// Returns an error if unable to access the alerts
    async fn get_alerts(&self) -> Result<Vec<Alert>> {
        let alerts = self.alerts.read()
            .map_err(|e| SquirrelError::generic(format!("Failed to acquire read lock: {e}")))
            .map_err(|e| CoreError::Monitoring(e.to_string()))?;
        
        Ok(alerts.clone())
    }
    
    /// Gets alerts within a time range
    /// 
    /// # Errors
    /// 
    /// Returns an error if unable to access the alerts
    async fn get_alerts_in_range(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> Result<Vec<Alert>> {
        let alerts = self.alerts.read()
            .map_err(|e| SquirrelError::generic(format!("Failed to acquire read lock: {e}")))
            .map_err(|e| CoreError::Monitoring(e.to_string()))?;
        
        // Filter alerts by timestamps within range
        Ok(alerts.iter()
            .filter(|a| {
                let timestamp = DateTime::<Utc>::from_timestamp(a.created_at, 0).unwrap_or_default();
                timestamp >= from && timestamp <= to
            })
            .cloned()
            .collect())
    }
    
    async fn start(&self) -> Result<()> {
        // Nothing to do for now
        Ok(())
    }
    
    async fn stop(&self) -> Result<()> {
        // Nothing to do for now
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
    use std::collections::HashMap;
    use squirrel_monitoring::AlertSeverity;
    use crate::monitoring::AlertManagerTrait;
    
    #[tokio::test]
    async fn test_alert_manager() {
        let alert_manager = AlertManagerImpl::new();
        
        // Start the alert manager
        alert_manager.start().await.unwrap();
        
        // Create and send an alert
        let mut labels = HashMap::new();
        labels.insert("source".to_string(), "test".to_string());
        
        let alert = Alert::new(
            "high_cpu".to_string(),
            "CPU usage exceeded threshold".to_string(),
            AlertSeverity::Critical,
            labels,
            "CPU usage is high".to_string(),
            "monitoring".to_string(),
        );
        
        alert_manager.send_alert(alert).await.unwrap();
        
        // Get alerts
        let alerts = alert_manager.get_alerts().await.unwrap();
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].name, "high_cpu");
        assert_eq!(alerts[0].message, "CPU usage is high");
        assert_eq!(alerts[0].component, "monitoring");
        
        // Stop the alert manager
        alert_manager.stop().await.unwrap();
    }
} 