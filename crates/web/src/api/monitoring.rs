use std::sync::Arc;
use serde_json::json;
use tracing::error;
use axum::{
    Router,
    routing::get,
};

/// Monitoring API service
/// 
/// This struct provides a REST API for the monitoring system.
#[derive(Debug, Clone)]
pub struct MonitoringService {
    monitoring_api: Arc<dyn squirrel_monitoring::api::MonitoringAPI>,
}

impl MonitoringService {
    /// Create a new monitoring service
    pub fn new(monitoring_api: Arc<dyn squirrel_monitoring::api::MonitoringAPI>) -> Self {
        Self {
            monitoring_api,
        }
    }
    
    /// Build the monitoring API routes
    pub fn routes(&self) -> Router {
        Router::new()
    }
    
    /// Get a list of all available components
    pub async fn get_components(&self) -> Result<Vec<String>, String> {
        match self.monitoring_api.get_available_components().await {
            Ok(components) => Ok(components),
            Err(e) => {
                error!("Error fetching components: {:?}", e);
                Err(format!("Failed to fetch components: {}", e))
            }
        }
    }
    
    /// Get data for a specific component
    pub async fn get_component_data(&self, component_id: &str) -> Result<serde_json::Value, String> {
        match self.monitoring_api.get_component_data(component_id).await {
            Ok(data) => Ok(data),
            Err(e) => {
                error!("Error fetching component data: {:?}", e);
                Err(format!("Failed to fetch component data: {}", e))
            }
        }
    }
    
    /// Get the current health status of the monitoring system
    pub async fn get_health_status(&self) -> Result<serde_json::Value, String> {
        match self.monitoring_api.get_health_status().await {
            Ok(status) => Ok(json!(status)),
            Err(e) => {
                error!("Error fetching health status: {:?}", e);
                Err(format!("Failed to fetch health status: {}", e))
            }
        }
    }
} 