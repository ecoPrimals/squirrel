// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Plugin dependency resolution system
//!
//! This module provides comprehensive dependency resolution capabilities for the plugin system,
//! including topological sorting, circular dependency detection, version conflict resolution,
//! and initialization ordering.

use anyhow::Result;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::PluginError;
use crate::plugin::Plugin;

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

/// Plugin dependency resolver
pub struct DependencyResolver {
    /// Registry of available plugins
    plugins: HashMap<Uuid, Arc<dyn Plugin>>,

    /// Enhanced dependency mapping
    dependencies: HashMap<Uuid, Vec<EnhancedPluginDependency>>,

    /// Name to ID mapping for quick lookups
    name_to_id: HashMap<String, Uuid>,

    /// Version mapping for conflict detection
    plugin_versions: HashMap<Uuid, Version>,

    /// Resolution cache for performance optimization
    resolution_cache: HashMap<String, ResolutionResult>,

    /// Current platform for platform-specific resolution
    current_platform: String,
}

impl DependencyResolver {
    /// Create a new dependency resolver
    #[must_use]
    pub fn new() -> Self {
        let current_platform = if cfg!(target_os = "windows") {
            "windows".to_string()
        } else if cfg!(target_os = "linux") {
            "linux".to_string()
        } else if cfg!(target_os = "macos") {
            "macos".to_string()
        } else {
            "unknown".to_string()
        };

        Self {
            plugins: HashMap::new(),
            dependencies: HashMap::new(),
            name_to_id: HashMap::new(),
            plugin_versions: HashMap::new(),
            resolution_cache: HashMap::new(),
            current_platform,
        }
    }

    /// Register a plugin with the dependency resolver
    ///
    /// # Errors
    ///
    /// Returns [`anyhow::Error`] if the plugin version is invalid or registration fails.
    pub fn register_plugin(&mut self, plugin: Arc<dyn Plugin>) -> Result<()> {
        let metadata = plugin.metadata();
        let id = metadata.id;

        // Parse version
        let version = Version::parse(&metadata.version).map_err(|e| {
            PluginError::InvalidVersion(format!("Invalid version {}: {}", metadata.version, e))
        })?;

        // Clone metadata before moving plugin
        let metadata_clone = metadata.clone();

        // Register plugin
        self.plugins.insert(id, plugin);
        self.name_to_id.insert(metadata_clone.name.clone(), id);
        self.plugin_versions.insert(id, version);

        // Convert legacy dependencies to enhanced dependencies
        let enhanced_deps: Vec<EnhancedPluginDependency> = metadata_clone
            .dependencies
            .iter()
            .map(|dep_id| EnhancedPluginDependency {
                id: *dep_id,
                name: format!("dependency-{dep_id}"),
                version_req: "*".to_string(), // Default to any version
                optional: false,
                dependency_type: DependencyType::Runtime,
                platform_constraints: vec!["any".to_string()],
            })
            .collect();

        self.dependencies.insert(id, enhanced_deps);

        debug!(
            "Registered plugin {} with dependency resolver",
            metadata_clone.name
        );
        Ok(())
    }

    /// Register enhanced dependencies for a plugin
    ///
    /// # Errors
    ///
    /// Returns [`anyhow::Error`] if dependency registration fails.
    pub fn register_enhanced_dependencies(
        &mut self,
        plugin_id: Uuid,
        dependencies: Vec<EnhancedPluginDependency>,
    ) -> Result<()> {
        self.dependencies.insert(plugin_id, dependencies);
        Ok(())
    }

    /// Resolve all plugin dependencies and return initialization order
    ///
    /// # Errors
    ///
    /// Returns [`anyhow::Error`] if validation or topological resolution fails.
    pub fn resolve_dependencies(&mut self) -> Result<ResolutionResult> {
        info!("Starting plugin dependency resolution");

        // Generate cache key based on current plugin state
        let cache_key = self.generate_cache_key();

        // Check cache first
        if let Some(cached_result) = self.resolution_cache.get(&cache_key) {
            debug!("Using cached resolution result");
            return Ok(cached_result.clone());
        }

        let mut result = ResolutionResult {
            initialization_order: Vec::new(),
            unresolved_plugins: Vec::new(),
            version_conflicts: Vec::new(),
            circular_dependencies: Vec::new(),
            warnings: Vec::new(),
        };

        // Step 1: Validate all dependencies exist and versions are compatible
        self.validate_dependencies(&mut result)?;

        // Step 2: Detect circular dependencies
        self.detect_circular_dependencies(&mut result)?;

        // Step 3: Resolve version conflicts
        self.resolve_version_conflicts(&mut result);

        // Step 4: Filter platform-specific dependencies
        self.filter_platform_dependencies(&mut result);

        // Step 5: Perform topological sort for initialization order
        self.topological_sort(&mut result)?;

        // Step 6: Optimize order by dependency types
        self.optimize_initialization_order(&mut result);

        // Cache the result
        self.resolution_cache.insert(cache_key, result.clone());

        info!(
            "Dependency resolution completed. Order: {:?}",
            result.initialization_order
        );
        Ok(result)
    }

    /// Validate that all dependencies exist and are compatible
    fn validate_dependencies(&self, result: &mut ResolutionResult) -> Result<()> {
        debug!("Validating plugin dependencies");

        for (plugin_id, deps) in &self.dependencies {
            let _plugin = self
                .plugins
                .get(plugin_id)
                .ok_or_else(|| PluginError::PluginNotFound(plugin_id.to_string()))?;

            let mut missing_deps = Vec::new();

            for dep in deps {
                // Check if dependency exists
                if !self.name_to_id.contains_key(&dep.name) && !dep.optional {
                    missing_deps.push(dep.name.clone());
                    continue;
                }

                if let Some(&dep_id) = self.name_to_id.get(&dep.name) {
                    // Check version compatibility
                    if let Some(dep_version) = self.plugin_versions.get(&dep_id) {
                        let version_req = VersionReq::parse(&dep.version_req).map_err(|e| {
                            PluginError::InvalidVersion(format!(
                                "Invalid version requirement {}: {}",
                                dep.version_req, e
                            ))
                        })?;

                        if !version_req.matches(dep_version) {
                            result.version_conflicts.push(VersionConflict {
                                plugin_id: *plugin_id,
                                dependency_name: dep.name.clone(),
                                required_version: dep.version_req.clone(),
                                available_version: dep_version.to_string(),
                                dependent_plugins: vec![*plugin_id],
                            });
                        }
                    }
                }
            }

            if !missing_deps.is_empty() {
                result.unresolved_plugins.push((*plugin_id, missing_deps));
            }
        }

        Ok(())
    }

    /// Detect circular dependencies in the plugin graph
    fn detect_circular_dependencies(&self, result: &mut ResolutionResult) -> Result<()> {
        debug!("Detecting circular dependencies");

        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for &plugin_id in self.plugins.keys() {
            if !visited.contains(&plugin_id)
                && self.has_cycle_dfs(plugin_id, &mut visited, &mut rec_stack, &mut path, result)?
            {
                // Cycle detected and added to result
            }
        }

        Ok(())
    }

    /// DFS helper for cycle detection
    fn has_cycle_dfs(
        &self,
        plugin_id: Uuid,
        visited: &mut HashSet<Uuid>,
        rec_stack: &mut HashSet<Uuid>,
        path: &mut Vec<Uuid>,
        result: &mut ResolutionResult,
    ) -> Result<bool> {
        visited.insert(plugin_id);
        rec_stack.insert(plugin_id);
        path.push(plugin_id);

        if let Some(deps) = self.dependencies.get(&plugin_id) {
            for dep in deps {
                if let Some(&dep_id) = self.name_to_id.get(&dep.name) {
                    if !visited.contains(&dep_id) {
                        if self.has_cycle_dfs(dep_id, visited, rec_stack, path, result)? {
                            return Ok(true);
                        }
                    } else if rec_stack.contains(&dep_id) {
                        // Found cycle - extract the cycle from path
                        if let Some(cycle_start) = path.iter().position(|&id| id == dep_id) {
                            let cycle = path[cycle_start..].to_vec();
                            result.circular_dependencies.push(cycle);
                            return Ok(true);
                        }
                    }
                }
            }
        }

        rec_stack.remove(&plugin_id);
        path.pop();
        Ok(false)
    }

    /// Attempt to resolve version conflicts
    #[expect(clippy::unused_self, reason = "Consistent resolver API")]
    fn resolve_version_conflicts(&self, result: &mut ResolutionResult) {
        debug!("Resolving version conflicts");

        for conflict in &result.version_conflicts {
            // For now, log warnings - in a full implementation, we might:
            // 1. Try to find compatible versions
            // 2. Suggest version upgrades
            // 3. Provide fallback strategies
            warn!(
                "Version conflict for plugin {}: requires {} but {} is available",
                conflict.dependency_name, conflict.required_version, conflict.available_version
            );

            result.warnings.push(format!(
                "Version conflict: {} requires {} but {} is available",
                conflict.dependency_name, conflict.required_version, conflict.available_version
            ));
        }
    }

    /// Filter dependencies based on current platform
    fn filter_platform_dependencies(&self, result: &mut ResolutionResult) {
        debug!(
            "Filtering platform-specific dependencies for platform: {}",
            self.current_platform
        );

        // Track plugins that should be excluded due to platform constraints
        let mut excluded_plugins = HashSet::new();

        for (plugin_id, deps) in &self.dependencies {
            for dep in deps {
                // Check if dependency has platform constraints
                if !dep.platform_constraints.is_empty()
                    && !dep.platform_constraints.contains(&"any".to_string())
                    && !dep.platform_constraints.contains(&self.current_platform)
                    && !dep.optional
                {
                    // Non-optional platform-specific dependency that doesn't match current platform
                    excluded_plugins.insert(*plugin_id);
                    result.warnings.push(format!(
                                "Plugin {} excluded due to platform constraint: requires {:?}, current platform: {}",
                                plugin_id, dep.platform_constraints, self.current_platform
                            ));
                }
            }
        }

        // Remove excluded plugins from unresolved list if they were there due to platform issues
        result
            .unresolved_plugins
            .retain(|(plugin_id, _)| !excluded_plugins.contains(plugin_id));
    }

    /// Perform topological sort to determine initialization order
    fn topological_sort(&self, result: &mut ResolutionResult) -> Result<()> {
        debug!("Performing topological sort for initialization order");

        let mut in_degree = HashMap::new();
        let mut adj_list = HashMap::new();

        // Initialize in-degree and adjacency list
        for &plugin_id in self.plugins.keys() {
            in_degree.insert(plugin_id, 0);
            adj_list.insert(plugin_id, Vec::new());
        }

        // Build graph and calculate in-degrees
        for (plugin_id, deps) in &self.dependencies {
            for dep in deps {
                let dep_id = dep.id;
                if let Some(adj_list_entry) = adj_list.get_mut(&dep_id) {
                    adj_list_entry.push(*plugin_id);
                }
                if let Some(in_degree_entry) = in_degree.get_mut(plugin_id) {
                    *in_degree_entry += 1;
                } else {
                    warn!(
                        "Plugin {} not found in in_degree map during topological sort",
                        plugin_id
                    );
                }
            }
        }

        // Kahn's algorithm for topological sorting
        let mut queue = VecDeque::new();

        // Add all nodes with in-degree 0 to queue
        for (&plugin_id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(plugin_id);
            }
        }

        let mut topo_order = Vec::new();

        while let Some(current) = queue.pop_front() {
            topo_order.push(current);

            // For each neighbor of current
            if let Some(neighbors) = adj_list.get(&current) {
                for &neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(&neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
        }

        // Check if all plugins were processed (no cycles)
        if topo_order.len() != self.plugins.len() {
            error!("Topological sort failed - circular dependencies detected");
            return Err(PluginError::CircularDependency(
                "Cannot resolve plugin dependencies due to cycles".to_string(),
            )
            .into());
        }

        result.initialization_order = topo_order;
        Ok(())
    }

    /// Optimize initialization order based on dependency types
    fn optimize_initialization_order(&self, result: &mut ResolutionResult) {
        debug!("Optimizing initialization order by dependency types");

        // Separate plugins by dependency types for better ordering
        let mut critical_plugins = Vec::new();
        let mut runtime_plugins = Vec::new();
        let mut extension_plugins = Vec::new();
        let mut other_plugins = Vec::new();

        for &plugin_id in &result.initialization_order {
            let has_critical_deps = self.dependencies.get(&plugin_id).is_some_and(|deps| {
                deps.iter()
                    .any(|d| d.dependency_type == DependencyType::Critical)
            });

            let has_extension_deps = self.dependencies.get(&plugin_id).is_some_and(|deps| {
                deps.iter()
                    .any(|d| d.dependency_type == DependencyType::Extension)
            });

            if has_critical_deps {
                critical_plugins.push(plugin_id);
            } else if has_extension_deps {
                extension_plugins.push(plugin_id);
            } else if self.dependencies.contains_key(&plugin_id) {
                runtime_plugins.push(plugin_id);
            } else {
                other_plugins.push(plugin_id);
            }
        }

        // Rebuild order: critical first, then runtime, then extensions, then others
        let mut optimized_order = Vec::new();
        optimized_order.extend(critical_plugins);
        optimized_order.extend(runtime_plugins);
        optimized_order.extend(other_plugins);
        optimized_order.extend(extension_plugins);

        result.initialization_order = optimized_order;
    }

    /// Generate a cache key based on current plugin and dependency state
    fn generate_cache_key(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Hash plugin IDs and versions
        for (&id, version) in &self.plugin_versions {
            id.hash(&mut hasher);
            version.to_string().hash(&mut hasher);
        }

        // Hash dependency information
        for (id, deps) in &self.dependencies {
            id.hash(&mut hasher);
            for dep in deps {
                dep.name.hash(&mut hasher);
                dep.version_req.hash(&mut hasher);
                dep.optional.hash(&mut hasher);
            }
        }

        format!("resolver_cache_{:x}", hasher.finish())
    }

    /// Get plugins that depend on a specific plugin
    pub fn get_dependents(&self, plugin_id: Uuid) -> Vec<Uuid> {
        let plugin_name = self
            .plugins
            .get(&plugin_id)
            .map(|p| p.metadata().name.clone());

        plugin_name.map_or_else(Vec::new, |name| {
            self.dependencies
                .iter()
                .filter_map(|(id, deps)| {
                    if deps.iter().any(|d| d.name == name) {
                        Some(*id)
                    } else {
                        None
                    }
                })
                .collect()
        })
    }

    /// Clear the resolution cache
    pub fn clear_cache(&mut self) {
        self.resolution_cache.clear();
        debug!("Resolution cache cleared");
    }

    /// Get statistics about the current resolution state
    pub fn get_statistics(&self) -> ResolutionStatistics {
        let total_plugins = self.plugins.len();
        let total_dependencies = self.dependencies.values().map(std::vec::Vec::len).sum();
        let optional_dependencies = self
            .dependencies
            .values()
            .flat_map(|deps| deps.iter())
            .filter(|dep| dep.optional)
            .count();

        ResolutionStatistics {
            total_plugins,
            total_dependencies,
            optional_dependencies,
            cache_entries: self.resolution_cache.len(),
        }
    }
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

impl Default for DependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::PluginMetadata;
    use async_trait::async_trait;
    use std::any::Any;

    struct MockPlugin {
        metadata: PluginMetadata,
    }

    impl MockPlugin {
        fn new(name: &str, version: &str) -> Self {
            Self {
                metadata: PluginMetadata::new(name, version, "Test plugin", "Test"),
            }
        }
    }

    #[async_trait]
    impl Plugin for MockPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }

        async fn initialize(&self) -> Result<()> {
            Ok(())
        }

        async fn shutdown(&self) -> Result<()> {
            Ok(())
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[tokio::test]
    async fn test_dependency_resolution() {
        let mut resolver = DependencyResolver::new();

        // Create test plugins
        let plugin_a = Arc::new(MockPlugin::new("plugin-a", "1.0.0"));
        let plugin_b = Arc::new(MockPlugin::new("plugin-b", "1.0.0"));
        let plugin_c = Arc::new(MockPlugin::new("plugin-c", "1.0.0"));

        // Register plugins
        resolver
            .register_plugin(plugin_a.clone())
            .expect("should succeed");
        resolver
            .register_plugin(plugin_b.clone())
            .expect("should succeed");
        resolver
            .register_plugin(plugin_c.clone())
            .expect("should succeed");

        // Set up dependencies: C -> B -> A
        let dep_a_to_b = EnhancedPluginDependency {
            id: plugin_b.metadata().id,
            name: "plugin-b".to_string(),
            version_req: "^1.0.0".to_string(),
            optional: false,
            dependency_type: DependencyType::Runtime,
            platform_constraints: vec!["any".to_string()],
        };

        let dep_b_to_c = EnhancedPluginDependency {
            id: plugin_c.metadata().id,
            name: "plugin-c".to_string(),
            version_req: "^1.0.0".to_string(),
            optional: false,
            dependency_type: DependencyType::Runtime,
            platform_constraints: vec!["any".to_string()],
        };

        resolver
            .register_enhanced_dependencies(plugin_a.metadata().id, vec![dep_a_to_b])
            .expect("should succeed");
        resolver
            .register_enhanced_dependencies(plugin_b.metadata().id, vec![dep_b_to_c])
            .expect("should succeed");

        // Resolve dependencies
        let result = resolver.resolve_dependencies().expect("should succeed");

        // Verify order: C should come before B, B before A
        assert_eq!(result.initialization_order.len(), 3);
        assert!(result.unresolved_plugins.is_empty());
        assert!(result.circular_dependencies.is_empty());

        // Find indices safely
        let c_index = result
            .initialization_order
            .iter()
            .position(|&id| id == plugin_c.metadata().id)
            .expect("Plugin C should be in initialization order");
        let b_index = result
            .initialization_order
            .iter()
            .position(|&id| id == plugin_b.metadata().id)
            .expect("Plugin B should be in initialization order");
        let a_index = result
            .initialization_order
            .iter()
            .position(|&id| id == plugin_a.metadata().id)
            .expect("Plugin A should be in initialization order");

        assert!(c_index < b_index);
        assert!(b_index < a_index);
    }

    #[tokio::test]
    async fn test_circular_dependency_detection() {
        let mut resolver = DependencyResolver::new();

        let plugin_a = Arc::new(MockPlugin::new("plugin-a", "1.0.0"));
        let plugin_b = Arc::new(MockPlugin::new("plugin-b", "1.0.0"));

        resolver
            .register_plugin(plugin_a.clone())
            .expect("should succeed");
        resolver
            .register_plugin(plugin_b.clone())
            .expect("should succeed");

        // Create circular dependency: A -> B, B -> A
        let dep_a_to_b = EnhancedPluginDependency {
            id: plugin_b.metadata().id,
            name: "plugin-b".to_string(),
            version_req: "^1.0.0".to_string(),
            optional: false,
            dependency_type: DependencyType::Runtime,
            platform_constraints: vec!["any".to_string()],
        };

        let dep_b_to_a = EnhancedPluginDependency {
            id: plugin_a.metadata().id,
            name: "plugin-a".to_string(),
            version_req: "^1.0.0".to_string(),
            optional: false,
            dependency_type: DependencyType::Runtime,
            platform_constraints: vec!["any".to_string()],
        };

        resolver
            .register_enhanced_dependencies(plugin_a.metadata().id, vec![dep_a_to_b])
            .expect("should succeed");
        resolver
            .register_enhanced_dependencies(plugin_b.metadata().id, vec![dep_b_to_a])
            .expect("should succeed");

        // Resolution should detect the circular dependency
        let result = resolver.resolve_dependencies();
        assert!(
            result.is_err()
                || !result
                    .expect("should succeed")
                    .circular_dependencies
                    .is_empty()
        );
    }

    #[test]
    fn test_version_conflict_detection() {
        let mut resolver = DependencyResolver::new();

        let plugin_a = Arc::new(MockPlugin::new("plugin-a", "1.0.0"));
        let plugin_b = Arc::new(MockPlugin::new("plugin-b", "2.0.0")); // Different version

        resolver
            .register_plugin(plugin_a.clone())
            .expect("should succeed");
        resolver
            .register_plugin(plugin_b.clone())
            .expect("should succeed");

        // A requires B version ^1.0.0, but B is 2.0.0
        let dep_conflicting = EnhancedPluginDependency {
            id: plugin_b.metadata().id,
            name: "plugin-b".to_string(),
            version_req: "^1.0.0".to_string(), // Incompatible with 2.0.0
            optional: false,
            dependency_type: DependencyType::Runtime,
            platform_constraints: vec!["any".to_string()],
        };

        resolver
            .register_enhanced_dependencies(plugin_a.metadata().id, vec![dep_conflicting])
            .expect("should succeed");

        let result = resolver.resolve_dependencies().expect("should succeed");
        assert!(!result.version_conflicts.is_empty());
    }
}
