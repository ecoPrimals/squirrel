//! Alert handler plugin implementation

use crate::plugins::common::{MonitoringPlugin, PluginMetadata};
use crate::alerts::{Alert, AlertLevel};
use crate::alerts::status::AlertStatusType;
use squirrel_core::error::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use tracing::{info, warn, error};
use serde_json::Value;
use std::fmt::Debug;
use anyhow::anyhow;

/// Alert Handler Plugin for managing and responding to system alerts
#[derive(Debug)]
pub struct AlertHandlerPlugin {
    /// Plugin metadata
    pub metadata: PluginMetadata,
    /// Active alerts
    alerts: Arc<RwLock<Vec<Alert>>>,
    /// Alert handlers registry
    handlers: Arc<RwLock<HashMap<String, Arc<dyn AlertHandler>>>>,
}

/// Alert handler trait for processing alerts
#[async_trait]
pub trait AlertHandler: Send + Sync + Debug {
    /// Get handler name
    fn name(&self) -> &str;
    
    /// Get supported alert types
    fn supported_types(&self) -> &[&str];
    
    /// Handle an alert
    async fn handle_alert(&self, alert: &Alert) -> anyhow::Result<()>;
}

impl AlertHandlerPlugin {
    /// Create a new alert handler plugin
    #[must_use]
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata::new(
                "Alert Handler Plugin",
                "1.0.0",
                "Handles and manages system alerts and notifications",
                "DataScienceBioLab",
            )
            .with_capability("alerts.handling")
            .with_capability("alerts.management")
            .with_capability("alerts.notification"),
            alerts: Arc::new(RwLock::new(Vec::new())),
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register an alert handler with the plugin
    pub async fn register_handler(&self, handler: Box<dyn AlertHandler>) {
        let name = handler.name().to_string();
        let mut handlers = self.handlers.write().await;
        handlers.insert(name.clone(), Arc::from(handler));
        info!("Registered alert handler: {}", name);
    }
    
    /// Add an alert to the system
    pub async fn add_alert(&self, alert: Alert) -> Result<()> {
        let alert_type = alert.alert_type.clone();
        let level = alert.level;
        let mut alerts = self.alerts.write().await;
        
        // Check if this is a duplicate of an existing alert
        for existing in &mut *alerts {
            if existing.source == alert.source && existing.alert_type == alert.alert_type {
                // Update existing alert instead of adding a new one
                existing.count += 1;
                existing.last_occurred = alert.occurred;
                existing.status = AlertStatusType::Active;
                
                if level > existing.level {
                    existing.level = level;
                    existing.message = alert.message.clone();
                }
                
                info!("Updated existing alert from {}: {} (count: {})", 
                      alert.source, alert.message, existing.count);
                return Ok(());
            }
        }
        
        // Log based on severity
        match level {
            AlertLevel::Critical => error!("Critical alert from {}: {}", alert.source, alert.message),
            AlertLevel::Warning => warn!("Warning alert from {}: {}", alert.source, alert.message),
            AlertLevel::Info => info!("Info alert from {}: {}", alert.source, alert.message),
            _ => info!("Alert from {}: {}", alert.source, alert.message),
        }
        
        // Add the new alert
        alerts.push(alert);
        
        // Try to find handlers for this alert type
        let _ = self.dispatch_alert_to_handlers(&alert_type).await;
        
        Ok(())
    }
    
    /// Get all active alerts
    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        let alerts = self.alerts.read().await;
        alerts.iter()
            .filter(|a| a.status == AlertStatusType::Active)
            .cloned()
            .collect()
    }
    
    /// Create a new alert
    pub fn create_alert(
        alert_type: &str,
        source: &str,
        message: &str,
        level: AlertLevel,
        details: HashMap<String, String>,
    ) -> Alert {
        Alert::new(
            alert_type.to_string(),
            source.to_string(),
            message.to_string(),
            level,
            details,
        )
    }
    
    /// Handle an alert from JSON data.
    /// 
    /// Returns a Result containing the added Alert on success
    pub async fn handle_alert_json(&self, json_data: Value) -> anyhow::Result<Alert> {
        let alert = match serde_json::from_value::<Alert>(json_data.clone()) {
            Ok(alert) => alert,
            Err(e) => return Err(anyhow!("Failed to parse alert from JSON: {}", e)),
        };
        
        self.add_alert(alert.clone()).await?;
        Ok(alert)
    }
    
    /// Dispatch an alert to appropriate handlers
    async fn dispatch_alert_to_handlers(&self, alert_type: &str) -> Result<()> {
        let handlers = self.handlers.read().await;
        let alerts = self.alerts.read().await;
        
        // Find the most recent alert of this type
        if let Some(alert) = alerts.iter()
            .filter(|a| a.alert_type == alert_type)
            .max_by_key(|a| a.last_occurred) {
            
            // Find handlers that support this alert type
            for handler in handlers.values() {
                if handler.supported_types().contains(&alert_type) {
                    if let Err(e) = handler.handle_alert(alert).await {
                        error!("Handler {} failed to process alert: {}", handler.name(), e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Resolve an alert by ID
    pub async fn resolve_alert(&self, id: &str) -> anyhow::Result<()> {
        let mut alerts = self.alerts.write().await;
        
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == id) {
            alert.status = AlertStatusType::Resolved;
            info!("Alert resolved: {}", id);
            Ok(())
        } else {
            Err(anyhow!("Alert not found: {}", id))
        }
    }
    
    /// Collect statistics about alerts
    pub async fn collect_alert_metrics(&self) -> anyhow::Result<Value> {
        let alerts = self.alerts.read().await;
        
        let total_alerts = alerts.len();
        let active_alerts = alerts.iter()
            .filter(|a| a.status == AlertStatusType::Active)
            .count();
        let critical_alerts = alerts.iter()
            .filter(|a| a.level == AlertLevel::Critical && a.status == AlertStatusType::Active)
            .count();
        
        // Create a summary of alert types
        let mut alert_types = HashMap::new();
        for alert in alerts.iter() {
            let count = alert_types.entry(alert.alert_type.clone()).or_insert(0);
            *count += 1;
        }
        
        Ok(serde_json::json!({
            "total_alerts": total_alerts,
            "active_alerts": active_alerts,
            "critical_alerts": critical_alerts,
            "alert_types": alert_types,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }))
    }
}

#[async_trait]
impl MonitoringPlugin for AlertHandlerPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn initialize(&self) -> anyhow::Result<()> {
        info!("Initializing Alert Handler Plugin");
        Ok(())
    }

    async fn shutdown(&self) -> anyhow::Result<()> {
        info!("Shutting down Alert Handler Plugin");
        Ok(())
    }
    
    async fn collect_metrics(&self) -> anyhow::Result<Value> {
        let alerts = self.alerts.read().await;
        
        // Convert alerts to a summary format
        let alert_count = alerts.len();
        let active_count = alerts.iter().filter(|a| a.status == AlertStatusType::Active).count();
        let critical_count = alerts.iter()
            .filter(|a| a.level == AlertLevel::Critical && a.status == AlertStatusType::Active)
            .count();
        
        Ok(serde_json::json!({
            "total": alert_count,
            "active": active_count,
            "critical": critical_count,
        }))
    }
    
    fn get_monitoring_targets(&self) -> Vec<String> {
        vec!["alerts".to_string()]
    }
    
    async fn handle_alert(&self, alert: Value) -> anyhow::Result<()> {
        self.handle_alert_json(alert).await?;
        Ok(())
    }
} 