// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! HTTP request/response types for the WASM client.

use crate::error::{PluginError, PluginResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

/// HTTP request methods
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    /// HEAD method
    Head,
    /// OPTIONS method
    Options,
}

impl FromStr for HttpMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "DELETE" => Ok(Self::Delete),
            "PATCH" => Ok(Self::Patch),
            "HEAD" => Ok(Self::Head),
            "OPTIONS" => Ok(Self::Options),
            _ => Err(format!("Unknown HTTP method: {}", s)),
        }
    }
}

impl HttpMethod {
    /// Parse from string (deprecated - use FromStr trait instead)
    #[deprecated(note = "Use FromStr::from_str instead")]
    pub fn parse_method(s: &str) -> Option<Self> {
        s.parse().ok()
    }

    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
        }
    }
}

/// HTTP request configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    /// Request URL
    pub url: String,
    /// HTTP method
    pub method: HttpMethod,
    /// Request headers
    pub headers: HashMap<String, String>,
    /// Request body
    pub body: Option<String>,
    /// Request timeout in milliseconds
    pub timeout_ms: Option<u32>,
    /// Whether to follow redirects
    pub follow_redirects: bool,
}

impl HttpRequest {
    /// Create a new HTTP request
    ///
    /// This constructor creates a new HTTP request with the specified URL and method.
    /// The request will have default configuration including a 30-second timeout
    /// and redirect following enabled.
    ///
    /// # Arguments
    ///
    /// * `url` - The target URL for the request
    /// * `method` - The HTTP method to use (GET, POST, PUT, DELETE, etc.)
    ///
    /// # Returns
    ///
    /// Returns a new `HttpRequest` instance with default configuration.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use squirrel_sdk::http::{HttpRequest, HttpMethod};
    ///
    /// let request = HttpRequest::new(
    ///     "https://api.example.com/users".to_string(),
    ///     HttpMethod::Get
    /// );
    /// ```
    pub fn new(url: String, method: HttpMethod) -> Self {
        Self {
            url,
            method,
            headers: HashMap::new(),
            body: None,
            timeout_ms: Some(30_000), // 30 seconds default
            follow_redirects: true,
        }
    }

    /// Add a header to the request
    ///
    /// This method adds a header to the HTTP request. If a header with the same
    /// name already exists, it will be replaced.
    ///
    /// # Arguments
    ///
    /// * `key` - The header name
    /// * `value` - The header value
    ///
    /// # Returns
    ///
    /// Returns `self` to allow method chaining.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use squirrel_sdk::http::{HttpRequest, HttpMethod};
    ///
    /// let request = HttpRequest::new("https://api.example.com".to_string(), HttpMethod::Get)
    ///     .header("Authorization".to_string(), "Bearer token123".to_string())
    ///     .header("Content-Type".to_string(), "application/json".to_string());
    /// ```
    pub fn header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }

    /// Set the request body
    ///
    /// This method sets the request body content. This is typically used
    /// with POST, PUT, or PATCH requests.
    ///
    /// # Arguments
    ///
    /// * `body` - The request body content as a string
    ///
    /// # Returns
    ///
    /// Returns `self` to allow method chaining.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use squirrel_sdk::http::{HttpRequest, HttpMethod};
    ///
    /// let request = HttpRequest::new("https://api.example.com/users".to_string(), HttpMethod::Post)
    ///     .body(r#"{"name": "John", "email": "john@example.com"}"#.to_string());
    /// ```
    pub fn body(mut self, body: String) -> Self {
        self.body = Some(body);
        self
    }

    /// Set the request timeout
    ///
    /// This method sets the maximum time to wait for a response from the server.
    /// If the timeout is exceeded, the request will fail.
    ///
    /// # Arguments
    ///
    /// * `timeout_ms` - The timeout in milliseconds
    ///
    /// # Returns
    ///
    /// Returns `self` to allow method chaining.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use squirrel_sdk::http::{HttpRequest, HttpMethod};
    ///
    /// let request = HttpRequest::new("https://api.example.com".to_string(), HttpMethod::Get)
    ///     .timeout(5000); // 5 seconds timeout
    /// ```
    pub fn timeout(mut self, timeout_ms: u32) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    /// Set JSON body with automatic Content-Type header
    ///
    /// This method serializes the provided data to JSON and sets it as the request body.
    /// It also automatically sets the Content-Type header to "application/json".
    ///
    /// # Arguments
    ///
    /// * `data` - The data to serialize to JSON
    ///
    /// # Returns
    ///
    /// Returns `Ok(Self)` on success for method chaining, or a `PluginError` if
    /// JSON serialization fails.
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The data cannot be serialized to JSON
    /// - The serialization process fails
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use squirrel_sdk::http::{HttpRequest, HttpMethod};
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct User {
    ///     name: String,
    ///     email: String,
    /// }
    ///
    /// let user = User {
    ///     name: "John Doe".to_string(),
    ///     email: "john@example.com".to_string(),
    /// };
    ///
    /// let request = HttpRequest::new("https://api.example.com/users".to_string(), HttpMethod::Post)
    ///     .json(&user)?;
    /// ```
    pub fn json<T: Serialize>(mut self, data: &T) -> PluginResult<Self> {
        let json_body =
            serde_json::to_string(data).map_err(|e| PluginError::SerializationError {
                message: e.to_string(),
            })?;

        self.body = Some(json_body);
        self.headers
            .insert("Content-Type".to_string(), "application/json".to_string());
        Ok(self)
    }
}

/// HTTP response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    /// Response status code
    pub status: u16,
    /// Response status text
    pub status_text: String,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body
    pub body: String,
    /// Whether the request was successful (2xx status)
    pub ok: bool,
    /// Response URL (may differ from request URL due to redirects)
    pub url: String,
}

impl HttpResponse {
    /// Parse response body as JSON
    pub fn json<T: for<'de> Deserialize<'de>>(&self) -> PluginResult<T> {
        serde_json::from_str(&self.body).map_err(|e| PluginError::SerializationError {
            message: e.to_string(),
        })
    }

    /// Get response body as text
    pub fn text(&self) -> &str {
        &self.body
    }

    /// Check if response is successful
    pub fn is_success(&self) -> bool {
        self.ok
    }

    /// Get header value
    pub fn get_header(&self, name: &str) -> Option<&String> {
        self.headers.get(name)
    }
}
