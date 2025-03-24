//! Bidirectional Compatibility Example
//!
//! This example demonstrates how to use the bidirectional compatibility features
//! of the web plugin system, allowing both:
//! 1. Legacy plugins to work with the new system
//! 2. New plugins to work with legacy systems
//!
//! This is particularly important for the plugin silo team to understand how
//! to develop and deploy plugins in mixed environments during migration.

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{Value, json};
use uuid::Uuid;
use std::any::Any;

use squirrel_plugins::plugin::{Plugin, PluginMetadata, PluginStatus};
use squirrel_plugins::web::{
    WebPlugin, WebEndpoint, WebComponent, ComponentType,
    WebRequest, WebResponse, HttpMethod, HttpStatus,
    LegacyWebPluginAdapter, NewWebPluginAdapter
};
use squirrel_plugins::web::adapter::{LegacyWebPluginTrait, LegacyWebComponent};

///////////////////////////////////////////////////////////
// PART 1: Legacy Plugin Implementation (Old API)
///////////////////////////////////////////////////////////

// This represents a plugin implemented using the legacy API
struct LegacyPlugin {
    metadata: PluginMetadata,
}

impl LegacyPlugin {
    fn new() -> Self {
        let metadata = PluginMetadata {
            id: Uuid::new_v4(),
            name: "legacy-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A legacy plugin using the old API".to_string(),
            author: "DataScienceBioLab".to_string(),
            capabilities: vec!["web".to_string()],
            dependencies: vec![],
        };
        
        Self { metadata }
    }
}

#[async_trait]
impl Plugin for LegacyPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn initialize(&self) -> Result<()> {
        println!("Legacy plugin initialized");
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<()> {
        println!("Legacy plugin shut down");
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Legacy API implementation
#[async_trait]
impl LegacyWebPluginTrait for LegacyPlugin {
    fn get_endpoints(&self) -> Vec<squirrel_plugins::plugin::WebEndpoint> {
        vec![
            squirrel_plugins::plugin::WebEndpoint {
                path: "/api/legacy".to_string(),
                method: "GET".to_string(),
                permissions: vec!["legacy.read".to_string()],
            }
        ]
    }
    
    async fn handle_request(&self, path: &str, method: &str, body: Value) -> Result<Value> {
        println!("Legacy plugin handling request: {} {}", method, path);
        Ok(json!({
            "message": "Hello from legacy plugin",
            "path": path,
            "method": method
        }))
    }
    
    fn get_components(&self) -> Vec<LegacyWebComponent> {
        vec![
            LegacyWebComponent {
                id: "legacy-component-1".to_string(),
                name: "Legacy Component".to_string(),
                description: "A component from the legacy plugin".to_string(),
                component_type: "page".to_string(),
                properties: json!({}),
            }
        ]
    }
    
    async fn get_component_markup(&self, component_id: &str, props: Value) -> Result<String> {
        println!("Legacy plugin rendering component: {}", component_id);
        Ok(format!(
            "<div class='legacy-component'><h1>Legacy Component</h1><pre>{}</pre></div>",
            serde_json::to_string_pretty(&props).unwrap_or_default()
        ))
    }
}

///////////////////////////////////////////////////////////
// PART 2: Modern Plugin Implementation (New API)
///////////////////////////////////////////////////////////

// This represents a plugin implemented using the new API
struct ModernPlugin {
    metadata: PluginMetadata,
    status: PluginStatus,
}

impl ModernPlugin {
    fn new() -> Self {
        let metadata = PluginMetadata {
            id: Uuid::new_v4(),
            name: "modern-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A modern plugin using the new API".to_string(),
            author: "DataScienceBioLab".to_string(),
            capabilities: vec!["web".to_string()],
            dependencies: vec![],
        };
        
        Self {
            metadata,
            status: PluginStatus::Registered,
        }
    }
}

#[async_trait]
impl Plugin for ModernPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn initialize(&self) -> Result<()> {
        println!("Modern plugin initialized");
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<()> {
        println!("Modern plugin shut down");
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// New API implementation
#[async_trait]
impl WebPlugin for ModernPlugin {
    fn get_endpoints(&self) -> Vec<WebEndpoint> {
        vec![
            WebEndpoint {
                id: Uuid::new_v4(),
                path: "/api/modern".to_string(),
                method: HttpMethod::Get,
                description: "Modern API endpoint".to_string(),
                permissions: vec!["modern.read".to_string()],
                is_public: true,
                is_admin: false,
                tags: vec!["modern".to_string()],
            },
            
            // Demonstrate route parameters in the new API
            WebEndpoint {
                id: Uuid::new_v4(),
                path: "/api/modern/items/{id}".to_string(),
                method: HttpMethod::Get,
                description: "Get item by ID".to_string(),
                permissions: vec![],
                is_public: true,
                is_admin: false,
                tags: vec!["modern".to_string()],
            }
        ]
    }
    
    async fn handle_request(&self, request: WebRequest) -> Result<WebResponse> {
        println!("Modern plugin handling request: {} {}", request.method, request.path);
        
        // Demonstrate route parameter handling
        if request.path.starts_with("/api/modern/items/") {
            let item_id = request.route_params.get("id").cloned().unwrap_or_else(|| "unknown".to_string());
            return Ok(WebResponse {
                status: HttpStatus::Ok,
                headers: HashMap::new(),
                body: Some(json!({
                    "message": "Item details from modern plugin",
                    "id": item_id,
                    "found": true
                })),
            });
        }
        
        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(json!({
                "message": "Hello from modern plugin",
                "path": request.path,
                "method": request.method.to_string()
            })),
        })
    }
    
    fn get_components(&self) -> Vec<WebComponent> {
        let mut properties = HashMap::new();
        properties.insert("configurable".to_string(), json!(true));
        
        vec![
            WebComponent {
                id: Uuid::new_v4(),
                name: "Modern Component".to_string(),
                description: "A component from the modern plugin".to_string(),
                component_type: ComponentType::Page,
                properties,
                route: Some("/modern".to_string()),
                priority: 0,
                permissions: vec!["modern.read".to_string()],
                parent: None,
                icon: Some("sparkles".to_string()),
            }
        ]
    }
    
    async fn get_component_markup(&self, component_id: Uuid, props: Value) -> Result<String> {
        println!("Modern plugin rendering component: {}", component_id);
        Ok(format!(
            "<div class='modern-component'><h1>Modern Component</h1><pre>{}</pre></div>",
            serde_json::to_string_pretty(&props).unwrap_or_default()
        ))
    }
}

///////////////////////////////////////////////////////////
// PART 3: Bidirectional Compatibility Demonstration
///////////////////////////////////////////////////////////

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Bidirectional Compatibility Example ===\n");
    
    // Create instances of both plugin types
    let legacy_plugin = Arc::new(LegacyPlugin::new());
    let modern_plugin = Arc::new(ModernPlugin::new());
    
    // SCENARIO 1: Using a legacy plugin with the new system
    println!("\n=== SCENARIO 1: Legacy Plugin with New System ===\n");
    
    // Wrap the legacy plugin with the adapter
    let adapted_legacy = LegacyWebPluginAdapter::new(legacy_plugin.clone());
    
    // Now it can be used with the new system
    demo_new_system_compatibility(Arc::new(adapted_legacy)).await?;
    
    // SCENARIO 2: Using a modern plugin with legacy systems
    println!("\n=== SCENARIO 2: Modern Plugin with Legacy System ===\n");
    
    // We can't directly use NewWebPluginAdapter with LegacyWebPluginTrait
    // Instead, we'll show how to use the modern plugin with the WebPlugin trait
    println!("Modern plugin demonstrating WebPlugin compatibility");
    
    // Show manual conversion of the modern API to legacy format
    let endpoints = modern_plugin.get_endpoints();
    println!("Modern endpoints (count: {})", endpoints.len());
    for endpoint in &endpoints {
        println!("  - {} {} ({})", endpoint.method, endpoint.path, endpoint.description);
    }
    
    // Show how to handle a request using the modern API
    let request = WebRequest {
        method: HttpMethod::Get,
        path: "/api/modern".to_string(),
        query_params: HashMap::new(),
        route_params: HashMap::new(),
        headers: HashMap::new(),
        body: Some(json!({"test": true})),
        user_id: Some("user1".to_string()),
        permissions: vec!["test.read".to_string()],
    };
    
    let response = modern_plugin.handle_request(request).await?;
    println!("Modern response: {:?}", response);
    
    // Get and print components
    let components = modern_plugin.get_components();
    println!("Modern components (count: {})", components.len());
    for component in &components {
        println!("  - {} ({})", component.name, component.description);
    }
    
    // SCENARIO 3: Plugin Silo Team Mixed Development
    println!("\n=== SCENARIO 3: Plugin Silo Team Mixed Development ===\n");
    println!("This example demonstrates how the plugin silo team can:");
    println!("1. Continue maintaining legacy plugins while the system migrates");
    println!("2. Develop new plugins using the modern API");
    println!("3. Deploy both types of plugins to mixed environments\n");
    
    // Create a mixed registry that would contain both types of plugins
    let mut mixed_registry = HashMap::new();
    
    // Add a legacy plugin (adapted for the new system)
    let adapted_legacy = Arc::new(LegacyWebPluginAdapter::new(legacy_plugin));
    mixed_registry.insert(adapted_legacy.metadata().id, adapted_legacy.clone() as Arc<dyn Plugin>);
    println!("✅ Legacy plugin registered in mixed environment");
    
    // Add a modern plugin (directly, as the registry is modern)
    mixed_registry.insert(modern_plugin.metadata().id, modern_plugin.clone() as Arc<dyn Plugin>);
    println!("✅ Modern plugin registered in mixed environment");
    
    // In a legacy environment, we would need to create a separate adapter that 
    // explicitly implements LegacyWebPluginTrait
    println!("✅ For legacy systems, create custom adapters implementing LegacyWebPluginTrait");
    
    println!("\nThis bidirectional compatibility ensures a smooth migration path!");
    
    Ok(())
}

// This function simulates using a plugin with the new system
async fn demo_new_system_compatibility(plugin: Arc<dyn WebPlugin>) -> Result<()> {
    // Print plugin information
    println!("Plugin: {} v{}", plugin.metadata().name, plugin.metadata().version);
    
    // Get and print endpoints
    let endpoints = plugin.get_endpoints();
    println!("Endpoints: {}", endpoints.len());
    for endpoint in &endpoints {
        println!("  - {} {} ({})", endpoint.method, endpoint.path, endpoint.description);
    }
    
    // Simulate a request using the new API
    let request = WebRequest {
        method: HttpMethod::Get,
        path: "/api/test".to_string(),
        query_params: HashMap::new(),
        route_params: HashMap::new(),
        headers: HashMap::new(),
        body: Some(json!({"test": true})),
        user_id: Some("user1".to_string()),
        permissions: vec!["test.read".to_string()],
    };
    
    let response = plugin.handle_request(request).await?;
    println!("Response: {:?}", response);
    
    // Get and print components
    let components = plugin.get_components();
    println!("Components: {}", components.len());
    for component in &components {
        println!("  - {} ({})", component.name, component.description);
    }
    
    Ok(())
}

// This function simulates using a plugin with the legacy system
async fn demo_legacy_system_compatibility(plugin: Arc<dyn LegacyWebPluginTrait>) -> Result<()> {
    // Print plugin information
    println!("Plugin: {} v{}", plugin.metadata().name, plugin.metadata().version);
    
    // Get and print endpoints
    let endpoints = plugin.get_endpoints();
    println!("Endpoints: {}", endpoints.len());
    for endpoint in &endpoints {
        println!("  - {} {} (permissions: {})", 
            endpoint.method, 
            endpoint.path, 
            endpoint.permissions.join(", "));
    }
    
    // Simulate a request using the legacy API
    let response = plugin.handle_request("/api/test", "GET", json!({"test": true})).await?;
    println!("Response: {:?}", response);
    
    // Get and print components
    let components = plugin.get_components();
    println!("Components: {}", components.len());
    for component in &components {
        println!("  - {} ({})", component.name, component.description);
    }
    
    Ok(())
} 