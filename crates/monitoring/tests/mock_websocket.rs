use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use serde_json::Value;
use squirrel_core::error::{Result, SquirrelError};
use squirrel_monitoring::websocket::{WebSocketConfig, WebSocketInterface};

/// A mock implementation of WebSocketInterface for testing
#[derive(Debug, Clone)]
pub struct MockWebSocketServer {
    config: WebSocketConfig,
    component_data: Arc<RwLock<HashMap<String, Value>>>,
    running: Arc<RwLock<bool>>,
    connection_count: Arc<RwLock<usize>>,
}

impl MockWebSocketServer {
    pub fn new(config: WebSocketConfig) -> Self {
        Self {
            config,
            component_data: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
            connection_count: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn add_connection(&self) {
        let mut count = self.connection_count.write().await;
        *count += 1;
    }

    pub async fn remove_connection(&self) {
        let mut count = self.connection_count.write().await;
        if *count > 0 {
            *count -= 1;
        }
    }
}

#[async_trait]
impl WebSocketInterface for MockWebSocketServer {
    async fn get_available_components(&self) -> Result<Vec<String>> {
        let data = self.component_data.read().await;
        Ok(data.keys().cloned().collect())
    }
    
    async fn get_component_data(&self, component_id: &str) -> Result<Value> {
        let data = self.component_data.read().await;
        match data.get(component_id) {
            Some(value) => Ok(value.clone()),
            None => Err(SquirrelError::Generic(format!("Component data not found for: {}", component_id)))
        }
    }
    
    async fn get_health_status(&self) -> Result<Value> {
        let is_running = *self.running.read().await;
        let connection_count = *self.connection_count.read().await;
        
        Ok(serde_json::json!({
            "running": is_running,
            "connection_count": connection_count,
            "uptime": "not implemented",
            "status": if is_running { "healthy" } else { "stopped" }
        }))
    }
    
    async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Err(SquirrelError::Generic("WebSocket server already running".to_string()));
        }
        *running = true;
        Ok(())
    }
    
    async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        Ok(())
    }
    
    async fn update_component_data(&self, component_id: &str, data: Value) -> Result<()> {
        let mut component_data = self.component_data.write().await;
        component_data.insert(component_id.to_string(), data);
        Ok(())
    }
} 