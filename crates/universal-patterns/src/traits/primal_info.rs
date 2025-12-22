//! Primal information types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::Hash;
use uuid::Uuid;

/// Primal information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    /// Primal name
    pub name: String,

    /// Primal version
    pub version: String,

    /// Unique instance identifier
    pub instance_id: Uuid,

    /// Primal type
    pub primal_type: PrimalType,

    /// Description
    pub description: String,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// Capabilities
    pub capabilities: Vec<String>,
}

/// Primal type categories
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimalType {
    /// AI coordination and MCP protocol management
    Coordinator,
    /// Security and authentication management (BearDog)
    Security,
    /// Orchestration and task management (Songbird)
    Orchestration,
    /// Data storage and retrieval (NestGate)
    Storage,
    /// Compute and processing (Toadstool)
    Compute,
    /// AI primal (Squirrel)
    AI,
    /// Network primal
    Network,
    /// Custom/Other primal types
    Custom(String),
}

impl fmt::Display for PrimalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrimalType::Coordinator => write!(f, "Coordinator"),
            PrimalType::Security => write!(f, "Security"),
            PrimalType::Orchestration => write!(f, "Orchestration"),
            PrimalType::Storage => write!(f, "Storage"),
            PrimalType::Compute => write!(f, "Compute"),
            PrimalType::AI => write!(f, "AI"),
            PrimalType::Network => write!(f, "Network"),
            PrimalType::Custom(s) => write!(f, "Custom({})", s),
        }
    }
}

/// Primal state enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PrimalState {
    /// Primal is initializing
    Initializing,
    /// Primal is starting up
    Starting,
    /// Primal is running and healthy
    Running,
    /// Primal is stopping
    Stopping,
    /// Primal is stopped
    #[default]
    Stopped,
    /// Primal is in an error state
    Error(String),
    /// Primal is restarting
    Restarting,
    /// Primal is in maintenance mode
    Maintenance,
}

impl fmt::Display for PrimalState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrimalState::Initializing => write!(f, "Initializing"),
            PrimalState::Starting => write!(f, "Starting"),
            PrimalState::Running => write!(f, "Running"),
            PrimalState::Stopping => write!(f, "Stopping"),
            PrimalState::Stopped => write!(f, "Stopped"),
            PrimalState::Error(err) => write!(f, "Error: {}", err),
            PrimalState::Restarting => write!(f, "Restarting"),
            PrimalState::Maintenance => write!(f, "Maintenance"),
        }
    }
}
