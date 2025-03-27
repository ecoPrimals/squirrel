//! Authentication for API clients
//!
//! This module provides authentication mechanisms for API clients, including
//! OAuth, token-based, and other authentication methods.

use async_trait::async_trait;
use reqwest::RequestBuilder;
use secrecy::{Secret, SecretString};
use std::sync::Arc;

mod basic;
mod bearer;
mod oauth;

pub use basic::BasicAuth;
pub use bearer::BearerAuth;
pub use oauth::{OAuth2Auth, OAuth2Config};

/// Authenticator trait for adding authentication to API requests
pub trait Authenticator: Send + Sync + 'static {
    /// Authenticate a request by adding the necessary headers or query parameters
    fn authenticate(&self, request: RequestBuilder) -> RequestBuilder;
}

/// Create a new authenticator for basic authentication
pub fn basic_auth(username: impl Into<String>, password: impl Into<String>) -> Arc<dyn Authenticator> {
    Arc::new(BasicAuth::new(username, password))
}

/// Create a new authenticator for bearer token authentication
pub fn bearer_auth(token: impl Into<String>) -> Arc<dyn Authenticator> {
    Arc::new(BearerAuth::new(token))
}

/// Create a new authenticator for OAuth2 authentication
pub fn oauth2_auth(config: OAuth2Config) -> Arc<dyn Authenticator> {
    Arc::new(OAuth2Auth::new(config))
} 