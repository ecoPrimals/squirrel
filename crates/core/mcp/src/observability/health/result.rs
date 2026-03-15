// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Health check result types

use std::collections::HashMap;
use std::time::SystemTime;
use serde::{Serialize, Deserialize};

use super::types::HealthStatus;

/// Result of a health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Health status
    pub status: HealthStatus,
    /// Additional details about the health status
    pub details: Option<String>,
}

impl HealthCheckResult {
    /// Create a new health check result
    pub fn new(status: HealthStatus, details: Option<String>) -> Self {
        Self { status, details }
    }

    /// Create a healthy result with no message
    pub fn healthy() -> Self {
        Self {
            status: HealthStatus::Healthy,
            details: None,
        }
    }
    
    /// Create a healthy result with a message
    pub fn healthy_with_message(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Healthy,
            details: Some(message.into()),
        }
    }
    
    /// Create a healthy result with details
    pub fn healthy_with_details(details: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Healthy,
            details: Some(details.into()),
        }
    }
    
    /// Create a degraded result with a message
    pub fn degraded(details: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Degraded,
            details: Some(details.into()),
        }
    }
    
    /// Create an unhealthy result with a message
    pub fn unhealthy(details: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            details: Some(details.into()),
        }
    }
    
    /// Create an unknown result with no message
    pub fn unknown() -> Self {
        Self {
            status: HealthStatus::Unknown,
            details: None,
        }
    }
    
    /// Create an unknown result with a message
    pub fn unknown_with_message(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Unknown,
            details: Some(message.into()),
        }
    }
    
    /// Create an unknown result with details
    pub fn unknown_with_details(details: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Unknown,
            details: Some(details.into()),
        }
    }
    
    /// Get the status of the result
    pub fn status(&self) -> HealthStatus {
        self.status
    }
    
    /// Get the message of the result
    pub fn message(&self) -> &str {
        match &self.details {
            Some(details) => details,
            None => "",
        }
    }
    
    /// Add a detail to the result
    pub fn with_detail(mut self, key: &str, value: &str) -> Self {
        let details = match &self.details {
            Some(details) => format!("{}; {}={}", details, key, value),
            None => format!("{}={}", key, value),
        };
        self.details = Some(details);
        self
    }
    
    /// Get the timestamp of when this result was created
    pub fn timestamp(&self) -> SystemTime {
        // For now, just return the current time
        // In future, we may want to store the timestamp in the result
        SystemTime::now()
    }
    
    /// Get the details as a key-value map
    pub fn details(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        
        if let Some(details) = &self.details {
            // Very simple parsing of the details string
            // In a more robust implementation, we would use a proper format
            for part in details.split(';') {
                let part = part.trim();
                if let Some(pos) = part.find('=') {
                    let key = part[..pos].trim();
                    let value = part[pos + 1..].trim();
                    map.insert(key.to_string(), value.to_string());
                }
            }
        }
        
        map
    }
} 