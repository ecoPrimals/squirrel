# Dependency Resolution Strategy for Squirrel Plugin System

## Current Issues

1. **Circular Dependencies**: Several crates have circular dependencies between them:
   - `squirrel-plugins` depends on `squirrel-mcp` and `squirrel-core`
   - `squirrel-mcp` depends on `squirrel-plugins`
   - `squirrel-core` depends on other crates with transitive dependencies

2. **Workspace Dependency Issues**:
   - Some crates are using workspace dependencies that don't exist in the workspace
   - Lint configuration inheritance is not working properly
   - Inconsistent use of dependency paths and versions

3. **Build System Configuration**:
   - Bench configuration references non-existent files
   - Workspace configuration has multiple issues

## Resolution Approach

### 1. Create Dependency Hierarchy

The first step is to establish a clear hierarchy of dependencies to break the circular references. We'll follow this pattern:

```
interfaces/       (No dependencies on other squirrel crates)
    ↓
core/             (Depends only on interfaces)
    ↓
context/          (Depends on interfaces, core)
    ↓
mcp/              (Depends on interfaces, core, context)
    ↓
plugins/          (Depends on interfaces, core, context, mcp)
    ↓
app/              (Depends on all other crates)
```

### 2. Break Circular Dependencies

For each crate, we need to:

1. Move shared types to the `interfaces` crate
2. Use trait-based design patterns instead of direct dependencies
3. Implement dependency inversion where needed
4. Use feature flags to make circular dependencies optional

Specific changes needed:

#### For squirrel-plugins:

```toml
[dependencies]
squirrel-interfaces = { path = "../interfaces" }  # Required
squirrel-core = { path = "../core" }              # Required
squirrel-mcp = { path = "../mcp", optional = true } # Made optional via feature
```

#### For squirrel-mcp:

```toml
[dependencies]
squirrel-interfaces = { path = "../interfaces" }  # Required
squirrel-core = { path = "../core" }              # Required
squirrel-plugins = { path = "../plugins", optional = true } # Made optional via feature
```

### 3. Fix Workspace Configuration

1. Ensure all required dependencies are properly declared in `[workspace.dependencies]`
2. Standardize on one format for lint configuration
3. Make sure version resolution is consistent

### 4. Implementation Steps

1. **Update interfaces crate**:
   - Add all shared types, traits, and interfaces
   - Keep implementation-free to avoid dependencies

2. **Refactor core crate**:
   - Remove dependencies on higher-level crates
   - Implement core functionality based on interfaces

3. **Update mcp crate**:
   - Remove direct dependency on plugins
   - Use trait-based polymorphism for plugin integration

4. **Refactor plugins crate**:
   - Make mcp dependency optional through features
   - Implement adapter pattern for mcp integration

5. **Fix app crate**:
   - Update to use the new dependency structure
   - Integrate all components at the app level

### 5. Workspace Configuration

Update the main Cargo.toml file:

```toml
[workspace.dependencies]
# All dependencies listed with explicit versions
# ...
```

### 6. Linting Configuration

Standardize on a single approach for lint inheritance:

```toml
[package.lints]
workspace = true
```

### 7. Testing Strategy

1. Create standalone tests for each module
2. Implement integration tests between pairs of crates
3. Add full system tests at the app level

## Implementation Plan

1. Start with the interfaces crate
2. Move up the dependency chain one crate at a time
3. Test each crate individually before proceeding to the next
4. Address workspace-level issues after individual crates are working
5. Implement the full test suite once all crates build successfully

## Next Steps

1. Extract common interfaces to the interfaces crate
2. Fix the plugins crate to work without circular dependencies
3. Test each crate independently 
4. Fix the workspace configuration 