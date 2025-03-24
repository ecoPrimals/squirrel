//! Plugin system data models
//!
//! This module contains the data models used by the plugin system.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    /// 429 Too Many Requests
    TooManyRequests = 429,
    /// 500 Internal Server Error
    InternalServerError = 500,
    /// 503 Service Unavailable
    ServiceUnavailable = 503,
}

/// Component types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentType {
    /// UI widget
    Widget,
    /// Menu item
    MenuItem,
    /// Dashboard widget
    Dashboard,
    /// Panel
    Panel,
    /// Modal
    Modal,
    /// Form
    Form,
    /// Custom component
    Custom,
}

/// Web request model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRequest {
    /// Request path
    pub path: String,
    /// HTTP method
    pub method: HttpMethod,
    /// Headers
    pub headers: HashMap<String, String>,
    /// Query parameters
    pub query_params: HashMap<String, String>,
    /// Request body
    pub body: Option<serde_json::Value>,
}

impl WebRequest {
    /// Create a new request
    pub fn new(path: String, method: HttpMethod) -> Self {
        Self {
            path,
            method,
            headers: HashMap::new(),
            query_params: HashMap::new(),
            body: None,
        }
    }
    
    /// Add a header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
    
    /// Add a query parameter
    pub fn with_query_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query_params.insert(key.into(), value.into());
        self
    }
    
    /// Set the body
    pub fn with_body(mut self, body: serde_json::Value) -> Self {
        self.body = Some(body);
        self
    }
}

/// Web response model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebResponse {
    /// HTTP status code
    pub status: HttpStatus,
    /// Headers
    pub headers: HashMap<String, String>,
    /// Response body
    pub body: Option<serde_json::Value>,
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
    
    /// Create a 200 OK response
    pub fn ok() -> Self {
        Self::new(HttpStatus::Ok)
    }
    
    /// Create a 201 Created response
    pub fn created() -> Self {
        Self::new(HttpStatus::Created)
    }
    
    /// Create a 204 No Content response
    pub fn no_content() -> Self {
        Self::new(HttpStatus::NoContent)
    }
    
    /// Create a 400 Bad Request response
    pub fn bad_request() -> Self {
        Self::new(HttpStatus::BadRequest)
    }
    
    /// Create a 401 Unauthorized response
    pub fn unauthorized() -> Self {
        Self::new(HttpStatus::Unauthorized)
    }
    
    /// Create a 403 Forbidden response
    pub fn forbidden() -> Self {
        Self::new(HttpStatus::Forbidden)
    }
    
    /// Create a 404 Not Found response
    pub fn not_found() -> Self {
        Self::new(HttpStatus::NotFound)
    }
    
    /// Create a 409 Conflict response
    pub fn conflict() -> Self {
        Self::new(HttpStatus::Conflict)
    }
    
    /// Create a 500 Internal Server Error response
    pub fn internal_server_error() -> Self {
        Self::new(HttpStatus::InternalServerError)
    }
    
    /// Create a 503 Service Unavailable response
    pub fn service_unavailable() -> Self {
        Self::new(HttpStatus::ServiceUnavailable)
    }
    
    /// Add a header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
    
    /// Set the body
    pub fn with_body(mut self, body: serde_json::Value) -> Self {
        self.body = Some(body);
        self
    }
}

/// Web endpoint model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebEndpoint {
    /// Endpoint ID
    pub id: Uuid,
    /// Endpoint path
    pub path: String,
    /// HTTP method
    pub method: HttpMethod,
    /// Endpoint description
    pub description: String,
    /// Required permissions
    pub permissions: Vec<String>,
    /// Whether the endpoint is public (no authentication required)
    pub is_public: bool,
    /// Whether the endpoint is admin-only
    pub is_admin: bool,
    /// Tags for categorization
    pub tags: Vec<String>,
}

impl WebEndpoint {
    /// Create a new endpoint
    pub fn new(path: String, method: HttpMethod, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            path,
            method,
            description,
            permissions: Vec::new(),
            is_public: false,
            is_admin: false,
            tags: Vec::new(),
        }
    }
    
    /// Add a permission
    pub fn with_permission(mut self, permission: String) -> Self {
        self.permissions.push(permission);
        self
    }
    
    /// Set as public
    pub fn with_is_public(mut self, is_public: bool) -> Self {
        self.is_public = is_public;
        self
    }
    
    /// Set as admin-only
    pub fn with_is_admin(mut self, is_admin: bool) -> Self {
        self.is_admin = is_admin;
        self
    }
    
    /// Add a tag
    pub fn with_tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }
}

/// Web component model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebComponent {
    /// Component ID
    pub id: Uuid,
    /// Component name
    pub name: String,
    /// Component type
    pub component_type: ComponentType,
    /// Component description
    pub description: String,
    /// Component properties schema
    pub properties: Option<serde_json::Value>,
    /// Component route (if applicable)
    pub route: Option<String>,
    /// Component priority (for ordering)
    pub priority: i32,
    /// Required permissions
    pub permissions: Vec<String>,
    /// Parent component ID (if applicable)
    pub parent: Option<Uuid>,
    /// Component icon
    pub icon: Option<String>,
}

impl WebComponent {
    /// Create a new component
    pub fn new(name: String, component_type: ComponentType, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            component_type,
            description,
            properties: None,
            route: None,
            priority: 0,
            permissions: Vec::new(),
            parent: None,
            icon: None,
        }
    }
    
    /// Set properties schema
    pub fn with_properties(mut self, properties: serde_json::Value) -> Self {
        self.properties = Some(properties);
        self
    }
    
    /// Set component route
    pub fn with_route(mut self, route: String) -> Self {
        self.route = Some(route);
        self
    }
    
    /// Set component priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
    
    /// Add a permission
    pub fn with_permission(mut self, permission: String) -> Self {
        self.permissions.push(permission);
        self
    }
    
    /// Set parent component
    pub fn with_parent(mut self, parent: Uuid) -> Self {
        self.parent = Some(parent);
        self
    }
    
    /// Set component icon
    pub fn with_icon(mut self, icon: String) -> Self {
        self.icon = Some(icon);
        self
    }
} 