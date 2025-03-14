use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::error::{MCPError, Result};
use crate::mcp::types::{ProtocolState, ProtocolVersion};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub message_count: u64,
    pub error_count: u64,
    pub latency_ms: f64,
    pub throughput: f64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub state: ProtocolState,
    pub version: ProtocolVersion,
    pub metrics: Metrics,
    pub last_updated: DateTime<Utc>,
}

pub struct MCPMonitor {
    metrics: Arc<RwLock<Metrics>>,
    health_status: Arc<RwLock<HealthStatus>>,
}

impl MCPMonitor {
    pub fn new() -> Self {
        let metrics = Metrics {
            message_count: 0,
            error_count: 0,
            latency_ms: 0.0,
            throughput: 0.0,
            last_updated: Utc::now(),
        };

        let health_status = HealthStatus {
            is_healthy: true,
            state: ProtocolState::Initializing,
            version: ProtocolVersion::default(),
            metrics: metrics.clone(),
            last_updated: Utc::now(),
        };

        Self {
            metrics: Arc::new(RwLock::new(metrics)),
            health_status: Arc::new(RwLock::new(health_status)),
        }
    }

    pub async fn record_message(&self, latency_ms: f64) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        metrics.message_count += 1;
        metrics.latency_ms = (metrics.latency_ms * (metrics.message_count - 1) as f64 + latency_ms) / metrics.message_count as f64;
        metrics.throughput = metrics.message_count as f64 / (Utc::now() - metrics.last_updated).num_seconds() as f64;
        metrics.last_updated = Utc::now();
        Ok(())
    }

    pub async fn record_error(&self) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        metrics.error_count += 1;
        metrics.last_updated = Utc::now();
        Ok(())
    }

    pub async fn update_health_status(&self, state: ProtocolState, version: ProtocolVersion) -> Result<()> {
        let mut health_status = self.health_status.write().await;
        health_status.state = state;
        health_status.version = version;
        health_status.is_healthy = state != ProtocolState::Error;
        health_status.last_updated = Utc::now();
        Ok(())
    }

    pub async fn get_metrics(&self) -> Result<Metrics> {
        let metrics = self.metrics.read().await;
        Ok(metrics.clone())
    }

    pub async fn get_health_status(&self) -> Result<HealthStatus> {
        let health_status = self.health_status.read().await;
        Ok(health_status.clone())
    }
}

impl Default for MCPMonitor {
    fn default() -> Self {
        Self::new()
    }
} 