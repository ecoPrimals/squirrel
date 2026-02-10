// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Status reporting for observability framework
//!
//! This module provides status structures and reporting
//! for the observability system.

use crate::observability::health::SystemHealthReport;

/// Overall status of the observability framework
#[derive(Debug, Clone)]
pub struct ObservabilityStatus {
    /// Overall health status of the system
    pub health_status: SystemHealthReport,
    /// Number of metrics being collected
    pub metrics_count: usize,
    /// Number of active alerts
    pub active_alerts: usize,
    /// Number of active trace spans
    pub trace_spans_count: usize,
    /// Whether dashboard is connected
    pub dashboard_connected: bool,
    /// Number of active exporters
    pub exporters_count: usize,
    /// Number of components being monitored
    pub components_monitored: usize,
    /// Total number of events processed
    pub events_processed: u64,
    /// System uptime in seconds
    pub uptime_seconds: u64,
}

impl ObservabilityStatus {
    /// Create a new observability status
    pub fn new() -> Self {
        Self {
            health_status: SystemHealthReport::new(),
            metrics_count: 0,
            active_alerts: 0,
            trace_spans_count: 0,
            dashboard_connected: false,
            exporters_count: 0,
            components_monitored: 0,
            events_processed: 0,
            uptime_seconds: 0,
        }
    }

    /// Check if the observability system is healthy
    pub fn is_healthy(&self) -> bool {
        self.health_status.is_healthy()
    }

    /// Get a summary of the status
    pub fn summary(&self) -> String {
        format!(
            "Observability Status: {} components, {} metrics, {} alerts, {} events processed, uptime: {}s",
            self.components_monitored,
            self.metrics_count,
            self.active_alerts,
            self.events_processed,
            self.uptime_seconds
        )
    }

    /// Get status as JSON value
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "health_status": self.health_status.to_json(),
            "metrics_count": self.metrics_count,
            "active_alerts": self.active_alerts,
            "trace_spans_count": self.trace_spans_count,
            "dashboard_connected": self.dashboard_connected,
            "exporters_count": self.exporters_count,
            "components_monitored": self.components_monitored,
            "events_processed": self.events_processed,
            "uptime_seconds": self.uptime_seconds
        })
    }

    /// Update metrics count
    pub fn update_metrics_count(&mut self, count: usize) {
        self.metrics_count = count;
    }

    /// Update active alerts count
    pub fn update_active_alerts(&mut self, count: usize) {
        self.active_alerts = count;
    }

    /// Update trace spans count
    pub fn update_trace_spans_count(&mut self, count: usize) {
        self.trace_spans_count = count;
    }

    /// Update dashboard connection status
    pub fn update_dashboard_connected(&mut self, connected: bool) {
        self.dashboard_connected = connected;
    }

    /// Update exporters count
    pub fn update_exporters_count(&mut self, count: usize) {
        self.exporters_count = count;
    }

    /// Update components monitored count
    pub fn update_components_monitored(&mut self, count: usize) {
        self.components_monitored = count;
    }

    /// Update events processed count
    pub fn update_events_processed(&mut self, count: u64) {
        self.events_processed = count;
    }

    /// Update uptime
    pub fn update_uptime(&mut self, seconds: u64) {
        self.uptime_seconds = seconds;
    }

    /// Update health status
    pub fn update_health_status(&mut self, health_status: SystemHealthReport) {
        self.health_status = health_status;
    }
} 