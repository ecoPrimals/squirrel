// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Health status subscription types

use std::time::SystemTime;
use tokio::sync::broadcast;
use serde::{Serialize, Deserialize};

use crate::observability::ObservabilityResult;
use crate::observability::ObservabilityError;

use super::types::HealthStatus;
use super::event::HealthStatusEvent;

/// Async subscriber for health status events
pub struct HealthStatusSubscriber {
    receiver: tokio::sync::broadcast::Receiver<HealthStatusEvent>,
}

impl HealthStatusSubscriber {
    /// Create a new subscriber from a broadcast receiver
    pub fn new(receiver: broadcast::Receiver<HealthStatusEvent>) -> Self {
        Self { receiver }
    }
    
    /// Receive a health status update asynchronously
    pub async fn receive(&mut self) -> ObservabilityResult<HealthStatusUpdate> {
        match self.receiver.recv().await {
            Ok(event) => Ok(HealthStatusUpdate::from(event)),
            Err(e) => Err(ObservabilityError::HealthError(format!("Failed to receive status update: {}", e))),
        }
    }
    
    /// Try to receive a health status update without waiting
    pub fn try_recv(&mut self) -> ObservabilityResult<HealthStatusUpdate> {
        match self.receiver.try_recv() {
            Ok(event) => Ok(HealthStatusUpdate::from(event)),
            Err(e) => Err(ObservabilityError::HealthError(format!("Failed to receive status update: {}", e))),
        }
    }
}

/// Non-blocking subscriber for health status events
pub struct HealthStatusSubscriberNonBlocking {
    receiver: broadcast::Receiver<HealthStatusEvent>,
}

impl HealthStatusSubscriberNonBlocking {
    /// Create a new subscriber from a broadcast receiver
    pub fn new(receiver: broadcast::Receiver<HealthStatusEvent>) -> Self {
        Self { receiver }
    }
    
    /// Try to receive a health status update without blocking
    pub fn try_receive(&mut self) -> ObservabilityResult<Option<HealthStatusUpdate>> {
        match self.receiver.try_recv() {
            Ok(event) => {
                let update = HealthStatusUpdate::from(event);
                Ok(Some(update))
            },
            Err(broadcast::error::TryRecvError::Empty) => {
                // No messages available (not an error)
                Ok(None)
            },
            Err(broadcast::error::TryRecvError::Closed) => {
                // Channel closed (this is an error)
                Err(ObservabilityError::HealthError("Status update channel closed".to_string()))
            },
            Err(e) => {
                Err(ObservabilityError::HealthError(format!("Failed to receive status update: {}", e)))
            }
        }
    }
}

/// Health status update from subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatusUpdate {
    /// Component ID
    pub component_id: String,
    /// New health status
    pub status: HealthStatus,
    /// Details about the status change
    pub details: Option<String>,
    /// When the status changed
    pub timestamp: SystemTime,
}

impl From<HealthStatusEvent> for HealthStatusUpdate {
    fn from(event: HealthStatusEvent) -> Self {
        // For simplicity, just use the system status and timestamp
        // In a more complete implementation, we would include all components
        // that changed status since the last update
        let system_status = event.system_status;
        
        // Find the first component that has this status
        let mut component_id = "system".to_string();
        let mut details = None;
        
        for (id, health) in event.component_statuses.iter() {
            if health.status == system_status {
                component_id = id.clone();
                details = health.details.clone();
                break;
            }
        }
        
        Self {
            component_id,
            status: system_status,
            details,
            timestamp: SystemTime::now(),
        }
    }
} 