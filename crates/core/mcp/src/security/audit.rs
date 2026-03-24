// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Audit service for MCP security
//!
//! This module provides audit functionality for the MCP system.
//! Actual auditing is delegated to the BearDog framework.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Audit event for security logging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event identifier.
    pub id: Uuid,
    /// When the event occurred (UTC).
    pub timestamp: DateTime<Utc>,
    /// High-level event category or name.
    pub event_type: String,
    /// Acting user id, if authenticated.
    pub user_id: Option<String>,
    /// Target resource id, if applicable.
    pub resource_id: Option<String>,
    /// Action verb or operation name.
    pub action: String,
    /// Outcome label (e.g. success, denied).
    pub status: String,
    /// Human-readable detail or error text.
    pub message: String,
}

/// Default audit service implementation
///
/// This provides a basic audit service that can be extended
/// or replaced with BearDog integration in the future.
#[derive(Debug, Clone)]
pub struct DefaultAuditService {
    events: Arc<RwLock<Vec<AuditEvent>>>,
}

impl DefaultAuditService {
    /// Create a new audit service
    #[must_use]
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Appends an audit event to the in-memory log.
    pub async fn log_event(&self, event: AuditEvent) {
        let mut events = self.events.write().await;
        events.push(event);
    }

    /// Returns a snapshot of all stored audit events.
    pub async fn get_events(&self) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events.clone()
    }

    /// Removes every audit event from storage.
    pub async fn clear_events(&self) {
        let mut events = self.events.write().await;
        events.clear();
    }
}

impl Default for DefaultAuditService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)] // Invariant or startup failure: unwrap/expect after validation
mod tests {
    use super::*;

    #[test]
    fn audit_event_serde_round_trip() {
        let e = AuditEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: "test".to_string(),
            user_id: None,
            resource_id: Some(String::new()),
            action: "a".to_string(),
            status: "ok".to_string(),
            message: "m".to_string(),
        };
        let json = serde_json::to_string(&e).expect("audit event serializes");
        let back: AuditEvent = serde_json::from_str(&json).expect("audit event deserializes");
        assert_eq!(back.id, e.id);
        assert_eq!(back.event_type, e.event_type);
        assert_eq!(back.resource_id, e.resource_id);
        assert_eq!(back.status, e.status);
    }

    #[tokio::test]
    async fn audit_service_lifecycle() {
        let s = DefaultAuditService::new();
        let _ = DefaultAuditService::default();
        let dbg = format!("{s:?}");
        assert!(dbg.contains("DefaultAuditService"));

        assert!(s.get_events().await.is_empty());

        let ev = AuditEvent {
            id: Uuid::nil(),
            timestamp: Utc::now(),
            event_type: "e".to_string(),
            user_id: Some("u".to_string()),
            resource_id: None,
            action: "act".to_string(),
            status: "success".to_string(),
            message: String::new(),
        };
        s.log_event(ev.clone()).await;
        let events = s.get_events().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, ev.id);

        s.clear_events().await;
        assert!(s.get_events().await.is_empty());
    }
}
