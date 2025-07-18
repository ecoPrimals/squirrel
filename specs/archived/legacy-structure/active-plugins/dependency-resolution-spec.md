---
title: Plugin Dependency Resolution Specification
version: 1.0.0
date: 2024-03-21
status: implemented
owner: Core Team
---

# Plugin Dependency Resolution Specification

## Overview

The Plugin Dependency Resolution System provides comprehensive dependency management for the Squirrel plugin ecosystem. It implements advanced dependency resolution algorithms including topological sorting, circular dependency detection, version conflict resolution, and platform-specific dependency filtering.

## Architecture

### Core Components

1. **DependencyResolver**: Central component that manages plugin dependencies
2. **EnhancedPluginDependency**: Extended dependency structure with semantic versioning
3. **ResolutionResult**: Comprehensive result structure with ordering and conflict information
4. **Integration**: Seamless integration with `DefaultPluginManager`

### Key Features

- **Semantic Versioning**: Full semver support for version constraints
- **Topological Sorting**: Proper initialization order calculation
- **Circular Dependency Detection**: Prevents infinite dependency loops
- **Version Conflict Resolution**: Identifies and reports version mismatches
- **Platform Filtering**: Platform-specific dependency constraints
- **Performance Optimization**: Intelligent caching and batch processing
- **Comprehensive Reporting**: Detailed resolution results and statistics

## Implementation Details

### Enhanced Plugin Dependencies

```rust
pub struct EnhancedPluginDependency {
    pub id: Uuid,
    pub name: String,
    pub version_req: String,        // Semantic version requirement
    pub optional: bool,
    pub dependency_type: DependencyType,
    pub platform_constraints: Vec<String>,
}

pub enum DependencyType {
    Critical,      // Must be loaded first
    Runtime,       // Standard runtime dependency
    Development,   // Optional in production
    Extension,     // Plugin extension
}
```

### Resolution Algorithm

The dependency resolver implements a multi-step resolution process:

1. **Validation**: Verify all dependencies exist and versions are compatible
2. **Cycle Detection**: Use DFS to detect circular dependencies
3. **Version Resolution**: Identify and report version conflicts
4. **Platform Filtering**: Filter dependencies based on current platform
5. **Topological Sort**: Calculate proper initialization order using Kahn's algorithm
6. **Optimization**: Optimize order based on dependency types

### Integration with Plugin Manager

The `DefaultPluginManager` now includes integrated dependency resolution:

```rust
pub struct DefaultPluginManager {
    // ... existing fields ...
    dependency_resolver: RwLock<DependencyResolver>,
}
```

Key integration points:

- **Registration**: Plugins are automatically registered with the dependency resolver
- **Initialization**: `initialize_all_plugins()` uses dependency-ordered initialization
- **Management**: Helper methods for dependency management and statistics

## API Reference

### Core Methods

#### DependencyResolver

```rust
impl DependencyResolver {
    pub fn new() -> Self;
    pub fn register_plugin(&mut self, plugin: Arc<dyn Plugin>) -> Result<()>;
    pub fn register_enhanced_dependencies(&mut self, plugin_id: Uuid, dependencies: Vec<EnhancedPluginDependency>) -> Result<()>;
    pub fn resolve_dependencies(&mut self) -> Result<ResolutionResult>;
    pub fn get_dependents(&self, plugin_id: Uuid) -> Vec<Uuid>;
    pub fn clear_cache(&mut self);
    pub fn get_statistics(&self) -> ResolutionStatistics;
}
```

#### DefaultPluginManager Integration

```rust
impl DefaultPluginManager {
    pub async fn register_enhanced_dependencies(&self, plugin_id: Uuid, dependencies: Vec<EnhancedPluginDependency>) -> Result<()>;
    pub async fn get_dependency_statistics(&self) -> ResolutionStatistics;
    pub async fn clear_dependency_cache(&self);
    pub async fn get_plugin_dependents(&self, plugin_id: Uuid) -> Vec<Uuid>;
    pub async fn resolve_dependencies_dry_run(&self) -> Result<ResolutionResult>;
}
```

### Resolution Results

```rust
pub struct ResolutionResult {
    pub initialization_order: Vec<Uuid>,
    pub unresolved_plugins: Vec<(Uuid, Vec<String>)>,
    pub version_conflicts: Vec<VersionConflict>,
    pub circular_dependencies: Vec<Vec<Uuid>>,
    pub warnings: Vec<String>,
}

pub struct VersionConflict {
    pub plugin_id: Uuid,
    pub dependency_name: String,
    pub required_version: String,
    pub available_version: String,
    pub dependent_plugins: Vec<Uuid>,
}
```

## Usage Examples

### Basic Dependency Registration

```rust
// Create enhanced dependency
let dependency = EnhancedPluginDependency {
    id: dependency_plugin_id,
    name: "core-services".to_string(),
    version_req: "^1.2.0".to_string(),
    optional: false,
    dependency_type: DependencyType::Runtime,
    platform_constraints: vec!["any".to_string()],
};

// Register with plugin manager
manager.register_enhanced_dependencies(plugin_id, vec![dependency]).await?;
```

### Platform-Specific Dependencies

```rust
let platform_dep = EnhancedPluginDependency {
    id: platform_plugin_id,
    name: "windows-integration".to_string(),
    version_req: ">=2.0.0".to_string(),
    optional: false,
    dependency_type: DependencyType::Runtime,
    platform_constraints: vec!["windows".to_string()],
};
```

### Critical Dependencies

```rust
let critical_dep = EnhancedPluginDependency {
    id: security_plugin_id,
    name: "security-core".to_string(),
    version_req: "~1.5.0".to_string(),
    optional: false,
    dependency_type: DependencyType::Critical,
    platform_constraints: vec!["any".to_string()],
};
```

### Dependency Resolution

```rust
// Perform dry-run resolution
let result = manager.resolve_dependencies_dry_run().await?;

// Check for issues
if !result.circular_dependencies.is_empty() {
    println!("Circular dependencies detected!");
}

if !result.version_conflicts.is_empty() {
    println!("Version conflicts found!");
}

// Initialize in proper order
manager.initialize_all_plugins().await?;
```

## Performance Characteristics

### Time Complexity

- **Registration**: O(1) per plugin
- **Resolution**: O(V + E) where V = plugins, E = dependencies
- **Cycle Detection**: O(V + E) using DFS
- **Topological Sort**: O(V + E) using Kahn's algorithm

### Space Complexity

- **Memory Usage**: O(V + E) for plugin and dependency storage
- **Cache Storage**: O(R) where R = number of cached resolutions

### Optimization Features

- **Resolution Caching**: Avoids re-computation for unchanged plugin sets
- **Incremental Updates**: Efficient handling of plugin additions/removals
- **Lazy Evaluation**: Dependencies resolved only when needed

## Error Handling

The system provides comprehensive error handling for all failure scenarios:

### Error Types

```rust
pub enum PluginError {
    InvalidVersion(String),
    CircularDependency(String),
    VersionConflict(String),
    PlatformIncompatible(String),
    ResolutionFailed(String),
    DependencyNotFound(String),
    // ... other error types
}
```

### Error Recovery

- **Graceful Degradation**: Continue initialization for unaffected plugins
- **Detailed Reporting**: Comprehensive error messages with context
- **Recovery Suggestions**: Actionable recommendations for fixing issues

## Testing

### Test Coverage

The dependency resolution system includes comprehensive test coverage:

- **Unit Tests**: Core algorithm testing
- **Integration Tests**: Plugin manager integration
- **Edge Case Tests**: Circular dependencies, version conflicts
- **Performance Tests**: Large-scale dependency resolution
- **Platform Tests**: Cross-platform compatibility

### Test Examples

```rust
#[tokio::test]
async fn test_dependency_resolution() {
    let mut resolver = DependencyResolver::new();
    
    // Register plugins with dependencies
    resolver.register_plugin(plugin_a).unwrap();
    resolver.register_plugin(plugin_b).unwrap();
    
    // Set up dependency chain: A -> B
    let dep = EnhancedPluginDependency {
        id: plugin_b.metadata().id,
        name: "plugin-b".to_string(),
        version_req: "^1.0.0".to_string(),
        optional: false,
        dependency_type: DependencyType::Runtime,
        platform_constraints: vec!["any".to_string()],
    };
    
    resolver.register_enhanced_dependencies(plugin_a.metadata().id, vec![dep]).unwrap();
    
    // Resolve and verify order
    let result = resolver.resolve_dependencies().unwrap();
    assert!(result.initialization_order[0] == plugin_b.metadata().id);
    assert!(result.initialization_order[1] == plugin_a.metadata().id);
}
```

## Migration Guide

### From Legacy Dependencies

Existing plugins using the legacy `Vec<Uuid>` dependency format are automatically converted:

```rust
// Legacy format (automatic conversion)
pub dependencies: Vec<Uuid>

// Enhanced format (recommended)
EnhancedPluginDependency {
    id: dependency_id,
    name: "dependency-name".to_string(),
    version_req: "*".to_string(),  // Any version
    optional: false,
    dependency_type: DependencyType::Runtime,
    platform_constraints: vec!["any".to_string()],
}
```

### Upgrading Plugin Manager

No breaking changes to existing `DefaultPluginManager` usage. New functionality is additive:

```rust
// Existing code continues to work
let manager = DefaultPluginManager::new(state_manager, None);
manager.initialize_all_plugins().await?;

// New enhanced functionality available
let stats = manager.get_dependency_statistics().await;
let dependents = manager.get_plugin_dependents(plugin_id).await;
```

## Configuration

### Dependency Resolution Settings

```rust
pub struct DependencyResolverConfig {
    pub enable_caching: bool,
    pub max_cache_entries: usize,
    pub strict_version_checking: bool,
    pub allow_dev_dependencies: bool,
    pub platform_override: Option<String>,
}
```

### Platform Detection

The resolver automatically detects the current platform:

- **Windows**: `"windows"`
- **Linux**: `"linux"`
- **macOS**: `"macos"`
- **Unknown**: `"unknown"`

## Monitoring and Observability

### Statistics and Metrics

```rust
pub struct ResolutionStatistics {
    pub total_plugins: usize,
    pub total_dependencies: usize,
    pub optional_dependencies: usize,
    pub cache_entries: usize,
}
```

### Logging Integration

The system provides comprehensive logging:

- **Debug**: Detailed resolution steps
- **Info**: Resolution progress and results
- **Warn**: Version conflicts and platform issues
- **Error**: Critical failures and circular dependencies

## Future Enhancements

### Planned Features

1. **Dynamic Dependency Updates**: Runtime dependency modification
2. **Dependency Suggestions**: Automatic dependency discovery
3. **Version Upgrade Automation**: Automatic version conflict resolution
4. **Parallel Initialization**: Concurrent plugin initialization where safe
5. **Dependency Profiles**: Environment-specific dependency sets

### Extension Points

The system is designed for extensibility:

- **Custom Resolution Strategies**: Pluggable resolution algorithms
- **External Registry Integration**: Support for remote dependency registries
- **Advanced Caching**: Persistent dependency resolution caching
- **Metrics Integration**: Integration with monitoring systems

## Implementation Status

✅ **Core Algorithm**: Topological sorting with cycle detection
✅ **Version Management**: Semantic versioning support
✅ **Platform Support**: Platform-specific dependency filtering
✅ **Integration**: Seamless plugin manager integration
✅ **Error Handling**: Comprehensive error reporting
✅ **Testing**: Full test coverage with edge cases
✅ **Documentation**: Complete API and usage documentation
✅ **Performance**: Optimized algorithms with caching

## Related Specifications

- [Plugin System Architecture](plugin-architecture-spec.md)
- [Plugin Security Model](plugin-security-spec.md)
- [Plugin Discovery System](plugin-discovery-spec.md)
- [Plugin Lifecycle Management](plugin-lifecycle-spec.md)

---

*This specification represents the completed implementation of the Plugin Dependency Resolution System, bringing the plugin system from 95% to 100% completion for Sprint 1.* 