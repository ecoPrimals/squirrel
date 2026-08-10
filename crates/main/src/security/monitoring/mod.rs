// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! # Production Security Monitoring
//!
//! This module provides security event collection, logging, and alerting via
//! delegation to the defense capability provider (skunkBat):
//!
//! - Security event collection and passive logging
//! - Threat classification delegated via IPC (`defense.classify_threat`)
//! - Behavioral anomaly detection delegated via IPC (`defense.detect_anomaly`)
//! - Security metrics and alert forwarding from defense responses
//!
//! Squirrel **observes and logs** suspicious activity but does **not** perform
//! local threat detection or classification — that is skunkBat's domain.

mod alerts;
mod config;
mod defense_client;
mod stats;
mod types;

// Re-export public API
pub use alerts::{AlertBuilder, AlertType, SecurityAlert};
pub use config::{AlertThresholds, SecurityMonitoringConfig};
pub use defense_client::DefenseClient;
pub use stats::SecurityMonitoringStats;
pub use types::{EventSeverity, SecurityEvent, SecurityEventType};

// Internal types
use types::BehavioralPattern;

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::{Mutex, RwLock, mpsc};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::error::PrimalError;
use crate::observability::CorrelationId;
use crate::shutdown::{ShutdownHandler, ShutdownPhase};

/// Security monitoring system
///
/// Collects and logs security events locally, delegating threat classification
/// and anomaly detection to the defense capability provider via IPC.
pub struct SecurityMonitoringSystem {
    /// Configuration
    config: SecurityMonitoringConfig,

    /// IPC client for defense capability delegation
    defense_client: DefenseClient,

    /// Event buffer for batching
    event_buffer: Arc<Mutex<Vec<SecurityEvent>>>,

    /// Historical events (limited retention)
    event_history: Arc<RwLock<VecDeque<SecurityEvent>>>,

    /// Active alerts
    active_alerts: Arc<RwLock<HashMap<Uuid, SecurityAlert>>>,

    /// Behavioral patterns for anomaly detection
    behavioral_patterns: Arc<RwLock<HashMap<String, BehavioralPattern>>>,

    /// Event channel for real-time processing
    event_sender: mpsc::UnboundedSender<SecurityEvent>,
    event_receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<SecurityEvent>>>>,

    /// Background task handles
    background_tasks: Arc<Mutex<Vec<JoinHandle<()>>>>,

    /// Monitoring statistics
    stats: Arc<RwLock<SecurityMonitoringStats>>,

    /// Shutdown flag
    shutdown_requested: Arc<RwLock<bool>>,
}

impl SecurityMonitoringSystem {
    /// Create a new security monitoring system
    #[must_use]
    pub fn new(config: SecurityMonitoringConfig) -> Self {
        Self::with_defense_client(config, DefenseClient::new())
    }

    /// Create with an explicit defense client (for testing without live I/O).
    #[cfg(test)]
    #[must_use]
    pub(crate) fn new_without_discovery(config: SecurityMonitoringConfig) -> Self {
        Self::with_defense_client(config, DefenseClient::without_discovery())
    }

    fn with_defense_client(config: SecurityMonitoringConfig, defense_client: DefenseClient) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        Self {
            config,
            defense_client,
            event_buffer: Arc::new(Mutex::new(Vec::new())),
            event_history: Arc::new(RwLock::new(VecDeque::new())),
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            behavioral_patterns: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            event_receiver: Arc::new(Mutex::new(Some(event_receiver))),
            background_tasks: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(RwLock::new(SecurityMonitoringStats {
                total_events: 0,
                alerts_generated: 0,
                active_threats: 0,
                monitored_clients: 0,
                events_per_second: 0.0,
                alert_rate: 0.0,
                uptime: Duration::from_secs(0),
                event_types: HashMap::new(),
            })),
            shutdown_requested: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the security monitoring system
    ///
    /// Launches background tasks for event processing, behavioral analysis,
    /// cleanup, and statistics collection.
    pub async fn start(&self) -> Result<(), PrimalError> {
        if !self.config.enable_real_time_monitoring {
            info!("Security monitoring disabled");
            return Ok(());
        }

        let correlation_id = CorrelationId::new();
        let mut tasks = self.background_tasks.lock().await;

        info!(
            correlation_id = %correlation_id,
            operation = "security_monitoring_start",
            "Starting security monitoring system"
        );

        // Start event processing task
        let event_processing_task = self.start_event_processing_task().await?;
        tasks.push(event_processing_task);

        // Start behavioral analysis task
        if self.config.enable_behavioral_analysis {
            let behavioral_analysis_task = self.start_behavioral_analysis_task().await;
            tasks.push(behavioral_analysis_task);
        }

        // Start cleanup task
        let cleanup_task = self.start_cleanup_task().await;
        tasks.push(cleanup_task);

        // Start statistics task
        let stats_task = self.start_statistics_task().await;
        tasks.push(stats_task);

        info!(
            correlation_id = %correlation_id,
            task_count = tasks.len(),
            operation = "security_monitoring_started",
            "Security monitoring system started successfully"
        );

        Ok(())
    }

    /// Record a security event
    ///
    /// Events are processed in real-time and stored for analysis.
    pub async fn record_event(&self, mut event: SecurityEvent) {
        // Ensure event has unique ID
        if event.event_id.is_nil() {
            event.event_id = Uuid::new_v4();
        }

        // Send to real-time processing if enabled
        if self.config.enable_real_time_monitoring
            && let Err(e) = self.event_sender.send(event.clone())
        {
            error!(
                event_id = %event.event_id,
                error = %e,
                operation = "event_recording_failed",
                "Failed to send security event for processing"
            );
        }

        // Add to buffer for batch processing
        {
            let mut buffer = self.event_buffer.lock().await;
            buffer.push(event.clone());

            // Flush buffer if full
            if buffer.len() >= self.config.event_buffer_size {
                let events_to_flush = buffer.drain(..).collect::<Vec<_>>();
                drop(buffer);
                self.flush_events_to_history(events_to_flush).await;
            }
        }

        debug!(
            event_id = %event.event_id,
            event_type = ?event.event_type,
            severity = ?event.severity,
            operation = "security_event_recorded",
            "Security event recorded"
        );
    }

    /// Classify a security event as a potential threat via the defense provider.
    pub async fn classify_threat(
        &self,
        event: &SecurityEvent,
    ) -> Result<Option<SecurityAlert>, PrimalError> {
        self.defense_client.classify_threat(event).await
    }

    /// Detect behavioral anomalies via the defense provider.
    pub async fn detect_anomaly(
        &self,
        pattern: &BehavioralPattern,
    ) -> Result<Option<SecurityAlert>, PrimalError> {
        self.defense_client.detect_anomaly(pattern).await
    }

    /// Get current security statistics
    pub async fn get_statistics(&self) -> SecurityMonitoringStats {
        self.stats.read().await.clone()
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<SecurityAlert> {
        let alerts = self.active_alerts.read().await;
        alerts.values().cloned().collect()
    }

    /// Test helper: get event buffer length (for verifying record_event)
    #[cfg(test)]
    pub(crate) async fn test_get_buffer_len(&self) -> usize {
        self.event_buffer.lock().await.len()
    }

    // Private methods

    /// Start event processing task
    async fn start_event_processing_task(&self) -> Result<JoinHandle<()>, PrimalError> {
        let receiver = {
            let mut receiver_guard = self.event_receiver.lock().await;
            receiver_guard
                .take()
                .ok_or_else(|| PrimalError::Internal("Event receiver already taken".to_string()))?
        };

        let active_alerts = Arc::clone(&self.active_alerts);
        let shutdown_requested = Arc::clone(&self.shutdown_requested);
        let stats = Arc::clone(&self.stats);
        let defense_client = self.defense_client.clone();

        let task = tokio::spawn(async move {
            let mut receiver = receiver;

            while let Some(event) = receiver.recv().await {
                // Check shutdown
                if *shutdown_requested.read().await {
                    info!("Event processing task shutting down");
                    break;
                }

                // Log event for passive observation
                debug!(
                    event_id = %event.event_id,
                    event_type = ?event.event_type,
                    severity = ?event.severity,
                    source_ip = %event.source_ip,
                    operation = "security_event_observed",
                    "Security event observed — delegating classification to defense provider"
                );

                // Delegate threat classification to defense provider (skunkBat)
                match defense_client.classify_threat(&event).await {
                    Ok(Some(alert)) => {
                        {
                            let mut alerts = active_alerts.write().await;
                            alerts.insert(alert.alert_id, alert.clone());
                        }
                        {
                            let mut stats_guard = stats.write().await;
                            stats_guard.alerts_generated += 1;
                        }
                        warn!(
                            alert_id = %alert.alert_id,
                            alert_type = ?alert.alert_type,
                            severity = ?alert.severity,
                            operation = "defense_alert_received",
                            "Defense provider alert: {}", alert.title
                        );
                    }
                    Ok(None) => {}
                    Err(e) => {
                        debug!(
                            event_id = %event.event_id,
                            error = %e,
                            operation = "defense_classify_unavailable",
                            "Defense provider unavailable — event logged only"
                        );
                    }
                }

                // Update stats
                {
                    let mut stats_guard = stats.write().await;
                    stats_guard.total_events += 1;

                    let event_type_key = format!("{:?}", event.event_type);
                    *stats_guard.event_types.entry(event_type_key).or_insert(0) += 1;
                }
            }
        });

        Ok(task)
    }

    /// Start behavioral analysis task
    async fn start_behavioral_analysis_task(&self) -> JoinHandle<()> {
        let behavioral_patterns = Arc::clone(&self.behavioral_patterns);
        let active_alerts = Arc::clone(&self.active_alerts);
        let shutdown_requested = Arc::clone(&self.shutdown_requested);
        let defense_client = self.defense_client.clone();

        tokio::spawn(async move {
            let mut analysis_interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes

            loop {
                analysis_interval.tick().await;

                if *shutdown_requested.read().await {
                    info!("Behavioral analysis task shutting down");
                    break;
                }

                // Forward behavioral patterns to defense provider for anomaly detection
                let patterns = behavioral_patterns.read().await;
                for (client_key, pattern) in patterns.iter() {
                    match defense_client.detect_anomaly(pattern).await {
                        Ok(Some(alert)) => {
                            {
                                let mut alerts = active_alerts.write().await;
                                alerts.insert(alert.alert_id, alert.clone());
                            }
                            info!(
                                alert_id = %alert.alert_id,
                                client = %client_key,
                                alert_type = ?alert.alert_type,
                                operation = "defense_anomaly_alert_received",
                                "Defense provider anomaly alert received"
                            );
                        }
                        Ok(None) => {}
                        Err(e) => {
                            debug!(
                                client = %client_key,
                                error = %e,
                                operation = "defense_anomaly_unavailable",
                                "Defense provider unavailable — pattern logged only"
                            );
                        }
                    }
                }

                debug!(
                    analyzed_patterns = patterns.len(),
                    operation = "behavioral_analysis_delegation_complete",
                    "Completed behavioral analysis delegation cycle"
                );
            }
        })
    }

    /// Start cleanup task
    async fn start_cleanup_task(&self) -> JoinHandle<()> {
        let event_history = Arc::clone(&self.event_history);
        let active_alerts = Arc::clone(&self.active_alerts);
        let behavioral_patterns = Arc::clone(&self.behavioral_patterns);
        let shutdown_requested = Arc::clone(&self.shutdown_requested);
        let retention_period = self.config.event_retention_period;

        tokio::spawn(async move {
            let mut cleanup_interval = tokio::time::interval(Duration::from_secs(3600)); // 1 hour

            loop {
                cleanup_interval.tick().await;

                if *shutdown_requested.read().await {
                    info!("Cleanup task shutting down");
                    break;
                }

                let now = SystemTime::now();

                // Cleanup old events
                let cleaned_events = {
                    let mut history = event_history.write().await;
                    let original_len = history.len();

                    history.retain(|event| {
                        now.duration_since(event.timestamp)
                            .map(|age| age < retention_period)
                            .unwrap_or_default()
                    });

                    original_len - history.len()
                };

                // Cleanup resolved alerts (older than 24 hours)
                let cleaned_alerts = {
                    let mut alerts = active_alerts.write().await;
                    let original_len = alerts.len();

                    alerts.retain(|_, alert| {
                        now.duration_since(alert.generated_at)
                            .map(|age| age < Duration::from_secs(24 * 3600))
                            .unwrap_or_default()
                    });

                    original_len - alerts.len()
                };

                // Cleanup old behavioral patterns
                let cleaned_patterns = {
                    let mut patterns = behavioral_patterns.write().await;
                    let original_len = patterns.len();

                    patterns.retain(|_, pattern| {
                        pattern.last_activity.elapsed() < Duration::from_secs(24 * 3600)
                    });

                    original_len - patterns.len()
                };

                if cleaned_events > 0 || cleaned_alerts > 0 || cleaned_patterns > 0 {
                    info!(
                        cleaned_events = cleaned_events,
                        cleaned_alerts = cleaned_alerts,
                        cleaned_patterns = cleaned_patterns,
                        operation = "security_monitoring_cleanup",
                        "Completed security monitoring cleanup"
                    );
                }
            }
        })
    }

    /// Start statistics task
    async fn start_statistics_task(&self) -> JoinHandle<()> {
        let stats = Arc::clone(&self.stats);
        let shutdown_requested = Arc::clone(&self.shutdown_requested);
        let start_time = Instant::now();

        tokio::spawn(async move {
            let mut stats_interval = tokio::time::interval(Duration::from_secs(60)); // 1 minute
            let mut last_total_events = 0;
            let mut last_alerts_generated = 0;

            loop {
                stats_interval.tick().await;

                if *shutdown_requested.read().await {
                    info!("Statistics task shutting down");
                    break;
                }

                let mut stats_guard = stats.write().await;

                // Calculate rates
                let uptime = start_time.elapsed();
                stats_guard.uptime = uptime;

                if uptime.as_secs() > 0 {
                    let current_events = stats_guard.total_events;
                    let current_alerts = stats_guard.alerts_generated;

                    stats_guard.events_per_second =
                        (current_events - last_total_events) as f64 / 60.0;
                    stats_guard.alert_rate = (current_alerts - last_alerts_generated) as f64 / 60.0;

                    last_total_events = current_events;
                    last_alerts_generated = current_alerts;
                }

                debug!(
                    total_events = stats_guard.total_events,
                    events_per_second = stats_guard.events_per_second,
                    alerts_generated = stats_guard.alerts_generated,
                    operation = "security_stats_update",
                    "Updated security monitoring statistics"
                );
            }
        })
    }

    /// Flush events to history
    async fn flush_events_to_history(&self, events: Vec<SecurityEvent>) {
        let mut history = self.event_history.write().await;

        for event in events {
            history.push_back(event);

            // Maintain size limit
            while history.len() > 10000 {
                history.pop_front();
            }
        }
    }
}

impl ShutdownHandler for SecurityMonitoringSystem {
    fn component_name(&self) -> &'static str {
        "security_monitoring"
    }

    async fn shutdown(&self, phase: ShutdownPhase) -> Result<(), PrimalError> {
        match phase {
            ShutdownPhase::StopAccepting => {
                info!("Security monitoring stopped accepting new events");
                Ok(())
            }
            ShutdownPhase::DrainRequests => {
                // Flush any remaining events
                let events_to_flush = {
                    let mut buffer = self.event_buffer.lock().await;
                    buffer.drain(..).collect::<Vec<_>>()
                };

                if !events_to_flush.is_empty() {
                    self.flush_events_to_history(events_to_flush).await;
                    info!("Flushed remaining security events");
                }
                Ok(())
            }
            ShutdownPhase::CloseConnections => {
                // No network connections to close
                Ok(())
            }
            ShutdownPhase::CleanupResources => {
                // Signal background tasks to shutdown
                {
                    let mut shutdown_flag = self.shutdown_requested.write().await;
                    *shutdown_flag = true;
                }
                Ok(())
            }
            ShutdownPhase::ShutdownTasks => {
                // Wait for background tasks to complete
                let mut tasks = self.background_tasks.lock().await;
                for task in tasks.drain(..) {
                    task.abort();
                    let _ = tokio::time::timeout(Duration::from_secs(5), task).await;
                }
                info!("Security monitoring background tasks shutdown completed");
                Ok(())
            }
            ShutdownPhase::FinalCleanup => {
                // Clear all data
                {
                    let mut history = self.event_history.write().await;
                    history.clear();
                }
                {
                    let mut alerts = self.active_alerts.write().await;
                    alerts.clear();
                }
                {
                    let mut patterns = self.behavioral_patterns.write().await;
                    patterns.clear();
                }
                info!("Security monitoring final cleanup completed");
                Ok(())
            }
        }
    }

    async fn is_shutdown_complete(&self) -> bool {
        *self.shutdown_requested.read().await
    }

    fn estimated_shutdown_time(&self) -> Duration {
        Duration::from_secs(10)
    }
}

#[cfg(test)]
#[path = "monitoring_tests.rs"]
mod tests;
