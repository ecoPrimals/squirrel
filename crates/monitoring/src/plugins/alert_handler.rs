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

impl Default for AlertHandlerPlugin {
    fn default() -> Self {
        Self::new()
    }
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
        let source = alert.source.clone();
        let message = alert.message.clone();
        let mut count = 0; // Initialize with default
        // We don't actually need alert_id for now
        
        // Scope the write lock to minimize lock time
        {
            let mut alerts = self.alerts.write().await;
            
            // Check if this is a duplicate of an existing alert
            let mut found_existing = false;
            for existing in &mut *alerts {
                if existing.source == alert.source && existing.alert_type == alert.alert_type {
                    // Update existing alert instead of adding a new one
                    existing.count += 1;
                    count = existing.count;
                    existing.last_occurred = alert.occurred;
                    existing.status = AlertStatusType::Active;
                    
                    if level > existing.level {
                        existing.level = level;
                        existing.message = alert.message.clone();
                    }
                    
                    found_existing = true;
                    break;
                }
            }
            
            // Add the new alert if no duplicate was found
            if !found_existing {
                count = 1; // First occurrence 
                alerts.push(alert);
            }
        }
        
        // Log based on severity - do this outside the lock
        match level {
            AlertLevel::Critical => error!("Critical alert from {}: {} (count: {})", source, message, count),
            AlertLevel::Warning => warn!("Warning alert from {}: {} (count: {})", source, message, count),
            AlertLevel::Info => info!("Info alert from {}: {} (count: {})", source, message, count),
            _ => info!("Alert from {}: {} (count: {})", source, message, count),
        }
        
        // Try to find handlers for this alert type - do this outside the lock
        if let Err(e) = self.dispatch_alert_to_handlers(&alert_type).await {
            error!("Failed to dispatch alert to handlers: {}", e);
        }
        
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
        // Clone the handlers to avoid holding locks while calling handlers
        let handlers_clone = {
            let handlers = self.handlers.read().await;
            handlers.values().cloned().collect::<Vec<_>>()
        };
        
        // Find the most recent alert of this type
        let alert = {
            let alerts = self.alerts.read().await;
            alerts.iter()
                .filter(|a| a.alert_type == alert_type)
                .max_by_key(|a| a.last_occurred)
                .cloned()
        };
            
        // Find handlers that support this alert type and call them without holding locks
        if let Some(alert) = alert {
            for handler in handlers_clone {
                if handler.supported_types().contains(&alert_type) {
                    if let Err(e) = handler.handle_alert(&alert).await {
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