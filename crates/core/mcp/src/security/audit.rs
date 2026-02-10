// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Audit service for MCP security
//!
//! This module provides audit functionality for the MCP system.
//! Actual auditing is delegated to the BearDog framework.

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Audit event for security logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub user_id: Option<String>,
    pub resource_id: Option<String>,
    pub action: String,
    pub status: String,
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
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Log an audit event
    pub async fn log_event(&self, event: AuditEvent) {
        let mut events = self.events.write().await;
        events.push(event);
    }

    /// Get all audit events
    pub async fn get_events(&self) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events.clone()
    }

    /// Clear all audit events
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