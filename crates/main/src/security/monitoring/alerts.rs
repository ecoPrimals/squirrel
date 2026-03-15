// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Security alert types and generation
//!
//! This module handles security alert creation, classification, and
//! severity escalation based on security events and behavioral analysis.

use serde::Serialize;
use std::collections::HashMap;
use std::time::SystemTime;
use uuid::Uuid;

use super::types::{EventSeverity, SecurityEvent};

/// Security alert
///
/// Represents an alert generated from security events, with severity,
/// recommended actions, and affected entities.
#[derive(Debug, Clone, Serialize)]
pub struct SecurityAlert {
    /// Alert identifier
    pub alert_id: Uuid,

    /// Alert type
    pub alert_type: AlertType,

    /// Alert severity
    pub severity: EventSeverity,

    /// Alert title
    pub title: String,

    /// Alert description
    pub description: String,

    /// Related events
    pub related_events: Vec<Uuid>,

    /// Affected entities (IPs, users, resources)
    pub affected_entities: Vec<String>,

    /// Recommended actions
    pub recommended_actions: Vec<String>,

    /// Timestamp when alert was generated
    pub generated_at: SystemTime,

    /// Additional alert metadata
    pub metadata: HashMap<String, String>,
}

/// Alert types
///
/// Categories of security alerts that can be generated.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum AlertType {
    /// Multiple failed authentication attempts (brute force)
    BruteForceAttempt,

    /// Excessive rate limit violations (DoS attack)
    RateLimitAbuse,

    /// Pattern of input validation violations (injection attack)
    InputValidationAbuse,

    /// Suspicious behavioral pattern detected
    SuspiciousBehavior,

    /// Security policy violation
    PolicyViolation,

    /// Anomalous access pattern
    AnomalousAccess,

    /// Multiple concurrent sessions
    ExcessiveConcurrentSessions,

    /// High failure rate detected
    HighFailureRate,

    /// System compromise indicators
    CompromiseIndicator,
}

impl SecurityAlert {
    /// Create a new security alert
    pub fn new(
        alert_type: AlertType,
        severity: EventSeverity,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            alert_id: Uuid::new_v4(),
            alert_type,
            severity,
            title: title.into(),
            description: description.into(),
            related_events: Vec::new(),
            affected_entities: Vec::new(),
            recommended_actions: Vec::new(),
            generated_at: SystemTime::now(),
            metadata: HashMap::new(),
        }
    }

    /// Add a related event
    pub fn with_event(mut self, event_id: Uuid) -> Self {
        self.related_events.push(event_id);
        self
    }

    /// Add an affected entity
    pub fn with_affected_entity(mut self, entity: impl Into<String>) -> Self {
        self.affected_entities.push(entity.into());
        self
    }

    /// Add a recommended action
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.recommended_actions.push(action.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Create an alert from a security event
    pub fn from_event(
        event: &SecurityEvent,
        alert_type: AlertType,
        title: impl Into<String>,
    ) -> Self {
        let severity = event.severity.clone();
        let description = format!("Alert generated from event {}", event.event_id);

        Self::new(alert_type, severity, title, description)
            .with_event(event.event_id)
            .with_affected_entity(&event.source_ip)
    }

    /// Escalate alert severity
    pub fn escalate(&mut self) {
        self.severity = match self.severity {
            EventSeverity::Info => EventSeverity::Warning,
            EventSeverity::Warning => EventSeverity::High,
            EventSeverity::High => EventSeverity::Critical,
            EventSeverity::Critical => EventSeverity::Critical, // Already at max
        };
    }
}

/// Alert builder for complex alert construction
pub struct AlertBuilder {
    alert_type: AlertType,
    severity: EventSeverity,
    title: String,
    description: String,
    events: Vec<Uuid>,
    entities: Vec<String>,
    actions: Vec<String>,
    metadata: HashMap<String, String>,
}

impl AlertBuilder {
    /// Create a new alert builder
    pub fn new(alert_type: AlertType) -> Self {
        Self {
            alert_type,
            severity: EventSeverity::Warning,
            title: String::new(),
            description: String::new(),
            events: Vec::new(),
            entities: Vec::new(),
            actions: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set severity
    pub fn severity(mut self, severity: EventSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Set title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Add related events
    pub fn add_events(mut self, events: impl IntoIterator<Item = Uuid>) -> Self {
        self.events.extend(events);
        self
    }

    /// Add affected entities
    pub fn add_entities(mut self, entities: impl IntoIterator<Item = String>) -> Self {
        self.entities.extend(entities);
        self
    }

    /// Add recommended actions
    pub fn add_actions(mut self, actions: impl IntoIterator<Item = String>) -> Self {
        self.actions.extend(actions);
        self
    }

    /// Add metadata
    pub fn add_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Build the alert
    pub fn build(self) -> SecurityAlert {
        SecurityAlert {
            alert_id: Uuid::new_v4(),
            alert_type: self.alert_type,
            severity: self.severity,
            title: self.title,
            description: self.description,
            related_events: self.events,
            affected_entities: self.entities,
            recommended_actions: self.actions,
            generated_at: SystemTime::now(),
            metadata: self.metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observability::CorrelationId;
    use crate::security::monitoring::types::{SecurityEvent, SecurityEventType};

    #[test]
    fn test_alert_creation() {
        let alert = SecurityAlert::new(
            AlertType::BruteForceAttempt,
            EventSeverity::High,
            "Multiple Authentication Failures",
            "Detected 10 failed login attempts from IP 192.168.1.1",
        );

        assert!(alert.alert_id != Uuid::nil());
        assert_eq!(alert.alert_type, AlertType::BruteForceAttempt);
        assert_eq!(alert.severity, EventSeverity::High);
    }

    #[test]
    fn test_alert_builder() {
        let alert = AlertBuilder::new(AlertType::BruteForceAttempt)
            .severity(EventSeverity::Critical)
            .title("Brute Force Attack Detected")
            .description("Coordinated attack from multiple IPs")
            .add_entities(vec!["192.168.1.1".to_string(), "192.168.1.2".to_string()])
            .add_actions(vec![
                "Block IPs".to_string(),
                "Notify security team".to_string(),
            ])
            .add_metadata("attack_pattern", "distributed")
            .build();

        assert_eq!(alert.severity, EventSeverity::Critical);
        assert_eq!(alert.affected_entities.len(), 2);
        assert_eq!(alert.recommended_actions.len(), 2);
        assert!(alert.metadata.contains_key("attack_pattern"));
    }

    #[test]
    fn test_alert_escalation() {
        let mut alert = SecurityAlert::new(
            AlertType::RateLimitAbuse,
            EventSeverity::Warning,
            "Rate Limit Violation",
            "Client exceeded rate limits",
        );

        alert.escalate();
        assert_eq!(alert.severity, EventSeverity::High);

        alert.escalate();
        assert_eq!(alert.severity, EventSeverity::Critical);

        alert.escalate();
        assert_eq!(alert.severity, EventSeverity::Critical); // Stays at max
    }

    #[test]
    fn test_alert_from_event() {
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
        );

        let alert =
            SecurityAlert::from_event(&event, AlertType::RateLimitAbuse, "Rate Limit Exceeded");

        assert_eq!(alert.related_events.len(), 1);
        assert_eq!(alert.related_events[0], event.event_id);
        assert!(alert.affected_entities.contains(&"192.168.1.1".to_string()));
    }
}
