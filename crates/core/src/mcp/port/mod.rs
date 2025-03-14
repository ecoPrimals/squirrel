use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use crate::error::{MCPError, Result};
use crate::mcp::types::{ProtocolVersion, ProtocolState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortConfig {
    pub port: u16,
    pub host: String,
    pub max_connections: u32,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortState {
    pub is_listening: bool,
    pub active_connections: u32,
    pub total_connections: u64,
    pub last_connection: chrono::DateTime<chrono::Utc>,
}

pub struct MCPPort {
    config: Arc<RwLock<PortConfig>>,
    state: Arc<RwLock<PortState>>,
}

impl MCPPort {
    pub fn new(config: PortConfig) -> Self {
        let state = PortState {
            is_listening: false,
            active_connections: 0,
            total_connections: 0,
            last_connection: chrono::Utc::now(),
        };

        Self {
            config: Arc::new(RwLock::new(config)),
            state: Arc::new(RwLock::new(state)),
        }
    }

    pub async fn start(&self) -> Result<()> {
        // TODO: Implement actual port listening
        let mut state = self.state.write().await;
        state.is_listening = true;
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        // TODO: Implement actual port stopping
        let mut state = self.state.write().await;
        state.is_listening = false;
        Ok(())
    }

    pub async fn update_config(&self, config: PortConfig) -> Result<()> {
        let mut current_config = self.config.write().await;
        *current_config = config;
        Ok(())
    }

    pub async fn get_config(&self) -> Result<PortConfig> {
        let config = self.config.read().await;
        Ok(config.clone())
    }

    pub async fn get_state(&self) -> Result<PortState> {
        let state = self.state.read().await;
        Ok(state.clone())
    }

    pub async fn record_connection(&self) -> Result<()> {
        let mut state = self.state.write().await;
        state.active_connections += 1;
        state.total_connections += 1;
        state.last_connection = chrono::Utc::now();
        Ok(())
    }

    pub async fn record_disconnection(&self) -> Result<()> {
        let mut state = self.state.write().await;
        if state.active_connections > 0 {
            state.active_connections -= 1;
        }
        Ok(())
    }
}

impl Default for PortConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            host: "127.0.0.1".to_string(),
            max_connections: 100,
            timeout_ms: 30000, // 30 seconds
        }
    }
}

impl Default for MCPPort {
    fn default() -> Self {
        Self::new(PortConfig::default())
    }
} 