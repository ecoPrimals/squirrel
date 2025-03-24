# Squirrel Web Plugin Migration Guide

This guide helps developers migrate legacy plugins to the new Squirrel Web Plugin Architecture.

## Overview

The Squirrel Web application has transitioned from a loosely-typed plugin system to a more robust, strongly-typed architecture. This migration guide helps you understand the changes and transition your existing plugins to the new system.

## Key Differences

| Feature | Legacy System | New System |
|---------|--------------|------------|
| Type Safety | Limited, uses `serde_json::Value` | Strong typing with `WebRequest`/`WebResponse` |
| Route Handling | Manual registration | Automatic based on endpoint definitions |
| Component Support | Basic rendering | Enhanced with properties, permissions, and parent-child relationships |
| Metadata | Basic plugin info | Rich metadata including repository, license, and tags |
| Lifecycle | Limited management | Full lifecycle with enable/disable controls |
| Testing | Manual testing | Integrated testing utilities |

## Migration Options

You have three main options for migrating your plugins:

1. **Use the Legacy Adapter (Easiest)**: Keep your existing plugin implementation and use the `LegacyWebPluginAdapter` to make it work with the new system.
2. **Partial Migration**: Update your plugin to implement the new traits but maintain the same core functionality.
3. **Full Migration**: Rewrite your plugin to take full advantage of the new architecture.

## Using the Legacy Adapter

The easiest way to migrate is to use the `LegacyWebPluginAdapter`:

```rust
use crate::plugins as legacy;
use crate::plugins::adapter::LegacyWebPluginAdapter;
use crate::plugins::WebPluginRegistry;

async fn register_with_adapter(registry: &WebPluginRegistry) -> anyhow::Result<()> {
    // Create or load your legacy plugin
    let legacy_plugin: Box<dyn legacy::WebPlugin> = /* your legacy plugin */;
    
    // Wrap it with the adapter
    let adapter = LegacyWebPluginAdapter::new(legacy_plugin);
    
    // Register the adapter with the registry
    registry.register_plugin(adapter).await?;
    
    Ok(())
}
```

This approach requires minimal changes to your existing code but may not provide access to all the new features.

## Partial Migration

For a partial migration, update your plugin to implement the new traits:

1. Implement the `Plugin` trait:

```rust
use crate::plugins::{Plugin, PluginMetadata, PluginStatus};
use async_trait::async_trait;

#[async_trait]
impl Plugin for MyExistingPlugin {
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

2. Implement the `WebPlugin` trait:

```rust
use crate::plugins::WebPlugin;
use crate::plugins::model::{WebRequest, WebResponse, WebEndpoint, WebComponent};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
impl WebPlugin for MyExistingPlugin {
    fn get_endpoints(&self) -> Vec<WebEndpoint> {
        // Convert your legacy endpoints to the new format
        self.legacy_endpoints.iter()
            .map(|e| WebEndpoint::new(
                e.path.clone(),
                convert_method(e.method),
                format!("Legacy endpoint: {}", e.path),
            ))
            .collect()
    }
    
    fn get_components(&self) -> Vec<WebComponent> {
        // Convert your legacy components to the new format
        self.legacy_components.iter()
            .map(|c| WebComponent::new(
                c.name.clone(),
                convert_component_type(c.component_type.as_str()),
                format!("Legacy component: {}", c.name),
            ))
            .collect()
    }
    
    async fn handle_request(&self, request: WebRequest) -> Result<WebResponse> {
        // Adapt your existing request handling
        let legacy_endpoint = convert_to_legacy_endpoint(&request);
        match self.legacy_handle_endpoint(&legacy_endpoint, request.body).await {
            Ok(value) => Ok(WebResponse::ok().with_body(value)),
            Err(err) => Ok(WebResponse::internal_server_error().with_body(json!({
                "error": err.to_string(),
            }))),
        }
    }
    
    async fn get_component_markup(&self, component_id: uuid::Uuid, props: serde_json::Value) -> Result<String> {
        // Adapt your existing component rendering
        let component = self.find_component_by_id(component_id)?;
        self.legacy_render_component(&component, props).await
    }
}
```

## Full Migration

For a full migration, create a new plugin implementation that takes advantage of all the new features:

1. Define your plugin struct with the necessary fields:

```rust
use crate::plugins::{Plugin, PluginMetadata, PluginStatus, WebPlugin};
use crate::plugins::model::{WebRequest, WebResponse, WebEndpoint, WebComponent, ComponentType, HttpMethod};
use std::collections::HashMap;
use anyhow::Result;

struct MyModernPlugin {
    metadata: PluginMetadata,
    status: PluginStatus,
    // Your plugin-specific state
    data: HashMap<String, String>,
}

impl MyModernPlugin {
    fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "my-plugin".to_string(),
                name: "My Modern Plugin".to_string(),
                version: "2.0.0".to_string(),
                description: "Modern version of my plugin".to_string(),
                author: "Your Name".to_string(),
                repository: Some("https://github.com/yourusername/my-plugin".to_string()),
                license: Some("MIT".to_string()),
                tags: vec!["modern".to_string(), "example".to_string()],
            },
            status: PluginStatus::Active,
            data: HashMap::new(),
        }
    }
}
```

2. Implement the `Plugin` trait:

```rust
use async_trait::async_trait;

#[async_trait]
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

3. Implement the `WebPlugin` trait with full use of new features:

```rust
use serde_json::json;

#[async_trait]
impl WebPlugin for MyModernPlugin {
    fn get_endpoints(&self) -> Vec<WebEndpoint> {
        vec![
            WebEndpoint::new(
                "/api/myplugin/data",
                HttpMethod::Get,
                "Get all data",
            ),
            WebEndpoint::new(
                "/api/myplugin/data",
                HttpMethod::Post,
                "Add new data",
            )
            .with_permission("data.write")
            .with_tag("data-management"),
            WebEndpoint::new(
                "/api/myplugin/data/{key}",
                HttpMethod::Get,
                "Get data by key",
            ),
            WebEndpoint::new(
                "/api/myplugin/data/{key}",
                HttpMethod::Delete,
                "Delete data by key",
            )
            .with_permission("data.delete")
            .with_admin_only(true),
        ]
    }
    
    fn get_components(&self) -> Vec<WebComponent> {
        vec![
            WebComponent::new(
                "Data Manager".to_string(),
                ComponentType::Widget,
                "Widget to manage data",
            )
            .with_route("/dashboard")
            .with_priority(10)
            .with_icon("data-icon"),
            
            WebComponent::new(
                "Data Settings".to_string(),
                ComponentType::Panel,
                "Settings panel for data management",
            )
            .with_route("/settings/data")
            .with_permission("admin")
            .with_parent("settings-panel"),
        ]
    }
    
    async fn handle_request(&self, request: WebRequest) -> Result<WebResponse> {
        match (request.path.as_str(), request.method) {
            ("/api/myplugin/data", HttpMethod::Get) => {
                let data_json = json!(self.data);
                Ok(WebResponse::ok().with_body(data_json))
            },
            ("/api/myplugin/data", HttpMethod::Post) => {
                // Parse request body
                if let Some(body) = request.body.as_object() {
                    let mut mutable_self = unsafe { &mut *(self as *const Self as *mut Self) };
                    
                    for (key, value) in body {
                        if let Some(value_str) = value.as_str() {
                            mutable_self.data.insert(key.clone(), value_str.to_string());
                        }
                    }
                    
                    Ok(WebResponse::created().with_body(json!({
                        "message": "Data added successfully",
                    })))
                } else {
                    Ok(WebResponse::bad_request().with_body(json!({
                        "error": "Invalid request body",
                    })))
                }
            },
            _ if request.path.starts_with("/api/myplugin/data/") => {
                let key = request.path.trim_start_matches("/api/myplugin/data/");
                
                match request.method {
                    HttpMethod::Get => {
                        if let Some(value) = self.data.get(key) {
                            Ok(WebResponse::ok().with_body(json!({
                                "key": key,
                                "value": value,
                            })))
                        } else {
                            Ok(WebResponse::not_found().with_body(json!({
                                "error": format!("Key '{}' not found", key),
                            })))
                        }
                    },
                    HttpMethod::Delete => {
                        let mut mutable_self = unsafe { &mut *(self as *const Self as *mut Self) };
                        if mutable_self.data.remove(key).is_some() {
                            Ok(WebResponse::ok().with_body(json!({
                                "message": format!("Key '{}' deleted", key),
                            })))
                        } else {
                            Ok(WebResponse::not_found().with_body(json!({
                                "error": format!("Key '{}' not found", key),
                            })))
                        }
                    },
                    _ => {
                        Ok(WebResponse::method_not_allowed().with_body(json!({
                            "error": "Method not allowed",
                        })))
                    }
                }
            },
            _ => {
                Ok(WebResponse::not_found().with_body(json!({
                    "error": "Endpoint not found",
                    "path": request.path,
                    "method": format!("{:?}", request.method),
                })))
            }
        }
    }
    
    async fn get_component_markup(&self, component_id: uuid::Uuid, props: serde_json::Value) -> Result<String> {
        // Find the component by ID
        let component_name = self.get_components().iter()
            .find(|c| c.id == component_id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());
        
        match component_name.as_str() {
            "Data Manager" => {
                let data_items = self.data.iter()
                    .map(|(k, v)| format!("<li><strong>{}</strong>: {}</li>", k, v))
                    .collect::<Vec<_>>()
                    .join("\n");
                
                Ok(format!(r#"
                    <div class="data-manager-widget">
                        <h3>Data Manager</h3>
                        <div class="content">
                            <ul class="data-list">
                                {}
                            </ul>
                        </div>
                        <div class="footer">
                            <button class="add-data-btn">Add Data</button>
                        </div>
                    </div>
                "#, if data_items.is_empty() { "<li>No data available</li>" } else { &data_items }))
            },
            "Data Settings" => {
                Ok(format!(r#"
                    <div class="data-settings-panel">
                        <h3>Data Settings</h3>
                        <div class="settings-form">
                            <div class="form-group">
                                <label>Storage Location</label>
                                <select>
                                    <option>In-memory</option>
                                    <option>Database</option>
                                    <option>File</option>
                                </select>
                            </div>
                            <div class="form-group">
                                <label>Auto-backup</label>
                                <input type="checkbox" checked />
                            </div>
                            <button class="save-settings-btn">Save Settings</button>
                        </div>
                    </div>
                "#))
            },
            _ => {
                Ok(format!(r#"<div class="unknown-component">Unknown component: {}</div>"#, component_name))
            }
        }
    }
}
```

## Converting from Legacy Types

To help with migration, here are some utility methods to convert between legacy and modern types:

```rust
use crate::plugins as legacy;
use crate::plugins::model::{HttpMethod, WebEndpoint, WebComponent, ComponentType};

// Convert from legacy HTTP method to modern
fn convert_method(method: legacy::HttpMethod) -> HttpMethod {
    match method {
        legacy::HttpMethod::Get => HttpMethod::Get,
        legacy::HttpMethod::Post => HttpMethod::Post,
        legacy::HttpMethod::Put => HttpMethod::Put,
        legacy::HttpMethod::Delete => HttpMethod::Delete,
        legacy::HttpMethod::Patch => HttpMethod::Patch,
        legacy::HttpMethod::Options => HttpMethod::Options,
        legacy::HttpMethod::Head => HttpMethod::Head,
    }
}

// Convert from legacy component type to modern
fn convert_component_type(legacy_type: &str) -> ComponentType {
    match legacy_type {
        "widget" => ComponentType::Widget,
        "menu" => ComponentType::MenuItem,
        "dashboard" => ComponentType::Dashboard,
        "panel" => ComponentType::Panel,
        "modal" => ComponentType::Modal,
        "form" => ComponentType::Form,
        _ => ComponentType::Custom,
    }
}

// Convert from modern WebRequest to legacy WebEndpoint
fn convert_to_legacy_endpoint(request: &crate::plugins::model::WebRequest) -> legacy::WebEndpoint {
    legacy::WebEndpoint {
        path: request.path.clone(),
        method: match request.method {
            HttpMethod::Get => legacy::HttpMethod::Get,
            HttpMethod::Post => legacy::HttpMethod::Post,
            HttpMethod::Put => legacy::HttpMethod::Put,
            HttpMethod::Delete => legacy::HttpMethod::Delete,
            HttpMethod::Patch => legacy::HttpMethod::Patch,
            HttpMethod::Options => legacy::HttpMethod::Options,
            HttpMethod::Head => legacy::HttpMethod::Head,
        },
        permissions: Vec::new(),
    }
}
```

## Testing Your Migrated Plugin

The new plugin system includes comprehensive testing utilities:

```rust
use crate::plugins::{Plugin, WebPlugin, WebPluginRegistry};
use crate::plugins::model::{WebRequest, HttpMethod};
use anyhow::Result;

#[tokio::test]
async fn test_my_plugin() -> Result<()> {
    // Create a registry and register your plugin
    let registry = WebPluginRegistry::new();
    let my_plugin = MyModernPlugin::new();
    registry.register_plugin(my_plugin).await?;
    
    // Test plugin functionality
    let request = WebRequest::new("/api/myplugin/data".to_string(), HttpMethod::Get);
    let response = registry.handle_request(request).await?;
    
    assert_eq!(response.status as u16, 200);
    assert!(response.body.is_some());
    
    // Test enabling/disabling
    registry.disable_plugin("my-plugin").await?;
    let disabled_plugins = registry.get_disabled_plugins().await;
    assert_eq!(disabled_plugins.len(), 1);
    assert_eq!(disabled_plugins[0].metadata().id, "my-plugin");
    
    Ok(())
}
```

## Best Practices for Modern Plugins

1. **Provide Comprehensive Metadata**: Include all metadata fields like repository URL and license.
2. **Use Descriptive Endpoint Paths**: Follow RESTful naming conventions with clear resource paths.
3. **Document Permissions**: Clearly define and document the permissions required by endpoints.
4. **Handle Errors Consistently**: Use appropriate HTTP status codes and error messages.
5. **Component Design**: Create components that integrate well with the application's UI.
6. **Testing**: Write thorough tests for your plugin functionality.
7. **State Management**: Carefully manage plugin state for thread safety.

## Getting Help

If you encounter issues during migration, please:

1. Check the API documentation in the source code.
2. Look at the `ExamplePlugin` implementation for reference.
3. Run the integration tests to see working examples.
4. Contact the Squirrel Web team for assistance. 