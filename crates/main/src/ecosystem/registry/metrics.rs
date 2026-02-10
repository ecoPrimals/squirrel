// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Metrics collection operations for the ecosystem registry manager

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, error};

use super::types::{DiscoveredService, ServiceHealthStatus};
use crate::monitoring::metrics::MetricsCollector;

/// Metrics collection operations
pub struct MetricsOps;

impl MetricsOps {
    /// Collect metrics for ecosystem monitoring
    pub async fn collect_metrics(
        service_registry: &Arc<RwLock<HashMap<String, DiscoveredService>>>,
        metrics_collector: &Arc<MetricsCollector>,
    ) {
        let registry = service_registry.read().await;

        // Collect service count metrics
        metrics_collector
            .record_metric(
                "ecosystem_services_discovered",
                registry.len() as f64,
                HashMap::new(),
            )
            .await
            .unwrap_or_else(|e| {
                error!("Failed to record metric: {}", e);
            });

        // Record healthy services count
        let healthy_count = registry
            .values()
            .filter(|s| s.health_status == ServiceHealthStatus::Healthy)
            .count();
        if healthy_count > 0 {
            metrics_collector
                .record_metric(
                    "ecosystem_healthy_services",
                    healthy_count as f64,
                    HashMap::new(),
                )
                .await
                .unwrap_or_else(|e| {
                    error!("Failed to record healthy services metric: {}", e);
                });
        }

        // Record service types
        for service in registry.values() {
            let service_type_string = format!("{:?}", service.primal_type);
            metrics_collector
                .record_metric(
                    "ecosystem_service_types",
                    1.0,
                    [("service_type".to_string(), service_type_string)]
                        .iter()
                        .cloned()
                        .collect(),
                )
                .await
                .unwrap_or_else(|e| {
                    error!("Failed to record service type metric: {}", e);
                });
        }
    }

    /// Start metrics collection background task
    pub async fn start_metrics_task(
        service_registry: Arc<RwLock<HashMap<String, DiscoveredService>>>,
        metrics_collector: Arc<MetricsCollector>,
        metrics_interval: std::time::Duration,
        shutdown_token: tokio_util::sync::CancellationToken,
    ) {
        let mut interval = interval(metrics_interval);

        loop {
            tokio::select! {
                () = shutdown_token.cancelled() => {
                    debug!("Metrics task shutting down");
                    break;
                }
                _ = interval.tick() => {
                    Self::collect_metrics(&service_registry, &metrics_collector).await;
                }
            }
        }
    }

    /// Get service statistics
    pub async fn get_service_stats(
        service_registry: &Arc<RwLock<HashMap<String, DiscoveredService>>>,
    ) -> ServiceStats {
        let registry = service_registry.read().await;

        let total_services = registry.len();
        let healthy_services = registry
            .values()
            .filter(|s| s.health_status == ServiceHealthStatus::Healthy)
            .count();
        let unhealthy_services = registry
            .values()
            .filter(|s| s.health_status == ServiceHealthStatus::Unhealthy)
            .count();
        let unknown_services = registry
            .values()
            .filter(|s| s.health_status == ServiceHealthStatus::Unknown)
            .count();

        let service_types = registry
            .values()
            .map(|s| format!("{:?}", s.primal_type))
            .collect::<std::collections::HashSet<_>>()
            .len();

        ServiceStats {
            total_services,
            healthy_services,
            unhealthy_services,
            unknown_services,
            service_types,
        }
    }

    /// Record service registration event
    pub async fn record_service_registration(
        metrics_collector: &Arc<MetricsCollector>,
        service_id: &str,
        primal_type: &str,
    ) {
        let labels = [
            ("service_id".to_string(), service_id.to_string()),
            ("primal_type".to_string(), primal_type.to_string()),
        ]
        .iter()
        .cloned()
        .collect();

        metrics_collector
            .record_metric("ecosystem_service_registered", 1.0, labels)
            .await
            .unwrap_or_else(|e| {
                error!("Failed to record service registration metric: {}", e);
            });
    }

    /// Record service health change event
    pub async fn record_health_change(
        metrics_collector: &Arc<MetricsCollector>,
        service_id: &str,
        old_status: &ServiceHealthStatus,
        new_status: &ServiceHealthStatus,
    ) {
        let labels = [
            ("service_id".to_string(), service_id.to_string()),
            ("old_status".to_string(), format!("{old_status:?}")),
            ("new_status".to_string(), format!("{new_status:?}")),
        ]
        .iter()
        .cloned()
        .collect();

        metrics_collector
            .record_metric("ecosystem_service_health_changed", 1.0, labels)
            .await
            .unwrap_or_else(|e| {
                error!("Failed to record health change metric: {}", e);
            });
    }
}

/// Service statistics
#[derive(Debug, Clone)]
pub struct ServiceStats {
    pub total_services: usize,
    pub healthy_services: usize,
    pub unhealthy_services: usize,
    pub unknown_services: usize,
    pub service_types: usize,
}
