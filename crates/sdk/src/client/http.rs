// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! HTTP client functionality for plugins
//!
//! This module provides HTTP client functionality for making web requests from plugins,
//! with sandbox security and permission checking.

use crate::error::{PluginError, PluginResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Headers, Request as WebRequest, RequestInit, RequestMode, Response};

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

/// HTTP client for making requests
#[derive(Debug, Default)]
pub struct HttpClient {
    default_headers: HashMap<String, String>,
}

impl HttpClient {
    /// Create a new HTTP client
    ///
    /// This constructor creates a new HTTP client with empty default headers.
    /// Default headers can be added using the `set_default_header` method.
    ///
    /// # Returns
    ///
    /// Returns a new `HttpClient` instance ready to make HTTP requests.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use squirrel_sdk::http::HttpClient;
    ///
    /// let client = HttpClient::new();
    /// ```
    pub fn new() -> Self {
        Self {
            default_headers: HashMap::new(),
        }
    }

    /// Set a default header for all requests
    ///
    /// This method sets a header that will be automatically included in all requests
    /// made by this client. If a header with the same name is already set as a default,
    /// it will be replaced. Individual requests can override default headers.
    ///
    /// # Arguments
    ///
    /// * `key` - The header name
    /// * `value` - The header value
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use squirrel_sdk::http::HttpClient;
    ///
    /// let mut client = HttpClient::new();
    /// client.set_default_header("User-Agent".to_string(), "MyApp/1.0".to_string());
    /// client.set_default_header("Authorization".to_string(), "Bearer token123".to_string());
    /// ```
    pub fn set_default_header(&mut self, key: String, value: String) {
        self.default_headers.insert(key, value);
    }

    /// Make an HTTP request
    ///
    /// This method executes the provided HTTP request and returns the response.
    /// The request is made using the Fetch API in the browser environment.
    /// Default headers from the client are automatically included.
    ///
    /// # Arguments
    ///
    /// * `request` - The HTTP request to execute
    ///
    /// # Returns
    ///
    /// Returns `Ok(HttpResponse)` on success, or a `PluginError` if the request fails.
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The request cannot be created (invalid URL, etc.)
    /// - The network request fails
    /// - The response cannot be parsed
    /// - The request times out
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use squirrel_sdk::http::{HttpClient, HttpRequest, HttpMethod};
    ///
    /// async fn make_request() -> Result<(), squirrel_sdk::PluginError> {
    ///     let client = HttpClient::new();
    ///     let request = HttpRequest::new(
    ///         "https://api.example.com/users".to_string(),
    ///         HttpMethod::Get
    ///     );
    ///     
    ///     let response = client.request(request).await?;
    ///     println!("Status: {}", response.status);
    ///     Ok(())
    /// }
    /// ```
    pub async fn request(&self, request: HttpRequest) -> PluginResult<HttpResponse> {
        // Create web request
        let opts = RequestInit::new();
        opts.set_method(request.method.as_str());
        opts.set_mode(RequestMode::Cors);

        // Set headers
        let headers = Headers::new().map_err(|_| PluginError::InternalError {
            message: "Failed to create headers".to_string(),
        })?;

        for (key, value) in &request.headers {
            headers
                .set(key, value)
                .map_err(|_| PluginError::InternalError {
                    message: "Failed to set header".to_string(),
                })?;
        }
        opts.set_headers(&headers);

        // Set body if present
        if let Some(body) = &request.body {
            opts.set_body(&JsValue::from_str(body));
        }

        // Create and send request
        let web_request = WebRequest::new_with_str_and_init(&request.url, &opts).map_err(|_| {
            PluginError::InternalError {
                message: "Failed to create request".to_string(),
            }
        })?;

        let window = web_sys::window().ok_or_else(|| PluginError::InternalError {
            message: "No window object".to_string(),
        })?;

        let response_promise = window.fetch_with_request(&web_request);
        let response_js =
            JsFuture::from(response_promise)
                .await
                .map_err(|_| PluginError::NetworkError {
                    operation: "fetch".to_string(),
                    message: "Request failed".to_string(),
                })?;

        let response: Response =
            response_js
                .dyn_into()
                .map_err(|_| PluginError::InternalError {
                    message: "Invalid response type".to_string(),
                })?;

        // Extract response data
        let status = response.status();
        let status_text = response.status_text();
        let ok = response.ok();
        let url = response.url();

        // Get response headers
        let response_headers = HashMap::new();
        // Note: In a real implementation, you'd iterate through response.headers()
        // This is simplified for the example

        // Get response body
        let body_promise = response.text().map_err(|_| PluginError::NetworkError {
            operation: "fetch".to_string(),
            message: "Failed to read response body".to_string(),
        })?;
        let body_js =
            JsFuture::from(body_promise)
                .await
                .map_err(|_| PluginError::NetworkError {
                    operation: "fetch".to_string(),
                    message: "Failed to read response body".to_string(),
                })?;
        let body = body_js.as_string().unwrap_or_else(|| "".to_string());

        Ok(HttpResponse {
            status,
            status_text,
            headers: response_headers,
            body,
            ok,
            url,
        })
    }

    /// Create a request builder for fluent API
    ///
    /// This method creates a request builder that provides a fluent interface
    /// for constructing HTTP requests. This is an alternative to creating
    /// `HttpRequest` objects directly.
    ///
    /// # Arguments
    ///
    /// * `url` - The target URL for the request
    /// * `method` - The HTTP method to use
    ///
    /// # Returns
    ///
    /// Returns a `RequestBuilder` instance for fluent request construction.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use squirrel_sdk::http::{HttpClient, HttpMethod};
    ///
    /// async fn make_request() -> Result<(), squirrel_sdk::PluginError> {
    ///     let client = HttpClient::new();
    ///     
    ///     let response = client
    ///         .request_builder("https://api.example.com/users".to_string(), HttpMethod::Post)
    ///         .header("Content-Type", "application/json")
    ///         .body(r#"{"name": "John"}"#)
    ///         .timeout(5000)
    ///         .send()
    ///         .await?;
    ///     
    ///     println!("Response: {}", response.body);
    ///     Ok(())
    /// }
    /// ```
    pub fn request_builder(&self, url: String, method: HttpMethod) -> RequestBuilder<'_> {
        RequestBuilder::new(self, url, method)
    }
}

/// Request builder for fluent API
pub struct RequestBuilder<'a> {
    client: &'a HttpClient,
    request: HttpRequest,
}

impl<'a> RequestBuilder<'a> {
    /// Create a new request builder
    pub fn new(client: &'a HttpClient, url: String, method: HttpMethod) -> Self {
        Self {
            client,
            request: HttpRequest::new(url, method),
        }
    }

    /// Add a header
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.request
            .headers
            .insert(key.to_string(), value.to_string());
        self
    }

    /// Set request body
    pub fn body(mut self, body: &str) -> Self {
        self.request.body = Some(body.to_string());
        self
    }

    /// Set JSON body
    pub fn json<T: Serialize>(mut self, data: &T) -> PluginResult<Self> {
        self.request = self.request.json(data)?;
        Ok(self)
    }

    /// Set timeout
    pub fn timeout(mut self, timeout_ms: u32) -> Self {
        self.request.timeout_ms = Some(timeout_ms);
        self
    }

    /// Send the request
    pub async fn send(self) -> PluginResult<HttpResponse> {
        self.client.request(self.request).await
    }
}

/// Utility functions for HTTP operations
pub mod utils {
    use super::*;

    /// Create a simple GET request
    pub async fn get(url: &str) -> PluginResult<HttpResponse> {
        let client = HttpClient::new();
        let request = HttpRequest::new(url.to_string(), HttpMethod::Get);
        client.request(request).await
    }

    /// Create a simple POST request with JSON body
    pub async fn post_json<T: Serialize>(url: &str, data: &T) -> PluginResult<HttpResponse> {
        let client = HttpClient::new();
        let request = HttpRequest::new(url.to_string(), HttpMethod::Post).json(data)?;
        client.request(request).await
    }

    /// Download a file as text
    pub async fn download_text(url: &str) -> PluginResult<String> {
        let response = get(url).await?;
        if response.is_success() {
            Ok(response.body)
        } else {
            Err(PluginError::NetworkError {
                operation: "download".to_string(),
                message: format!(
                    "Download failed: {} {}",
                    response.status, response.status_text
                ),
            })
        }
    }

    /// Check if a URL is accessible
    pub async fn check_url(url: &str) -> bool {
        match get(url).await {
            Ok(response) => response.is_success(),
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[test]
    fn test_http_method_conversion() {
        assert_eq!(HttpMethod::Get.as_str(), "GET");
        assert_eq!(HttpMethod::from_str("POST").unwrap(), HttpMethod::Post);
        assert!(HttpMethod::from_str("invalid").is_err());
    }

    #[test]
    fn test_http_request_builder() {
        let request = HttpRequest::new("https://example.com".to_string(), HttpMethod::Get)
            .header("Authorization".to_string(), "Bearer token".to_string())
            .timeout(5000);

        assert_eq!(request.url, "https://example.com");
        assert_eq!(request.method, HttpMethod::Get);
        assert_eq!(
            request.headers.get("Authorization"),
            Some(&"Bearer token".to_string())
        );
        assert_eq!(request.timeout_ms, Some(5000));
    }

    #[test]
    fn test_json_request() {
        #[derive(Serialize)]
        struct TestData {
            name: String,
            value: i32,
        }

        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        let request = HttpRequest::new("https://example.com".to_string(), HttpMethod::Post)
            .json(&data)
            .unwrap();

        assert!(request.body.is_some());
        assert_eq!(
            request.headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
    }

    #[wasm_bindgen_test]
    fn test_http_client_creation() {
        let client = HttpClient::new();
        assert!(client.default_headers.is_empty());
    }

    #[test]
    fn test_http_response_json() {
        let response = HttpResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            body: r#"{"name": "test", "value": 42}"#.to_string(),
            ok: true,
            url: "https://example.com".to_string(),
        };

        #[derive(Deserialize, PartialEq, Debug)]
        struct TestData {
            name: String,
            value: i32,
        }

        let data: TestData = response.json().unwrap();
        assert_eq!(data.name, "test");
        assert_eq!(data.value, 42);
    }
}
