# Web Interface

The Web Interface provides HTTP and WebSocket APIs for the Squirrel system.

## Current Status

The Web Interface is currently operational with the following features:
- HTTP API for command execution
- WebSocket API for real-time events
- Authentication with JWT tokens
- Job management for long-running operations
- Health checking endpoints
- Plugin system (completed and ready for use)

## Plugin System

The Web Interface includes a comprehensive plugin system that allows extending the application with custom functionality. The plugin system has the following features:

- Modern, type-safe plugin architecture
- Support for HTTP endpoints and UI components
- Plugin lifecycle management (enable, disable, unregister)
- Dynamic loading of plugins from directory
- Support for native (dynamic library) plugins and script-based plugins

### Using Plugins

To use the plugin system, you can:

1. **Create a custom plugin** by implementing the `Plugin` and `WebPlugin` traits
2. **Register the plugin programmatically** using `WebPluginRegistry::register_plugin`
3. **Load plugins dynamically** from a directory using `WebPluginRegistry::load_plugins_from_directory`
4. **Create a plugin dynamic library** that can be loaded at runtime

### Creating a Custom Plugin

```rust
use squirrel_web::plugins::{Plugin, WebPlugin, PluginMetadata, PluginStatus, CloneablePlugin};

// Define your plugin struct
struct MyPlugin {
    metadata: PluginMetadata,
    status: PluginStatus,
}

// Implement the Plugin trait
#[async_trait]
impl Plugin for MyPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    fn status(&self) -> PluginStatus {
        self.status
    }
    
    fn set_status(&mut self, status: PluginStatus) {
        self.status = status;
    }
}

// Implement the WebPlugin trait
#[async_trait]
impl WebPlugin for MyPlugin {
    // ... implementation details ...
}

// Implement CloneablePlugin for proper cloning
impl CloneablePlugin for MyPlugin {
    fn clone_plugin(&self) -> Box<dyn WebPlugin> {
        Box::new(self.clone())
    }
}
```

### Dynamic Library Plugins

To create a plugin that can be loaded as a dynamic library, you need to:

1. Implement the CloneablePlugin trait
2. Export a `create_plugin` function

```rust
#[no_mangle]
pub extern "C" fn create_plugin() -> *mut dyn WebPlugin {
    let plugin = MyPlugin::new();
    let boxed = Box::new(plugin);
    Box::into_raw(boxed)
}
```

Build your plugin as a dynamic library:

```bash
cargo build --release --crate-type=cdylib
```

Then place the resulting .dll or .so file in the plugins directory.

### Script-Based Plugins

Support for JavaScript and Python plugins is planned for future releases.

### Feature Flags

The plugin system includes several feature flags:

- `dynamic-plugins`: Enable loading plugins from dynamic libraries (.dll/.so)
- `script-plugins`: Enable loading plugins from scripts (JavaScript/Python)

## Running the Web Interface

### Prerequisites

- Rust 1.67 or higher
- SQLite (if using DB mode)

### Installation

```bash
git clone <repository>
cd squirrel/crates/web
```

### Build and Run

With mock database (no actual DB needed):

```bash
cargo build --features mock-db
cargo run --features mock-db
```

With real database:

```bash
# Create the database
./setup-db.ps1    # On Windows
./setup-db.sh     # On Unix/Linux/Mac

# Run with database
cargo build --features db
cargo run --features db
```

### Configuration

Configuration is loaded from environment variables:

- `SQUIRREL_WEB_HOST`: Hostname to bind to (default: 127.0.0.1)
- `SQUIRREL_WEB_PORT`: Port to listen on (default: 8080)
- `SQUIRREL_DB_URL`: Database URL (default: sqlite:test.db)
- `SQUIRREL_JWT_SECRET`: JWT secret key (default: generated randomly)
- `SQUIRREL_PLUGIN_DIR`: Directory to load plugins from (default: plugins)

## API Documentation

API documentation is available at:

- `/api/docs` - OpenAPI documentation (coming soon)
- `/api/health` - Health check endpoint
- `/api/commands` - Command execution endpoints
- `/api/jobs` - Job management endpoints
- `/api/plugins` - Plugin management endpoints
- `/ws` - WebSocket endpoint

## Development Notes

### Feature Flags

- `mock-db` - Use in-memory database (for development)
- `db` - Use real database
- `server` - Build the HTTP server (always enabled by default)
- `dynamic-plugins` - Enable loading plugins from dynamic libraries
- `script-plugins` - Enable loading plugins from scripts

### Testing

```bash
cargo test
```

### Adding a New Plugin

Currently, the Web Interface is migrating its plugin system. During this transition, you can still create plugins using the legacy system, which will be gradually migrated to the unified system.

Legacy plugin example:

```rust
use squirrel_web::plugins::{Plugin, WebPlugin, PluginMetadata, WebEndpoint, HttpMethod};
use async_trait::async_trait;
use serde_json::Value;
use anyhow::Result;
use uuid::Uuid;

#[derive(Debug)]
pub struct MyPlugin {
    metadata: PluginMetadata,
}

impl MyPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: Uuid::new_v4(),
                name: "My Plugin".to_string(),
                version: "0.1.0".to_string(),
                description: "My awesome plugin".to_string(),
                author: "Your Name".to_string(),
                capabilities: vec!["awesome".to_string()],
                dependencies: vec![],
            },
        }
    }
}

#[async_trait]
impl Plugin for MyPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn initialize(&self) -> Result<()> {
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl WebPlugin for MyPlugin {
    fn get_endpoints(&self) -> Vec<WebEndpoint> {
        vec![
            WebEndpoint {
                path: "/my-endpoint".to_string(),
                method: HttpMethod::Get,
                permissions: vec![],
            },
        ]
    }
    
    async fn handle_web_endpoint(&self, _endpoint: &WebEndpoint, _data: Option<Value>) -> Result<Value> {
        Ok(serde_json::json!({ "status": "ok" }))
    }
}
```

After the migration is complete, plugins will be created using the unified plugin system:

```rust
// Future API (coming soon)
use squirrel_plugins::core::Plugin;
use squirrel_plugins::web::WebPlugin;
// ... implementation ...
```

Stay tuned for updates on the plugin system migration!

# Squirrel Web Plugin Architecture

This document provides an overview of the Squirrel Web Plugin Architecture. The plugin system allows for extending the Squirrel Web application with custom functionality while maintaining strong typing and a consistent interface.

## Core Components

### Plugin Traits

The plugin system is built around two core traits:

1. **Plugin**: The base trait that all plugins must implement. It defines the basic metadata and lifecycle methods.
2. **WebPlugin**: Extension of the `Plugin` trait specifically for web-based plugins, adding HTTP endpoint and component functionality.

### Plugin Registry

The `WebPluginRegistry` acts as the central coordinator for all plugins, providing:

- Plugin registration and discovery
- Lifecycle management (enable/disable plugins)
- Request routing to appropriate plugin handlers
- Component retrieval and rendering

### Data Models

The system includes several key data structures:

- **WebRequest**: Represents an HTTP request with path, method, headers, query parameters, and body.
- **WebResponse**: Represents an HTTP response with status, headers, and body.
- **WebEndpoint**: Defines an API endpoint with path, method, description, and permissions.
- **WebComponent**: Defines a UI component with name, type, description, and rendering properties.

## Plugin Lifecycle

Plugins go through the following lifecycle phases:

1. **Registration**: Plugins are registered with the registry either programmatically or dynamically loaded.
2. **Activation**: When enabled, plugins register their endpoints and components with the system.
3. **Request Handling**: Incoming HTTP requests are routed to the appropriate plugin endpoint handlers.
4. **Component Rendering**: UI components are rendered when requested by the application.
5. **Deactivation**: Plugins can be temporarily disabled without unregistering.
6. **Unregistration**: Plugins can be completely removed from the system.

## Legacy Compatibility

The system includes an adapter layer for compatibility with the legacy plugin system:

- `LegacyWebPluginAdapter`: Wraps legacy plugins to make them compatible with the new system.
- Route mapping from legacy to modern endpoint format.
- Data conversion between legacy and modern data structures.

## Creating Plugins

### Basic Plugin Structure

```rust
use async_trait::async_trait;
use anyhow::Result;
use serde_json::json;

use crate::plugins::{Plugin, PluginMetadata, PluginStatus, WebPlugin};
use crate::plugins::model::{WebRequest, WebResponse, WebEndpoint, WebComponent, ComponentType, HttpMethod};

// Define your plugin struct
struct MyPlugin {
    metadata: PluginMetadata,
    status: PluginStatus,
}

// Implement the Plugin trait
#[async_trait]
impl Plugin for MyPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    fn status(&self) -> PluginStatus {
        self.status
    }
    
    fn set_status(&mut self, status: PluginStatus) {
        self.status = status;
    }
}

// Implement the WebPlugin trait
#[async_trait]
impl WebPlugin for MyPlugin {
    fn get_endpoints(&self) -> Vec<WebEndpoint> {
        // Define your API endpoints
        vec![
            WebEndpoint::new(
                "/api/myplugin/hello", 
                HttpMethod::Get, 
                "Returns a hello message"
            ),
        ]
    }
    
    fn get_components(&self) -> Vec<WebComponent> {
        // Define your UI components
        vec![
            WebComponent::new(
                "My Widget",
                ComponentType::Widget,
                "A custom widget"
            )
            .with_route("/dashboard"),
        ]
    }
    
    async fn handle_request(&self, request: WebRequest) -> Result<WebResponse> {
        // Handle API requests
        match (request.path.as_str(), request.method) {
            ("/api/myplugin/hello", HttpMethod::Get) => {
                Ok(WebResponse::ok().with_body(json!({
                    "message": "Hello from my plugin!"
                })))
            },
            _ => {
                Ok(WebResponse::not_found().with_body(json!({
                    "error": "Endpoint not found"
                })))
            }
        }
    }
    
    async fn get_component_markup(&self, component_id: uuid::Uuid, props: serde_json::Value) -> Result<String> {
        // Generate UI markup for your components
        Ok(format!(r#"
            <div class="my-plugin-component">
                <h3>My Plugin</h3>
                <div class="content">
                    <p>This is a component from my plugin.</p>
                </div>
            </div>
        "#))
    }
}
```

### Registering a Plugin

```rust
// In application startup code
let registry = WebPluginRegistry::new();
let my_plugin = MyPlugin::new();
registry.register_plugin(my_plugin).await?;
```

## Integration with Main Application

The plugin system is integrated with the main Squirrel Web application through:

1. The `AppState` struct, which includes the `WebPluginRegistry`.
2. Dynamic route generation in `create_plugin_routes`.
3. Plugin discovery and loading during application startup.

## Migration Guide

When migrating from the legacy plugin system to the modern architecture:

1. Update plugin implementations to use the new traits.
2. Replace direct interactions with plugin manager with registry calls.
3. Use the `LegacyWebPluginAdapter` for plugins that cannot be immediately migrated.

## Example Plugin

See the `ExamplePlugin` implementation in `crates/web/src/plugins/example.rs` for a complete working example of a plugin.

## Testing

The plugin system includes comprehensive integration tests in `crates/web/tests/plugin_integration_test.rs` that demonstrate proper usage and expected behavior. 