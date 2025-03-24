# Plugin Migration Guide

This guide provides step-by-step instructions for migrating existing plugins from the legacy system to the new unified plugin architecture.

## Table of Contents

1. [Overview](#overview)
2. [Comparison of Legacy and Modern Systems](#comparison-of-legacy-and-modern-systems)
3. [Migration Options](#migration-options)
4. [Option 1: Using the Legacy Adapter](#option-1-using-the-legacy-adapter)
5. [Option 2: Full Migration](#option-2-full-migration)
6. [Testing Your Migrated Plugin](#testing-your-migrated-plugin)
7. [Best Practices](#best-practices)
8. [Troubleshooting](#troubleshooting)

## Overview

The Squirrel Web application is transitioning from the legacy plugin system to a more robust, type-safe, and feature-rich unified plugin architecture. During this transition period, both systems will coexist to ensure a smooth migration process.

### Key Benefits of the New System

- **Type Safety**: Strongly typed API for plugin interactions
- **Enhanced Lifecycle Management**: Better control over plugin activation and deactivation
- **Rich Metadata**: More comprehensive metadata for plugin discovery
- **Improved Testing Tools**: Better testing utilities for plugin development
- **Enhanced Component Model**: More capabilities for UI components
- **Permissions System**: Fine-grained permission control

## Comparison of Legacy and Modern Systems

| Feature | Legacy System | Modern System |
|---------|--------------|---------------|
| Plugin Definition | `WebPlugin` trait | `Plugin` + `WebPlugin` traits |
| Endpoint Definition | `WebEndpoint` struct | Enhanced `WebEndpoint` with more metadata |
| Component Definition | `WebComponent` struct | Enhanced `WebComponent` with more capabilities |
| Request Handling | Custom handler functions | Unified `WebRequest`/`WebResponse` model |
| Metadata | Basic information | Rich metadata including author, repo, license |
| Lifecycle Management | Limited | Comprehensive (enable, disable, unregister) |
| Testing | Manual testing | Integrated testing utilities |

## Migration Options

You have two main options for migrating your plugins:

1. **Use the Legacy Adapter**: Wrap your existing plugin with minimal changes
2. **Full Migration**: Reimplement your plugin using the new architecture

## Option 1: Using the Legacy Adapter

The simplest approach is to use the `LegacyWebPluginAdapter` to wrap your existing plugin, allowing it to work with the modern registry.

### Step 1: Prepare your legacy plugin

Ensure your legacy plugin correctly implements the legacy `WebPlugin` trait.

```rust
// Legacy plugin implementation
use squirrel_web::plugins as legacy;

struct MyLegacyPlugin {
    // Your plugin fields
}

impl legacy::WebPlugin for MyLegacyPlugin {
    // Your existing implementation
}
```

### Step 2: Register with the modern registry using the adapter

```rust
use squirrel_web::plugin_adapter;
use squirrel_web::plugins::WebPluginRegistry;

async fn register_legacy_plugin() -> anyhow::Result<()> {
    let registry = WebPluginRegistry::new();
    
    let legacy_plugin = Box::new(MyLegacyPlugin::new()) as Box<dyn legacy::WebPlugin>;
    plugin_adapter::register_legacy_plugin(&registry, legacy_plugin).await?;
    
    println!("Legacy plugin registered with modern registry!");
    Ok(())
}
```

### Advantages of Using the Adapter

- Minimal code changes required
- Immediate compatibility with the modern system
- Gradual migration path

### Limitations of Using the Adapter

- Won't benefit from all new features
- May have some performance overhead
- Limited access to new plugin capabilities

## Option 2: Full Migration

For the best experience and to take advantage of all new features, consider fully migrating your plugin to the new architecture.

### Step 1: Create a new plugin structure

```rust
use squirrel_web::plugins::{
    Plugin, PluginMetadata, PluginStatus, WebPlugin,
    model::{WebRequest, WebResponse, WebEndpoint, HttpMethod, ComponentType, WebComponent},
};
use anyhow::Result;
use serde_json::json;

struct MyModernPlugin {
    metadata: PluginMetadata,
    status: PluginStatus,
    // Your plugin fields
}

impl MyModernPlugin {
    fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: uuid::Uuid::new_v4().to_string(),
                name: "My Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "A migrated plugin".to_string(),
                author: "Your Name".to_string(),
                repository: Some("https://github.com/yourusername/my-plugin".to_string()),
                license: Some("MIT".to_string()),
                tags: vec!["migrated".to_string()],
            },
            status: PluginStatus::Active,
            // Initialize your plugin fields
        }
    }
}
```

### Step 2: Implement the base Plugin trait

```rust
impl Plugin for MyModernPlugin {
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
```

### Step 3: Implement the WebPlugin trait

```rust
impl WebPlugin for MyModernPlugin {
    fn get_endpoints(&self) -> Vec<WebEndpoint> {
        // Convert your legacy endpoints to modern ones
        vec![
            WebEndpoint::new(
                "/my-endpoint".to_string(),
                HttpMethod::Get,
                "My endpoint description".to_string(),
            )
            .with_permission("user:read".to_string())
            .with_tag("api".to_string())
            .with_is_public(true),
        ]
    }
    
    fn get_components(&self) -> Vec<WebComponent> {
        // Convert your legacy components to modern ones
        vec![
            WebComponent::new(
                "My Component".to_string(),
                ComponentType::Widget,
                "My component description".to_string(),
            )
            .with_permission("user:read".to_string())
            .with_route("/my-component".to_string())
            .with_priority(10),
        ]
    }
    
    async fn handle_request(&self, request: WebRequest) -> Result<WebResponse> {
        // Convert your legacy request handler to the new format
        match (request.path.as_str(), request.method) {
            ("/my-endpoint", HttpMethod::Get) => {
                // Your implementation for handling this endpoint
                Ok(WebResponse::ok().with_body(json!({
                    "message": "This is my endpoint response",
                })))
            },
            _ => {
                Ok(WebResponse::not_found())
            }
        }
    }
    
    async fn get_component_markup(&self, id: uuid::Uuid, props: serde_json::Value) -> Result<String> {
        // Generate markup for your component
        Ok(format!(
            "<div class=\"my-component\">{}</div>",
            props.get("content").and_then(|c| c.as_str()).unwrap_or("Default Content")
        ))
    }
}
```

### Step 4: Register your modern plugin

```rust
use squirrel_web::plugins::WebPluginRegistry;

async fn register_modern_plugin() -> anyhow::Result<()> {
    let registry = WebPluginRegistry::new();
    
    let my_plugin = MyModernPlugin::new();
    registry.register_plugin(my_plugin).await?;
    
    println!("Modern plugin registered successfully!");
    Ok(())
}
```

### Advantages of Full Migration

- Access to all new features and capabilities
- Improved type safety and error handling
- Better lifecycle management
- Enhanced metadata for discovery

### Conversion Table for Types

| Legacy Type | Modern Type |
|------------|-------------|
| `legacy::WebPlugin` | `plugins::Plugin` + `plugins::WebPlugin` |
| `legacy::WebEndpoint` | `plugins::model::WebEndpoint` |
| `legacy::HttpMethod` | `plugins::model::HttpMethod` |
| `legacy::WebComponent` | `plugins::model::WebComponent` |
| `legacy::PluginInfo` | `plugins::PluginMetadata` |

## Testing Your Migrated Plugin

The new architecture includes testing utilities to verify your plugin works correctly:

```rust
#[tokio::test]
async fn test_my_plugin() -> anyhow::Result<()> {
    // Create a WebPluginRegistry
    let registry = WebPluginRegistry::new();
    
    // Register your plugin
    let my_plugin = MyModernPlugin::new();
    let plugin_id = my_plugin.metadata().id.clone();
    registry.register_plugin(my_plugin).await?;
    
    // Test endpoints directly through the registry
    let request = WebRequest::new("/my-endpoint".to_string(), HttpMethod::Get);
    let response = registry.handle_request(request).await?;
    
    assert_eq!(response.status, 200);
    assert!(response.body.is_some());
    
    let body = response.body.unwrap();
    let message = body.get("message").and_then(|m| m.as_str()).unwrap_or("");
    assert_eq!(message, "This is my endpoint response");
    
    // Test through HTTP routes (requires setting up a test server)
    // See the comprehensive testing documentation for details
    
    Ok(())
}
```

## Best Practices

1. **Start with the Adapter**: Begin with the adapter approach to ensure compatibility, then migrate fully when ready
2. **Incremental Migration**: Migrate one plugin at a time, starting with simpler ones
3. **Comprehensive Testing**: Write tests for your plugin to verify behavior
4. **Rich Metadata**: Include detailed metadata to help with discovery and management
5. **Clear Documentation**: Document your plugin's functionality clearly
6. **Proper Error Handling**: Use the Result type for robust error handling

## Troubleshooting

### Common Issues

#### Plugin Not Found After Registration

- Check that your `metadata().id` is correctly set and consistent
- Verify that the registration is successful (no errors)
- Ensure the registry instance is the same one being used for lookup

#### Endpoints Not Being Called

- Verify that your endpoints are correctly defined in `get_endpoints()`
- Check that the path and method match in both the endpoint definition and handler
- Ensure the plugin is in the `Active` status, not `Disabled`

#### Component Markup Not Rendering

- Check that your components are correctly defined in `get_components()`
- Verify that the component ID is being correctly passed to `get_component_markup()`
- Make sure the returned markup is valid HTML

#### Legacy Adapter Not Working

- Confirm that your legacy plugin correctly implements all required methods
- Check for any missing functionality that might be required by the adapter
- Ensure all dependencies are correctly handled

For more assistance, refer to the comprehensive API documentation or reach out to the development team. 