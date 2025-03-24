//! Example of a custom plugin that can be compiled as a dynamic library
//!
//! This example shows how to implement a plugin that can be loaded
//! by the Squirrel Web application at runtime.
//!
//! To build this as a dynamic library:
//! ```
//! cargo build --example custom_plugin --release --crate-type=cdylib
//! ```
//!
//! The resulting .dll or .so file can be placed in the plugins directory
//! and will be loaded by the WebPluginRegistry.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use uuid::Uuid;

use squirrel_web::plugins::{
    Plugin, PluginMetadata, PluginStatus, WebPlugin,
    model::{WebRequest, WebResponse, WebEndpoint, HttpMethod, ComponentType, WebComponent},
};

/// A custom plugin that can be compiled as a dynamic library
#[derive(Clone)]
pub struct CustomPlugin {
    metadata: PluginMetadata,
    status: PluginStatus,
    counter: Arc<AtomicUsize>,
}

impl CustomPlugin {
    /// Create a new custom plugin
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "custom-dynamic-plugin".to_string(),
                name: "Custom Dynamic Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "A custom plugin loaded dynamically".to_string(),
                author: "Squirrel Team".to_string(),
                repository: Some("https://github.com/squirrel/custom-plugin".to_string()),
                license: Some("MIT".to_string()),
                tags: vec!["example".to_string(), "dynamic".to_string()],
            },
            status: PluginStatus::Active,
            counter: Arc::new(AtomicUsize::new(0)),
        }
    }
}

#[async_trait]
impl Plugin for CustomPlugin {
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

#[async_trait]
impl WebPlugin for CustomPlugin {
    fn get_endpoints(&self) -> Vec<WebEndpoint> {
        vec![
            WebEndpoint::new(
                "/api/custom/hello".to_string(),
                HttpMethod::Get,
                "Returns a hello message".to_string(),
            ),
            WebEndpoint::new(
                "/api/custom/counter".to_string(),
                HttpMethod::Get,
                "Gets the current counter value".to_string(),
            ),
            WebEndpoint::new(
                "/api/custom/counter".to_string(),
                HttpMethod::Post,
                "Increments the counter value".to_string(),
            ),
        ]
    }
    
    fn get_components(&self) -> Vec<WebComponent> {
        vec![
            WebComponent::new(
                "Custom Dynamic Widget".to_string(),
                ComponentType::Widget,
                "A custom dynamic widget".to_string(),
            )
            .with_route("/dashboard".to_string())
            .with_priority(5)
            .with_icon("custom-icon".to_string()),
        ]
    }
    
    async fn handle_request(&self, request: WebRequest) -> Result<WebResponse> {
        match (request.path.as_str(), request.method) {
            ("/api/custom/hello", HttpMethod::Get) => {
                Ok(WebResponse::ok().with_body(json!({
                    "message": "Hello from Custom Dynamic Plugin!",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                })))
            },
            ("/api/custom/counter", HttpMethod::Get) => {
                Ok(WebResponse::ok().with_body(json!({
                    "counter": self.counter.load(Ordering::SeqCst),
                })))
            },
            ("/api/custom/counter", HttpMethod::Post) => {
                // Update the counter
                let new_value = self.counter.fetch_add(1, Ordering::SeqCst) + 1;
                
                Ok(WebResponse::ok().with_body(json!({
                    "counter": new_value,
                    "message": "Counter incremented",
                })))
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
    
    async fn get_component_markup(&self, component_id: Uuid, props: Value) -> Result<String> {
        let counter = self.counter.load(Ordering::SeqCst);
        
        Ok(format!(r#"
            <div class="custom-dynamic-plugin">
                <h3>Custom Dynamic Plugin</h3>
                <div class="content">
                    <p>This component was loaded dynamically.</p>
                    <p>Counter: {}</p>
                    <pre>{}</pre>
                </div>
                <div class="footer">
                    <small>Component ID: {}</small>
                </div>
            </div>
        "#, counter, props, component_id))
    }
}

/// Export the create_plugin function for dynamic loading
///
/// This function will be called by the WebPluginRegistry when the
/// dynamic library is loaded. It should return a raw pointer to
/// a heap-allocated plugin instance that implements WebPlugin.
#[no_mangle]
pub extern "C" fn create_plugin() -> *mut dyn WebPlugin {
    let plugin = CustomPlugin::new();
    let boxed = Box::new(plugin);
    Box::into_raw(boxed)
}

/// Main function for standalone compilation
/// 
/// This is needed when compiling as an example, but not used
/// when compiling as a dynamic library with --crate-type=cdylib
fn main() {
    println!("Custom plugin example");
    println!("To build as a dynamic library:");
    println!("cargo build --example custom_plugin --release --crate-type=cdylib");
    
    // Demonstrate creating a plugin instance
    let plugin = CustomPlugin::new();
    println!("Created plugin: {} v{}", plugin.metadata.name, plugin.metadata.version);
} 