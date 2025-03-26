---
description: Specification for Web Plugin System with bidirectional compatibility support
---

# Web Plugin System Specification

## Overview

The Web Plugin System provides a framework for extending the web application with plugins that deliver endpoints, components, and user interface elements. This specification outlines the design, architecture, and testing considerations for the Web Plugin System.

## Context

- When implementing web-based extensions to the application
- When building UI components for rendering in the browser
- When exposing HTTP endpoints through plugins
- When migrating between legacy and modern plugin interfaces
- When testing plugin functionality

## Architecture

### Core Components

#### 1. WebPluginRegistry

The `WebPluginRegistry` serves as the central manager for all web plugins, providing:

- Plugin discovery and registration
- Endpoint and component management
- Route caching and parameter extraction
- Thread-safe access to plugin resources

```rust
pub struct WebPluginRegistry {
    // Registry reference for plugin lifecycle management
    registry: Arc<PluginRegistry>,
    // Thread-safe storage for endpoints and components
    endpoints: RwLock<HashMap<Uuid, Vec<WebEndpoint>>>,
    components: RwLock<HashMap<Uuid, Vec<WebComponent>>>,
    // Cached routes for fast matching
    routes: RwLock<HashMap<String, Route>>,
}
```

#### 2. WebPlugin Trait

The modern `WebPlugin` trait defines the interface for web plugins:

```rust
#[async_trait]
pub trait WebPlugin: Plugin + Send + Sync {
    fn get_endpoints(&self) -> Vec<WebEndpoint>;
    fn get_components(&self) -> Vec<WebComponent>;
    async fn handle_request(&self, request: WebRequest) -> Result<WebResponse>;
    async fn get_component_markup(&self, component_id: Uuid, props: Value) -> Result<String>;
}
```

#### 3. LegacyWebPluginTrait

The `LegacyWebPluginTrait` defines the interface for legacy web plugins:

```rust
#[async_trait]
pub trait LegacyWebPluginTrait: Plugin + Send + Sync {
    fn get_endpoints(&self) -> Vec<crate::plugin::WebEndpoint>;
    async fn handle_request(&self, path: &str, method: &str, body: Value) -> Result<Value>;
    fn get_components(&self) -> Vec<LegacyWebComponent>;
    async fn get_component_markup(&self, component_id: &str, props: Value) -> Result<String>;
}
```

### Bidirectional Compatibility

The system supports bidirectional compatibility between legacy and modern plugins through adapter classes:

#### 1. LegacyWebPluginAdapter

Adapts legacy plugins to work with the modern plugin system:

```rust
pub struct LegacyWebPluginAdapter<T: Plugin + Send + Sync + ?Sized> {
    plugin: Arc<T>,
    endpoints: Vec<WebEndpoint>,
    components: Vec<WebComponent>,
}
```

#### 2. NewWebPluginAdapter

Adapts modern plugins to work with legacy systems:

```rust
pub struct NewWebPluginAdapter<T: Plugin + Send + Sync + ?Sized> {
    plugin: Arc<T>,
}
```

### Data Models

#### WebEndpoint

```rust
pub struct WebEndpoint {
    id: Uuid,
    path: String,
    method: HttpMethod,
    description: String,
    permissions: Vec<String>,
    is_public: bool,
    is_admin: bool,
    tags: Vec<String>,
}
```

#### WebComponent

```rust
pub struct WebComponent {
    id: Uuid,
    name: String,
    description: String,
    component_type: ComponentType,
    properties: HashMap<String, Value>,
    route: Option<String>,
    priority: i32,
    permissions: Vec<String>,
    parent: Option<Uuid>,
    icon: Option<String>,
}
```

#### WebRequest and WebResponse

```rust
pub struct WebRequest {
    path: String,
    method: HttpMethod,
    headers: HashMap<String, String>,
    query_params: HashMap<String, String>,
    route_params: HashMap<String, String>,
    body: Option<Value>,
    user_id: Option<String>,
    permissions: Vec<String>,
}

pub struct WebResponse {
    status: HttpStatus,
    headers: HashMap<String, String>,
    body: Option<Value>,
}
```

## Testing

### Test Helpers

The system includes specialized test helpers in the `WebPluginRegistry` for effective testing:

```rust
// For testing registration of endpoints
pub(crate) fn register_endpoints(&self, plugin_id: Uuid, endpoints: Vec<WebEndpoint>) {
    // Implementation for test registration of endpoints
}

// For testing registration of components
pub(crate) fn register_components(&self, plugin_id: Uuid, components: Vec<WebComponent>) {
    // Implementation for test registration of components
}

// For testing endpoint discovery
pub(crate) async fn find_endpoint(&self, path: &str, method: HttpMethod) -> Option<WebEndpoint> {
    // Implementation for test endpoint discovery
}

// For testing retrieval of all endpoints
pub(crate) async fn get_all_endpoints(&self) -> Vec<WebEndpoint> {
    // Implementation returning mock endpoints for testing
}

// For testing retrieval of all components
pub(crate) async fn get_all_components(&self) -> Vec<WebComponent> {
    // Implementation returning mock components for testing
}
```

### Test Cases

The system includes comprehensive test cases:

1. **Adapter Tests**
   - `test_legacy_adapter`: Testing adapters for legacy plugins
   - `test_new_adapter`: Testing adapters for modern plugins

2. **General Tests**
   - `test_example_plugin_components`: Testing component registration
   - `test_example_plugin_endpoints`: Testing endpoint registration
   - `test_example_plugin_request_handling`: Testing request handling
   - `test_example_plugin_component_markup`: Testing component markup generation
   - `test_registry`: Testing the WebPluginRegistry functionality

3. **Routing Tests**
   - `test_route_basic_matching`: Testing route matching
   - `test_route_with_single_parameter`: Testing routes with parameters
   - `test_route_with_multiple_parameters`: Testing routes with multiple parameters
   - `test_route_with_special_characters`: Testing routes with special characters
   - `test_web_request_with_route_params`: Testing parameter extraction
   - `test_non_matching_route`: Testing non-matching routes
   - `test_real_world_api_endpoints`: Testing realistic API endpoints

## Implementation Requirements

### WebPluginRegistry

- Must provide thread-safe access to plugin endpoints and components
- Must support route parameter extraction for dynamic paths
- Must support efficient caching of route patterns
- Must handle both legacy and modern plugin types

### WebPlugin Trait

- Must support getting endpoints and components
- Must handle requests asynchronously
- Must generate component markup for rendering in the browser
- Must support proper error handling and response generation

### Bidirectional Compatibility

- Must support legacy plugins working with the modern system
- Must support modern plugins working with legacy systems
- Must provide proper conversion between legacy and modern data structures
- Must maintain consistent behavior across both systems

## Migration Path

As the system transitions from legacy to modern plugin implementations:

1. **Phase 1: Parallel Support**
   - Both legacy and modern plugins work side-by-side
   - Adapters provide compatibility layer

2. **Phase 2: Modern-First Development**
   - New plugins use modern API
   - Legacy plugins continue to function through adapters

3. **Phase 3: Legacy Deprecation**
   - Legacy interface marked deprecated
   - Migration guide provided for plugin developers

4. **Phase 4: Full Modern Implementation**
   - Complete removal of legacy interfaces
   - All plugins use modern API

## Example Implementation

The repository includes the `ExampleWebPlugin` to demonstrate proper implementation:

```rust
pub struct ExampleWebPlugin {
    metadata: PluginMetadata,
    status: RwLock<PluginStatus>,
    data: RwLock<HashMap<String, Value>>,
}

#[async_trait]
impl WebPlugin for ExampleWebPlugin {
    fn get_endpoints(&self) -> Vec<WebEndpoint> {
        // Implementation returning properly structured endpoints
    }

    fn get_components(&self) -> Vec<WebComponent> {
        // Implementation returning properly structured components
    }

    async fn handle_request(&self, request: WebRequest) -> Result<WebResponse> {
        // Implementation handling various request types
    }

    async fn get_component_markup(&self, component_id: Uuid, props: Value) -> Result<String> {
        // Implementation generating markup for components
    }
}
```

## Best Practices

1. **Thread Safety**
   - Use `RwLock` for shared data structures
   - Minimize lock contention by keeping critical sections small
   - Use appropriate synchronization primitives

2. **Error Handling**
   - Properly propagate errors with context
   - Use appropriate status codes in responses
   - Provide clear error messages

3. **Performance**
   - Cache frequently accessed data
   - Minimize allocations in hot paths
   - Use efficient route matching algorithms

4. **Testing**
   - Use test helpers for simulating plugin registry
   - Test both happy and error paths
   - Ensure compatibility between legacy and modern systems

5. **Documentation**
   - Document public interfaces thoroughly
   - Provide clear examples for plugin developers
   - Document migration paths for legacy plugin maintainers

## Technical Metadata
- Category: Web Plugins
- Priority: High
- Dependencies:
  - Plugin registry system
  - Async runtime
  - Web routing system
- Testing Requirements:
  - Unit tests for all components
  - Integration tests for plugin system
  - Compatibility tests for legacy and modern plugins

<version>1.0.0</version> 