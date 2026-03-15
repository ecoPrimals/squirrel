// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primal_type_display() {
        assert_eq!(PrimalType::Coordinator.to_string(), "Coordinator");
        assert_eq!(PrimalType::Security.to_string(), "Security");
        assert_eq!(PrimalType::Orchestration.to_string(), "Orchestration");
        assert_eq!(PrimalType::Storage.to_string(), "Storage");
        assert_eq!(PrimalType::Compute.to_string(), "Compute");
        assert_eq!(PrimalType::AI.to_string(), "AI");
        assert_eq!(PrimalType::Network.to_string(), "Network");
        assert_eq!(
            PrimalType::Custom("test".into()).to_string(),
            "Custom(test)"
        );
    }

    #[test]
    fn test_primal_type_serde() {
        for pt in [
            PrimalType::Coordinator,
            PrimalType::Security,
            PrimalType::Orchestration,
            PrimalType::Storage,
            PrimalType::Compute,
            PrimalType::AI,
            PrimalType::Network,
            PrimalType::Custom("my_primal".into()),
        ] {
            let json = serde_json::to_string(&pt).expect("serialize");
            let deser: PrimalType = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(deser, pt);
        }
    }

    #[test]
    fn test_primal_state_display() {
        assert_eq!(PrimalState::Initializing.to_string(), "Initializing");
        assert_eq!(PrimalState::Starting.to_string(), "Starting");
        assert_eq!(PrimalState::Running.to_string(), "Running");
        assert_eq!(PrimalState::Stopping.to_string(), "Stopping");
        assert_eq!(PrimalState::Stopped.to_string(), "Stopped");
        assert_eq!(PrimalState::Error("oops".into()).to_string(), "Error: oops");
        assert_eq!(PrimalState::Restarting.to_string(), "Restarting");
        assert_eq!(PrimalState::Maintenance.to_string(), "Maintenance");
    }

    #[test]
    fn test_primal_state_default() {
        let state = PrimalState::default();
        assert_eq!(state, PrimalState::Stopped);
    }

    #[test]
    fn test_primal_state_serde() {
        for state in [
            PrimalState::Initializing,
            PrimalState::Starting,
            PrimalState::Running,
            PrimalState::Stopping,
            PrimalState::Stopped,
            PrimalState::Error("test error".into()),
            PrimalState::Restarting,
            PrimalState::Maintenance,
        ] {
            let json = serde_json::to_string(&state).expect("serialize");
            let deser: PrimalState = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(deser, state);
        }
    }

    #[test]
    fn test_primal_info_serde() {
        let info = PrimalInfo {
            name: "squirrel".to_string(),
            version: "1.0.0".to_string(),
            instance_id: Uuid::new_v4(),
            primal_type: PrimalType::AI,
            description: "AI coordinator".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: vec!["ai".to_string()],
            capabilities: vec!["inference".to_string()],
        };
        let json = serde_json::to_string(&info).expect("serialize");
        let deser: PrimalInfo = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.name, "squirrel");
        assert_eq!(deser.primal_type, PrimalType::AI);
    }

    #[test]
    fn test_primal_type_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(PrimalType::AI);
        set.insert(PrimalType::Storage);
        assert!(set.contains(&PrimalType::AI));
        assert!(!set.contains(&PrimalType::Compute));
    }
}
