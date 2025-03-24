# Web Module Plugin Migration Plan

## Overview
This document outlines the process for migrating the web plugin system from `crates/web` to the unified `squirrel_plugins` crate. The migration follows the "Direct Conversion Strategy" as outlined in the `specs/plugins/migration-plan.md` document.

## Current State
The web crate currently has a basic plugin system implemented that includes:
- Plugin and WebPlugin traits (in `plugins.rs`)
- PluginManager implementation
- WebEndpoint and WebComponent structures
- Method for initializing the plugin system
- Routes for plugin management

The implementation is currently stub-based with minimal functionality and is not integrated with other crates' plugin systems.

## Migration Steps

### 1. Implement WebPlugin Interface in the Unified Plugin Crate

#### 1.1 Add Web-Specific Interfaces
Update the squirrel_plugins crate to include web-specific plugin interfaces:

```rust
// in squirrel_plugins/src/web/mod.rs
use async_trait::async_trait;
use serde_json::Value;
use crate::core::Plugin;

/// Web plugin trait for integrating with the web interface
#[async_trait]
pub trait WebPlugin: Plugin {
    /// Get the endpoints provided by this plugin
    fn get_endpoints(&self) -> Vec<WebEndpoint>;
    
    /// Handle web endpoint request
    async fn handle_web_endpoint(&self, endpoint: &WebEndpoint, data: Option<Value>) -> Result<Value>;
}

/// Web endpoint definition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WebEndpoint {
    /// Path to the endpoint
    pub path: String,
    
    /// HTTP method
    pub method: HttpMethod,
    
    /// Required permissions
    pub permissions: Vec<String>,
}

/// Web component definition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WebComponent {
    /// Component name
    pub name: String,
    
    /// Component type
    pub component_type: String,
    
    /// Mount point
    pub mount_point: String,
}

/// HTTP methods supported by the plugin system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Options,
    Head,
}
```

### 2. Update Web Crate Dependencies

#### 2.1 Update Cargo.toml
Modify the `crates/web/Cargo.toml` to add dependency on the unified plugin crate:

```toml
[dependencies]
# ... existing dependencies ...
squirrel-plugins = { path = "../plugins", features = ["web"] }
```

#### 2.2 Remove the existing plugin implementation 
The existing plugin system in `plugins.rs` will be gradually migrated to use the new unified plugin system.

### 3. Adapt Web Crate to Use Unified Plugin System

#### 3.1 Create Adapter Module
Create a adapter module to transition from the old to the new plugin system:

```rust
// in crates/web/src/plugin_adapter.rs
use std::sync::Arc;
use anyhow::Result;
use axum::Router;
use squirrel_plugins::web::{WebPlugin, WebEndpoint, HttpMethod};
use squirrel_plugins::registry::PluginRegistry;
use crate::AppState;

pub async fn init_plugin_system(registry: Arc<PluginRegistry>) -> Result<()> {
    // Initialize the plugin system with the unified registry
    Ok(())
}

pub async fn create_plugin_routes<S>(router: Router<S>, state: Arc<AppState>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    // Create routes from the unified plugin registry
    router
}

// Additional adapter functions for plugin handling
```

#### 3.2 Update AppState to Use Unified Plugin Registry
Modify the AppState structure to use the unified plugin registry:

```rust
// in crates/web/src/state.rs
use std::sync::Arc;
use squirrel_plugins::registry::PluginRegistry;

pub struct AppState {
    // Existing fields...
    pub plugin_registry: Arc<PluginRegistry>,
}
```

#### 3.3 Update Web Server Initialization
Modify the web server initialization to use the unified plugin system:

```rust
// in crates/web/src/lib.rs
use squirrel_plugins::registry::PluginRegistry;
use squirrel_plugins::web::registry::WebPluginRegistry;

pub async fn create_app(db: DbPool, config: Config) -> Router {
    // Initialize the unified plugin registry
    let plugin_registry = Arc::new(PluginRegistry::new());
    
    // Load web plugins
    let web_registry = WebPluginRegistry::new(plugin_registry.clone());
    web_registry.load_plugins().await?;
    
    // Create app state with unified registry
    let state = Arc::new(AppState {
        // Existing fields...
        plugin_registry,
    });
    
    // Create router with plugin routes
    // ...
}
```

### 4. Implement Plugin API Endpoints

#### 4.1 Update Plugin API Endpoints
Update the plugin API handlers to use the unified plugin system:

```rust
// in crates/web/src/handlers/plugins.rs
use squirrel_plugins::registry::PluginRegistry;

pub async fn list_plugins(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PluginInfo>>, String> {
    // Use unified plugin registry
    let plugins = state.plugin_registry
        .get_plugins_by_type::<dyn WebPlugin>()
        .await
        .map_err(|e| e.to_string())?;
    
    // Format response...
}

// Additional handler updates...
```

### 5. Testing and Validation

#### 5.1 Create Tests for Plugin Migration
Create tests to validate that the migrated plugin system works correctly:

```rust
// in crates/web/tests/plugin_migration_tests.rs
#[tokio::test]
async fn test_web_plugin_loading() {
    // Initialize plugin registry
    let registry = PluginRegistry::new();
    
    // Load test plugin
    registry.load_plugin::<TestWebPlugin>().await.unwrap();
    
    // Verify plugin is loaded
    assert!(registry.get_plugins::<dyn WebPlugin>().await.unwrap().len() > 0);
}

// Additional tests...
```

### 6. Documentation

#### 6.1 Update Documentation
Update the documentation to reflect the migrated plugin system:

```markdown
# Web Plugin System

The web plugin system is now integrated with the unified Squirrel plugin system. 
Web plugins should implement the `WebPlugin` trait from `squirrel_plugins::web`.

## Creating a Web Plugin

```rust
use squirrel_plugins::core::Plugin;
use squirrel_plugins::web::WebPlugin;

struct MyWebPlugin {
    // Plugin state...
}

#[async_trait]
impl Plugin for MyWebPlugin {
    // Implement Plugin trait...
}

#[async_trait]
impl WebPlugin for MyWebPlugin {
    // Implement WebPlugin trait...
}
```

## Plugin Registration

Plugins are registered with the unified plugin registry:

```rust
let registry = PluginRegistry::new();
registry.register_plugin::<MyWebPlugin>().await.unwrap();
```
```

## Timeline and Milestones

| Milestone | Description | Estimated Time |
|-----------|-------------|----------------|
| Interface Definition | Define web interfaces in squirrel_plugins | 1 day |
| Dependency Updates | Update Cargo.toml and dependencies | 0.5 day |
| Adapter Creation | Create adapter for unified plugin system | 1 day |
| API Updates | Update API endpoints to use unified system | 1 day |
| Testing | Create and run tests for migrated system | 1 day |
| Documentation | Update documentation | 0.5 day |

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking existing functionality | High | Implement and test incrementally with adapters |
| Integration issues with other modules | Medium | Coordinate with other teams through the unified plugin repo |
| Performance impact | Low | Profile before and after to identify issues |

## Conclusion

This migration will align the web module's plugin system with the unified plugin architecture, improving maintainability, security, and interoperability with other system components. 