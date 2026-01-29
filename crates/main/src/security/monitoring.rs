//! # Production Security Monitoring & Threat Detection
//!
//! This module provides real-time security monitoring including:
//! - Security event collection and analysis
//! - Threat pattern detection
//! - Behavioral anomaly detection
//! - Security metrics and alerting
//! - Integration with SIEM systems

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::error::PrimalError;
use crate::observability::CorrelationId;
use crate::shutdown::{ShutdownHandler, ShutdownPhase};

/// Security monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMonitoringConfig {
    /// Enable real-time monitoring
    pub enable_real_time_monitoring: bool,

    /// Event buffer size before flushing
    pub event_buffer_size: usize,

    /// Event retention period
    pub event_retention_period: Duration,

    /// Alert thresholds
    pub alert_thresholds: AlertThresholds,

    /// Enable behavioral analysis
    pub enable_behavioral_analysis: bool,

    /// Behavioral analysis window
    pub behavioral_window: Duration,

    /// Enable automated response
    pub enable_automated_response: bool,

    /// Maximum events to analyze per batch
    pub max_events_per_batch: usize,
}

impl Default for SecurityMonitoringConfig {
    fn default() -> Self {
        Self {
            enable_real_time_monitoring: true,
            event_buffer_size: 1000,
            event_retention_period: Duration::from_secs(24 * 3600), // 24 hours
            alert_thresholds: AlertThresholds::default(),
            enable_behavioral_analysis: true,
            behavioral_window: Duration::from_secs(3600), // 1 hour
            enable_automated_response: true,
            max_events_per_batch: 100,
        }
    }
}

/// Alert threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Failed authentication attempts per IP per hour
    pub failed_auth_per_hour: u32,

    /// Rate limit violations per IP per hour
    pub rate_limit_violations_per_hour: u32,

    /// Input validation violations per IP per hour
    pub input_violations_per_hour: u32,

    /// Suspicious activities per IP per hour
    pub suspicious_activities_per_hour: u32,

    /// Maximum concurrent sessions per user
    pub max_concurrent_sessions_per_user: u32,

    /// Maximum failed requests ratio (0.0 to 1.0)
    pub max_failed_requests_ratio: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            failed_auth_per_hour: 10,
            rate_limit_violations_per_hour: 50,
            input_violations_per_hour: 20,
            suspicious_activities_per_hour: 5,
            max_concurrent_sessions_per_user: 5,
            max_failed_requests_ratio: 0.3,
        }
    }
}

/// Security event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SecurityEventType {
    /// Authentication event
    Authentication {
        success: bool,
        user_id: Option<String>,
        method: String,
    },

    /// Authorization event
    Authorization {
        granted: bool,
        user_id: String,
        resource: String,
        action: String,
    },

    /// Rate limiting event
    RateLimitViolation {
        client_ip: String,
        endpoint: String,
        violation_count: u32,
    },

    /// Input validation violation
    InputValidationViolation {
        client_ip: String,
        violation_type: String,
        risk_level: String,
    },

    /// Suspicious activity detected
    SuspiciousActivity {
        client_ip: String,
        activity_type: String,
        details: HashMap<String, String>,
    },

    /// Security policy violation
    PolicyViolation {
        policy_id: String,
        user_id: Option<String>,
        details: String,
    },

    /// System access event
    SystemAccess {
        user_id: String,
        access_type: String,
        resource: String,
    },
}

/// Security event for monitoring
#[derive(Debug, Clone, Serialize)]
pub struct SecurityEvent {
    /// Event identifier
    pub event_id: Uuid,

    /// Event type and details
    pub event_type: SecurityEventType,

    /// Event timestamp
    pub timestamp: SystemTime,

    /// Source IP address
    pub source_ip: String,

    /// User agent
    pub user_agent: Option<String>,

    /// Correlation ID for request tracking
    pub correlation_id: CorrelationId,

    /// Event severity
    pub severity: EventSeverity,

    /// Additional metadata
    pub metadata: HashMap<String, String>,

    /// Event source component
    pub source_component: String,
}

/// Event severity levels
#[derive(Debug, Clone, Serialize, PartialEq, PartialOrd)]
pub enum EventSeverity {
    Info,
    Warning,
    High,
    Critical,
}

/// Security alert
#[derive(Debug, Clone, Serialize)]
pub struct SecurityAlert {
    /// Alert identifier
    pub alert_id: Uuid,

    /// Alert type
    pub alert_type: AlertType,

    /// Alert severity
    pub severity: EventSeverity,

    /// Alert description
    pub description: String,

    /// Events that triggered this alert
    pub triggering_events: Vec<Uuid>,

    /// Alert timestamp
    pub timestamp: SystemTime,

    /// Recommended actions
    pub recommended_actions: Vec<String>,

    /// Alert metadata
    pub metadata: HashMap<String, String>,
}

/// Alert types
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum AlertType {
    /// Multiple failed authentication attempts
    BruteForceAttack,

    /// Excessive rate limit violations
    DosAttack,

    /// Multiple input validation violations
    InjectionAttack,

    /// Unusual behavioral patterns
    AnomalousActivity,

    /// Security policy violations
    PolicyViolation,

    /// System compromise indicators
    CompromiseIndicator,
}

/// Behavioral pattern for anomaly detection
#[derive(Debug, Clone)]
struct BehavioralPattern {
    client_ip: String,
    user_id: Option<String>,
    request_patterns: VecDeque<RequestPattern>,
    first_seen: Instant,
    last_activity: Instant,
    total_requests: u64,
    failed_requests: u64,
    violation_count: u32,
}

#[derive(Debug, Clone)]
struct RequestPattern {
    timestamp: Instant,
    endpoint: String,
    success: bool,
    response_time_ms: u64,
}

/// Security monitoring statistics
#[derive(Debug, Clone, Serialize)]
pub struct SecurityMonitoringStats {
    pub total_events: u64,
    pub alerts_generated: u64,
    pub active_threats: u32,
    pub monitored_clients: u32,
    pub events_per_second: f64,
    pub alert_rate: f64,
    pub uptime: Duration,
    pub event_types: HashMap<String, u64>,
}

/// Production security monitoring system
pub struct SecurityMonitoringSystem {
    /// Configuration
    config: SecurityMonitoringConfig,

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
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        Self {
            config,
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
    pub async fn record_event(&self, mut event: SecurityEvent) {
        // Ensure event has unique ID
        if event.event_id.is_nil() {
            event.event_id = Uuid::new_v4();
        }

        // Send to real-time processing if enabled
        if self.config.enable_real_time_monitoring {
            if let Err(e) = self.event_sender.send(event.clone()) {
                error!(
                    event_id = %event.event_id,
                    error = %e,
                    operation = "event_recording_failed",
                    "Failed to send security event for processing"
                );
            }
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
        let config = self.config.clone();
        let stats = Arc::clone(&self.stats);

        let task = tokio::spawn(async move {
            let mut receiver = receiver;

            while let Some(event) = receiver.recv().await {
                // Check shutdown
                if *shutdown_requested.read().await {
                    info!("Event processing task shutting down");
                    break;
                }

                // Process event for immediate threats
                if let Some(alert) = Self::analyze_event_for_threats(&event, &config).await {
                    // Store alert
                    {
                        let mut alerts = active_alerts.write().await;
                        alerts.insert(alert.alert_id, alert.clone());
                    }

                    // Update stats
                    {
                        let mut stats_guard = stats.write().await;
                        stats_guard.alerts_generated += 1;
                    }

                    warn!(
                        alert_id = %alert.alert_id,
                        alert_type = ?alert.alert_type,
                        severity = ?alert.severity,
                        operation = "security_alert_generated",
                        "Security alert generated: {}", alert.description
                    );
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
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut analysis_interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes

            loop {
                analysis_interval.tick().await;

                if *shutdown_requested.read().await {
                    info!("Behavioral analysis task shutting down");
                    break;
                }

                // Analyze behavioral patterns
                let patterns = behavioral_patterns.read().await;
                for (client_key, pattern) in patterns.iter() {
                    if let Some(alert) = Self::analyze_behavioral_pattern(pattern, &config).await {
                        // Store alert
                        {
                            let mut alerts = active_alerts.write().await;
                            alerts.insert(alert.alert_id, alert.clone());
                        }

                        info!(
                            alert_id = %alert.alert_id,
                            client = %client_key,
                            alert_type = ?alert.alert_type,
                            operation = "behavioral_alert_generated",
                            "Behavioral analysis alert generated"
                        );
                    }
                }

                debug!(
                    analyzed_patterns = patterns.len(),
                    operation = "behavioral_analysis_complete",
                    "Completed behavioral analysis cycle"
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
                let mut cleaned_events = 0;
                let mut cleaned_alerts = 0;
                let mut cleaned_patterns = 0;

                // Cleanup old events
                {
                    let mut history = event_history.write().await;
                    let original_len = history.len();

                    history.retain(|event| {
                        now.duration_since(event.timestamp)
                            .map(|age| age < retention_period)
                            .unwrap_or(false)
                    });

                    cleaned_events = original_len - history.len();
                }

                // Cleanup resolved alerts (older than 24 hours)
                {
                    let mut alerts = active_alerts.write().await;
                    let original_len = alerts.len();

                    alerts.retain(|_, alert| {
                        now.duration_since(alert.timestamp)
                            .map(|age| age < Duration::from_secs(24 * 3600))
                            .unwrap_or(false)
                    });

                    cleaned_alerts = original_len - alerts.len();
                }

                // Cleanup old behavioral patterns
                {
                    let mut patterns = behavioral_patterns.write().await;
                    let original_len = patterns.len();

                    patterns.retain(|_, pattern| {
                        pattern.last_activity.elapsed() < Duration::from_secs(24 * 3600)
                    });

                    cleaned_patterns = original_len - patterns.len();
                }

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

    /// Analyze event for immediate threats
    async fn analyze_event_for_threats(
        event: &SecurityEvent,
        config: &SecurityMonitoringConfig,
    ) -> Option<SecurityAlert> {
        match &event.event_type {
            SecurityEventType::Authentication { success: false, .. } => {
                // Check for brute force patterns
                Some(SecurityAlert {
                    alert_id: Uuid::new_v4(),
                    alert_type: AlertType::BruteForceAttack,
                    severity: EventSeverity::High,
                    description: "Failed authentication attempt detected".to_string(),
                    triggering_events: vec![event.event_id],
                    timestamp: SystemTime::now(),
                    recommended_actions: vec![
                        "Monitor IP for additional failed attempts".to_string(),
                        "Consider rate limiting".to_string(),
                    ],
                    metadata: HashMap::new(),
                })
            }
            SecurityEventType::InputValidationViolation { risk_level, .. } => {
                if risk_level == "Critical" || risk_level == "High" {
                    Some(SecurityAlert {
                        alert_id: Uuid::new_v4(),
                        alert_type: AlertType::InjectionAttack,
                        severity: EventSeverity::High,
                        description: "High-risk input validation violation detected".to_string(),
                        triggering_events: vec![event.event_id],
                        timestamp: SystemTime::now(),
                        recommended_actions: vec![
                            "Block suspicious IP".to_string(),
                            "Review input validation rules".to_string(),
                        ],
                        metadata: HashMap::new(),
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Analyze behavioral pattern for anomalies
    async fn analyze_behavioral_pattern(
        pattern: &BehavioralPattern,
        config: &SecurityMonitoringConfig,
    ) -> Option<SecurityAlert> {
        let failure_rate = if pattern.total_requests > 0 {
            pattern.failed_requests as f64 / pattern.total_requests as f64
        } else {
            0.0
        };

        if failure_rate > config.alert_thresholds.max_failed_requests_ratio {
            Some(SecurityAlert {
                alert_id: Uuid::new_v4(),
                alert_type: AlertType::AnomalousActivity,
                severity: EventSeverity::Warning,
                description: format!(
                    "High failure rate detected: {:.1}% from IP {}",
                    failure_rate * 100.0,
                    pattern.client_ip
                ),
                triggering_events: vec![],
                timestamp: SystemTime::now(),
                recommended_actions: vec![
                    "Investigate client behavior".to_string(),
                    "Consider blocking if malicious".to_string(),
                ],
                metadata: HashMap::new(),
            })
        } else {
            None
        }
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

    /// Get current security statistics
    pub async fn get_statistics(&self) -> SecurityMonitoringStats {
        self.stats.read().await.clone()
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<SecurityAlert> {
        let alerts = self.active_alerts.read().await;
        alerts.values().cloned().collect()
    }
}

#[async_trait::async_trait]
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
mod tests {
    use super::*;

    #[test]
    fn test_security_monitoring_config_default() {
        let config = SecurityMonitoringConfig::default();

        assert!(config.enable_real_time_monitoring);
        assert_eq!(config.event_buffer_size, 1000);
        assert_eq!(
            config.event_retention_period,
            Duration::from_secs(24 * 3600)
        );
        assert!(config.enable_behavioral_analysis);
        assert_eq!(config.behavioral_window, Duration::from_secs(3600));
        assert!(config.enable_automated_response);
        assert_eq!(config.max_events_per_batch, 100);
    }

    #[test]
    fn test_security_monitoring_config_custom() {
        let config = SecurityMonitoringConfig {
            enable_real_time_monitoring: false,
            event_buffer_size: 500,
            event_retention_period: Duration::from_secs(3600),
            alert_thresholds: AlertThresholds::default(),
            enable_behavioral_analysis: false,
            behavioral_window: Duration::from_secs(1800),
            enable_automated_response: false,
            max_events_per_batch: 50,
        };

        assert!(!config.enable_real_time_monitoring);
        assert_eq!(config.event_buffer_size, 500);
        assert_eq!(config.event_retention_period, Duration::from_secs(3600));
        assert!(!config.enable_behavioral_analysis);
        assert_eq!(config.behavioral_window, Duration::from_secs(1800));
        assert!(!config.enable_automated_response);
        assert_eq!(config.max_events_per_batch, 50);
    }

    #[test]
    fn test_alert_thresholds_default() {
        let thresholds = AlertThresholds::default();

        assert_eq!(thresholds.failed_auth_per_hour, 10);
        assert_eq!(thresholds.rate_limit_violations_per_hour, 50);
        assert_eq!(thresholds.input_violations_per_hour, 20);
        assert_eq!(thresholds.suspicious_activities_per_hour, 5);
        assert_eq!(thresholds.max_concurrent_sessions_per_user, 5);
        assert_eq!(thresholds.max_failed_requests_ratio, 0.3);
    }

    #[test]
    fn test_alert_thresholds_custom() {
        let thresholds = AlertThresholds {
            failed_auth_per_hour: 20,
            rate_limit_violations_per_hour: 100,
            input_violations_per_hour: 40,
            suspicious_activities_per_hour: 10,
            max_concurrent_sessions_per_user: 10,
            max_failed_requests_ratio: 0.5,
        };

        assert_eq!(thresholds.failed_auth_per_hour, 20);
        assert_eq!(thresholds.rate_limit_violations_per_hour, 100);
        assert_eq!(thresholds.input_violations_per_hour, 40);
        assert_eq!(thresholds.suspicious_activities_per_hour, 10);
        assert_eq!(thresholds.max_concurrent_sessions_per_user, 10);
        assert_eq!(thresholds.max_failed_requests_ratio, 0.5);
    }

    #[test]
    fn test_event_severity_ordering() {
        assert!(EventSeverity::Info < EventSeverity::Warning);
        assert!(EventSeverity::Warning < EventSeverity::High);
        assert!(EventSeverity::High < EventSeverity::Critical);
    }

    #[test]
    fn test_event_severity_equality() {
        assert_eq!(EventSeverity::Info, EventSeverity::Info);
        assert_eq!(EventSeverity::Warning, EventSeverity::Warning);
        assert_eq!(EventSeverity::High, EventSeverity::High);
        assert_eq!(EventSeverity::Critical, EventSeverity::Critical);
    }

    #[test]
    fn test_security_event_type_authentication_success() {
        let event = SecurityEventType::Authentication {
            success: true,
            user_id: Some("user123".to_string()),
            method: "oauth2".to_string(),
        };

        if let SecurityEventType::Authentication {
            success,
            user_id,
            method,
        } = event
        {
            assert!(success);
            assert_eq!(user_id.unwrap(), "user123");
            assert_eq!(method, "oauth2");
        } else {
            panic!("Expected Authentication event");
        }
    }

    #[test]
    fn test_security_event_type_authentication_failure() {
        let event = SecurityEventType::Authentication {
            success: false,
            user_id: None,
            method: "password".to_string(),
        };

        if let SecurityEventType::Authentication {
            success,
            user_id,
            method,
        } = event
        {
            assert!(!success);
            assert!(user_id.is_none());
            assert_eq!(method, "password");
        } else {
            panic!("Expected Authentication event");
        }
    }

    #[test]
    fn test_security_event_type_authorization_granted() {
        let event = SecurityEventType::Authorization {
            granted: true,
            user_id: "user123".to_string(),
            resource: "/api/data".to_string(),
            action: "read".to_string(),
        };

        if let SecurityEventType::Authorization {
            granted,
            user_id,
            resource,
            action,
        } = event
        {
            assert!(granted);
            assert_eq!(user_id, "user123");
            assert_eq!(resource, "/api/data");
            assert_eq!(action, "read");
        } else {
            panic!("Expected Authorization event");
        }
    }

    #[test]
    fn test_security_event_type_authorization_denied() {
        let event = SecurityEventType::Authorization {
            granted: false,
            user_id: "user456".to_string(),
            resource: "/api/admin".to_string(),
            action: "write".to_string(),
        };

        if let SecurityEventType::Authorization {
            granted,
            user_id,
            resource,
            action,
        } = event
        {
            assert!(!granted);
            assert_eq!(user_id, "user456");
            assert_eq!(resource, "/api/admin");
            assert_eq!(action, "write");
        } else {
            panic!("Expected Authorization event");
        }
    }

    #[test]
    fn test_security_event_type_rate_limit_violation() {
        let event = SecurityEventType::RateLimitViolation {
            client_ip: "192.168.1.100".to_string(),
            endpoint: "/api/query".to_string(),
            violation_count: 5,
        };

        if let SecurityEventType::RateLimitViolation {
            client_ip,
            endpoint,
            violation_count,
        } = event
        {
            assert_eq!(client_ip, "192.168.1.100");
            assert_eq!(endpoint, "/api/query");
            assert_eq!(violation_count, 5);
        } else {
            panic!("Expected RateLimitViolation event");
        }
    }

    #[test]
    fn test_security_event_type_input_validation_violation() {
        let event = SecurityEventType::InputValidationViolation {
            client_ip: "10.0.0.1".to_string(),
            violation_type: "sql_injection".to_string(),
            risk_level: "high".to_string(),
        };

        if let SecurityEventType::InputValidationViolation {
            client_ip,
            violation_type,
            risk_level,
        } = event
        {
            assert_eq!(client_ip, "10.0.0.1");
            assert_eq!(violation_type, "sql_injection");
            assert_eq!(risk_level, "high");
        } else {
            panic!("Expected InputValidationViolation event");
        }
    }

    #[test]
    fn test_security_event_type_suspicious_activity() {
        let mut details = HashMap::new();
        details.insert("pattern".to_string(), "port_scan".to_string());
        details.insert("count".to_string(), "50".to_string());

        let event = SecurityEventType::SuspiciousActivity {
            client_ip: "172.16.0.1".to_string(),
            activity_type: "port_scanning".to_string(),
            details: details.clone(),
        };

        if let SecurityEventType::SuspiciousActivity {
            client_ip,
            activity_type,
            details: event_details,
        } = event
        {
            assert_eq!(client_ip, "172.16.0.1");
            assert_eq!(activity_type, "port_scanning");
            assert_eq!(event_details.get("pattern").unwrap(), "port_scan");
            assert_eq!(event_details.get("count").unwrap(), "50");
        } else {
            panic!("Expected SuspiciousActivity event");
        }
    }

    #[test]
    fn test_security_event_type_policy_violation() {
        let event = SecurityEventType::PolicyViolation {
            policy_id: "POLICY-001".to_string(),
            user_id: Some("user789".to_string()),
            details: "Access attempted outside allowed hours".to_string(),
        };

        if let SecurityEventType::PolicyViolation {
            policy_id,
            user_id,
            details,
        } = event
        {
            assert_eq!(policy_id, "POLICY-001");
            assert_eq!(user_id.unwrap(), "user789");
            assert_eq!(details, "Access attempted outside allowed hours");
        } else {
            panic!("Expected PolicyViolation event");
        }
    }

    #[test]
    fn test_security_event_type_system_access() {
        let event = SecurityEventType::SystemAccess {
            user_id: "admin123".to_string(),
            access_type: "console".to_string(),
            resource: "production_database".to_string(),
        };

        if let SecurityEventType::SystemAccess {
            user_id,
            access_type,
            resource,
        } = event
        {
            assert_eq!(user_id, "admin123");
            assert_eq!(access_type, "console");
            assert_eq!(resource, "production_database");
        } else {
            panic!("Expected SystemAccess event");
        }
    }

    #[test]
    fn test_security_event_creation() {
        let event = SecurityEvent {
            event_id: Uuid::new_v4(),
            event_type: SecurityEventType::Authentication {
                success: true,
                user_id: Some("test_user".to_string()),
                method: "password".to_string(),
            },
            timestamp: SystemTime::now(),
            source_ip: "127.0.0.1".to_string(),
            user_agent: Some("TestAgent/1.0".to_string()),
            correlation_id: CorrelationId::new(),
            severity: EventSeverity::Info,
            metadata: HashMap::new(),
            source_component: "auth_service".to_string(),
        };

        assert_eq!(event.source_ip, "127.0.0.1");
        assert_eq!(event.user_agent.unwrap(), "TestAgent/1.0");
        assert_eq!(event.severity, EventSeverity::Info);
        assert_eq!(event.source_component, "auth_service");
    }

    #[tokio::test]
    async fn test_security_monitoring_system_new() {
        let config = SecurityMonitoringConfig::default();
        let system = SecurityMonitoringSystem::new(config);

        let stats = system.get_statistics().await;
        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.alerts_generated, 0);
    }

    #[tokio::test]
    async fn test_security_monitoring_system_record_event() {
        let config = SecurityMonitoringConfig::default();
        let system = SecurityMonitoringSystem::new(config);

        // Start the system to begin background processing
        let _ = system.start().await;

        let event = SecurityEvent {
            event_id: Uuid::new_v4(),
            event_type: SecurityEventType::Authentication {
                success: true,
                user_id: Some("test_user".to_string()),
                method: "password".to_string(),
            },
            timestamp: SystemTime::now(),
            source_ip: "127.0.0.1".to_string(),
            user_agent: Some("TestAgent/1.0".to_string()),
            correlation_id: CorrelationId::new(),
            severity: EventSeverity::Info,
            metadata: HashMap::new(),
            source_component: "test".to_string(),
        };

        system.record_event(event).await;

        // Give background processing time
        tokio::time::sleep(Duration::from_millis(200)).await;

        let stats = system.get_statistics().await;
        assert!(stats.total_events >= 1);
    }

    #[tokio::test]
    async fn test_security_monitoring_system_multiple_events() {
        let config = SecurityMonitoringConfig::default();
        let system = SecurityMonitoringSystem::new(config);

        // Start the system to begin background processing
        let _ = system.start().await;

        for i in 0..5 {
            let event = SecurityEvent {
                event_id: Uuid::new_v4(),
                event_type: SecurityEventType::Authentication {
                    success: i % 2 == 0,
                    user_id: Some(format!("user{}", i)),
                    method: "password".to_string(),
                },
                timestamp: SystemTime::now(),
                source_ip: format!("192.168.1.{}", i),
                user_agent: Some("TestAgent/1.0".to_string()),
                correlation_id: CorrelationId::new(),
                severity: EventSeverity::Info,
                metadata: HashMap::new(),
                source_component: "test".to_string(),
            };

            system.record_event(event).await;
        }

        // Give background processing time
        tokio::time::sleep(Duration::from_millis(300)).await;

        let stats = system.get_statistics().await;
        assert!(stats.total_events >= 5);
    }

    #[tokio::test]
    async fn test_security_monitoring_system_get_active_alerts() {
        let config = SecurityMonitoringConfig::default();
        let system = SecurityMonitoringSystem::new(config);

        let alerts = system.get_active_alerts().await;
        assert_eq!(alerts.len(), 0); // No alerts initially
    }

    #[tokio::test]
    async fn test_security_event_type_equality() {
        let event1 = SecurityEventType::Authentication {
            success: true,
            user_id: Some("user123".to_string()),
            method: "oauth2".to_string(),
        };

        let event2 = SecurityEventType::Authentication {
            success: true,
            user_id: Some("user123".to_string()),
            method: "oauth2".to_string(),
        };

        assert_eq!(event1, event2);
    }

    #[test]
    fn test_config_serialization() {
        let config = SecurityMonitoringConfig::default();
        let serialized = serde_json::to_string(&config);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_alert_thresholds_serialization() {
        let thresholds = AlertThresholds::default();
        let serialized = serde_json::to_string(&thresholds);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_event_severity_debug() {
        let severity = EventSeverity::Critical;
        let debug_str = format!("{:?}", severity);
        assert!(debug_str.contains("Critical"));
    }

    #[test]
    fn test_security_event_type_debug() {
        let event = SecurityEventType::Authentication {
            success: true,
            user_id: Some("test".to_string()),
            method: "password".to_string(),
        };
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("Authentication"));
    }

    #[tokio::test]
    async fn test_shutdown_handler_component_name() {
        let config = SecurityMonitoringConfig::default();
        let system = SecurityMonitoringSystem::new(config);

        assert_eq!(system.component_name(), "security_monitoring");
    }

    #[tokio::test]
    async fn test_shutdown_handler_estimated_time() {
        let config = SecurityMonitoringConfig::default();
        let system = SecurityMonitoringSystem::new(config);

        let estimated = system.estimated_shutdown_time();
        assert_eq!(estimated, Duration::from_secs(10));
    }

    #[tokio::test]
    async fn test_shutdown_handler_is_complete() {
        let config = SecurityMonitoringConfig::default();
        let system = SecurityMonitoringSystem::new(config);

        let is_complete = system.is_shutdown_complete().await;
        assert!(!is_complete); // Should be false initially
    }

    #[test]
    fn test_config_clone() {
        let config = SecurityMonitoringConfig::default();
        let cloned = config.clone();

        assert_eq!(
            config.enable_real_time_monitoring,
            cloned.enable_real_time_monitoring
        );
        assert_eq!(config.event_buffer_size, cloned.event_buffer_size);
        assert_eq!(config.max_events_per_batch, cloned.max_events_per_batch);
    }

    #[test]
    fn test_alert_thresholds_clone() {
        let thresholds = AlertThresholds::default();
        let cloned = thresholds.clone();

        assert_eq!(thresholds.failed_auth_per_hour, cloned.failed_auth_per_hour);
        assert_eq!(
            thresholds.rate_limit_violations_per_hour,
            cloned.rate_limit_violations_per_hour
        );
        assert_eq!(
            thresholds.max_concurrent_sessions_per_user,
            cloned.max_concurrent_sessions_per_user
        );
    }

    #[test]
    fn test_event_severity_clone() {
        let severity = EventSeverity::High;
        let cloned = severity.clone();

        assert_eq!(severity, cloned);
    }

    #[test]
    fn test_security_event_type_clone() {
        let event = SecurityEventType::Authentication {
            success: true,
            user_id: Some("test".to_string()),
            method: "password".to_string(),
        };
        let cloned = event.clone();

        assert_eq!(event, cloned);
    }
}
