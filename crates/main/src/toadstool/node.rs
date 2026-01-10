//! Compute node management
//!
//! This module defines types for managing compute nodes in the
//! `ToadStool` cluster.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::resource::AllocatedResources;

/// Compute node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeNode {
    pub node_id: String,
    pub node_type: NodeType,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub total_resources: AllocatedResources,
    pub available_resources: AllocatedResources,
    pub health: NodeHealth,
    pub last_heartbeat: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// Types of compute nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    CPU,
    GPU,
    Hybrid,
    Specialized(String),
}

/// Node health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Offline,
}
