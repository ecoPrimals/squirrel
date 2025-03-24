# Plugin Silo Team Specification

## Bidirectional Compatibility for Web Plugins

This document provides technical specifications for the plugin silo team to implement and test bidirectional compatibility for web plugins during the migration phase.

## Architecture Overview

The web plugin system supports bidirectional compatibility through adapter patterns:

```
                    ┌───────────────────┐
                    │                   │
                    │  Plugin Registry  │
                    │                   │
                    └───────────────────┘
                             ▲
                             │
                             │
              ┌──────────────┴───────────────┐
              │                              │
┌─────────────┴─────────────┐   ┌────────────┴────────────┐
│                           │   │                          │
│ Legacy Plugin + Adapter   │   │ Modern Plugin (direct)   │
│ (LegacyWebPluginAdapter)  │   │                          │
│                           │   │                          │
└───────────────────────────┘   └──────────────────────────┘


                    ┌───────────────────┐
                    │                   │
                    │  Legacy Registry  │
                    │                   │
                    └───────────────────┘
                             ▲
                             │
                             │
              ┌──────────────┴───────────────┐
              │                              │
┌─────────────┴─────────────┐   ┌────────────┴────────────┐
│                           │   │                          │
│ Legacy Plugin (direct)    │   │ Modern Plugin + Adapter  │
│                           │   │ (NewWebPluginAdapter)    │
│                           │   │                          │
└───────────────────────────┘   └──────────────────────────┘
```

## Adapter Implementation Details

### 1. LegacyWebPluginAdapter

Allows legacy plugins to work with the new system:

- **Input**: A legacy plugin implementing the old `WebPlugin` trait
- **Output**: A plugin that implements the new `WebPlugin` trait
- **Key Transformations**:
  - Converts string-based HTTP methods to `HttpMethod` enum
  - Generates UUIDs for endpoints and components
  - Transforms legacy request/response format to structured objects
  - Maps component types from strings to enum values

### 2. NewWebPluginAdapter

Allows modern plugins to work with legacy systems:

- **Input**: A modern plugin implementing the new `WebPlugin` trait
- **Output**: A plugin that implements the legacy `WebPlugin` trait
- **Key Transformations**:
  - Converts `HttpMethod` enum to string-based methods
  - Serializes UUIDs to string identifiers
  - Transforms structured request/response objects to legacy format
  - Maps component type enums to string values

## Implementation Guidelines

### For Legacy Plugins Working with New Systems

```rust
// 1. Import the adapter
use squirrel_plugins::web::LegacyWebPluginAdapter;

// 2. Initialize your legacy plugin
let legacy_plugin = Arc::new(MyLegacyPlugin::new());

// 3. Create the adapter
let adapted_plugin = LegacyWebPluginAdapter::new(legacy_plugin);

// 4. Register with the new system
modern_registry.register_plugin(Arc::new(adapted_plugin)).await?;
```

### For Modern Plugins Working with Legacy Systems

```rust
// 1. Import the adapter
use squirrel_plugins::web::NewWebPluginAdapter;

// 2. Initialize your modern plugin
let modern_plugin = Arc::new(MyModernPlugin::new());

// 3. Create the adapter
let adapted_plugin = NewWebPluginAdapter::new(modern_plugin);

// 4. Register with the legacy system
legacy_registry.register_plugin(Arc::new(adapted_plugin))?;
```

## Testing Procedures

### 1. Legacy Plugin Compatibility Testing

Test that legacy plugins work correctly with the new system:

```rust
#[tokio::test]
async fn test_legacy_plugin_compatibility() {
    // 1. Create a legacy plugin
    let legacy_plugin = Arc::new(TestLegacyPlugin::new());
    
    // 2. Create the adapter
    let adapted_plugin = Arc::new(LegacyWebPluginAdapter::new(legacy_plugin));
    
    // 3. Verify endpoints are correctly transformed
    let endpoints = adapted_plugin.get_endpoints();
    assert_eq!(endpoints[0].method, HttpMethod::Get);
    
    // 4. Test request handling
    let request = WebRequest::new(
        HttpMethod::Get,
        "/api/test".to_string(),
        HashMap::new(),
        HashMap::new(),
        Some(json!({"test": true})),
        None,
        vec![],
    );
    
    let response = adapted_plugin.handle_request(request).await.unwrap();
    assert_eq!(response.status, HttpStatus::Ok);
    
    // 5. Verify components are correctly transformed
    let components = adapted_plugin.get_components();
    assert_eq!(components[0].component_type, ComponentType::Page);
}
```

### 2. Modern Plugin Compatibility Testing

Test that modern plugins work correctly with legacy systems:

```rust
#[tokio::test]
async fn test_modern_plugin_compatibility() {
    // 1. Create a modern plugin
    let modern_plugin = Arc::new(TestModernPlugin::new());
    
    // 2. Create the adapter
    let adapted_plugin = Arc::new(NewWebPluginAdapter::new(modern_plugin));
    
    // 3. Verify endpoints are correctly transformed
    let endpoints = adapted_plugin.get_endpoints();
    assert_eq!(endpoints[0].method, "GET");
    
    // 4. Test request handling
    let response = adapted_plugin.handle_request(
        "/api/test", 
        "GET", 
        json!({"test": true})
    ).await.unwrap();
    
    assert!(response.is_object());
    
    // 5. Verify components are correctly transformed
    let components = adapted_plugin.get_components();
    assert_eq!(components[0].component_type, "page");
}
```

### 3. Integration Testing with Mixed Plugin Types

```rust
#[tokio::test]
async fn test_mixed_plugin_environment() {
    // 1. Create a registry
    let registry = PluginRegistry::new();
    
    // 2. Register both types of plugins
    let legacy_plugin = Arc::new(LegacyWebPluginAdapter::new(
        Arc::new(TestLegacyPlugin::new())
    ));
    registry.register_plugin(legacy_plugin.clone()).await.unwrap();
    
    let modern_plugin = Arc::new(TestModernPlugin::new());
    registry.register_plugin(modern_plugin.clone()).await.unwrap();
    
    // 3. Verify both plugins are accessible
    let plugins = registry.list_plugins().await;
    assert_eq!(plugins.len(), 2);
    
    // 4. Test endpoint routing for both plugin types
    let web_registry = WebPluginRegistry::new(Arc::new(registry));
    web_registry.load_plugins().await.unwrap();
    
    // 5. Test requests to both plugins
    let legacy_response = web_registry.handle_request(
        WebRequest::new(
            HttpMethod::Get,
            "/api/legacy".to_string(),
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            vec![],
        )
    ).await.unwrap();
    
    let modern_response = web_registry.handle_request(
        WebRequest::new(
            HttpMethod::Get,
            "/api/modern".to_string(),
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            vec![],
        )
    ).await.unwrap();
    
    assert!(legacy_response.body.is_some());
    assert!(modern_response.body.is_some());
}
```

## Performance Considerations

The adapters introduce minimal overhead, primarily in:

1. **UUID Generation/Parsing**: When converting between string IDs and UUIDs
2. **JSON Transformation**: When converting between different request/response formats
3. **Method String/Enum Conversion**: When mapping HTTP methods

Most of this overhead is at plugin initialization or endpoint registration time, not during request handling.

## For Push Operations

When pushing from the plugin silo team to the main repository:

1. Include unit tests specifically for adapter compatibility
2. Document any changes to the adapter patterns or API
3. Ensure backward compatibility is maintained
4. Include integration tests with mixed plugin environments

## Reference Implementation

A complete reference implementation is available at:
- `crates/plugins/examples/bidirectional_compatibility.rs`

## Contact

For questions or assistance with the bidirectional compatibility implementation, contact:
- DataScienceBioLab Team (support@datasciencebiolab.com) 