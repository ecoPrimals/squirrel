// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Security monitoring statistics
#![allow(dead_code)] // Security monitoring infrastructure awaiting activation
//!
//! Tracks and reports statistics about security events, alerts, and
//! monitoring system health.

use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use super::types::SecurityEventType;

/// Security monitoring statistics
///
/// Real-time statistics about the security monitoring system's operation
/// and the security events it has processed.
#[derive(Debug, Clone, Serialize)]
pub struct SecurityMonitoringStats {
    /// Total events processed
    pub total_events: u64,

    /// Total alerts generated
    pub alerts_generated: u64,

    /// Currently active threats
    pub active_threats: u32,

    /// Number of monitored clients
    pub monitored_clients: u32,

    /// Events processed per second
    pub events_per_second: f64,

    /// Alert generation rate (alerts per hour)
    pub alert_rate: f64,

    /// System uptime
    pub uptime: Duration,

    /// Event counts by type
    pub event_types: HashMap<String, u64>,
}

impl Default for SecurityMonitoringStats {
    fn default() -> Self {
        Self {
            total_events: 0,
            alerts_generated: 0,
            active_threats: 0,
            monitored_clients: 0,
            events_per_second: 0.0,
            alert_rate: 0.0,
            uptime: Duration::from_secs(0),
            event_types: HashMap::new(),
        }
    }
}

/// Statistics collector for security monitoring
///
/// Thread-safe statistics collection and calculation.
pub struct StatsCollector {
    stats: Arc<RwLock<SecurityMonitoringStats>>,
    start_time: Instant,
    last_calculation: Arc<RwLock<Instant>>,
}

impl StatsCollector {
    /// Create a new statistics collector
    pub fn new() -> Self {
        Self {
            stats: Arc::new(RwLock::new(SecurityMonitoringStats::default())),
            start_time: Instant::now(),
            last_calculation: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Record a new event
    pub async fn record_event(&self, event_type: &SecurityEventType) {
        let mut stats = self.stats.write().await;
        stats.total_events += 1;

        // Update event type counter
        let type_name = match event_type {
            SecurityEventType::Authentication { .. } => "authentication",
            SecurityEventType::Authorization { .. } => "authorization",
            SecurityEventType::RateLimitViolation { .. } => "rate_limit_violation",
            SecurityEventType::InputValidationViolation { .. } => "input_validation_violation",
            SecurityEventType::SuspiciousActivity { .. } => "suspicious_activity",
            SecurityEventType::PolicyViolation { .. } => "policy_violation",
            SecurityEventType::SystemAccess { .. } => "system_access",
        };

        *stats.event_types.entry(type_name.to_string()).or_insert(0) += 1;
    }

    /// Record a new alert
    pub async fn record_alert(&self) {
        let mut stats = self.stats.write().await;
        stats.alerts_generated += 1;
    }

    /// Update active threats count
    pub async fn set_active_threats(&self, count: u32) {
        let mut stats = self.stats.write().await;
        stats.active_threats = count;
    }

    /// Update monitored clients count
    pub async fn set_monitored_clients(&self, count: u32) {
        let mut stats = self.stats.write().await;
        stats.monitored_clients = count;
    }

    /// Calculate derived statistics (rates, etc.)
    ///
    /// Always recalculates from current time. Safe to call at any frequency.
    pub async fn calculate_derived_stats(&self) {
        let now = Instant::now();
        let mut last_calc = self.last_calculation.write().await;
        let mut stats = self.stats.write().await;

        // Update uptime
        stats.uptime = now.duration_since(self.start_time);

        // Calculate events per second (use fractional seconds for sub-second precision)
        let uptime_secs = stats.uptime.as_secs_f64();
        if uptime_secs > 0.0 {
            stats.events_per_second = stats.total_events as f64 / uptime_secs;
            stats.alert_rate = (stats.alerts_generated as f64 / uptime_secs) * 3600.0;
        }

        *last_calc = now;
    }

    /// Get current statistics
    pub async fn get_stats(&self) -> SecurityMonitoringStats {
        self.stats.read().await.clone()
    }

    /// Reset statistics
    pub async fn reset(&self) {
        let mut stats = self.stats.write().await;
        *stats = SecurityMonitoringStats::default();
    }
}

impl Default for StatsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::monitoring::types::SecurityEventType;

    #[tokio::test]
    async fn test_stats_collector_creation() {
        let collector = StatsCollector::new();
        let stats = collector.get_stats().await;

        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.alerts_generated, 0);
        assert_eq!(stats.active_threats, 0);
    }

    #[tokio::test]
    async fn test_record_event() {
        let collector = StatsCollector::new();

        collector
            .record_event(&SecurityEventType::Authentication {
                success: true,
                user_id: Some("user123".to_string()),
                method: "password".to_string(),
            })
            .await;

        let stats = collector.get_stats().await;
        assert_eq!(stats.total_events, 1);
        assert_eq!(stats.event_types.get("authentication"), Some(&1));
    }

    #[tokio::test]
    async fn test_record_multiple_events() {
        let collector = StatsCollector::new();

        // Record different event types
        collector
            .record_event(&SecurityEventType::Authentication {
                success: false,
                user_id: None,
                method: "password".to_string(),
            })
            .await;

        collector
            .record_event(&SecurityEventType::RateLimitViolation {
                client_ip: "192.168.1.1".to_string(),
                endpoint: "/api/data".to_string(),
                violation_count: 5,
            })
            .await;

        collector
            .record_event(&SecurityEventType::Authentication {
                success: true,
                user_id: Some("user456".to_string()),
                method: "oauth".to_string(),
            })
            .await;

        let stats = collector.get_stats().await;
        assert_eq!(stats.total_events, 3);
        assert_eq!(stats.event_types.get("authentication"), Some(&2));
        assert_eq!(stats.event_types.get("rate_limit_violation"), Some(&1));
    }

    #[tokio::test]
    async fn test_record_alert() {
        let collector = StatsCollector::new();

        collector.record_alert().await;
        collector.record_alert().await;

        let stats = collector.get_stats().await;
        assert_eq!(stats.alerts_generated, 2);
    }

    #[tokio::test]
    async fn test_update_counters() {
        let collector = StatsCollector::new();

        collector.set_active_threats(5).await;
        collector.set_monitored_clients(100).await;

        let stats = collector.get_stats().await;
        assert_eq!(stats.active_threats, 5);
        assert_eq!(stats.monitored_clients, 100);
    }

    #[tokio::test]
    async fn test_calculate_derived_stats() {
        let collector = StatsCollector::new();

        // Record some events
        for _ in 0..10 {
            collector
                .record_event(&SecurityEventType::Authentication {
                    success: true,
                    user_id: Some("user".to_string()),
                    method: "password".to_string(),
                })
                .await;
        }

        // Minimal wait for clock to advance past zero
        tokio::time::sleep(Duration::from_millis(10)).await;

        collector.calculate_derived_stats().await;

        let stats = collector.get_stats().await;
        // Uptime should be measurably > 0; events_per_second is events/uptime_secs
        assert!(
            stats.uptime > Duration::ZERO,
            "uptime should be > 0, got {:?}",
            stats.uptime
        );
        assert!(stats.events_per_second > 0.0);
    }

    #[tokio::test]
    async fn test_stats_reset() {
        let collector = StatsCollector::new();

        collector
            .record_event(&SecurityEventType::Authentication {
                success: true,
                user_id: Some("user".to_string()),
                method: "password".to_string(),
            })
            .await;
        collector.record_alert().await;
        collector.set_active_threats(3).await;

        collector.reset().await;

        let stats = collector.get_stats().await;
        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.alerts_generated, 0);
        assert_eq!(stats.active_threats, 0);
        assert!(stats.event_types.is_empty());
    }
}
