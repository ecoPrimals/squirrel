// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! # MCP Observability Framework
//! 
//! This module provides a comprehensive observability framework for the Machine Context Protocol,
//! including metrics collection, distributed tracing, structured logging, health checking,
//! and alerting capabilities.
//!
//! ## Core Features
//!
//! - **Metrics Collection**: Collect and expose metrics from various MCP components
//! - **Distributed Tracing**: Track request flows across components and services
//! - **Structured Logging**: Consistent, structured logging with context
//! - **Health Checking**: Component and system-level health monitoring
//! - **Alerting**: Event-based alerting for critical system conditions
//!
//! ## Architecture
//!
//! The Observability Framework follows a modular design with these key components:
//!
//! 1. **Metrics Registry**: Central registry for all system metrics
//! 2. **Tracer**: Distributed tracing implementation
//! 3. **Logger**: Structured logging with context propagation
//! 4. **Health Checker**: Component health status monitoring
//! 5. **Alert Manager**: Alert generation and routing

use std::sync::Arc;
use std::sync::RwLock;
use log;

// Core modules
pub mod error;
pub mod metrics;
pub mod tracing;
pub mod logging;
pub mod health;
pub mod alerting;
pub mod exporters;

// New organized modules
pub mod dashboard;
pub mod registry;
pub mod events;
pub mod monitoring;
pub mod config;
pub mod status;
pub mod bridge;

#[cfg(test)]
pub mod tests;

pub use error::{ObservabilityError, ObservabilityResult};
pub use config::ObservabilityConfig;
pub use status::ObservabilityStatus;
pub use registry::{ComponentRegistry, ComponentInfo, ComponentType};
pub use events::{EventBus, ObservabilityEvent};
pub use monitoring::{PerformanceMonitor, PerformanceMetrics};
pub use dashboard::DashboardClient;
pub use bridge::MonitoringBridge;

/// Main entry point for the Observability Framework
/// 
/// Provides access to all observability components through a unified API.
#[derive(Clone)]
pub struct ObservabilityFramework {
    /// Health checker for monitoring component health
    pub health_checker: Arc<health::HealthChecker>,
    /// Alert manager for handling alerts
    pub alert_manager: Arc<alerting::AlertManager>,
    /// Metrics registry for collecting metrics
    pub metrics: Arc<metrics::MetricsRegistry>,
    /// Tracer for distributed tracing
    pub tracer: Arc<tracing::Tracer>,
    /// Logger for structured logging
    pub logger: Arc<logging::Logger>,
    /// Configuration
    pub config: Arc<RwLock<ObservabilityConfig>>,
    /// Exporters for tracing data
    pub exporters: Arc<RwLock<Vec<Arc<dyn tracing::external::SpanExporter + Send + Sync>>>>,
    /// Dashboard integration client
    pub dashboard_client: Arc<RwLock<Option<DashboardClient>>>,
    /// Component registry for tracking all monitored components
    pub component_registry: Arc<RwLock<ComponentRegistry>>,
    /// Event bus for real-time observability events
    pub event_bus: Arc<EventBus>,
    /// Performance monitor for tracking system performance
    pub performance_monitor: Arc<PerformanceMonitor>,
}

impl ObservabilityFramework {
    /// Create a new ObservabilityFramework with default configuration
    pub async fn new() -> ObservabilityResult<Self> {
        Self::new_with_config(ObservabilityConfig::default()).await
    }
    
    /// Create a new ObservabilityFramework with custom configuration
    pub async fn new_with_config(config: ObservabilityConfig) -> ObservabilityResult<Self> {
        let framework = Self {
            health_checker: Arc::new(health::HealthChecker::new()),
            alert_manager: Arc::new(alerting::AlertManager::new()),
            metrics: Arc::new(metrics::MetricsRegistry::new()),
            tracer: Arc::new(tracing::Tracer::new()),
            logger: Arc::new(logging::Logger::new()),
            config: Arc::new(RwLock::new(config)),
            exporters: Arc::new(RwLock::new(Vec::new())),
            dashboard_client: Arc::new(RwLock::new(None)),
            component_registry: Arc::new(RwLock::new(ComponentRegistry::new())),
            event_bus: Arc::new(EventBus::new()),
            performance_monitor: Arc::new(PerformanceMonitor::new()),
        };
        
        // Initialize components based on config
        framework.initialize_core_components().await?;
        framework.initialize_standard_components().await?;
        framework.register_standard_metrics()?;
        framework.setup_external_integrations().await?;
        
        Ok(framework)
    }
    
    /// Initialize core observability components
    async fn initialize_core_components(&self) -> ObservabilityResult<()> {
        let config = self.config.read()?;
        
        if config.enable_health_checks {
            self.health_checker.initialize()?;
        }
        
        if config.enable_alerting {
            self.alert_manager.initialize()?;
        }
        
        if config.enable_metrics {
            self.metrics.initialize()?;
        }
        
        if config.enable_tracing {
            self.tracer.initialize()?;
        }
        
        self.logger.initialize()?;
        self.performance_monitor.initialize()?;
        
        Ok(())
    }
    
    /// Setup external integrations (dashboard, exporters)
    async fn setup_external_integrations(&self) -> ObservabilityResult<()> {
        let config = self.config.read()?;
        
        // Setup dashboard integration
        if config.enable_dashboard_integration {
            let dashboard_client = DashboardClient::new(
                &config.dashboard_url,
                config.dashboard_auth_token.clone(),
            ).await?;
            
            *self.dashboard_client.write()? = Some(dashboard_client);
            
            // Start dashboard data synchronization
            self.start_dashboard_sync().await?;
        }
        
        // Setup external tracing exporters
        if config.enable_external_tracing {
            self.setup_tracing_exporters().await?;
        }
        
        Ok(())
    }
    
    /// Setup external tracing exporters
    async fn setup_tracing_exporters(&self) -> ObservabilityResult<()> {
        let config = self.config.read()?;
        let mut exporters = self.exporters.write()?;
        
        // OTLP Exporter (OpenTelemetry)
        if config.enable_otlp_exporter {
            let otlp_config = tracing::external::ExternalTracingConfig {
                endpoint_url: config.otlp_endpoint.clone(),
                auth_token: config.otlp_auth_token.clone(),
                service_name: config.service_name.clone(),
                environment: config.environment.clone(),
                ..Default::default()
            };
            let otlp_exporter = tracing::external::OpenTelemetryExporter::new(otlp_config);
            exporters.push(Arc::new(otlp_exporter));
        }
        
        // Jaeger Exporter
        if config.enable_jaeger_exporter {
            let jaeger_config = tracing::external::ExternalTracingConfig {
                endpoint_url: config.jaeger_endpoint.clone(),
                auth_token: config.jaeger_auth_token.clone(),
                service_name: config.service_name.clone(),
                environment: config.environment.clone(),
                ..Default::default()
            };
            let jaeger_exporter = tracing::external::JaegerExporter::new(jaeger_config);
            exporters.push(Arc::new(jaeger_exporter));
        }
        
        Ok(())
    }
    
    /// Start dashboard synchronization
    async fn start_dashboard_sync(&self) -> ObservabilityResult<()> {
        let config = self.config.read()?;
        
        if let Some(ref dashboard_client) = *self.dashboard_client.read()? {
            // Start metrics sync
            self.start_metrics_sync(
                dashboard_client.clone(),
                config.dashboard_metrics_interval,
                config.dashboard_max_metrics_per_batch,
            ).await?;
            
            // Start traces sync
            self.start_traces_sync(
                dashboard_client.clone(),
                config.dashboard_traces_interval,
                config.dashboard_max_traces_per_batch,
            ).await?;
            
            // Start health sync
            self.start_health_sync(
                dashboard_client.clone(),
                config.dashboard_health_interval,
            ).await?;
            
            // Start alerts sync
            self.start_alerts_sync(
                dashboard_client.clone(),
                config.dashboard_alerts_interval,
            ).await?;
        }
        
        Ok(())
    }
    
    /// Start metrics synchronization with dashboard
    async fn start_metrics_sync(
        &self,
        dashboard_client: DashboardClient,
        interval_secs: u64,
        max_batch_size: usize,
    ) -> ObservabilityResult<()> {
        let metrics = self.metrics.clone();
        let event_bus = self.event_bus.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
            
            loop {
                interval.tick().await;
                
                // Collect metrics snapshots
                let snapshots = metrics.collect_snapshots().await.unwrap_or_default();
                
                // Send metrics in batches
                let mut sent = 0;
                while sent < snapshots.len() {
                    let end = std::cmp::min(sent + max_batch_size, snapshots.len());
                    let batch = snapshots[sent..end].to_vec();
                    
                    if let Err(e) = dashboard_client.send_metrics(batch).await {
                        let _ = event_bus.publish(ObservabilityEvent::DashboardSyncError {
                            component: "metrics".to_string(),
                            error: e.to_string(),
                        }).await;
                    }
                    
                    sent = end;
                }
            }
        });
        
        Ok(())
    }
    
    /// Start traces synchronization with dashboard
    async fn start_traces_sync(
        &self,
        dashboard_client: DashboardClient,
        interval_secs: u64,
        max_batch_size: usize,
    ) -> ObservabilityResult<()> {
        let tracer = self.tracer.clone();
        let event_bus = self.event_bus.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
            
            loop {
                interval.tick().await;
                
                // Collect trace snapshots
                let snapshots = tracer.collect_snapshots().await.unwrap_or_default();
                
                // Send traces in batches
                let mut sent = 0;
                while sent < snapshots.len() {
                    let end = std::cmp::min(sent + max_batch_size, snapshots.len());
                    let batch = snapshots[sent..end].to_vec();
                    
                    if let Err(e) = dashboard_client.send_traces(batch).await {
                        let _ = event_bus.publish(ObservabilityEvent::DashboardSyncError {
                            component: "traces".to_string(),
                            error: e.to_string(),
                        }).await;
                    }
                    
                    sent = end;
                }
            }
        });
        
        Ok(())
    }
    
    /// Start health synchronization with dashboard
    async fn start_health_sync(
        &self,
        dashboard_client: DashboardClient,
        interval_secs: u64,
    ) -> ObservabilityResult<()> {
        let health_checker = self.health_checker.clone();
        let event_bus = self.event_bus.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
            
            loop {
                interval.tick().await;
                
                // Get system health report
                if let Ok(health_report) = health_checker.get_system_health().await {
                    if let Err(e) = dashboard_client.send_health_report(health_report).await {
                        let _ = event_bus.publish(ObservabilityEvent::DashboardSyncError {
                            component: "health".to_string(),
                            error: e.to_string(),
                        }).await;
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Start alerts synchronization with dashboard
    async fn start_alerts_sync(
        &self,
        dashboard_client: DashboardClient,
        interval_secs: u64,
    ) -> ObservabilityResult<()> {
        let alert_manager = self.alert_manager.clone();
        let event_bus = self.event_bus.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
            
            loop {
                interval.tick().await;
                
                // Get pending alerts
                if let Ok(alerts) = alert_manager.get_pending_alerts().await {
                    if !alerts.is_empty() {
                        if let Err(e) = dashboard_client.send_alerts(alerts).await {
                            let _ = event_bus.publish(ObservabilityEvent::DashboardSyncError {
                                component: "alerts".to_string(),
                                error: e.to_string(),
                            }).await;
                        }
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Initialize standard components
    async fn initialize_standard_components(&self) -> ObservabilityResult<()> {
        // Register core MCP components
        self.register_standard_components().await?;
        
        // Setup component dependencies
        self.setup_component_dependencies().await?;
        
        Ok(())
    }
    
    /// Register standard MCP components
    async fn register_standard_components(&self) -> ObservabilityResult<()> {
        let mut registry = self.component_registry.write()?;
        
        // Register core components
        registry.register_component(ComponentInfo::new(
            "mcp-core".to_string(),
            "MCP Core".to_string(),
            ComponentType::McpCore,
        ));
        
        registry.register_component(ComponentInfo::new(
            "mcp-protocol".to_string(),
            "MCP Protocol".to_string(),
            ComponentType::McpProtocol,
        ));
        
        registry.register_component(ComponentInfo::new(
            "mcp-transport".to_string(),
            "MCP Transport".to_string(),
            ComponentType::McpTransport,
        ));
        
        Ok(())
    }
    
    /// Setup component dependencies
    async fn setup_component_dependencies(&self) -> ObservabilityResult<()> {
        // Setup health check monitoring
        let health_checker = self.health_checker.clone();
        let component_registry = self.component_registry.clone();
        let event_bus = self.event_bus.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                // Check component health
                if let Ok(registry) = component_registry.read() {
                    for component in registry.get_components() {
                        if let Ok(health_status) = health_checker.check_component_health(&component.id).await {
                            let _ = event_bus.publish(ObservabilityEvent::HealthStatusChanged {
                                component_id: component.id.clone(),
                                old_status: component.health_status.clone(),
                                new_status: health_status.clone(),
                                timestamp: std::time::SystemTime::now(),
                            }).await;
                        }
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Register standard metrics
    fn register_standard_metrics(&self) -> ObservabilityResult<()> {
        self.metrics.register_counter("mcp.messages.processed", "Total messages processed")?;
        self.metrics.register_counter("mcp.messages.failed", "Total messages failed")?;
        self.metrics.register_gauge("mcp.connections.active", "Active connections")?;
        self.metrics.register_histogram("mcp.message.processing_time", "Message processing time")?;
        self.metrics.register_counter("mcp.sessions.created", "Sessions created")?;
        self.metrics.register_counter("mcp.sessions.closed", "Sessions closed")?;
        self.metrics.register_histogram("mcp.session.duration", "Session duration")?;
        
        Ok(())
    }
    
    /// Record a processed message
    pub async fn record_message_processed(&self, message_type: &str, processing_time: f64) -> ObservabilityResult<()> {
        self.metrics.increment_counter("mcp.messages.processed")?;
        self.metrics.record_histogram("mcp.message.processing_time", processing_time)?;
        
        Ok(())
    }
    
    /// Record a failed message
    pub async fn record_message_failed(&self, message_type: &str, error: &str) -> ObservabilityResult<()> {
        self.metrics.increment_counter("mcp.messages.failed")?;
        
        // Publish event
        self.event_bus.publish(ObservabilityEvent::MetricThresholdExceeded {
            metric_name: "mcp.messages.failed".to_string(),
            current_value: 1.0,
            threshold: 0.0,
            timestamp: std::time::SystemTime::now(),
        }).await?;
        
        Ok(())
    }
    
    /// Update connection count
    pub async fn update_connection_count(&self, count: i64) -> ObservabilityResult<()> {
        self.metrics.update_gauge("mcp.connections.active", count as f64)?;
        Ok(())
    }
    
    /// Record a session event
    pub async fn record_session_event(&self, event_type: &str, session_id: &str) -> ObservabilityResult<()> {
        match event_type {
            "created" => self.metrics.increment_counter("mcp.sessions.created")?,
            "closed" => self.metrics.increment_counter("mcp.sessions.closed")?,
            _ => {}
        }
        
        Ok(())
    }
    
    /// Get overall framework status
    pub async fn get_status(&self) -> ObservabilityResult<ObservabilityStatus> {
        let health_status = self.health_checker.get_system_health().await?;
        let metrics_count = self.metrics.get_metric_count().await;
        let active_alerts = self.alert_manager.get_active_alert_count().await;
        let trace_spans_count = self.tracer.get_active_span_count().await;
        let dashboard_connected = self.dashboard_client.read()?.is_some();
        let exporters_count = self.exporters.read()?.len();
        let components_monitored = self.component_registry.read()?.component_count();
        let events_processed = self.event_bus.get_events_processed_count().await;
        let uptime_seconds = self.performance_monitor.get_uptime_seconds().await;
        
        Ok(ObservabilityStatus {
            health_status,
            metrics_count,
            active_alerts,
            trace_spans_count,
            dashboard_connected,
            exporters_count,
            components_monitored,
            events_processed,
            uptime_seconds,
        })
    }
    
    /// Check MCP component health
    pub async fn check_mcp_health(&self, component_id: &str) -> ObservabilityResult<health::HealthStatus> {
        self.health_checker.check_component_health(component_id).await
    }
    
    /// Check if framework is initialized
    pub fn is_initialized(&self) -> bool {
        self.health_checker.is_initialized() && 
        self.metrics.is_initialized() && 
        self.tracer.is_initialized()
    }
    
    /// Shutdown the framework
    pub async fn shutdown(&self) -> ObservabilityResult<()> {
        // Shutdown in reverse order
        if let Some(dashboard_client) = &*self.dashboard_client.read()? {
            dashboard_client.shutdown().await?;
        }
        
        self.event_bus.shutdown().await?;
        self.performance_monitor.shutdown().await?;
        self.alert_manager.shutdown().await?;
        self.health_checker.shutdown().await?;
        self.tracer.shutdown().await?;
        self.metrics.shutdown().await?;
        self.logger.shutdown().await?;
        
        Ok(())
    }
    
    /// Update component health
    pub async fn update_component_health(
        &self,
        component_id: &str,
        status: health::HealthStatus,
        details: Option<String>,
    ) -> Result<(), ObservabilityError> {
        // Update component registry
        if let Ok(mut registry) = self.component_registry.write() {
            registry.update_component_health(component_id, status.clone());
        }
        
        // Update health checker
        self.health_checker.update_component_status(component_id, status.clone(), details).await?;
        
        // Publish event
        self.event_bus.publish(ObservabilityEvent::HealthStatusChanged {
            component_id: component_id.to_string(),
            old_status: health::HealthStatus::Unknown,
            new_status: status,
            timestamp: std::time::SystemTime::now(),
        }).await?;
        
        Ok(())
    }
}

/// Initialize the observability framework with default configuration
pub async fn initialize() -> ObservabilityResult<ObservabilityFramework> {
    ObservabilityFramework::new().await
}

/// Initialize the observability framework with custom configuration
pub async fn initialize_with_config(config: ObservabilityConfig) -> ObservabilityResult<ObservabilityFramework> {
    ObservabilityFramework::new_with_config(config).await
}

// Include example module for documentation
pub mod example;

// Error conversion implementations
impl<T> From<std::sync::PoisonError<std::sync::RwLockReadGuard<'_, T>>> for ObservabilityError {
    fn from(err: std::sync::PoisonError<std::sync::RwLockReadGuard<'_, T>>) -> Self {
        ObservabilityError::LockError(err.to_string())
    }
}

impl<T> From<std::sync::PoisonError<std::sync::RwLockWriteGuard<'_, T>>> for ObservabilityError {
    fn from(err: std::sync::PoisonError<std::sync::RwLockWriteGuard<'_, T>>) -> Self {
        ObservabilityError::LockError(err.to_string())
    }
}

impl<T> From<std::sync::PoisonError<std::sync::MutexGuard<'_, T>>> for ObservabilityError {
    fn from(err: std::sync::PoisonError<std::sync::MutexGuard<'_, T>>) -> Self {
        ObservabilityError::LockError(err.to_string())
    }
} 