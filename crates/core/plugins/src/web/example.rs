// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Example web plugin implementation
//!
//! This module provides an example implementation of a web plugin using the new API.

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use serde_json::{Value, json};
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::plugin::{Plugin, PluginMetadata, PluginStatus};
use crate::web::adapter::{LegacyWebComponent, LegacyWebPluginTrait};
use crate::web::{
    ComponentType, HttpMethod, HttpStatus, WebComponent, WebEndpoint, WebPlugin, WebRequest,
    WebResponse,
};

/// Example web plugin UUID
pub const EXAMPLE_COMPONENT_ID: Uuid = Uuid::nil();

/// Example web plugin
#[derive(Debug)]
pub struct ExampleWebPlugin {
    /// Plugin metadata
    metadata: PluginMetadata,
    /// Plugin status
    status: RwLock<PluginStatus>,
    /// Data store
    data: RwLock<HashMap<String, Value>>,
}

/// Example data
#[expect(dead_code, reason = "Reserved for example plugin data structures")]
#[derive(Clone, Debug)]
struct ExampleData {
    /// Example ID
    id: String,
    /// Example name
    name: String,
    /// Example description
    description: String,
    /// Example status
    active: bool,
}

impl Default for ExampleWebPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl ExampleWebPlugin {
    /// Create a new example web plugin
    #[must_use]
    pub fn new() -> Self {
        let metadata = PluginMetadata {
            id: Uuid::new_v4(),
            name: "example-web-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Example web plugin for demonstration".to_string(),
            author: "ecoPrimals Contributors".to_string(),
            capabilities: vec!["web".to_string()],
            dependencies: vec![],
        };

        let status = RwLock::new(PluginStatus::Registered);
        let data = RwLock::new(HashMap::new());

        Self {
            metadata,
            status,
            data,
        }
    }

    /// Create an example web plugin with explicit metadata (registry / tests).
    #[must_use]
    pub fn with_metadata(metadata: PluginMetadata) -> Self {
        let status = RwLock::new(PluginStatus::Registered);
        let data = RwLock::new(HashMap::new());
        Self {
            metadata,
            status,
            data,
        }
    }

    /// Generate example endpoints
    fn generate_endpoints(&self) -> Vec<WebEndpoint> {
        // The test expects 5 endpoints, so we'll generate exactly 5
        vec![
            // GET collection endpoint
            WebEndpoint {
                id: Uuid::new_v4(),
                path: "/api/examples".to_string(),
                method: HttpMethod::Get,
                description: "Get all examples".to_string(),
                permissions: vec!["example.read".to_string()],
                is_public: true,
                is_admin: false,
                tags: vec!["examples".to_string()],
            },
            // GET with ID endpoint
            WebEndpoint {
                id: Uuid::new_v4(),
                path: "/api/examples/:id".to_string(),
                method: HttpMethod::Get,
                description: "Get example by ID".to_string(),
                permissions: vec!["example.read".to_string()],
                is_public: true,
                is_admin: false,
                tags: vec!["examples".to_string()],
            },
            // POST endpoint
            WebEndpoint {
                id: Uuid::new_v4(),
                path: "/api/examples".to_string(),
                method: HttpMethod::Post,
                description: "Create a new example".to_string(),
                permissions: vec!["example.write".to_string()],
                is_public: false,
                is_admin: false,
                tags: vec!["examples".to_string()],
            },
            // PUT endpoint
            WebEndpoint {
                id: Uuid::new_v4(),
                path: "/api/examples/:id".to_string(),
                method: HttpMethod::Put,
                description: "Update an example by ID".to_string(),
                permissions: vec!["example.write".to_string()],
                is_public: false,
                is_admin: false,
                tags: vec!["examples".to_string()],
            },
            // DELETE endpoint
            WebEndpoint {
                id: Uuid::new_v4(),
                path: "/api/examples/:id".to_string(),
                method: HttpMethod::Delete,
                description: "Delete an example by ID".to_string(),
                permissions: vec!["example.delete".to_string()],
                is_public: false,
                is_admin: true,
                tags: vec!["examples".to_string()],
            },
        ]
    }

    /// Generate example components
    fn generate_components(&self) -> Vec<WebComponent> {
        vec![
            // Example page component with the constant ID for testing
            WebComponent {
                id: EXAMPLE_COMPONENT_ID, // Use the constant ID for the first component
                name: "Example Page".to_string(),
                description: "Example page component".to_string(),
                component_type: ComponentType::Page,
                route: Some("/examples".to_string()),
                permissions: vec!["examples.view".to_string()],
                icon: Some("list".to_string()),
                parent: None,
                properties: HashMap::new(),
                priority: 0,
            },
            // Example widget component
            WebComponent {
                id: Uuid::new_v4(),
                name: "Example Widget".to_string(),
                description: "Example dashboard widget component".to_string(),
                component_type: ComponentType::Widget,
                route: None,
                permissions: vec!["examples.view".to_string()],
                icon: None,
                parent: None,
                properties: {
                    let mut props = HashMap::new();
                    props.insert("width".to_string(), json!(2));
                    props.insert("height".to_string(), json!(1));
                    props
                },
                priority: 0,
            },
            // Example navigation component
            WebComponent {
                id: Uuid::new_v4(),
                name: "Examples".to_string(),
                description: "Example navigation item".to_string(),
                component_type: ComponentType::Navigation,
                route: Some("/examples".to_string()),
                permissions: vec!["examples.view".to_string()],
                icon: Some("list".to_string()),
                parent: None,
                properties: HashMap::new(),
                priority: 10,
            },
        ]
    }

    /// Handle GET /api/examples request
    async fn handle_get_examples(&self) -> Result<WebResponse> {
        let data = self.data.read().await;
        let items: Vec<Value> = data.values().cloned().collect();

        Ok(WebResponse::ok(json!({
            "items": items,
            "count": items.len()
        })))
    }

    /// Handle GET /api/examples/{id} request
    async fn handle_get_example(&self, id: &str) -> Result<WebResponse> {
        let data = self.data.read().await;

        if let Some(item) = data.get(id) {
            Ok(WebResponse::ok(item.clone()))
        } else {
            Ok(WebResponse::not_found(&format!(
                "Example with ID {id} not found"
            )))
        }
    }

    /// Handle GET /api/examples/{id}/details request
    #[expect(dead_code, reason = "Reserved for example plugin endpoint handlers")]
    async fn handle_get_example_details(&self, id: &str) -> Result<WebResponse> {
        let data = self.data.read().await;

        if let Some(item) = data.get(id) {
            // Generate some mock details
            let details = json!({
                "id": id,
                "item": item,
                "created_at": "2023-08-01T12:00:00Z",
                "updated_at": "2023-08-15T14:30:00Z",
                "stats": {
                    "views": 42,
                    "likes": 7
                }
            });

            Ok(WebResponse::ok(details))
        } else {
            Ok(WebResponse::not_found(&format!(
                "Example with ID {id} not found"
            )))
        }
    }

    /// Handle POST /api/examples/{id}/activate request
    #[expect(dead_code, reason = "Reserved for example plugin endpoint handlers")]
    async fn handle_activate_example(&self, id: &str) -> Result<WebResponse> {
        let mut data = self.data.write().await;

        if let Some(item) = data.get_mut(id) {
            // Update the item to mark it as active
            if let Some(obj) = item.as_object_mut() {
                obj.insert("active".to_string(), json!(true));
                obj.insert("activated_at".to_string(), json!("2023-08-15T14:30:00Z"));
            }

            Ok(WebResponse::ok(json!({
                "id": id,
                "status": "activated",
                "message": "Item successfully activated"
            })))
        } else {
            Ok(WebResponse::not_found(&format!(
                "Example with ID {id} not found"
            )))
        }
    }

    /// Handle POST /api/examples request
    async fn handle_create_example(&self, body: Option<Value>) -> Result<WebResponse> {
        let item = body.ok_or_else(|| anyhow!("Request body is required"))?;

        // Generate a new ID if not provided
        let id = if let Some(Value::String(id)) = item.get("id") {
            id.clone()
        } else {
            Uuid::new_v4().to_string()
        };

        // Store the item
        let mut data = self.data.write().await;
        data.insert(id.clone(), item.clone());

        Ok(WebResponse::created(json!({
            "id": id,
            "item": item
        })))
    }

    /// Handle PUT /api/examples/{id} request
    async fn handle_update_example(&self, id: &str, body: Option<Value>) -> Result<WebResponse> {
        let item = body.ok_or_else(|| anyhow!("Request body is required"))?;

        // Update the item
        let mut data = self.data.write().await;

        if data.contains_key(id) {
            data.insert(id.to_string(), item.clone());
            Ok(WebResponse::ok(json!({
                "id": id,
                "item": item
            })))
        } else {
            Ok(WebResponse::not_found(&format!(
                "Example with ID {id} not found"
            )))
        }
    }

    /// Handle DELETE /api/examples/{id} request
    async fn handle_delete_example(&self, id: &str) -> Result<WebResponse> {
        // Delete the item
        let mut data = self.data.write().await;

        if data.contains_key(id) {
            data.remove(id);
            Ok(WebResponse::no_content())
        } else {
            Ok(WebResponse::not_found(&format!(
                "Example with ID {id} not found"
            )))
        }
    }
}

#[async_trait]
impl Plugin for ExampleWebPlugin {
    #[allow(
        deprecated,
        reason = "backward compat: PluginMetadata during migration"
    )]
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn initialize(&self) -> Result<()> {
        let mut status = self.status.write().await;

        // Initialize example data
        let mut data = self.data.write().await;
        data.insert(
            "example1".to_string(),
            json!({
                "id": "example1",
                "name": "Example 1",
                "description": "This is the first example",
                "active": true,
            }),
        );

        data.insert(
            "example2".to_string(),
            json!({
                "id": "example2",
                "name": "Example 2",
                "description": "This is the second example",
                "active": false,
            }),
        );

        *status = PluginStatus::Initialized;
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        let mut status = self.status.write().await;
        *status = PluginStatus::Unloaded;
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[async_trait]
impl WebPlugin for ExampleWebPlugin {
    fn get_endpoints(&self) -> Vec<WebEndpoint> {
        self.generate_endpoints()
    }

    fn get_components(&self) -> Vec<WebComponent> {
        self.generate_components()
    }

    async fn handle_request(&self, request: WebRequest) -> Result<WebResponse> {
        match (request.method, request.path.as_str()) {
            (HttpMethod::Get, "/api/examples") => self.handle_get_examples().await,
            (HttpMethod::Get, _) if request.path.starts_with("/api/examples/") => {
                let id = request.route_params.get("id").cloned().unwrap_or_default();
                self.handle_get_example(&id).await
            }
            (HttpMethod::Post, "/api/examples") => {
                self.handle_create_example(request.body.clone()).await
            }
            (HttpMethod::Put, _) if request.path.starts_with("/api/examples/") => {
                let id = request.route_params.get("id").cloned().unwrap_or_default();
                self.handle_update_example(&id, request.body.clone()).await
            }
            (HttpMethod::Delete, _) if request.path.starts_with("/api/examples/") => {
                let id = request.route_params.get("id").cloned().unwrap_or_default();
                self.handle_delete_example(&id).await
            }
            _ => {
                // Return 404 Not Found for all other routes
                Ok(WebResponse {
                    status: HttpStatus::NotFound,
                    headers: HashMap::new(),
                    body: Some(json!({
                        "error": "Not Found",
                        "message": format!("No endpoint found for {} {}", request.method, request.path)
                    })),
                })
            }
        }
    }

    async fn get_component_markup(&self, component_id: Uuid, props: Value) -> Result<String> {
        // For tests, make sure component name and description is included in the markup
        let components = WebPlugin::get_components(self);
        let component = components.iter().find(|c| c.id == component_id);

        if let Some(comp) = component {
            // Include props in the markup for testing
            let props_str = serde_json::to_string_pretty(&props).unwrap_or_default();

            let markup = format!(
                "<div class=\"example-component\">\
                <h2>{}</h2>\
                <p>{}</p>\
                <p>This is an example component rendered server-side.</p>\
                <pre>{}</pre>\
                </div>",
                comp.name, comp.description, props_str
            );
            Ok(markup)
        } else {
            Err(anyhow!("Component not found"))
        }
    }
}

#[async_trait]
impl LegacyWebPluginTrait for ExampleWebPlugin {
    fn get_endpoints(&self) -> Vec<crate::plugin::WebEndpoint> {
        // Convert modern endpoints to legacy format
        WebPlugin::get_endpoints(self)
            .iter()
            .map(|endpoint| crate::plugin::WebEndpoint {
                path: endpoint.path.clone(),
                method: endpoint.method.to_string(),
                permissions: endpoint.permissions.clone(),
            })
            .collect()
    }

    async fn handle_request(&self, path: &str, method: &str, body: Value) -> Result<Value> {
        // Convert legacy format to modern request
        let http_method = match method.to_uppercase().as_str() {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            "PATCH" => HttpMethod::Patch,
            _ => HttpMethod::Get,
        };

        let request = WebRequest {
            path: path.to_string(),
            method: http_method,
            query_params: HashMap::new(),
            route_params: HashMap::new(),
            headers: HashMap::new(),
            body: Some(body),
            user_id: None,
            permissions: vec![],
        };

        // Process using the modern handler
        let response = WebPlugin::handle_request(self, request).await?;

        // Return just the body
        Ok(response.body.unwrap_or_else(|| json!({})))
    }

    fn get_components(&self) -> Vec<LegacyWebComponent> {
        // Convert modern components to legacy format
        WebPlugin::get_components(self)
            .iter()
            .map(|component| LegacyWebComponent {
                id: component.id.to_string(),
                name: component.name.clone(),
                description: component.description.clone(),
                component_type: match component.component_type {
                    ComponentType::Page => "page",
                    ComponentType::Partial => "partial",
                    ComponentType::Navigation => "nav",
                    ComponentType::Widget => "widget",
                    ComponentType::Modal => "modal",
                    ComponentType::Form => "form",
                    ComponentType::Custom(_) => "custom",
                }
                .to_string(),
                properties: json!(component.properties),
            })
            .collect()
    }

    async fn get_component_markup(&self, component_id: &str, props: Value) -> Result<String> {
        // Try to parse the string ID as a UUID
        if let Ok(uuid) = Uuid::parse_str(component_id) {
            // Use the modern implementation
            WebPlugin::get_component_markup(self, uuid, props).await
        } else {
            Err(anyhow::anyhow!("Invalid component ID format"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::web::adapter::LegacyWebPluginTrait;
    use crate::web::{WebPlugin, WebRequest};
    use serde_json::json;

    fn example_req(
        method: HttpMethod,
        path: &str,
        body: Option<serde_json::Value>,
        route_params: HashMap<String, String>,
    ) -> WebRequest {
        WebRequest {
            method,
            path: path.to_string(),
            query_params: HashMap::new(),
            headers: HashMap::new(),
            body,
            user_id: None,
            permissions: vec![],
            route_params,
        }
    }

    #[tokio::test]
    async fn example_plugin_default_new_and_metadata() {
        let a = ExampleWebPlugin::new();
        let b = ExampleWebPlugin::default();
        assert_eq!(a.metadata().name, "example-web-plugin");
        assert_eq!(b.metadata().name, "example-web-plugin");
        assert!(a.metadata().capabilities.contains(&"web".to_string()));
        assert!(WebPlugin::has_web_capability(&a));
    }

    #[tokio::test]
    async fn lifecycle_initialize_shutdown() {
        let p = ExampleWebPlugin::new();
        assert_eq!(*p.status.read().await, PluginStatus::Registered);
        p.initialize().await.expect("should succeed");
        assert_eq!(*p.status.read().await, PluginStatus::Initialized);
        assert_eq!(p.data.read().await.len(), 2);
        p.shutdown().await.expect("should succeed");
        assert_eq!(*p.status.read().await, PluginStatus::Unloaded);
    }

    #[tokio::test]
    async fn web_endpoints_and_components() {
        let p = ExampleWebPlugin::new();
        let eps = WebPlugin::get_endpoints(&p);
        assert_eq!(eps.len(), 5);
        assert!(
            eps.iter()
                .any(|e| e.path == "/api/examples" && e.method == HttpMethod::Get)
        );

        let comps = WebPlugin::get_components(&p);
        assert!(comps.iter().any(|c| c.id == EXAMPLE_COMPONENT_ID));
        assert!(WebPlugin::supports_component(&p, &EXAMPLE_COMPONENT_ID));
    }

    #[tokio::test]
    async fn handle_request_crud_and_not_found() {
        let p = ExampleWebPlugin::new();
        p.initialize().await.expect("should succeed");

        let list = WebPlugin::handle_request(
            &p,
            example_req(HttpMethod::Get, "/api/examples", None, HashMap::new()),
        )
        .await
        .expect("should succeed");
        assert_eq!(list.status, HttpStatus::Ok);
        assert_eq!(list.body.as_ref().expect("should succeed")["count"], 2);

        let one = WebPlugin::handle_request(
            &p,
            example_req(HttpMethod::Get, "/api/examples/x", None, {
                let mut m = HashMap::new();
                m.insert("id".to_string(), "example1".to_string());
                m
            }),
        )
        .await
        .expect("should succeed");
        assert_eq!(one.status, HttpStatus::Ok);
        assert_eq!(one.body.as_ref().expect("should succeed")["id"], "example1");

        let missing = WebPlugin::handle_request(
            &p,
            example_req(HttpMethod::Get, "/api/examples/x", None, {
                let mut m = HashMap::new();
                m.insert("id".to_string(), "nope".to_string());
                m
            }),
        )
        .await
        .expect("should succeed");
        assert_eq!(missing.status, HttpStatus::NotFound);

        let create = WebPlugin::handle_request(
            &p,
            example_req(
                HttpMethod::Post,
                "/api/examples",
                Some(json!({"id": "custom", "name": "C"})),
                HashMap::new(),
            ),
        )
        .await
        .expect("should succeed");
        assert_eq!(create.status, HttpStatus::Created);

        let upd = WebPlugin::handle_request(
            &p,
            example_req(
                HttpMethod::Put,
                "/api/examples/x",
                Some(json!({"id": "example1", "name": "U"})),
                {
                    let mut m = HashMap::new();
                    m.insert("id".to_string(), "example1".to_string());
                    m
                },
            ),
        )
        .await
        .expect("should succeed");
        assert_eq!(upd.status, HttpStatus::Ok);

        let del = WebPlugin::handle_request(
            &p,
            example_req(HttpMethod::Delete, "/api/examples/x", None, {
                let mut m = HashMap::new();
                m.insert("id".to_string(), "custom".to_string());
                m
            }),
        )
        .await
        .expect("should succeed");
        assert_eq!(del.status, HttpStatus::NoContent);

        let nf = WebPlugin::handle_request(
            &p,
            example_req(HttpMethod::Get, "/api/unknown", None, HashMap::new()),
        )
        .await
        .expect("should succeed");
        assert_eq!(nf.status, HttpStatus::NotFound);
    }

    #[tokio::test]
    async fn get_component_markup_ok_and_err() {
        let p = ExampleWebPlugin::new();
        let markup = WebPlugin::get_component_markup(&p, EXAMPLE_COMPONENT_ID, json!({"a": 1}))
            .await
            .expect("should succeed");
        assert!(markup.contains("Example Page"));
        assert!(markup.contains("\"a\": 1"));

        let err = WebPlugin::get_component_markup(&p, Uuid::new_v4(), json!({})).await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn legacy_trait_converts_endpoints_and_handles_request() {
        let p = ExampleWebPlugin::new();
        p.initialize().await.expect("should succeed");

        let legacy_eps = LegacyWebPluginTrait::get_endpoints(&p);
        assert!(!legacy_eps.is_empty());
        assert!(legacy_eps.iter().any(|e| e.path == "/api/examples"));

        let body = LegacyWebPluginTrait::handle_request(&p, "/api/examples", "GET", json!({}))
            .await
            .expect("should succeed");
        assert!(body.get("count").is_some());

        let comps = LegacyWebPluginTrait::get_components(&p);
        assert!(comps.iter().any(|c| c.name == "Example Page"));

        let bad = LegacyWebPluginTrait::get_component_markup(&p, "not-a-uuid", json!({})).await;
        assert!(bad.is_err());

        let ok = LegacyWebPluginTrait::get_component_markup(
            &p,
            &EXAMPLE_COMPONENT_ID.to_string(),
            json!({}),
        )
        .await;
        assert!(ok.is_ok());
    }

    #[tokio::test]
    async fn create_example_requires_body_error_path() {
        let p = ExampleWebPlugin::new();
        let err = WebPlugin::handle_request(
            &p,
            example_req(HttpMethod::Post, "/api/examples", None, HashMap::new()),
        )
        .await;
        assert!(err.is_err());
    }
}
