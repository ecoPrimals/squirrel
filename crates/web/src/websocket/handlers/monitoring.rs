use std::sync::Arc;
use serde_json::json;

use squirrel_monitoring::api::MonitoringAPI;

/// Monitoring WebSocket Handler
/// 
/// This handler provides access to monitoring data through the WebSocket API.
/// It connects to the monitoring crate's API and exposes the data to clients.
/// 
/// Topics:
/// - components: List of available components
/// - component:{name}: Data for a specific component
/// - health: System health status
#[derive(Debug)]
pub struct MonitoringHandler {
    monitoring_api: Arc<dyn MonitoringAPI>,
}

impl MonitoringHandler {
    /// Create a new monitoring handler
    pub fn new(monitoring_api: Arc<dyn MonitoringAPI>) -> Self {
        Self {
            monitoring_api,
        }
    }
} 