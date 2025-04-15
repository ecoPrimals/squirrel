//! Common types for the integration crate
//!
//! This module provides common types used across different integration components.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Context data representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextData {
    /// Context ID
    pub id: String,
    
    /// Context name
    pub name: String,
    
    /// Context content
    pub content: String,
    
    /// Context metadata
    pub metadata: HashMap<String, String>,
    
    /// Creation timestamp
    pub created_at: String,
    
    /// Last update timestamp
    pub updated_at: String,
}

/// Integration status data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusData {
    /// Integration component name
    pub component: String,
    
    /// Whether the component is online
    pub online: bool,
    
    /// Last heartbeat timestamp
    pub last_heartbeat: String,
    
    /// Error count
    pub error_count: u32,
    
    /// Current load (0-100)
    pub load: u8,
    
    /// Additional status information
    pub details: HashMap<String, String>,
} 