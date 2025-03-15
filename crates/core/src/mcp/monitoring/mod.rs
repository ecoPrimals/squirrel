use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use opentelemetry::{
    metrics::{Counter, Histogram, Meter, MeterProvider, Unit},
    KeyValue,
};
use opentelemetry_sdk::{
    metrics::{MeterProvider as SdkMeterProvider, PeriodicReader},
    runtime,
};
use opentelemetry_otlp::WithExportConfig;
use crate::error::{MCPError, Result};
use crate::mcp::types::{ProtocolState, ProtocolVersion};
use crate::mcp::sync::{MCPSync, StateOperation};
use crate::mcp::context::Context;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub total_messages: u64,
    pub total_errors: u64,
    pub sync_operations: u64,
    pub context_operations: u64,
    pub active_contexts: u64,
    pub last_sync_duration_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub last_check: DateTime<Utc>,
    pub sync_status: SyncHealth,
    pub persistence_status: PersistenceHealth,
    pub resource_status: ResourceHealth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncHealth {
    pub is_syncing: bool,
    pub last_successful_sync: DateTime<Utc>,
    pub consecutive_failures: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceHealth {
    pub storage_available: bool,
    pub last_write_success: DateTime<Utc>,
    pub storage_usage_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceHealth {
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub disk_usage_percent: f64,
}

pub struct MCPMonitor {
    metrics: Arc<RwLock<Metrics>>,
    health: Arc<RwLock<HealthStatus>>,
    meter: Meter,
    message_counter: Counter<u64>,
    error_counter: Counter<u64>,
    sync_duration: Histogram<f64>,
    context_operation_counter: Counter<u64>,
}

impl MCPMonitor {
    pub async fn new() -> Result<Self> {
        // Initialize OpenTelemetry
        let meter_provider = SdkMeterProvider::builder()
            .with_reader(PeriodicReader::builder(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint("http://localhost:4317"),
                runtime::Tokio,
            ).build())
            .build();

        let meter = meter_provider.meter("mcp_monitor");

        // Create metrics
        let message_counter = meter
            .u64_counter("mcp.messages")
            .with_description("Total number of messages processed")
            .with_unit(Unit::new("messages"))
            .init();

        let error_counter = meter
            .u64_counter("mcp.errors")
            .with_description("Total number of errors encountered")
            .with_unit(Unit::new("errors"))
            .init();

        let sync_duration = meter
            .f64_histogram("mcp.sync_duration")
            .with_description("Duration of sync operations")
            .with_unit(Unit::new("milliseconds"))
            .init();

        let context_operation_counter = meter
            .u64_counter("mcp.context_operations")
            .with_description("Total number of context operations")
            .with_unit(Unit::new("operations"))
            .init();

        Ok(Self {
            metrics: Arc::new(RwLock::new(Metrics {
                total_messages: 0,
                total_errors: 0,
                sync_operations: 0,
                context_operations: 0,
                active_contexts: 0,
                last_sync_duration_ms: 0.0,
            })),
            health: Arc::new(RwLock::new(HealthStatus {
                is_healthy: true,
                last_check: Utc::now(),
                sync_status: SyncHealth {
                    is_syncing: false,
                    last_successful_sync: Utc::now(),
                    consecutive_failures: 0,
                },
                persistence_status: PersistenceHealth {
                    storage_available: true,
                    last_write_success: Utc::now(),
                    storage_usage_percent: 0.0,
                },
                resource_status: ResourceHealth {
                    cpu_usage_percent: 0.0,
                    memory_usage_percent: 0.0,
                    disk_usage_percent: 0.0,
                },
            })),
            meter,
            message_counter,
            error_counter,
            sync_duration,
            context_operation_counter,
        })
    }

    pub async fn record_message(&self, message_type: &str) {
        let mut metrics = self.metrics.write().await;
        metrics.total_messages += 1;
        self.message_counter.add(1, &[KeyValue::new("type", message_type.to_string())]);
    }

    pub async fn record_error(&self, error_type: &str) {
        let mut metrics = self.metrics.write().await;
        metrics.total_errors += 1;
        self.error_counter.add(1, &[KeyValue::new("type", error_type.to_string())]);
    }

    pub async fn record_sync_operation(&self, duration_ms: f64, success: bool) {
        let mut metrics = self.metrics.write().await;
        metrics.sync_operations += 1;
        metrics.last_sync_duration_ms = duration_ms;

        let mut health = self.health.write().await;
        if success {
            health.sync_status.consecutive_failures = 0;
            health.sync_status.last_successful_sync = Utc::now();
        } else {
            health.sync_status.consecutive_failures += 1;
        }
        health.sync_status.is_syncing = false;

        self.sync_duration.record(duration_ms, &[KeyValue::new("success", success.to_string())]);
    }

    pub async fn record_context_operation(&self, operation: StateOperation, context: &Context) {
        let mut metrics = self.metrics.write().await;
        metrics.context_operations += 1;
        
        match operation {
            StateOperation::Create => metrics.active_contexts += 1,
            StateOperation::Delete => metrics.active_contexts = metrics.active_contexts.saturating_sub(1),
            _ => {}
        }

        self.context_operation_counter.add(1, &[
            KeyValue::new("operation", format!("{:?}", operation)),
            KeyValue::new("context_type", context.name.clone()),
        ]);
    }

    pub async fn update_health(&self) -> Result<()> {
        let mut health = self.health.write().await;
        health.last_check = Utc::now();

        // Update resource metrics
        if let Ok(sys_info) = sysinfo::System::new_all() {
            health.resource_status = ResourceHealth {
                cpu_usage_percent: sys_info.global_cpu_info().cpu_usage() as f64,
                memory_usage_percent: (sys_info.used_memory() as f64 / sys_info.total_memory() as f64) * 100.0,
                disk_usage_percent: sys_info.disks().iter()
                    .map(|disk| (disk.total_space() - disk.available_space()) as f64 / disk.total_space() as f64 * 100.0)
                    .sum::<f64>() / sys_info.disks().len() as f64,
            };
        }

        // Update overall health status
        health.is_healthy = health.sync_status.consecutive_failures < 3 
            && health.persistence_status.storage_available
            && health.resource_status.cpu_usage_percent < 90.0
            && health.resource_status.memory_usage_percent < 90.0
            && health.resource_status.disk_usage_percent < 90.0;

        Ok(())
    }

    pub async fn get_metrics(&self) -> Result<Metrics> {
        Ok(self.metrics.read().await.clone())
    }

    pub async fn get_health(&self) -> Result<HealthStatus> {
        self.update_health().await?;
        Ok(self.health.read().await.clone())
    }

    pub async fn update_persistence_status(&self, available: bool) {
        let mut health = self.health.write().await;
        health.persistence_status.storage_available = available;
        if available {
            health.persistence_status.last_write_success = Utc::now();
        }
    }
}

impl Clone for MCPMonitor {
    fn clone(&self) -> Self {
        Self {
            metrics: self.metrics.clone(),
            health: self.health.clone(),
            meter: self.meter.clone(),
            message_counter: self.message_counter.clone(),
            error_counter: self.error_counter.clone(),
            sync_duration: self.sync_duration.clone(),
            context_operation_counter: self.context_operation_counter.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_metrics_recording() {
        let monitor = MCPMonitor::new().await.unwrap();

        // Record various operations
        monitor.record_message("test").await;
        monitor.record_error("test_error").await;
        monitor.record_sync_operation(100.0, true).await;

        let context = Context {
            id: Uuid::new_v4(),
            name: "test".to_string(),
            data: serde_json::json!({}),
            metadata: None,
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
        };
        monitor.record_context_operation(StateOperation::Create, &context).await;

        // Verify metrics
        let metrics = monitor.get_metrics().await.unwrap();
        assert_eq!(metrics.total_messages, 1);
        assert_eq!(metrics.total_errors, 1);
        assert_eq!(metrics.sync_operations, 1);
        assert_eq!(metrics.context_operations, 1);
        assert_eq!(metrics.active_contexts, 1);
        assert_eq!(metrics.last_sync_duration_ms, 100.0);
    }

    #[tokio::test]
    async fn test_health_status() {
        let monitor = MCPMonitor::new().await.unwrap();

        // Initial health check
        let health = monitor.get_health().await.unwrap();
        assert!(health.is_healthy);
        assert_eq!(health.sync_status.consecutive_failures, 0);

        // Record failed sync operations
        monitor.record_sync_operation(100.0, false).await;
        monitor.record_sync_operation(100.0, false).await;
        monitor.record_sync_operation(100.0, false).await;

        // Verify health degradation
        let health = monitor.get_health().await.unwrap();
        assert!(!health.is_healthy);
        assert_eq!(health.sync_status.consecutive_failures, 3);

        // Record successful sync
        monitor.record_sync_operation(100.0, true).await;

        // Verify health recovery
        let health = monitor.get_health().await.unwrap();
        assert!(health.is_healthy);
        assert_eq!(health.sync_status.consecutive_failures, 0);
    }
} 