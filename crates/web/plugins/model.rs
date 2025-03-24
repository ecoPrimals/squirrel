//! Web plugin model
//!
//! This module defines the web-specific plugin interfaces and data structures.

use std::collections::HashMap;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use uuid::Uuid;

use crate::plugins::core::Plugin;

/// HTTP methods supported by the plugin system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HttpMethod {
    /// GET method
    Get,
    /// POST method
    Post,
    /// PUT method
    Put,
    /// DELETE method
    Delete,
    /// PATCH method
    Patch,
    /// OPTIONS method
    Options,
    /// HEAD method
    Head,
}

/// HTTP status codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HttpStatus {
    /// 200 OK
    Ok = 200,
    /// 201 Created
    Created = 201,
    /// 202 Accepted
    Accepted = 202,
    /// 204 No Content
    NoContent = 204,
    /// 400 Bad Request
    BadRequest = 400,
    /// 401 Unauthorized
    Unauthorized = 401,
    /// 403 Forbidden
    Forbidden = 403,
    /// 404 Not Found
    NotFound = 404,
    /// 405 Method Not Allowed
    MethodNotAllowed = 405,
    /// 409 Conflict
    Conflict = 409,
    /// 422 Unprocessable Entity
    UnprocessableEntity = 422,
    /// 500 Internal Server Error
    InternalServerError = 500,
}

/// Component types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentType {
    /// UI component
    UI,
    /// Form component
    Form,
    /// Dashboard component
    Dashboard,
    /// Navigation component
    Navigation,
    /// Card component
    Card,
    /// Modal component
    Modal,
    /// Custom component
    Custom(String),
}

/// Web endpoint definition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WebEndpoint {
    /// Endpoint ID
    pub id: Uuid,
    /// Path to the endpoint
    pub path: String,
    /// HTTP method
    pub method: HttpMethod,
    /// Description
    pub description: String,
    /// Required permissions
    pub permissions: Vec<String>,
    /// Whether this endpoint is public
    pub is_public: bool,
    /// Whether this endpoint is admin-only
    pub is_admin: bool,
    /// Tags for the endpoint
    pub tags: Vec<String>,
}

impl WebEndpoint {
    /// Create a new endpoint
    pub fn new(
        path: impl Into<String>, 
        method: HttpMethod, 
        description: impl Into<String>
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            path: path.into(),
            method,
            description: description.into(),
            permissions: Vec::new(),
            is_public: false,
            is_admin: false,
            tags: Vec::new(),
        }
    }

    /// Add a permission to the endpoint
    pub fn with_permission(mut self, permission: impl Into<String>) -> Self {
        self.permissions.push(permission.into());
        self
    }

    /// Set whether this endpoint is public
    pub fn with_public(mut self, is_public: bool) -> Self {
        self.is_public = is_public;
        self
    }

    /// Set whether this endpoint is admin-only
    pub fn with_admin(mut self, is_admin: bool) -> Self {
        self.is_admin = is_admin;
        self
    }

    /// Add a tag to the endpoint
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

/// Web component definition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WebComponent {
    /// Component ID
    pub id: Uuid,
    /// Component name
    pub name: String,
    /// Component description
    pub description: String,
    /// Component type
    pub component_type: ComponentType,
    /// Component properties
    pub properties: HashMap<String, Value>,
    /// Route to mount the component
    pub route: Option<String>,
    /// Component priority
    pub priority: i32,
    /// Required permissions
    pub permissions: Vec<String>,
    /// Parent component ID
    pub parent: Option<Uuid>,
    /// Component icon
    pub icon: Option<String>,
}

impl WebComponent {
    /// Create a new component
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        component_type: ComponentType,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: description.into(),
            component_type,
            properties: HashMap::new(),
            route: None,
            priority: 0,
            permissions: Vec::new(),
            parent: None,
            icon: None,
        }
    }

    /// Add a property to the component
    pub fn with_property(mut self, key: impl Into<String>, value: Value) -> Self {
        self.properties.insert(key.into(), value);
        self
    }

    /// Set the route for the component
    pub fn with_route(mut self, route: impl Into<String>) -> Self {
        self.route = Some(route.into());
        self
    }

    /// Set the priority for the component
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Add a permission to the component
    pub fn with_permission(mut self, permission: impl Into<String>) -> Self {
        self.permissions.push(permission.into());
        self
    }

    /// Set the parent component
    pub fn with_parent(mut self, parent: Uuid) -> Self {
        self.parent = Some(parent);
        self
    }

    /// Set the component icon
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

/// Web request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WebRequest {
    /// Request path
    pub path: String,
    /// HTTP method
    pub method: HttpMethod,
    /// Request headers
    pub headers: HashMap<String, String>,
    /// Query parameters
    pub query_params: HashMap<String, String>,
    /// Route parameters
    pub route_params: HashMap<String, String>,
    /// Request body
    pub body: Option<Value>,
    /// User ID
    pub user_id: Option<String>,
    /// User permissions
    pub permissions: Vec<String>,
}

impl WebRequest {
    /// Create a new request
    pub fn new(path: impl Into<String>, method: HttpMethod) -> Self {
        Self {
            path: path.into(),
            method,
            headers: HashMap::new(),
            query_params: HashMap::new(),
            route_params: HashMap::new(),
            body: None,
            user_id: None,
            permissions: Vec::new(),
        }
    }

    /// Add a header to the request
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Add a query parameter to the request
    pub fn with_query_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query_params.insert(key.into(), value.into());
        self
    }

    /// Add a route parameter to the request
    pub fn with_route_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.route_params.insert(key.into(), value.into());
        self
    }

    /// Set the request body
    pub fn with_body(mut self, body: Value) -> Self {
        self.body = Some(body);
        self
    }

    /// Set the user ID
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Add a permission to the request
    pub fn with_permission(mut self, permission: impl Into<String>) -> Self {
        self.permissions.push(permission.into());
        self
    }
}

/// Web response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WebResponse {
    /// HTTP status
    pub status: HttpStatus,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body
    pub body: Option<Value>,
}

impl WebResponse {
    /// Create a new response
    pub fn new(status: HttpStatus) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: None,
        }
    }

    /// Add a header to the response
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set the response body
    pub fn with_body(mut self, body: Value) -> Self {
        self.body = Some(body);
        self
    }

    /// Create a success response
    pub fn ok() -> Self {
        Self::new(HttpStatus::Ok)
    }

    /// Create a success response with body
    pub fn ok_with_body(body: Value) -> Self {
        Self::new(HttpStatus::Ok).with_body(body)
    }

    /// Create a created response
    pub fn created() -> Self {
        Self::new(HttpStatus::Created)
    }

    /// Create a created response with body
    pub fn created_with_body(body: Value) -> Self {
        Self::new(HttpStatus::Created).with_body(body)
    }

    /// Create a no content response
    pub fn no_content() -> Self {
        Self::new(HttpStatus::NoContent)
    }

    /// Create a bad request response
    pub fn bad_request() -> Self {
        Self::new(HttpStatus::BadRequest)
    }

    /// Create a bad request response with message
    pub fn bad_request_with_message(message: impl Into<String>) -> Self {
        Self::new(HttpStatus::BadRequest).with_body(serde_json::json!({
            "error": message.into()
        }))
    }

    /// Create an unauthorized response
    pub fn unauthorized() -> Self {
        Self::new(HttpStatus::Unauthorized)
    }

    /// Create a forbidden response
    pub fn forbidden() -> Self {
        Self::new(HttpStatus::Forbidden)
    }

    /// Create a not found response
    pub fn not_found() -> Self {
        Self::new(HttpStatus::NotFound)
    }

    /// Create an internal server error response
    pub fn internal_server_error() -> Self {
        Self::new(HttpStatus::InternalServerError)
    }
}

/// Web plugin trait
#[async_trait]
pub trait WebPlugin: Plugin + Send + Sync {
    /// Get endpoints provided by this plugin
    fn get_endpoints(&self) -> Vec<WebEndpoint>;
    
    /// Get components provided by this plugin
    fn get_components(&self) -> Vec<WebComponent>;
    
    /// Handle a web request
    async fn handle_request(&self, request: WebRequest) -> Result<WebResponse>;
    
    /// Get component markup
    async fn get_component_markup(&self, component_id: Uuid, props: Value) -> Result<String>;
} 