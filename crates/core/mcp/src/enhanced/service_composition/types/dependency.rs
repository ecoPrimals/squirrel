// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Dependency-related types for the service composition system

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Service dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependency {
    /// Dependency ID
    pub id: String,
    
    /// Dependency type
    pub dependency_type: DependencyType,
    
    /// Required service
    pub service_id: String,
    
    /// Required capabilities
    pub required_capabilities: Vec<String>,
    
    /// Dependency constraints
    pub constraints: Vec<DependencyConstraint>,
    
    /// Dependency metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Dependency types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DependencyType {
    /// Hard dependency (must be available)
    Hard,
    
    /// Soft dependency (optional)
    Soft,
    
    /// Sequential dependency (must execute in order)
    Sequential,
    
    /// Parallel dependency (can execute simultaneously)
    Parallel,
    
    /// Conditional dependency (depends on conditions)
    Conditional,
    
    /// Circular dependency (mutual dependency)
    Circular,
}

/// Dependency constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyConstraint {
    /// Constraint type
    pub constraint_type: DependencyConstraintType,
    
    /// Constraint value
    pub value: serde_json::Value,
    
    /// Constraint description
    pub description: String,
    
    /// Constraint metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Dependency constraint types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyConstraintType {
    /// Version constraint
    Version,
    
    /// Capability constraint
    Capability,
    
    /// Performance constraint
    Performance,
    
    /// Security constraint
    Security,
    
    /// Resource constraint
    Resource,
    
    /// Custom constraint
    Custom(String),
}

/// Dependency graph
#[derive(Debug, Clone, Default)]
pub struct DependencyGraph {
    /// Graph nodes (services)
    pub nodes: HashMap<String, DependencyNode>,
    
    /// Dependencies mapping (service_id -> dependencies)
    pub dependencies: HashMap<String, Vec<ServiceDependency>>,
    
    /// Reverse dependencies (service_id -> dependents)
    pub dependents: HashMap<String, Vec<String>>,
}

/// Dependency node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyNode {
    /// Service ID
    pub service_id: String,
    
    /// Service name
    pub service_name: String,
    
    /// Service version
    pub version: String,
    
    /// Node status
    pub status: DependencyNodeStatus,
    
    /// Node metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Dependency node status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DependencyNodeStatus {
    /// Service is available
    Available,
    
    /// Service is unavailable
    Unavailable,
    
    /// Service is degraded
    Degraded,
    
    /// Service status is unknown
    Unknown,
}

/// Dependency validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyValidationResult {
    /// Service ID
    pub service_id: String,
    
    /// Validation successful
    pub is_valid: bool,
    
    /// Missing dependencies
    pub missing_dependencies: Vec<String>,
    
    /// Circular dependencies
    pub circular_dependencies: Vec<Vec<String>>,
    
    /// Validation errors
    pub errors: Vec<String>,
    
    /// Validation warnings
    pub warnings: Vec<String>,
    
    /// Validation timestamp
    pub validated_at: DateTime<Utc>,
}

/// Dependency resolver trait
#[async_trait::async_trait]
pub trait DependencyResolver: Send + Sync + std::fmt::Debug {
    /// Resolve dependencies for a service
    async fn resolve_dependencies(
        &self,
        service_id: &str,
        dependencies: &[ServiceDependency],
    ) -> Result<Vec<ResolvedDependency>, crate::error::types::MCPError>;
    
    /// Check if a dependency is available
    async fn check_dependency_availability(
        &self,
        dependency: &ServiceDependency,
    ) -> Result<bool, crate::error::types::MCPError>;
    
    /// Get resolver name
    fn resolver_name(&self) -> &str;
}

/// Resolved dependency
#[derive(Debug, Clone)]
pub struct ResolvedDependency {
    /// Original dependency specification
    pub dependency: ServiceDependency,
    
    /// Resolved service ID
    pub resolved_service_id: String,
    
    /// Resolved endpoint
    pub endpoint: Option<String>,
    
    /// Resolution metadata
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Resolution timestamp
    pub resolved_at: DateTime<Utc>,
} 