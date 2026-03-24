// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Comprehensive monitoring system for MCP
//!
//! This module provides a complete monitoring solution for the Machine Context Protocol,
//! including metrics collection, health monitoring, alerting, and dashboard capabilities.
//! The monitoring system has been split into focused modules for better organization.
//!
//! ## Architecture
//!
//! The monitoring system is composed of several focused modules:
//!
//! * **health**: System health monitoring including CPU, memory, disk, and service health
//! * **metrics**: Metrics collection with OpenTelemetry integration
//! * **alerts**: Alert management and notification system
//! * **dashboard**: Web dashboard for monitoring visualization
//! * **clients**: Monitoring client interfaces and implementations
//! * **system**: Main monitoring system orchestration
//! * **songbird_client**: Integration with Songbird monitoring service
//!
//! ## Features
//!
//! * Real-time health monitoring with system resource tracking
//! * Comprehensive metrics collection with OpenTelemetry support
//! * Intelligent alerting with multiple severity levels
//! * Web dashboard for visualization and management
//! * Circuit breaker monitoring and reporting
//! * Integration with external monitoring services
//! * In-memory and production clients for monitoring backends
//!
//! ## Usage
//!
//! ```rust,no_run
//! use crate::monitoring::{MonitoringSystem, MCPMonitor};
//!
//! // Create and start the monitoring system
//! let mut monitoring_system = MonitoringSystem::new();
//! monitoring_system.enable_dashboard(8080);
//! monitoring_system.start().await?;
//!
//! // Create an MCP monitor for tracking operations
//! let monitor = MCPMonitor::new().await?;
//! monitor.record_message("request").await;
//! monitor.record_sync_operation(150.0, true).await;
//!
//! // Get health status and metrics
//! let health = monitor.get_health().await?;
//! let metrics = monitor.get_metrics().await?;
//! ```

// Core modules
pub mod health;
pub mod metrics;
pub mod alerts;
pub mod dashboard;
pub mod clients;
pub mod system;
pub mod songbird_client;

// Re-export main types for convenience
pub use health::{HealthStatus, HealthMonitor, SyncHealth, PersistenceHealth, ResourceHealth, HealthAlert, HealthAlertType, HealthAlertSeverity};
pub use metrics::{Metrics, MetricsCollector};
pub use alerts::{AlertManager, AlertLevel, AlertSummary};
pub use dashboard::DashboardServer;
pub use clients::{MonitoringClient, MonitoringEvent, MetricValue, InMemoryMonitoringClient, ProductionMonitoringClient, MonitoringClientConfig};
pub use system::{MonitoringSystem, MonitoringStatus, MonitoringSystemSummary, MonitoringError};
pub use songbird_client::{SongbirdMonitoringClient, SongbirdClientConfig, create_songbird_client, create_songbird_client_with_config};

// Main types and functionality
use crate::context_manager::Context;
use crate::error::{MCPError, Result};
use chrono::{DateTime, Utc};
use opentelemetry::{
    metrics::{Counter, Histogram, Meter, MeterProvider, Unit},
    KeyValue,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{error, info};
use std::sync::Arc;

/// Create a production monitoring client
/// 
/// This function replaces InMemoryMonitoringClient usage with a real Songbird integration.
/// In production, this connects to Songbird for observability.
/// In development/testing, it provides safe fallback behavior.
pub fn create_production_monitoring_client() -> Arc<dyn MonitoringClient> {
    let mut config = SongbirdClientConfig::default();
    
    // Check if we're in a test environment
    if std::env::var("RUST_TEST").is_ok() || cfg!(test) {
        // For tests, use a different port to avoid conflicts
        let test_port = std::env::var("SONGBIRD_TEST_PORT").unwrap_or_else(|_| "18900".to_string());
        config.endpoint = format!("http://localhost:{}", test_port);
        config.timeout_ms = 1000; // Shorter timeout for tests
        config.collection_interval = 5; // More frequent collection for tests
    }
    
    // PRODUCTION SAFE: Handle Songbird client creation with safe fallbacks
    match SongbirdMonitoringClient::new(config) {
        Ok(client) => Arc::new(client) as Arc<dyn MonitoringClient>,
        Err(e) => {
            tracing::error!("Failed to create Songbird monitoring client in production factory: {}", e);
            // Use the safe factory function from songbird_client module
            create_songbird_client() as Arc<dyn MonitoringClient>
        }
    }
}

/// Create a monitoring client with custom Songbird configuration
pub fn create_monitoring_client_with_config(config: SongbirdClientConfig) -> Arc<dyn MonitoringClient> {
    // PRODUCTION SAFE: Handle Songbird client creation with safe fallbacks
    match SongbirdMonitoringClient::new(config.clone()) {
        Ok(client) => Arc::new(client) as Arc<dyn MonitoringClient>,
        Err(e) => {
            tracing::error!("Failed to create Songbird monitoring client with custom config in factory: {}", e);
            // Use the safe factory function with the custom config
            create_songbird_client_with_config(config) as Arc<dyn MonitoringClient>
        }
    }
}

/// Monitoring system for the Machine Context Protocol
///
/// The `MCPMonitor` provides real-time tracking of MCP operations, performance metrics,
/// and health status information. It integrates with OpenTelemetry for metrics collection
/// and supports both synchronous and asynchronous monitoring operations.
#[derive(Debug)]
pub struct MCPMonitor {
    /// Current metrics for MCP operations
    metrics: Arc<RwLock<Metrics>>,
    /// Health monitor for system health tracking
    health_monitor: Arc<HealthMonitor>,
    /// OpenTelemetry meter for collecting metrics
    meter: Meter,
    /// Counter for tracking processed messages
    message_counter: Counter<u64>,
    /// Counter for tracking encountered errors
    error_counter: Counter<u64>,
    /// Histogram for measuring sync operation durations
    sync_duration: Histogram<f64>,
    /// Counter for tracking context operations
    context_operation_counter: Counter<u64>,
}

impl MCPMonitor {
    /// Creates a new `MCPMonitor` instance
    ///
    /// # Returns
    /// A Result containing the new `MCPMonitor` instance or an error
    ///
    /// # Errors
    /// Returns an error if the OpenTelemetry metrics initialization fails
    pub async fn new() -> Result<Self> {
        // Initialize OpenTelemetry MeterProvider
        let meter_provider = opentelemetry_sdk::metrics::MeterProvider::builder().build();
        let meter = meter_provider.meter("mcp_monitor");

        // Create metrics
        let message_counter = meter
            .u64_counter("mcp.messages")
            .with_description("Total number of messages processed")
            .init();

        let error_counter = meter
            .u64_counter("mcp.errors")
            .with_description("Total number of errors encountered")
            .init();

        let sync_duration = meter
            .f64_histogram("mcp.sync_duration")
            .with_description("Duration of sync operations in milliseconds")
            .with_unit(Unit::new("ms"))
            .init();

        let context_operation_counter = meter
            .u64_counter("mcp.context_operations")
            .with_description("Number of context operations performed")
            .init();

        // Create health monitor
        let health_monitor = Arc::new(HealthMonitor::new());

        // Create initial metrics
        let initial_metrics = Metrics {
            total_messages: 0,
            total_errors: 0,
            sync_operations: 0,
            context_operations: 0,
            active_contexts: 0,
            last_sync_duration_ms: 0.0,
        };

        Ok(Self {
            metrics: Arc::new(RwLock::new(initial_metrics)),
            health_monitor,
            meter,
            message_counter,
            error_counter,
            sync_duration,
            context_operation_counter,
        })
    }

    /// Creates a new `MCPMonitor` instance with synchronous initialization
    pub fn default_sync() -> Self {
        // Create health monitor
        let health_monitor = Arc::new(HealthMonitor::new());

        // Create initial metrics without OpenTelemetry (for sync context)
        let initial_metrics = Metrics {
            total_messages: 0,
            total_errors: 0,
            sync_operations: 0,
            context_operations: 0,
            active_contexts: 0,
            last_sync_duration_ms: 0.0,
        };

        // Create a simple meter provider for sync context
        let meter_provider = opentelemetry_sdk::metrics::MeterProvider::builder().build();
        let meter = meter_provider.meter("mcp_monitor_sync");

        let message_counter = meter.u64_counter("mcp.messages").init();
        let error_counter = meter.u64_counter("mcp.errors").init();
        let sync_duration = meter.f64_histogram("mcp.sync_duration").init();
        let context_operation_counter = meter.u64_counter("mcp.context_operations").init();

        Self {
            metrics: Arc::new(RwLock::new(initial_metrics)),
            health_monitor,
            meter,
            message_counter,
            error_counter,
            sync_duration,
            context_operation_counter,
        }
    }

    /// Records a processed message
    pub async fn record_message(&self, message_type: &str) {
        let mut metrics = self.metrics.write().await;
        metrics.total_messages += 1;
        
        // Record in OpenTelemetry
        self.message_counter.add(1, &[KeyValue::new("type", message_type.to_string())]);
    }

    /// Records an error
    pub async fn record_error(&self, error_type: &str) {
        let mut metrics = self.metrics.write().await;
        metrics.total_errors += 1;
        
        // Record in OpenTelemetry
        self.error_counter.add(1, &[KeyValue::new("type", error_type.to_string())]);
    }

    /// Records a sync operation
    pub async fn record_sync_operation(&self, duration_ms: f64, success: bool) {
        let mut metrics = self.metrics.write().await;
        metrics.sync_operations += 1;
        metrics.last_sync_duration_ms = duration_ms;
        
        // Record in OpenTelemetry
        self.sync_duration.record(
            duration_ms,
            &[KeyValue::new("success", success.to_string())],
        );

        // Update health monitor
        self.health_monitor.record_sync_result(success, duration_ms).await;
    }

    /// Records a context operation
    pub async fn record_context_operation(&self, _operation: &str, context: &Context) {
        let mut metrics = self.metrics.write().await;
        metrics.context_operations += 1;
        
        // Update active contexts count (simplified)
        metrics.active_contexts = 1; // This would be more sophisticated in a real implementation
        
        // Record in OpenTelemetry
        self.context_operation_counter.add(1, &[
            KeyValue::new("operation", _operation.to_string()),
            KeyValue::new("context_id", context.id.clone()),
        ]);
    }

    /// Updates the health status
    pub async fn update_health(&self) -> Result<()> {
        self.health_monitor.update_health().await
    }

    /// Gets the current metrics
    pub async fn get_metrics(&self) -> Result<Metrics> {
        Ok(self.metrics.read().await.clone())
    }

    /// Gets the current health status
    pub async fn get_health(&self) -> Result<HealthStatus> {
        Ok(self.health_monitor.get_health().await)
    }

    /// Updates persistence status
    pub async fn update_persistence_status(&self, available: bool) {
        self.health_monitor.update_persistence_status(available, available).await;
    }

    /// Records a successful sync operation with details
    pub async fn record_sync_success(&self, local_changes: usize, remote_changes: usize, duration_ms: u64) {
        self.record_sync_operation(duration_ms as f64, true).await;
        info!("Sync completed successfully: {} local changes, {} remote changes, {}ms", 
              local_changes, remote_changes, duration_ms);
    }

    /// Records a failed sync operation
    pub async fn record_sync_failure(&self, error: String) {
        self.record_sync_operation(0.0, false).await;
        error!("Sync failed: {}", error);
    }

    /// Get the health monitor
    pub fn health_monitor(&self) -> Arc<HealthMonitor> {
        Arc::clone(&self.health_monitor)
    }
}

impl Clone for MCPMonitor {
    fn clone(&self) -> Self {
        Self {
            metrics: Arc::clone(&self.metrics),
            health_monitor: Arc::clone(&self.health_monitor),
            meter: self.meter.clone(),
            message_counter: self.message_counter.clone(),
            error_counter: self.error_counter.clone(),
            sync_duration: self.sync_duration.clone(),
            context_operation_counter: self.context_operation_counter.clone(),
        }
    }
}

impl Default for MCPMonitor {
    fn default() -> Self {
        Self::default_sync()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_recording() {
        let monitor = MCPMonitor::new().await.expect("should succeed");
        
        // Record some operations
        monitor.record_message("test_message").await;
        monitor.record_error("test_error").await;
        monitor.record_sync_operation(100.0, true).await;
        
        // Check metrics
        let metrics = monitor.get_metrics().await.expect("should succeed");
        assert_eq!(metrics.total_messages, 1);
        assert_eq!(metrics.total_errors, 1);
        assert_eq!(metrics.sync_operations, 1);
        assert_eq!(metrics.last_sync_duration_ms, 100.0);
    }

    #[tokio::test]
    async fn test_health_status() {
        let monitor = MCPMonitor::new().await.expect("should succeed");
        
        // Update health status
        monitor.update_health().await.expect("should succeed");
        
        // Check health
        let health = monitor.get_health().await.expect("should succeed");
        assert!(health.is_healthy);
    }
} 