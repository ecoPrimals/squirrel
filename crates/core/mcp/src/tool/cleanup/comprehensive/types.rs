// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Core types and data structures for comprehensive cleanup system.
//!
//! This module defines the fundamental data structures used throughout the 
//! comprehensive cleanup system including resource types, identifiers, 
//! dependencies, allocations, and cleanup records.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Resource dependency type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    /// Memory allocation
    Memory,
    
    /// File handle
    File,
    
    /// Network connection
    Network,
    
    /// Database connection
    Database,
    
    /// Thread/Task
    Thread,
    
    /// Lock/Mutex
    Lock,
    
    /// Custom resource type
    Custom(String),
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Memory => write!(f, "Memory"),
            Self::File => write!(f, "File"),
            Self::Network => write!(f, "Network"),
            Self::Database => write!(f, "Database"),
            Self::Thread => write!(f, "Thread"),
            Self::Lock => write!(f, "Lock"),
            Self::Custom(name) => write!(f, "Custom({name})"),
        }
    }
}

/// Resource identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId {
    /// Resource type
    pub resource_type: ResourceType,
    
    /// Resource name/identifier
    pub name: String,
    
    /// Resource owner tool ID
    pub owner: String,
}

impl ResourceId {
    /// Create a new resource ID
    pub fn new(resource_type: ResourceType, name: impl Into<String>, owner: impl Into<String>) -> Self {
        Self {
            resource_type,
            name: name.into(),
            owner: owner.into(),
        }
    }
}

impl fmt::Display for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.resource_type, self.name)
    }
}

/// Resource dependency relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDependency {
    /// Parent resource
    pub parent: ResourceId,
    
    /// Child resource
    pub child: ResourceId,
    
    /// Whether the relationship is strong (child cannot exist without parent)
    pub is_strong: bool,
}

/// Resource allocation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    /// Resource ID
    pub id: ResourceId,
    
    /// Allocation timestamp
    pub allocated_at: DateTime<Utc>,
    
    /// Is the resource currently active
    pub is_active: bool,
    
    /// Resource size/quantity
    pub size: u64,
    
    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

/// Resource cleanup record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupRecord {
    /// Resource ID
    pub resource_id: ResourceId,
    
    /// Cleanup timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Whether cleanup was successful
    pub success: bool,
    
    /// Error message if cleanup failed
    pub error: Option<String>,
    
    /// Cleanup method used
    pub method: CleanupMethod,
    
    /// Duration of cleanup in milliseconds
    pub duration_ms: u64,
}

/// Cleanup method used
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CleanupMethod {
    /// Normal cleanup
    Normal,
    
    /// Forced cleanup
    Forced,
    
    /// Cascading cleanup (triggered by parent)
    Cascading,
    
    /// Auto-recovery cleanup
    Recovery,
    
    /// Timeout-triggered cleanup
    Timeout,
}

/// Resource cleanup strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CleanupStrategy {
    /// Normal cleanup with timeout
    Normal {
        /// Timeout in milliseconds
        timeout_ms: u64,
    },
    
    /// Forced cleanup after normal fails
    Forced {
        /// Force method description
        method: String,
    },
    
    /// Cascading cleanup (clean up children first)
    Cascading {
        /// Whether to continue if child cleanup fails
        continue_on_error: bool,
    },
    
    /// Custom cleanup with parameters
    Custom {
        /// Strategy name
        name: String,
        
        /// Strategy parameters
        params: HashMap<String, String>,
    },
}

impl Default for CleanupStrategy {
    fn default() -> Self {
        Self::Normal { timeout_ms: 5000 }
    }
}

/// Helper functions for working with cleanup types
impl CleanupStrategy {
    /// Create a normal cleanup strategy with specified timeout
    pub fn normal(timeout_ms: u64) -> Self {
        Self::Normal { timeout_ms }
    }
    
    /// Create a forced cleanup strategy
    pub fn forced(method: impl Into<String>) -> Self {
        Self::Forced { method: method.into() }
    }
    
    /// Create a cascading cleanup strategy
    pub fn cascading(continue_on_error: bool) -> Self {
        Self::Cascading { continue_on_error }
    }
    
    /// Create a custom cleanup strategy
    pub fn custom(name: impl Into<String>, params: HashMap<String, String>) -> Self {
        Self::Custom { name: name.into(), params }
    }
}

/// Default cleanup strategies for different resource types
impl ResourceType {
    /// Get the default cleanup strategy for this resource type
    pub fn default_strategy(&self) -> CleanupStrategy {
        match self {
            ResourceType::Memory => CleanupStrategy::normal(5000),
            ResourceType::File => CleanupStrategy::normal(2000),
            ResourceType::Network => CleanupStrategy::normal(10000),
            ResourceType::Database => CleanupStrategy::normal(5000),
            ResourceType::Thread => CleanupStrategy::forced("cancel"),
            ResourceType::Lock => CleanupStrategy::forced("release"),
            ResourceType::Custom(_) => CleanupStrategy::normal(5000),
        }
    }
}

/// Helper trait for resource-related operations
pub trait ResourceOperations {
    /// Check if a resource is of a specific type
    fn is_type(&self, resource_type: &ResourceType) -> bool;
    
    /// Get the owner of the resource
    fn owner(&self) -> &str;
}

impl ResourceOperations for ResourceId {
    fn is_type(&self, resource_type: &ResourceType) -> bool {
        &self.resource_type == resource_type
    }
    
    fn owner(&self) -> &str {
        &self.owner
    }
}

impl ResourceOperations for ResourceAllocation {
    fn is_type(&self, resource_type: &ResourceType) -> bool {
        &self.id.resource_type == resource_type
    }
    
    fn owner(&self) -> &str {
        &self.id.owner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_resource_id_creation() {
        let id = ResourceId::new(ResourceType::Memory, "test-memory", "test-tool");
        assert_eq!(id.resource_type, ResourceType::Memory);
        assert_eq!(id.name, "test-memory");
        assert_eq!(id.owner, "test-tool");
    }
    
    #[test]
    fn test_resource_id_display() {
        let id = ResourceId::new(ResourceType::Memory, "test-memory", "test-tool");
        assert_eq!(format!("{}", id), "Memory(test-memory)");
    }
    
    #[test]
    fn test_cleanup_strategy_creation() {
        let strategy = CleanupStrategy::normal(1000);
        match strategy {
            CleanupStrategy::Normal { timeout_ms } => assert_eq!(timeout_ms, 1000),
            _ => unreachable!("Expected Normal strategy"),
        }
    }
    
    #[test]
    fn test_default_strategies() {
        assert!(matches!(ResourceType::Memory.default_strategy(), CleanupStrategy::Normal { .. }));
        assert!(matches!(ResourceType::Thread.default_strategy(), CleanupStrategy::Forced { .. }));
    }
    
    #[test]
    fn test_resource_operations() {
        let id = ResourceId::new(ResourceType::Memory, "test", "owner");
        assert!(id.is_type(&ResourceType::Memory));
        assert!(!id.is_type(&ResourceType::File));
        assert_eq!(id.owner(), "owner");
    }
} 