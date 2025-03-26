//! Dashboard update types.
//!
//! This module defines types for updating dashboard data and publishing updates.

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Types of dashboard updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DashboardUpdate {
    /// Full dashboard data update
    FullUpdate(crate::data::DashboardData),
    
    /// Partial update with only changed metrics
    MetricsUpdate {
        /// Updated metrics values
        metrics: HashMap<String, f64>,
        /// Timestamp of the update
        timestamp: DateTime<Utc>,
    },
    
    /// Alert triggered or updated
    AlertUpdate {
        /// The updated alert
        alert: crate::data::Alert,
        /// Timestamp of the update
        timestamp: DateTime<Utc>,
    },
    
    /// System resource usage update
    SystemUpdate {
        /// The updated system snapshot
        system: crate::data::SystemSnapshot,
        /// Timestamp of the update
        timestamp: DateTime<Utc>,
    },
    
    /// Network statistics update
    NetworkUpdate {
        /// The updated network snapshot
        network: crate::data::NetworkSnapshot,
        /// Timestamp of the update
        timestamp: DateTime<Utc>,
    },
    
    /// Request to acknowledge an alert
    AcknowledgeAlert {
        /// The ID of the alert to acknowledge
        alert_id: String,
        /// User who acknowledged the alert
        acknowledged_by: String,
        /// Timestamp of acknowledgment
        timestamp: DateTime<Utc>,
    },
    
    /// Configuration update
    ConfigUpdate {
        /// The updated configuration
        config: crate::config::DashboardConfig,
    },
} 