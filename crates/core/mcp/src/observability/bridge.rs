// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Monitoring bridge for observability framework
//!
//! This module provides a bridge between the observability framework
//! and MCP-specific monitoring components.

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::observability::health::HealthStatus;
use crate::observability::metrics::McpMetrics;

/// Bridge for MCP-specific monitoring integration
pub struct MonitoringBridge {
    metrics: Arc<McpMetrics>,
    health_checker: Arc<RwLock<HealthChecker>>,
    alert_manager: Arc<AlertManager>,
    distributed_tracer: Arc<DistributedTracer>,
}

impl MonitoringBridge {
    /// Create a new monitoring bridge
    pub fn new(
        metrics: Arc<McpMetrics>,
        health_checker: Arc<RwLock<HealthChecker>>,
        alert_manager: Arc<AlertManager>,
        distributed_tracer: Arc<DistributedTracer>,
    ) -> Self {
        Self {
            metrics,
            health_checker,
            alert_manager,
            distributed_tracer,
        }
    }

    /// Track a connection event
    pub async fn track_connection(&self, connection_id: &str, event: ConnectionEvent) {
        match event {
            ConnectionEvent::Connected => {
                self.metrics.increment_counter("connections.established").await;
                self.metrics.set_gauge("connections.active", 1.0).await;
                
                let span = self.distributed_tracer.start_span(
                    "connection.established",
                    &[("connection_id", connection_id)]
                );
                
                // Connection established successfully
                span.finish();
            }
            ConnectionEvent::Disconnected => {
                self.metrics.increment_counter("connections.closed").await;
                self.metrics.set_gauge("connections.active", -1.0).await;
                
                let span = self.distributed_tracer.start_span(
                    "connection.closed",
                    &[("connection_id", connection_id)]
                );
                
                // Connection closed
                span.finish();
            }
            ConnectionEvent::Error(error) => {
                self.metrics.increment_counter("connections.errors").await;
                
                let span = self.distributed_tracer.start_span(
                    "connection.error",
                    &[("connection_id", connection_id), ("error", &error)]
                );
                
                // Send alert for connection error
                self.alert_manager.send_alert(Alert::ConnectionError {
                    connection_id: connection_id.to_string(),
                    error: error.clone(),
                }).await;
                
                span.finish();
            }
        }
    }

    /// Track command execution
    pub async fn track_command_execution(&self, command: &str, duration: f64, success: bool) {
        self.metrics.increment_counter("commands.executed").await;
        self.metrics.record_histogram("commands.duration", duration).await;
        
        if success {
            self.metrics.increment_counter("commands.success").await;
        } else {
            self.metrics.increment_counter("commands.failed").await;
        }
        
        let span = self.distributed_tracer.start_span(
            "command.execution",
            &[
                ("command", command),
                ("success", &success.to_string()),
                ("duration", &duration.to_string())
            ]
        );
        
        // Command execution completed
        span.finish();
    }

    /// Track resource access
    pub async fn track_resource_access(&self, resource_uri: &str, access_type: ResourceAccessType) {
        let access_type_str = match access_type {
            ResourceAccessType::Read => "read",
            ResourceAccessType::Write => "write",
            ResourceAccessType::Delete => "delete",
        };
        
        self.metrics.increment_counter(&format!("resources.{}", access_type_str)).await;
        
        let span = self.distributed_tracer.start_span(
            "resource.access",
            &[
                ("resource_uri", resource_uri),
                ("access_type", access_type_str)
            ]
        );
        
        // Resource access completed
        span.finish();
    }

    /// Update health status
    pub async fn update_health_status(&self, component: &str, status: HealthStatus) {
        if let Ok(mut health_checker) = self.health_checker.write().await {
            health_checker.update_component_health(component, status.clone()).await;
        }
        
        // Send alert if component becomes unhealthy
        if matches!(status, HealthStatus::Unhealthy(_)) {
            self.alert_manager.send_alert(Alert::HealthCheck {
                component: component.to_string(),
                reason: format!("Component {} became unhealthy", component),
            }).await;
        }
    }
}

/// Connection events for tracking
#[derive(Debug, Clone)]
pub enum ConnectionEvent {
    Connected,
    Disconnected,
    Error(String),
}

/// Resource access types for tracking
#[derive(Debug, Clone)]
pub enum ResourceAccessType {
    Read,
    Write,
    Delete,
}

/// Alert types for the monitoring bridge
#[derive(Debug, Clone)]
pub enum Alert {
    ConnectionError {
        connection_id: String,
        error: String,
    },
    HealthCheck {
        component: String,
        reason: String,
    },
}

/// Alert manager for handling alerts
pub struct AlertManager {
    // Implementation for alert management
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new() -> Self {
        Self {}
    }

    /// Send an alert
    pub async fn send_alert(&self, alert: Alert) {
        // Implementation would send alert to configured destinations
        match alert {
            Alert::ConnectionError { connection_id, error } => {
                tracing::error!("Connection error for {}: {}", connection_id, error);
            }
            Alert::HealthCheck { component, reason } => {
                tracing::warn!("Health check alert for {}: {}", component, reason);
            }
        }
    }
}

/// Distributed tracer for tracking spans
pub struct DistributedTracer {
    // Implementation for distributed tracing
}

impl DistributedTracer {
    /// Create a new distributed tracer
    pub fn new() -> Self {
        Self {}
    }

    /// Start a new span
    pub fn start_span(&self, operation: &str, tags: &[(&str, &str)]) -> Span {
        let span_id = uuid::Uuid::new_v4().to_string();
        
        tracing::info!("Starting span: {} [{}]", operation, span_id);
        for (key, value) in tags {
            tracing::info!("  {}: {}", key, value);
        }
        
        Span { id: span_id }
    }
}

/// Span for tracking operations
pub struct Span {
    id: String,
}

impl Span {
    /// Finish the span
    pub fn finish(self) {
        tracing::info!("Finished span: {}", self.id);
    }
}

/// Health checker for component health tracking
pub struct HealthChecker {
    // Implementation for health checking
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new() -> Self {
        Self {}
    }

    /// Update component health
    pub async fn update_component_health(&mut self, component: &str, status: HealthStatus) {
        tracing::info!("Health status for {}: {:?}", component, status);
    }
} 