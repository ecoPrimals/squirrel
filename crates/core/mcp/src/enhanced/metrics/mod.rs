// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Enhanced MCP Metrics Collection System
//!
//! This module provides comprehensive metrics collection and monitoring for all
//! enhanced MCP components, including real-time performance tracking, alerting,
//! and integration with external monitoring systems.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Mutex};
use tokio::time::interval;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::{debug, info, warn, error, instrument};
use uuid::Uuid;

use crate::error::{Result, types::MCPError};

pub mod aggregator;
pub mod collector;
pub mod exporter;
pub mod alerts;
pub mod dashboard;

// Re-export key types
pub use aggregator::*;
pub use collector::*;
pub use exporter::*;
pub use alerts::*;
pub use dashboard::*;

/// Unified metrics manager for all enhanced MCP components
#[derive(Debug)]
pub struct EnhancedMetricsManager {
    /// Metrics collector
    collector: Arc<UnifiedMetricsCollector>,
    
    /// Metrics aggregator
    aggregator: Arc<MetricsAggregator>,
    
    /// Alert manager
    alert_manager: Arc<MetricsAlertManager>,
    
    /// Metrics exporters
    exporters: Arc<RwLock<HashMap<String, Box<dyn MetricsExporter>>>>,
    
    /// Configuration
    config: MetricsConfig,
    
    /// Manager state
    state: Arc<RwLock<ManagerState>>,
}

/// Configuration for the metrics system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Collection interval in seconds
    pub collection_interval_secs: u64,
    
    /// Enable detailed performance tracking
    pub enable_performance_tracking: bool,
    
    /// Enable real-time alerting
    pub enable_alerting: bool,
    
    /// Enable metrics export
    pub enable_export: bool,
    
    /// Retention period for metrics data
    pub retention_period_secs: u64,
    
    /// Maximum number of historical data points
    pub max_history_points: usize,
    
    /// Export destinations
    pub export_destinations: Vec<ExportDestination>,
    
    /// Alert configuration
    pub alert_config: AlertConfig,
}

/// Export destination configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportDestination {
    /// Destination name
    pub name: String,
    
    /// Destination type (prometheus, datadog, etc.)
    pub destination_type: String,
    
    /// Connection parameters
    pub parameters: HashMap<String, String>,
    
    /// Export interval in seconds
    pub export_interval_secs: u64,
    
    /// Enabled flag
    pub enabled: bool,
}

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Enable alerts
    pub enabled: bool,
    
    /// Alert thresholds
    pub thresholds: HashMap<String, AlertThreshold>,
    
    /// Notification channels
    pub notification_channels: Vec<NotificationChannel>,
    
    /// Cooldown period between alerts
    pub cooldown_period_secs: u64,
}

/// Alert threshold definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThreshold {
    /// Metric name pattern
    pub metric_pattern: String,
    
    /// Threshold value
    pub threshold: f64,
    
    /// Comparison operator (gt, lt, eq, gte, lte)
    pub operator: String,
    
    /// Alert severity
    pub severity: AlertSeverity,
    
    /// Description
    pub description: String,
}

/// Notification channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    /// Channel name
    pub name: String,
    
    /// Channel type (email, slack, webhook, etc.)
    pub channel_type: String,
    
    /// Channel configuration
    pub config: HashMap<String, String>,
    
    /// Enabled flag
    pub enabled: bool,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Manager state
#[derive(Debug, Clone)]
pub struct ManagerState {
    /// Manager status
    pub status: ManagerStatus,
    
    /// Start time
    pub started_at: Option<DateTime<Utc>>,
    
    /// Last collection time
    pub last_collection: Option<DateTime<Utc>>,
    
    /// Collection count
    pub collection_count: u64,
    
    /// Export count
    pub export_count: u64,
    
    /// Alert count
    pub alert_count: u64,
    
    /// Error count
    pub error_count: u64,
}

/// Manager status
#[derive(Debug, Clone, PartialEq)]
pub enum ManagerStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error(String),
}

impl EnhancedMetricsManager {
    /// Create a new metrics manager
    pub async fn new(config: MetricsConfig) -> Result<Self> {
        let collector = Arc::new(UnifiedMetricsCollector::new().await?);
        let aggregator = Arc::new(MetricsAggregator::new(config.clone()).await?);
        let alert_manager = Arc::new(MetricsAlertManager::new(config.alert_config.clone()).await?);
        
        let state = Arc::new(RwLock::new(ManagerState {
            status: ManagerStatus::Stopped,
            started_at: None,
            last_collection: None,
            collection_count: 0,
            export_count: 0,
            alert_count: 0,
            error_count: 0,
        }));
        
        Ok(Self {
            collector,
            aggregator,
            alert_manager,
            exporters: Arc::new(RwLock::new(HashMap::new())),
            config,
            state,
        })
    }
    
    /// Start the metrics manager
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<()> {
        info!("Starting Enhanced MCP Metrics Manager");
        
        // Update state
        {
            let mut state = self.state.write().await;
            state.status = ManagerStatus::Starting;
            state.started_at = Some(Utc::now());
        }
        
        // Initialize components
        self.collector.start().await?;
        self.aggregator.start().await?;
        self.alert_manager.start().await?;
        
        // Initialize exporters
        self.initialize_exporters().await?;
        
        // Start collection loop
        self.start_collection_loop().await;
        
        // Start aggregation loop
        self.start_aggregation_loop().await;
        
        // Start alert processing
        self.start_alert_processing().await;
        
        // Start export loop
        self.start_export_loop().await;
        
        // Update state to running
        {
            let mut state = self.state.write().await;
            state.status = ManagerStatus::Running;
        }
        
        info!("Enhanced MCP Metrics Manager started successfully");
        Ok(())
    }
    
    /// Stop the metrics manager
    #[instrument(skip(self))]
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping Enhanced MCP Metrics Manager");
        
        // Update state
        {
            let mut state = self.state.write().await;
            state.status = ManagerStatus::Stopping;
        }
        
        // Stop components
        self.collector.stop().await?;
        self.aggregator.stop().await?;
        self.alert_manager.stop().await?;
        
        // Stop exporters
        let exporters = self.exporters.read().await;
        for exporter in exporters.values() {
            if let Err(e) = exporter.stop().await {
                warn!("Error stopping exporter: {}", e);
            }
        }
        
        // Update state to stopped
        {
            let mut state = self.state.write().await;
            state.status = ManagerStatus::Stopped;
        }
        
        info!("Enhanced MCP Metrics Manager stopped");
        Ok(())
    }
    
    /// Get current metrics snapshot
    pub async fn get_metrics_snapshot(&self) -> Result<MetricsSnapshot> {
        let raw_metrics = self.collector.collect_all_metrics().await?;
        let aggregated_metrics = self.aggregator.get_current_aggregation().await?;
        let alerts = self.alert_manager.get_active_alerts().await?;
        
        Ok(MetricsSnapshot {
            timestamp: Utc::now(),
            raw_metrics,
            aggregated_metrics,
            active_alerts: alerts,
            system_health: self.calculate_system_health().await?,
        })
    }
    
    /// Get performance summary
    pub async fn get_performance_summary(&self) -> Result<PerformanceSummary> {
        let snapshot = self.get_metrics_snapshot().await?;
        
        Ok(PerformanceSummary {
            overall_health_score: snapshot.system_health.overall_score,
            avg_response_time: snapshot.aggregated_metrics.avg_response_time,
            throughput_per_second: snapshot.aggregated_metrics.throughput_per_second,
            error_rate: snapshot.aggregated_metrics.error_rate,
            active_connections: snapshot.aggregated_metrics.active_connections,
            memory_usage_mb: snapshot.aggregated_metrics.memory_usage_bytes / 1024 / 1024,
            cpu_usage_percent: snapshot.aggregated_metrics.cpu_usage_percent,
            critical_alerts: snapshot.active_alerts.iter()
                .filter(|a| a.severity == AlertSeverity::Critical)
                .count() as u64,
        })
    }
    
    /// Register a custom metrics exporter
    pub async fn register_exporter(&self, name: String, exporter: Box<dyn MetricsExporter>) -> Result<()> {
        let mut exporters = self.exporters.write().await;
        exporters.insert(name, exporter);
        Ok(())
    }
    
    /// Get manager state
    pub async fn get_state(&self) -> ManagerState {
        self.state.read().await.clone()
    }
    
    // Private helper methods
    
    /// Initialize exporters based on configuration
    async fn initialize_exporters(&self) -> Result<()> {
        if !self.config.enable_export {
            return Ok(());
        }
        
        let mut exporters = self.exporters.write().await;
        
        for destination in &self.config.export_destinations {
            if !destination.enabled {
                continue;
            }
            
            let exporter: Box<dyn MetricsExporter> = match destination.destination_type.as_str() {
                "prometheus" => Box::new(PrometheusExporter::new(destination.clone()).await?),
                "datadog" => Box::new(DatadogExporter::new(destination.clone()).await?),
                "json" => Box::new(JsonExporter::new(destination.clone()).await?),
                "influxdb" => Box::new(InfluxDbExporter::new(destination.clone()).await?),
                _ => {
                    warn!("Unknown exporter type: {}", destination.destination_type);
                    continue;
                }
            };
            
            exporters.insert(destination.name.clone(), exporter);
            info!("Initialized exporter: {} ({})", destination.name, destination.destination_type);
        }
        
        Ok(())
    }
    
    /// Start the metrics collection loop
    async fn start_collection_loop(&self) {
        let collector = self.collector.clone();
        let aggregator = self.aggregator.clone();
        let state = self.state.clone();
        let interval_secs = self.config.collection_interval_secs;
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(interval_secs));
            
            loop {
                interval.tick().await;
                
                // Check if we should still be running
                let should_continue = {
                    let state_guard = state.read().await;
                    matches!(state_guard.status, ManagerStatus::Running)
                };
                
                if !should_continue {
                    break;
                }
                
                // Collect metrics
                match collector.collect_all_metrics().await {
                    Ok(metrics) => {
                        // Send to aggregator
                        if let Err(e) = aggregator.process_metrics(metrics).await {
                            error!("Error processing metrics in aggregator: {}", e);
                        }
                        
                        // Update state
                        let mut state_guard = state.write().await;
                        state_guard.last_collection = Some(Utc::now());
                        state_guard.collection_count += 1;
                    }
                    Err(e) => {
                        error!("Error collecting metrics: {}", e);
                        let mut state_guard = state.write().await;
                        state_guard.error_count += 1;
                    }
                }
            }
        });
    }
    
    /// Start the aggregation loop
    async fn start_aggregation_loop(&self) {
        let aggregator = self.aggregator.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = aggregator.update_aggregations().await {
                    error!("Error updating aggregations: {}", e);
                }
            }
        });
    }
    
    /// Start alert processing
    async fn start_alert_processing(&self) {
        if !self.config.enable_alerting {
            return;
        }
        
        let alert_manager = self.alert_manager.clone();
        let aggregator = self.aggregator.clone();
        let state = self.state.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));
            
            loop {
                interval.tick().await;
                
                // Get current metrics
                match aggregator.get_current_aggregation().await {
                    Ok(metrics) => {
                        // Check for alerts
                        match alert_manager.check_alerts(&metrics).await {
                            Ok(triggered_alerts) => {
                                for alert in triggered_alerts {
                                    if let Err(e) = alert_manager.trigger_alert(alert).await {
                                        error!("Error triggering alert: {}", e);
                                    }
                                    
                                    let mut state_guard = state.write().await;
                                    state_guard.alert_count += 1;
                                }
                            }
                            Err(e) => {
                                error!("Error checking alerts: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error getting metrics for alert processing: {}", e);
                    }
                }
            }
        });
    }
    
    /// Start the export loop
    async fn start_export_loop(&self) {
        if !self.config.enable_export {
            return;
        }
        
        let exporters = self.exporters.clone();
        let aggregator = self.aggregator.clone();
        let state = self.state.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                // Get current metrics
                match aggregator.get_current_aggregation().await {
                    Ok(metrics) => {
                        // Export to all configured destinations
                        let exporters_guard = exporters.read().await;
                        for (name, exporter) in exporters_guard.iter() {
                            match exporter.export_metrics(&metrics).await {
                                Ok(_) => {
                                    debug!("Successfully exported metrics to {}", name);
                                }
                                Err(e) => {
                                    warn!("Error exporting metrics to {}: {}", name, e);
                                }
                            }
                        }
                        
                        let mut state_guard = state.write().await;
                        state_guard.export_count += 1;
                    }
                    Err(e) => {
                        error!("Error getting metrics for export: {}", e);
                    }
                }
            }
        });
    }
    
    /// Calculate overall system health
    async fn calculate_system_health(&self) -> Result<SystemHealth> {
        let metrics = self.aggregator.get_current_aggregation().await?;
        let alerts = self.alert_manager.get_active_alerts().await?;
        
        // Calculate health score based on various factors
        let mut health_score = 1.0;
        
        // Deduct for high error rates
        if metrics.error_rate > 0.05 {
            health_score -= (metrics.error_rate - 0.05) * 2.0;
        }
        
        // Deduct for high response times
        if metrics.avg_response_time.as_millis() > 1000 {
            health_score -= 0.2;
        }
        
        // Deduct for critical alerts
        let critical_alerts = alerts.iter()
            .filter(|a| a.severity == AlertSeverity::Critical)
            .count();
        
        health_score -= critical_alerts as f64 * 0.3;
        
        // Ensure score is between 0 and 1
        health_score = health_score.max(0.0).min(1.0);
        
        let status = if health_score >= 0.8 {
            HealthStatus::Healthy
        } else if health_score >= 0.6 {
            HealthStatus::Warning
        } else if health_score >= 0.3 {
            HealthStatus::Critical
        } else {
            HealthStatus::Emergency
        };
        
        Ok(SystemHealth {
            status,
            overall_score: health_score,
            component_scores: HashMap::new(), // Would be populated with individual component scores
            last_updated: Utc::now(),
        })
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            collection_interval_secs: 30,
            enable_performance_tracking: true,
            enable_alerting: true,
            enable_export: false,
            retention_period_secs: 86400, // 24 hours
            max_history_points: 2880, // 30-second intervals for 24 hours
            export_destinations: vec![],
            alert_config: AlertConfig {
                enabled: true,
                thresholds: HashMap::new(),
                notification_channels: vec![],
                cooldown_period_secs: 300, // 5 minutes
            },
        }
    }
}

/// Comprehensive metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// Snapshot timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Raw metrics from all components
    pub raw_metrics: UnifiedMetrics,
    
    /// Aggregated metrics
    pub aggregated_metrics: AggregatedMetrics,
    
    /// Active alerts
    pub active_alerts: Vec<Alert>,
    
    /// System health assessment
    pub system_health: SystemHealth,
}

/// Performance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    /// Overall system health score (0.0 to 1.0)
    pub overall_health_score: f64,
    
    /// Average response time across all components
    pub avg_response_time: Duration,
    
    /// System throughput (operations per second)
    pub throughput_per_second: f64,
    
    /// Overall error rate
    pub error_rate: f64,
    
    /// Total active connections
    pub active_connections: u64,
    
    /// Memory usage in MB
    pub memory_usage_mb: u64,
    
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    
    /// Number of critical alerts
    pub critical_alerts: u64,
}

/// System health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    /// Overall health status
    pub status: HealthStatus,
    
    /// Overall health score (0.0 to 1.0)
    pub overall_score: f64,
    
    /// Individual component health scores
    pub component_scores: HashMap<String, f64>,
    
    /// Last health check timestamp
    pub last_updated: DateTime<Utc>,
}

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Emergency,
} 