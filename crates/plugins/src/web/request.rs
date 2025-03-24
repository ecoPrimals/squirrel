//! Request handling for web plugins
//!
//! This module provides structs and traits for handling HTTP requests in web plugins.

use std::collections::HashMap;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use serde_json::{Value, json};

use crate::web::http::{HttpMethod, HttpStatus};
use crate::web::routing::Route;

/// Represents an HTTP request to be handled by a web plugin
#[derive(Debug, Clone, Deserialize)]
pub struct WebRequest {
    /// The HTTP method (GET, POST, etc.)
    pub method: HttpMethod,
    /// The request path
    pub path: String,
    /// Query parameters
    pub query_params: HashMap<String, String>,
    /// Headers
    pub headers: HashMap<String, String>,
    /// Request body as JSON (if present)
    pub body: Option<Value>,
    /// User ID if authenticated
    pub user_id: Option<String>,
    /// User permissions
    pub permissions: Vec<String>,
    /// Route parameters (extracted from path)
    #[serde(default)]
    pub route_params: HashMap<String, String>,
}

impl WebRequest {
    /// Create a new WebRequest
    pub fn new(
        method: HttpMethod,
        path: String,
        query_params: HashMap<String, String>,
        headers: HashMap<String, String>,
        body: Option<Value>,
        user_id: Option<String>,
        permissions: Vec<String>,
    ) -> Self {
        Self {
            method,
            path,
            query_params,
            headers,
            body,
            user_id,
            permissions,
            route_params: HashMap::new(),
        }
    }

    /// Extract route parameters from the path based on a route pattern
    pub fn with_route_params(mut self, route_pattern: &str) -> Self {
        let route = Route::new(route_pattern);
        if route.matches(&self.path) {
            if let Some(params) = route.extract_params(&self.path) {
                self.route_params = params;
            }
        }
        self
    }

    /// Check if the request has a specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }

    /// Get a query parameter
    pub fn query(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }

    /// Get a header
    pub fn header(&self, key: &str) -> Option<&String> {
        self.headers.get(&key.to_lowercase())
    }

    /// Get a route parameter
    pub fn param(&self, key: &str) -> Option<&String> {
        self.route_params.get(key)
    }

    /// Get a route parameter, falling back to a query parameter if not found
    pub fn param_or_query(&self, key: &str) -> Option<&String> {
        self.param(key).or_else(|| self.query(key))
    }

    /// Check if the request is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.user_id.is_some()
    }

    /// Parse the body as a specific type
    pub fn parse_body<T>(&self) -> Result<T, serde_json::Error>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        match &self.body {
            Some(body) => serde_json::from_value(body.clone()),
            None => {
                use std::io::{Error, ErrorKind};
                // Create a custom error with a message
                let io_err = Error::new(ErrorKind::InvalidInput, "Request body is empty");
                // Convert to serde_json::Error
                Err(serde_json::Error::io(io_err))
            },
        }
    }
}

/// Represents an HTTP response from a web plugin
#[derive(Debug, Clone, Serialize)]
pub struct WebResponse {
    /// HTTP status code
    pub status: HttpStatus,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body as JSON (if present)
    pub body: Option<Value>,
}

impl WebResponse {
    /// Create a new response with 200 OK status
    pub fn ok(body: Value) -> Self {
        Self {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(body),
        }
    }

    /// Create an empty response with 204 No Content status
    pub fn no_content() -> Self {
        Self {
            status: HttpStatus::NoContent,
            headers: HashMap::new(),
            body: None,
        }
    }

    /// Create a response with 201 Created status
    pub fn created(body: Value) -> Self {
        Self {
            status: HttpStatus::Created,
            headers: HashMap::new(),
            body: Some(body),
        }
    }

    /// Create a response with 400 Bad Request status
    pub fn bad_request(message: &str) -> Self {
        Self {
            status: HttpStatus::BadRequest,
            headers: HashMap::new(),
            body: Some(json!({ "error": message })),
        }
    }

    /// Create a response with 401 Unauthorized status
    pub fn unauthorized(message: &str) -> Self {
        Self {
            status: HttpStatus::Unauthorized,
            headers: HashMap::new(),
            body: Some(json!({ "error": message })),
        }
    }

    /// Create a response with 403 Forbidden status
    pub fn forbidden(message: &str) -> Self {
        Self {
            status: HttpStatus::Forbidden,
            headers: HashMap::new(),
            body: Some(json!({ "error": message })),
        }
    }

    /// Create a response with 404 Not Found status
    pub fn not_found(message: &str) -> Self {
        Self {
            status: HttpStatus::NotFound,
            headers: HashMap::new(),
            body: Some(json!({ "error": message })),
        }
    }

    /// Create a response with 500 Internal Server Error status
    pub fn internal_error(message: &str) -> Self {
        Self {
            status: HttpStatus::InternalServerError,
            headers: HashMap::new(),
            body: Some(json!({ "error": message })),
        }
    }

    /// Add a header to the response
    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    /// Set JSON content type header
    pub fn with_json_content_type(self) -> Self {
        self.with_header("Content-Type", "application/json")
    }
} 