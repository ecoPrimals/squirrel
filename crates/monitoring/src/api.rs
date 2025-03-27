use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;
use tokio::sync::RwLock;
use async_trait::async_trait;

use squirrel_core::error::Result;

/// Monitoring System API
/// 
/// This trait defines the interface for accessing monitoring data from other crates.
/// It provides a clean API for accessing component data, health status, and metrics
/// without exposing the internal details of the monitoring system.
#[async_trait]
pub trait MonitoringAPI: Send + Sync + std::fmt::Debug {
    /// Get data for a specific component
    async fn get_component_data(&self, component_id: &str) -> Result<Value>;
    
    /// Get a list of all available components
    async fn get_available_components(&self) -> Result<Vec<String>>;
    
    /// Get the current health status of the monitoring system
    async fn get_health_status(&self) -> Result<HashMap<String, Value>>;
    
    /// Subscribe to component updates
    /// 
    /// Returns a unique subscription ID that can be used to unsubscribe
    async fn subscribe_to_component(&self, component_id: &str) -> Result<String>;
    
    /// Unsubscribe from component updates
    async fn unsubscribe(&self, subscription_id: &str) -> Result<()>;
}

/// Monitoring System API Provider
/// 
/// This struct implements the MonitoringAPI trait and provides
/// access to the monitoring system data.
#[derive(Debug)]
pub struct MonitoringAPIProvider {
    component_data: Arc<RwLock<HashMap<String, Value>>>,
    subscriptions: Arc<RwLock<HashMap<String, String>>>,
}

impl MonitoringAPIProvider {
    /// Create a new MonitoringAPIProvider
    pub fn new() -> Self {
        Self {
            component_data: Arc::new(RwLock::new(HashMap::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Update component data
    /// 
    /// This method is called by the monitoring system when component data changes.
    /// It updates the internal data store and can be used to trigger notifications
    /// to subscribers in future implementations.
    pub async fn update_component_data(&self, component_id: &str, data: Value) -> Result<()> {
        let mut component_data = self.component_data.write().await;
        component_data.insert(component_id.to_string(), data);
        Ok(())
    }
}

#[async_trait]
impl MonitoringAPI for MonitoringAPIProvider {
    async fn get_component_data(&self, component_id: &str) -> Result<Value> {
        let component_data = self.component_data.read().await;
        match component_data.get(component_id) {
            Some(data) => Ok(data.clone()),
            None => Err(squirrel_core::error::SquirrelError::Generic(
                format!("Component data not found for: {}", component_id)
            )),
        }
    }
    
    async fn get_available_components(&self) -> Result<Vec<String>> {
        let component_data = self.component_data.read().await;
        Ok(component_data.keys().cloned().collect())
    }
    
    async fn get_health_status(&self) -> Result<HashMap<String, Value>> {
        // For now, just return a simple health status
        // In a real implementation, this would query the health monitoring system
        let mut health = HashMap::new();
        health.insert("status".to_string(), Value::String("healthy".to_string()));
        health.insert("components_count".to_string(), Value::Number(
            serde_json::Number::from(self.component_data.read().await.len())
        ));
        Ok(health)
    }
    
    async fn subscribe_to_component(&self, component_id: &str) -> Result<String> {
        let subscription_id = format!("sub_{}", uuid::Uuid::new_v4());
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(subscription_id.clone(), component_id.to_string());
        Ok(subscription_id)
    }
    
    async fn unsubscribe(&self, subscription_id: &str) -> Result<()> {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.remove(subscription_id);
        Ok(())
    }
}

// Export the API provider instance
lazy_static::lazy_static! {
    static ref MONITORING_API: MonitoringAPIProvider = MonitoringAPIProvider::new();
}

/// Get a reference to the global MonitoringAPI instance
pub fn get_monitoring_api() -> Result<&'static dyn MonitoringAPI> {
    Ok(&*MONITORING_API as &dyn MonitoringAPI)
}

/// Update component data in the global API instance
pub async fn update_component_data(component_id: &str, data: Value) -> Result<()> {
    MONITORING_API.update_component_data(component_id, data).await
} 