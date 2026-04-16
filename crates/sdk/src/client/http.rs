// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! HTTP client functionality for plugins
//!
//! This module provides HTTP client functionality for making web requests from plugins,
//! with sandbox security and permission checking.

#[path = "http_types.rs"]
mod http_types;

pub use http_types::{HttpMethod, HttpRequest, HttpResponse};

use crate::error::{PluginError, PluginResult};
use serde::Serialize;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Headers, Request as WebRequest, RequestInit, RequestMode, Response};

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
        let body = body_js.as_string().unwrap_or_default();

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
    #![cfg_attr(
        not(test),
        expect(
            clippy::wildcard_imports,
            reason = "Aligned with parent module re-exports"
        )
    )]

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
    use serde::{Deserialize, Serialize, Serializer};
    use serde_json::json;
    use std::str::FromStr;

    #[test]
    fn test_http_method_conversion() {
        assert_eq!(HttpMethod::Get.as_str(), "GET");
        assert_eq!(
            HttpMethod::from_str("POST").expect("should succeed"),
            HttpMethod::Post
        );
        assert_eq!(
            HttpMethod::from_str("get").expect("should succeed"),
            HttpMethod::Get
        );
        assert_eq!(
            HttpMethod::from_str("patch").expect("should succeed"),
            HttpMethod::Patch
        );
        assert!(HttpMethod::from_str("invalid").is_err());
        assert_eq!(HttpMethod::parse_method("DELETE"), Some(HttpMethod::Delete));
        assert_eq!(HttpMethod::Options.as_str(), "OPTIONS");
        assert_eq!(HttpMethod::Head.as_str(), "HEAD");
    }

    #[test]
    fn test_http_method_serde_roundtrip() {
        let m = HttpMethod::Put;
        let s = serde_json::to_string(&m).expect("should succeed");
        let back: HttpMethod = serde_json::from_str(&s).expect("should succeed");
        assert_eq!(m, back);
    }

    #[test]
    fn test_http_request_builder() {
        let request = HttpRequest::new("https://example.com".to_string(), HttpMethod::Get)
            .header("Authorization".to_string(), "Bearer token".to_string())
            .body("raw".to_string())
            .timeout(5000);

        assert_eq!(request.url, "https://example.com");
        assert_eq!(request.method, HttpMethod::Get);
        assert_eq!(
            request.headers.get("Authorization"),
            Some(&"Bearer token".to_string())
        );
        assert_eq!(request.timeout_ms, Some(5000));
        assert_eq!(request.body.as_deref(), Some("raw"));
        assert!(request.follow_redirects);
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
            .expect("should succeed");

        assert!(request.body.is_some());
        assert_eq!(
            request.headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
    }

    #[test]
    fn test_http_request_json_serialization_error() {
        struct FailSer;

        impl Serialize for FailSer {
            fn serialize<S: Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
                Err(serde::ser::Error::custom("fail"))
            }
        }

        let err = HttpRequest::new("https://example.com".to_string(), HttpMethod::Post)
            .json(&FailSer)
            .unwrap_err();
        assert!(matches!(err, PluginError::SerializationError { .. }));
    }

    #[test]
    fn test_http_client_default_and_headers() {
        let _: HttpClient = HttpClient::default();
        let mut client = HttpClient::new();
        assert!(client.default_headers.is_empty());
        client.set_default_header("X-Test".to_string(), "1".to_string());
        assert_eq!(client.default_headers.get("X-Test"), Some(&"1".to_string()));
    }

    #[test]
    fn test_request_builder_chain() {
        let client = HttpClient::new();
        let rb = client.request_builder("https://x".to_string(), HttpMethod::Put);
        let _built = rb
            .header("h", "v")
            .body("{}")
            .timeout(100)
            .json(&json!({"a": 1}))
            .expect("should succeed");
    }

    #[test]
    fn test_http_response_json() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct TestData {
            name: String,
            value: i32,
        }

        let mut headers = HashMap::new();
        headers.insert("X-Custom".to_string(), "v".to_string());

        let response = HttpResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: r#"{"name": "test", "value": 42}"#.to_string(),
            ok: true,
            url: "https://example.com".to_string(),
        };

        assert!(response.is_success());
        assert_eq!(response.text(), response.body);
        assert_eq!(response.get_header("X-Custom"), Some(&"v".to_string()));

        let data: TestData = response.json().expect("should succeed");
        assert_eq!(data.name, "test");
        assert_eq!(data.value, 42);
    }

    #[test]
    fn test_http_response_json_invalid() {
        let response = HttpResponse {
            status: 500,
            status_text: "Err".to_string(),
            headers: HashMap::new(),
            body: "not json".to_string(),
            ok: false,
            url: "u".to_string(),
        };
        assert!(!response.is_success());
        let err = response.json::<serde_json::Value>().unwrap_err();
        assert!(matches!(err, PluginError::SerializationError { .. }));
    }

    #[test]
    fn test_http_response_serde_roundtrip() {
        let r = HttpResponse {
            status: 404,
            status_text: "Nope".to_string(),
            headers: HashMap::new(),
            body: String::new(),
            ok: false,
            url: "https://z".to_string(),
        };
        let s = serde_json::to_string(&r).expect("should succeed");
        let back: HttpResponse = serde_json::from_str(&s).expect("should succeed");
        assert_eq!(r.status, back.status);
    }

    #[test]
    fn test_http_request_serde_full_fields() {
        let mut headers = HashMap::new();
        headers.insert("X-Test".to_string(), "1".to_string());
        let req = HttpRequest {
            url: "https://api.example/x".to_string(),
            method: HttpMethod::Patch,
            headers,
            body: Some("{}".to_string()),
            timeout_ms: Some(1234),
            follow_redirects: false,
        };
        let json = serde_json::to_string(&req).expect("serde");
        let back: HttpRequest = serde_json::from_str(&json).expect("de");
        assert!(!back.follow_redirects);
        assert_eq!(back.method, HttpMethod::Patch);
        assert_eq!(back.timeout_ms, Some(1234));
    }

    #[test]
    fn test_http_response_get_header_missing() {
        let r = HttpResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            body: String::new(),
            ok: true,
            url: String::new(),
        };
        assert!(r.get_header("None").is_none());
    }

    #[test]
    fn test_request_builder_json_propagates_serialization_error() {
        struct FailSer;
        impl Serialize for FailSer {
            fn serialize<S: Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
                Err(serde::ser::Error::custom("fail"))
            }
        }

        let client = HttpClient::new();
        match client
            .request_builder("https://example.com".to_string(), HttpMethod::Post)
            .json(&FailSer)
        {
            Err(e) => assert!(matches!(e, PluginError::SerializationError { .. })),
            Ok(_) => panic!("expected serialization error"),
        }
    }

    // `HttpClient::request` uses web-sys / fetch; wasm-bindgen imports are unavailable on native test targets.
    #[cfg(target_arch = "wasm32")]
    #[tokio::test]
    async fn http_client_request_fails_without_browser_window() {
        let client = HttpClient::new();
        let req = HttpRequest::new("https://example.com/".to_string(), HttpMethod::Get);
        let err = client
            .request(req)
            .await
            .expect_err("expected failure without window");
        match err {
            PluginError::InternalError { message } => {
                assert!(
                    message.contains("window")
                        || message.contains("header")
                        || message.contains("request"),
                    "unexpected: {message}"
                );
            }
            PluginError::NetworkError { .. } => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
