// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Component health tracking

use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

use super::types::HealthStatus;

/// Health status of a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component ID
    pub component_id: String,
    /// Component name
    pub name: String,
    /// Health status
    pub status: HealthStatus,
    /// Additional details about the health status
    pub details: Option<String>,
    /// Last time the health was checked
    pub last_checked: u64,
    /// Last time the status changed
    pub last_status_change: u64,
    /// Optional metadata about the component
    pub metadata: HashMap<String, String>,
    /// Tags for the component
    pub tags: HashSet<String>,
}

impl ComponentHealth {
    /// Create a new component health
    pub fn new(component_id: impl Into<String>, name: impl Into<String>, status: HealthStatus) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        Self {
            component_id: component_id.into(),
            name: name.into(),
            status,
            details: None,
            last_checked: now,
            last_status_change: now,
            metadata: HashMap::new(),
            tags: HashSet::new(),
        }
    }
    
    /// Add details to the component health
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
    
    /// Add metadata to the component health
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Update the component status
    pub fn update_status(&mut self, status: HealthStatus, details: Option<String>) {
        // Check if status is actually changing
        let status_changing = self.status != status;
        
        // Update fields
        self.status = status;
        self.details = details;
        
        // Get current time
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Always update the last checked time
        self.last_checked = now;
        
        // Only update last status change time if status actually changed
        if status_changing {
            self.last_status_change = now;
        }
    }
    
    /// Check if the component is healthy
    pub fn is_healthy(&self) -> bool {
        self.status == HealthStatus::Healthy
    }
    
    /// Check if the component is degraded
    pub fn is_degraded(&self) -> bool {
        self.status == HealthStatus::Degraded
    }
    
    /// Check if the component is unhealthy
    pub fn is_unhealthy(&self) -> bool {
        self.status == HealthStatus::Unhealthy
    }
    
    /// Get time (in seconds) since the status last changed
    pub fn time_since_status_change(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now.saturating_sub(self.last_status_change)
    }
    
    /// Get the number of metadata items
    pub fn len(&self) -> usize {
        self.metadata.len()
    }
}

impl PartialEq<HealthStatus> for ComponentHealth {
    fn eq(&self, other: &HealthStatus) -> bool {
        self.status == *other
    }
} 