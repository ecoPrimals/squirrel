//! Alert status definitions

use serde::{Serialize, Deserialize};

/// Status of an alert
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertStatus {
    /// Alert is active and requires attention
    Active,
    /// Alert has been acknowledged but not resolved
    Acknowledged,
    /// Alert has been resolved
    Resolved,
}

impl AlertStatus {
    /// Check if the alert is active
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }
    
    /// Check if the alert is acknowledged
    pub fn is_acknowledged(&self) -> bool {
        matches!(self, Self::Acknowledged)
    }
    
    /// Check if the alert is resolved
    pub fn is_resolved(&self) -> bool {
        matches!(self, Self::Resolved)
    }
} 