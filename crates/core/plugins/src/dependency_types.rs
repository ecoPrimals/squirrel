// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Data types for plugin dependency resolution.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Enhanced plugin dependency with semantic versioning support
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnhancedPluginDependency {
    /// Dependency ID
    pub id: Uuid,

    /// Dependency name for human-readable reference
    pub name: String,

    /// Version requirement using semantic versioning
    pub version_req: String,

    /// Whether the dependency is optional
    pub optional: bool,

    /// Dependency type for resolution ordering
    pub dependency_type: DependencyType,

    /// Platform constraints (e.g., "windows", "linux", "any")
    pub platform_constraints: Vec<String>,
}

/// Types of dependencies that affect resolution ordering
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DependencyType {
    /// Critical dependency that must be loaded first
    Critical,
    /// Normal runtime dependency
    Runtime,
    /// Development-time dependency (optional in production)
    Development,
    /// Plugin extension dependency
    Extension,
}

/// Result of dependency resolution
#[derive(Debug, Clone)]
pub struct ResolutionResult {
    /// Ordered list of plugin IDs for initialization
    pub initialization_order: Vec<Uuid>,

    /// Plugins that couldn't be resolved due to missing dependencies
    pub unresolved_plugins: Vec<(Uuid, Vec<String>)>,

    /// Version conflicts detected during resolution
    pub version_conflicts: Vec<VersionConflict>,

    /// Circular dependencies detected
    pub circular_dependencies: Vec<Vec<Uuid>>,

    /// Warnings generated during resolution
    pub warnings: Vec<String>,
}

/// Version conflict information
#[derive(Debug, Clone)]
pub struct VersionConflict {
    /// Plugin ID that has the conflict
    pub plugin_id: Uuid,

    /// Dependency name causing the conflict
    pub dependency_name: String,

    /// Required version
    pub required_version: String,

    /// Available version
    pub available_version: String,

    /// Plugins that depend on this conflicting dependency
    pub dependent_plugins: Vec<Uuid>,
}

/// Statistics about the dependency resolution state
#[derive(Debug, Clone)]
pub struct ResolutionStatistics {
    /// Total number of plugins registered
    pub total_plugins: usize,
    /// Total number of dependencies across all plugins
    pub total_dependencies: usize,
    /// Number of optional dependencies
    pub optional_dependencies: usize,
    /// Number of entries in the resolution cache
    pub cache_entries: usize,
}
