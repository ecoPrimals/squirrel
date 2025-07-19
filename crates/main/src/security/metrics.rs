//! Security Metrics Collection
//!
//! This module provides security-specific metrics collection capabilities,
//! including authentication, authorization, encryption, and audit metrics.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::debug;

use super::SecuritySession;
use crate::error::PrimalError;

/// Security metrics collector
#[derive(Debug, Clone)]
pub struct SecurityMetrics {
    /// Authentication metrics
    auth_metrics: Arc<RwLock<AuthenticationMetrics>>,
    /// Authorization metrics
    authz_metrics: Arc<RwLock<AuthorizationMetrics>>,
    /// Encryption metrics
    crypto_metrics: Arc<RwLock<CryptographyMetrics>>,
    /// Audit metrics
    audit_metrics: Arc<RwLock<AuditMetrics>>,
    /// Session metrics
    session_metrics: Arc<RwLock<SessionMetrics>>,
    /// Threat detection metrics
    threat_metrics: Arc<RwLock<ThreatMetrics>>,
}

/// Authentication metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationMetrics {
    /// Total authentication attempts
    pub total_attempts: u64,
    /// Successful authentications
    pub successful_authentications: u64,
    /// Failed authentications
    pub failed_authentications: u64,
    /// Average authentication time
    pub avg_auth_time: Duration,
    /// Authentication attempts by method
    pub attempts_by_method: HashMap<String, u64>,
    /// Last authentication time
    pub last_auth_time: Option<DateTime<Utc>>,
}

/// Authorization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationMetrics {
    /// Total authorization checks
    pub total_checks: u64,
    /// Successful authorizations
    pub successful_authorizations: u64,
    /// Failed authorizations
    pub failed_authorizations: u64,
    /// Average authorization time
    pub avg_authz_time: Duration,
    /// Authorization checks by resource
    pub checks_by_resource: HashMap<String, u64>,
    /// Last authorization time
    pub last_authz_time: Option<DateTime<Utc>>,
}

/// Cryptography metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptographyMetrics {
    /// Total encryption operations
    pub total_encryptions: u64,
    /// Total decryption operations
    pub total_decryptions: u64,
    /// Total bytes encrypted
    pub total_bytes_encrypted: u64,
    /// Total bytes decrypted
    pub total_bytes_decrypted: u64,
    /// Average encryption time
    pub avg_encryption_time: Duration,
    /// Average decryption time
    pub avg_decryption_time: Duration,
    /// Encryption operations by algorithm
    pub encryptions_by_algorithm: HashMap<String, u64>,
}

/// Audit metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditMetrics {
    /// Total audit events
    pub total_events: u64,
    /// Audit events by type
    pub events_by_type: HashMap<String, u64>,
    /// Audit events by severity
    pub events_by_severity: HashMap<String, u64>,
    /// Failed audit writes
    pub failed_writes: u64,
    /// Last audit time
    pub last_audit_time: Option<DateTime<Utc>>,
}

/// Session metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetrics {
    /// Total sessions created
    pub total_sessions_created: u64,
    /// Total sessions destroyed
    pub total_sessions_destroyed: u64,
    /// Active sessions count
    pub active_sessions: u64,
    /// Expired sessions count
    pub expired_sessions: u64,
    /// Average session duration
    pub avg_session_duration: Duration,
    /// Sessions by type
    pub sessions_by_type: HashMap<String, u64>,
}

/// Threat detection metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatMetrics {
    /// Total threats detected
    pub total_threats: u64,
    /// Threats by type
    pub threats_by_type: HashMap<String, u64>,
    /// Threats by severity
    pub threats_by_severity: HashMap<String, u64>,
    /// Blocked threats
    pub blocked_threats: u64,
    /// Last threat detection time
    pub last_threat_time: Option<DateTime<Utc>>,
}

/// Security event for metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    /// Event ID
    pub event_id: String,
    /// Event type
    pub event_type: SecurityEventType,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Event duration
    pub duration: Option<Duration>,
    /// Event result
    pub result: SecurityEventResult,
    /// Event metadata
    pub metadata: HashMap<String, String>,
}

/// Security event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    /// Authentication event
    Authentication,
    /// Authorization event
    Authorization,
    /// Encryption event
    Encryption,
    /// Decryption event
    Decryption,
    /// Audit event
    Audit,
    /// Session creation event
    SessionCreated,
    /// Session destruction event
    SessionDestroyed,
    /// Threat detection event
    ThreatDetected,
}

/// Security event result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventResult {
    /// Event succeeded
    Success,
    /// Event failed
    Failed,
    /// Event blocked
    Blocked,
    /// Event suspicious
    Suspicious,
}

impl SecurityMetrics {
    /// Create a new security metrics collector
    pub fn new() -> Self {
        Self {
            auth_metrics: Arc::new(RwLock::new(AuthenticationMetrics::new())),
            authz_metrics: Arc::new(RwLock::new(AuthorizationMetrics::new())),
            crypto_metrics: Arc::new(RwLock::new(CryptographyMetrics::new())),
            audit_metrics: Arc::new(RwLock::new(AuditMetrics::new())),
            session_metrics: Arc::new(RwLock::new(SessionMetrics::new())),
            threat_metrics: Arc::new(RwLock::new(ThreatMetrics::new())),
        }
    }

    /// Record a security event
    pub async fn record_event(&self, event: SecurityEvent) -> Result<(), PrimalError> {
        debug!("Recording security event: {:?}", event.event_type);

        match event.event_type {
            SecurityEventType::Authentication => {
                self.record_authentication_event(&event).await?;
            }
            SecurityEventType::Authorization => {
                self.record_authorization_event(&event).await?;
            }
            SecurityEventType::Encryption => {
                self.record_encryption_event(&event).await?;
            }
            SecurityEventType::Decryption => {
                self.record_decryption_event(&event).await?;
            }
            SecurityEventType::Audit => {
                self.record_audit_event(&event).await?;
            }
            SecurityEventType::SessionCreated => {
                self.record_session_created_event(&event).await?;
            }
            SecurityEventType::SessionDestroyed => {
                self.record_session_destroyed_event(&event).await?;
            }
            SecurityEventType::ThreatDetected => {
                self.record_threat_event(&event).await?;
            }
        }

        Ok(())
    }

    /// Record authentication event
    async fn record_authentication_event(&self, event: &SecurityEvent) -> Result<(), PrimalError> {
        let mut metrics = self.auth_metrics.write().await;
        metrics.total_attempts += 1;

        match event.result {
            SecurityEventResult::Success => {
                metrics.successful_authentications += 1;
            }
            SecurityEventResult::Failed => {
                metrics.failed_authentications += 1;
            }
            _ => {}
        }

        // Update method-specific metrics
        if let Some(method) = event.metadata.get("method") {
            *metrics
                .attempts_by_method
                .entry(method.clone())
                .or_insert(0) += 1;
        }

        // Update timing metrics
        if let Some(duration) = event.duration {
            metrics.avg_auth_time = if metrics.total_attempts == 1 {
                duration
            } else {
                Duration::from_nanos(
                    (metrics.avg_auth_time.as_nanos() as u64 + duration.as_nanos() as u64) / 2,
                )
            };
        }

        metrics.last_auth_time = Some(event.timestamp);
        Ok(())
    }

    /// Record authorization event
    async fn record_authorization_event(&self, event: &SecurityEvent) -> Result<(), PrimalError> {
        let mut metrics = self.authz_metrics.write().await;
        metrics.total_checks += 1;

        match event.result {
            SecurityEventResult::Success => {
                metrics.successful_authorizations += 1;
            }
            SecurityEventResult::Failed => {
                metrics.failed_authorizations += 1;
            }
            _ => {}
        }

        // Update resource-specific metrics
        if let Some(resource) = event.metadata.get("resource") {
            *metrics
                .checks_by_resource
                .entry(resource.clone())
                .or_insert(0) += 1;
        }

        // Update timing metrics
        if let Some(duration) = event.duration {
            metrics.avg_authz_time = if metrics.total_checks == 1 {
                duration
            } else {
                Duration::from_nanos(
                    (metrics.avg_authz_time.as_nanos() as u64 + duration.as_nanos() as u64) / 2,
                )
            };
        }

        metrics.last_authz_time = Some(event.timestamp);
        Ok(())
    }

    /// Record encryption event
    async fn record_encryption_event(&self, event: &SecurityEvent) -> Result<(), PrimalError> {
        let mut metrics = self.crypto_metrics.write().await;
        metrics.total_encryptions += 1;

        // Update byte count if available
        if let Some(bytes_str) = event.metadata.get("bytes") {
            if let Ok(bytes) = bytes_str.parse::<u64>() {
                metrics.total_bytes_encrypted += bytes;
            }
        }

        // Update algorithm-specific metrics
        if let Some(algorithm) = event.metadata.get("algorithm") {
            *metrics
                .encryptions_by_algorithm
                .entry(algorithm.clone())
                .or_insert(0) += 1;
        }

        // Update timing metrics
        if let Some(duration) = event.duration {
            metrics.avg_encryption_time = if metrics.total_encryptions == 1 {
                duration
            } else {
                Duration::from_nanos(
                    (metrics.avg_encryption_time.as_nanos() as u64 + duration.as_nanos() as u64)
                        / 2,
                )
            };
        }

        Ok(())
    }

    /// Record decryption event
    async fn record_decryption_event(&self, event: &SecurityEvent) -> Result<(), PrimalError> {
        let mut metrics = self.crypto_metrics.write().await;
        metrics.total_decryptions += 1;

        // Update byte count if available
        if let Some(bytes_str) = event.metadata.get("bytes") {
            if let Ok(bytes) = bytes_str.parse::<u64>() {
                metrics.total_bytes_decrypted += bytes;
            }
        }

        // Update timing metrics
        if let Some(duration) = event.duration {
            metrics.avg_decryption_time = if metrics.total_decryptions == 1 {
                duration
            } else {
                Duration::from_nanos(
                    (metrics.avg_decryption_time.as_nanos() as u64 + duration.as_nanos() as u64)
                        / 2,
                )
            };
        }

        Ok(())
    }

    /// Record audit event
    async fn record_audit_event(&self, event: &SecurityEvent) -> Result<(), PrimalError> {
        let mut metrics = self.audit_metrics.write().await;
        metrics.total_events += 1;

        // Update type-specific metrics
        if let Some(event_type) = event.metadata.get("type") {
            *metrics
                .events_by_type
                .entry(event_type.clone())
                .or_insert(0) += 1;
        }

        // Update severity-specific metrics
        if let Some(severity) = event.metadata.get("severity") {
            *metrics
                .events_by_severity
                .entry(severity.clone())
                .or_insert(0) += 1;
        }

        // Record failed writes
        if matches!(event.result, SecurityEventResult::Failed) {
            metrics.failed_writes += 1;
        }

        metrics.last_audit_time = Some(event.timestamp);
        Ok(())
    }

    /// Record session created event
    async fn record_session_created_event(&self, event: &SecurityEvent) -> Result<(), PrimalError> {
        let mut metrics = self.session_metrics.write().await;
        metrics.total_sessions_created += 1;
        metrics.active_sessions += 1;

        // Update type-specific metrics
        if let Some(session_type) = event.metadata.get("type") {
            *metrics
                .sessions_by_type
                .entry(session_type.clone())
                .or_insert(0) += 1;
        }

        Ok(())
    }

    /// Record session destroyed event
    async fn record_session_destroyed_event(
        &self,
        event: &SecurityEvent,
    ) -> Result<(), PrimalError> {
        let mut metrics = self.session_metrics.write().await;
        metrics.total_sessions_destroyed += 1;
        metrics.active_sessions = metrics.active_sessions.saturating_sub(1);

        // Update session duration if available
        if let Some(duration) = event.duration {
            metrics.avg_session_duration = if metrics.total_sessions_destroyed == 1 {
                duration
            } else {
                Duration::from_nanos(
                    (metrics.avg_session_duration.as_nanos() as u64 + duration.as_nanos() as u64)
                        / 2,
                )
            };
        }

        Ok(())
    }

    /// Record threat event
    async fn record_threat_event(&self, event: &SecurityEvent) -> Result<(), PrimalError> {
        let mut metrics = self.threat_metrics.write().await;
        metrics.total_threats += 1;

        // Update type-specific metrics
        if let Some(threat_type) = event.metadata.get("type") {
            *metrics
                .threats_by_type
                .entry(threat_type.clone())
                .or_insert(0) += 1;
        }

        // Update severity-specific metrics
        if let Some(severity) = event.metadata.get("severity") {
            *metrics
                .threats_by_severity
                .entry(severity.clone())
                .or_insert(0) += 1;
        }

        // Record blocked threats
        if matches!(event.result, SecurityEventResult::Blocked) {
            metrics.blocked_threats += 1;
        }

        metrics.last_threat_time = Some(event.timestamp);
        Ok(())
    }

    /// Get authentication metrics
    pub async fn get_authentication_metrics(&self) -> AuthenticationMetrics {
        self.auth_metrics.read().await.clone()
    }

    /// Get authorization metrics
    pub async fn get_authorization_metrics(&self) -> AuthorizationMetrics {
        self.authz_metrics.read().await.clone()
    }

    /// Get cryptography metrics
    pub async fn get_cryptography_metrics(&self) -> CryptographyMetrics {
        self.crypto_metrics.read().await.clone()
    }

    /// Get audit metrics
    pub async fn get_audit_metrics(&self) -> AuditMetrics {
        self.audit_metrics.read().await.clone()
    }

    /// Get session metrics
    pub async fn get_session_metrics(&self) -> SessionMetrics {
        self.session_metrics.read().await.clone()
    }

    /// Get threat metrics
    pub async fn get_threat_metrics(&self) -> ThreatMetrics {
        self.threat_metrics.read().await.clone()
    }

    /// Get security overview
    pub async fn get_security_overview(&self) -> SecurityOverview {
        let auth = self.get_authentication_metrics().await;
        let authz = self.get_authorization_metrics().await;
        let crypto = self.get_cryptography_metrics().await;
        let audit = self.get_audit_metrics().await;
        let session = self.get_session_metrics().await;
        let threat = self.get_threat_metrics().await;

        SecurityOverview {
            authentication: auth,
            authorization: authz,
            cryptography: crypto,
            audit,
            session,
            threat,
            timestamp: Utc::now(),
        }
    }
}

/// Complete security metrics overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityOverview {
    /// Authentication metrics
    pub authentication: AuthenticationMetrics,
    /// Authorization metrics
    pub authorization: AuthorizationMetrics,
    /// Cryptography metrics
    pub cryptography: CryptographyMetrics,
    /// Audit metrics
    pub audit: AuditMetrics,
    /// Session metrics
    pub session: SessionMetrics,
    /// Threat metrics
    pub threat: ThreatMetrics,
    /// Overview timestamp
    pub timestamp: DateTime<Utc>,
}

impl AuthenticationMetrics {
    fn new() -> Self {
        Self {
            total_attempts: 0,
            successful_authentications: 0,
            failed_authentications: 0,
            avg_auth_time: Duration::from_millis(0),
            attempts_by_method: HashMap::new(),
            last_auth_time: None,
        }
    }
}

impl AuthorizationMetrics {
    fn new() -> Self {
        Self {
            total_checks: 0,
            successful_authorizations: 0,
            failed_authorizations: 0,
            avg_authz_time: Duration::from_millis(0),
            checks_by_resource: HashMap::new(),
            last_authz_time: None,
        }
    }
}

impl CryptographyMetrics {
    fn new() -> Self {
        Self {
            total_encryptions: 0,
            total_decryptions: 0,
            total_bytes_encrypted: 0,
            total_bytes_decrypted: 0,
            avg_encryption_time: Duration::from_millis(0),
            avg_decryption_time: Duration::from_millis(0),
            encryptions_by_algorithm: HashMap::new(),
        }
    }
}

impl AuditMetrics {
    fn new() -> Self {
        Self {
            total_events: 0,
            events_by_type: HashMap::new(),
            events_by_severity: HashMap::new(),
            failed_writes: 0,
            last_audit_time: None,
        }
    }
}

impl SessionMetrics {
    fn new() -> Self {
        Self {
            total_sessions_created: 0,
            total_sessions_destroyed: 0,
            active_sessions: 0,
            expired_sessions: 0,
            avg_session_duration: Duration::from_millis(0),
            sessions_by_type: HashMap::new(),
        }
    }
}

impl ThreatMetrics {
    fn new() -> Self {
        Self {
            total_threats: 0,
            threats_by_type: HashMap::new(),
            threats_by_severity: HashMap::new(),
            blocked_threats: 0,
            last_threat_time: None,
        }
    }
}

impl Default for SecurityMetrics {
    fn default() -> Self {
        Self::new()
    }
}
