use serde::{Serialize, Deserialize};
use crate::health::status::Status;
use squirrel_core::error::Result;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use async_trait::async_trait;
use std::fmt::Debug;

/// Health information for a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Name of the component
    pub name: String,
    /// Health status of the component
    pub status: Status,
    /// Detailed message about the health status
    pub message: Option<String>,
    /// Timestamp when the health was last checked
    pub last_check: DateTime<Utc>,
    /// Additional details about the component health
    pub details: HashMap<String, String>,
}

impl ComponentHealth {
    /// Create a new component health
    #[must_use] pub fn new(name: String, status: Status, message: Option<String>) -> Self {
        Self {
            name,
            status,
            message,
            last_check: Utc::now(),
            details: HashMap::new(),
        }
    }
    
    /// Add details to the component health
    pub fn with_details(mut self, details: HashMap<String, String>) -> Self {
        self.details = details;
        self
    }
}

/// Health check trait for components that can report their health status
#[async_trait]
pub trait HealthCheck: Send + Sync + Debug {
    /// Get component name
    fn name(&self) -> &str;
    
    /// Check component health
    async fn check(&self) -> Result<ComponentHealth>;
}

/// Health status of a component
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Name of the component
    pub name: String,
    /// Health status of the component
    pub status: Status,
    /// Detailed message about the health status
    pub message: Option<String>,
    /// Timestamp when the health was last checked
    pub last_check: DateTime<Utc>,
    /// Additional details about the component health
    pub details: HashMap<String, String>,
} 