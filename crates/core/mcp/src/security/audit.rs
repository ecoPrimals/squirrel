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
