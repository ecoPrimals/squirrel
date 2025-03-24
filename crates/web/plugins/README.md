# Web Plugin Architecture

## Overview
This directory contains the implementation of the plugin architecture for the web interface. The plugin system allows for extending the web application with new functionality without modifying the core codebase.

## Directory Structure
```
plugins/
├── mod.rs               # Main module file with exports
├── core/                # Core plugin functionality
│   └── mod.rs           # Plugin trait and related structures
├── model.rs             # Web-specific plugin models
├── registry.rs          # Plugin registry for managing plugins
├── adapter.rs           # Adapters for legacy and modern plugins
├── example.rs           # Example plugin implementation
├── tests.rs             # Plugin system tests
├── PROGRESS_UPDATE.md   # Implementation progress report
└── README.md            # This file
```

## Core Components

### Plugin Trait
The base `Plugin` trait defines the lifecycle methods that all plugins must implement:

```rust
#[async_trait]
pub trait Plugin: Send + Sync + Debug {
    fn metadata(&self) -> &PluginMetadata;
    async fn status(&self) -> PluginStatus;
    async fn initialize(&self) -> Result<()>;
    async fn shutdown(&self) -> Result<()>;
    fn has_feature(&self, feature: &str) -> bool;
}
```

### Web Plugin Trait
The `WebPlugin` trait extends the base `Plugin` trait with web-specific functionality:

```rust
#[async_trait]
pub trait WebPlugin: Plugin + Send + Sync {
    fn get_endpoints(&self) -> Vec<WebEndpoint>;
    fn get_components(&self) -> Vec<WebComponent>;
    async fn handle_request(&self, request: WebRequest) -> Result<WebResponse>;
    async fn get_component_markup(&self, component_id: Uuid, props: Value) -> Result<String>;
}
```

### Plugin Registry
The `WebPluginRegistry` manages the lifecycle of plugins and provides access to their functionality:

```rust
pub struct WebPluginRegistry {
    plugins: RwLock<HashMap<Uuid, PluginRef<dyn WebPlugin>>>,
    endpoints: RwLock<HashMap<Uuid, Vec<WebEndpoint>>>,
    components: RwLock<HashMap<Uuid, Vec<WebComponent>>>,
    routes: RwLock<HashMap<String, Route>>,
}
```

## Usage Examples

### Creating a Plugin
To create a new plugin, implement the `Plugin` and `WebPlugin` traits:

```rust
#[derive(Debug)]
pub struct MyPlugin {
    metadata: PluginMetadata,
    state: PluginState,
}

#[async_trait]
impl Plugin for MyPlugin {
    // Implement Plugin methods...
}

#[async_trait]
impl WebPlugin for MyPlugin {
    // Implement WebPlugin methods...
}
```

### Registering a Plugin
To register a plugin with the registry:

```rust
let registry = WebPluginRegistry::new();
let plugin = MyPlugin::new();
registry.register_plugin(plugin).await?;
```

### Handling Requests
To handle a request using the plugin registry:

```rust
let request = WebRequest::new("/api/my-plugin/endpoint", HttpMethod::Get);
let response = registry.handle_request(request).await?;
```

### Getting Component Markup
To get markup for a component:

```rust
let markup = registry.get_component_markup(
    component_id,
    json!({ "name": "Test User" }),
).await?;
```

## Migration Path
The plugin system includes adapters for bidirectional compatibility between legacy and modern plugins:

1. `LegacyWebPluginAdapter`: Allows legacy plugins to work with the modern system
2. `NewWebPluginAdapter`: Allows modern plugins to work with legacy systems

## Testing
The plugin system includes comprehensive tests in `tests.rs`. To run the tests:

```bash
cargo test -p squirrel-web
```

## Next Steps
1. Integration with the main web server
2. Dynamic plugin loading
3. Comprehensive testing
4. Migration guide for legacy plugins

## Related Documentation
- [specs/plugins/plugin-system.md](../../../specs/plugins/plugin-system.md)
- [specs/plugins/web-plugins.md](../../../specs/plugins/web-plugins.md)
- [PLUGIN_MIGRATION_PLAN.md](../PLUGIN_MIGRATION_PLAN.md)
- [PROGRESS_UPDATE.md](./PROGRESS_UPDATE.md) 