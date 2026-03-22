// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Tracing-based fallback when no external monitoring provider is registered.

use crate::HealthStatus;

use super::config::FallbackConfig;
use super::types::{Metric, MonitoringEvent, PerformanceMetrics};

/// Fallback logger for when no monitoring providers are available
pub struct FallbackLogger {
    pub(super) config: FallbackConfig,
}

impl FallbackLogger {
    /// Creates a new fallback logger with the given config.
    #[must_use]
    pub const fn new(config: FallbackConfig) -> Self {
        Self { config }
    }

    /// Logs an event to tracing when no provider is available.
    pub fn log_event(&self, event: &MonitoringEvent) {
        match self.config.log_level.as_str() {
            "debug" => tracing::debug!("📊 Event: {:?}", event),
            "info" => tracing::info!("📊 Event: {}", self.format_event(event)),
            "warn" => tracing::warn!("📊 Event: {}", self.format_event(event)),
            _ => {}
        }
    }

    /// Logs a metric to tracing when no provider is available.
    pub fn log_metric(&self, metric: &Metric) {
        if self.config.include_metrics {
            match self.config.log_level.as_str() {
                "debug" => tracing::debug!("📈 Metric: {:?}", metric),
                "info" => tracing::info!("📈 Metric: {} = {:?}", metric.name, metric.value),
                _ => {}
            }
        }
    }

    /// Logs health status to tracing when no provider is available.
    pub fn log_health(&self, component: &str, health: &HealthStatus) {
        if self.config.include_health {
            match self.config.log_level.as_str() {
                "debug" => tracing::debug!("🏥 Health: {} = {:?}", component, health),
                "info" => tracing::info!("🏥 Health: {} = {:?}", component, health),
                "warn" if matches!(health, HealthStatus::Degraded | HealthStatus::Unhealthy) => {
                    tracing::warn!("🏥 Health: {} = {:?}", component, health);
                }
                _ => {}
            }
        }
    }

    /// Logs performance metrics to tracing when no provider is available.
    pub fn log_performance(&self, component: &str, metrics: &PerformanceMetrics) {
        if self.config.include_performance {
            match self.config.log_level.as_str() {
                "debug" => tracing::debug!("⚡ Performance: {} = {:?}", component, metrics),
                "info" => tracing::info!(
                    "⚡ Performance: {} = {}",
                    component,
                    self.format_performance(metrics)
                ),
                _ => {}
            }
        }
    }

    fn format_event(&self, event: &MonitoringEvent) -> String {
        match event {
            MonitoringEvent::ServiceStarted { service, .. } => {
                format!("Service {service} started")
            }
            MonitoringEvent::ServiceStopped { service, .. } => {
                format!("Service {service} stopped")
            }
            MonitoringEvent::TaskCompleted {
                task_id, success, ..
            } => {
                format!(
                    "Task {} {}",
                    task_id,
                    if *success { "completed" } else { "failed" }
                )
            }
            MonitoringEvent::ErrorOccurred {
                error_type,
                component,
                ..
            } => {
                format!("Error in {component}: {error_type}")
            }
            _ => "Event occurred".to_string(),
        }
    }

    fn format_performance(&self, metrics: &PerformanceMetrics) -> String {
        let mut parts = Vec::new();

        if let Some(cpu) = metrics.cpu_usage {
            parts.push(format!("CPU: {cpu:.1}%"));
        }
        if let Some(memory) = metrics.memory_usage {
            parts.push(format!("Memory: {memory:.1}%"));
        }
        if let Some(response_time) = metrics.response_time {
            parts.push(format!("Response: {response_time:?}"));
        }

        parts.join(", ")
    }
}
