// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! # Alerts Module
//!
//! This module provides alerting capabilities for the monitoring system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::metrics::MetricsCollector;
use crate::error::PrimalError;

/// Alert management system
pub struct AlertManager {
    /// Active alerts
    active_alerts: Arc<RwLock<HashMap<String, Alert>>>,
    /// Alert history
    alert_history: Arc<RwLock<Vec<Alert>>>,
}

/// Alert information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Alert ID
    pub id: String,
    /// Alert name
    pub name: String,
    /// Alert message
    pub message: String,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert timestamp
    pub timestamp: DateTime<Utc>,
    /// Alert status
    pub status: AlertStatus,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Critical severity requiring immediate attention.
    Critical,
    /// High severity.
    High,
    /// Medium severity.
    Medium,
    /// Low severity.
    Low,
    /// Informational only.
    Info,
}

/// Alert status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertStatus {
    /// Alert is active and unaddressed.
    Active,
    /// Alert has been resolved.
    Resolved,
    /// Alert has been acknowledged but may still need attention.
    Acknowledged,
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AlertManager {
    /// Create a new alert manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Evaluate alerts based on current metrics
    pub async fn evaluate_alerts(
        &self,
        _metrics_collector: &MetricsCollector,
    ) -> Result<(), PrimalError> {
        // Stub implementation
        Ok(())
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Result<Vec<Alert>, PrimalError> {
        let alerts = self.active_alerts.read().await;
        Ok(alerts.values().cloned().collect())
    }

    /// Create an alert
    pub async fn create_alert(&self, alert: Alert) -> Result<(), PrimalError> {
        let mut alerts = self.active_alerts.write().await;
        alerts.insert(alert.id.clone(), alert);
        Ok(())
    }

    /// Resolve an alert
    pub async fn resolve_alert(&self, alert_id: &str) -> Result<(), PrimalError> {
        let mut alerts = self.active_alerts.write().await;
        if let Some(mut alert) = alerts.remove(alert_id) {
            alert.status = AlertStatus::Resolved;
            let mut history = self.alert_history.write().await;
            history.push(alert);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::metrics::MetricsCollector;
    use chrono::Utc;

    fn sample_alert(id: &str) -> Alert {
        Alert {
            id: id.to_string(),
            name: "n".to_string(),
            message: "m".to_string(),
            severity: AlertSeverity::High,
            timestamp: Utc::now(),
            status: AlertStatus::Active,
        }
    }

    #[tokio::test]
    async fn alert_manager_default_equals_new() {
        let a = AlertManager::default();
        let b = AlertManager::new();
        let ea = a.get_active_alerts().await.unwrap();
        let eb = b.get_active_alerts().await.unwrap();
        assert_eq!(ea.len(), eb.len());
    }

    #[tokio::test]
    async fn create_get_resolve_alert_roundtrip() {
        let mgr = AlertManager::new();
        mgr.evaluate_alerts(&MetricsCollector::default())
            .await
            .unwrap();
        mgr.create_alert(sample_alert("a1")).await.unwrap();
        let active = mgr.get_active_alerts().await.unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, "a1");
        mgr.resolve_alert("a1").await.unwrap();
        assert!(mgr.get_active_alerts().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn resolve_unknown_alert_is_ok() {
        let mgr = AlertManager::new();
        mgr.resolve_alert("missing").await.unwrap();
    }

    #[test]
    fn alert_severity_status_serde_roundtrip() {
        let a = Alert {
            id: "i".into(),
            name: "n".into(),
            message: "m".into(),
            severity: AlertSeverity::Info,
            timestamp: Utc::now(),
            status: AlertStatus::Acknowledged,
        };
        let json = serde_json::to_string(&a).unwrap();
        let back: Alert = serde_json::from_str(&json).unwrap();
        assert!(matches!(back.severity, AlertSeverity::Info));
        assert!(matches!(back.status, AlertStatus::Acknowledged));
    }
}
