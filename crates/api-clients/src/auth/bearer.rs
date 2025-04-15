//! Bearer token authentication for API clients

use async_trait::async_trait;
use reqwest::RequestBuilder;
use reqwest::header::AUTHORIZATION;
use secrecy::{ExposeSecret, Secret};

use crate::Result;
use super::Authenticator;

/// Bearer token authentication provider
#[derive(Debug, Clone)]
pub struct BearerAuthenticator {
    token: Secret<String>,
}

impl BearerAuthenticator {
    /// Create a new bearer token authenticator
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: Secret::new(token.into()),
        }
    }
}

#[async_trait]
impl Authenticator for BearerAuthenticator {
    async fn authenticate(&self, request: RequestBuilder) -> Result<RequestBuilder> {
        Ok(request.header(
            AUTHORIZATION,
            format!("Bearer {}", self.token.expose_secret()),
        ))
    }
} 