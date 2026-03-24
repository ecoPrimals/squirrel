// SPDX-License-Identifier: AGPL-3.0-or-later
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
            "info" => tracing::info!("📊 Event: {}", Self::format_event(event)),
            "warn" => tracing::warn!("📊 Event: {}", Self::format_event(event)),
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
                    Self::format_performance(metrics)
                ),
                _ => {}
            }
        }
    }

    fn format_event(event: &MonitoringEvent) -> String {
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

    fn format_performance(metrics: &PerformanceMetrics) -> String {
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

#[cfg(test)]
mod tests {
    use super::FallbackLogger;
    use crate::HealthStatus;
    use crate::monitoring::config::FallbackConfig;
    use crate::monitoring::types::{Metric, MetricValue, MonitoringEvent, PerformanceMetrics};
    use chrono::Utc;

    fn logger(cfg: FallbackConfig) -> FallbackLogger {
        FallbackLogger::new(cfg)
    }

    #[test]
    fn log_event_respects_level_and_formats_variants() {
        let l = logger(FallbackConfig {
            log_level: "debug".to_string(),
            include_metrics: true,
            include_health: true,
            include_performance: true,
        });
        l.log_event(&MonitoringEvent::ServiceStarted {
            service: "s".to_string(),
            version: "1".to_string(),
            timestamp: Utc::now(),
        });
        l.log_event(&MonitoringEvent::TaskCompleted {
            task_id: "t".to_string(),
            execution_time: std::time::Duration::from_millis(1),
            success: true,
            timestamp: Utc::now(),
        });
        l.log_event(&MonitoringEvent::TaskCompleted {
            task_id: "t2".to_string(),
            execution_time: std::time::Duration::from_millis(1),
            success: false,
            timestamp: Utc::now(),
        });
        l.log_event(&MonitoringEvent::ErrorOccurred {
            error_type: "E".to_string(),
            error_message: "m".to_string(),
            component: "c".to_string(),
            timestamp: Utc::now(),
        });
        l.log_event(&MonitoringEvent::Custom {
            event_type: "custom".to_string(),
            data: serde_json::json!({}),
            timestamp: Utc::now(),
        });

        let l_info = logger(FallbackConfig {
            log_level: "info".to_string(),
            include_metrics: true,
            include_health: true,
            include_performance: true,
        });
        l_info.log_event(&MonitoringEvent::ServiceStopped {
            service: "x".to_string(),
            timestamp: Utc::now(),
        });

        let l_warn = logger(FallbackConfig {
            log_level: "warn".to_string(),
            include_metrics: true,
            include_health: true,
            include_performance: true,
        });
        l_warn.log_event(&MonitoringEvent::ServiceStarted {
            service: "y".to_string(),
            version: "0".to_string(),
            timestamp: Utc::now(),
        });

        let l_silent = logger(FallbackConfig {
            log_level: "error".to_string(),
            include_metrics: true,
            include_health: true,
            include_performance: true,
        });
        l_silent.log_event(&MonitoringEvent::ServiceStarted {
            service: "z".to_string(),
            version: "0".to_string(),
            timestamp: Utc::now(),
        });
    }

    #[test]
    fn log_metric_branches() {
        let m = Metric {
            name: "n".to_string(),
            value: MetricValue::Gauge(1.0),
            labels: std::collections::HashMap::new(),
            timestamp: Utc::now(),
        };
        let on = logger(FallbackConfig {
            log_level: "debug".to_string(),
            include_metrics: true,
            include_health: true,
            include_performance: true,
        });
        on.log_metric(&m);

        let off = logger(FallbackConfig {
            log_level: "debug".to_string(),
            include_metrics: false,
            include_health: true,
            include_performance: true,
        });
        off.log_metric(&m);
    }

    #[test]
    fn log_health_and_performance_branches() {
        let base = FallbackConfig {
            log_level: "info".to_string(),
            include_metrics: true,
            include_health: true,
            include_performance: true,
        };
        let l = logger(base.clone());
        l.log_health("c", &HealthStatus::Healthy);

        let lw = logger(FallbackConfig {
            log_level: "warn".to_string(),
            include_metrics: base.include_metrics,
            include_health: base.include_health,
            include_performance: base.include_performance,
        });
        lw.log_health("c", &HealthStatus::Degraded);

        let no_health = logger(FallbackConfig {
            log_level: base.log_level.clone(),
            include_metrics: base.include_metrics,
            include_health: false,
            include_performance: base.include_performance,
        });
        no_health.log_health("c", &HealthStatus::Unhealthy);

        let perf = PerformanceMetrics {
            cpu_usage: Some(0.5),
            memory_usage: Some(0.25),
            network_usage: None,
            response_time: Some(std::time::Duration::from_millis(12)),
            throughput: None,
            error_rate: None,
            queue_length: None,
            active_connections: None,
            custom_metrics: std::collections::HashMap::new(),
        };
        let l = logger(base.clone());
        l.log_performance("cpu", &perf);

        let no_perf = logger(FallbackConfig {
            include_performance: false,
            ..base
        });
        no_perf.log_performance("cpu", &perf);
    }

    #[test]
    fn fallback_config_default() {
        let d = FallbackConfig::default();
        assert_eq!(d.log_level, "info");
        assert!(d.include_metrics);
    }
}
