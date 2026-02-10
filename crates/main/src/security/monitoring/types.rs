// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Core security monitoring types
#![allow(dead_code)] // Security monitoring types awaiting activation
//!
//! This module contains the fundamental types used throughout the security
//! monitoring system, including events, severity levels, and patterns.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{Instant, SystemTime};
use uuid::Uuid;

use crate::observability::CorrelationId;

/// Security event types
///
/// Represents different categories of security events that can be monitored
/// and analyzed by the security monitoring system.
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
///
/// Represents a single security event with all relevant metadata for
/// tracking, analysis, and correlation.
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
///
/// Ordered from least to most severe for comparison.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventSeverity {
    Info,
    Warning,
    High,
    Critical,
}

/// Behavioral pattern for anomaly detection
#[derive(Debug, Clone)]
pub(crate) struct BehavioralPattern {
    /// Client IP address
    pub client_ip: String,

    /// User ID if authenticated
    pub user_id: Option<String>,

    /// Request patterns observed
    pub request_patterns: VecDeque<RequestPattern>,

    /// First observation time
    pub first_seen: Instant,

    /// Last activity time
    pub last_activity: Instant,

    /// Total request count
    pub total_requests: u64,

    /// Failed request count
    pub failed_requests: u64,

    /// Violation count
    pub violation_count: u32,
}

/// Request pattern for behavioral analysis
#[derive(Debug, Clone)]
pub(crate) struct RequestPattern {
    /// Request timestamp
    pub timestamp: Instant,

    /// Endpoint accessed
    pub endpoint: String,

    /// Request success status
    pub success: bool,

    /// Response time in milliseconds
    pub response_time_ms: u64,
}

impl SecurityEvent {
    /// Create a new security event
    pub fn new(
        event_type: SecurityEventType,
        source_ip: String,
        severity: EventSeverity,
        source_component: String,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type,
            timestamp: SystemTime::now(),
            source_ip,
            user_agent: None,
            correlation_id,
            severity,
            metadata: HashMap::new(),
            source_component,
        }
    }

    /// Add metadata to the event
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set user agent
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }
}

impl BehavioralPattern {
    /// Create a new behavioral pattern
    pub fn new(client_ip: String, user_id: Option<String>) -> Self {
        let now = Instant::now();
        Self {
            client_ip,
            user_id,
            request_patterns: VecDeque::new(),
            first_seen: now,
            last_activity: now,
            total_requests: 0,
            failed_requests: 0,
            violation_count: 0,
        }
    }

    /// Update pattern with new request
    pub fn record_request(&mut self, endpoint: String, success: bool, response_time_ms: u64) {
        self.total_requests += 1;
        if !success {
            self.failed_requests += 1;
        }
        self.last_activity = Instant::now();

        // Add request pattern
        self.request_patterns.push_back(RequestPattern {
            timestamp: Instant::now(),
            endpoint,
            success,
            response_time_ms,
        });

        // Limit history size
        while self.request_patterns.len() > 1000 {
            self.request_patterns.pop_front();
        }
    }

    /// Calculate failure rate
    pub fn failure_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.failed_requests as f64 / self.total_requests as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_event_creation() {
        let correlation_id = CorrelationId::new();
        let event = SecurityEvent::new(
            SecurityEventType::Authentication {
                success: true,
                user_id: Some("user123".to_string()),
                method: "password".to_string(),
            },
            "192.168.1.1".to_string(),
            EventSeverity::Info,
            "auth_service".to_string(),
            correlation_id,
        );

        assert!(event.event_id != Uuid::nil());
        assert_eq!(event.source_ip, "192.168.1.1");
        assert_eq!(event.severity, EventSeverity::Info);
    }

    #[test]
    fn test_security_event_builder() {
        let correlation_id = CorrelationId::new();
        let event = SecurityEvent::new(
            SecurityEventType::RateLimitViolation {
                client_ip: "192.168.1.1".to_string(),
                endpoint: "/api/data".to_string(),
                violation_count: 5,
            },
            "192.168.1.1".to_string(),
            EventSeverity::Warning,
            "rate_limiter".to_string(),
            correlation_id,
        )
        .with_metadata("endpoint", "/api/data")
        .with_user_agent("curl/7.68.0");

        assert!(event.user_agent.is_some());
        assert!(event.metadata.contains_key("endpoint"));
    }

    #[test]
    fn test_behavioral_pattern_update() {
        let mut pattern = BehavioralPattern::new("192.168.1.1".to_string(), None);

        pattern.record_request("/api/data".to_string(), true, 100);
        pattern.record_request("/api/data".to_string(), true, 150);
        pattern.record_request("/api/users".to_string(), false, 200);

        assert_eq!(pattern.total_requests, 3);
        assert_eq!(pattern.failed_requests, 1);
        assert_eq!(pattern.request_patterns.len(), 3);
        assert!((pattern.failure_rate() - 0.333).abs() < 0.01);
    }

    #[test]
    fn test_event_severity_ordering() {
        assert!(EventSeverity::Info < EventSeverity::Warning);
        assert!(EventSeverity::Warning < EventSeverity::High);
        assert!(EventSeverity::High < EventSeverity::Critical);
    }
}
