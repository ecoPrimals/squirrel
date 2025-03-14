use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub interval: std::time::Duration,
    pub metrics_enabled: bool,
    pub tracing_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_io: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct MonitoringService {
    config: MonitoringConfig,
    health_checks: Arc<RwLock<Vec<HealthCheck>>>,
    metrics: Arc<RwLock<Vec<PerformanceMetric>>>,
}

impl MonitoringService {
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            config,
            health_checks: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn record_health_check(&self, check: HealthCheck) {
        self.health_checks.write().await.push(check);
    }

    pub async fn record_metric(&self, metric: PerformanceMetric) {
        self.metrics.write().await.push(metric);
    }

    pub async fn get_health_checks(&self) -> Vec<HealthCheck> {
        self.health_checks.read().await.clone()
    }

    pub async fn get_metrics(&self) -> Vec<PerformanceMetric> {
        self.metrics.read().await.clone()
    }
} 