---
title: Plugin System Dependency Resolution
version: 1.0.0
date: 2024-05-17
status: active
priority: highest
---

# Plugin System Dependency Resolution

## Overview

This document addresses the dependency management challenges in the plugin system migration, particularly focusing on resolving circular dependencies between crates.

## Current Dependency Issues

During the migration to the unified plugin architecture, we've encountered circular dependencies between the following crates:

```
squirrel-app → squirrel-monitoring → squirrel-plugins → squirrel-app
```

This circular dependency prevents successful compilation and must be resolved to continue the migration process.

## Resolution Strategy

### 1. Feature-Gated Optional Dependencies

The primary resolution strategy is to make dependencies optional and feature-gated:

```toml
# Example in Cargo.toml
[dependencies]
squirrel-app = { path = "../app", optional = true }

[features]
app-integration = ["squirrel-app"]
```

This allows a crate to optionally include another crate based on features, breaking circular dependencies.

### 2. Dependency Inversion

Reorganize dependencies to follow the dependency inversion principle:

1. Create abstraction interfaces in lower-level crates
2. Implement concrete implementations in higher-level crates
3. Use traits to define interactions between components

This allows higher-level crates to depend on abstractions rather than concrete implementations.

### 3. Dependency Injection

Use dependency injection patterns:

1. Accept dependencies as parameters rather than importing them directly
2. Use generics and trait bounds for flexible component implementations
3. Implement factory patterns to create concrete instances

## Implementation Guide

### For the Plugins Crate

1. **Remove direct dependencies on consumer crates**:
   ```toml
   # REMOVE these dependencies
   squirrel-app = { path = "../app", optional = true }
   squirrel-monitoring = { path = "../monitoring", optional = true }
   ```

2. **Define trait interfaces instead**:
   ```rust
   pub trait PluginHost {
       fn register_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<()>;
       // Other methods...
   }
   ```

3. **Use trait objects for callbacks**:
   ```rust
   pub struct PluginManager {
       host: Arc<dyn PluginHost>,
   }
   ```

### For Consumer Crates (Monitoring, App, etc.)

1. **Use optional features for plugins integration**:
   ```toml
   [dependencies]
   squirrel-plugins = { path = "../plugins", default-features = false }
   
   [features]
   plugins-integration = ["squirrel-plugins/core"]
   ```

2. **Implement plugin traits from the plugins crate**:
   ```rust
   impl squirrel_plugins::monitoring::MonitoringPlugin for MyMonitoringPlugin {
       // Implementation...
   }
   ```

## Immediate Actions

The following immediate actions are required to resolve the current build issues:

1. Update `crates/plugins/Cargo.toml` to remove direct dependencies on app, monitoring, etc.
2. Add abstraction interfaces in the plugins crate for all needed functionality
3. Implement these interfaces in the consumer crates
4. Update the monitoring plugins to use the new abstraction pattern
5. Ensure all tests pass with the new dependency structure

## Long-term Recommendations

1. Maintain a clear dependency hierarchy:
   - Core utilities at the bottom
   - Plugin system in the middle
   - Application-specific code at the top

2. Use feature flags consistently to control optional functionality

3. Create a dependency diagram and update it when adding new dependencies

4. Review all new dependencies for potential circular references

## Example: Monitoring Plugin Integration

```rust
// In plugins crate:
pub trait MonitoringPlugin: Plugin {
    async fn collect_metrics(&self) -> Result<Value>;
    fn get_monitoring_targets(&self) -> Vec<String>;
    async fn handle_alert(&self, alert: Value) -> Result<()>;
}

// In monitoring crate:
pub struct SystemMetricsPlugin {
    metadata: PluginMetadata,
    // Implementation details...
}

impl Plugin for SystemMetricsPlugin {
    // Base plugin implementation...
}

impl MonitoringPlugin for SystemMetricsPlugin {
    // Monitoring-specific implementation...
}
```

<version>1.0.0</version> 