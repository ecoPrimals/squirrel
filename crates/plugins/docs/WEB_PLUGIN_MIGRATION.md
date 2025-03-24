# Web Plugin Migration Guide

This guide provides instructions for migrating existing web plugins to the new unified plugin system.

## Overview

The Squirrel plugin system is being migrated to a unified architecture. This includes significant changes to the web plugin system. This document will guide you through the process of updating your existing web plugins to be compatible with the new system.

## Migration Options

You have two options for migrating your web plugins:

1. **Use the compatibility adapter**: This is the easiest approach that allows you to keep your existing code while gradually transitioning to the new API.
2. **Complete migration**: Fully update your plugin to use the new API.

## Bidirectional Compatibility

Our plugin system supports **bidirectional compatibility**, meaning:

1. **Legacy plugins can work with the new system**: Using the `LegacyWebPluginAdapter`, existing plugins can be wrapped to work with the new system without modification.
2. **New plugins can work with legacy systems**: Using the `NewWebPluginAdapter`, new plugins built with the latest API can be deployed on systems still using the legacy infrastructure.

This bidirectional approach is especially important for the plugin silo team, as it allows:
- Gradual migration of the plugin ecosystem
- Development of new plugins using the modern API without waiting for all systems to be updated
- Testing new plugins against both new and legacy infrastructures

### Example: Using NewWebPluginAdapter for Legacy Systems

```rust
use std::sync::Arc;
use squirrel_plugins::web::NewWebPluginAdapter;

// Your new plugin implementing the modern API
let modern_plugin = Arc::new(MyModernWebPlugin::new());

// Create an adapter that can be used with legacy systems
let adapted_for_legacy = NewWebPluginAdapter::new(modern_plugin);

// Register with legacy systems
legacy_registry.register_plugin(Arc::new(adapted_for_legacy))?;
```

## Option 1: Using the Compatibility Adapter

If you want to get your plugin working with the new system quickly while minimizing code changes, you can use the `LegacyWebPluginAdapter`.

### Step 1: Update your dependencies

Update your `Cargo.toml` to include the new plugin crate:

```toml
[dependencies]
squirrel-plugins = "0.1.0"
```

### Step 2: Wrap your plugin with the adapter

```rust
use std::sync::Arc;
use squirrel_plugins::web::LegacyWebPluginAdapter;

// Your existing plugin
let my_plugin = Arc::new(MyLegacyWebPlugin::new());

// Create an adapter that can be used with the new system
let adapted_plugin = LegacyWebPluginAdapter::new(my_plugin);

// Register the adapted plugin with the registry
registry.register_plugin(Arc::new(adapted_plugin)).await?;
```

The adapter will handle converting between the old and new interfaces, allowing your plugin to function with the new system.

## Option 2: Complete Migration

For new plugins or if you want to fully migrate to the new API, follow these steps:

### Step 1: Update your dependencies

Update your `Cargo.toml` to include the new plugin crate:

```toml
[dependencies]
squirrel-plugins = "0.1.0"
```

### Step 2: Update your plugin implementation

#### Old API (before):

```rust
use async_trait::async_trait;
use anyhow::Result;
use serde_json::Value;

#[async_trait]
impl WebPlugin for MyPlugin {
    fn get_endpoints(&self) -> Vec<WebEndpoint> {
        vec![
            WebEndpoint {
                path: "/api/my-plugin".to_string(),
                method: "GET".to_string(),
                description: "My plugin endpoint".to_string(),
                permissions: vec!["my-plugin.read".to_string()],
            }
        ]
    }
    
    async fn handle_request(&self, path: &str, method: &str, body: Value) -> Result<Value> {
        // Handle request
        Ok(json!({"result": "success"}))
    }
    
    fn get_components(&self) -> Vec<WebComponent> {
        vec![
            WebComponent {
                id: "my-component-1".to_string(),
                name: "My Component".to_string(),
                description: "My component description".to_string(),
                component_type: "page".to_string(),
                properties: json!({}),
            }
        ]
    }
    
    async fn get_component_markup(&self, component_id: &str, props: Value) -> Result<String> {
        // Generate markup
        Ok("<div>My component</div>".to_string())
    }
}
```

#### New API (after):

```rust
use async_trait::async_trait;
use anyhow::Result;
use serde_json::{Value, json};
use uuid::Uuid;
use squirrel_plugins::web::{
    WebPlugin, WebEndpoint, WebComponent, ComponentType,
    WebRequest, WebResponse, HttpMethod
};

#[async_trait]
impl WebPlugin for MyPlugin {
    fn get_endpoints(&self) -> Vec<WebEndpoint> {
        vec![
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/my-plugin".to_string(),
                HttpMethod::Get,
                "My plugin endpoint".to_string(),
            )
            .with_permission("my-plugin.read")
        ]
    }
    
    async fn handle_request(&self, request: WebRequest) -> Result<WebResponse> {
        // Handle request
        Ok(WebResponse::ok(json!({"result": "success"})))
    }
    
    fn get_components(&self) -> Vec<WebComponent> {
        vec![
            WebComponent::new(
                Uuid::new_v4(),
                "My Component".to_string(),
                "My component description".to_string(),
                ComponentType::Page,
            )
            .with_route("/my-component")
            .with_permission("my-plugin.read")
        ]
    }
    
    async fn get_component_markup(&self, component_id: Uuid, props: Value) -> Result<String> {
        // Generate markup
        Ok("<div>My component</div>".to_string())
    }
}
```

### Key Changes

1. **UUIDs**: The new API uses UUIDs for identifying endpoints and components.
2. **HTTP Methods**: The new API uses an enum for HTTP methods rather than strings.
3. **Builder Pattern**: The new API uses a builder pattern for creating endpoints and components.
4. **Request/Response Objects**: The new API uses structured request and response objects.
5. **Component Types**: The new API uses enums for component types.

## Route Parameter Handling

The new API includes enhanced route parameter handling capabilities, making it easier to work with path parameters in your endpoints.

### Path Parameters in Endpoint Definitions

Define endpoint paths with parameters using curly braces:

```rust
WebEndpoint::new(
    Uuid::new_v4(),
    "/api/users/{userId}/posts/{postId}".to_string(),
    HttpMethod::Get,
    "Get a user's post by ID".to_string(),
)
```

### Automatic Parameter Extraction

The web plugin system will automatically extract path parameters from incoming requests and make them available in the `WebRequest` object:

```rust
async fn handle_request(&self, request: WebRequest) -> Result<WebResponse> {
    // Access path parameters:
    if let Some(user_id) = request.param("userId") {
        if let Some(post_id) = request.param("postId") {
            // Use the extracted parameters
            return self.get_user_post(user_id, post_id).await;
        }
    }
    
    Ok(WebResponse::bad_request("Missing parameters"))
}
```

### Helper Methods

The `WebRequest` struct provides helpful methods for working with parameters:

```rust
// Get a path parameter
let user_id = request.param("userId");

// Get a query parameter
let sort_by = request.query("sortBy");

// Get a parameter from path or query (path takes precedence)
let id = request.param_or_query("id");

// Parse the request body into a specific type
let user_data: UserData = request.parse_body()?;
```

## Testing Your Migration

The plugin crate includes tests and examples to help you verify your migration:

```rust
#[tokio::test]
async fn test_my_migrated_plugin() {
    let plugin = MyPlugin::new();
    
    // Test endpoints
    let endpoints = plugin.get_endpoints();
    assert!(!endpoints.is_empty());
    
    // Test handling a request
    let request = WebRequest::new(
        HttpMethod::Get,
        "/api/my-plugin".to_string(),
        HashMap::new(),
        HashMap::new(),
        None,
        Some("test-user".to_string()),
        vec!["my-plugin.read".to_string()],
    );
    
    let response = plugin.handle_request(request).await.unwrap();
    assert_eq!(response.status, HttpStatus::Ok);
    
    // Test components
    let components = plugin.get_components();
    assert!(!components.is_empty());
}
```

## Examples

For complete examples of plugins using the new API, see:

- `crates/plugins/src/web/example.rs`: Example implementation of a web plugin
- `crates/plugins/examples/`: Additional examples

## Need Help?

If you encounter issues during migration, please:

1. Check the API documentation in the plugin crate
2. Look at the example implementations
3. Contact the DataScienceBioLab team for assistance

## Timeline

The migration timeline is as follows:

1. **Phase 1 (Current)**: Both old and new APIs supported via adapters
2. **Phase 2 (3 months)**: New API recommended for all new plugins
3. **Phase 3 (6 months)**: Old API deprecated
4. **Phase 4 (12 months)**: Old API removed 