//! Authentication for API clients
//!
//! This module provides authentication mechanisms for API clients, including
//! OAuth, token-based, and other authentication methods.

use async_trait::async_trait;
use reqwest::RequestBuilder;
use std::sync::Arc;

use crate::Result;

mod basic;
mod bearer;
mod oauth;

pub use self::basic::BasicAuthenticator;
pub use self::bearer::BearerAuthenticator;
pub use self::oauth::OAuthAuthenticator;

/// Authentication provider for API clients
#[async_trait]
pub trait Authenticator: Send + Sync + 'static {
    /// Add authentication to the request
    async fn authenticate(&self, request: RequestBuilder) -> Result<RequestBuilder>;
}

/// Create a basic auth authenticator with username and password
pub fn basic_auth(username: impl Into<String>, password: impl Into<String>) -> BasicAuthenticator {
    BasicAuthenticator::new(username, password)
}

/// Create a bearer token authenticator
pub fn bearer_auth(token: impl Into<String>) -> BearerAuthenticator {
    BearerAuthenticator::new(token)
}

/// Create a new authenticator for OAuth2 authentication
pub fn oauth2_auth(config: OAuthAuthenticator) -> Arc<dyn Authenticator> {
    Arc::new(config)
} 